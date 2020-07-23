#![no_main]
#![no_std]

extern crate panic_halt;

use cortex_m::{iprintln, peripheral};
extern crate stm32f4xx_hal as hal;
use bbqueue::{consts::*, BBBuffer, ConstBBBuffer, Consumer, Producer};
use hal::{
    nb::block,
    prelude::*,
    serial::{config::Config, Event as SerialEvent, Serial},
    stm32,
    stm32::USART3,
    timer::{Event as TimerEvent, Timer},
};

// Create a buffer with 1024 elements
static BB: BBBuffer<U1024> = BBBuffer(ConstBBBuffer::new());

fn itm() -> &'static mut peripheral::itm::Stim {
    unsafe { &mut (*peripheral::ITM::ptr()).stim[0] }
}

#[rtic::app(device = hal::stm32, peripherals = true)]
const APP: () = {
    struct Resources {
        cons: Consumer<'static, U1024>,
        prod: Producer<'static, U1024>,
        tx: hal::serial::Tx<USART3>,
        rx: hal::serial::Rx<USART3>,
        timer: Timer<stm32::TIM2>,
    }

    #[init]
    fn init(cx: init::Context) -> init::LateResources {
        let rcc = cx.device.RCC.constrain();
        let clocks = rcc.cfgr.freeze();

        // Split bbqueue Producer and Consumer
        let (prod, cons) = BB.try_split().unwrap();

        // Set up UART
        let gpioc = cx.device.GPIOC.split();
        let tx = gpioc.pc10.into_alternate_af7();
        let rx = gpioc.pc11.into_alternate_af7();
        let mut serial = Serial::usart3(
            cx.device.USART3,
            (tx, rx),
            Config::default().baudrate(9_600.bps()),
            clocks,
        )
        .unwrap();
        serial.listen(SerialEvent::Rxne);
        // Split TX and RX
        let (tx, rx) = serial.split();

        // Set up 1 Hz Timer
        let mut timer = Timer::tim2(cx.device.TIM2, 1.hz(), clocks);
        timer.listen(TimerEvent::TimeOut);

        // Initialization of late resources
        init::LateResources {
            cons,
            prod,
            tx,
            rx,
            timer,
        }
    }

    // UART interrupt, read from the RX buffer and write to the queue
    #[task(binds = USART3, resources = [prod, rx])]
    fn usart3(cx: usart3::Context) {
        match block!(cx.resources.rx.read()) {
            Ok(byte) => {
                if let Ok(mut wgr) = cx.resources.prod.grant_exact(1) {
                    wgr[0] = byte;
                    wgr.commit(1);
                }
            }
            Err(error) => {
                iprintln!(itm(), "[RX] Err: {:?}", error);
            }
        }
    }

    // Timer interrupt, read the currently available data from the queue and write to the TX buffer
    #[task(binds = TIM2, resources = [timer, cons, tx])]
    fn tim2(cx: tim2::Context) {
        cx.resources.timer.clear_interrupt(TimerEvent::TimeOut);
        let rgr = match cx.resources.cons.read() {
            Ok(it) => it,
            _ => return,
        };
        let len = rgr.len();
        iprintln!(itm(), "{:?}", rgr.buf());
        rgr.buf()
            .iter()
            .for_each(|&byte| match block!(cx.resources.tx.write(byte)) {
                Ok(_) => (),
                Err(error) => {
                    iprintln!(itm(), "[TX] Err: {:?}", error);
                }
            });

        // Release the space for later writes
        rgr.release(len);
    }
};

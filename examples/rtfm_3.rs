#![deny(unsafe_code)]
#![deny(warnings)]
#![no_main]
#![no_std]

use rtfm::cyccnt::U32Ext;
use cortex_m::{iprintln, peripheral};
extern crate panic_halt;
use bbqueue::{consts::*, BBBuffer, ConstBBBuffer, Consumer, Producer};
extern crate stm32f4xx_hal as hal;
use hal::{
    nb::block,
    prelude::*,
    serial::{config::Config, Event, Serial},
    stm32::USART3,
};

static BB: BBBuffer<U1024> = BBBuffer(ConstBBBuffer::new());
const PERIOD: u32 = 16_000_000;

#[rtfm::app(device = hal::stm32, peripherals = true, monotonic = rtfm::cyccnt::CYCCNT)]
const APP: () = {
    struct Resources {
        cons: Consumer<'static, U1024>,
        prod: Producer<'static, U1024>,
        tx: hal::serial::Tx<USART3>,
        rx: hal::serial::Rx<USART3>,
    }

    #[init(schedule = [tx_write])]
    fn init(cx: init::Context) -> init::LateResources {
        // Schedule a task
        cx.schedule
            .tx_write(cx.start + PERIOD.cycles())
            .unwrap();

        // Split bbqueue Producer and Consumer
        let (prod, cons) = BB.try_split().unwrap();

        // Set up USART
        let rcc = cx.device.RCC.constrain();
        let clocks = rcc.cfgr.freeze();
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
        serial.listen(Event::Rxne);
        let (tx, rx) = serial.split();

        // Initialization of late resources
        init::LateResources { cons, prod, tx, rx }
    }

    #[task(binds = USART3, resources = [prod, rx])]
    fn usart3(cx: usart3::Context) {
        match block!(cx.resources.rx.read()) {
            Ok(byte) => {
                if let Ok(mut wgr) = cx.resources.prod.grant_exact(1) {
                    wgr[0] = byte;
                    wgr.commit(1);
                }
            }
            _ => (),
        }
    }

    #[task(schedule = [tx_write], resources = [cons, tx])]
    fn tx_write(cx: tx_write::Context) {
        // Reschedule a task
        cx.schedule
            .tx_write(cx.scheduled + PERIOD.cycles())
            .unwrap();

        let rgr = match cx.resources.cons.read() {
            Ok(it) => it,
            _ => return,
        };

        rgr.buf()
            .iter()
            .for_each(|&byte| block!(cx.resources.tx.write(byte)).unwrap());

        // Release the space for later writes
        let len = rgr.len();
        rgr.release(len);
    }

    // This is required for the software task fn tx_write()
    // This can be any interrupt not used by hardware
    extern "C" {
        fn USART1();
    }
};

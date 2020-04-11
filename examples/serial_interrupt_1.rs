#![no_main]
#![no_std]

extern crate panic_halt;

use core::cell::RefCell;
use core::ops::DerefMut;
use cortex_m;
use cortex_m::interrupt::{free, Mutex};
use cortex_m::{iprintln, peripheral};
use cortex_m_rt::entry;
use stm32f4xx_hal as hal;
use hal::{
    nb::block,
    prelude::*,
    serial::{config::Config, Event, Serial},
    stm32,
    stm32::{interrupt, USART3},
};

static TX: Mutex<RefCell<Option<hal::serial::Tx<USART3>>>> = Mutex::new(RefCell::new(None));
static RX: Mutex<RefCell<Option<hal::serial::Rx<USART3>>>> = Mutex::new(RefCell::new(None));

fn itm() -> &'static mut peripheral::itm::Stim {
    unsafe { &mut (*peripheral::ITM::ptr()).stim[0] }
}

#[interrupt]
fn USART3() {
    free(|cs| {
        if let (Some(ref mut tx), Some(ref mut rx)) = (
            TX.borrow(cs).borrow_mut().deref_mut(),
            RX.borrow(cs).borrow_mut().deref_mut(),
        ) {
            match block!(rx.read()) {
                Ok(byte) => {
                    iprintln!(itm(), "[RX] Ok: {:#04X}", byte);
                    match block!(tx.write(byte)) {
                        Ok(_) => {
                            iprintln!(itm(), "[TX] Ok: {:#04X}", byte);
                        }
                        Err(error) => {
                            iprintln!(itm(), "[TX] Err: {:?}", error);
                        }
                    }
                }
                Err(error) => {
                    iprintln!(itm(), "[RX] Err: {:?}", error);
                }
            }
        }
    });
}

#[entry]
fn main() -> ! {
    // Set up Clocks
    let dp = stm32::Peripherals::take().unwrap();
    let rcc = dp.RCC.constrain();
    let clocks = rcc.cfgr.freeze();

    // Set up UART
    let gpioc = dp.GPIOC.split();
    let tx = gpioc.pc10.into_alternate_af7();
    let rx = gpioc.pc11.into_alternate_af7();
    let mut serial = Serial::usart3(
        dp.USART3,
        (tx, rx),
        Config::default().baudrate(9_600.bps()),
        clocks,
    )
    .unwrap();
    serial.listen(Event::Rxne);

    // Enable interrupt
    stm32::NVIC::unpend(stm32::Interrupt::USART3);
    unsafe {
        stm32::NVIC::unmask(stm32::Interrupt::USART3);
    }

    // Split TX and RX
    let (tx, rx) = serial.split();

    free(|cs| {
        TX.borrow(cs).replace(Some(tx));
        RX.borrow(cs).replace(Some(rx));
    });

    loop {
        continue;
    }
}

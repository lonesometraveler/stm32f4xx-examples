#![no_main]
#![no_std]

extern crate panic_halt;

use cortex_m::iprintln;
use cortex_m_rt::entry;
use stm32f4xx_hal as hal;
use hal::{
    nb::block,
    prelude::*,
    serial::{config::Config, Serial},
    stm32,
};

#[entry]
fn main() -> ! {
    // Set up ITM
    let mut cp = stm32::CorePeripherals::take().unwrap();
    let stim = &mut cp.ITM.stim[0];

    // Set up Clocks
    let dp = stm32::Peripherals::take().unwrap();
    let rcc = dp.RCC.constrain();
    let clocks = rcc.cfgr.freeze();

    // Set up UART
    let gpioc = dp.GPIOC.split();
    let tx = gpioc.pc10.into_alternate_af7();
    let rx = gpioc.pc11.into_alternate_af7();
    let serial = Serial::usart3(
        dp.USART3,
        (tx, rx),
        Config::default().baudrate(9_600.bps()),
        clocks,
    )
    .unwrap();

    // Split TX and RX
    let (mut tx, mut rx) = serial.split();

    let msg = "Hello, I will repeat what you say.\r\n";
    match msg
        .as_bytes()
        .iter()
        .map(|c| block!(tx.write(*c)))
        .collect::<Result<(), _>>()
    {
        Err(error) => iprintln!(stim, "[TX] Err: {:?}", error),
        Ok(_) => {}
    }

    loop {
        match block!(rx.read()) {
            Ok(byte) => {
                iprintln!(stim, "[RX] Ok: {}", byte);
                match block!(tx.write(byte)) {
                    Ok(_) => {
                        iprintln!(stim, "[TX] Ok: {}", byte);
                    }
                    Err(error) => {
                        iprintln!(stim, "[TX] Err: {:?}", error);
                    }
                }
            }
            Err(error) => {
                iprintln!(stim, "[RX] Err: {:?}", error);
            }
        }
    }
}

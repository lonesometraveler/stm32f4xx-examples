#![no_main]
#![no_std]

extern crate panic_halt;

use cortex_m;
use cortex_m::{iprintln, peripheral};
use cortex_m_rt::entry;
use maxsonar::{MaxSonar, Model};
use stm32f4xx_hal::{prelude::*, stm32};

fn itm() -> &'static mut peripheral::itm::Stim {
    unsafe { &mut (*peripheral::ITM::ptr()).stim[0] }
}

#[entry]
fn main() -> ! {
    let dp = stm32::Peripherals::take().unwrap();
    let rcc = dp.RCC.constrain();
    let clocks = rcc.cfgr.sysclk(16.mhz()).freeze();
    let sysclk = clocks.sysclk();

    // Set up the pulse input pin
    let gpioc = dp.GPIOC.split();
    let pin = gpioc.pc10.into_pull_down_input();

    // Set up sonar
    let mut sonar = MaxSonar::new(dp.TIM2, Model::LV, pin, sysclk);

    loop {
        iprintln!(itm(), "{}{}", sonar.read(), sonar.unit());
    }
}

mod maxsonar {
    use stm32f4xx_hal::hal::digital::v2::InputPin;
    use stm32f4xx_hal::stm32::{RCC, TIM2};
    use stm32f4xx_hal::time::Hertz;

    pub struct MaxSonar<T> {
        timer: TIM2,
        model: Model,
        pin: T,
    }

    impl<T, E> MaxSonar<T>
    where
        T: InputPin<Error = E>,
        E: core::fmt::Debug,
    {
        pub fn new(timer: TIM2, model: Model, pin: T, sysclk: Hertz) -> Self {
            // Configure timer for 1Mhz
            let rcc = unsafe { &(*RCC::ptr()) };
            rcc.apb1enr.modify(|_, w| w.tim2en().set_bit());
            let psc = (sysclk.0 / 1_000_000) as u16;
            timer.psc.write(|w| w.psc().bits(psc - 1));
            timer.egr.write(|w| w.ug().set_bit());
            // Start MaxSonar
            let mut sonar = MaxSonar { timer, model, pin };
            sonar.start();
            sonar
        }
        /// Calculates the distance
        pub fn read(&mut self) -> u32 {
            while self.pin.is_low().unwrap() {}
            self.timer.cnt.reset();
            while self.pin.is_high().unwrap() {}
            self.timer.cnt.read().bits() / self.model.factor()
        }
        /// Returns the unit for the model
        pub fn unit(&self) -> &'static str {
            self.model.unit()
        }
        /// Starts the timer
        fn start(&mut self) {
            self.timer.cnt.reset();
            self.timer.cr1.write(|w| w.cen().set_bit());
        }
    }

    /// Maxbotix Ultra Sensor Models
    #[derive(Debug, Clone, Copy)]
    pub enum Model {
        LV,
        XL,
        HR,
    }

    impl Model {
        /// scale factor
        fn factor(self) -> u32 {
            match self {
                Model::LV => 147,
                Model::XL => 58,
                Model::HR => 1,
            }
        }
        /// unit
        fn unit(self) -> &'static str {
            match self {
                Model::LV => "\"",
                Model::XL => "cm",
                Model::HR => "mm",
            }
        }
    }
}

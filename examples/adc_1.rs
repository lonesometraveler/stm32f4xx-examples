#![no_main]
#![no_std]

extern crate panic_halt;

use cortex_m::{iprintln, peripheral};
use cortex_m_rt::entry;
use stm32f4xx_hal::{
    adc::{config::AdcConfig, Adc},
    prelude::*,
    pwm, stm32,
};

fn itm() -> &'static mut peripheral::itm::Stim {
    unsafe { &mut (*peripheral::ITM::ptr()).stim[0] }
}

#[entry]
fn main() -> ! {
    let dp = stm32::Peripherals::take().unwrap();
    // Enable ADC
    let mut adc = Adc::adc1(dp.ADC1, true, AdcConfig::default());
    // Configure ADC pin
    let gpioa = dp.GPIOA.split();
    let mut pa3 = gpioa.pa3.into_analog();

    // Configure PWM
    let rcc = dp.RCC.constrain();
    let clocks = rcc.cfgr.freeze();
    let pa8 = gpioa.pa8.into_alternate_af1();
    let mut pwm = pwm::tim1(dp.TIM1, pa8, clocks, 50.hz());
    let max_duty = pwm.get_max_duty();
    pwm.enable();

    loop {
        let sample = match adc.read(&mut pa3) {
            Ok(x) => x,
            Err(_) => continue,
        };
        iprintln!(itm(), "PA3: {}", sample);
        // Scale 12bit ADC value to the range of 0..=max_duty
        let scale = sample as f32 / 0x0FFF as f32;
        pwm.set_duty((scale * max_duty as f32) as u16);
    }
}

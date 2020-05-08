#![no_main]
#![no_std]

extern crate panic_halt;

use core::cell::RefCell;
use core::ops::DerefMut;
use cortex_m::interrupt::{free, Mutex};
use cortex_m_rt::entry;
use stm32f4xx_hal::{
    adc::{
        config::AdcConfig,
        config::Eoc,
        config::{SampleTime, Sequence},
        Adc,
    },
    prelude::*,
    pwm, stm32,
    stm32::interrupt,
};

static ADC: Mutex<RefCell<Option<Adc<stm32::ADC1>>>> = Mutex::new(RefCell::new(None));
static PWM: Mutex<RefCell<Option<pwm::PwmChannels<stm32::TIM1, pwm::C1>>>> =
    Mutex::new(RefCell::new(None));

#[interrupt]
fn ADC() {
    free(|cs| {
        if let (Some(ref mut adc), Some(ref mut pwm)) = (
            ADC.borrow(cs).borrow_mut().deref_mut(),
            PWM.borrow(cs).borrow_mut().deref_mut(),
        ) {
            // Reading the result from ADC_DR clears the EOC flag automatically.
            let sample = adc.current_sample();
            let scale = sample as f32 / 0x0FFF as f32;
            pwm.set_duty((scale * pwm.get_max_duty() as f32) as u16);
            // restart ADC conversion
            adc.start_conversion();
        }
    });
}

#[entry]
fn main() -> ! {
    let dp = stm32::Peripherals::take().unwrap();
    let rcc = dp.RCC.constrain();
    let clocks = rcc.cfgr.freeze();
    let gpioa = dp.GPIOA.split();

    // Configure PWM
    let pa8 = gpioa.pa8.into_alternate_af1();
    let mut pwm = pwm::tim1(dp.TIM1, pa8, clocks, 50.hz());
    pwm.enable();

    // Configure ADC
    let config = AdcConfig::default().end_of_conversion_interrupt(Eoc::Conversion);
    let mut adc = Adc::adc1(dp.ADC1, true, config);
    let pa3 = gpioa.pa3.into_analog();
    adc.configure_channel(&pa3, Sequence::One, SampleTime::Cycles_112);
    adc.start_conversion();

    // Move shared resources to Mutex
    free(|cs| {
        ADC.borrow(cs).replace(Some(adc));
        PWM.borrow(cs).replace(Some(pwm));
    });

    // Enable interrupt
    unsafe {
        stm32::NVIC::unmask(stm32::interrupt::ADC);
    }

    loop {}
}

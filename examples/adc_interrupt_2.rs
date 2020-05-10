#![no_main]
#![no_std]

extern crate panic_halt;

use core::cell::RefCell;
use core::ops::DerefMut;
use cortex_m::{
    interrupt::{free, Mutex},
    {iprintln, peripheral},
};
use cortex_m_rt::entry;
use stm32f4xx_hal::{
    adc::{
        config::AdcConfig, config::Eoc, config::ExternalTrigger, config::SampleTime,
        config::Sequence, config::TriggerMode, Adc,
    },
    prelude::*,
    pwm, stm32,
    stm32::interrupt,
};

static ADC: Mutex<RefCell<Option<Adc<stm32::ADC1>>>> = Mutex::new(RefCell::new(None));

fn itm() -> &'static mut peripheral::itm::Stim {
    unsafe { &mut (*peripheral::ITM::ptr()).stim[0] }
}

#[interrupt]
fn ADC() {
    free(|cs| {
        if let Some(ref mut adc) = ADC.borrow(cs).borrow_mut().deref_mut() {
            // Reading the result from ADC_DR clears the EOC flag automatically.
            let sample = adc.current_sample();
            iprintln!(itm(), "PA3: {}", sample);
        }
    });
}

#[entry]
fn main() -> ! {
    let dp = stm32::Peripherals::take().unwrap();
    let gpioa = dp.GPIOA.split();
    let rcc = dp.RCC.constrain();
    let clocks = rcc.cfgr.freeze();

    // Configure PWM
    let pa8 = gpioa.pa8.into_alternate_af1();
    let mut pwm = pwm::tim1(dp.TIM1, pa8, clocks, 10.hz());
    let max_duty = pwm.get_max_duty();
    pwm.set_duty(max_duty / 2);
    pwm.enable();

    // Configure ADC
    let config = AdcConfig::default()
        .end_of_conversion_interrupt(Eoc::Conversion)
        .external_trigger(TriggerMode::RisingEdge, ExternalTrigger::Tim_1_cc_1);
    let mut adc = Adc::adc1(dp.ADC1, true, config);
    let pa3 = gpioa.pa3.into_analog();
    adc.configure_channel(&pa3, Sequence::One, SampleTime::Cycles_112);
    adc.enable();

    // Move the shared resource to Mutex
    free(|cs| {
        ADC.borrow(cs).replace(Some(adc));
    });

    // Enable interrupt
    unsafe {
        stm32::NVIC::unmask(stm32::interrupt::ADC);
    }

    loop {}
}

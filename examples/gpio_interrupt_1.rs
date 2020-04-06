#![no_main]
#![no_std]

extern crate panic_halt;

use core::cell::RefCell;
use core::ops::DerefMut;
use cortex_m;
use cortex_m::interrupt::{free, Mutex};
use cortex_m_rt::entry;
use stm32f4xx_hal::{
    gpio::gpiob::PB7,
    gpio::gpioc::PC13,
    gpio::{Edge, ExtiPin, Input, PullDown},
    gpio::{Output, PushPull},
    prelude::*,
    stm32,
    stm32::interrupt,
};

// Global resources
static BUTTON: Mutex<RefCell<Option<PC13<Input<PullDown>>>>> = Mutex::new(RefCell::new(None));
static LED: Mutex<RefCell<Option<PB7<Output<PushPull>>>>> = Mutex::new(RefCell::new(None));

#[entry]
fn main() -> ! {
    let mut dp = stm32::Peripherals::take().unwrap();
    // Enable GPIO Clock
    dp.RCC.apb2enr.write(|w| w.syscfgen().enabled());

    // Set up a LED
    let gpiob = dp.GPIOB.split();
    let led = gpiob.pb7.into_push_pull_output();

    // Set up the user button
    let gpioc = dp.GPIOC.split();
    let mut user_button = gpioc.pc13.into_pull_down_input();
    user_button.make_interrupt_source(&mut dp.SYSCFG);
    user_button.enable_interrupt(&mut dp.EXTI);
    user_button.trigger_on_edge(&mut dp.EXTI, Edge::RISING);

    // Move the shared resources to Mutex
    free(|cs| {
        BUTTON.borrow(cs).replace(Some(user_button));
        LED.borrow(cs).replace(Some(led));
    });

    // Enable interrupt
    stm32::NVIC::unpend(stm32::interrupt::EXTI15_10);
    unsafe {
        stm32::NVIC::unmask(stm32::interrupt::EXTI15_10);
    }

    loop {}
}

// Interrupt Handler
#[interrupt]
fn EXTI15_10() {
    free(|cs| {
        if let Some(ref mut btn) = BUTTON.borrow(cs).borrow_mut().deref_mut() {
            // Clear the interrupt flag
            btn.clear_interrupt_pending_bit();

            if let Some(ref mut led) = LED.borrow(cs).borrow_mut().deref_mut() {
                led.toggle().unwrap();
            }
        }
    });
}

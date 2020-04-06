#![no_main]
#![no_std]

extern crate panic_halt;

use core::cell::RefCell;
use core::ops::DerefMut;
use cortex_m;
use cortex_m::interrupt::{free, Mutex};
use cortex_m_rt::entry;
use stm32f4xx_hal::{
    gpio::gpiob::{PB14, PB7},
    gpio::gpioc::{PC10, PC13},
    gpio::{Edge, ExtiPin, Input, PullDown, PullUp},
    gpio::{Output, PushPull},
    prelude::*,
    stm32,
    stm32::interrupt,
};

// Global resources
static BUTTON: Mutex<RefCell<Option<PC13<Input<PullDown>>>>> = Mutex::new(RefCell::new(None));
static LED: Mutex<RefCell<Option<PB7<Output<PushPull>>>>> = Mutex::new(RefCell::new(None));
static ANOTHER_BUTTON: Mutex<RefCell<Option<PC10<Input<PullUp>>>>> = Mutex::new(RefCell::new(None));
static ANOTHER_LED: Mutex<RefCell<Option<PB14<Output<PushPull>>>>> = Mutex::new(RefCell::new(None));
static EXTI: Mutex<RefCell<Option<stm32::EXTI>>> = Mutex::new(RefCell::new(None));

#[entry]
fn main() -> ! {
    let mut dp = stm32::Peripherals::take().unwrap();
    // Enable GPIO Clock
    dp.RCC.apb2enr.write(|w| w.syscfgen().enabled());

    // Set up LEDs
    let gpiob = dp.GPIOB.split();
    let led = gpiob.pb7.into_push_pull_output();
    let another_led = gpiob.pb14.into_push_pull_output();

    // Set up the user button
    let gpioc = dp.GPIOC.split();
    let mut user_button = gpioc.pc13.into_pull_down_input();
    user_button.make_interrupt_source(&mut dp.SYSCFG);
    user_button.enable_interrupt(&mut dp.EXTI);
    user_button.trigger_on_edge(&mut dp.EXTI, Edge::RISING);

    // Set up another button
    let mut another_button = gpioc.pc10.into_pull_up_input();
    another_button.make_interrupt_source(&mut dp.SYSCFG);
    another_button.enable_interrupt(&mut dp.EXTI);
    another_button.trigger_on_edge(&mut dp.EXTI, Edge::FALLING);

    // External Interrupt controller
    let exti = dp.EXTI;

    // Move the shared resources to Mutex
    free(|cs| {
        BUTTON.borrow(cs).replace(Some(user_button));
        LED.borrow(cs).replace(Some(led));
        ANOTHER_BUTTON.borrow(cs).replace(Some(another_button));
        ANOTHER_LED.borrow(cs).replace(Some(another_led));
        EXTI.borrow(cs).replace(Some(exti));
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
        if let Some(ref mut exti) = EXTI.borrow(cs).borrow_mut().deref_mut() {
            let pr = exti.pr.read();
            // Interrupt on line 13?
            if pr.pr13().bit_is_set() {
                user_button_cb();
            }
            // Interrupt on line 10?
            if pr.pr10().bit_is_set() {
                another_button_cb();
            }
        }
    });
}

fn user_button_cb() {
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

fn another_button_cb() {
    free(|cs| {
        if let Some(ref mut btn) = ANOTHER_BUTTON.borrow(cs).borrow_mut().deref_mut() {
            // Clear the interrupt flag
            btn.clear_interrupt_pending_bit();

            if let Some(ref mut led) = ANOTHER_LED.borrow(cs).borrow_mut().deref_mut() {
                led.toggle().unwrap();
            }
        }
    });
}

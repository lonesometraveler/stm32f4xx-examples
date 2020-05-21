#![no_main]
#![no_std]

extern crate panic_halt;

extern crate stm32f4xx_hal as hal;
use hal::{
    gpio::gpiob::PB7,
    gpio::{Output, PushPull},
    prelude::*,
    stm32,
    timer::{Event, Timer},
};

#[rtfm::app(device = hal::stm32, peripherals = true)]
const APP: () = {
    struct Resources {
        led: PB7<Output<PushPull>>,
        timer: Timer<stm32::TIM2>,
    }

    #[init]
    fn init(cx: init::Context) -> init::LateResources {
        let rcc = cx.device.RCC.constrain();
        let clocks = rcc.cfgr.freeze();

        // Set up LED
        let gpiob = cx.device.GPIOB.split();
        let led = gpiob.pb7.into_push_pull_output();

        // Set up Timer
        let mut timer = Timer::tim2(cx.device.TIM2, 5.hz(), clocks);
        timer.listen(Event::TimeOut);

        // Initialization of late resources
        init::LateResources { led, timer }
    }

    #[task(binds = TIM2, resources = [timer, led])]
    fn tim2(cx: tim2::Context) {
        cx.resources.timer.clear_interrupt(Event::TimeOut);
        cx.resources.led.toggle().unwrap();
    }
};

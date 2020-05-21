# Examples for stm32f4xx_hal

A collection of small examples built with `stm32f4xx_hal`. I also have [another example collection](https://github.com/lonesometraveler/stm32f3xx-examples) for `stm32f3xx_hal`.


## Overview
You will find the follwoings in `examples`:

- `timer_interrupt_1.rs`: Timer interrupt flips a `bool`. A LED gets flipped based on the bool in the main.
- `timer_interrupt_2.rs`: Timer interrupt toggles a LED.
- `gpio_interrupt_1.rs`: GPIO interrupts with one button. 
- `gpio_interrupt_2.rs`: GPIO interrupts with two buttons 1. 
- `gpio_interrupt_3.rs`: GPIO interrupts with two buttons 2. 
- `serial_1.rs`: Serial Echo.
- `serial_interrupt_1.rs`: Serial Echo with interrupt.
- `timer_counter_1.rs`: Pulse width reading with a timer. ([Maxbotix](https://www.maxbotix.com) Ultrasonic sensors demo)
- `adc_1.rs`: ADC reading and PWM output example.
- `adc_interrupt_1.rs`: ADC EOC End of Conversion Interrupt. An interrupt version of `adc_1.rs`.
- `adc_interrupt_2.rs`: ADC External trigger. Injected Conversion Mode.
- `rtfm_1.rs`: [Real Time For the Masses (RTFM) framework](https://github.com/rtfm-rs/cortex-m-rtfm) + [BBQueue](https://github.com/jamesmunns/bbqueue) (SPSC, lockless, no_std, thread safe, queue) example.
- `rtfm_2.rs`: [Real Time For the Masses (RTFM) framework](https://github.com/rtfm-rs/cortex-m-rtfm) example. 

I am planning to add more.

## Usage

**Note** I wrote these for [Nucleo-F429ZI board](https://www.st.com/en/evaluation-tools/nucleo-f429zi.html) which has a STM32F429 microcontroller. If you use a different microcontroller, you need to adjust the settings accordingly.

1. Clone this repo.
``` console
$ git clone https://github.com/lonesometraveler/stm32f4xx-examples.git
```

2. If necessary, set a default target in `.cargo/config` and edit the memory region info in `memory.x`.

3. Build the examples.

``` console
$ cargo build --examples
```

### Cortex Debug

The config file for [Cortex-Debug extension for VS Code](https://marketplace.visualstudio.com/items?itemName=marus25.cortex-debug) is in `.vscode` folder. If your board is Nucleo-F429ZI and you plan to use JLink, it's pretty much ready to go. Just specify an executable in `.vscode/launch.json`.

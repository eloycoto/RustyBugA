#![no_std]
#![allow(non_camel_case_types)]

// reexport hal crates to allow users to directly refer to them
// like in https://github.com/therealprof/nucleo-f103rb/blob/master/src/lib.rs
pub use stm32f1xx_hal as hal;

use hal::pac::*;
use hal::prelude::*;
use hal::serial::*;
use hal::timer::SysDelay;

// These are public just to make the code testeable from the sub-crate.
pub mod engine;
pub mod motor;

pub use crate::hal::*;

pub mod prelude {
    pub use cortex_m_rt::entry;
    pub use stm32f1xx_hal::prelude::{
        _embedded_hal_blocking_delay_DelayMs, _embedded_hal_blocking_delay_DelayUs, _fugit_ExtU32,
    };
}

pub struct UART {
    pub rx: Rx<USART1>,
    pub tx: Tx<USART1>,
}
pub struct Leds {
    pub d1: gpio::Pin<'C', 13, gpio::Output>,
}

pub struct Mightybuga_BSC {
    pub delay: SysDelay,
    pub uart: UART,
    pub leds: Leds,
}

impl Mightybuga_BSC {
    pub fn take() -> Result<Self, ()> {
        let dp = hal::pac::Peripherals::take().ok_or(())?;
        let rcc = dp.RCC.constrain();
        let clocks = rcc.cfgr.freeze(&mut dp.FLASH.constrain().acr);

        let cp = cortex_m::Peripherals::take().unwrap();
        let mut delay = cp.SYST.delay(&clocks);
        delay.delay(300.millis());

        let mut afio = dp.AFIO.constrain();
        let mut gpioa = dp.GPIOA.split();
        let tx = gpioa.pa9.into_alternate_push_pull(&mut gpioa.crh);
        let rx = gpioa.pa10;
        let serial = Serial::new(
            dp.USART1,
            (tx, rx),
            &mut afio.mapr,
            Config::default()
                .baudrate(115_200.bps())
                .wordlength_8bits()
                .parity_none(),
            &clocks,
        );
        let (tx, rx) = serial.split();

        let mut gpioc = dp.GPIOC.split();
        let d1: gpio::Pin<'C', 13, gpio::Output> = gpioc.pc13.into_push_pull_output(&mut gpioc.crh);

        // @TODO:
        // These pins are incorrrect, need to be validated witht the right hardware
        // I guess that a PWM pin should be here to control the motor's ESC
        let motor_left = motor::Motor::new(gpioc.pc10.into_push_pull_output(&mut gpioc.crh));
        let motor_right = motor::Motor::new(gpioc.pc9.into_push_pull_output(&mut gpioc.crh));

        // Engine is the struct which contains all the logics regarding the motors
        let mut engine = engine::Engine::new(motor_left, motor_right);
        // This is to validate that everything is in place
        engine.forward(10);
        engine.backward(100);
        engine.left(10, 5);
        engine.right(10, 5);
        engine.stop();

        Ok(Mightybuga_BSC {
            delay,
            uart: UART { rx, tx },
            leds: Leds { d1 },
        })
    }
}

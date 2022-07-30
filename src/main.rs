#![no_std]
#![no_main]
#![deny(warnings)]

extern crate cortex_m;
extern crate cortex_m_rt as rt;
extern crate panic_halt;
extern crate rtic;

use defmt_rtt as _;

use core::fmt::Write;
use loco::hal::{prelude::*, serial, stm32, timer::*};
use loco::*;

#[rtic::app(device = stm32, peripherals = true)]
mod app {
    use super::*;

    #[shared]
    struct Shared {}

    #[local]
    struct Local {
        led: LedPin,
        uart: UartDev,
        timer: Timer<stm32::TIM2>,
    }

    #[init]
    fn init(ctx: init::Context) -> (Shared, Local, init::Monotonics) {
        defmt::info!("init");

        let mut rcc = ctx.device.RCC.constrain();

        let pins = Pins::new(
            ctx.device.GPIOA,
            ctx.device.GPIOB,
            ctx.device.GPIOC,
            &mut rcc,
        );

        let led = led(pins.g8);

        let uart_cfg = serial::BasicConfig::default().baudrate(115_200.bps());
        let uart = serial(
            ctx.device.USART2,
            pins.uart_tx,
            pins.uart_rx,
            uart_cfg,
            &mut rcc,
        );

        let mut timer = ctx.device.TIM2.timer(&mut rcc);
        timer.start(500.millis());
        timer.listen();

        defmt::info!("init completed");

        (Shared {}, Local { timer, led, uart }, init::Monotonics())
    }

    #[task(binds = TIM2, local = [timer, led, uart])]
    fn timer_tick(ctx: timer_tick::Context) {
        let timer_tick::LocalResources { led, timer, uart } = ctx.local;

        led.toggle().ok();

        if led.is_high().unwrap() {
            write!(uart, "tick\r\n").unwrap();
        } else {
            write!(uart, "tock\r\n").unwrap();
        }

        timer.clear_irq();
    }

    #[idle]
    fn idle(_: idle::Context) -> ! {
        loop {
            cortex_m::asm::nop();
        }
    }
}

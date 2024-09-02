#![no_std]
#![no_main]

use nrf52840_hal as hal;
use hal::pac;
use panic_halt as _;

use hal::{
    pac::Peripherals,
    gpio::{p0::Parts, Level},
    prelude::*,
    Timer,
};
use embedded_hal_0_2::timer::{CountDown, Periodic};

use fugit::RateExtU32;
use fugit::MicrosDurationU32;
use smart_leds::{
    hsv::{hsv2rgb, Hsv},
    SmartLedsWrite, RGB,
};
use ws2812_timer_delay as ws2812;

use smart_leds_fx::colors::HsColor;
use smart_leds_fx::colors::RESTFUL_ORANGE;
use smart_leds_fx::iterators::BrightnessRange;
use cortex_m::asm::delay as cycle_delay;

#[rtic::app(device = pac, dispatchers = [UARTE1])]
mod app {
    use super::*;
    use cortex_m::asm;
    use embedded_hal::digital::{OutputPin, StatefulOutputPin};
    use hal::{
        gpio::{p0::Parts, Level, Output, Pin, PushPull},
        monotonic::MonotonicRtc,
    };
    use pac::RTC0;
    use rtt_target::{rprintln, rtt_init_print};

    const DELAY: fugit::Duration<u32, 1, 32768> =  fugit::Duration::<u32, 1, 32768>::millis(8);
    const LED_COLOR: HsColor<u8> = RESTFUL_ORANGE;
    const NUM_LEDS: usize = 10;

    #[monotonic(binds = RTC0, default = true)]
    type MyMono = MonotonicRtc<RTC0, 32_768>;

    #[shared]
    struct Shared {}

    #[local]
    struct Local {
        neopixel: Pin<Output<PushPull>>,
        brightness_range: BrightnessRange<u8>,
    }

    #[init]
    fn init(cx: init::Context) -> (Shared, Local, init::Monotonics) {
        rtt_init_print!();
        rprintln!("init");

        let brightness_range = BrightnessRange::new(1, 254, 1);

        let port0 = Parts::new(cx.device.P0);
        // .degrade() ?
        let neopixel_pin = port0.p0_13.into_push_pull_output(Level::Low);
        let _power_switch = port0.p0_06.into_push_pull_output(Level::Low);

        let clocks = hal::clocks::Clocks::new(cx.device.CLOCK);
        let clocks = clocks.start_lfclk();
        // Will throw error if freq is invalid
        let mono = MyMono::new(cx.device.RTC0, &clocks).unwrap();
        let mut neopixel = ws2812::Ws2812::new(mono, neopixel_pin);

        blink::spawn().ok();
        (Shared {}, Local { neopixel, brightness_range }, init::Monotonics(mono))
    }

    #[idle]
    fn idle(_: idle::Context) -> ! {
        loop {
            rprintln!("idle");
            // Put core to sleep until next interrupt
            asm::wfe();
        }
    }

    #[task(local = [neopixel, brightness_range])]
    fn blink(ctx: blink::Context) {
        rprintln!("Blink!");
        let neopixel = ctx.local.neopixel;
        let brightness_range = ctx.local.brightness_range;

        let rgb: RGB<u8> = hsv2rgb(Hsv {
            hue: LED_COLOR.hue,
            sat: LED_COLOR.saturation,
            val: brightness_range.next().unwrap(),
        });
        let data = [rgb; NUM_LEDS];
        neopixel.write(data.iter().cloned()).unwrap();

        // spawn after current time + delay
        blink::spawn_after(DELAY).ok();
    }
}

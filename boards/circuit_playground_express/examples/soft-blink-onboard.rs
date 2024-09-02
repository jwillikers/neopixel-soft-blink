#![no_main]
#![no_std]

use panic_halt as _;

use bsp::entry;
use bsp::hal;
use circuit_playground_express as bsp;
use hal::clock::GenericClockController;
use hal::delay::Delay;
use hal::pac::{CorePeripherals, Peripherals};
use hal::prelude::*;
use hal::timer::TimerCounter;
use embedded_hal::spi::SpiDevice;

use fugit::MicrosDurationU32;
use smart_leds_trait::SmartLedsWrite;
use smart_leds::{
    hsv::{hsv2rgb, Hsv},
    SmartLedsWrite, RGB,
};
use ws2812_timer_delay::Ws2812;

use smart_leds_fx::colors::HsColor;
use smart_leds_fx::colors::RESTFUL_ORANGE;
use smart_leds_fx::iterators::BrightnessRange;
// use bsp::prelude::_embedded_hal_blocking_spi_Write;
use embedded_hal_02::blocking::spi::write;
use embedded_hal_02::prelude::*;

#[entry]
fn main() -> ! {
    const DELAY: MicrosDurationU32 =  MicrosDurationU32::millis(8);
    const LED_COLOR: HsColor<u8> = RESTFUL_ORANGE;
    const NUM_LEDS: usize = 10;

    let brightness_range = BrightnessRange::new(1, 254, 1);

    let mut peripherals = Peripherals::take().unwrap();
    let core = CorePeripherals::take().unwrap();
    let mut clocks = GenericClockController::with_internal_32kosc(
        peripherals.GCLK,
        &mut peripherals.PM,
        &mut peripherals.SYSCTRL,
        &mut peripherals.NVMCTRL,
    );
    let pins = bsp::Pins::new(peripherals.PORT);
    let mut delay = Delay::new(core.SYST, &mut clocks);

    let gclk0 = clocks.gclk0();
    let timer_clock = clocks.tcc2_tc3(&gclk0).unwrap();
    let mut timer = TimerCounter::tc3_(&timer_clock, peripherals.TC3, &mut peripherals.PM);
    // timer.start(Hertz::MHz(3).into_duration());
    timer.start(3.mhz());

    // InterruptDrivenTimer::start(&mut timer, Hertz::MHz(3).into_duration());
    // InterruptDrivenTimer::start(&mut timer, 5.millis());
    // nb::block!(InterruptDrivenTimer::wait(&mut timer)).unwrap();


    // let mut ws_data_pin = pins
    //     .d8
    //     .init(&mut clocks, peripherals.TC4, &mut peripherals.MCLK);

    let ws_data_pin: bsp::NeoPixel = pins.d8.into();
    // let ws_data_pin = pins.d8.into_push_pull_output();
    let mut ws = Ws2812::new(timer, ws_data_pin);

    loop {
        for j in brightness_range {
            let rgb: RGB<u8> = hsv2rgb(Hsv {
                hue: LED_COLOR.hue,
                sat: LED_COLOR.saturation,
                val: j,
            });
            let data = [rgb; NUM_LEDS];
            ws.write(data.iter().cloned()).unwrap();
            delay.delay_ms(DELAY.to_millis());
        }
    }
}

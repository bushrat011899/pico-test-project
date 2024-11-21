#![no_std]

use bevy_app::{App, Plugin};
use bevy_ecs::system::Resource;
use bsp::hal::{
    clocks::{init_clocks_and_plls, Clock},
    gpio, pac,
    sio::Sio,
    watchdog::Watchdog,
};
use defmt_rtt as _;
use embedded_hal_bus::spi::ExclusiveDevice;
use epd_waveshare::{epd2in9_v2::*, prelude::*};
use panic_probe as _;
use rp_pico::hal::gpio::bank0::{Gpio10, Gpio11, Gpio12, Gpio13, Gpio8, Gpio9};
use rp_pico::hal::gpio::{FunctionSio, FunctionSpi, Pin, PullDown, SioInput, SioOutput};
use rp_pico::hal::spi::Enabled;
use rp_pico::hal::{Spi, Timer};
use rp_pico::pac::SPI1;
use rp_pico::{self as bsp, hal::fugit::RateExtU32};

pub struct PiPicoDemoPlugin;

impl Plugin for PiPicoDemoPlugin {
    fn build(&self, app: &mut App) {
        let mut pac = pac::Peripherals::take().unwrap();
        let mut watchdog = Watchdog::new(pac.WATCHDOG);
        let sio = Sio::new(pac.SIO);

        let external_xtal_freq_hz = 12_000_000u32;
        let clocks = init_clocks_and_plls(
            external_xtal_freq_hz,
            pac.XOSC,
            pac.CLOCKS,
            pac.PLL_SYS,
            pac.PLL_USB,
            &mut pac.RESETS,
            &mut watchdog,
        )
        .ok()
        .unwrap();

        let mut timer = bsp::hal::Timer::new(pac.TIMER, &mut pac.RESETS, &clocks);

        let pins = bsp::Pins::new(
            pac.IO_BANK0,
            pac.PADS_BANK0,
            sio.gpio_bank0,
            &mut pac.RESETS,
        );

        let dc = pins.gpio8.into_function::<gpio::FunctionSioOutput>();
        let cs = pins.gpio9.into_function::<gpio::FunctionSioOutput>();
        let spi_sclk = pins.gpio10.into_function::<gpio::FunctionSpi>();
        let spi_mosi = pins.gpio11.into_function::<gpio::FunctionSpi>();
        let rst = pins.gpio12.into_function::<gpio::FunctionSioOutput>();
        let busy = pins.gpio13.into_function::<gpio::FunctionSioInput>();

        let spi = bsp::hal::Spi::<_, _, _>::new(pac.SPI1, (spi_mosi, spi_sclk));

        let spi = spi.init(
            &mut pac.RESETS,
            clocks.peripheral_clock.freq(),
            2_500_000u32.Hz(),
            &embedded_hal::spi::MODE_3,
        );

        let mut spi = ExclusiveDevice::new(spi, cs, timer.clone()).unwrap();

        let epd = Epd2in9::new(&mut spi, busy, dc, rst, &mut timer, None).unwrap();

        let display = Display2in9::default();

        app.set_runner(|mut app| loop {
            app.update();

            if let Some(exit) = app.should_exit() {
                return exit;
            }
        })
        .insert_resource(DisplayBuffer(display))
        .insert_non_send_resource(Display { output: epd, spi })
        .insert_non_send_resource(timer);
    }
}

#[derive(Resource)]
pub struct DisplayBuffer(pub Display2in9);

pub struct Display {
    pub output: Epd2in9<
        ExclusiveDevice<
            Spi<
                Enabled,
                SPI1,
                (
                    Pin<Gpio11, FunctionSpi, PullDown>,
                    Pin<Gpio10, FunctionSpi, PullDown>,
                ),
            >,
            Pin<Gpio9, FunctionSio<SioOutput>, PullDown>,
            Timer,
        >,
        Pin<Gpio13, FunctionSio<SioInput>, PullDown>,
        Pin<Gpio8, FunctionSio<SioOutput>, PullDown>,
        Pin<Gpio12, FunctionSio<SioOutput>, PullDown>,
        Timer,
    >,
    pub spi: ExclusiveDevice<
        Spi<
            Enabled,
            SPI1,
            (
                Pin<Gpio11, FunctionSpi, PullDown>,
                Pin<Gpio10, FunctionSpi, PullDown>,
            ),
        >,
        Pin<Gpio9, FunctionSio<SioOutput>, PullDown>,
        Timer,
    >,
}

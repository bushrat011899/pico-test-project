#![no_std]
#![no_main]

extern crate alloc;

use alloc::format;
use bevy_app::{App, Startup, Update};
use bevy_ecs::{
    schedule::IntoSystemConfigs,
    system::{NonSendMut, Res, ResMut, Resource},
};
use defmt_rtt as _;
use embedded_alloc::LlffHeap as Heap;
use embedded_graphics::{
    mono_font::{ascii::FONT_6X10, MonoTextStyle},
    prelude::*,
    text::Text,
};
use embedded_hal::delay::DelayNs;
use epd_waveshare::prelude::*;
use panic_probe as _;
use rp_pico::{entry, hal::Timer};

use pico_example::{Display, DisplayBuffer, PiPicoDemoPlugin};

#[global_allocator]
static HEAP: Heap = Heap::empty();
const HEAP_SIZE: usize = 96 * 1024;

const BEVY: &[u8] = include_bytes!("../assets/bevy_bird_dark.data");

#[derive(Resource, Default)]
pub struct Counter(pub u32);

#[entry]
fn main() -> ! {
    init_heap();

    App::new()
        .add_plugins(PiPicoDemoPlugin)
        .init_resource::<Counter>()
        .add_systems(Startup, clear_display)
        .add_systems(Update, (draw_scene, render, update_counter).chain())
        .run();

    loop {}
}

fn draw_scene(mut buffer: ResMut<DisplayBuffer>, counter: Res<Counter>) {
    let style = MonoTextStyle::new(&FONT_6X10, Color::Black);

    buffer.0.clear(Color::White).unwrap();

    Text::new("Bevy?!", Point::new(10, 30), style)
        .draw(&mut buffer.0)
        .unwrap();

    Text::new("Not a toaster...", Point::new(10, 50), style)
        .draw(&mut buffer.0)
        .unwrap();

    Text::new("...but still small!", Point::new(10, 70), style)
        .draw(&mut buffer.0)
        .unwrap();

    for (index, value) in BEVY.iter().step_by(2).enumerate() {
        let x = index % 64;
        let y = (index - x) / 64;

        if *value == 0 {
            Pixel(Point::new(32 + x as i32, 96 + y as i32), Color::Black)
                .draw(&mut buffer.0)
                .unwrap();
        }
    }

    Text::new("Frames Rendered:", Point::new(10, 200), style)
        .draw(&mut buffer.0)
        .unwrap();

    let message = format!("{}", counter.0);

    Text::new(&message, Point::new(10, 220), style)
        .draw(&mut buffer.0)
        .unwrap();

    Text::new("Heap:", Point::new(10, 240), style)
        .draw(&mut buffer.0)
        .unwrap();

    let message = format!("{} / {} KiB", HEAP.free() / 1024, HEAP_SIZE / 1024);

    Text::new(&message, Point::new(10, 260), style)
        .draw(&mut buffer.0)
        .unwrap();
}

fn clear_display(mut display: NonSendMut<Display>, mut timer: NonSendMut<Timer>) {
    let Display { output, spi } = display.as_mut();
    output.clear_frame(spi, &mut timer).unwrap();
}

fn render(
    mut display: NonSendMut<Display>,
    buffer: Res<DisplayBuffer>,
    mut timer: NonSendMut<Timer>,
) {
    let Display { output, spi } = display.as_mut();
    output
        .update_frame(spi, &buffer.0.buffer(), &mut timer)
        .unwrap();
    output.display_frame(spi, &mut timer).unwrap();
    output.wait_until_idle(spi, &mut timer).unwrap();
    timer.delay_ms(5000);
}

fn update_counter(mut counter: ResMut<Counter>) {
    counter.0 += 1;
}

fn init_heap() {
    use core::mem::MaybeUninit;
    static mut HEAP_MEM: [MaybeUninit<u8>; HEAP_SIZE] = [MaybeUninit::uninit(); HEAP_SIZE];
    unsafe { HEAP.init(HEAP_MEM.as_ptr() as usize, HEAP_SIZE) }
}

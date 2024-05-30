#![no_main]
#![no_std]

mod consts;

extern crate alloc;

use core::fmt::Display;

use alloc::{format, string::ToString};
use embedded_graphics::{
    draw_target::DrawTarget,
    geometry::{OriginDimensions, Point},
    mono_font::MonoTextStyle,
    pixelcolor::{Rgb888, RgbColor},
    text::Text,
    Drawable, Pixel,
};
use log::info;
use profont::PROFONT_18_POINT;
use rustic_mountain_core::Celeste;
use uefi::{
    helpers::system_table,
    prelude::*,
    proto::console::{
        gop::GraphicsOutput,
        text::{Key, ScanCode},
    },
    Char16,
};
use uefi_graphics2::{UefiDisplay, UefiDisplayError};

#[derive(Debug)]
enum UefilesteError {
    Uefi(uefi::Error),
    Display(UefiDisplayError),
}

impl From<uefi::Error> for UefilesteError {
    fn from(value: uefi::Error) -> Self {
        Self::Uefi(value)
    }
}

impl From<UefiDisplayError> for UefilesteError {
    fn from(value: UefiDisplayError) -> Self {
        Self::Display(value)
    }
}

impl Display for UefilesteError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Uefi(err) => err.fmt(f),
            Self::Display(err) => err.fmt(f),
        }
    }
}

fn real_main(mut system: SystemTable<Boot>) -> Result<(), UefilesteError> {
    uefi::helpers::init(&mut system).unwrap();
    // can't do system.stdin() because of https://github.com/rust-osdev/uefi-rs/issues/838
    let mut input_table = system_table();
    let input = input_table.stdin();
    let boot = system.boot_services();

    boot.set_watchdog_timer(0, 0x10000, None)?;

    info!("CELESTE: UEFI");

    let gop_handle = boot.get_handle_for_protocol::<GraphicsOutput>()?;
    let mut gop = boot.open_protocol_exclusive::<GraphicsOutput>(gop_handle)?;
    let mode = gop.current_mode_info();
    let mut display = UefiDisplay::new(gop.frame_buffer(), mode);

    info!("created display...");

    let uefi_string = format!(
        "{:?} (v{})",
        system.firmware_vendor().to_string(),
        system.firmware_revision()
    );

    let palette = &[
        Rgb888::new(0, 0, 0),
        Rgb888::new(29, 43, 83),
        Rgb888::new(126, 37, 83),
        Rgb888::new(0, 135, 81),
        Rgb888::new(171, 82, 54),
        Rgb888::new(95, 87, 79),
        Rgb888::new(194, 195, 199),
        Rgb888::new(255, 241, 232),
        Rgb888::new(255, 0, 77),
        Rgb888::new(255, 163, 0),
        Rgb888::new(255, 236, 85),
        Rgb888::new(0, 228, 54),
        Rgb888::new(41, 173, 255),
        Rgb888::new(131, 118, 156),
        Rgb888::new(255, 119, 168),
        Rgb888::new(255, 204, 170),
    ];

    info!("init...");
    let mut engine = Celeste::new(
        consts::MAPDATA.into(),
        consts::SPRITES.into(),
        consts::FLAGS.into(),
        consts::FONTATLAS.into(),
    );
    info!("inited...");

    let key_z = Char16::try_from('z').unwrap();
    let key_c = Char16::try_from('c').unwrap();
    let key_x = Char16::try_from('x').unwrap();

    let mut timing = [0u8; 4];
    let key_duration = 5;
    let scale = 4;

    let text_style = MonoTextStyle::new(&PROFONT_18_POINT, Rgb888::WHITE);

    let display_size = display.size();
    let celeste_topleft = Point::new(
        display_size.width as i32 / 2 - (64 * scale),
        display_size.height as i32 / 2 - (64 * scale),
    );

    loop {
        info!("ticking...");
        engine.next_tick();
        engine.draw();
        info!("ticked...");

        Text::new(
            &format!("CELESTE: UEFI: {}", uefi_string),
            Point::new(0, 22),
            text_style,
        )
        .draw(&mut display)?;

        for x in 0..scale {
            for y in 0..scale {
                display.draw_iter(engine.mem.graphics.iter().enumerate().map(|(i, col)| {
                    Pixel(
                        Point::new(
                            celeste_topleft.x + ((i as i32 % 128) * scale) + x,
                            celeste_topleft.y + ((i as i32 / 128) * scale) + y,
                        ),
                        palette[*col as usize],
                    )
                }))?;
            }
        }

        display.flush();

        for (i, t) in timing.iter_mut().enumerate() {
            if *t <= 0 {
                engine.mem.buttons[i] = false;
            } else {
                *t -= 1;
            }
        }

        engine.mem.buttons[4] = false;
        engine.mem.buttons[5] = false;
        while let Some(key) = input.read_key()? {
            match key {
                Key::Printable(key) if key == key_z || key == key_c => engine.mem.buttons[4] = true,
                Key::Printable(key) if key == key_x => engine.mem.buttons[5] = true,
                Key::Special(ScanCode::LEFT) => {
                    engine.mem.buttons[0] = true;
                    timing[0] = key_duration;
                }
                Key::Special(ScanCode::RIGHT) => {
                    engine.mem.buttons[1] = true;
                    timing[1] = key_duration;
                }
                Key::Special(ScanCode::UP) => {
                    engine.mem.buttons[2] = true;
                    timing[2] = key_duration;
                }
                Key::Special(ScanCode::DOWN) => {
                    engine.mem.buttons[3] = true;
                    timing[3] = key_duration;
                }
                _ => {}
            }
        }

        boot.stall(33_000);
    }
}

#[entry]
fn main(_image_handle: Handle, mut system_table: SystemTable<Boot>) -> Status {
    system_table.stdout().clear().unwrap();
    match real_main(system_table) {
        Ok(_) => Status::SUCCESS,
        Err(UefilesteError::Uefi(err)) => err.status(),
        Err(UefilesteError::Display(_)) => Status::UNSUPPORTED,
    }
}

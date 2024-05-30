#![no_main]
#![no_std]

mod consts;

extern crate alloc;

use core::fmt::Display;

use alloc::{format, string::ToString};
use embedded_graphics::{
    draw_target::DrawTarget,
    geometry::{OriginDimensions, Point, Size},
    mono_font::MonoTextStyle,
    pixelcolor::{Rgb888, RgbColor},
    primitives::{PrimitiveStyle, PrimitiveStyleBuilder, Rectangle, StyledDrawable},
    text::{renderer::TextRenderer, Baseline, Text},
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
    table::boot::{OpenProtocolAttributes, OpenProtocolParams},
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

fn draw_text(
    display: &mut UefiDisplay,
    text: &str,
    position: Point,
    selected: bool,
    text_style: MonoTextStyle<Rgb888>,
    text_style_selected: MonoTextStyle<Rgb888>,
    selected_background_style: &PrimitiveStyle<Rgb888>,
) -> Result<(), UefilesteError> {
    if selected {
        let text_size = text_style_selected
            .measure_string(text, position, Baseline::Alphabetic)
            .bounding_box;
        Rectangle::new(
            Point::new(text_size.top_left.x - 2, text_size.top_left.y - 2),
            Size::new(text_size.size.width + 4, text_size.size.height + 4),
        )
        .draw_styled(selected_background_style, display)?;
        Text::new(text, position, text_style_selected).draw(display)?;
    } else {
        Text::new(text, position, text_style).draw(display)?;
    }
    Ok(())
}

fn celeste_loop(
    mut display: UefiDisplay,
    key_duration: u8,
    scale: i32,
) -> Result<(), UefilesteError> {
    let mut input_table = system_table();
    let input = input_table.stdin();
    let boot_table = system_table();
    let boot = boot_table.boot_services();

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

    let mut engine = Celeste::new(
        consts::MAPDATA.into(),
        consts::SPRITES.into(),
        consts::FLAGS.into(),
        consts::FONTATLAS.into(),
    );

    let key_z = Char16::try_from('z').unwrap();
    let key_c = Char16::try_from('c').unwrap();
    let key_x = Char16::try_from('x').unwrap();

    let mut timing = [0u8; 4];

    let display_size = display.size();
    let celeste_topleft = Point::new(
        display_size.width as i32 / 2 - (64 * scale),
        display_size.height as i32 / 2 - (64 * scale),
    );

    loop {
        engine.next_tick();
        engine.draw();

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

fn real_main() -> Result<(), UefilesteError> {
    // can't do system.stdin() because of https://github.com/rust-osdev/uefi-rs/issues/838
    let system = system_table();
    let boot = system.boot_services();
    let mut input_table = system_table();
    let input = input_table.stdin();

    boot.set_watchdog_timer(0, 0x10000, None)?;

    info!("CELESTE: UEFI");

    let gop_handle = boot.get_handle_for_protocol::<GraphicsOutput>()?;
    let mut gop = unsafe {
        boot.open_protocol::<GraphicsOutput>(
            OpenProtocolParams {
                handle: gop_handle,
                agent: boot.image_handle(),
                controller: None,
            },
            OpenProtocolAttributes::GetProtocol,
        )?
    };
    let mode = gop.current_mode_info();
    let mut display = UefiDisplay::new(gop.frame_buffer(), mode);

    info!("created display...");

    let max_scale = (display.size().width / 128).min(display.size().height / 128);

    let title_string = format!(
        "CELESTE: UEFI: {:?} ({}x{} px)",
        system.firmware_vendor().to_string(),
        display.size().width,
        display.size().height,
    );

    let text_style = MonoTextStyle::new(&PROFONT_18_POINT, Rgb888::WHITE);
    let text_style_selected = MonoTextStyle::new(&PROFONT_18_POINT, Rgb888::BLACK);
    let bg_style_selected = PrimitiveStyleBuilder::new()
        .fill_color(Rgb888::WHITE)
        .build();

    let mut start_game = false;
    let mut selected: u8 = 0;
    let mut key_duration = 15;
    let mut scale = max_scale / 2;

    let key_enter = Char16::try_from('\r').unwrap();

    while !start_game {
        display.clear(Rgb888::BLACK)?;

        Text::new(&title_string, Point::new(4, 4 + (22 + 4) * 1), text_style).draw(&mut display)?;

        Text::new("LEFT/RIGHT ARROW - CHANGE SETTING", Point::new(4, 4 + (22 + 4) * 2), text_style).draw(&mut display)?;
        Text::new("UP/DOWN ARROW - CHANGE SELECTION", Point::new(4, 4 + (22 + 4) * 3), text_style).draw(&mut display)?;
        Text::new("ENTER - PERFORM ACTION", Point::new(4, 4 + (22 + 4) * 4), text_style).draw(&mut display)?;

        draw_text(
            &mut display,
            &format!("KEY DURATION (FRAMES): {}", key_duration),
            Point::new(4, 4 + (22 + 4) * 5),
            selected == 0,
            text_style,
            text_style_selected,
            &bg_style_selected,
        )?;

        draw_text(
            &mut display,
            &format!("SCALE: {}", scale),
            Point::new(4, 4 + (22 + 4) * 6),
            selected == 1,
            text_style,
            text_style_selected,
            &bg_style_selected,
        )?;

        draw_text(
            &mut display,
            "START GAME",
            Point::new(4, 4 + (22 + 4) * 7),
            selected == 2,
            text_style,
            text_style_selected,
            &bg_style_selected,
        )?;

        display.flush();

        while let Some(key) = input.read_key()? {
            match key {
                Key::Special(ScanCode::LEFT) => {
                    if selected == 0 {
                        key_duration = (key_duration - 1).max(1);
                    } else if selected == 1 {
                        scale = (scale - 1).max(1);
                    }
                }
                Key::Special(ScanCode::RIGHT) => {
                    if selected == 0 {
                        key_duration = (key_duration + 1).min(30);
                    } else if selected == 1 {
                        scale = (scale + 1).min(max_scale);
                    }
                }
                Key::Special(ScanCode::UP) => {
                    if selected == 0 {
                        selected = 2;
                    }
                    selected -= 1;
                }
                Key::Special(ScanCode::DOWN) => {
                    if selected == 2 {
                        selected = 0;
                    }
                    selected += 1;
                }
                Key::Printable(key) if key == key_enter && selected == 2 => {
                    start_game = true;
                }
                _ => {}
            }
        }

        boot.stall(33_000);
    }

    display.clear(Rgb888::BLACK)?;

    celeste_loop(display, key_duration, scale as i32)
}

#[entry]
fn main(_image_handle: Handle, mut system: SystemTable<Boot>) -> Status {
    uefi::helpers::init(&mut system).unwrap();
    system.stdout().clear().unwrap();

    info!("ret: {:?}", real_main());

    system.boot_services().stall(usize::MAX);

    Status::SUCCESS
}

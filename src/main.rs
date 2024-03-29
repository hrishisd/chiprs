extern crate sdl2;
use std::error::Error;
use std::io::ErrorKind;

use chiprs::{Chip8, DisplayState};

use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::render::Canvas;
use sdl2::video::Window;
use sdl2::{audio, event};
use std::env;
use std::time::Duration;

const INSTRUCTIONS_PER_SECOND: u32 = 720;
const INSTRUCTIONS_PER_TIMER_TICK: u32 = 720 / 60;

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();
    let file_path = match &args[..] {
        [_, path] => path,
        _ => {
            eprintln!("USAGE: <script> <program.ch8>");
            std::process::exit(1);
        }
    };
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let window = video_subsystem
        .window("chip-8", 640, 320)
        .position_centered()
        .build()
        .expect("Unable to build sdl2 window");

    // Initialize audio device for beeping
    let audio_subsystem = sdl_context.audio().unwrap();
    let desired_spec = sdl2::audio::AudioSpecDesired {
        freq: Some(44100),
        channels: Some(1), // mono
        samples: None,     // default sample size
    };
    let device = audio_subsystem
        .open_playback(None, &desired_spec, |spec| {
            // initialize the audio callback
            SquareWave {
                phase_inc: 440.0 / spec.freq as f32,
                phase: 0.0,
                volume: 0.25,
            }
        })
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();

    canvas.set_draw_color(Color::BLACK);
    canvas.clear();
    canvas.present();

    let program = match std::fs::read(file_path) {
        Ok(program) => program,
        Err(e) => {
            match e.kind() {
                ErrorKind::NotFound => eprintln!("{file_path} does not exist."),
                ErrorKind::PermissionDenied => eprintln!("no read permissions for {file_path}"),
                _ => eprintln!("Unexpected error: {e}"),
            }
            std::process::exit(1);
        }
    };
    let mut emulator = Chip8::load_program(&program);
    let mut event_pump = sdl_context.event_pump().unwrap();
    let mut inst_count = 0;

    let mut keypresses = [false; 16];
    loop {
        for event in event_pump.poll_iter() {
            match event {
                event::Event::Quit { .. }
                | event::Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => return Ok(()),
                event::Event::KeyDown {
                    keycode: Some(code),
                    ..
                } => {
                    if let Some(chip8_key_code) = key2btn(code) {
                        keypresses[chip8_key_code] = true;
                    }
                }
                event::Event::KeyUp {
                    keycode: Some(key), ..
                } => {
                    if let Some(chip8_key_code) = key2btn(key) {
                        keypresses[chip8_key_code] = false;
                    }
                }
                _ => {}
            };
        }

        let display_state = emulator.step(keypresses);
        if inst_count % INSTRUCTIONS_PER_TIMER_TICK == 0 {
            emulator.tick_timers()
        }
        if emulator.is_sound_on() {
            device.resume(); // Start or continue beeping
        } else {
            device.pause(); // Stop beeping if the sound was previously on
        }
        if display_state == DisplayState::Updated {
            render(&emulator.display, &mut canvas)?;
        }
        inst_count = inst_count.wrapping_add(1);
        std::thread::sleep(Duration::new(0, 1_000_000_000u32 / INSTRUCTIONS_PER_SECOND));
    }
}

fn render(display: &[[bool; 64]; 32], canvas: &mut Canvas<Window>) -> Result<(), Box<dyn Error>> {
    canvas.set_draw_color(Color::BLACK);
    canvas.clear();
    canvas.set_draw_color(Color::WHITE);
    for (y, row) in display.iter().enumerate() {
        for (x, pixel) in row.iter().enumerate() {
            if *pixel {
                let white_box = sdl2::rect::Rect::new(x as i32 * 10, y as i32 * 10, 10, 10);
                canvas.fill_rect(white_box)?;
            }
        }
    }
    canvas.present();
    Ok(())
}

/*
    Keyboard                    Chip-8
    +---+---+---+---+           +---+---+---+---+
    | 1 | 2 | 3 | 4 |           | 1 | 2 | 3 | C |
    +---+---+---+---+           +---+---+---+---+
    | Q | W | E | R |           | 4 | 5 | 6 | D |
    +---+---+---+---+     =>    +---+---+---+---+
    | A | S | D | F |           | 7 | 8 | 9 | E |
    +---+---+---+---+           +---+---+---+---+
    | Z | X | C | V |           | A | 0 | B | F |
    +---+---+---+---+           +---+---+---+---+
*/
fn key2btn(key: Keycode) -> Option<usize> {
    match key {
        Keycode::Num1 => Some(0x1),
        Keycode::Num2 => Some(0x2),
        Keycode::Num3 => Some(0x3),
        Keycode::Num4 => Some(0xC),
        Keycode::Q => Some(0x4),
        Keycode::W => Some(0x5),
        Keycode::E => Some(0x6),
        Keycode::R => Some(0xD),
        Keycode::A => Some(0x7),
        Keycode::S => Some(0x8),
        Keycode::D => Some(0x9),
        Keycode::F => Some(0xE),
        Keycode::Z => Some(0xA),
        Keycode::X => Some(0x0),
        Keycode::C => Some(0xB),
        Keycode::V => Some(0xF),
        _ => None,
    }
}

struct SquareWave {
    phase_inc: f32,
    phase: f32,
    volume: f32,
}

impl audio::AudioCallback for SquareWave {
    type Channel = f32;

    fn callback(&mut self, out: &mut [f32]) {
        // Generate a square wave
        for x in out.iter_mut() {
            *x = if self.phase <= 0.5 {
                self.volume
            } else {
                -self.volume
            };
            self.phase = (self.phase + self.phase_inc) % 1.0;
        }
    }
}

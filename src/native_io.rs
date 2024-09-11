extern crate sdl2;

use crate::IODevice;
use crate::UserInput;

use sdl2::audio::AudioDevice;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::render::Canvas;
use sdl2::video::Window;
use sdl2::{audio, event, EventPump};

use std::error::Error;

pub struct NativeWindow {
    canvas: Canvas<Window>,
    audio_device: AudioDevice<SquareWave>,
    event_pump: EventPump,
    pressed_keys: [bool; 16],
}

impl NativeWindow {
    pub fn initialize() -> NativeWindow {
        let sdl_context = sdl2::init().unwrap();
        let video_subsystem = sdl_context.video().unwrap();
        let window = video_subsystem
            .window("chip-8", 640, 320)
            .position_centered()
            .build()
            .expect("Unable to build sdl2 window");
        let audio_subsystem = sdl_context.audio().unwrap();
        let desired_spec = sdl2::audio::AudioSpecDesired {
            freq: Some(44100),
            channels: Some(1), // mono
            samples: None,     // default sample size
        };
        let audio_device = audio_subsystem
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

        let event_pump = sdl_context.event_pump().unwrap();
        NativeWindow {
            canvas,
            audio_device,
            event_pump,
            pressed_keys: [false; 16],
        }
    }
}

impl IODevice for NativeWindow {
    fn poll_input(&mut self) -> UserInput {
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
        for event in self.event_pump.poll_iter() {
            match event {
                event::Event::Quit { .. }
                | event::Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => {
                    return UserInput::Exit;
                }
                event::Event::KeyDown {
                    keycode: Some(code),
                    ..
                } => {
                    if let Some(chip8_key_code) = key2btn(code) {
                        self.pressed_keys[chip8_key_code] = true;
                    }
                }
                event::Event::KeyUp {
                    keycode: Some(key), ..
                } => {
                    if let Some(chip8_key_code) = key2btn(key) {
                        self.pressed_keys[chip8_key_code] = false;
                    }
                }
                _ => {}
            };
        }
        UserInput::PressedKeys(self.pressed_keys)
    }

    fn pause_beep(&mut self) {
        self.audio_device.pause();
    }

    fn resume_beep(&mut self) {
        self.audio_device.resume();
    }

    fn render(&mut self, display: &[[bool; 64]; 32]) -> Result<(), Box<dyn Error>> {
        self.canvas.set_draw_color(Color::BLACK);
        self.canvas.clear();
        self.canvas.set_draw_color(Color::WHITE);
        for (y, row) in display.iter().enumerate() {
            for (x, pixel) in row.iter().enumerate() {
                if *pixel {
                    let white_box = sdl2::rect::Rect::new(x as i32 * 10, y as i32 * 10, 10, 10);
                    self.canvas.fill_rect(white_box)?;
                }
            }
        }
        self.canvas.present();
        Ok(())
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

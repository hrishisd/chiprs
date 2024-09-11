extern crate sdl2;

mod native_io;

use std::error::Error;
use std::io::ErrorKind;
use std::time::Duration;
use std::{env, process};

use chiprs::{Chip8, DisplayState};
use native_io::NativeWindow;

const INSTRUCTIONS_PER_SECOND: u32 = 720;
const INSTRUCTIONS_PER_TIMER_TICK: u32 = 720 / 60;

trait IODevice {
    /// Returns a bitset of the keys that are currently pressed.
    fn poll_input(&mut self) -> UserInput;
    fn render(&mut self, display: &[[bool; 64]; 32]) -> Result<(), Box<dyn Error>>;
    fn pause_beep(&mut self);
    fn resume_beep(&mut self);
}

enum UserInput {
    PressedKeys([bool; 16]),
    Exit,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();
    let file_path = match &args[..] {
        [_, path] => path,
        _ => {
            eprintln!("USAGE: <script> <program.ch8>");
            std::process::exit(1);
        }
    };

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
    let mut io_device = NativeWindow::initialize();
    let mut emulator = Chip8::load_program(&program);

    let mut inst_count = 0;
    loop {
        let pressed_keys = match io_device.poll_input() {
            UserInput::Exit => {
                process::exit(0);
            }
            UserInput::PressedKeys(pressed_keys) => pressed_keys,
        };
        let display_state = emulator.step(pressed_keys);
        if inst_count % INSTRUCTIONS_PER_TIMER_TICK == 0 {
            emulator.tick_timers()
        }
        if emulator.is_sound_on() {
            io_device.resume_beep(); // Start or continue beeping
        } else {
            io_device.pause_beep(); // Stop beeping if the sound was previously on
        }
        if display_state == DisplayState::Updated {
            io_device.render(&emulator.display)?;
        }
        inst_count = inst_count.wrapping_add(1);
        std::thread::sleep(Duration::new(0, 1_000_000_000u32 / INSTRUCTIONS_PER_SECOND));
    }
}

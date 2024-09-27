extern crate sdl2;

mod native_io;
mod terminal_io;

use std::error::Error;
use std::io::ErrorKind;
use std::path::PathBuf;
use std::str::FromStr;
use std::time::Duration;
use std::time::Instant;

use clap::Parser;

use chiprs::{Chip8, DisplayState};
use native_io::NativeWindow;
use terminal_io::TerminalWindow;

const FRAMES_PER_SECOND: u32 = 120;
const INSTRUCTIONS_PER_FRAME: u32 = 10;

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

#[derive(clap::ValueEnum, Debug, Clone, Copy, PartialEq, Eq)]
/// What frontend to run the emulator with.
enum Frontend {
    /// Run in native window
    Native,
    /// Run in terminal
    Terminal,
}

/// A chip-8 emulator that can run in a native window or directly in the terminal
#[derive(Parser, Debug)]
#[command()]
struct Args {
    /// Path to a .ch8 file
    program: PathBuf,
    #[arg(short, long)]
    frontend: Frontend,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    let program = match std::fs::read(&args.program) {
        Ok(program) => program,
        Err(e) => {
            return Err(match e.kind() {
                ErrorKind::NotFound => format!("{:?} does not exist.", &args.program),
                ErrorKind::PermissionDenied => {
                    format!("no read permissions for {:?}", &args.program)
                }
                _ => format!("{e}"),
            }
            .into());
        }
    };
    let mut io_device: Box<dyn IODevice> = match args.frontend {
        Frontend::Native => Box::new(NativeWindow::initialize()),
        Frontend::Terminal => Box::new(TerminalWindow::initialize()),
    };
    let mut emulator = Chip8::load_program(&program);

    let mut inst_count = 0i64;
    loop {
        let start_time = Instant::now();
        let pressed_keys = match io_device.poll_input() {
            UserInput::Exit => return Ok(()),
            UserInput::PressedKeys(pressed_keys) => pressed_keys,
        };
        let mut display_updated = false;
        for _ in 0..INSTRUCTIONS_PER_FRAME {
            match emulator.step(pressed_keys) {
                DisplayState::Updated => display_updated = true,
                DisplayState::NotUpdated => {}
            };
            inst_count = inst_count.wrapping_add(1);
        }
        if display_updated {
            io_device.render(&emulator.display)?;
        }
        emulator.tick_timers();
        if emulator.is_sound_on() {
            io_device.resume_beep();
        } else {
            io_device.pause_beep();
        }
        let elapsed_time = start_time.elapsed();
        let time_between_frames = Duration::new(0, 1_000_000_000u32 / FRAMES_PER_SECOND);
        if elapsed_time < time_between_frames {
            let sleep_time = time_between_frames - elapsed_time;
            std::thread::sleep(sleep_time);
        }
    }
}

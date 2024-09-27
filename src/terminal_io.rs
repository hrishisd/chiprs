use std::io;
use std::io::Read;
use std::io::Stdout;
use std::io::Write;
use std::time;
use std::time::Duration;
use std::time::Instant;

use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use termion::screen::IntoAlternateScreen;

use crate::IODevice;
use crate::UserInput;

const OFF_COLOR_CODE: i32 = 232;
const ON_COLOR_CODE: i32 = 214;

pub struct TerminalWindow {
    /// The display state is None when uninitialized, before the first display state is received from the emulator
    prev_display_state: Option<[[bool; 64]; 32]>,
    stdout: termion::screen::AlternateScreen<termion::raw::RawTerminal<Stdout>>,
    stdin: termion::AsyncReader,
    last_key_press_times: [Option<time::Instant>; 16],
}

impl TerminalWindow {
    pub fn initialize() -> Self {
        let mut stdout = io::stdout()
            .into_raw_mode()
            .expect("Failed to switch terminal to raw mode")
            .into_alternate_screen()
            .expect("Failed to switch to alternate screen buffer");
        write!(stdout, "{esc}[2J{esc}[1;1H", esc = 27 as char).unwrap();
        stdout.flush().unwrap();
        TerminalWindow {
            prev_display_state: None,
            stdout,
            stdin: termion::async_stdin(),
            last_key_press_times: [None; 16],
        }
    }
}

impl IODevice for TerminalWindow {
    fn poll_input(&mut self) -> UserInput {
        for key in self.stdin.by_ref().keys() {
            match key {
                Ok(Key::Esc) => return UserInput::Exit,
                Ok(Key::Char(c)) => match c {
                    '1' => self.last_key_press_times[0x1] = Some(Instant::now()),
                    '2' => self.last_key_press_times[0x2] = Some(Instant::now()),
                    '3' => self.last_key_press_times[0x3] = Some(Instant::now()),
                    '4' => self.last_key_press_times[0xC] = Some(Instant::now()),
                    'q' => self.last_key_press_times[0x4] = Some(Instant::now()),
                    'w' => self.last_key_press_times[0x5] = Some(Instant::now()),
                    'e' => self.last_key_press_times[0x6] = Some(Instant::now()),
                    'r' => self.last_key_press_times[0xD] = Some(Instant::now()),
                    'a' => self.last_key_press_times[0x7] = Some(Instant::now()),
                    's' => self.last_key_press_times[0x8] = Some(Instant::now()),
                    'd' => self.last_key_press_times[0x9] = Some(Instant::now()),
                    'f' => self.last_key_press_times[0xE] = Some(Instant::now()),
                    'z' => self.last_key_press_times[0xA] = Some(Instant::now()),
                    'x' => self.last_key_press_times[0x0] = Some(Instant::now()),
                    'c' => self.last_key_press_times[0xB] = Some(Instant::now()),
                    'v' => self.last_key_press_times[0xF] = Some(Instant::now()),
                    _ => {}
                },
                Ok(Key::Ctrl('c')) => {
                    // Show the cursor
                    return UserInput::Exit;
                }
                _ => {}
            }
        }

        let now = Instant::now();
        let pressed_keys = self.last_key_press_times.map(|t| match t {
            Some(t) => now - t < Duration::from_millis(50),
            None => false,
        });
        UserInput::PressedKeys(pressed_keys)
    }

    fn render(&mut self, display: &[[bool; 64]; 32]) -> Result<(), Box<dyn std::error::Error>> {
        if self.prev_display_state == Some(*display) {
            return Ok(());
        }

        let display_string = generate_display_string(*display);
        write!(self.stdout, "{display_string}").unwrap();
        self.stdout.flush().unwrap();
        self.prev_display_state = Some(*display);
        Ok(())
    }

    fn pause_beep(&mut self) {}

    fn resume_beep(&mut self) {}
}

impl Drop for TerminalWindow {
    fn drop(&mut self) {
        // Show the cursor
        write!(self.stdout, "\x1b[?25h").unwrap();
        self.stdout.flush().unwrap();
    }
}

// Generate a string, that when printed in raw mode, draws the display to the terminal window
fn generate_display_string(display: [[bool; 64]; 32]) -> String {
    let mut output = String::new();
    // Hide the cursor before rendering
    output.push_str("\x1b[?25l");
    // Move the cursor to the top-left corner of the terminal
    // "\x1b[H" is the escape sequence to move the cursor to (1,1)
    output.push_str("\x1b[H");
    let lower_half_block = '▄';
    let upper_half_block = '▀';
    let full_block = '█';
    assert!(
        display.len() % 2 == 0,
        "Expected an even number of rows in the display, got {}",
        display.len()
    );
    // set the background color
    output.push_str(format!("\x1b[48;5;{}m", OFF_COLOR_CODE).as_str());
    // set the foreground color
    output.push_str(format!("\x1b[38;5;{}m", ON_COLOR_CODE).as_str());
    for row_idx in (0..display.len()).step_by(2) {
        for col_idx in 0..display[0].len() {
            let top_pixel = display[row_idx][col_idx];
            let bottom_pixel = display[row_idx + 1][col_idx];
            if top_pixel && bottom_pixel {
                output.push(full_block)
            } else if top_pixel {
                output.push(upper_half_block);
            } else if bottom_pixel {
                output.push(lower_half_block);
            } else {
                output.push(' ');
            }
        }
        // Need to push a carriage return because \n does not set the cursor position to the beginning of the line in raw mode.
        output.push_str("\r\n");
    }
    output
}

#[test]
fn test_generate_display_string() {
    let mut display = [[false; 64]; 32];
    for row_idx in 0..32 {
        for col_idx in 0..64 {
            display[row_idx][col_idx] = match row_idx % 2 == 0 {
                false => match col_idx % 4 {
                    0 | 3 => true,
                    _ => false,
                },
                true => match col_idx % 4 {
                    1 | 3 => true,
                    _ => false,
                },
            }
        }
    }
    let display_str = generate_display_string(display);
    print!("{display_str}");
}

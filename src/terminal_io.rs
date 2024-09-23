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

use crate::IODevice;
use crate::UserInput;

const WHITE_BLOCK: &str = "\x1b[38;5;214m█\x1b[0m";
const BLACK_BLOCK: &str = "\x1b[38;5;0m█\x1b[0m";

pub struct TerminalWindow {
    /// The display state is None when uninitialized, before the first display state is received from the emulator
    prev_display_state: Option<[[bool; 64]; 32]>,
    stdout: termion::raw::RawTerminal<Stdout>,
    stdin: termion::AsyncReader,
    last_key_press_times: [Option<time::Instant>; 16],
}

impl TerminalWindow {
    pub fn initialize() -> Self {
        let mut stdout = io::stdout()
            .into_raw_mode()
            .expect("Failed to switch terminal to raw mode");
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
        let mut pressed_keys = [false; 16];
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

        let mut output = String::new();
        // Hide the cursor before rendering
        output.push_str("\x1b[?25l");
        // Move the cursor to the top-left corner of the terminal
        // "\x1b[H" is the escape sequence to move the cursor to (1,1)
        output.push_str("\x1b[H");
        // push screen contents
        for row in display {
            for pixel in row {
                output.push_str(if *pixel { WHITE_BLOCK } else { BLACK_BLOCK });
            }
            output.push_str("\r\n");
        }
        write!(self.stdout, "{output}").unwrap();
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

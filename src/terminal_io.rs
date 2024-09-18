use std::io;
use std::io::Read;
use std::io::Stdout;
use std::io::Write;

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
                    '1' => pressed_keys[0x1] = true,
                    '2' => pressed_keys[0x2] = true,
                    '3' => pressed_keys[0x3] = true,
                    '4' => pressed_keys[0xC] = true,
                    'q' => pressed_keys[0x4] = true,
                    'w' => pressed_keys[0x5] = true,
                    'e' => pressed_keys[0x6] = true,
                    'r' => pressed_keys[0xD] = true,
                    'a' => pressed_keys[0x7] = true,
                    's' => pressed_keys[0x8] = true,
                    'd' => pressed_keys[0x9] = true,
                    'f' => pressed_keys[0xE] = true,
                    'z' => pressed_keys[0xA] = true,
                    'x' => pressed_keys[0x0] = true,
                    'c' => pressed_keys[0xB] = true,
                    'v' => pressed_keys[0xF] = true,
                    _ => {}
                },
                Ok(Key::Ctrl('c')) => {
                    // Show the cursor
                    return UserInput::Exit;
                }
                _ => {}
            }
        }

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

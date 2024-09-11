use std::io::stdout;
use std::io::Write;

use crate::IODevice;
use crate::UserInput;

const OFF_CODE: u8 = 0;
const ON_CODE: u8 = 210;

pub struct TerminalWindow {
    /// The display state is None when uninitialized, before the first display state is received from the emulator
    prev_display_state: Option<[[bool; 64]; 32]>,
}

impl TerminalWindow {}

impl IODevice for TerminalWindow {
    fn initialize() -> Self {
        // print!("{esc}c", esc = 27 as char);
        print!("{esc}[2J{esc}[1;1H", esc = 27 as char);
        // io::stdout().flush().unwrap();
        TerminalWindow {
            prev_display_state: None,
        }
    }

    fn poll_input(&mut self) -> UserInput {
        UserInput::PressedKeys([false; 16])
    }

    fn render(&mut self, display: &[[bool; 64]; 32]) -> Result<(), Box<dyn std::error::Error>> {
        match self.prev_display_state {
            Some(prev_display) if prev_display == *display => Ok(()),
            _ => {
                let block = String::from_utf8(vec![0xE2u8, 0x96, 0x88])?;
                let mut screen = String::new();
                for row in display {
                    for pixel in row {
                        if *pixel {
                            screen.push_str(format!("\x1b[38;5;{ON_CODE}m{block}\x1b[0m").as_str());
                        } else {
                            screen
                                .push_str(format!("\x1b[38;5;{OFF_CODE}m{block}\x1b[0m").as_str());
                        }
                    }
                    screen.push('\n');
                }
                let mut lock = stdout().lock();
                // Hide the cursor before rendering
                write!(lock, "\x1b[?25l").unwrap();
                // Move the cursor to the top-left corner of the terminal
                // "\x1b[H" is the escape sequence to move the cursor to (1,1)
                write!(lock, "\x1b[H").unwrap();
                // Render
                write!(lock, "{screen}").unwrap();
                lock.flush().unwrap();
                self.prev_display_state = Some(*display);
                Ok(())
            }
        }
    }

    fn pause_beep(&mut self) {}

    fn resume_beep(&mut self) {}
}

impl Drop for TerminalWindow {
    fn drop(&mut self) {
        print!("\x1b[?25h");
        stdout().flush().unwrap();
    }
}

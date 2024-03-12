use rand::{self, random, seq::index};
use std::panic;
mod font;

pub struct Chip8 {
    /// Program should be loaded into memory starting at 0x200 (512)
    memory: [u8; 4096],
    /// indexed as `[row][col]` or `[y][x]`
    pub display: [[bool; 64]; 32],
    /// points to the current instruction in memory
    /// Only 12 bits are usable
    pc: u16,
    /// also called 'I'
    /// used to point at locations in memory
    index_reg: u16,
    /// Used to call and return from subroutines
    stack: Vec<u16>,
    /// TODO: timer loop decrement is separate from emulation speed
    /// decremented at 60 Hz until it reaches 0
    delay_timer: u8,
    /// decremented like delay timer
    /// gives off a beeping sound while non-zero
    sound_timer: u8,
    /// numbered 0x0 to 0xF or 0 to 15
    /// also called V0 to VF
    /// VF is also used as a flag register
    registers: [u8; 16],
}

impl Chip8 {
    /// Loads a program and returns an emulator instance.
    /// A program consists of 16-bit instructions, but we require bytes.
    pub fn load_program(program: &[u8]) -> Self {
        let mut memory = [0u8; 4096];
        if program.len() > memory.len() - 512 {
            panic!("Program is too large to load into memory");
        }
        // program should be loaded at address 0x200 (512)
        let program_start_addr = 0x200_usize;
        memory[program_start_addr..(program_start_addr + program.len())].copy_from_slice(program);
        // Store fonts at addresses 0x50 to 0x9F
        for (idx, font::Font(bytes)) in font::FONTS.iter().enumerate() {
            let font_start_addr = 0x50 + 5 * idx;
            memory[font_start_addr..(font_start_addr + 5)].copy_from_slice(bytes);
        }

        Chip8 {
            memory,
            display: [[false; 64]; 32],
            pc: program_start_addr as u16,
            index_reg: 0x00,
            stack: Vec::new(),
            delay_timer: 0,
            sound_timer: 0,
            registers: [0u8; 16],
        }
    }

    pub fn step(&mut self, key: Option<char>) {
        // fetch
        let first_byte = self.memory[self.pc as usize];
        let second_byte = self.memory[self.pc as usize + 1];
        let inst = ((first_byte as u16) << 8) | (second_byte as u16);
        self.pc += 2;

        // decode
        let first_half_byte = first_byte >> 4;
        // second half-byte. used to look up one of the 16 registers.
        let X = first_byte as usize & 0x0f;
        // third half-byte. used to look up one of the 16 registers.
        let Y = second_byte as usize >> 4;
        // fourth half-byte
        let N = second_byte & 0x0f;
        // The second byte
        let NN = second_byte;
        // a 12-bit immediate memory address, comprised of half-bytes 2-4
        let NNN = inst & 0x0fff;

        // execute
        match first_half_byte {
            0x0 => {
                match NNN {
                    0x0E0 => {
                        // clear screen
                        self.display = [[false; 64]; 32];
                    }
                    0x0EE => {
                        self.pc = self.stack.pop().expect("Can't return from function call without a return address on the stack.");
                    }
                    _ => panic!("Invalid instruction: {inst:#x}"),
                }
            }
            0x1 => {
                // 1NNN
                // jump
                self.pc = NNN;
            }
            0x2 => {
                // call function at address NNN
                // push the return address onto the stack first
                self.stack.push(self.pc);
                self.pc = NNN;
            }
            0x3 => {
                // conditional skip
                if self.registers[X] == NN {
                    self.pc += 2;
                }
            }
            0x4 => {
                // conditional skip
                if self.registers[X] != NN {
                    self.pc += 2;
                }
            }
            0x5 => {
                // conditional skip
                if self.registers[X] == self.registers[Y] {
                    self.pc += 2;
                }
            }
            0x6 => {
                // 6XNN
                // set register VX
                self.registers[X] = NN
            }
            0x7 => {
                // 7XNN
                // add value to register Vx
                if let Some(sum) = self.registers[X].checked_add(NN) {
                    self.registers[X] = sum;
                }
            }
            0x8 => {
                // logical and arithmetic instructions operating on X and Y
                // behavior toggled by last half-byte
                match N {
                    // set
                    0x0 => self.registers[X] = self.registers[Y],
                    // or
                    0x1 => self.registers[X] |= self.registers[Y],
                    // and
                    0x2 => self.registers[X] &= self.registers[Y],
                    // xor
                    0x3 => self.registers[X] ^= self.registers[Y],
                    // add
                    0x4 => self.registers[X] = self.registers[X].wrapping_add(self.registers[Y]),
                    // subtract
                    0x5 => self.registers[X] = self.registers[X].wrapping_sub(self.registers[Y]),
                    0x7 => self.registers[X] = self.registers[Y].wrapping_sub(self.registers[X]),
                    // shift
                    // (Optional, or configurable) Set VX to the value of VY
                    0x6 => {
                        // set flag register to low bit
                        self.registers[0xF] = self.registers[X] & 0x1;
                        self.registers[X] >>= 1;
                    }
                    0xE => {
                        // set flag register to high bit
                        self.registers[0xF] = self.registers[X] & 0xA0;
                        self.registers[X] <<= 1;
                    }
                    _ => panic!("Invalid instruction: {inst:#x}"),
                }
            }
            0x9 => {
                // conditional skip
                if self.registers[X] != self.registers[Y] {
                    self.pc += 2;
                }
            }
            0xa => {
                // ANNN
                // set index register to NNN
                self.index_reg = NNN;
            }
            0xb => {
                // jump with offset
                self.pc = NNN + self.registers[0] as u16;
            }
            0xc => {
                // random
                self.registers[X] = rand::random::<u8>() & NN;
            }
            0xd => {
                // DXYN
                // draw
                let mut y_coord = self.registers[Y] as usize % 32;
                self.registers[0xF] = 0;
                let bytes =
                    &self.memory[self.index_reg as usize..(self.index_reg as usize + N as usize)];
                for byte in bytes {
                    let mut x_coord = self.registers[X] as usize % 64;
                    for bit_idx in 0..8 {
                        let mask = 0b1000_0000u8 >> bit_idx;
                        let bit = byte & mask > 0;
                        if bit {
                            if self.display[y_coord][x_coord] {
                                self.display[y_coord][x_coord] = false;
                                self.registers[0xF] = 1;
                            } else {
                                self.display[y_coord][x_coord] = true;
                            }
                        }
                        x_coord += 1;
                        if x_coord == 64 {
                            break;
                        }
                    }
                    y_coord += 1;
                    if y_coord == 32 {
                        break;
                    }
                }
            }
            0xe => {
                // skip if key
                todo!("Not implemented");
            }
            0xf => {
                // FX07 sets VX to the current value of the delay timer
                // FX15 sets the delay timer to the value in VX
                // FX18 sets the sound timer to the value in VX
                match NN {
                    // timers
                    0x07 => self.registers[X] = self.delay_timer,
                    0x15 => self.delay_timer = self.registers[X],
                    0x18 => self.sound_timer = self.registers[X],
                    // add to index
                    0x1e => self.index_reg += self.registers[X] as u16,
                    0x0a => panic!("haven't implemented keypress instruction"),
                    // The interpreter takes the decimal value of Vx, and places the hundreds digit in memory at location in I, the tens digit at location I+1, and the ones digit at location I+2.
                    0x33 => {
                        let decimal_val = self.registers[X];
                        let ones = decimal_val % 10;
                        let tens = (decimal_val / 10) % 10;
                        let hundreds = decimal_val / 100;
                        self.memory[self.index_reg as usize] = hundreds;
                        self.memory[self.index_reg as usize + 1] = tens;
                        self.memory[self.index_reg as usize + 2] = ones;
                    }
                    // Store registers V0 through Vx in memory starting at location I.
                    0x55 => {
                        for i in 0..X {
                            self.memory[self.index_reg as usize + i] = self.registers[i]
                        }
                    }
                    // Read registers V0 through Vx from memory starting at location I.
                    0x65 => {
                        for i in 0..X {
                            self.registers[i] = self.memory[self.index_reg as usize + i];
                        }
                    }
                    _ => panic!("Invalid instruction: {inst:#x}"),
                }
            }
            _ => panic!("programming error: unhandled leading half-byte: {inst:#x}"),
        }
    }
}

#[test]
fn test() {}

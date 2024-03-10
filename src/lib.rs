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

    pub fn step(&mut self) {
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
            0x0 => match NNN {
                0x0E0 => {
                    // clear screen
                    self.display = [[false; 64]; 32];
                }
                _ => todo!(),
            },
            0x1 => {
                // 1NNN
                // jump
                self.pc = NNN;
            }
            0x2 => {}
            0x3 => {}
            0x4 => {}
            0x5 => {}
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
            0x8 => {}
            0x9 => {}
            0xa => {
                // ANNN
                // set index register to NNN
                self.index_reg = NNN;
            }
            0xb => {}
            0xc => {}
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
            0xe => {}
            0xf => {}
            _ => panic!("programming error: unhandled leading half-byte"),
        }
    }
}

#[test]
fn test() {}

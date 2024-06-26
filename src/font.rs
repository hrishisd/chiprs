pub struct Font(pub [u8; 5]);

// For some reason, it’s become popular to put it at 050–09F,
// so you can follow that convention if you want
const ZERO: Font = Font([0xF0, 0x90, 0x90, 0x90, 0xF0]);
const ONE: Font = Font([0x20, 0x60, 0x20, 0x20, 0x70]);
const TWO: Font = Font([0xF0, 0x10, 0xF0, 0x80, 0xF0]);
const THREE: Font = Font([0xF0, 0x10, 0xF0, 0x10, 0xF0]);
const FOUR: Font = Font([0x90, 0x90, 0xF0, 0x10, 0x10]);
const FIVE: Font = Font([0xF0, 0x80, 0xF0, 0x10, 0xF0]);
const SIX: Font = Font([0xF0, 0x80, 0xF0, 0x90, 0xF0]);
const SEVEN: Font = Font([0xF0, 0x10, 0x20, 0x40, 0x40]);
const EIGHT: Font = Font([0xF0, 0x90, 0xF0, 0x90, 0xF0]);
const NINE: Font = Font([0xF0, 0x90, 0xF0, 0x10, 0xF0]);
const A: Font = Font([0xF0, 0x90, 0xF0, 0x90, 0x90]);
const B: Font = Font([0xE0, 0x90, 0xE0, 0x90, 0xE0]);
const C: Font = Font([0xF0, 0x80, 0x80, 0x80, 0xF0]);
const D: Font = Font([0xE0, 0x90, 0x90, 0x90, 0xE0]);
const E: Font = Font([0xF0, 0x80, 0xF0, 0x80, 0xF0]);
const F: Font = Font([0xF0, 0x80, 0xF0, 0x80, 0x80]);

pub const FONTS: [Font; 16] = [
    ZERO, ONE, TWO, THREE, FOUR, FIVE, SIX, SEVEN, EIGHT, NINE, A, B, C, D, E, F,
];

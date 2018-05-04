struct Chip8 {
    opcode: u16,         // The current opcode.
    memory: [u8; 4096],  // Chip8 memory, 4k.
    V:      [u8; 16],    // General purpose registers.
    I:      u16,         // Index register.
    pc:     u16,         // Program counter.
    gfx:    [u8; 64*32], // Black and white screen of 2048 pixels.
    delay_timer: u8,
    sound_timer: u8,
    stack:  [u16; 16],   // Stack used to remember location before a jump.
    sp:     u16,         // Stack pointer.
    key:    [u16; 16]
}

fn main() {

}

extern crate sdl2;

use std::io::prelude::*;
use std::fs::File;

const W_BOUNDS: (u32, u32)   =                (640,320); // Window resolution.
const TITLE:    &'static str =                  "Chip8"; // Title to be displayed on the window.
const FILENAME: &'static str = "roms/Chip8 Picture.ch8";

const CHI8_FONTSET: [u8; 80] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
    0x20, 0x60, 0x20, 0x20, 0x70, // 1
    0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
    0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
    0x90, 0x90, 0xF0, 0x10, 0x10, // 4
    0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
    0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
    0xF0, 0x10, 0x20, 0x40, 0x40, // 7
    0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
    0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
    0xF0, 0x90, 0xF0, 0x90, 0x90, // A
    0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
    0xF0, 0x80, 0x80, 0x80, 0xF0, // C
    0xE0, 0x90, 0x90, 0x90, 0xE0, // D
    0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
    0xF0, 0x80, 0xF0, 0x80, 0x80  // F
];

struct Chip8 {
    opcode:      u16,           // The current opcode.
    memory:      [u8; 4096],    // Chip8 memory, 4k.
    v:           [u8; 16],      // General purpose registers.
    i:           u16,           // Index register.
    pc:          u16,           // Program counter.
    gfx:         [u8; 64*32],   // Pixel data.
    delay_timer: u8,
    sound_timer: u8,
    stack:       [u16; 16],     // Stack used to remember location before a jump.
    sp:          u16,           // Stack pointer.
    key:         [u16; 16]
}

fn main() {
    // Initialise Window
    let (mut canvas, mut events) = window_initialise();

    // Initialise chip8
    let mut c8 = chip8_initialise();

    // Load fontset
    chip8_load_fontset(&mut c8);

    // Load game into memory
    chip8_load_game(&mut c8, FILENAME).expect("Could not load file.");

    'a : loop {
        chip8_execute(&mut c8);
        for event in events.poll_iter() {
            match event {
                sdl2::event::Event::Quit{..} => break 'a,
                sdl2::event::Event::KeyDown {keycode: Some(keycode), ..} => {
                    if keycode == sdl2::keyboard::Keycode::Escape {
                        break 'a
                    }
                    else if keycode == sdl2::keyboard::Keycode::Space {
                        chip8_draw(&c8, &mut canvas);
                    }
                }
                _                            => continue
            }
        }
    }
}

/// Initialise a new SDL2 window.
///
/// Initialises a new sdl2 context from which is creates a
/// video context and event pump. From the video context a new
/// window is created and shown, and from the window the canvas
/// is taken. The function then returns the canvas (for later
/// rendering to) and the event pump (to detect key presses).
///
/// # Panics
/// If the window cannot be created from the video context the
/// program will panic.
fn window_initialise() -> (sdl2::render::Canvas<sdl2::video::Window>, sdl2::EventPump) {
    let ctx = sdl2::init().unwrap();
    let video_ctx = ctx.video().unwrap();
    let events = ctx.event_pump().unwrap();

    let mut window = match video_ctx.window(TITLE, W_BOUNDS.0, W_BOUNDS.1).position_centered().opengl().build() {
        Ok(window) => window,
        Err(err) => panic!("Failed to create window: {}", err)
    };

    window.show();
    let canvas = window.into_canvas().build().unwrap();

    (canvas, events)
}

/// Constructs a new 'Chip8' struct.
///
/// Initialises a new 'Chip8' and sets all integer
/// fields and arrays to zero, then returns it.
fn chip8_initialise() -> Chip8 {
    Chip8 {
        opcode:      0,
        memory:      [0_u8; 4096],
        v:           [0_u8; 16],
        i:           0,
        pc:          0x200,
        gfx:         [0_u8; 64*32],
        delay_timer: 0,
        sound_timer: 0,
        stack:       [0_u16; 16],
        sp:          0,
        key:         [0_u16; 16]
    }
}

/// Loads the contents of CHIP8_FONTSET into the first
/// 80 bytes of chip8 memory.
fn chip8_load_fontset(c8: &mut Chip8) {
    for i in 0..CHI8_FONTSET.len() {
        c8.memory[i] = CHI8_FONTSET[i];
    }
}

fn chip8_load_game(c8: &mut Chip8, filename: &str) -> Result<(), std::io::Error> {
    // Total available memory (4096) minus that used by the system (512)
    let mut buffer = [0; 3584];
    let mut file = File::open(filename)?;
    file.read(&mut buffer)?;

    // Load buffer into chip8 memory.
    for i in 0..3584 {
        c8.memory[i + 512] = buffer[i];
    }

    Ok(())
}

fn chip8_execute(c8: &mut Chip8) {
    // Fetch the 16 bit opcode from two sequential 8 bit pc locations, then
    // combine them by shifting the first byte back by 8 bits and ORing
    // by the second byte to combine both.
    c8.opcode = (c8.memory[c8.pc as usize] as u16) << 8 | c8.memory[(c8.pc + 1) as usize] as u16;

    // Decode opcode by removing the first nibble to get operation type.
    match c8.opcode & 0xF000 {
        // Execute opcode.
        0x0000 =>
            match c8.opcode & 0x000F {
                // Clear the screen.
                0x0000 => { for elem in c8.gfx.iter_mut() { *elem = 0; } },
                // Return from subroutine.
                0x000E => {},
                _      => {}
            },
        // Jump to address.
        0x1000 => {  },
        // Call subroutine.
        0x2000 => {  },
        // If Vx == NN skip next instruction.
        0x3000 => {
            if c8.v[((c8.opcode & 0x0F00) >> 8) as usize] == (c8.opcode & 0x00FF) as u8 {
                c8.pc += 2;
            }},
        // If Vx != NN skip next instruction.
        0x4000 => {
            if c8.v[((c8.opcode & 0x0F00) >> 8) as usize] != (c8.opcode & 0x00FF) as u8 {
                c8.pc += 2;
            }},
        // If Vx == Vy skip next instruction.
        0x5000 => {
            if c8.v[((c8.opcode & 0x0F00) >> 8) as usize] != c8.v[((c8.opcode & 0x00F0) >> 4) as usize] {
                c8.pc += 2;
            }},
        // Set Vx == NN
        0x6000 => { c8.v[((c8.opcode & 0x0F00) >> 8) as usize] = (c8.opcode & 0x00FF) as u8;},
        // Add NN to Vx (Carry flag is not changed)
        0x7000 => { c8.v[((c8.opcode & 0x0F00) >> 8) as usize] += (c8.opcode & 0x00FF) as u8;},
        0x8000 =>
            match c8.opcode & 0x000F {
                // Set Vx to Vy
                0x0000 => {
                    c8.v[((c8.opcode & 0x0F00) >> 8) as usize] = c8.v[((c8.opcode & 0x00F0) >> 4) as usize];
                },
                // Set Vx to Vx OR Vy
                0x0001 => {
                    c8.v[((c8.opcode & 0x0F00) >> 8) as usize] =
                    c8.v[((c8.opcode & 0x0F00) >> 8) as usize] |
                    c8.v[((c8.opcode & 0x00F0) >> 4) as usize];
                },
                // Set Vx to Vx AND Vy
                0x0002 => {
                    c8.v[((c8.opcode & 0x0F00) >> 8) as usize] =
                    c8.v[((c8.opcode & 0x0F00) >> 8) as usize] &
                    c8.v[((c8.opcode & 0x00F0) >> 4) as usize];
                },
                // Set Vx to Vx XOR Vy
                0x0003 => {
                    c8.v[((c8.opcode & 0x0F00) >> 8) as usize] =
                    c8.v[((c8.opcode & 0x0F00) >> 8) as usize] ^
                    c8.v[((c8.opcode & 0x00F0) >> 4) as usize];
                },
                // Set Vx to Vx + Vy (Vf is set to 1 on carry)
                0x0004 => {
                    let total: u16 =
                    c8.v[((c8.opcode & 0x0F00) >> 8) as usize] as u16 +
                    c8.v[((c8.opcode & 0x00F0) >> 4) as usize] as u16;

                    c8.v[((c8.opcode & 0x0F00) >> 8) as usize] = total as u8;

                    if total > 255 {
                        c8.v[16] = 1;
                    } else {
                        c8.v[16] = 0;
                    }
                },
                // Set Vx to Vx - Vy (Vf is set to 1 on borrow)
                0x0005 => {
                    let total: u16 =
                    c8.v[((c8.opcode & 0x0F00) >> 8) as usize] as u16 -
                    c8.v[((c8.opcode & 0x00F0) >> 4) as usize] as u16;

                    c8.v[((c8.opcode & 0x0F00) >> 8) as usize] = total as u8;

                    if total <= 0{
                        c8.v[16] = 1;
                    } else {
                        c8.v[16] = 0;
                    }
 
                },
                // Set Vf to least significant bit of Vy and Vx to Vy >> 1
                0x0006 => {
                    c8.v[16] = c8.v[((c8.opcode & 0x00F0) >> 4) as usize] & 0b00000001;
                    c8.v[((c8.opcode & 0x00F0) >> 4) as usize] =
                    c8.v[((c8.opcode & 0x00F0) >> 4) as usize] >> 1;

                    c8.v[((c8.opcode & 0x0F00) >> 8) as usize] =
                    c8.v[((c8.opcode & 0x00F0) >> 4) as usize];
                },
                // Sets Vx to Vy - Vx. Vf is set to 0 on borrow.
                0x0007 => {
                    let total: u16 =
                    c8.v[((c8.opcode & 0x00F0) >> 8) as usize] as u16 -
                    c8.v[((c8.opcode & 0x0F00) >> 8) as usize] as u16;

                    c8.v[((c8.opcode & 0x0F00) >> 8) as usize] = total as u8;

                    if total <= 0{
                        c8.v[16] = 0;
                    } else {
                        c8.v[16] = 1;
                    } 
                },
                // Set Vf to most significant bit of Vy and Vx to Vy << 1
                0x000E => {
                    c8.v[16] = c8.v[((c8.opcode & 0x00F0) >> 4) as usize] & 0b10000000;
                    c8.v[((c8.opcode & 0x00F0) >> 4) as usize] =
                    c8.v[((c8.opcode & 0x00F0) >> 4) as usize] << 1;

                    c8.v[((c8.opcode & 0x0F00) >> 8) as usize] =
                    c8.v[((c8.opcode & 0x00F0) >> 4) as usize];
                },
                _      => {}
            },
        0x9000 => {},
        0xA000 => {},
        0xB000 => {},
        0xC000 => {},
        0xD000 => {},
        0xE000 =>
            match c8.opcode & 0x000F {
                0x000E => {},
                0x0001 => {},
                _      => {}
            },
        0xF000 =>
            match c8.opcode & 0x00FF {
                0x0007 => {},
                0x000A => {},
                0x0015 => {},
                0x0018 => {},
                0x001E => {},
                0x0029 => {},
                0x0033 => {},
                0x0055 => {},
                0x0065 => {},
                _      => {}
            }
        _      => { panic!("Undefined instruction: 0x{:X}", c8.opcode) }
    };

    // Update timers.
    if c8.delay_timer > 0 {
        c8.delay_timer = c8.delay_timer - 1;
    }

    if c8.sound_timer > 0 {
        if c8.sound_timer == 1 {
            println!{"BEEP!"};
        }
        c8.sound_timer = c8.sound_timer - 1;
    }
}

/// Clears the screen and draws the contents of c8.gfx.
fn chip8_draw(c8: &Chip8, canvas: &mut sdl2::render::Canvas<sdl2::video::Window>) {
    canvas.set_draw_color(sdl2::pixels::Color::RGB(0,0,0));
    canvas.clear();
    canvas.set_draw_color(sdl2::pixels::Color::RGB(255,255,255));
    for i in 0..c8.gfx.len() {
        if c8.gfx[i] != 0 {
            let x : i32 = (i as i32 % 64) * (W_BOUNDS.0 as i32 / 64);
            let y : i32 = (i as i32 / 64) * (W_BOUNDS.1 as i32 / 32);
            canvas.fill_rect(sdl2::rect::Rect::new(x,y,W_BOUNDS.0/64,W_BOUNDS.1/32)).expect("Could not draw to screen.");
        }
    }
    canvas.present();
}

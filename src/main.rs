extern crate sdl2;
extern crate rand;

use std::io::prelude::*;
use std::fs::File;
use rand::Rng;
use sdl2::keyboard::Keycode;

const W_BOUNDS: (u32, u32)   =                (640,320); // Window resolution.
const TITLE:    &'static str =                  "Chip8"; // Title to be displayed on the window.
const FILENAME: &'static str = "roms/PONG";

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

const KEYMAP: [Keycode; 16] = [
    Keycode::Num0,
    Keycode::Num1,
    Keycode::Num2,
    Keycode::Num3,
    Keycode::Num4,
    Keycode::Num5,
    Keycode::Num6,
    Keycode::Num7,
    Keycode::Num8,
    Keycode::Num9,
    Keycode::A,
    Keycode::B,
    Keycode::C,
    Keycode::D,
    Keycode::E,
    Keycode::F
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
    key:         [u8; 16],
    draw_flag:   bool
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

        chip8_handle_input(&mut c8, &mut events);
        chip8_fetch(&mut c8);
        chip8_execute(&mut c8);
        if c8.draw_flag {
            chip8_draw(&mut c8, &mut canvas);
        }

        /*
        'b : loop {
            for event in events.poll_iter() {
                match event {
                    sdl2::event::Event::Quit{..} => { std::process::exit(1) },
                    sdl2::event::Event::KeyDown {keycode: Some(keycode), ..} => {
                        if keycode == sdl2::keyboard::Keycode::Space {
                            break 'b;
                        }
                    },
                    _ => { }
                }
            }
        }
        */
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
        key:         [0_u8; 16],
        draw_flag:   false
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
        c8.memory[i + 0x200] = buffer[i];
    }

    Ok(())
}

fn chip8_handle_input(c8: &mut Chip8, events: &mut sdl2::EventPump) {
    c8.key = [0_u8; 16];

    for event in events.poll_iter() {
        match event {
            sdl2::event::Event::Quit{..} => { std::process::exit(1) },
            sdl2::event::Event::KeyDown {keycode: Some(keycode), ..} => {
                if keycode == sdl2::keyboard::Keycode::Escape {
                    std::process::exit(1);
                }

                let pos = KEYMAP.iter().position(|&key| key == keycode);

                match pos {
                    Some(i) => c8.key[i] = 1,
                    None => {}
                }
            },
            _                            => continue
        }
    }
}

/// Fetch the current opcode from c8.memory and set c8.opcode.
///
/// Fetches the 16 bit opcode from two sequential 8 bit locations
/// in memory pointed to by c8.pc, then combines them by shifting
/// the first byte back by 8 bits and ORing by the second byte.
fn chip8_fetch(c8: &mut Chip8) {
    c8.opcode = (c8.memory[c8.pc as usize] as u16) << 8 | c8.memory[(c8.pc + 1) as usize] as u16;
}

fn chip8_execute(c8: &mut Chip8) {

    c8.pc += 2;

    println!{ "pc: {:X}, sp: {:X}, i: {:X}", c8.pc, c8.sp, c8.i};
    println!{ "v: {:?}", c8.v};
    //c8.gfx.iter().for_each(|a| print!{"{:X}", a});
    for i in 0..c8.gfx.len() {
        print!{"{}", c8.gfx[i]};
        if i % 64 == 0 {
            println!{""};
        }
    }
    println!{"Executing opcode: 0x{:X}", c8.opcode};

    let x   : usize = ((c8.opcode & 0x0F00) >> 8) as usize;
    let y   : usize = ((c8.opcode & 0x00F0) >> 4) as usize;
    let n   : usize =  (c8.opcode & 0x000F)       as usize;
    let nn  : usize =  (c8.opcode & 0x00FF)       as usize;
    let nnn : usize =  (c8.opcode & 0x0FFF)       as usize;

    // Decode opcode by removing the first nibble to get operation type.
    match c8.opcode & 0xF000 {
        // Execute opcode.
        0x0000 =>
            match c8.opcode & 0x000F {
                // Clear the screen.
                0x0000 => {
                    for elem in c8.gfx.iter_mut() { *elem = 0; };
                },
                // Return from subroutine.
                0x000E => {
                    c8.sp -= 1;
                    c8.pc = c8.stack[c8.sp as usize];
                },
                _      => { panic!("Undefined instruction: 0x{:X}", c8.opcode) }
            },
        // Jump to address NNN.
        0x1000 => {
            c8.pc = nnn as u16;
        },
        // Call subroutine.
        0x2000 => {
            c8.stack[c8.sp as usize] = c8.pc;
            c8.pc = nnn as u16;
            c8.sp += 1;
        },
        // If Vx == NN skip next instruction.
        0x3000 => {
            if c8.v[x] == nn as u8 {
                println!{"Skipped!"};
                c8.pc += 2;
            }},
        // If Vx != NN skip next instruction.
        0x4000 => {
            if c8.v[x] != nn as u8 {
                c8.pc += 2;
            }},
        // If Vx == Vy skip next instruction.
        0x5000 => {
            if c8.v[x] == c8.v[y] {
                c8.pc += 2;
            }},
        // Set Vx == NN
        0x6000 => {
            c8.v[x] = nn as u8;
        },
        // Add NN to Vx (Carry flag is not changed)
        0x7000 => {
            println!{"c8.v[x]: {:X}, nn: {:X}",
                    (c8.v[x]), (nn)};

            let total: u16 = c8.v[x] as u16 + nn as u16;
            c8.v[x] = total as u8;

            //c8.v[x] += nn as u8;
        },
        0x8000 =>
            match c8.opcode & 0x000F {
                // Set Vx to Vy
                0x0000 => {
                    c8.v[x] = c8.v[y];
                },
                // Set Vx to Vx OR Vy
                0x0001 => {
                    c8.v[x] = c8.v[x] | c8.v[y];
                },
                // Set Vx to Vx AND Vy
                0x0002 => {
                    c8.v[x] = c8.v[x] & c8.v[y];
                },
                // Set Vx to Vx XOR Vy
                0x0003 => {
                    c8.v[x] = c8.v[x] ^ c8.v[y];
                },
                // Set Vx to Vx + Vy (Vf is set to 1 on carry)
                0x0004 => {
                    let total: u16 = c8.v[x] as u16 + c8.v[y] as u16;

                    c8.v[x] = total as u8;

                    if total > 0xFF {
                        c8.v[15] = 1;
                    } else {
                        c8.v[15] = 0;
                    }
                },
                // Set Vx to Vx - Vy (Vf is set to 1 on borrow)
                0x0005 => {
                    let total = c8.v[x] as i8 - c8.v[y] as i8;

                    c8.v[x] = c8.v[x].wrapping_sub(c8.v[y]);

                    if total < 0{
                        c8.v[15] = 1;
                    } else {
                        c8.v[15] = 0;
                    }
                },
                // Set Vf to least significant bit of Vx and shift Vx right
                0x0006 => {
                    c8.v[15] = c8.v[x] & 0x1;
                    c8.v[x] >>= 1;
                },
                // Sets Vx to Vy - Vx. Vf is set to 0 on borrow.
                0x0007 => {
                    let total = c8.v[y] as i8 - c8.v[x] as i8;

                    c8.v[x] = c8.v[y].wrapping_sub(c8.v[x]);

                    if total < 0 {
                        c8.v[15] = 1;
                    } else {
                        c8.v[15] = 0;
                    }
                },
                // Set Vf to most significant bit of Vx and shift Vx left
                0x000E => {
                    c8.v[15] = c8.v[x] & 0x80;
                    c8.v[x] <<= 1;
                },
                _      => { panic!("Undefined instruction: 0x{:X}", c8.opcode) }
            },
        // If Vx != Vy skip next instruction.
        0x9000 => {
            if c8.v[x] != c8.v[y] {
                c8.pc += 2;
        }},
        // Sets I to the address NNN.
        0xA000 => {
            c8.i = nnn as u16;
        },
        // Jumps to the address NNN plus V0.
        0xB000 => {
            c8.pc = nnn as u16 + c8.v[0] as u16;
        },
        // Sets VX to the result a random u8 AND NN
        0xC000 => {
            c8.v[x] =
                rand::thread_rng().gen::<u8>() & nn as u8;
        },
        // Draw a sprite at Vx, Vy, with a width of 8 and height N
        0xD000 => {
            c8.v[15] = 0;

            // For height N
            for h in 0..n {
                for w in 0..8 {
                    // Each byte at memory[i] represents a row of 8 pixels
                    if c8.memory[(c8.i + h as u16) as usize] & (0x80 >> w) != 0 {
                        if c8.gfx[((c8.v[y] as usize + h * 64) + (c8.v[x]+w) as usize) as usize] != 0 {
                            c8.v[15] = 1;
                        }
                        c8.gfx[((c8.v[y] as usize +h * 64) + (c8.v[x]+w) as usize) as usize] ^= 0xFF;
                    }
                }
            }

            c8.draw_flag = true;
        },
        0xE000 =>
            match c8.opcode & 0x000F {
                // Skips the next instruction if the key stored in Vx is pressed.
                0x000E => {
                    if c8.key[c8.v[x] as usize] == 1 {
                        c8.pc += 2;
                }},
                // Skips the next instruction if the key stored in VX is not pressed.
                0x0001 => {
                    if c8.key[c8.v[x] as usize] != 1 {
                        c8.pc +=2;
                }},
                _      => { panic!("Undefined instruction: 0x{:X}", c8.opcode) }
            },
        0xF000 =>
            match c8.opcode & 0x00FF {
                // Set VX to the value of the delay timer.
                0x0007 => {
                    c8.v[x] = c8.delay_timer;
                },
                // A key press is awaited, and then stored in Vx.
                0x000A => {
                    c8.pc -= 2;
                    let pos = c8.key.iter().position(|&key| key == 1);

                    match pos {
                        Some(i) => {
                            c8.v[x] = i as u8;
                            c8.pc +=2;
                        },
                        None => {  }
                    };
                },
                // Set the delay timer to Vx.
                0x0015 => {
                    c8.delay_timer = c8.v[x];
                },
                // Set the sound timer to Vx.
                0x0018 => {
                    c8.sound_timer = c8.v[x];
                },
                // Adds Vx to I.
                0x001E => { 
                    c8.i += c8.v[x] as u16;
                },
                // Set I to the sprite for the character in Vx.
                0x0029 => {
                    c8.i = (c8.v[x] * 5) as u16;
                },
                // Stores the binary-coded decimal representation of Vx, in i to i+2.
                0x0033 => {
                    c8.memory[c8.i as usize]     = c8.v[x] / 100;
                    c8.memory[(c8.i+1) as usize] = (c8.v[x] / 10)  % 10;
                    c8.memory[(c8.i+2) as usize] = (c8.v[x] % 100) % 10;
                },
                // Stores V0 to Vx in memory starting at address i.
                0x0055 => {
                    c8.memory[(c8.i as usize)..(c8.i + x as u16 + 1) as usize]
                        .copy_from_slice(&c8.v[0..(x as usize + 1)]);
                    //for i in 0..(((c8.opcode & 0x0F00) >> 8) + 1)  {
                    //    c8.memory[(c8.i + i) as usize] = c8.v[i as usize];
                    //   c8.i += 1;
                    //}
                },
                // Fills V0 to Vx with values from memory starting at address i.
                0x0065 => {
                    c8.v[0..(x as usize + 1)]
                        .copy_from_slice(&c8.memory[(c8.i as usize)..(c8.i + x as u16 + 1) as usize]);

                    //for i in 0..(((c8.opcode & 0x0F00) >> 8) + 1)  {
                    //    c8.v[i as usize] = c8.memory[(c8.i + i) as usize];
                    //}
                },
                _      => { panic!("Undefined instruction: 0x{:X}", c8.opcode) }
            }
        _      => { panic!("Undefined instruction: 0x{:X}", c8.opcode) }
    }

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

#[test]
fn test_opcode_0x0000() {
    let mut c8 = chip8_initialise();
    c8.opcode = 0x0000;
    c8.gfx = [255; 2048];

    chip8_execute(&mut c8);

    assert!(c8.gfx.iter().zip([0; 2048].iter()).all(|(a,b)| a == b), "c8.gfx not cleared properly.");
    assert_eq!(c8.pc, 514);
}

#[test]
fn test_opcode_0x000e() {
    let mut c8 = chip8_initialise();
    c8.opcode = 0x000E;
    c8.sp = 1;
    c8.stack[0] = 524;

    chip8_execute(&mut c8);

    assert_eq!(c8.pc, 524, "Program counter set to new address.");
    assert_eq!(c8.sp, 0, "Stack pointer decremented.");
}

#[test]
fn test_opcode_0x1000() {
    let mut c8 = chip8_initialise();
    c8.opcode = 0x1A2A;

    chip8_execute(&mut c8);

    assert_eq!(c8.pc, 0x0A2A, "Program counter updated.");
}

#[test]
fn test_opcode_0x2000() {
    let mut c8 = chip8_initialise();
    c8.opcode = 0x2ABC;
    c8.pc = 0x23;

    chip8_execute(&mut c8);

    assert_eq!(c8.pc, 0x0ABC, "Program counter updated to new address.");
    assert_eq!(c8.sp, 1, "Stack poiter incremented.");
    assert_eq!(c8.stack[0], 0x23 + 2, "Stack holds previous address.");
}

#[test]
fn test_opcode_0x3000() {
    let mut c8 = chip8_initialise();
    c8.opcode = 0x31AB;
    c8.v[1] = 0xAB;

    chip8_execute(&mut c8);

    assert_eq!(c8.pc, 516, "Instruction skipped.");

    c8.opcode = 0x31AA;

    chip8_execute(&mut c8);

    assert_eq!(c8.pc, 518, "Program counter incremented.");
}

#[test]
fn test_opcode_0x4000() {
    let mut c8 = chip8_initialise();
    c8.opcode = 0x41AA;
    c8.v[1] = 0xAB;

    chip8_execute(&mut c8);

    assert_eq!(c8.pc, 516, "Instruction skipped.");

    c8.opcode = 0x41AB;

    chip8_execute(&mut c8);

    assert_eq!(c8.pc, 518, "Program counter incremented.");
}

#[test]
fn test_opcode_0x5000() {
    let mut c8 = chip8_initialise();
    c8.opcode = 0x5AB0;
    c8.v[0xA] = 1;
    c8.v[0xB] = 1;

    chip8_execute(&mut c8);

    assert_eq!(c8.pc, 516, "Instruction skipped.");

    c8.v[0xB] = 0;

    chip8_execute(&mut c8);

    assert_eq!(c8.pc, 518, "Program counter incremented.");
}

#[test]
fn test_opcode_0x7000(){
    let mut c8 = chip8_initialise();
    c8.opcode = 0x71FF;
    c8.v[1] = 0xFF;

    chip8_execute(&mut c8);
    assert_eq!{c8.v[1], 0xFF};

    c8.opcode = 0x71FF;
    c8.v[1] = 0;

    chip8_execute(&mut c8);
    assert_eq!{c8.v[1], 0xFF};

    c8.opcode = 0x7100;
    c8.v[1] = 0;

    chip8_execute(&mut c8);
    assert_eq!{c8.v[1], 0};

    c8.opcode = 0x71AB;
    c8.v[1] = 0x11;

    chip8_execute(&mut c8);
    assert_eq!{c8.v[1], 0xBC};
}

fn test_opcode_0x8000(){
    let mut c8 = chip8_initialise();
    c8.opcode = 0x0000;

    chip8_execute(&mut c8);
    //assert_eq!{};
}

fn test_opcode_0x9000(){
    let mut c8 = chip8_initialise();
    c8.opcode = 0x0000;

    chip8_execute(&mut c8);
    //assert_eq!{};
}

#[test]
fn test_opcode_0x8006() {
    let mut c8 = chip8_initialise();
    c8.opcode = 0x8106;
    c8.v[1] = 0b00110000;
    c8.v[15] = 1;

    chip8_execute(&mut c8);
    assert_eq!(c8.v[1], 0b00011000, "Vx shifted 1 bit right.");
    assert_eq!(c8.v[15], 0, "Carry flag set to 0.");

    c8.v[1] = 0b00110001;
    c8.v[15] = 0;
    chip8_execute(&mut c8);
    assert_eq!(c8.v[1], 0b00011000, "Vx shifted 1 bit right.");
    assert_eq!(c8.v[15], 1, "Carry flag set to 1.");
}

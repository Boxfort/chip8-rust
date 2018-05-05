extern crate sdl2;

const W_BOUNDS: (u32, u32)   = (640,320); // Window resolution.
const TITLE:    &'static str =   "Chip8"; // Title to be displayed on the window.

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
    gfx:         [u8; 64*32], // Pixel data.
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
    // Load game into memory

    'a : loop {
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
    let mut events = ctx.event_pump().unwrap();

    let mut window = match video_ctx.window(TITLE, W_BOUNDS.0, W_BOUNDS.1).position_centered().opengl().build() {
        Ok(window) => window,
        Err(err) => panic!("Failed to create window: {}", err)
    };

    window.show();
    let mut canvas = window.into_canvas().build().unwrap();

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

fn chip8_load_game(c8: &mut Chip8, filename: &str) {
    // Load game from file.
}

fn chip8_execute(c8: &mut Chip8) {
    // Fetch the 16 bit opcode from two sequential 8 bit pc locations, then
    // combine them by shifting the first byte back by 8 bits and ORing
    // by the second byte to combine both.
    c8.opcode = (c8.memory[c8.pc as usize] as u16) << 8 | c8.memory[(c8.pc + 1) as usize] as u16;

    // Decode opcode by removing the first nibble to get operation type.
    let decoded = c8.opcode & 0xF000;

    // Execute opcode.
    match decoded {
        0xA00 => {},
        _     => { panic!("Undefined instruction: {}", c8.opcode) }
    };

    // Update timers.
}

fn chip8_draw(c8: &Chip8, canvas: &mut sdl2::render::Canvas<sdl2::video::Window>) {
    canvas.set_draw_color(sdl2::pixels::Color::RGB(0,0,0));
    canvas.clear();
    canvas.set_draw_color(sdl2::pixels::Color::RGB(255,255,255));
    for i in 0..c8.gfx.len() {
        if c8.gfx[i] != 0 {
            let x : i32 = (i as i32 % 64) * (W_BOUNDS.0 as i32 / 64);
            let y : i32 = (i as i32 / 64) * (W_BOUNDS.1 as i32 / 32);
            canvas.fill_rect(sdl2::rect::Rect::new(x,y,W_BOUNDS.0/64,W_BOUNDS.1/32));
        }
    }
    canvas.present();
}

extern crate sdl2;

const DIMESIONS : (u32, u32)   = (600, 400);
const TITLE     : &'static str =    "Chip8";

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
    opcode: u16,         // The current opcode.
    memory: [u8; 4096],  // Chip8 memory, 4k.
    v:      [u8; 16],    // General purpose registers.
    i:      u16,         // Index register.
    pc:     u16,         // Program counter.
    gfx:    [u8; 64*32], // Black and white screen of 2048 pixels.
    delay_timer: u8,
    sound_timer: u8,
    stack:  [u16; 16],   // Stack used to remember location before a jump.
    sp:     u16,         // Stack pointer.
    key:    [u16; 16]
}

fn main() {
    // Initialise graphics
    let (mut canvas, mut events) = window_initialise();
    // Initialise input

    // Initialise chip8
    let mut c8 = chip8_initialise();
    // Load game into memory

    'a : loop {
        window_render(&mut canvas);
        for event in events.poll_iter() {
            match event {
                sdl2::event::Event::Quit{..} => break 'a,
                _                            => continue
            }
        }
    }
}

fn window_initialise() -> (sdl2::render::Canvas<sdl2::video::Window>, sdl2::EventPump) {
    let ctx = sdl2::init().unwrap();
    let video_ctx = ctx.video().unwrap();
    let mut events = ctx.event_pump().unwrap();

    let mut window = match video_ctx.window(TITLE, DIMESIONS.0, DIMESIONS.1).position_centered().opengl().build() {
        Ok(window) => window,
        Err(err) => panic!("Failed to create window: {}", err)
    };

    window.show();
    let mut canvas = window.into_canvas().build().unwrap();

    (canvas, events)
}

fn window_render(canvas: &mut sdl2::render::Canvas<sdl2::video::Window>) {
    canvas.set_draw_color(sdl2::pixels::Color::RGB(255,0,0));
    canvas.clear();
    canvas.present();
}

fn chip8_initialise() -> Chip8 {
    let mut c8 = Chip8 {
        opcode : 0,
        memory : [0_u8; 4096],
        v      : [0_u8; 16],
        i      : 0,
        pc     : 0x200,
        gfx    : [0_u8; 64*32],
        delay_timer: 0,
        sound_timer: 0,
        stack  : [0_u16; 16],
        sp     : 0,
        key    : [0_u16; 16]
    };

    c8
}

fn chip8_execute(c8: &mut Chip8) {
    // Fetch opcode.
    // Decode opcode.
    // Execute opcode.

    // Update timers.
}

fn chip8_draw(c8: &mut Chip8) {
    // Draw to screen.
}

extern crate sdl2;

use sdl::sdl2::pixels::Color;
use sdl::sdl2::event::Event;
use sdl::sdl2::keyboard::Keycode;
use sdl::sdl2::keyboard::Scancode;
use sdl::sdl2::rect::Rect;
use sdl::sdl2::render::WindowCanvas;
use sdl::sdl2::EventPump;
use sdl::sdl2::audio::{AudioCallback, AudioSpecDesired, AudioDevice};
use system::{SCREEN_WIDTH, SCREEN_HEIGHT};

struct SquareWave {
    phase_inc: f32,
    phase: f32,
    volume: f32
}

impl AudioCallback for SquareWave {
    type Channel = f32;

    fn callback(&mut self, out: &mut [f32]) {
        // Generate a square wave
        for x in out.iter_mut() {
            *x = if self.phase <= 0.5 { self.volume } else { -self.volume };
            self.phase = (self.phase + self.phase_inc) % 1.0;
        }
    }
}

// So we don't have to leak the template type
pub struct AudioController {
    device: AudioDevice<SquareWave>,
}

impl AudioController {
    pub fn resume(&self) {
        self.device.resume();
    }

    pub fn pause(&self) {
        self.device.pause();
    }
}

pub fn sdl_init(pixel_size: i32) -> (WindowCanvas, EventPump, AudioController) {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem.window("RustChip8",
                                        (SCREEN_WIDTH as u32)*(pixel_size as u32),
                                        (SCREEN_HEIGHT as u32)*(pixel_size as u32))
        .position_centered()
        .opengl()
        .build()
        .unwrap();

    let canvas = window.into_canvas().build().unwrap();
    let event_pump = sdl_context.event_pump().unwrap();

    let audio_subsystem = sdl_context.audio().unwrap();

    let desired_spec = AudioSpecDesired {
        freq: Some(44_100),
        channels: Some(1),  // mono
        samples: Some(128),
    };

    let device = audio_subsystem.open_playback(None, &desired_spec, |spec| {
        SquareWave {
            phase_inc: 440.0 / spec.freq as f32,
            phase: 0.0,
            volume: 0.25
        }
    }).unwrap();

    (canvas, event_pump, AudioController{device})
}

//Note that these are in the chip8's order, not the PC's keyboard layout
static CHIP8_KEYS : [Scancode; 16] = [
    Scancode::X,    Scancode::Num1, Scancode::Num2, Scancode::Num3,
    Scancode::Q,    Scancode::W,    Scancode::E,    Scancode::A,
    Scancode::S,    Scancode::D,    Scancode::Z,    Scancode::C,
    Scancode::Num4, Scancode::R,    Scancode::F,    Scancode::V,
];

pub fn wait_on_key(event_pump: &mut EventPump) -> usize {
    loop {
        {
            let key_state = event_pump.keyboard_state();
            for (idx, scancode) in CHIP8_KEYS.iter().enumerate() {
                if key_state.is_scancode_pressed(*scancode) {
                    return idx
                }
            }
        } // To force event_pump ref to be dropped here
        
        // Need this here so application still responds while waiting
        // This is ugly but will do for now, allow us to quit during key waits
        if process_events(event_pump) {
            return 16; // aka non existent key
        }
    }
}

pub fn read_keys(event_pump: &EventPump) -> [bool; 16] {
    let mut chip8_keystate: [bool; 16] = [false; 16];
    let key_state = event_pump.keyboard_state();

    for (scancode, chip8key) in CHIP8_KEYS.iter().zip(chip8_keystate.iter_mut()) {
        *chip8key = key_state.is_scancode_pressed(*scancode);
    }

    chip8_keystate
}

pub fn process_events(event_pump: &mut EventPump) -> bool{
    for event in event_pump.poll_iter() {
        match event {
            Event::Quit {..} | Event::KeyDown { keycode: Some(Keycode::Escape), ..} => {
                return true
            }
            _ => {},
        };
    }

    false
}

pub fn draw_screen(pixel_size: i32, canvas: &mut WindowCanvas, screen: &[bool]) {
    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();

    /*
    // Slow path with blur effect
    let pixels = scale_pixels(apply_blur(screen_to_pixels(*screen)), pixel_size);
    
    for (y, row) in pixels.iter().enumerate() {
        for (x, pixel) in row.iter().enumerate() {
            canvas.set_draw_color(Color::RGB(0, *pixel, 0));
            //Since set_pixel doesn't appear to be avaiable
            if let Err(why) = canvas.fill_rect(
                Rect::new(x as i32, y as i32, 1, 1)) {
                panic!("Couldn't draw!: {}", why);
            }
        }
    }
    */
    
    canvas.set_draw_color(Color::RGB(0, 255, 0));
    for (idx, pixel) in screen.iter().enumerate() {
        if *pixel {
            let x = ((idx as i32) % (SCREEN_WIDTH as i32)) * pixel_size;
            let y = ((idx as i32) / (SCREEN_WIDTH as i32)) * pixel_size;

            if let Err(why) = canvas.fill_rect(
                Rect::new(x, y, pixel_size as u32,
                pixel_size as u32)) {
                panic!("couldn't draw to screen!: {}", why);
            }
        }
    }

    canvas.present();
}

#[allow(dead_code)]
fn screen_to_pixels(screen: [bool; SCREEN_WIDTH*SCREEN_HEIGHT]) -> Vec<Vec<u8>> {
    let mut ret: Vec<Vec<u8>> = vec![];
    let mut row: Vec<u8> = vec![];
    for (idx, pixel) in screen.iter().enumerate() {
        if (idx != 0) && ((idx % SCREEN_WIDTH) == 0) {
            ret.push(row.clone());
            row.clear();
        }
        if *pixel {
            row.push(255);
        } else {
            row.push(0);
        }
    }
    ret
}

#[allow(dead_code)]
fn apply_blur(pixels: &[Vec<u8>]) -> Vec<Vec<u8>> {
    let strength = 20; // Amount of blur
    let mut new_pixels = pixels.to_owned();
    for (y, row) in pixels.iter().enumerate() {
        for (x, v) in row.iter().enumerate() {
            // Each lit pixel will bleed some light to the surrounding pixels
            if *v == 255 {
                // Apply bleed to surrounding pixels
                let co_ords: Vec<(usize, usize)> = vec![
                    (y.saturating_sub(1), x),
                    (y+1,                 x),
                    (y,                   x.saturating_sub(1)),
                    (y,                   x+1),
                    (y.saturating_sub(1), x.saturating_sub(1)),
                    (y.saturating_sub(1), x+1),
                    (y+1,                 x.saturating_sub(1)),
                    (y+1,                 x+1),
                ];

                for (y, x) in co_ords {
                    if (x < pixels[0].len()) &&
                       (y < pixels.len()) {
                        new_pixels[y][x] = new_pixels[y][x].saturating_add(strength);
                    }
                }
            }
        }
    }
    new_pixels
}

#[allow(dead_code)]
fn scale_pixels(pixels: &[Vec<u8>], scaling_factor: i32) -> Vec<Vec<u8>> {
    let mut new_pixels: Vec<Vec<u8>> = vec![];
    for row in pixels.iter() {
        let mut new_row: Vec<u8> = vec![];
        for pixel in row.iter() {
            for _ in 0..scaling_factor {
                new_row.push(*pixel);
            }
        }
        for _ in 0..scaling_factor {
            new_pixels.push(new_row.clone());
        }
    }
    new_pixels
}
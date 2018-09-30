extern crate sdl2;

use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::keyboard::Scancode;
//use std::time::Duration;
use sdl2::rect::Rect;

mod system;

pub fn main() {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let pixel_size: i32 = 10;

    let window = video_subsystem.window("RChip8",
                                        64*(pixel_size as u32),
                                        32*(pixel_size as u32))
        .position_centered()
        .opengl()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();
    let mut event_pump = sdl_context.event_pump().unwrap();

    let rom = String::from("INVADERS");
    let mut c8 = system::make_system(&rom);

    /*TODO: Hammer the instruction encodings!
    for opcode in 0..0xFFFF {
    }*/

    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} | Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'running
                },
                Event::KeyDown { keycode: Some(Keycode::Tab), ..} => {
                    c8.screen_to_file();
                },
                _ => {}
            }
        }
        //TODO: frame limit? Some kind of 'virtual clock speed'?
        //::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));

        //Note that these are in the chip8's order, not the PC's keyboard layout
        let chip8_keys = [
           Scancode::X,    Scancode::Num1, Scancode::Num2, Scancode::Num3,
           Scancode::Q,    Scancode::W,    Scancode::E,    Scancode::A,
           Scancode::S,    Scancode::D,    Scancode::Z,    Scancode::C,
           Scancode::Num4, Scancode::R,    Scancode::F,    Scancode::V,
        ];
 
        let instr = c8.fetch_and_decode();
        let flags = instr.get_flags();
        match flags {
            system::InstrFlags::Keys => {
                let mut chip8_keystate: [bool; 16] = [false; 16];
                let key_state = event_pump.keyboard_state();

                for (scancode, chip8key) in chip8_keys.iter().zip(chip8_keystate.iter_mut()) {
                    *chip8key = key_state.is_scancode_pressed(*scancode);
                }

                c8.update_keys(chip8_keystate);
            }
            system::InstrFlags::WaitKey => {
                'polling : loop {
                    {
                        let key_state = event_pump.keyboard_state();
                        for (idx, scancode) in chip8_keys.iter().enumerate() {
                            if key_state.is_scancode_pressed(*scancode) {
                                c8.pressed_key = idx;
                                break 'polling
                            }
                        }
                    } // To force event_pump ref to be dropped here
                   
                    // Need this here so application still responds while waiting
                    // TODO: de-dupe
                    for event in event_pump.poll_iter() {
                        match event {
                            Event::Quit {..} | Event::KeyDown { keycode: Some(Keycode::Escape), ..} => {
                                break 'running
                            }
                            _ => {},
                        };
                    }
                }
            }
            _ => {},
        }

        c8.execute(&instr);

        match flags {
            system::InstrFlags::Screen => {
                canvas.set_draw_color(Color::RGB(0, 0, 0));
                canvas.clear();

                canvas.set_draw_color(Color::RGB(0, 255, 0));
                for (idx, pixel) in c8.screen.iter().enumerate() {
                    if *pixel {
                        let x = ((idx as i32) % 64) * pixel_size;
                        let y = ((idx as i32) / 64) * pixel_size;

                        if let Err(why) = canvas.fill_rect(
                            Rect::new(x, y, pixel_size as u32,
                            pixel_size as u32)) {
                            panic!("couldn't draw to screen!: {}", why);
                        }
                    }
                }

                canvas.present();
            }
            _ => {},
        }
    }
}

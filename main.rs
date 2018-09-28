extern crate sdl2;

use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use std::time::Duration;
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

    let mut c8 = system::make_system(String::from("INVADERS"));

    /*TODO: Hammer the instruction encodings!
    for opcode in 0..0xFFFF {
    }*/

    let mut instr_count = 0;

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
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
 
        // TODO: only do this when the instruction that needs it runs
        c8.update_keys(event_pump.keyboard_state());
        c8.do_opcode();
        
        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.clear();

        canvas.set_draw_color(Color::RGB(0, 255, 0));
        for (idx, pixel) in c8.screen.iter().enumerate() {
            if *pixel {
                let x = ((idx as i32) % 64) * pixel_size;
                let y = ((idx as i32) / 64) * pixel_size;
                canvas.fill_rect(Rect::new(x, y, pixel_size as u32, pixel_size as u32));
            }
        }

        canvas.present();

        instr_count += 1;
        if (instr_count == 200) {
            c8.screen_to_file();
            break 'running
        }
    }
}

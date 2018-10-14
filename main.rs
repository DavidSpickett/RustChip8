mod system;
mod sdl;
use system::{make_system, read_rom};
use sdl::{sdl_init, process_events, draw_screen, read_keys, wait_on_key};
use std::{env, process};
use std::path::Path;

pub fn main() {
    let help = "\
            ./rchip8 <rom path> <scaling factor>\n\
            \n\
            ROM path is relative to the exe. (required)\n\
            Scaling factor multiplies the size of each Chip8 pixel. (default 1)\n\
            e.g. 2 means each block is 2x2 pixels in the final output.";

    let mut rom_path: Option<String> = None;
    let mut scaling_factor = 1;

    let args = env::args();
    if args.len() < 2 {
        println!("ROM file path is required.");
        process::exit(1);
    }
    for (pos, argument) in args.enumerate() {
        if argument == "-h" {
            println!("{}", help);
            process::exit(0);
        }

        match pos {
            0 => {},
            1 => {
                if Path::new(&argument).exists() {
                    rom_path = Some(argument);
                } else {
                    println!("ROM file \"{}\" not found.", argument);
                    process::exit(1);
                }
            },
            2 => {
                scaling_factor = match argument.parse::<i32>() {
                    Err(msg) => {
                        println!("Invalid scaling factor \"{}\": {}", argument, msg);
                        process::exit(1);
                    }
                    Ok(v) => v,
                }
            }
            _ => panic!("Unexpected argument \"{}\" in position {}", argument, pos),
        }
    }

    let (mut canvas, mut event_pump) = sdl_init(scaling_factor);
    let mut c8 = match rom_path {
        Some(p) => make_system(&read_rom(&p)),
        None    => panic!("No ROM path set!"),
    };

    'running: loop {
        if process_events(&mut event_pump) {
            break 'running
        }
 
        let instr = c8.fetch_and_decode();
        let flags = instr.get_flags();
        match flags {
            system::InstrFlags::Keys => c8.update_keys(read_keys(&event_pump)),
            system::InstrFlags::WaitKey => {
                c8.pressed_key = wait_on_key(&mut event_pump);
                if c8.pressed_key == 16 {
                    break 'running
                }
            }
            _ => {},
        }

        c8.execute(&instr);

        if flags == system::InstrFlags::Screen {
            draw_screen(scaling_factor, &mut canvas, &c8.screen);
        }
    }
}

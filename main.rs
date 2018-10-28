mod system;
mod asm;
mod sdl;
use system::{make_system, read_rom, instrs_to_rom};
use asm::parse_asm;
use sdl::{sdl_init, process_events, draw_screen, read_keys, wait_on_key};
use std::{env, process};
use std::path::Path;
use std::fs::File;
use std::io::BufReader;
use std::io::Read;
use std::error::Error;
use std::fs::OpenOptions;
use std::io::Write;

pub fn main() {
    let help = "\
            ./rchip8 <mode> <file> <scaling factor (-i) / output file name (-a)>\n\
            \n\
            The two modes are:\n\
                -a : assembler, where <file> is an assembly file\n\
                -i : interpret, where <file> is a ROM file\n\
            \n\
            Scaling factor multiplies the size of each Chip8 pixel. (default 1)\n\
            e.g. 2 means each block is 2x2 pixels in the final output.";

    enum Mode {
        Interpret,
        Assemble,
    };
    let mut mode = Mode::Interpret;

    let mut rom_path: Option<String> = None;
    let mut output_file: Option<String> = None;
    let mut scaling_factor = 1;

    let args = env::args().collect::<Vec<String>>();

    if args.contains(&"-h".to_string()) {
        println!("{}", help);
        process::exit(0);
    }

    if args.len() < 2 {
        println!("Mode argument required, one of '-i' (interpret) or '-a' (assemble).");
        process::exit(1);
    }
    if args.len() < 3 {
        println!("ROM or assembly file path is required.");
        process::exit(1);
    }

    for (pos, argument) in args.iter().enumerate() {
        match pos {
            0 => {},
            1 => { // Mode
                mode = match argument.as_str() {
                    "-i" => Mode::Interpret,
                    "-a" => Mode::Assemble,
                    _ => {
                        println!("Unknown mode argument: \"{}\"", argument);
                        process::exit(1);
                    }
                };
            }
            2 => {
                if Path::new(&argument).exists() {
                    rom_path = Some(argument.to_string());
                } else {
                    let file_type = match mode {
                        Mode::Interpret => "ROM",
                        Mode::Assemble => "Assembly",
                    };
                    println!("{} file \"{}\" not found.", argument, file_type);
                    process::exit(1);
                }
            },
            3 => {
                match mode {
                    Mode::Interpret => {
                        scaling_factor = match argument.parse::<i32>() {
                            Err(msg) => {
                                println!("Invalid scaling factor \"{}\": {}", argument, msg);
                                process::exit(1);
                            }
                            Ok(v) => v,
                        };
                    },
                    Mode::Assemble => output_file = Some(argument.to_string()),
                };
            },
            _ => {
                println!("Unexpected argument \"{}\" in position {}", argument, pos);
                process::exit(1);
            },
        }
    }

    match mode {
        Mode::Interpret => interpret_file(scaling_factor, &rom_path.unwrap()),
        Mode::Assemble => assemble_file(&rom_path.unwrap(), &output_file.unwrap()),
    }
}

fn assemble_file(asm_path: &str, output_file: &str) {
    let file = match File::open(asm_path) {
        Err(why) => panic!("Couldn't open assembly file: {}",why.description()),
        Ok(file) => file,
    };

    let mut buf_reader = BufReader::new(file);
    let mut contents = String::new();
    if let Err(msg) = buf_reader.read_to_string(&mut contents) {
        panic!("Couldn't read assembly file: {}", msg);
    };

    let mut warnings: Vec<String> = vec![];
    let res = parse_asm(&contents, asm_path, &mut warnings);
    for w in warnings {
        println!("{}", w);
    }
    let instrs = match res {
        Err(msgs) => {
            println!("{}", msgs);
            process::exit(1);
        },
        Ok(i) => i,
    };

    let binary = instrs_to_rom(&instrs);
    let mut file = OpenOptions::new()
                    .write(true)
                    .truncate(true)
                    .create(true)
                    .open(output_file)
                    .unwrap();

    if let Err(why) = file.write(&binary) {
        panic!("Couldn't write to output file: {}",
        why.description())
    }
}

fn interpret_file(scaling_factor: i32, rom_path: &str) {
    let (mut canvas, mut event_pump) = sdl_init(scaling_factor);
    let mut c8 = make_system(&read_rom(rom_path));

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

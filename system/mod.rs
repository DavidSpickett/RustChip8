use std::fs::File;
use std::path::Path;
use std::io::prelude::*;
use std::error::Error;
use system::instr::*;

pub fn make_system(rom_name: String) -> Chip8System {
    let mut c = Chip8System::new();
    c.init_memory(rom_name);
    return c;
}

#[derive(Copy, Clone)]
pub enum InstrFlags {
    _None,
    Screen,
    Keys,
}

pub struct Chip8System {
    pc : u16,
    memory : [u8 ; 0xFFFF],
    pub screen : [bool ; 64*32],
    pub keys : [bool ; 16],
    v_regs : [u8 ; 16],
    i_reg : u16,
    stack : [u16 ; 16],
    stack_ptr : u8,
    delay_timer : u8,
    sound_timer : u8,
}

impl Chip8System {
    pub fn new() -> Chip8System {
        Chip8System {
            pc: 0x200,
            memory: [0; 0xFFFF],
            screen: [false; 64*32],
            keys: [false; 16],
            v_regs: [0; 16],
            i_reg: 0,
            stack: [0; 16],
            stack_ptr: 0,
            delay_timer: 0,
            sound_timer: 0,
        }
    }

    pub fn screen_to_file(&self) {
        let path = Path::new("screen.txt");
        let mut file = match File::create(&path) {
            Err(why) => panic!("Couldn't create screen dump!: {}", why.description()),
            Ok(file) => file,
        };

        let mut row = String::from("");
        for (i, pixel) in self.screen.iter().enumerate() { //TODO: remove magic numbers
            if ((i % 64) == 0) &&  i != 0 { // TODO: make the screen a 2d array?
                row.push('\n');
                match file.write(row.as_bytes()) {
                    Err(why) => {
                        panic!("couldn't write to screen dump!: {}",
                                why.description())
                    },
                    Ok(_) => {},
                };
            }

            match *pixel {
                true => row.push('*'),
                false => row.push(' '),
            }
        }
    }

    pub fn update_keys(&mut self, key_state: [bool; 16]) {
        for (state, chip8key) in key_state.iter().zip(self.keys.iter_mut()) {
            *chip8key = *state;
        }
    }

    #[allow(dead_code)]
    fn dump(&self) {
        println!("----- Chip8 State -----");
        println!("PC: 0x{:04x} I: 0x{:04x}", self.pc, self.i_reg);
        print!("Delay Timer: {} Sound Timer: {}", self.delay_timer, self.sound_timer);
        for (i, v) in self.v_regs.iter().enumerate() {
            if i % 8 == 0 {
                println!();
            }
            print!("V{:02}: 0x{:02x} ", i, *v);
        }
        println!();
        println!("Stack:");
        for (i, addr) in self.stack.iter().enumerate() {
            print!("{:02}: 0x{:04x}", i, *addr);
            
            if i == (self.stack_ptr as usize) {
                println!(" <<<");
            } else {
                println!();
            }
        }
    }

    fn init_memory(&mut self, rom_name: String) {
        let font_data = [
                        0xF0, 0x90, 0x90, 0x90, 0xF0,  // Zero
                        0x20, 0x60, 0x20, 0x20, 0x70,  // One
                        0xF0, 0x10, 0xF0, 0x80, 0xF0,  // Two
                        0xF0, 0x10, 0xF0, 0x10, 0xF0,  // Three
                        0x90, 0x90, 0xF0, 0x10, 0x10,  // Four
                        0xF0, 0x80, 0xF0, 0x10, 0xF0,  // Five
                        0xF0, 0x80, 0xF0, 0x90, 0xF0,  // Six
                        0xF0, 0x10, 0x20, 0x40, 0x40,  // Seven
                        0xF0, 0x90, 0xF0, 0x90, 0xF0,  // Eight
                        0xF0, 0x90, 0xF0, 0x10, 0xF0,  // Nine
                        0xF0, 0x90, 0xF0, 0x90, 0x90,  // charA
                        0xE0, 0x90, 0xE0, 0x90, 0xE0,  // charB
                        0xF0, 0x80, 0x80, 0x80, 0xF0,  // charC
                        0xE0, 0x90, 0x90, 0x90, 0xE0,  // charD
                        0xF0, 0x80, 0xF0, 0x80, 0xF0,  // charE
                        0xF0, 0x80, 0xF0, 0x80, 0x80,  // charF
                        ];

        self.memory[..font_data.len()].clone_from_slice(&font_data);

        let mut file = match File::open(&rom_name) {
            Err(why) => panic!("couldn't open ROM: {}",why.description()),
            Ok(file) => file,
        };

        let mut contents = Vec::new();
        file.read_to_end(&mut contents).expect("Error reading ROM file.");
        self.memory[0x200..0x200+contents.len()].clone_from_slice(&contents);
    }

    fn fetch(&mut self) -> u16 {
        let opcode = ((self.memory[self.pc as usize] as u16) << 8) | (self.memory[(self.pc+1) as usize] as u16);
        self.pc += 2;
        return opcode
    }

    fn panic_unknown(&self, opcode: u16) {
        panic!("Unknown instruction 0x{:04X} at PC 0x{:04X}", opcode, self.pc-2);
    }

    fn get_opcode_obj(&self, opcode: u16) -> Box<Instr>{
        match opcode >> 12 {
            0x0 => {
                match opcode & 0xFF {
                    0xE0 => Box::new(ClearDisplayInstr::new(opcode)) as Box<Instr>,
                    0xEE => Box::new(RetInstr::new(opcode)) as Box<Instr>,
                    _ => {
                        self.panic_unknown(opcode);
                        panic!("");
                    }
                }
            }
            0x1 => Box::new(JumpInstr::new(opcode)) as Box<Instr>,
            0x2 => Box::new(CallInstr::new(opcode)) as Box<Instr>,
            0x3 => Box::new(SkipEqualInstr::new(opcode)) as Box<Instr>,
            0x4 => Box::new(SkipNotEqualInstr::new(opcode)) as Box<Instr>,
          //0x5 =>
            0x6 => Box::new(LoadByteInstr::new(opcode)) as Box<Instr>,
            0x7 => Box::new(AddByteInstr::new(opcode)) as Box<Instr>,
            0x8 => {
                match opcode & 0xF00F {
                    0x8000 => Box::new(MovRegInstr::new(opcode)) as Box<Instr>,
                    //0x8001 =>
                    //0x8002 =>
                    //0x8003 =>
                    //0x8004 =>
                    //0x8005 =>
                    //0x8006 =>
                    //0x8006 =>
                    //0x800E =>
                    _ => {
                        self.panic_unknown(opcode);
                        panic!("");
                    }
                }
            }
          //0x9 =>
            0xA => Box::new(LoadIInstr::new(opcode)) as Box<Instr>,
          //0xB =>
          //0xC =>
            0xD => Box::new(DrawSpriteInstr::new(opcode)) as Box<Instr>,
            0xE => {
                match opcode & 0xFF {
                    0x9E => Box::new(SkipKeyIfPressedInstr::new(opcode)) as Box<Instr>,
                    0xA1 => Box::new(SkipKeyIfNotPressedInstr::new(opcode)) as Box<Instr>,
                    _ => {
                        self.panic_unknown(opcode);
                        panic!("");
                    }
                }
            }
            0xF => {
                match opcode & 0xF0FF {
                    0xF007 => Box::new(GetDelayTimerInstr::new(opcode)) as Box<Instr>,
                    //0xF00A
                    0xF015 => Box::new(SetDelayTimerInstr::new(opcode)) as Box<Instr>,
                    //0xF018
                    0xF01E => Box::new(AddIVInstr::new(opcode)) as Box<Instr>, 
                    //0xF029
                    //0xF033
                    //0xF055
                    0xF065 => Box::new(ReadRegsFromMemInstr::new(opcode)) as Box<Instr>,
                    _ => {
                        self.panic_unknown(opcode);
                        panic!("");
                    }
                }
            }
            _ => {
                self.panic_unknown(opcode);
                panic!("");
            }
        }
    }

    pub fn fetch_and_decode(&mut self) -> Box<Instr> {
        let opc = self.fetch();
        self.get_opcode_obj(opc)
    }

    pub fn execute(&mut self, instr: Box<Instr>) {
        //TODO: check that fetch and decode has been called
        instr.exec(self);
        // -2 because we already fetched beyond this instr
        println!("0x{:04x} : 0x{:04x} : {}",
                 self.pc-2, instr.get_opcode(), instr.repr());
    }
}

mod instr;

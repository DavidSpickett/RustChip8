// Until I work out a way around Box<dyn Instr>, impl trait is close
#![cfg_attr(feature = "cargo-clippy", allow(clippy::borrowed_box))]

use std::fs::OpenOptions;
use std::io::prelude::*;
use system::instr::*;
use std::fs::File;
use std::io::Read;

mod test;
pub mod instr;

pub fn read_rom(filename: &str) -> Vec<u8> {
    let mut file = match File::open(filename) {
        Err(why) => panic!("couldn't open ROM: {}",why.to_string()),
        Ok(file) => file,
    };

    let mut contents = Vec::new();
    if let Err(msg) = file.read_to_end(&mut contents) {
        panic!("Error reading ROM file: {}", msg);
    }

    contents
}

pub fn instrs_to_rom(instrs: &[Box<dyn Instr>]) -> Vec<u8> {
    let mut rom: Vec<u8> = vec![];
    for i in instrs {
        let opc = i.get_opcode();
        rom.push((opc >> 8) as u8);
        rom.push(opc as u8);
    }
    rom
}

pub fn make_system(rom: &[u8]) -> Chip8System {
    let mut c = Chip8System::new();
    c.init_memory(&rom);
    c
}

#[derive(Copy, Clone, PartialEq)]
pub enum InstrFlags {
    _None,
    Screen,
    Keys,
    WaitKey,
    Sound,
}

pub const SCREEN_WIDTH: usize = 64;
pub const SCREEN_HEIGHT: usize = 32;

pub struct Chip8System {
    pc : u16,
    memory : [u8 ; 0xFFFF],
    pub screen : [bool ; SCREEN_WIDTH*SCREEN_HEIGHT],
    pub keys : [bool ; 16],
    pub pressed_key : usize,
    v_regs : [u8 ; 16],
    i_reg : u16,
    stack : Vec<u16>,
    delay_timer : u8,
    pub sound_timer : u8,
}

impl Chip8System {
    pub fn new() -> Chip8System {
        Chip8System {
            pc: 0x200,
            memory: [0; 0xFFFF],
            screen: [false; SCREEN_WIDTH*SCREEN_HEIGHT],
            keys: [false; 16],
            stack: vec![],
            pressed_key: 0,
            v_regs: [0; 16],
            i_reg: 0,
            delay_timer: 0,
            sound_timer: 0,
        }
    }

    #[allow(dead_code)]
    pub fn reset_regs(&mut self) {
        self.pc = 0x200;
        self.v_regs = [0; 16];
        self.i_reg = 0;
        self.stack = vec![];
        self.delay_timer = 0;
        self.sound_timer = 0;
    }

    pub fn bounds_check_i(&self, length: u8) -> usize {
        let end = (self.i_reg as usize) + (length as usize);
        if end >= self.memory.len() {
            panic!("I register memory access at 0x{:04x} with length {} is out of bounds!",
                   self.i_reg, length);
        }
        self.i_reg as usize
    }

    pub fn screen_to_str(&self) -> String {
        let mut ret = String::from("");
        let mut row = String::from("");
        for (i, pixel) in self.screen.iter().enumerate() {
            if ((i % SCREEN_WIDTH) == 0) &&  i != 0 {
                row.push('\n');
                ret.push_str(&row);
                row.clear();
            }

            if *pixel { row.push('@') } else { row.push('-') }
        }
        ret.push_str(&row);
        ret
    }
    
    #[allow(dead_code)]
    pub fn screen_to_file(&self) {
        let mut file = OpenOptions::new()
                        .write(true)
                        .truncate(true)
                        .create(true)
                        .open("screen.txt")
                        .unwrap();

        if let Err(why) = file.write(self.screen_to_str().as_bytes()) {
            panic!("couldn't write to screen dump!: {}",
                why.to_string())
        }
    }

    pub fn update_keys(&mut self, key_state: [bool; 16]) {
        for (state, chip8key) in key_state.iter().zip(self.keys.iter_mut()) {
            *chip8key = *state;
        }
    }

    pub fn get_keystate(&self, idx: u8) -> bool {
        if idx >= 16 {
            panic!("Key number {} out of range!", idx);
        }
        self.keys[idx as usize]
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
        if self.stack.is_empty() {
            println!("<...>");
        } else {
            for (i, addr) in self.stack.iter().enumerate() {
                println!("{:02}: 0x{:04x}", i, *addr);
            }
        }
    }

    fn init_memory(&mut self, rom: &[u8]) {
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
        self.memory[0x200..0x200+rom.len()].clone_from_slice(&rom);
    }

    fn fetch(&mut self) -> u16 {
        let opcode = (u16::from(self.memory[self.pc as usize]) << 8) | u16::from(self.memory[(self.pc+1) as usize]);
        self.pc += 2;
        opcode
    }

    fn format_unknown(&self, opcode: u16) -> String {
        // Not the best place for this but it works
        self.dump();
        format!("Unknown instruction 0x{:04X} at PC 0x{:04X}", opcode, self.pc-2)
    }

    fn get_opcode_obj(&self, opcode: u16) -> Result<Box<dyn Instr>, String> {
        match opcode >> 12 {
            0x0 => {
                match opcode & 0xFFF {
                    // Note that these first two *must* begin with 0, as in 0x00E0
                    0x0E0 => Ok(Box::new(ClearDisplayInstr::new(opcode)) as Box<dyn Instr>),
                    0x0EE => Ok(Box::new(RetInstr::new(opcode))          as Box<dyn Instr>),
                    0xFFF => {
                        // Special BRK instr
                        self.dump();
                        panic!(format!("BRK instruction encountered at PC 0x{:04X}", self.pc-2));
                    },
                    _ =>    Ok(Box::new(SysInstr::new(opcode))          as Box<dyn Instr>),
                }
            }
            0x1 => Ok(Box::new(JumpInstr::new(opcode))         as Box<dyn Instr>),
            0x2 => Ok(Box::new(CallInstr::new(opcode))         as Box<dyn Instr>),
            0x3 => Ok(Box::new(SkipEqualInstr::new(opcode))    as Box<dyn Instr>),
            0x4 => Ok(Box::new(SkipNotEqualInstr::new(opcode)) as Box<dyn Instr>),
            0x5 => {
                match opcode & 0xF {
                    0 => Ok(Box::new(SkipIfRegsEqualInstr::new(opcode)) as Box<dyn Instr>),
                    _ => Err(self.format_unknown(opcode)),
                }
            }
            0x6 => Ok(Box::new(LoadByteInstr::new(opcode)) as Box<dyn Instr>),
            0x7 => Ok(Box::new(AddByteInstr::new(opcode))  as Box<dyn Instr>),
            0x8 => {
                match opcode & 0xF {
                    0x0 => Ok(Box::new(MovRegInstr::new(opcode))  as Box<dyn Instr>),
                    0x1 => Ok(Box::new(OrRegInstr::new(opcode))   as Box<dyn Instr>),
                    0x2 => Ok(Box::new(AndRegInstr::new(opcode))  as Box<dyn Instr>),
                    0x3 => Ok(Box::new(XORRegInstr::new(opcode))  as Box<dyn Instr>),
                    0x4 => Ok(Box::new(AddRegInstr::new(opcode))  as Box<dyn Instr>),
                    0x5 => Ok(Box::new(SubRegInstr::new(opcode))  as Box<dyn Instr>),
                    0x6 => Ok(Box::new(ShrRegInstr::new(opcode))  as Box<dyn Instr>),
                    0x7 => Ok(Box::new(SubNRegInstr::new(opcode)) as Box<dyn Instr>),
                    0xE => Ok(Box::new(ShlRegInstr::new(opcode))  as Box<dyn Instr>),
                    _   => Err(self.format_unknown(opcode)),
                }
            }
            0x9 => {
                match opcode & 0xF {
                    0 => Ok(Box::new(SkipIfRegsNotEqualInstr::new(opcode)) as Box<dyn Instr>),
                    _ => Err(self.format_unknown(opcode)),
                }
            }
            0xA => Ok(Box::new(LoadIInstr::new(opcode))         as Box<dyn Instr>),
            0xB => Ok(Box::new(JumpPlusVZeroInstr::new(opcode)) as Box<dyn Instr>),
            0xC => Ok(Box::new(RandomInstr::new(opcode))        as Box<dyn Instr>),
            0xD => Ok(Box::new(DrawSpriteInstr::new(opcode))    as Box<dyn Instr>),
            0xE => {
                match opcode & 0xFF {
                    0x9E => Ok(Box::new(SkipKeyIfPressedInstr::new(opcode))    as Box<dyn Instr>),
                    0xA1 => Ok(Box::new(SkipKeyIfNotPressedInstr::new(opcode)) as Box<dyn Instr>),
                    _    => Err(self.format_unknown(opcode)),
                }
            }
            0xF => {
                match opcode & 0xFF {
                    0x07 => Ok(Box::new(GetDelayTimerInstr::new(opcode))   as Box<dyn Instr>),
                    0x0A => Ok(Box::new(WaitForKeyInstr::new(opcode))      as Box<dyn Instr>),
                    0x15 => Ok(Box::new(SetDelayTimerInstr::new(opcode))   as Box<dyn Instr>),
                    0x18 => Ok(Box::new(SetSoundTimerInstr::new(opcode))   as Box<dyn Instr>),
                    0x1E => Ok(Box::new(AddIVInstr::new(opcode))           as Box<dyn Instr>),
                    0x29 => Ok(Box::new(GetDigitAddrInstr::new(opcode))    as Box<dyn Instr>),
                    0x33 => Ok(Box::new(StoreBCDInstr::new(opcode))        as Box<dyn Instr>),
                    0x55 => Ok(Box::new(WriteRegsToMemInstr::new(opcode))  as Box<dyn Instr>),
                    0x65 => Ok(Box::new(ReadRegsFromMemInstr::new(opcode)) as Box<dyn Instr>),
                    _    => Err(self.format_unknown(opcode)),
                }
            }
            _ => Err(self.format_unknown(opcode)),
        }
    }

    pub fn fetch_and_decode(&mut self) -> Box<dyn Instr> {
        // TODO: base these on actual frequencies, not number of instructions
        unsafe {
            static DT_FUDGE_RESET: u16 = 100;
            static mut DT_FUDGE: u16 = DT_FUDGE_RESET;
            DT_FUDGE = match DT_FUDGE {
                0 => {
                    if self.delay_timer != 0 {
                        self.delay_timer -= 1;
                    }
                    DT_FUDGE_RESET
                }
                _ => DT_FUDGE - 1,
            };

            static ST_FUDGE_RESET: u16 = 150;
            static mut ST_FUDGE: u16 = ST_FUDGE_RESET;
            ST_FUDGE = match ST_FUDGE {
                0 => {
                    if self.sound_timer != 0 {
                        self.sound_timer -= 1;
                    }
                    ST_FUDGE_RESET
                },
                _ => ST_FUDGE - 1,
            };
        }

        let opc = self.fetch();
        let decode = self.get_opcode_obj(opc);
        match decode {
            Ok(instr) => {
                // -2 because we already fetched beyond this instr
                // Print this now because otherwise jumps won't look right
                // You'll see the post jump PC, not the PC we fetched the
                // jump from.
                println!("0x{:04x} : 0x{:04x} : {}",
                         self.pc-2, instr.get_opcode(), instr.repr());

                instr
            }
            Err(msg) => panic!(msg),
        }
    }

    pub fn execute(&mut self, instr: &Box<dyn Instr>) {
        //TODO: check that fetch and decode has been called
        instr.exec(self);
    }
}

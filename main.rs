use std::fs::File;
use std::io::prelude::*;
use std::error::Error;

fn op_to_kk(opcode: u16) -> u8 {
    (opcode & 0xFF) as u8
}

fn op_to_nnn(opcode: u16) -> u16 {
    opcode & 0xFFF
}

struct Chip8System {
    pc : u16,
    memory : [u8 ; 0xFFFF],
    screen : [bool ; 64*32],
    v_regs : [u8 ; 16],
    i_reg : u16,
    stack : [u16 ; 16],
    stack_ptr : u8,
    delay_timer : u8,
    sound_timer : u8,
}

impl Chip8System {
    fn new() -> Chip8System {
        Chip8System {
            pc: 0x200,
            memory: [0; 0xFFFF],
            screen: [false; 64*32],
            v_regs: [0; 16],
            i_reg: 0,
            stack: [0; 16],
            stack_ptr: 0,
            delay_timer: 0,
            sound_timer: 0,
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

    fn get_vx(&self, opcode: u16) -> u8 {
        self.v_regs[((opcode >> 8) & 0xF) as usize]
    }

    fn get_vy(&self, opcode: u16) -> u8 {
        self.v_regs[((opcode >> 4) & 0xF) as usize]
    }

    fn set_vx(&mut self, opcode: u16, value: u8) {
        self.v_regs[((opcode >> 8) & 0xF) as usize] = value;
    }

    fn panic_unknown(&self, opcode: u16) {
        panic!("Unknown instruction 0x{:04X} at PC 0x{:04X}", opcode, self.pc-2);
    }

    fn do_opcode(&mut self) {
        let opcode = self.fetch();

        match opcode >> 12 {
            0x0 => {
                match opcode & 0xFF {
                    0xE0 => {
                        for p in self.screen.iter_mut() {
                            *p = false;
                        }
                    }
                    0xEE => {
                        self.pc = self.stack[self.stack_ptr as usize];
                        self.stack_ptr -= 1;
                    }
                    _ => self.panic_unknown(opcode),
                }
            }
            0x1 => self.pc = op_to_nnn(opcode),
            0x2 => {
                self.stack_ptr += 1;
                self.stack[self.stack_ptr as usize] = self.pc;
                self.pc = op_to_nnn(opcode);
            }
            0x3 => {
                if self.get_vx(opcode) == op_to_kk(opcode) {
                    self.pc += 2;
                }
            }
            0x6 => self.set_vx(opcode, op_to_kk(opcode)), 
            0x7 => {
                let cur = self.get_vx(opcode);
                self.set_vx(opcode, cur + op_to_kk(opcode));
            }
            0x8 => {
                let x = self.get_vx(opcode);
                let y = self.get_vy(opcode);
                self.set_vx(opcode, x & y);
            }
            0xA => self.i_reg = opcode & 0xFFF,
            0xD => println!("FIXME: draw sprite!"),
            0xE => println!("FIXME: Skip if key pressed!"),
            0xF => println!("FIXME: delay timer!"),
            
            _ => self.panic_unknown(opcode),
        }
    }
}

fn main() {
    let mut c8 = Chip8System::new();
    c8.init_memory(String::from("INVADERS"));

    loop {
        c8.do_opcode();
    }
}

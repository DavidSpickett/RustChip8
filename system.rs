use std::fs::File;
use std::io::prelude::*;
use std::error::Error;

pub fn make_system(rom_name: String) -> Chip8System {
    let mut c = Chip8System::new();
    c.init_memory(rom_name);
    return c;
}

fn op_to_kk(opcode: u16) -> u8 {
    (opcode & 0xFF) as u8
}

fn op_to_nnn(opcode: u16) -> u16 {
    opcode & 0xFFF
}

fn op_to_vx(opcode: u16) -> u8 {
    ((opcode >> 8) & 0xf) as u8
}

fn op_to_vy(opcode: u16) -> u8 {
    ((opcode >> 4) & 0xF) as u8
}

pub struct Chip8System {
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
    pub fn new() -> Chip8System {
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

    fn get_opcode_obj(&self, opcode: u16) -> Box<Instr>{
        match opcode >> 12 {
            0x0 => {
                match opcode & 0xFF {
                    0xE0 => Box::new(clear_display_instr::new(opcode)) as Box<Instr>,
                    _ => panic!("Unknown instr!"),
                }
            }
            0x1 => Box::new(jump_instr::new(opcode)) as Box<Instr>,
            0x2 => Box::new(call_instr::new(opcode)) as Box<Instr>,
            0x3 => Box::new(skip_equal_instr::new(opcode)) as Box<Instr>,
            0x6 => Box::new(load_byte_instr::new(opcode)) as Box<Instr>,
            0x8 => Box::new(mov_reg_instr::new(opcode)) as Box<Instr>,
            0xD => panic!("FIXME: draw sprite!"),
            0xE => panic!("FIXME: Skip if key pressed!"),
            0xF => panic!("FIXME: delay timer!"),
            _ => {
                self.panic_unknown(opcode);
                panic!("<< so it knows this won't return");
            }
        }
    }

    pub fn do_opcode(&mut self) {
        self.dump();
        let opc = self.fetch();
        let opcode = self.get_opcode_obj(opc);
        println!("{}", opcode.repr());
        opcode.exec(self);

        /*match opcode >> 12 {
            0x0 => {
                match opcode & 0xFF {
                    0xEE => {
                        self.pc = self.stack[self.stack_ptr as usize];
                        self.stack_ptr -= 1;
                    }
                }
            }
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
        }*/
    }
}



trait Instr {
    fn repr(&self) -> String;
    fn exec(&self, C8: &mut Chip8System);
    //fn construct() -> instr; //because writing an assembler is too hard
}

struct call_instr {
    opcode: u16,
    target: u16,
}

impl call_instr {
    fn new(opc: u16) -> call_instr {
        call_instr {
            opcode: opc,
            target: opc & 0xFFF,
        }
    }
}

impl Instr for call_instr {
    fn repr(&self) -> String {
        format!("CALL 0x{:03x}", self.target)
    }

    fn exec(&self, C8: &mut Chip8System) {
        C8.stack_ptr += 1;
        C8.stack[C8.stack_ptr as usize] = C8.pc;
        C8.pc = op_to_nnn(self.opcode);
    }
}

struct jump_instr {
    opcode: u16,
    target: u16,
}

impl jump_instr {
    fn new(opc: u16) -> jump_instr {
        jump_instr {
            opcode: opc,
            target: op_to_nnn(opc),
        }
    }
}

impl Instr for jump_instr {
    fn repr(&self) -> String {
        format!("JP 0x{:03x}", self.target)
    }

    fn exec(&self, C8: &mut Chip8System) {
        C8.pc = self.target;
    }
}

struct skip_equal_instr {
    opcode: u16,
    vx: u8,
    kk: u8,
}

impl skip_equal_instr {
    fn new(opc: u16) -> skip_equal_instr {
        skip_equal_instr {
            opcode: opc,
            vx: op_to_vx(opc),
            kk: op_to_kk(opc),
        }
    }
}

impl Instr for skip_equal_instr {
    fn repr(&self) -> String {
        format!("SE V{}, 0x{:02x}", self.vx, self.kk)
    }

    fn exec(&self, C8: &mut Chip8System) {
        if C8.v_regs[self.vx as usize] == self.kk {
            C8.pc += 2;
        }
    }
}

struct load_byte_instr {
    opcode: u16,
    vx: u8,
    kk: u8,
}

impl load_byte_instr {
    fn new(opc: u16) -> load_byte_instr {
        load_byte_instr {
            opcode: opc,
            vx: op_to_vx(opc),
            kk: op_to_kk(opc),
        }
    }
}

impl Instr for load_byte_instr {
    fn repr(&self) -> String {
        format!("LD V{}, 0x{:02x}", self.vx, self.kk)
    }

    fn exec(&self, C8: &mut Chip8System) {
        C8.v_regs[self.vx as usize] = self.kk;
    }
}

struct clear_display_instr {
    opcode: u16,
}

impl clear_display_instr {
    fn new(opc: u16) -> clear_display_instr {
        clear_display_instr {
            opcode: opc
        }
    }
}

impl Instr for clear_display_instr {
    fn repr(&self) -> String {
        String::from("CLS")
    }

    fn exec(&self, C8: &mut Chip8System) {
        for p in C8.screen.iter_mut() {
            *p = false;
        }
    }
}

struct mov_reg_instr {
    opcode: u16,
    vx: u8,
    vy: u8,
}

impl mov_reg_instr {
    fn new(opc: u16) -> mov_reg_instr {
        mov_reg_instr {
            opcode: opc,
            vx: op_to_vx(opc),
            vy: op_to_vy(opc),
        }
    }
}

impl Instr for mov_reg_instr {
    fn repr(&self) -> String {
        format!("LD V{}, V{}", self.vx, self.vy)
    }

    fn exec(&self, C8: &mut Chip8System) {
        C8.v_regs[self.vx as usize] = C8.v_regs[self.vy as usize];
    }
}

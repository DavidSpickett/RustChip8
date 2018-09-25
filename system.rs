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
                    0xEE => Box::new(ret_instr::new(opcode)) as Box<Instr>,
                    _ => {
                        self.panic_unknown(opcode);
                        panic!("");
                    }
                }
            }
            0x1 => Box::new(jump_instr::new(opcode)) as Box<Instr>,
            0x2 => Box::new(call_instr::new(opcode)) as Box<Instr>,
            0x3 => Box::new(skip_equal_instr::new(opcode)) as Box<Instr>,
            0x6 => Box::new(load_byte_instr::new(opcode)) as Box<Instr>,
            0x7 => Box::new(add_byte_instr::new(opcode)) as Box<Instr>,
            0x8 => Box::new(mov_reg_instr::new(opcode)) as Box<Instr>,
            0xA => Box::new(load_i_instr::new(opcode)) as Box<Instr>,
            0xD => Box::new(undef_instr::new(opcode, String::from("FIXME: draw sprite!"))),
            0xE => Box::new(undef_instr::new(opcode, String::from("FIXME: Skip if key pressed!"))),
            0xF => {
                match opcode & 0xF0FF {
                    0xF01E => Box::new(add_iv_instr::new(opcode)) as Box<Instr>, 
                    0xF015 => Box::new(set_delay_timer_instr::new(opcode)) as Box<Instr>,
                    0xF007 => Box::new(get_delay_timer_instr::new(opcode)) as Box<Instr>,
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

    pub fn do_opcode(&mut self) {
        let opc = self.fetch();
        let opcode = self.get_opcode_obj(opc);
        // -2 since we already fetched
        println!("0x{:04x} : 0x{:04x} : {}", self.pc-2, opc, opcode.repr());
        opcode.exec(self);
        self.dump();
        println!();
    }
}



trait Instr {
    fn repr(&self) -> String;
    fn exec(&self, C8: &mut Chip8System);
    //fn construct() -> instr; //because writing an assembler is too hard
}

struct undef_instr {
    opcode: u16,
    message: String,
}

impl undef_instr {
    fn new(opc: u16, msg: String) -> undef_instr {
        undef_instr {
            opcode: opc,
            message: msg,
        }
    }
}

impl Instr for undef_instr {
    fn repr(&self) -> String {
        format!("{}", self.message) 
    }

    fn exec(&self, C8: &mut Chip8System) {}
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

struct ret_instr {
    opcode: u16,
}

impl ret_instr {
    fn new(opc: u16) -> ret_instr {
        ret_instr {
            opcode: opc,
        }
    }
}

impl Instr for ret_instr {
    fn repr(&self) -> String {
        format!("RET")
    }

    fn exec(&self, C8: &mut Chip8System) {
        C8.pc = C8.stack[C8.stack_ptr as usize];
        C8.stack_ptr -= 1;
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

struct load_i_instr {
    opcode: u16,
    nnn: u16,
}

impl load_i_instr {
    fn new(opc: u16) -> load_i_instr {
        load_i_instr {
            opcode: opc,
            nnn: op_to_nnn(opc),
        }
    }
}

impl Instr for load_i_instr {
    fn repr(&self) -> String {
        format!("LD I, 0x{:03x}", self.nnn)
    }

    fn exec(&self, C8: &mut Chip8System) {
        C8.i_reg = self.nnn;
    }
}

struct add_byte_instr {
    opcode: u16,
    vx: u8,
    kk: u8,
}

impl add_byte_instr {
    fn new(opc: u16) -> add_byte_instr {
        add_byte_instr {
            opcode: opc,
            vx: op_to_vx(opc),
            kk: op_to_kk(opc),
        }
    }
}

impl Instr for add_byte_instr {
    fn repr(&self) -> String {
        format!("ADD V{}, 0x{:02x}", self.vx, self.kk)
    }

    fn exec(&self, C8: &mut Chip8System) {
        C8.v_regs[self.vx as usize] = C8.v_regs[self.vx as usize].wrapping_add(self.kk)
    }
}

struct add_iv_instr {
    opcode: u16,
    vx: u8,
}

impl add_iv_instr {
    fn new(opc: u16) -> add_iv_instr {
        add_iv_instr {
            opcode: opc,
            vx: op_to_vx(opc),
        }
    }
}

impl Instr for add_iv_instr {
    fn repr(&self) -> String {
        format!("ADD I, V{}", self.vx)
    }

    fn exec(&self, C8: &mut Chip8System) {
        C8.i_reg += C8.v_regs[self.vx as usize] as u16
    }
}

struct set_delay_timer_instr {
    opcode: u16,
    vx: u8,
}

impl set_delay_timer_instr {
    fn new(opc: u16) -> set_delay_timer_instr {
        set_delay_timer_instr {
            opcode: opc,
            vx: op_to_vx(opc),
        }
    }
}

impl Instr for set_delay_timer_instr {
    fn repr(&self) -> String {
        format!("LD DT, V{}", self.vx)
    }

    fn exec(&self, C8: &mut Chip8System) {
        C8.delay_timer = C8.v_regs[self.vx as usize]
    }
}

struct get_delay_timer_instr {
    opcode: u16,
    vx: u8,
}

impl get_delay_timer_instr {
    fn new(opc: u16) -> get_delay_timer_instr {
        get_delay_timer_instr {
            opcode: opc,
            vx: op_to_vx(opc),
        }
    }
}

impl Instr for get_delay_timer_instr {
    fn repr(&self) -> String {
        format!("LD V{}, DT", self.vx)
    }

    fn exec(&self, C8: &mut Chip8System) {
        C8.v_regs[self.vx as usize] = C8.delay_timer
    }
}

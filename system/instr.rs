use system::Chip8System;

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

pub trait Instr {
    fn repr(&self) -> String;
    fn exec(&self, C8: &mut Chip8System);
    //fn construct() -> instr; //because writing an assembler is too hard
}

pub struct undef_instr {
    opcode: u16,
    message: String,
}

impl undef_instr {
    pub fn new(opc: u16, msg: String) -> undef_instr {
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

pub struct call_instr {
    opcode: u16,
    target: u16,
}

impl call_instr {
    pub fn new(opc: u16) -> call_instr {
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

pub struct jump_instr {
    opcode: u16,
    target: u16,
}

impl jump_instr {
    pub fn new(opc: u16) -> jump_instr {
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

pub struct ret_instr {
    opcode: u16,
}

impl ret_instr {
    pub fn new(opc: u16) -> ret_instr {
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

pub struct skip_equal_instr {
    opcode: u16,
    vx: u8,
    kk: u8,
}

impl skip_equal_instr {
    pub fn new(opc: u16) -> skip_equal_instr {
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

pub struct load_byte_instr {
    opcode: u16,
    vx: u8,
    kk: u8,
}

impl load_byte_instr {
    pub fn new(opc: u16) -> load_byte_instr {
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

pub struct clear_display_instr {
    opcode: u16,
}

impl clear_display_instr {
    pub fn new(opc: u16) -> clear_display_instr {
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

pub struct mov_reg_instr {
    opcode: u16,
    vx: u8,
    vy: u8,
}

impl mov_reg_instr {
    pub fn new(opc: u16) -> mov_reg_instr {
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

pub struct load_i_instr {
    opcode: u16,
    nnn: u16,
}

impl load_i_instr {
    pub fn new(opc: u16) -> load_i_instr {
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

pub struct add_byte_instr {
    opcode: u16,
    vx: u8,
    kk: u8,
}

impl add_byte_instr {
    pub fn new(opc: u16) -> add_byte_instr {
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

pub struct add_iv_instr {
    opcode: u16,
    vx: u8,
}

impl add_iv_instr {
    pub fn new(opc: u16) -> add_iv_instr {
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

pub struct set_delay_timer_instr {
    opcode: u16,
    vx: u8,
}

impl set_delay_timer_instr {
    pub fn new(opc: u16) -> set_delay_timer_instr {
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

pub struct get_delay_timer_instr {
    opcode: u16,
    vx: u8,
}

impl get_delay_timer_instr {
    pub fn new(opc: u16) -> get_delay_timer_instr {
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

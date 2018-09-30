use system::Chip8System;
use system::InstrFlags;

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
    fn get_opcode(&self) -> u16;
    fn get_flags(&self) -> InstrFlags;
}

struct InstrCore {
    opcode: u16,
    flags: InstrFlags,
}

impl InstrCore {
    fn new(opc: u16, flags: InstrFlags) -> InstrCore {
        InstrCore {
            opcode: opc,
            flags: flags,
        }
    }
}

pub struct undef_instr {
    core: InstrCore,
    message: String,
}

impl undef_instr {
    pub fn new(opc: u16, msg: String) -> undef_instr {
        undef_instr {
            core: InstrCore::new(opc, InstrFlags::_None),
            message: msg,
        }
    }
}

impl Instr for undef_instr {
    fn repr(&self) -> String {
        format!("{}", self.message) 
    }

    fn exec(&self, C8: &mut Chip8System) {}

    fn get_opcode(&self) -> u16 {
        self.core.opcode
    }

    fn get_flags(&self) -> InstrFlags {
        self.core.flags
    }
}

pub struct call_instr {
    core: InstrCore,
    target: u16,
}

impl call_instr {
    pub fn new(opc: u16) -> call_instr {
        call_instr {
            core: InstrCore::new(opc, InstrFlags::_None),
            target: op_to_nnn(opc),
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
        C8.pc = op_to_nnn(self.core.opcode);
    }

    fn get_opcode(&self) -> u16 {
        self.core.opcode
    }

    fn get_flags(&self) -> InstrFlags {
        self.core.flags
    }
}

pub struct jump_instr {
    core: InstrCore,
    target: u16,
}

impl jump_instr {
    pub fn new(opc: u16) -> jump_instr {
        jump_instr {
            core: InstrCore::new(opc, InstrFlags::_None),
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

    fn get_opcode(&self) -> u16 {
        self.core.opcode
    }

    fn get_flags(&self) -> InstrFlags {
        self.core.flags
    }
}

pub struct ret_instr {
    core: InstrCore,
}

impl ret_instr {
    pub fn new(opc: u16) -> ret_instr {
        ret_instr {
            core: InstrCore::new(opc, InstrFlags::_None),
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

    fn get_opcode(&self) -> u16 {
        self.core.opcode
    }

    fn get_flags(&self) -> InstrFlags {
        self.core.flags
    }
}

pub struct skip_equal_instr {
    core: InstrCore,
    vx: u8,
    kk: u8,
}

impl skip_equal_instr {
    pub fn new(opc: u16) -> skip_equal_instr {
        skip_equal_instr {
            core: InstrCore::new(opc, InstrFlags::_None),
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

    fn get_opcode(&self) -> u16 {
        self.core.opcode
    }

    fn get_flags(&self) -> InstrFlags {
        self.core.flags
    }
}

pub struct skip_not_equal_instr {
    core: InstrCore,
    vx: u8,
    kk: u8,
}

impl skip_not_equal_instr {
    pub fn new(opc: u16) -> skip_not_equal_instr {
        skip_not_equal_instr {
            core: InstrCore::new(opc, InstrFlags::_None),
            vx: op_to_vx(opc),
            kk: op_to_kk(opc),
        }
    }
}

impl Instr for skip_not_equal_instr {
    fn repr(&self) -> String {
        format!("SNE V{}, 0x{:02x}", self.vx, self.kk)
    }

    fn exec(&self, C8: &mut Chip8System) {
        if C8.v_regs[self.vx as usize] != self.kk {
            C8.pc +=2;
        }
    }

    fn get_opcode(&self) -> u16 {
        self.core.opcode
    }

    fn get_flags(&self) -> InstrFlags {
        self.core.flags
    }
}

pub struct load_byte_instr {
    core: InstrCore,
    vx: u8,
    kk: u8,
}

impl load_byte_instr {
    pub fn new(opc: u16) -> load_byte_instr {
        load_byte_instr {
            core: InstrCore::new(opc, InstrFlags::_None),
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

    fn get_opcode(&self) -> u16 {
        self.core.opcode
    }

    fn get_flags(&self) -> InstrFlags {
        self.core.flags
    }
}

pub struct clear_display_instr {
    core: InstrCore,
}

impl clear_display_instr {
    pub fn new(opc: u16) -> clear_display_instr {
        clear_display_instr {
            core: InstrCore::new(opc, InstrFlags::_None),
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

    fn get_opcode(&self) -> u16 {
        self.core.opcode
    }

    fn get_flags(&self) -> InstrFlags {
        self.core.flags
    }
}

pub struct mov_reg_instr {
    core: InstrCore,
    vx: u8,
    vy: u8,
}

impl mov_reg_instr {
    pub fn new(opc: u16) -> mov_reg_instr {
        mov_reg_instr {
            core: InstrCore::new(opc, InstrFlags::_None),
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

    fn get_opcode(&self) -> u16 {
        self.core.opcode
    }

    fn get_flags(&self) -> InstrFlags {
        self.core.flags
    }
}

pub struct load_i_instr {
    core: InstrCore,
    nnn: u16,
}

impl load_i_instr {
    pub fn new(opc: u16) -> load_i_instr {
        load_i_instr {
            core: InstrCore::new(opc, InstrFlags::_None),
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

    fn get_opcode(&self) -> u16 {
        self.core.opcode
    }

    fn get_flags(&self) -> InstrFlags {
        self.core.flags
    }
}

pub struct add_byte_instr {
    core: InstrCore,
    vx: u8,
    kk: u8,
}

impl add_byte_instr {
    pub fn new(opc: u16) -> add_byte_instr {
        add_byte_instr {
            core: InstrCore::new(opc, InstrFlags::_None),
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

    fn get_opcode(&self) -> u16 {
        self.core.opcode
    }

    fn get_flags(&self) -> InstrFlags {
        self.core.flags
    }
}

pub struct add_iv_instr {
    core: InstrCore,
    vx: u8,
}

impl add_iv_instr {
    pub fn new(opc: u16) -> add_iv_instr {
        add_iv_instr {
            core: InstrCore::new(opc, InstrFlags::_None),
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

    fn get_opcode(&self) -> u16 {
        self.core.opcode
    }

    fn get_flags(&self) -> InstrFlags {
        self.core.flags
    }
}

pub struct set_delay_timer_instr {
    core: InstrCore,
    vx: u8,
}

impl set_delay_timer_instr {
    pub fn new(opc: u16) -> set_delay_timer_instr {
        set_delay_timer_instr {
            core: InstrCore::new(opc, InstrFlags::_None),
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

    fn get_opcode(&self) -> u16 {
        self.core.opcode
    }

    fn get_flags(&self) -> InstrFlags {
        self.core.flags
    }
}

pub struct get_delay_timer_instr {
    core: InstrCore,
    vx: u8,
}

impl get_delay_timer_instr {
    pub fn new(opc: u16) -> get_delay_timer_instr {
        get_delay_timer_instr {
            core: InstrCore::new(opc, InstrFlags::_None),
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

    fn get_opcode(&self) -> u16 {
        self.core.opcode
    }

    fn get_flags(&self) -> InstrFlags {
        self.core.flags
    }
}

pub struct draw_sprite_instr {
    core: InstrCore,
    vx: u8,
    vy: u8,
    n: u8,
}

impl draw_sprite_instr {
    pub fn new(opc: u16) -> draw_sprite_instr {
        draw_sprite_instr {
            core: InstrCore::new(opc, InstrFlags::Screen),
            vx: op_to_vx(opc),
            vy: op_to_vy(opc),
            n: (opc & 0xF) as u8,
        }
    }
}

impl Instr for draw_sprite_instr {
    fn repr(&self) -> String {
        format!("DRW V{}, V{}, {}", self.vx, self.vy, self.n)
    }

    fn exec(&self, C8: &mut Chip8System) {
        //Clear overlap flag
        C8.v_regs[15] = 0;

        let x = C8.v_regs[self.vx as usize];
        let y = C8.v_regs[self.vy as usize];
        let addr = C8.i_reg as usize;
        let sprite_data = &C8.memory[addr..addr+(self.n as usize)];

        for (y_offset, row) in sprite_data.iter().enumerate() {
            for sprite_x in (0..8).rev() {
                let final_x = ((x+7-sprite_x) % 64) as usize;
                let final_y = ((y as usize) + y_offset) % 32;
                let screen_idx = (final_y*64)+final_x;

                let pixel_set = *row & (1 << sprite_x) != 0; 
                let pixel_was = C8.screen[screen_idx];
                 
                if pixel_set && pixel_was {
                    C8.v_regs[15] = 1;
                }
                C8.screen[screen_idx] ^= pixel_set;
            }
        }
    }

    fn get_opcode(&self) -> u16 {
        self.core.opcode
    }

    fn get_flags(&self) -> InstrFlags {
        self.core.flags
    }
}

pub struct skip_key_if_pressed_instr {
    core: InstrCore,
    vx: u8,
}

impl skip_key_if_pressed_instr {
    pub fn new(opc: u16) -> skip_key_if_pressed_instr {
        skip_key_if_pressed_instr {
            core: InstrCore::new(opc, InstrFlags::Keys),
            vx: op_to_vx(opc),
        }
    }
}

impl Instr for skip_key_if_pressed_instr {
    fn repr(&self) -> String {
        format!("SKP V{}", self.vx)
    }

    fn exec(&self, C8: &mut Chip8System) {
        let key_num = C8.v_regs[self.vx as usize] as usize;
        if C8.keys[key_num] {
            C8.pc += 2;
        }
    }

    fn get_opcode(&self) -> u16 {
        self.core.opcode
    }

    fn get_flags(&self) -> InstrFlags {
        self.core.flags
    }
}

pub struct skip_key_if_not_pressed_instr {
    core: InstrCore,
    vx: u8,
}

impl skip_key_if_not_pressed_instr {
    pub fn new(opc: u16) -> skip_key_if_not_pressed_instr {
        skip_key_if_not_pressed_instr {
            core: InstrCore::new(opc, InstrFlags::Keys),
            vx: op_to_vx(opc),
        }
    }
}

impl Instr for skip_key_if_not_pressed_instr {
    fn repr(&self) -> String {
        format!("SKNP V{}", self.vx)
    }

    fn exec(&self, C8: &mut Chip8System) {
        let key_num = C8.v_regs[self.vx as usize] as usize;
        if !C8.keys[key_num] {
            C8.pc += 2;
        }
    }

    fn get_opcode(&self) -> u16 {
        self.core.opcode
    }

    fn get_flags(&self) -> InstrFlags {
        self.core.flags
    }
}

pub struct read_regs_from_mem_instr {
    core: InstrCore,
    vx: u8,
}

impl read_regs_from_mem_instr {
    pub fn new(opc: u16) -> read_regs_from_mem_instr {
        read_regs_from_mem_instr {
            core: InstrCore::new(opc, InstrFlags::_None),
            vx: op_to_vx(opc),
        }
    }
}

impl Instr for read_regs_from_mem_instr {
    fn repr(&self) -> String {
        format!("LD V{}, [I]", self.vx)
    }

    fn exec(&self, C8: &mut Chip8System) {
        let addr = C8.i_reg as usize;
        for reg_idx in 0..(self.vx+1) {
            C8.v_regs[reg_idx as usize] = C8.memory[addr];
        }
    }

    fn get_opcode(&self) -> u16 {
        self.core.opcode
    }

    fn get_flags(&self) -> InstrFlags {
        self.core.flags
    }
}

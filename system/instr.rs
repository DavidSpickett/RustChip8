use system::Chip8System;
use system::InstrFlags;

extern crate rand;
use system::instr::rand::Rng;

mod instr_builder {
    fn check_v_reg(num: u8, name: &str) {
        if num >= 16 {
            panic!("V{} cannot be >= 16 !", name);
        }
    }

    pub fn no_args(base: u16) -> u16 {
        base
    }

    pub fn arg_nnn(base: u16, target: u16) -> u16 {
        if target > 0xFFF {
            panic!("nnn cannot be > 0xFFF");
        }
        (base & 0xF000) | (target & 0x0FFF)
    }

    pub fn arg_x_kk(base: u16, x: u8, kk: u8) -> u16 {
        check_v_reg(x, "X");
        (base & 0xF000) | ((u16::from(x) & 0xF) << 8) | u16::from(kk)
    }

    pub fn arg_x_y(base: u16, x: u8, y: u8) -> u16 {
        check_v_reg(x, "X");
        check_v_reg(y, "Y");

        (base & 0xF00F) | ((u16::from(x) & 0xF) << 8) | ((u16::from(y) & 0xF) << 4)
    }

    pub fn arg_x_y_n(base: u16, x: u8, y: u8, n: u8) -> u16 {
        check_v_reg(x, "X");
        check_v_reg(y, "Y");
        if n >= 16 { panic!("n cannot be >= 16 !"); }

        (base & 0xF000) | ((u16::from(x) & 0xF) << 8) | ((u16::from(y) & 0xF) << 4) |
            (u16::from(n) & 0xF)
    }

    pub fn arg_x(base: u16, x: u8) -> u16 {
        check_v_reg(x, "X");
        (base & 0xF0FF) | ((u16::from(x) & 0xF) << 8)
    }
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

pub trait Instr {
    fn repr(&self) -> String {
        let mut ret = format!("{}", self.get_mnemonic());
        let args = self.get_formatted_args();
        if !args.is_empty() {
            ret += &format!(" {}", args);
        }
        ret
    }

    fn exec(&self, c8: &mut Chip8System);
    fn get_mnemonic(&self) -> &String;
    fn get_formatted_args(&self) -> String;
    fn get_opcode(&self) -> u16;
    fn get_flags(&self) -> InstrFlags;
}

struct InstrCore {
    opcode: u16,
    flags: InstrFlags,
    mnemonic: String,
}

impl InstrCore {
    fn new(opcode: u16, flags: InstrFlags, mnemonic: &str) -> InstrCore {
        InstrCore {
            opcode,
            flags,
            mnemonic : mnemonic.to_string(),
        }
    }
}

macro_rules! impl_instr {
    () => (
        fn get_opcode(&self) -> u16 { self.core.opcode }
        fn get_flags(&self) -> InstrFlags { self.core.flags}
        fn get_mnemonic(&self) -> &String { &self.core.mnemonic }
    )
}

#[allow(dead_code)]
pub struct UndefInstr {
    core: InstrCore,
    message: String,
}

#[allow(dead_code)]
impl UndefInstr {
    pub fn new(opc: u16, msg: String) -> UndefInstr {
        UndefInstr {
            core: InstrCore::new(opc, InstrFlags::_None, "UNDEF"),
            message: msg,
        }
    }
}

#[allow(dead_code)]
impl Instr for UndefInstr {
    impl_instr!();

    fn get_formatted_args(&self) -> String {
        self.message.to_string()
    }

    fn exec(&self, _c8: &mut Chip8System) {}
}

pub struct SysInstr {
    core: InstrCore,
    target: u16,
}

impl SysInstr {
    pub fn new(opc: u16) -> SysInstr {
        SysInstr {
            core: InstrCore::new(opc, InstrFlags::_None, "SYS"),
            target: op_to_nnn(opc),
        }
    }

    pub fn create(target: u16) -> SysInstr {
        SysInstr::new(instr_builder::arg_nnn(0x0000, target))
    }
}

impl Instr for SysInstr {
    impl_instr!();

    fn get_formatted_args(&self) -> String {
        format!("0x{:03x}", self.target)
    }

    fn exec(&self, _c8: &mut Chip8System) {}
}

pub struct CallInstr {
    core: InstrCore,
    target: u16,
}

impl CallInstr {
    pub fn new(opc: u16) -> CallInstr {
        CallInstr {
            core: InstrCore::new(opc, InstrFlags::_None, "CALL"),
            target: op_to_nnn(opc),
        }
    }

    pub fn create(target: u16) -> CallInstr {
        CallInstr::new(instr_builder::arg_nnn(0x2000, target))
    }
}

impl Instr for CallInstr {
    impl_instr!();

    fn get_formatted_args(&self) -> String {
        format!("0x{:03x}", self.target)
    }

    fn exec(&self, c8: &mut Chip8System) {
        if c8.stack.len() == 16 {
            panic!("Stack is full!")
        }

        c8.stack.push(c8.pc);
        c8.pc = self.target;
    }
}

pub struct JumpInstr {
    core: InstrCore,
    target: u16,
}

impl JumpInstr {
    pub fn new(opc: u16) -> JumpInstr {
        JumpInstr {
            core: InstrCore::new(opc, InstrFlags::_None, "JP"),
            target: op_to_nnn(opc),
        }
    }

    pub fn create(target: u16) -> JumpInstr {
        JumpInstr::new(instr_builder::arg_nnn(0x1000, target))
    }
}

impl Instr for JumpInstr {
    impl_instr!();

    fn get_formatted_args(&self) -> String {
        format!("0x{:03x}", self.target)
    }

    fn exec(&self, c8: &mut Chip8System) {
        c8.pc = self.target;
    }
}

pub struct RetInstr {
    core: InstrCore,
}

impl RetInstr {
    pub fn new(opc: u16) -> RetInstr {
        RetInstr {
            core: InstrCore::new(opc, InstrFlags::_None, "RET"),
        }
    }

    pub fn create() -> RetInstr {
        RetInstr::new(instr_builder::no_args(0x00EE))
    }
}

impl Instr for RetInstr {
    impl_instr!();

    fn get_formatted_args(&self) -> String {
        "".to_string()
    }

    fn exec(&self, c8: &mut Chip8System) {
        if c8.stack.is_empty() {
            panic!("Stack is empty!");
        }
        c8.pc = c8.stack.pop().unwrap();
    }
}

pub struct SkipEqualInstr {
    core: InstrCore,
    vx: u8,
    kk: u8,
}

impl SkipEqualInstr {
    pub fn new(opc: u16) -> SkipEqualInstr {
        SkipEqualInstr {
            core: InstrCore::new(opc, InstrFlags::_None, "SE"),
            vx: op_to_vx(opc),
            kk: op_to_kk(opc),
        }
    }

    pub fn create(x: u8, kk: u8) -> SkipEqualInstr {
        SkipEqualInstr::new(instr_builder::arg_x_kk(0x3000, x, kk))
    }
}

impl Instr for SkipEqualInstr {
    impl_instr!();

    fn get_formatted_args(&self) -> String {
        format!("V{}, 0x{:02x}", self.vx, self.kk)
    }

    fn exec(&self, c8: &mut Chip8System) {
        if c8.v_regs[self.vx as usize] == self.kk {
            c8.pc += 2;
        }
    }
}

pub struct SkipNotEqualInstr {
    core: InstrCore,
    vx: u8,
    kk: u8,
}

impl SkipNotEqualInstr {
    pub fn new(opc: u16) -> SkipNotEqualInstr {
        SkipNotEqualInstr {
            core: InstrCore::new(opc, InstrFlags::_None, "SNE"),
            vx: op_to_vx(opc),
            kk: op_to_kk(opc),
        }
    }

    pub fn create(x: u8, kk: u8) -> SkipNotEqualInstr {
        SkipNotEqualInstr::new(instr_builder::arg_x_kk(0x4000, x, kk))
    }
}

impl Instr for SkipNotEqualInstr {
    impl_instr!();

    fn get_formatted_args(&self) -> String {
        format!("V{}, 0x{:02x}", self.vx, self.kk)
    }

    fn exec(&self, c8: &mut Chip8System) {
        if c8.v_regs[self.vx as usize] != self.kk {
            c8.pc +=2;
        }
    }
}

pub struct LoadByteInstr {
    core: InstrCore,
    vx: u8,
    kk: u8,
}

impl LoadByteInstr {
    pub fn new(opc: u16) -> LoadByteInstr {
        LoadByteInstr {
            core: InstrCore::new(opc, InstrFlags::_None, "LD"),
            vx: op_to_vx(opc),
            kk: op_to_kk(opc),
        }
    }

    pub fn create(x: u8, kk: u8) -> LoadByteInstr {
        LoadByteInstr::new(instr_builder::arg_x_kk(0x6000, x, kk))
    }
}

impl Instr for LoadByteInstr {
    impl_instr!();

    fn get_formatted_args(&self) -> String {
        format!("V{}, 0x{:02x}", self.vx, self.kk)
    }

    fn exec(&self, c8: &mut Chip8System) {
        c8.v_regs[self.vx as usize] = self.kk;
    }
}

pub struct ClearDisplayInstr {
    core: InstrCore,
}

impl ClearDisplayInstr {
    pub fn new(opc: u16) -> ClearDisplayInstr {
        ClearDisplayInstr {
            core: InstrCore::new(opc, InstrFlags::Screen, "CLS"),
        }
    }

    pub fn create() -> ClearDisplayInstr {
        ClearDisplayInstr::new(instr_builder::no_args(0x00E0))
    }
}

impl Instr for ClearDisplayInstr {
    impl_instr!();

    fn get_formatted_args(&self) -> String {
        String::from("")
    }

    fn exec(&self, c8: &mut Chip8System) {
        for p in c8.screen.iter_mut() {
            *p = false;
        }
    }
}

pub struct MovRegInstr {
    core: InstrCore,
    vx: u8,
    vy: u8,
}

impl MovRegInstr {
    pub fn new(opc: u16) -> MovRegInstr {
        MovRegInstr {
            core: InstrCore::new(opc, InstrFlags::_None, "LD"),
            vx: op_to_vx(opc),
            vy: op_to_vy(opc),
        }
    }

    pub fn create(x: u8, y: u8) -> MovRegInstr {
        MovRegInstr::new(instr_builder::arg_x_y(0x8000, x, y))
    }
}

impl Instr for MovRegInstr {
    impl_instr!();

    fn get_formatted_args(&self) -> String {
        format!("V{}, V{}", self.vx, self.vy)
    }

    fn exec(&self, c8: &mut Chip8System) {
        c8.v_regs[self.vx as usize] = c8.v_regs[self.vy as usize];
    }
}

pub struct OrRegInstr {
    core: InstrCore,
    vx: u8,
    vy: u8,
}

impl OrRegInstr {
    pub fn new(opc: u16) -> OrRegInstr {
        OrRegInstr {
            core: InstrCore::new(opc, InstrFlags::_None, "OR"),
            vx: op_to_vx(opc),
            vy: op_to_vy(opc),
        }
    }

    pub fn create(x: u8, y: u8) -> OrRegInstr {
        OrRegInstr::new(instr_builder::arg_x_y(0x8001, x, y))
    }

}

impl Instr for OrRegInstr {
    impl_instr!();

    fn get_formatted_args(&self) -> String {
        format!("V{}, V{}", self.vx, self.vy)
    }

    fn exec(&self, c8: &mut Chip8System) {
        c8.v_regs[self.vx as usize] |= c8.v_regs[self.vy as usize];
    }
}

pub struct AndRegInstr {
    core: InstrCore,
    vx: u8,
    vy: u8,
}

impl AndRegInstr {
    pub fn new(opc: u16) -> AndRegInstr {
        AndRegInstr {
            core: InstrCore::new(opc, InstrFlags::_None, "AND"),
            vx: op_to_vx(opc),
            vy: op_to_vy(opc),
        }
    }

    pub fn create(x: u8, y: u8) -> AndRegInstr {
        AndRegInstr::new(instr_builder::arg_x_y(0x8002, x, y))
    }
}

impl Instr for AndRegInstr {
    impl_instr!();

    fn get_formatted_args(&self) -> String {
        format!("V{}, V{}", self.vx, self.vy)
    }

    fn exec(&self, c8: &mut Chip8System) {
        c8.v_regs[self.vx as usize] &= c8.v_regs[self.vy as usize];
    }
}

pub struct XORRegInstr {
    core: InstrCore,
    vx: u8,
    vy: u8,
}

impl XORRegInstr {
    pub fn new(opc: u16) -> XORRegInstr {
        XORRegInstr {
            core: InstrCore::new(opc, InstrFlags::_None, "XOR"),
            vx: op_to_vx(opc),
            vy: op_to_vy(opc),
        }
    }

    pub fn create(x: u8, y: u8) -> XORRegInstr {
        XORRegInstr::new(instr_builder::arg_x_y(0x8003, x, y))
    }
}

impl Instr for XORRegInstr {
    impl_instr!();

    fn get_formatted_args(&self) -> String {
        format!("V{}, V{}", self.vx, self.vy)
    }

    fn exec(&self, c8: &mut Chip8System) {
        c8.v_regs[self.vx as usize] ^= c8.v_regs[self.vy as usize];
    }
}

pub struct AddRegInstr {
    core: InstrCore,
    vx: u8,
    vy: u8,
}

impl AddRegInstr {
    pub fn new(opc: u16) -> AddRegInstr {
        AddRegInstr {
            core: InstrCore::new(opc, InstrFlags::_None, "ADD"),
            vx: op_to_vx(opc),
            vy: op_to_vy(opc),
        }
    }

    pub fn create(x: u8, y: u8) -> AddRegInstr {
        AddRegInstr::new(instr_builder::arg_x_y(0x8004, x, y))
    }
}

impl Instr for AddRegInstr {
    impl_instr!();

    fn get_formatted_args(&self) -> String {
        format!("V{}, V{}", self.vx, self.vy)
    }

    fn exec(&self, c8: &mut Chip8System) {
        let x = c8.v_regs[self.vx as usize];
        let y = c8.v_regs[self.vy as usize];

        c8.v_regs[self.vx as usize] = c8.v_regs[self.vx as usize].wrapping_add(y);

        if (u16::from(x) + u16::from(y)) > 0xFF {
            c8.v_regs[15] = 1;
        } else {
            c8.v_regs[15] = 0;
        }
    }
}

pub struct SubRegInstr {
    core: InstrCore,
    vx: u8,
    vy: u8,
}

impl SubRegInstr {
    pub fn new(opc: u16) -> SubRegInstr {
        SubRegInstr {
            core: InstrCore::new(opc, InstrFlags::_None, "SUB"),
            vx: op_to_vx(opc),
            vy: op_to_vy(opc),
        }
    }

    pub fn create(x: u8, y: u8) -> SubRegInstr {
        SubRegInstr::new(instr_builder::arg_x_y(0x8005, x, y))
    }
}

impl Instr for SubRegInstr {
    impl_instr!();

    fn get_formatted_args(&self) -> String {
        format!("V{}, V{}", self.vx, self.vy)
    }

    fn exec(&self, c8: &mut Chip8System) {
        let x = c8.v_regs[self.vx as usize];
        let y = c8.v_regs[self.vy as usize];

        c8.v_regs[self.vx as usize] = x.wrapping_sub(y);
        c8.v_regs[15] = (x>y) as u8;
    }
}

pub struct SubNRegInstr {
    core: InstrCore,
    vx: u8,
    vy: u8,
}

impl SubNRegInstr {
    pub fn new(opc: u16) -> SubNRegInstr {
        SubNRegInstr {
            core: InstrCore::new(opc, InstrFlags::_None, "SUBN"),
            vx: op_to_vx(opc),
            vy: op_to_vy(opc),
        }
    }

    pub fn create(x: u8, y: u8) -> SubNRegInstr {
        SubNRegInstr::new(instr_builder::arg_x_y(0x8007, x, y))
    }
}

impl Instr for SubNRegInstr {
    impl_instr!();

    fn get_formatted_args(&self) -> String {
        format!("V{}, V{}", self.vx, self.vy)
    }

    fn exec(&self, c8: &mut Chip8System) {
        let x = c8.v_regs[self.vx as usize];
        let y = c8.v_regs[self.vy as usize];

        c8.v_regs[self.vx as usize] = y.wrapping_sub(x);
        c8.v_regs[15] = (y>x) as u8;
    }
}

pub struct ShrRegInstr {
    core: InstrCore,
    vx: u8,
    // Docs indicate Vy but it isn't actually used
}

impl ShrRegInstr {
    pub fn new(opc: u16) -> ShrRegInstr {
        ShrRegInstr {
            core: InstrCore::new(opc, InstrFlags::_None, "SHR"),
            vx: op_to_vx(opc),
        }
    }

    pub fn create(x: u8) -> ShrRegInstr {
        ShrRegInstr::new(instr_builder::arg_x(0x8006, x))
    }
}

impl Instr for ShrRegInstr {
    impl_instr!();

    fn get_formatted_args(&self) -> String {
        format!("V{}", self.vx)
    }

    fn exec(&self, c8: &mut Chip8System) {
        let x = c8.v_regs[self.vx as usize];
        c8.v_regs[15] = x & 1;
        c8.v_regs[self.vx as usize] >>= 1;
    }
}

pub struct ShlRegInstr {
    core: InstrCore,
    vx: u8,
}

impl ShlRegInstr {
    pub fn new(opc: u16) -> ShlRegInstr {
        ShlRegInstr {
            core: InstrCore::new(opc, InstrFlags::_None, "SHL"),
            vx: op_to_vx(opc),
        }
    }

    pub fn create(x: u8) -> ShlRegInstr {
        ShlRegInstr::new(instr_builder::arg_x(0x800E, x))
    }
}

impl Instr for ShlRegInstr {
    impl_instr!();

    fn get_formatted_args(&self) -> String {
        format!("V{}", self.vx)
    }

    fn exec(&self, c8: &mut Chip8System) {
        let x = c8.v_regs[self.vx as usize];
        c8.v_regs[15] = x >> 7;
        c8.v_regs[self.vx as usize] <<= 1;
    }
}

pub struct LoadIInstr {
    core: InstrCore,
    nnn: u16,
}

impl LoadIInstr {
    pub fn new(opc: u16) -> LoadIInstr {
        LoadIInstr {
            core: InstrCore::new(opc, InstrFlags::_None, "LD"),
            nnn: op_to_nnn(opc),
        }
    }

    pub fn create(target: u16) -> LoadIInstr {
        LoadIInstr::new(instr_builder::arg_nnn(0xA000, target))
    }
}

impl Instr for LoadIInstr {
    impl_instr!();

    fn get_formatted_args(&self) -> String {
        format!("I, 0x{:03x}", self.nnn)
    }

    fn exec(&self, c8: &mut Chip8System) {
        c8.i_reg = self.nnn;
    }
}

pub struct AddByteInstr {
    core: InstrCore,
    vx: u8,
    kk: u8,
}

impl AddByteInstr {
    pub fn new(opc: u16) -> AddByteInstr {
        AddByteInstr {
            core: InstrCore::new(opc, InstrFlags::_None, "ADD"),
            vx: op_to_vx(opc),
            kk: op_to_kk(opc),
        }
    }

    pub fn create(x: u8, kk: u8) -> AddByteInstr {
        AddByteInstr::new(instr_builder::arg_x_kk(0x7000, x, kk))
    }
}

impl Instr for AddByteInstr {
    impl_instr!();

    fn get_formatted_args(&self) -> String {
        format!("V{}, 0x{:02x}", self.vx, self.kk)
    }

    fn exec(&self, c8: &mut Chip8System) {
        c8.v_regs[self.vx as usize] = c8.v_regs[self.vx as usize].wrapping_add(self.kk)
    }
}

pub struct AddIVInstr {
    core: InstrCore,
    vx: u8,
}

impl AddIVInstr {
    pub fn new(opc: u16) -> AddIVInstr {
        AddIVInstr {
            core: InstrCore::new(opc, InstrFlags::_None, "ADD"),
            vx: op_to_vx(opc),
        }
    }

    pub fn create(x: u8) -> AddIVInstr {
        AddIVInstr::new(instr_builder::arg_x(0xF01E, x))
    }
}

impl Instr for AddIVInstr {
    impl_instr!();

    fn get_formatted_args(&self) -> String {
        format!("I, V{}", self.vx)
    }

    fn exec(&self, c8: &mut Chip8System) {
        c8.i_reg = c8.i_reg.wrapping_add(u16::from(c8.v_regs[self.vx as usize]))
    }
}

pub struct SetDelayTimerInstr {
    core: InstrCore,
    vx: u8,
}

impl SetDelayTimerInstr {
    pub fn new(opc: u16) -> SetDelayTimerInstr {
        SetDelayTimerInstr {
            core: InstrCore::new(opc, InstrFlags::_None, "LD"),
            vx: op_to_vx(opc),
        }
    }

    pub fn create(x: u8) -> SetDelayTimerInstr {
        SetDelayTimerInstr::new(instr_builder::arg_x(0xF015, x))
    }
}

impl Instr for SetDelayTimerInstr {
    impl_instr!();

    fn get_formatted_args(&self) -> String {
        format!("DT, V{}", self.vx)
    }

    fn exec(&self, c8: &mut Chip8System) {
        c8.delay_timer = c8.v_regs[self.vx as usize]
    }
}

pub struct GetDelayTimerInstr {
    core: InstrCore,
    vx: u8,
}

impl GetDelayTimerInstr {
    pub fn new(opc: u16) -> GetDelayTimerInstr {
        GetDelayTimerInstr {
            core: InstrCore::new(opc, InstrFlags::_None, "LD"),
            vx: op_to_vx(opc),
        }
    }

    pub fn create(x: u8) -> GetDelayTimerInstr {
        GetDelayTimerInstr::new(instr_builder::arg_x(0xF007, x))
    }
}

impl Instr for GetDelayTimerInstr {
    impl_instr!();

    fn get_formatted_args(&self) -> String {
        format!("V{}, DT", self.vx)
    }

    fn exec(&self, c8: &mut Chip8System) {
        c8.v_regs[self.vx as usize] = c8.delay_timer
    }
}

pub struct DrawSpriteInstr {
    core: InstrCore,
    vx: u8,
    vy: u8,
    n: u8,
}

impl DrawSpriteInstr {
    pub fn new(opc: u16) -> DrawSpriteInstr {
        DrawSpriteInstr {
            core: InstrCore::new(opc, InstrFlags::Screen, "DRW"),
            vx: op_to_vx(opc),
            vy: op_to_vy(opc),
            n: (opc & 0xF) as u8,
        }
    }

    pub fn create(x: u8, y: u8, n: u8) -> DrawSpriteInstr {
        DrawSpriteInstr::new(instr_builder::arg_x_y_n(0xD000, x, y, n))
    }
}

impl Instr for DrawSpriteInstr {
    impl_instr!();

    fn get_formatted_args(&self) -> String {
        format!("V{}, V{}, {}", self.vx, self.vy, self.n)
    }

    fn exec(&self, c8: &mut Chip8System) {
        //Clear overlap flag
        c8.v_regs[15] = 0;

        let x = c8.v_regs[self.vx as usize] as usize;
        let y = c8.v_regs[self.vy as usize] as usize;
        let addr = c8.bounds_check_i(self.n);
        let sprite_data = &c8.memory[addr..addr+(self.n as usize)];

        for (y_offset, row) in sprite_data.iter().enumerate() {
            for sprite_x in (0..8).rev() {
                let final_x = (x+7-sprite_x) % 64;
                let final_y = (y + y_offset) % 32;
                let screen_idx = (final_y*64)+final_x;

                let pixel_set = *row & (1 << sprite_x) != 0; 
                let pixel_was = c8.screen[screen_idx];
                 
                if pixel_set && pixel_was {
                    c8.v_regs[15] = 1;
                }
                c8.screen[screen_idx] ^= pixel_set;
            }
        }
    }
}

pub struct SkipKeyIfPressedInstr {
    core: InstrCore,
    vx: u8,
}

impl SkipKeyIfPressedInstr {
    pub fn new(opc: u16) -> SkipKeyIfPressedInstr {
        SkipKeyIfPressedInstr {
            core: InstrCore::new(opc, InstrFlags::Keys, "SKP"),
            vx: op_to_vx(opc),
        }
    }

    pub fn create(x: u8) -> SkipKeyIfPressedInstr {
        SkipKeyIfPressedInstr::new(instr_builder::arg_x(0xE09E, x))
    }
}

impl Instr for SkipKeyIfPressedInstr {
    impl_instr!();

    fn get_formatted_args(&self) -> String {
        format!("V{}", self.vx)
    }

    fn exec(&self, c8: &mut Chip8System) {
        if c8.get_keystate(c8.v_regs[self.vx as usize]) {
            c8.pc += 2;
        }
    }
}

pub struct SkipKeyIfNotPressedInstr {
    core: InstrCore,
    vx: u8,
}

impl SkipKeyIfNotPressedInstr {
    pub fn new(opc: u16) -> SkipKeyIfNotPressedInstr {
        SkipKeyIfNotPressedInstr {
            core: InstrCore::new(opc, InstrFlags::Keys, "SKNP"),
            vx: op_to_vx(opc),
        }
    }

    pub fn create(x: u8) -> SkipKeyIfNotPressedInstr {
        SkipKeyIfNotPressedInstr::new(instr_builder::arg_x(0xE0A1, x))
    }
}

impl Instr for SkipKeyIfNotPressedInstr {
    impl_instr!();

    fn get_formatted_args(&self) -> String {
        format!("V{}", self.vx)
    }

    fn exec(&self, c8: &mut Chip8System) {
        if !c8.get_keystate(c8.v_regs[self.vx as usize]) {
            c8.pc += 2;
        }
    }
}

pub struct ReadRegsFromMemInstr {
    core: InstrCore,
    vx: u8,
}

impl ReadRegsFromMemInstr {
    pub fn new(opc: u16) -> ReadRegsFromMemInstr {
        ReadRegsFromMemInstr {
            core: InstrCore::new(opc, InstrFlags::_None, "LD"),
            vx: op_to_vx(opc),
        }
    }

    pub fn create(x: u8) -> ReadRegsFromMemInstr {
        ReadRegsFromMemInstr::new(instr_builder::arg_x(0xF065, x))
    }
}

impl Instr for ReadRegsFromMemInstr {
    impl_instr!();

    fn get_formatted_args(&self) -> String {
        format!("V{}, [I]", self.vx)
    }

    fn exec(&self, c8: &mut Chip8System) {
        let addr = c8.bounds_check_i(self.vx+1);
        for reg_idx in 0..(self.vx+1) {
            c8.v_regs[reg_idx as usize] = c8.memory[addr+(reg_idx as usize)];
        }
    }
}

pub struct WriteRegsToMemInstr {
    core: InstrCore,
    vx: u8,
}

impl WriteRegsToMemInstr {
    pub fn new(opc: u16) -> WriteRegsToMemInstr {
        WriteRegsToMemInstr {
            core: InstrCore::new(opc, InstrFlags::_None, "LD"),
            vx: op_to_vx(opc),
        }
    }

    pub fn create(x: u8) -> WriteRegsToMemInstr {
        WriteRegsToMemInstr::new(instr_builder::arg_x(0xF055, x))
    }
}

impl Instr for WriteRegsToMemInstr {
    impl_instr!();

    fn get_formatted_args(&self) -> String {
        format!("[I], V{}", self.vx)
    }

    fn exec(&self, c8: &mut Chip8System) {
        let addr = c8.bounds_check_i(self.vx+1);
        for reg_idx in 0..(self.vx+1) {
            c8.memory[addr+(reg_idx as usize)] = c8.v_regs[reg_idx as usize];
        }
    }
}

pub struct SetSoundTimerInstr {
    core: InstrCore,
    vx: u8,
}

impl SetSoundTimerInstr {
    pub fn new(opc: u16) -> SetSoundTimerInstr {
        SetSoundTimerInstr {
            core: InstrCore::new(opc, InstrFlags::Sound, "LD"),
            vx: op_to_vx(opc),
        }
    }

    pub fn create(x: u8) -> SetSoundTimerInstr {
        SetSoundTimerInstr::new(instr_builder::arg_x(0xF018, x))
    }
}

impl Instr for SetSoundTimerInstr {
    impl_instr!();

    fn get_formatted_args(&self) -> String {
        format!("ST, V{}", self.vx)
    }

    fn exec(&self, c8: &mut Chip8System) {
        c8.sound_timer = c8.v_regs[self.vx as usize];
    }
}

pub struct RandomInstr {
    core: InstrCore,
    vx: u8,
    kk: u8,
}

impl RandomInstr {
    pub fn new(opc: u16) -> RandomInstr {
        RandomInstr {
            core: InstrCore::new(opc, InstrFlags::Sound, "RND"),
            vx: op_to_vx(opc),
            kk: op_to_kk(opc),
        }
    }

    pub fn create(x: u8, kk: u8) -> RandomInstr {
        RandomInstr::new(instr_builder::arg_x_kk(0xC000, x, kk))
    }
}

impl Instr for RandomInstr {
    impl_instr!();

    fn get_formatted_args(&self) -> String {
        format!("V{}, 0x{:02x}", self.vx, self.kk)
    }

    fn exec(&self, c8: &mut Chip8System) {
        let mut rng = rand::thread_rng();
        c8.v_regs[self.vx as usize] = self.kk & rng.gen::<u8>();
    }
}

pub struct SkipIfRegsEqualInstr {
    core: InstrCore,
    vx: u8,
    vy: u8,
}

impl SkipIfRegsEqualInstr {
    pub fn new(opc: u16) -> SkipIfRegsEqualInstr {
        SkipIfRegsEqualInstr {
            core: InstrCore::new(opc, InstrFlags::_None, "SE"),
            vx: op_to_vx(opc),
            vy: op_to_vy(opc),
        }
    }

    pub fn create(x: u8, y: u8) -> SkipIfRegsEqualInstr {
        SkipIfRegsEqualInstr::new(instr_builder::arg_x_y(0x5000, x, y))
    }
}

impl Instr for SkipIfRegsEqualInstr {
    impl_instr!();

    fn get_formatted_args(&self) -> String {
        format!("V{}, V{}", self.vx, self.vy)
    }

    fn exec(&self, c8: &mut Chip8System) {
        if c8.v_regs[self.vx as usize] == c8.v_regs[self.vy as usize] {
            c8.pc += 2;
        }
    }
}

pub struct SkipIfRegsNotEqualInstr {
    core: InstrCore,
    vx: u8,
    vy: u8,
}

impl SkipIfRegsNotEqualInstr {
    pub fn new(opc: u16) -> SkipIfRegsNotEqualInstr {
        SkipIfRegsNotEqualInstr {
            core: InstrCore::new(opc, InstrFlags::_None, "SNE"),
            vx: op_to_vx(opc),
            vy: op_to_vy(opc),
        }
    }

    pub fn create(x: u8, y: u8) -> SkipIfRegsNotEqualInstr {
        SkipIfRegsNotEqualInstr::new(instr_builder::arg_x_y(0x9000, x, y))
    }
}

impl Instr for SkipIfRegsNotEqualInstr {
    impl_instr!();

    fn get_formatted_args(&self) -> String {
        format!("V{}, V{}", self.vx, self.vy)
    }

    fn exec(&self, c8: &mut Chip8System) {
        if c8.v_regs[self.vx as usize] != c8.v_regs[self.vy as usize] {
            c8.pc += 2;
        }
    }
}

pub struct JumpPlusVZeroInstr {
    core: InstrCore,
    target: u16,
}

impl JumpPlusVZeroInstr {
    pub fn new(opc: u16) -> JumpPlusVZeroInstr {
        JumpPlusVZeroInstr {
            core: InstrCore::new(opc, InstrFlags::_None, "JP"),
            target: op_to_nnn(opc),
        }
    }

    pub fn create(target: u16) -> JumpPlusVZeroInstr {
        JumpPlusVZeroInstr::new(instr_builder::arg_nnn(0xB000, target))
    }
}

impl Instr for JumpPlusVZeroInstr {
    impl_instr!();

    fn get_formatted_args(&self) -> String {
        format!("V0, 0x{:03x}", self.target)
    }

    fn exec(&self, c8: &mut Chip8System) {
        c8.pc = self.target + u16::from(c8.v_regs[0]);
    }
}

pub struct GetDigitAddrInstr {
    core: InstrCore,
    vx: u8,
}

impl GetDigitAddrInstr {
    pub fn new(opc: u16) -> GetDigitAddrInstr {
        GetDigitAddrInstr {
            core: InstrCore::new(opc, InstrFlags::_None, "LD"),
            vx: op_to_vx(opc),
        }
    }

    pub fn create(x: u8) -> GetDigitAddrInstr {
        GetDigitAddrInstr::new(instr_builder::arg_x(0xF029, x))
    }
}

impl Instr for GetDigitAddrInstr {
    impl_instr!();

    fn get_formatted_args(&self) -> String {
        format!("F, V{}", self.vx)
    }

    fn exec(&self, c8: &mut Chip8System) {
        let digit = u16::from(c8.v_regs[self.vx as usize]);
        c8.i_reg = digit*5;
    }
}

pub struct StoreBCDInstr {
    core: InstrCore,
    vx: u8,
}

impl StoreBCDInstr {
    pub fn new(opc: u16) -> StoreBCDInstr {
        StoreBCDInstr {
            core: InstrCore::new(opc, InstrFlags::_None, "LD"),
            vx: op_to_vx(opc),
        }
    }

    pub fn create(x: u8) -> StoreBCDInstr {
        StoreBCDInstr::new(instr_builder::arg_x(0xF033, x))
    }
}

impl Instr for StoreBCDInstr {
    impl_instr!();

    fn get_formatted_args(&self) -> String {
        format!("B, V{}", self.vx)
    }

    fn exec(&self, c8: &mut Chip8System) {
        let mut value = c8.v_regs[self.vx as usize];
        let mut addr = c8.bounds_check_i(3);

        let hundreds = value / 100;
        c8.memory[addr] = hundreds;
        value -= 100 * hundreds;
        addr += 1;

        let tens = value / 10;
        c8.memory[addr] = tens;
        value -= 10 * tens;
        addr += 1;

        c8.memory[addr] = value;
    }
}

pub struct WaitForKeyInstr {
    core: InstrCore,
    vx: u8,
}

impl WaitForKeyInstr {
    pub fn new(opc: u16) -> WaitForKeyInstr {
        WaitForKeyInstr {
            core: InstrCore::new(opc, InstrFlags::WaitKey, "LD"),
            vx: op_to_vx(opc),
        }
    }

    pub fn create(x: u8) -> WaitForKeyInstr {
        WaitForKeyInstr::new(instr_builder::arg_x(0xF00A, x))
    }
}

impl Instr for WaitForKeyInstr {
    impl_instr!();

    fn get_formatted_args(&self) -> String {
        format!("V{}, K", self.vx)
    }

    fn exec(&self, c8: &mut Chip8System) {
        c8.v_regs[self.vx as usize] = c8.pressed_key as u8;
    }
}

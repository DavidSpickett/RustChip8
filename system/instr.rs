use system::Chip8System;
use system::InstrFlags;

extern crate rand;
use system::instr::rand::Rng;

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
    fn exec(&self, c8: &mut Chip8System);
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

#[allow(dead_code)]
pub struct UndefInstr {
    core: InstrCore,
    message: String,
}

#[allow(dead_code)]
impl UndefInstr {
    pub fn new(opc: u16, msg: String) -> UndefInstr {
        UndefInstr {
            core: InstrCore::new(opc, InstrFlags::_None),
            message: msg,
        }
    }
}

#[allow(dead_code)]
impl Instr for UndefInstr {
    fn repr(&self) -> String {
        format!("{}", self.message) 
    }

    fn exec(&self, _c8: &mut Chip8System) {}

    fn get_opcode(&self) -> u16 {
        self.core.opcode
    }

    fn get_flags(&self) -> InstrFlags {
        self.core.flags
    }
}

pub struct CallInstr {
    core: InstrCore,
    target: u16,
}

impl CallInstr {
    pub fn new(opc: u16) -> CallInstr {
        CallInstr {
            core: InstrCore::new(opc, InstrFlags::_None),
            target: op_to_nnn(opc),
        }
    }
}

impl Instr for CallInstr {
    fn repr(&self) -> String {
        format!("CALL 0x{:03x}", self.target)
    }

    fn exec(&self, c8: &mut Chip8System) {
        c8.stack_ptr += 1;
        c8.stack[c8.stack_ptr as usize] = c8.pc;
        c8.pc = op_to_nnn(self.core.opcode);
    }

    fn get_opcode(&self) -> u16 {
        self.core.opcode
    }

    fn get_flags(&self) -> InstrFlags {
        self.core.flags
    }
}

pub struct JumpInstr {
    core: InstrCore,
    target: u16,
}

impl JumpInstr {
    pub fn new(opc: u16) -> JumpInstr {
        JumpInstr {
            core: InstrCore::new(opc, InstrFlags::_None),
            target: op_to_nnn(opc),
        }
    }
}

impl Instr for JumpInstr {
    fn repr(&self) -> String {
        format!("JP 0x{:03x}", self.target)
    }

    fn exec(&self, c8: &mut Chip8System) {
        c8.pc = self.target;
    }

    fn get_opcode(&self) -> u16 {
        self.core.opcode
    }

    fn get_flags(&self) -> InstrFlags {
        self.core.flags
    }
}

pub struct RetInstr {
    core: InstrCore,
}

impl RetInstr {
    pub fn new(opc: u16) -> RetInstr {
        RetInstr {
            core: InstrCore::new(opc, InstrFlags::_None),
        }
    }
}

impl Instr for RetInstr {
    fn repr(&self) -> String {
        format!("RET")
    }

    fn exec(&self, c8: &mut Chip8System) {
        c8.pc = c8.stack[c8.stack_ptr as usize];
        c8.stack_ptr -= 1;
    }

    fn get_opcode(&self) -> u16 {
        self.core.opcode
    }

    fn get_flags(&self) -> InstrFlags {
        self.core.flags
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
            core: InstrCore::new(opc, InstrFlags::_None),
            vx: op_to_vx(opc),
            kk: op_to_kk(opc),
        }
    }
}

impl Instr for SkipEqualInstr {
    fn repr(&self) -> String {
        format!("SE V{}, 0x{:02x}", self.vx, self.kk)
    }

    fn exec(&self, c8: &mut Chip8System) {
        if c8.v_regs[self.vx as usize] == self.kk {
            c8.pc += 2;
        }
    }

    fn get_opcode(&self) -> u16 {
        self.core.opcode
    }

    fn get_flags(&self) -> InstrFlags {
        self.core.flags
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
            core: InstrCore::new(opc, InstrFlags::_None),
            vx: op_to_vx(opc),
            kk: op_to_kk(opc),
        }
    }
}

impl Instr for SkipNotEqualInstr {
    fn repr(&self) -> String {
        format!("SNE V{}, 0x{:02x}", self.vx, self.kk)
    }

    fn exec(&self, c8: &mut Chip8System) {
        if c8.v_regs[self.vx as usize] != self.kk {
            c8.pc +=2;
        }
    }

    fn get_opcode(&self) -> u16 {
        self.core.opcode
    }

    fn get_flags(&self) -> InstrFlags {
        self.core.flags
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
            core: InstrCore::new(opc, InstrFlags::_None),
            vx: op_to_vx(opc),
            kk: op_to_kk(opc),
        }
    }
}

impl Instr for LoadByteInstr {
    fn repr(&self) -> String {
        format!("LD V{}, 0x{:02x}", self.vx, self.kk)
    }

    fn exec(&self, c8: &mut Chip8System) {
        c8.v_regs[self.vx as usize] = self.kk;
    }

    fn get_opcode(&self) -> u16 {
        self.core.opcode
    }

    fn get_flags(&self) -> InstrFlags {
        self.core.flags
    }
}

pub struct ClearDisplayInstr {
    core: InstrCore,
}

impl ClearDisplayInstr {
    pub fn new(opc: u16) -> ClearDisplayInstr {
        ClearDisplayInstr {
            core: InstrCore::new(opc, InstrFlags::_None),
        }
    }
}

impl Instr for ClearDisplayInstr {
    fn repr(&self) -> String {
        String::from("CLS")
    }

    fn exec(&self, c8: &mut Chip8System) {
        for p in c8.screen.iter_mut() {
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

pub struct MovRegInstr {
    core: InstrCore,
    vx: u8,
    vy: u8,
}

impl MovRegInstr {
    pub fn new(opc: u16) -> MovRegInstr {
        MovRegInstr {
            core: InstrCore::new(opc, InstrFlags::_None),
            vx: op_to_vx(opc),
            vy: op_to_vy(opc),
        }
    }
}

impl Instr for MovRegInstr {
    fn repr(&self) -> String {
        format!("LD V{}, V{}", self.vx, self.vy)
    }

    fn exec(&self, c8: &mut Chip8System) {
        c8.v_regs[self.vx as usize] = c8.v_regs[self.vy as usize];
    }

    fn get_opcode(&self) -> u16 {
        self.core.opcode
    }

    fn get_flags(&self) -> InstrFlags {
        self.core.flags
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
            core: InstrCore::new(opc, InstrFlags::_None),
            vx: op_to_vx(opc),
            vy: op_to_vy(opc),
        }
    }
}

impl Instr for OrRegInstr {
    fn repr(&self) -> String {
        format!("OR V{}, V{}", self.vx, self.vy)
    }

    fn exec(&self, c8: &mut Chip8System) {
        c8.v_regs[self.vx as usize] |= c8.v_regs[self.vy as usize];
    }

    fn get_opcode(&self) -> u16 {
        self.core.opcode
    }

    fn get_flags(&self) -> InstrFlags {
        self.core.flags
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
            core: InstrCore::new(opc, InstrFlags::_None),
            vx: op_to_vx(opc),
            vy: op_to_vy(opc),
        }
    }
}

impl Instr for AndRegInstr {
    fn repr(&self) -> String {
        format!("AND V{}, V{}", self.vx, self.vy)
    }

    fn exec(&self, c8: &mut Chip8System) {
        c8.v_regs[self.vx as usize] &= c8.v_regs[self.vy as usize];
    }

    fn get_opcode(&self) -> u16 {
        self.core.opcode
    }

    fn get_flags(&self) -> InstrFlags {
        self.core.flags
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
            core: InstrCore::new(opc, InstrFlags::_None),
            vx: op_to_vx(opc),
            vy: op_to_vy(opc),
        }
    }
}

impl Instr for XORRegInstr {
    fn repr(&self) -> String {
        format!("XOR V{}, V{}", self.vx, self.vy)
    }

    fn exec(&self, c8: &mut Chip8System) {
        c8.v_regs[self.vx as usize] ^= c8.v_regs[self.vy as usize];
    }

    fn get_opcode(&self) -> u16 {
        self.core.opcode
    }

    fn get_flags(&self) -> InstrFlags {
        self.core.flags
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
            core: InstrCore::new(opc, InstrFlags::_None),
            vx: op_to_vx(opc),
            vy: op_to_vy(opc),
        }
    }
}

impl Instr for AddRegInstr {
    fn repr(&self) -> String {
        format!("ADD V{}, V{}", self.vx, self.vy)
    }

    fn exec(&self, c8: &mut Chip8System) {
        let x = c8.v_regs[self.vx as usize];
        let y = c8.v_regs[self.vy as usize];

        c8.v_regs[self.vx as usize].wrapping_add(y);

        if ((x as u16) + (y as u16)) > 0xFF {
            c8.v_regs[15] = 1;
            //TODO: clear carry if this doesn't happen?
        }
    }

    fn get_opcode(&self) -> u16 {
        self.core.opcode
    }

    fn get_flags(&self) -> InstrFlags {
        self.core.flags
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
            core: InstrCore::new(opc, InstrFlags::_None),
            vx: op_to_vx(opc),
            vy: op_to_vy(opc),
        }
    }
}

impl Instr for SubRegInstr {
    fn repr(&self) -> String {
        format!("SUB V{}, V{}", self.vx, self.vy)
    }

    fn exec(&self, c8: &mut Chip8System) {
        let x = c8.v_regs[self.vx as usize];
        let y = c8.v_regs[self.vy as usize];

        c8.v_regs[self.vx as usize].wrapping_sub(y);

        if x > y {
            c8.v_regs[15] = 1;
            //TODO: clear carry if this doesn't happen?
        }
    }

    fn get_opcode(&self) -> u16 {
        self.core.opcode
    }

    fn get_flags(&self) -> InstrFlags {
        self.core.flags
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
            core: InstrCore::new(opc, InstrFlags::_None),
            vx: op_to_vx(opc),
            vy: op_to_vy(opc),
        }
    }
}

impl Instr for SubNRegInstr {
    fn repr(&self) -> String {
        format!("SUB V{}, V{}", self.vx, self.vy)
    }

    fn exec(&self, c8: &mut Chip8System) {
        let x = c8.v_regs[self.vx as usize];
        let y = c8.v_regs[self.vy as usize];

        c8.v_regs[self.vx as usize] = y.wrapping_sub(x);

        if y > x {
            c8.v_regs[15] = 1;
            //TODO: clear carry if this doesn't happen?
        }
    }

    fn get_opcode(&self) -> u16 {
        self.core.opcode
    }

    fn get_flags(&self) -> InstrFlags {
        self.core.flags
    }
}

pub struct ShrRegInstr {
    core: InstrCore,
    vx: u8,
}

impl ShrRegInstr {
    pub fn new(opc: u16) -> ShrRegInstr {
        ShrRegInstr {
            core: InstrCore::new(opc, InstrFlags::_None),
            vx: op_to_vx(opc),
        }
    }
}

impl Instr for ShrRegInstr {
    fn repr(&self) -> String {
        format!("SHR V{}", self.vx)
    }

    fn exec(&self, c8: &mut Chip8System) {
        let x = c8.v_regs[self.vx as usize];
        c8.v_regs[15] = x & 1;
        c8.v_regs[self.vx as usize] /= 2;        
    }

    fn get_opcode(&self) -> u16 {
        self.core.opcode
    }

    fn get_flags(&self) -> InstrFlags {
        self.core.flags
    }
}

pub struct ShlNRegInstr {
    core: InstrCore,
    vx: u8,
}

impl ShlNRegInstr {
    pub fn new(opc: u16) -> ShlNRegInstr {
        ShlNRegInstr {
            core: InstrCore::new(opc, InstrFlags::_None),
            vx: op_to_vx(opc),
        }
    }
}

impl Instr for ShlNRegInstr {
    fn repr(&self) -> String {
        format!("SHL V{}", self.vx)
    }

    fn exec(&self, c8: &mut Chip8System) {
        let x = c8.v_regs[self.vx as usize];
        c8.v_regs[15] = x >> 7;
        c8.v_regs[self.vx as usize] *= 2;        
    }

    fn get_opcode(&self) -> u16 {
        self.core.opcode
    }

    fn get_flags(&self) -> InstrFlags {
        self.core.flags
    }
}

pub struct LoadIInstr {
    core: InstrCore,
    nnn: u16,
}

impl LoadIInstr {
    pub fn new(opc: u16) -> LoadIInstr {
        LoadIInstr {
            core: InstrCore::new(opc, InstrFlags::_None),
            nnn: op_to_nnn(opc),
        }
    }
}

impl Instr for LoadIInstr {
    fn repr(&self) -> String {
        format!("LD I, 0x{:03x}", self.nnn)
    }

    fn exec(&self, c8: &mut Chip8System) {
        c8.i_reg = self.nnn;
    }

    fn get_opcode(&self) -> u16 {
        self.core.opcode
    }

    fn get_flags(&self) -> InstrFlags {
        self.core.flags
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
            core: InstrCore::new(opc, InstrFlags::_None),
            vx: op_to_vx(opc),
            kk: op_to_kk(opc),
        }
    }
}

impl Instr for AddByteInstr {
    fn repr(&self) -> String {
        format!("ADD V{}, 0x{:02x}", self.vx, self.kk)
    }

    fn exec(&self, c8: &mut Chip8System) {
        c8.v_regs[self.vx as usize] = c8.v_regs[self.vx as usize].wrapping_add(self.kk)
    }

    fn get_opcode(&self) -> u16 {
        self.core.opcode
    }

    fn get_flags(&self) -> InstrFlags {
        self.core.flags
    }
}

pub struct AddIVInstr {
    core: InstrCore,
    vx: u8,
}

impl AddIVInstr {
    pub fn new(opc: u16) -> AddIVInstr {
        AddIVInstr {
            core: InstrCore::new(opc, InstrFlags::_None),
            vx: op_to_vx(opc),
        }
    }
}

impl Instr for AddIVInstr {
    fn repr(&self) -> String {
        format!("ADD I, V{}", self.vx)
    }

    fn exec(&self, c8: &mut Chip8System) {
        c8.i_reg += c8.v_regs[self.vx as usize] as u16
    }

    fn get_opcode(&self) -> u16 {
        self.core.opcode
    }

    fn get_flags(&self) -> InstrFlags {
        self.core.flags
    }
}

pub struct SetDelayTimerInstr {
    core: InstrCore,
    vx: u8,
}

impl SetDelayTimerInstr {
    pub fn new(opc: u16) -> SetDelayTimerInstr {
        SetDelayTimerInstr {
            core: InstrCore::new(opc, InstrFlags::_None),
            vx: op_to_vx(opc),
        }
    }
}

impl Instr for SetDelayTimerInstr {
    fn repr(&self) -> String {
        format!("LD DT, V{}", self.vx)
    }

    fn exec(&self, c8: &mut Chip8System) {
        c8.delay_timer = c8.v_regs[self.vx as usize]
    }

    fn get_opcode(&self) -> u16 {
        self.core.opcode
    }

    fn get_flags(&self) -> InstrFlags {
        self.core.flags
    }
}

pub struct GetDelayTimerInstr {
    core: InstrCore,
    vx: u8,
}

impl GetDelayTimerInstr {
    pub fn new(opc: u16) -> GetDelayTimerInstr {
        GetDelayTimerInstr {
            core: InstrCore::new(opc, InstrFlags::_None),
            vx: op_to_vx(opc),
        }
    }
}

impl Instr for GetDelayTimerInstr {
    fn repr(&self) -> String {
        format!("LD V{}, DT", self.vx)
    }

    fn exec(&self, c8: &mut Chip8System) {
        c8.v_regs[self.vx as usize] = c8.delay_timer
    }

    fn get_opcode(&self) -> u16 {
        self.core.opcode
    }

    fn get_flags(&self) -> InstrFlags {
        self.core.flags
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
            core: InstrCore::new(opc, InstrFlags::Screen),
            vx: op_to_vx(opc),
            vy: op_to_vy(opc),
            n: (opc & 0xF) as u8,
        }
    }
}

impl Instr for DrawSpriteInstr {
    fn repr(&self) -> String {
        format!("DRW V{}, V{}, {}", self.vx, self.vy, self.n)
    }

    fn exec(&self, c8: &mut Chip8System) {
        //Clear overlap flag
        c8.v_regs[15] = 0;

        let x = c8.v_regs[self.vx as usize];
        let y = c8.v_regs[self.vy as usize];
        let addr = c8.i_reg as usize;
        let sprite_data = &c8.memory[addr..addr+(self.n as usize)];

        for (y_offset, row) in sprite_data.iter().enumerate() {
            for sprite_x in (0..8).rev() {
                let final_x = ((x+7-sprite_x) % 64) as usize;
                let final_y = ((y as usize) + y_offset) % 32;
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

    fn get_opcode(&self) -> u16 {
        self.core.opcode
    }

    fn get_flags(&self) -> InstrFlags {
        self.core.flags
    }
}

pub struct SkipKeyIfPressedInstr {
    core: InstrCore,
    vx: u8,
}

impl SkipKeyIfPressedInstr {
    pub fn new(opc: u16) -> SkipKeyIfPressedInstr {
        SkipKeyIfPressedInstr {
            core: InstrCore::new(opc, InstrFlags::Keys),
            vx: op_to_vx(opc),
        }
    }
}

impl Instr for SkipKeyIfPressedInstr {
    fn repr(&self) -> String {
        format!("SKP V{}", self.vx)
    }

    fn exec(&self, c8: &mut Chip8System) {
        let key_num = c8.v_regs[self.vx as usize] as usize;
        if c8.keys[key_num] {
            c8.pc += 2;
        }
    }

    fn get_opcode(&self) -> u16 {
        self.core.opcode
    }

    fn get_flags(&self) -> InstrFlags {
        self.core.flags
    }
}

pub struct SkipKeyIfNotPressedInstr {
    core: InstrCore,
    vx: u8,
}

impl SkipKeyIfNotPressedInstr {
    pub fn new(opc: u16) -> SkipKeyIfNotPressedInstr {
        SkipKeyIfNotPressedInstr {
            core: InstrCore::new(opc, InstrFlags::Keys),
            vx: op_to_vx(opc),
        }
    }
}

impl Instr for SkipKeyIfNotPressedInstr {
    fn repr(&self) -> String {
        format!("SKNP V{}", self.vx)
    }

    fn exec(&self, c8: &mut Chip8System) {
        let key_num = c8.v_regs[self.vx as usize] as usize;
        if !c8.keys[key_num] {
            c8.pc += 2;
        }
    }

    fn get_opcode(&self) -> u16 {
        self.core.opcode
    }

    fn get_flags(&self) -> InstrFlags {
        self.core.flags
    }
}

pub struct ReadRegsFromMemInstr {
    core: InstrCore,
    vx: u8,
}

impl ReadRegsFromMemInstr {
    pub fn new(opc: u16) -> ReadRegsFromMemInstr {
        ReadRegsFromMemInstr {
            core: InstrCore::new(opc, InstrFlags::_None),
            vx: op_to_vx(opc),
        }
    }
}

impl Instr for ReadRegsFromMemInstr {
    fn repr(&self) -> String {
        format!("LD V{}, [I]", self.vx)
    }

    fn exec(&self, c8: &mut Chip8System) {
        let addr = c8.i_reg as usize;
        for reg_idx in 0..(self.vx+1) {
            c8.v_regs[reg_idx as usize] = c8.memory[addr];
        }
    }

    fn get_opcode(&self) -> u16 {
        self.core.opcode
    }

    fn get_flags(&self) -> InstrFlags {
        self.core.flags
    }
}

pub struct SetSoundTimerInstr {
    core: InstrCore,
    vx: u8,
}

impl SetSoundTimerInstr {
    pub fn new(opc: u16) -> SetSoundTimerInstr {
        SetSoundTimerInstr {
            core: InstrCore::new(opc, InstrFlags::Sound),
            vx: op_to_vx(opc),
        }
    }
}

impl Instr for SetSoundTimerInstr {
    fn repr(&self) -> String {
        format!("LD ST, V{}", self.vx)
    }

    fn exec(&self, c8: &mut Chip8System) {
        c8.sound_timer = c8.v_regs[self.vx as usize];
    }

    fn get_opcode(&self) -> u16 {
        self.core.opcode
    }

    fn get_flags(&self) -> InstrFlags {
        self.core.flags
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
            core: InstrCore::new(opc, InstrFlags::Sound),
            vx: op_to_vx(opc),
            kk: op_to_kk(opc),
        }
    }
}

impl Instr for RandomInstr {
    fn repr(&self) -> String {
        format!("RND V{}, 0x{:02x}", self.vx, self.kk)
    }

    fn exec(&self, c8: &mut Chip8System) {
        let mut rng = rand::thread_rng();
        c8.v_regs[self.vx as usize] = self.kk & rng.gen::<u8>();
    }

    fn get_opcode(&self) -> u16 {
        self.core.opcode
    }

    fn get_flags(&self) -> InstrFlags {
        self.core.flags
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
            core: InstrCore::new(opc, InstrFlags::_None),
            vx: op_to_vx(opc),
            vy: op_to_vy(opc),
        }
    }
}

impl Instr for SkipIfRegsEqualInstr {
    fn repr(&self) -> String {
        format!("SE V{}, V{}", self.vx, self.vy)
    }

    fn exec(&self, c8: &mut Chip8System) {
        if c8.v_regs[self.vx as usize] == c8.v_regs[self.vy as usize] {
            c8.pc += 2;
        }
    }

    fn get_opcode(&self) -> u16 {
        self.core.opcode
    }

    fn get_flags(&self) -> InstrFlags {
        self.core.flags
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
            core: InstrCore::new(opc, InstrFlags::_None),
            vx: op_to_vx(opc),
            vy: op_to_vy(opc),
        }
    }
}

impl Instr for SkipIfRegsNotEqualInstr {
    fn repr(&self) -> String {
        format!("SNE V{}, V{}", self.vx, self.vy)
    }

    fn exec(&self, c8: &mut Chip8System) {
        if c8.v_regs[self.vx as usize] != c8.v_regs[self.vy as usize] {
            c8.pc += 2;
        }
    }

    fn get_opcode(&self) -> u16 {
        self.core.opcode
    }

    fn get_flags(&self) -> InstrFlags {
        self.core.flags
    }
}

pub struct JumpPlusVZeroInstr {
    core: InstrCore,
    target: u16,
}

impl JumpPlusVZeroInstr {
    pub fn new(opc: u16) -> JumpPlusVZeroInstr {
        JumpPlusVZeroInstr {
            core: InstrCore::new(opc, InstrFlags::_None),
            target: op_to_nnn(opc),
        }
    }
}

impl Instr for JumpPlusVZeroInstr {
    fn repr(&self) -> String {
        format!("JP V0, 0x{:03x}", self.target)
    }

    fn exec(&self, c8: &mut Chip8System) {
        c8.pc = self.target + (c8.v_regs[0] as u16);
    }

    fn get_opcode(&self) -> u16 {
        self.core.opcode
    }

    fn get_flags(&self) -> InstrFlags {
        self.core.flags
    }
}

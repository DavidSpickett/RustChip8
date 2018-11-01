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
        let mut ret = self.get_mnemonic().to_string();
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
    fn get_symbol(&self) -> Option<String>;
    fn resolve_symbol(&mut self, addr: u16);
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

macro_rules! impl_instr_base {
    () => (
        fn get_flags(&self) -> InstrFlags { self.core.flags}
        fn get_mnemonic(&self) -> &String { &self.core.mnemonic }
    )
}

macro_rules! impl_instr {
    () => (
        impl_instr_base!();
        fn get_opcode(&self) -> u16 { self.core.opcode }
        fn get_symbol(&self) -> Option<String> { None }
        fn resolve_symbol(&mut self, _addr: u16) {
            panic!("Can't resolve symbol for instruction without an address!");
        }
    )
}

macro_rules! format_no_args {
    () => (
        fn get_formatted_args(&self) -> String {
            "".to_string()
        }
    )
}

macro_rules! format_x_args {
    () => (
        fn get_formatted_args(&self) -> String {
            format!("V{}", self.vx)
        }
    )
}

macro_rules! format_reg_x_args {
    ($reg:expr) => (
        fn get_formatted_args(&self) -> String {
            format!("{}, V{}", $reg, self.vx)
        }
    )
}

macro_rules! format_x_reg_args {
    ($reg:expr) => (
        fn get_formatted_args(&self) -> String {
            format!("V{}, {}", self.vx, $reg)
        }
    )
}

fn make_nnn_format() -> impl Fn(&AddressOrSymbol) -> String {
    | nnn: &AddressOrSymbol | {
        match *nnn {
            AddressOrSymbol::Address(a) => format!("0x{:03X}", a),
            AddressOrSymbol::Symbol(ref s) => s.to_string(),
        }
    }
}

pub enum AddressOrSymbol {
    Address(u16),
    Symbol(String),
}

macro_rules! instr_symbol {
    ( $instr_name:ident, $mnemonic:expr, $flags:path, $base:expr,
      $exec:expr, $formatter:expr ) => (
        pub struct $instr_name {
            core: InstrCore,
            nnn: AddressOrSymbol,
        }

        impl $instr_name {
            pub fn new(opc: u16) -> $instr_name {
                $instr_name {
                    core: InstrCore::new(opc, $flags, $mnemonic),
                    nnn: AddressOrSymbol::Address(op_to_nnn(opc)),
                }
            }

            pub fn create(target: u16) -> $instr_name {
                $instr_name::new(instr_builder::arg_nnn($base, target))
            }

            pub fn create_with_symbol(sym: String) -> $instr_name {
                let mut i = $instr_name::create(0);
                i.nnn = AddressOrSymbol::Symbol(sym);
                i
            }
            
            fn get_addr(&self) -> u16 {
                match self.nnn {
                    AddressOrSymbol::Address(a) => a,
                    AddressOrSymbol::Symbol(ref s) => panic!("Cannot get address for unresolved symbol \"{}\"", s),
                }
            }
        }

        impl Instr for $instr_name {
            impl_instr_base!();

            // We call a closure because we can't access self directly
            fn get_formatted_args(&self) -> String {
                $formatter(&self.nnn)
            }

            fn get_opcode(&self) -> u16 {
                let _ = self.get_addr();
                self.core.opcode
            }

            fn get_symbol(&self) -> Option<String> {
                match self.nnn {
                    AddressOrSymbol::Symbol(ref s) => Some(s.to_string()),
                    AddressOrSymbol::Address(_) => None,
                }
            }

            fn resolve_symbol(&mut self, addr: u16) {
                match self.nnn {
                    AddressOrSymbol::Symbol(_) => {
                        self.nnn = AddressOrSymbol::Address(addr);
                        // Update stored encoding
                        self.core.opcode |= 0x0FFF & addr;
                    }
                    AddressOrSymbol::Address(_) => panic!("Symbol already resolved for this instruction!"),
                }
            }

            fn exec(&self, c8: &mut Chip8System) {
                $exec(self.get_addr(), c8);
            }
        }
    )
}

macro_rules! instr_no_args {
    ( $instr_name:ident, $mnemonic:expr, $flags:path, $base:expr ) => (
        pub struct $instr_name {
            core: InstrCore,
        }

        impl $instr_name {
            pub fn new(opc: u16) -> $instr_name {
                $instr_name {
                    core: InstrCore::new(opc, $flags, $mnemonic),
                }
            }

            pub fn create() -> $instr_name {
                $instr_name::new(instr_builder::no_args($base))
            }
        }
    )
}

macro_rules! instr_x_kk {
    ( $instr_name:ident, $mnemonic:expr, $flags:path,
      $base:expr, $exec:expr ) => (
        pub struct $instr_name {
            core: InstrCore,
            vx: u8,
            kk: u8,
        }

        impl $instr_name {
            pub fn new(opc: u16) -> $instr_name {
                $instr_name {
                    core: InstrCore::new(opc, $flags, $mnemonic),
                    vx: op_to_vx(opc),
                    kk: op_to_kk(opc),
                }
            }

            pub fn create(x: u8, kk: u8) -> $instr_name {
                $instr_name::new(instr_builder::arg_x_kk($base, x, kk))
            }
        }

        impl Instr for $instr_name {
            impl_instr!();

            fn get_formatted_args(&self) -> String {
                format!("V{}, 0x{:02X}", self.vx, self.kk)
            }

            fn exec(&self, c8: &mut Chip8System) {
                $exec(c8, self.vx, self.kk);
            }
        }
    )
}

macro_rules! instr_x_y {
    ( $instr_name:ident, $mnemonic:expr, $flags:path,
      $base:expr, $exec:expr ) => (
        pub struct $instr_name {
            core: InstrCore,
            vx: u8,
            vy: u8,
        }

        impl $instr_name {
            pub fn new(opc: u16) -> $instr_name {
                $instr_name {
                    core: InstrCore::new(opc, $flags, $mnemonic),
                    vx: op_to_vx(opc),
                    vy: op_to_vy(opc),
                }
            }

            pub fn create(x: u8, y: u8) -> $instr_name {
                $instr_name::new(instr_builder::arg_x_y($base, x, y))
            }
        }

        impl Instr for $instr_name {
            impl_instr!();

            fn get_formatted_args(&self) -> String {
                format!("V{}, V{}", self.vx, self.vy)
            }

            fn exec(&self, c8: &mut Chip8System) {
                $exec(c8, self.vx, self.vy);
            }
        }
    )
}

macro_rules! instr_x {
    ( $instr_name:ident, $mnemonic:expr, $flags:path, $base:expr ) => (
        pub struct $instr_name {
            core: InstrCore,
            vx: u8,
        }

        impl $instr_name {
            pub fn new(opc: u16) -> $instr_name {
                $instr_name {
                    core: InstrCore::new(opc, $flags, $mnemonic),
                    vx: op_to_vx(opc),
                }
            }

            pub fn create(x: u8) -> $instr_name {
                $instr_name::new(instr_builder::arg_x($base, x))
            }
        }
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

pub struct WordInstr {
    core: InstrCore,
}

impl WordInstr {
    pub fn new(opc: u16) -> WordInstr {
        WordInstr {
            core: InstrCore::new(opc, InstrFlags::_None, ".word"),
        }
    }

    pub fn create(word: u16) -> WordInstr {
        WordInstr::new(word)
    }
}

impl Instr for WordInstr {
    impl_instr!();

    fn get_formatted_args(&self) -> String {
        format!("0x{:04X}", self.core.opcode)
    }

    fn exec(&self, _c8: &mut Chip8System) {
        panic!("Cannot execute a .word pseudo instruction!")
    }
}

instr_symbol!(SysInstr, "SYS", InstrFlags::_None, 0x0000,
| _addr, _c8 | {}, make_nnn_format());

instr_symbol!(CallInstr, "CALL", InstrFlags::_None, 0x2000,
| addr, c8: &mut Chip8System | {
    if c8.stack.len() == 16 {
        panic!("Stack is full!")
    }

    c8.stack.push(c8.pc);
    c8.pc = addr; 
}, make_nnn_format());

instr_symbol!(JumpInstr, "JP", InstrFlags::_None, 0x1000,
| addr, c8: &mut Chip8System | {
    c8.pc = addr;
}, make_nnn_format());

instr_no_args!(RetInstr, "RET", InstrFlags::_None, 0x00EE);
impl Instr for RetInstr {
    impl_instr!();
    format_no_args!();

    fn exec(&self, c8: &mut Chip8System) {
        if c8.stack.is_empty() {
            panic!("Stack is empty!");
        }
        c8.pc = c8.stack.pop().unwrap();
    }
}

instr_x_kk!(SkipEqualInstr, "SE", InstrFlags::_None, 0x3000,
| c8: &mut Chip8System, vx, kk | {
    if c8.v_regs[vx as usize] == kk {
            c8.pc += 2;
    }
});

instr_x_kk!(SkipNotEqualInstr, "SNE", InstrFlags::_None, 0x4000,
| c8: &mut Chip8System, vx, kk | {
    if c8.v_regs[vx as usize] != kk {
            c8.pc +=2;
    }
});

instr_x_kk!(LoadByteInstr, "LD", InstrFlags::_None, 0x6000,
| c8: &mut Chip8System, vx, kk | {
    c8.v_regs[vx as usize] = kk;
});

instr_no_args!(ClearDisplayInstr, "CLS", InstrFlags::Screen, 0x00E0);
impl Instr for ClearDisplayInstr {
    impl_instr!();
    format_no_args!();

    fn exec(&self, c8: &mut Chip8System) {
        for p in c8.screen.iter_mut() {
            *p = false;
        }
    }
}

instr_x_y!(MovRegInstr, "LD", InstrFlags::_None, 0x8000,
| c8: &mut Chip8System, vx, vy | {
    c8.v_regs[vx as usize] = c8.v_regs[vy as usize];
});

instr_x_y!(OrRegInstr, "OR", InstrFlags::_None, 0x8001,
| c8: &mut Chip8System, vx, vy | {
    c8.v_regs[vx as usize] |= c8.v_regs[vy as usize];
});

instr_x_y!(AndRegInstr, "AND", InstrFlags::_None, 0x8002,
| c8: &mut Chip8System, vx, vy | {
    c8.v_regs[vx as usize] &= c8.v_regs[vy as usize];
});

instr_x_y!(XORRegInstr, "XOR", InstrFlags::_None, 0x8003,
| c8: &mut Chip8System, vx, vy | {
    c8.v_regs[vx as usize] ^= c8.v_regs[vy as usize];
});

instr_x_y!(AddRegInstr, "ADD", InstrFlags::_None, 0x8004,
| c8: &mut Chip8System, vx, vy | {
    let x = c8.v_regs[vx as usize];
    let y = c8.v_regs[vy as usize];

    c8.v_regs[vx as usize] = c8.v_regs[vx as usize].wrapping_add(y);

    if (u16::from(x) + u16::from(y)) > 0xFF {
        c8.v_regs[15] = 1;
    } else {
        c8.v_regs[15] = 0;
    }
});

instr_x_y!(SubRegInstr, "SUB", InstrFlags::_None, 0x8005,
| c8: &mut Chip8System, vx, vy | {
    let x = c8.v_regs[vx as usize];
    let y = c8.v_regs[vy as usize];

    c8.v_regs[vx as usize] = x.wrapping_sub(y);
    c8.v_regs[15] = (x>y) as u8;
});

instr_x_y!(SubNRegInstr, "SUBN", InstrFlags::_None, 0x8007,
| c8: &mut Chip8System, vx, vy | {
    let x = c8.v_regs[vx as usize];
    let y = c8.v_regs[vy as usize];

    c8.v_regs[vx as usize] = y.wrapping_sub(x);
    c8.v_regs[15] = (y>x) as u8;
});

instr_x!(ShrRegInstr, "SHR", InstrFlags::_None, 0x8006);
impl Instr for ShrRegInstr {
    impl_instr!();
    format_x_args!();

    fn exec(&self, c8: &mut Chip8System) {
        let x = c8.v_regs[self.vx as usize];
        c8.v_regs[15] = x & 1;
        c8.v_regs[self.vx as usize] >>= 1;
    }
}

instr_x!(ShlRegInstr, "SHL", InstrFlags::_None, 0x800E);
impl Instr for ShlRegInstr {
    impl_instr!();
    format_x_args!();

    fn exec(&self, c8: &mut Chip8System) {
        let x = c8.v_regs[self.vx as usize];
        c8.v_regs[15] = x >> 7;
        c8.v_regs[self.vx as usize] <<= 1;
    }
}

instr_symbol!(LoadIInstr, "LD", InstrFlags::_None, 0xA000,
| addr, c8: &mut Chip8System | {
    c8.i_reg = addr;
},
| nnn: &AddressOrSymbol | {
    match nnn {
        AddressOrSymbol::Address(a) => format!("I, 0x{:03X}", a),
        AddressOrSymbol::Symbol(ref s) => format!("I, {}", s),
    }
});

instr_x_kk!(AddByteInstr, "ADD", InstrFlags::_None, 0x7000,
| c8: &mut Chip8System, vx, kk | {
    c8.v_regs[vx as usize] = c8.v_regs[vx as usize].wrapping_add(kk)
});

instr_x!(AddIVInstr, "ADD", InstrFlags::_None, 0xF01E);
impl Instr for AddIVInstr {
    impl_instr!();
    format_reg_x_args!("I");

    fn exec(&self, c8: &mut Chip8System) {
        c8.i_reg = c8.i_reg.wrapping_add(u16::from(c8.v_regs[self.vx as usize]))
    }
}

instr_x!(SetDelayTimerInstr, "LD", InstrFlags::_None, 0xF015);
impl Instr for SetDelayTimerInstr {
    impl_instr!();
    format_reg_x_args!("DT");

    fn exec(&self, c8: &mut Chip8System) {
        c8.delay_timer = c8.v_regs[self.vx as usize]
    }
}

instr_x!(GetDelayTimerInstr, "LD", InstrFlags::_None, 0xF007);
impl Instr for GetDelayTimerInstr {
    impl_instr!();
    format_x_reg_args!("DT");

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

instr_x!(SkipKeyIfPressedInstr, "SKP", InstrFlags::Keys, 0xE09E);
impl Instr for SkipKeyIfPressedInstr {
    impl_instr!();
    format_x_args!();

    fn exec(&self, c8: &mut Chip8System) {
        if c8.get_keystate(c8.v_regs[self.vx as usize]) {
            c8.pc += 2;
        }
    }
}

instr_x!(SkipKeyIfNotPressedInstr, "SKNP", InstrFlags::Keys, 0xE0A1);
impl Instr for SkipKeyIfNotPressedInstr {
    impl_instr!();
    format_x_args!();

    fn exec(&self, c8: &mut Chip8System) {
        if !c8.get_keystate(c8.v_regs[self.vx as usize]) {
            c8.pc += 2;
        }
    }
}

instr_x!(ReadRegsFromMemInstr, "LD", InstrFlags::_None, 0xF065);
impl Instr for ReadRegsFromMemInstr {
    impl_instr!();
    format_x_reg_args!("[I]");

    fn exec(&self, c8: &mut Chip8System) {
        let addr = c8.bounds_check_i(self.vx+1);
        for reg_idx in 0..(self.vx+1) {
            c8.v_regs[reg_idx as usize] = c8.memory[addr+(reg_idx as usize)];
        }
    }
}

instr_x!(WriteRegsToMemInstr, "LD", InstrFlags::_None, 0xF055);
impl Instr for WriteRegsToMemInstr {
    impl_instr!();
    format_reg_x_args!("[I]");

    fn exec(&self, c8: &mut Chip8System) {
        let addr = c8.bounds_check_i(self.vx+1);
        for reg_idx in 0..(self.vx+1) {
            c8.memory[addr+(reg_idx as usize)] = c8.v_regs[reg_idx as usize];
        }
    }
}

instr_x!(SetSoundTimerInstr, "LD", InstrFlags::Sound, 0xF018);
impl Instr for SetSoundTimerInstr {
    impl_instr!();
    format_reg_x_args!("ST");

    fn exec(&self, c8: &mut Chip8System) {
        c8.sound_timer = c8.v_regs[self.vx as usize];
    }
}

instr_x_kk!(RandomInstr, "RND", InstrFlags::_None, 0xC000,
| c8: &mut Chip8System, vx, kk | {
    let mut rng = rand::thread_rng();
    c8.v_regs[vx as usize] = kk & rng.gen::<u8>();
});

instr_x_y!(SkipIfRegsEqualInstr, "SE", InstrFlags::_None, 0x5000,
| c8: &mut Chip8System, vx, vy | {
    if c8.v_regs[vx as usize] == c8.v_regs[vy as usize] {
        c8.pc += 2;
    }
});

instr_x_y!(SkipIfRegsNotEqualInstr, "SNE", InstrFlags::_None, 0x9000,
| c8: &mut Chip8System, vx, vy | {
    if c8.v_regs[vx as usize] != c8.v_regs[vy as usize] {
        c8.pc += 2;
    }
});

instr_symbol!(JumpPlusVZeroInstr, "JP", InstrFlags::_None, 0xB000,
| addr, c8: &mut Chip8System | {
    c8.pc = addr + u16::from(c8.v_regs[0]);
},
| nnn: &AddressOrSymbol | {
    match nnn {
        AddressOrSymbol::Address(a) => format!("V0, 0x{:03X}", a),
        AddressOrSymbol::Symbol(ref s) => format!("V0, {}", s),
    }
});

instr_x!(GetDigitAddrInstr, "LD", InstrFlags::_None, 0xF029);
impl Instr for GetDigitAddrInstr {
    impl_instr!();
    format_reg_x_args!("F");

    fn exec(&self, c8: &mut Chip8System) {
        let digit = u16::from(c8.v_regs[self.vx as usize]);
        c8.i_reg = digit*5;
    }
}

instr_x!(StoreBCDInstr, "LD", InstrFlags::_None, 0xF033);
impl Instr for StoreBCDInstr {
    impl_instr!();
    format_reg_x_args!("B");

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

instr_x!(WaitForKeyInstr, "LD", InstrFlags::WaitKey, 0xF00A);
impl Instr for WaitForKeyInstr {
    impl_instr!();
    format_x_reg_args!("K");

    fn exec(&self, c8: &mut Chip8System) {
        c8.v_regs[self.vx as usize] = c8.pressed_key as u8;
    }
}

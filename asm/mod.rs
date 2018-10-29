use system::instr::*;
use std::collections::{HashMap, HashSet};
mod test;

struct AsmError {
    line_no: usize,
    line: String,
    msg: String,
    char_no: usize,
    len: usize,
}

impl AsmError {
    fn new(line_no: usize, line: String, msg: String,
           char_no: usize, len: usize) -> AsmError {
        AsmError { line_no, line, msg, char_no, len}
    }
}

#[cfg(test)]
pub fn parse_asm_str(asm: &str) -> Result<Vec<Box<Instr>>, String> {
    let mut warnings: Vec<String> = vec![];
    parse_asm(asm, &"<str>".to_string(), &mut warnings)
}

#[cfg(test)]
fn parse_asm_str_with_warnings(asm: &str, warnings: &mut Vec<String>)
    -> Result<Vec<Box<Instr>>, String> {
    parse_asm(asm, &"<str>".to_string(), warnings)
}

pub fn parse_asm(asm: &str, filename: &str, warnings: &mut Vec<String>) -> Result<Vec<Box<Instr>>, String> {
    let mut instrs: Vec<Box<Instr>> = vec![];
    let mut symbols: HashMap<String, u16> = HashMap::new();
    let mut addr: u16 = 0x0200;
    let mut errs: Vec<AsmError> = vec![];

    for (line_no, line) in asm.lines().enumerate() {
        match parse_line(&line, &mut symbols, addr) {
            Err(err) => errs.push(AsmError::new(
                    line_no, line.to_string(),
                    err.msg, err.pos, err.len)),
            Ok(mut i) => {
                addr += 2*(i.len() as u16);
                instrs.append(&mut i);
            },
        }
    }

    // Patch up symbol addresses
    let mut resolved_syms = HashSet::new();
    for ins in &mut instrs {
        if let Some(sym) = ins.get_symbol() {
            match symbols.get(&sym) {
                Some(addr) => {
                    ins.resolve_symbol(*addr);
                    let _ = resolved_syms.insert(sym);
                }
                None => {
                    errs.push(AsmError::new(
                        //TODO: line info for these
                        0, "".to_string(), format!("Could not resolve symbol \"{}\"", sym),
                        0, 1));
                },
            }
        }
    }

    // Check for unused labels
    for (sym, _) in symbols {
        if !resolved_syms.contains(&sym) {
            warnings.push(format!("{}: warning: Unused label \"{}\"", filename, sym));
        }
    }

    if !errs.is_empty() {
        let mut err_msg = String::from("");
        for (i, err) in errs.iter().enumerate() {
            if i != 0 {
                err_msg.push('\n');
            }
            err_msg += &format!("{}:{}:{}: error: {}", filename, err.line_no, err.char_no, err.msg);
            let pointer = String::from(" ").repeat(err.char_no);
            let len = match err.len {
                // Mark whole line
                0 => err.line.len()-err.char_no,
                // Only section
                _ => err.len,
            };
            // -1 because the '^' is there too
            let extent = String::from("~").repeat(len-1);
            err_msg += &format!("\n{}\n{}^{}", err.line, pointer, extent);
        }
        // In future we might want to keep these seperate
        return Err(err_msg);
    }

    Ok(instrs)
}


#[derive(PartialEq, Debug)]
struct AsmArg {
    s: String,
    upper: String,
    pos: usize,
}

impl AsmArg {
    fn new(s: String, pos: usize) -> AsmArg {
        AsmArg{
            upper: s.to_uppercase(),
            s,
            pos,
        }
    }

    fn str_cmp(&self, other: &str) -> bool {
        self.s == other
    }

    fn len(&self) -> usize {
        self.s.len()
    }
}

fn split_asm_line(line: &str) -> Vec<AsmArg> {
    let mut start = 0;
    let mut part = String::from("");
    let mut parts: Vec<AsmArg> = vec![];
    let terminators = [' ', '\t', ','];

    for (idx, c) in line.chars().enumerate() {
        let is_terminator = terminators.contains(&c);
        let is_last = idx == line.len()-1;

        if !is_terminator {
            if part.is_empty() {
                start = idx;
            }
            part.push(c);
        }

        if (is_terminator || is_last) && !part.is_empty() {
            parts.push(AsmArg::new(part.to_owned(), start));
            part.clear();
        }
    }

    parts
}

#[derive(Debug)]
struct ErrInfo {
    msg: String,
    pos: usize,
    len: usize,
}

impl ErrInfo {
    fn new(msg: String, pos: usize, len: usize) -> ErrInfo {
        ErrInfo { msg, pos, len }
    }
}

fn parse_line(line: &str,
              symbols: &mut HashMap<String, u16>,
              current_addr: u16)
                -> Result<Vec<Box<Instr>>, ErrInfo> {
    // This function will add new symbols to the map and return an
    // instruction object if one was required.
    // That object may have an unresolved symbol in it, parse_asm
    // will take care of that.
    let mut instrs: Vec<Box<Instr>> = vec![];

    let comment_chars = "//";
    let mut no_comments_line = line;
    if let Some(idx) = no_comments_line.find(comment_chars) {
        no_comments_line = no_comments_line.split_at(idx).0;
    }

    let mut args = split_asm_line(no_comments_line);

    // Lines consisting of only whitespace
    if args.is_empty() {
        return Ok(instrs);
    }

    let mnemonic = args.remove(0);

    // Check for labels
    if args.is_empty() && mnemonic.s.ends_with(':') {
        // Add a symbol for this address
        let sym_name = mnemonic.s[..mnemonic.len()-1].to_string();
        if let Some(_) = symbols.insert(sym_name, current_addr) {
            return Err(ErrInfo::new(
                "Label repeated".to_string(),
                mnemonic.pos, mnemonic.len()));
        };
        return Ok(instrs);
    }

    if mnemonic.upper == "JP" {
        // JP can have one or two args
        if args.is_empty() || (args.len() > 2) {
            return Err(ErrInfo::new( 
                format!("Expected 1 or 2 args for JP instruction, got {}", args.len()),
                mnemonic.pos, 0));
        }
    } else {
        match check_num_args(&mnemonic, args.len()) {
            Ok(_) => {},
            Err(e) => return Err(e),
        }
    }

    match get_args_type(&mnemonic) {
        ArgsType::Custom => {
            match mnemonic.upper.as_str() {
                // No arguments
                "CLS"   => instrs.push(Box::new(ClearDisplayInstr::create())),
                "RET"   => instrs.push(Box::new(RetInstr::create())),
                "BRK"   => instrs.push(Box::new(SysInstr::create(0xFFF))),
                // Single argument
                ".WORD" => instrs.push(Box::new(WordInstr::create(
                            parse_extended_addr(&args[0]).unwrap()))),
                "SYS"   => {
                    match parse_nnn_or_symbol(&args[0]) {
                        AddressOrSymbol::Symbol(s) => {
                            instrs.push(Box::new(SysInstr::create_with_symbol(s)));
                        }
                        AddressOrSymbol::Address(a) => {
                            instrs.push(Box::new(SysInstr::create(a)));
                        }
                    }
                }
                "JP"   => {
                    if args.len() == 2 {
                        // Use the parser here to allow different formatting
                        if parse_vx(&args[0]).unwrap() != 0 {
                            return Err(ErrInfo::new(
                                    "Jump plus instruction can only use V0!".to_string(),
                                    args[0].pos, args[0].len()));
                        }

                        // JP V0, addr so use the 2nd arg
                        match parse_nnn_or_symbol(&args[1]) {
                            AddressOrSymbol::Symbol(s) => {
                                instrs.push(Box::new(JumpPlusVZeroInstr::create_with_symbol(s)));
                            }
                            AddressOrSymbol::Address(a) => {
                                instrs.push(Box::new(JumpPlusVZeroInstr::create(a)));
                            }
                        }
                    } else {
                        //Usual JP addr
                        match parse_nnn_or_symbol(&args[0]) {
                            AddressOrSymbol::Symbol(s) => {
                                instrs.push(Box::new(JumpInstr::create_with_symbol(s)));
                            }
                            AddressOrSymbol::Address(a) => {
                                instrs.push(Box::new(JumpInstr::create(a)));
                            }
                        }
                    }
                }
                "CALL" => {
                    match parse_nnn_or_symbol(&args[0]) {
                        AddressOrSymbol::Symbol(s) => {
                            instrs.push(Box::new(CallInstr::create_with_symbol(s)));
                        }
                        AddressOrSymbol::Address(a) => {
                            instrs.push(Box::new(CallInstr::create(a)));
                        }
                    }
                }

                // Two arguments
                "RND"    => {
                    match parse_vx(&args[0]) {
                        Err(e) => return Err(e),
                        Ok(v) => match parse_xx(&args[1]) {
                            Err(e) => return Err(e),
                            Ok(b) => instrs.push(Box::new(RandomInstr::create(v, b))),
                        }
                    }
                }
                "SE"     => {
                    let vx = parse_vx(&args[0]).unwrap();
                    // Byte or register versions
                    if let Ok(a) = parse_vx(&args[1]) {
                        instrs.push(Box::new(SkipIfRegsEqualInstr::create(vx, a)))
                    } else if let Ok(a) = parse_xx(&args[1]) {
                        instrs.push(Box::new(SkipEqualInstr::create(vx, a)))
                    } else {
                        return Err(ErrInfo::new(
                                "Invalid argument 2 for SE instruction".to_string(),
                                args[1].pos, args[1].len()));
                    }
                },

                "SNE"   => {
                    let vx = parse_vx(&args[0]).unwrap();
                    // Byte or register versions
                    if let Ok(a) = parse_vx(&args[1]) {
                        instrs.push(Box::new(SkipIfRegsNotEqualInstr::create(vx, a)))
                    } else if let Ok(a) = parse_xx(&args[1]) {
                        instrs.push(Box::new(SkipNotEqualInstr::create(vx, a)))
                    } else {
                        return Err(ErrInfo::new(
                                "Invalid argument 2 for SNE instruction".to_string(),
                                args[1].pos, args[1].len()));
                    }
                }

                "ADD"   => {
                    if let Ok(a) = parse_vx(&args[0]) {
                        // Vx, byte
                        if let Ok(b) = parse_vx(&args[1]) {
                            instrs.push(Box::new(AddRegInstr::create(a, b)));
                        // Vx, Vy
                        } else if let Ok(b) = parse_xx(&args[1]) {
                            instrs.push(Box::new(AddByteInstr::create(a, b)));
                        } else {
                            return Err(ErrInfo::new(
                                    "Invalid arguments for ADD instruction".to_string(),
                                    args[1].pos, 0));
                        }
                    // I, Vx
                    } else if args[0].str_cmp("I") {
                        match parse_vx(&args[1]) {
                            Err(e) => return Err(e),
                            Ok(v) => instrs.push(Box::new(AddIVInstr::create(v))),
                        };
                    } else {
                        return Err(ErrInfo::new(
                                "Invalid args for ADD instruction".to_string(),
                                args[0].pos, 0));
                    }
                }

                "LD"    => {
                    if let Ok(a) = parse_vx(&args[0]) {
                        if let Ok(b) = parse_xx(&args[1]) {
                            // LD V, byte
                            instrs.push(Box::new(LoadByteInstr::create(a, b)));
                        } else if let Ok(b) = parse_vx(&args[1]) {
                            // LD V, V
                            instrs.push(Box::new(MovRegInstr::create(a, b)));
                        } else if args[1].str_cmp("DT") {
                            // LD V, DT
                            instrs.push(Box::new(GetDelayTimerInstr::create(a)));
                        } else if args[1].str_cmp("K") {
                            // LD V, K
                            instrs.push(Box::new(WaitForKeyInstr::create(a)));
                        } else if args[1].str_cmp("[I]") {
                            // LD V, [I]
                            instrs.push(Box::new(ReadRegsFromMemInstr::create(a)));
                        } else {
                            return Err(ErrInfo::new(
                                    "Invalid args to LD instruction".to_string(),
                                    args[0].pos, 0));
                        }
                    } else if args[0].str_cmp("I") {
                        // Special 16 bit address sequence
                        if let Ok(addr) = parse_extended_addr(&args[1]) {
                            if addr <= 0xFFF {
                                instrs.push(Box::new(LoadIInstr::create(addr)));
                            } else {
                                // We're going to change I anyway so we can trash it
                                let rest_of_addr = addr - 0xFFF;
                                instrs.push(Box::new(LoadIInstr::create(0xFFF)));

                                // Number of ADD I, Vx we have to do with 0xFF
                                // Can't think of another way other than reserving a register here
                                let regnum: u8 = 14;
                                let max_regval: u16 = 0xFF;
                                let num_adds = (rest_of_addr / max_regval) as u8;
                                // Remainder value for the last ADD
                                let remainder = (rest_of_addr % max_regval) as u8;

                                if num_adds != 0 {
                                    instrs.push(Box::new(LoadByteInstr::create(regnum, max_regval as u8)));
                                    for _ in 0..num_adds {
                                        instrs.push(Box::new(AddIVInstr::create(regnum)));
                                    }
                                }

                                if remainder != 0 {
                                    instrs.push(Box::new(LoadByteInstr::create(regnum, remainder)));
                                    instrs.push(Box::new(AddIVInstr::create(regnum)));
                                }

                                /* TADA! You just loaded a 16 bit address into I
                                   but gave up a register temporarily to do it.

                                   The reason you can't save/restore is as follows:
                                   - Set I to some location (font memory/high addr?)
                                   - Save V0 to memory
                                   - Do stuff with it to get I to the high address
                                   - Then set I back to the saved V0 location
                                   ....

                                   Which defeats the point of this whole silly exercise.
                                   Also restoring the memory you save to is tricky.
                                */
                            }
                        } else {
                            // LD I, nnn
                            // Using the *2nd* argument!
                            match parse_nnn_or_symbol(&args[1]) {
                                AddressOrSymbol::Symbol(s) => {
                                    instrs.push(Box::new(LoadIInstr::create_with_symbol(s)));
                                }
                                AddressOrSymbol::Address(a) => {
                                    instrs.push(Box::new(LoadIInstr::create(a)));
                                }
                            }
                        }
                    } else if args[0].str_cmp("DT") {
                        // LD DT, V
                        instrs.push(Box::new(SetDelayTimerInstr::create(parse_vx(&args[1]).unwrap())));
                    } else if args[0].str_cmp("ST") {
                        // LD ST, V
                        instrs.push(Box::new(SetSoundTimerInstr::create(parse_vx(&args[1]).unwrap())));
                    } else if args[0].str_cmp("F") {
                        // LD F, V
                        instrs.push(Box::new(GetDigitAddrInstr::create(parse_vx(&args[1]).unwrap())));
                    } else if args[0].str_cmp("B") {
                        // LD B, V
                        instrs.push(Box::new(StoreBCDInstr::create(parse_vx(&args[1]).unwrap())));
                    } else if args[0].str_cmp("[I]") {
                        // LD [I], V
                        instrs.push(Box::new(WriteRegsToMemInstr::create(parse_vx(&args[1]).unwrap())));
                    } else {
                        return Err(ErrInfo::new(
                                "Invalid args to LD instruction".to_string(),
                                args[0].pos, 0));
                    }
                }

                // Only draw has 3
                "DRW"   => instrs.push(Box::new(DrawSpriteInstr::create(
                            parse_vx(&args[0]).unwrap(),
                            parse_vx(&args[1]).unwrap(),
                            parse_n(&args[2]).unwrap()))),
                _ => return Err(ErrInfo::new(
                        format!("Unrecognised mnemonic: {}", mnemonic.s),
                        mnemonic.pos, mnemonic.len())),
            }
        }
        ArgsType::VX => {
            let x = match parse_vx(&args[0]) {
                Err(e) => return Err(e),
                Ok(v) => v,
            };

            match mnemonic.upper.as_str() {
                "SHR"   => instrs.push(Box::new(ShrRegInstr::create(x))),
                "SHL"   => instrs.push(Box::new(ShlRegInstr::create(x))),
                "SKP"   => instrs.push(Box::new(SkipKeyIfPressedInstr::create(x))),
                "SKNP"  => instrs.push(Box::new(SkipKeyIfNotPressedInstr::create(x))),
                _ => panic!("Unknown mnemonic {} with VX args", mnemonic.s),
            }
        }
        ArgsType::VXVY => {
            let x = match parse_vx(&args[0]) {
                Err(e) => return Err(e),
                Ok(v) => v,
            };

            let y = match parse_vx(&args[1]) {
                Err(e) => return Err(e),
                Ok(v) => v,
            };

            match mnemonic.upper.as_str() {
                "OR"    => instrs.push(Box::new(OrRegInstr::create(x, y))),
                "XOR"    => instrs.push(Box::new(XORRegInstr::create(x, y))),
                "AND"    => instrs.push(Box::new(AndRegInstr::create(x, y))),
                "SUB"    => instrs.push(Box::new(SubRegInstr::create(x, y))),
                "SUBN"   => instrs.push(Box::new(SubNRegInstr::create(x, y))),
                _ => panic!("Unknown mnemonic {} with VXVY args", mnemonic.s),
            }
        }
    }

    Ok(instrs)
}

enum ArgsType {
    Custom,
    VX,
    VXVY,
}

fn get_args_type(mnemonic: &AsmArg) -> ArgsType {
    match mnemonic.upper.as_str() {
        "SHR" | "SHL" | "SKP" | "SKNP" => ArgsType::VX,
        "OR" | "XOR" | "AND" | "SUB" | "SUBN" => ArgsType::VXVY,
        _ => ArgsType::Custom,
    }
}

fn check_num_args(mnemonic: &AsmArg, num: usize) -> Result<usize, ErrInfo> {
    let expected: usize = match &mnemonic.upper[..] {
        "CLS" | "RET" | "BRK" => 0,
        "SYS" | "CALL" | "SHR" | "SHL" | "SKP" | "SKNP" | ".WORD" => 1,
        // Some variants of LD only have 1 variable arg, but for asm
        // purposes they all have two
        "LD" | "ADD" | "SE" | "SNE" | "OR" | "AND" | "XOR" | "SUB" | "SUBN" | "RND" => 2,
        "DRW" => 3,
        _ => return Err(ErrInfo::new(
                format!("Unrecognised mnemonic: {}", mnemonic.s),
                mnemonic.pos, mnemonic.len())),
    };
    if expected != num {
        return Err(ErrInfo::new(
                format!("Expected {} args for {}, got {}", expected, mnemonic.s, num),
                mnemonic.pos, mnemonic.len()));
    }
    Ok(num)
}

fn parse_vx(arg: &AsmArg) -> Result<u8, ErrInfo> {
    let c1 = arg.s.chars().nth(0).unwrap();
    if (c1 != 'V') && (c1 != 'v') {
        return Err(ErrInfo::new(
                "VX arg does not begin with \"V\"".to_string(),
                arg.pos, arg.len()));
    }

    let num = &arg.s[1..];
    let idx: u8;

    match num.parse::<u8>() {
        Err(_) => {
            match u8::from_str_radix(&arg.s[1..], 16) {
                Err(_) => return Err(
                    ErrInfo::new(format!("Invalid V register: \"{}\"", arg.s),
                    arg.pos, arg.len())),
                Ok(v) => idx = v,
            }
        }
        Ok(v) => idx = v,
    }

    if idx > 0xF {
        return Err(ErrInfo::new(
                "V register index cannot be > 0xF".to_string(),
                arg.pos, arg.len()));
    }

    Ok(idx)
}

fn parse_hex(arg: &AsmArg) -> Result<u16, ErrInfo> {
    if arg.len() < 2 {
        return Err(ErrInfo::new("Arg too short to be a hex number".to_string(), arg.pos, arg.len()));
    }
    if &arg.s[..2] != "0x" {
        return Err(ErrInfo::new("Hex number must start with \"0x\"".to_string(), arg.pos, arg.len()));
    }
    match u16::from_str_radix(&arg.s[2..], 16) {
        Err(e) => Err(ErrInfo::new(format!("Invalid hex number: {}", e.to_string()), arg.pos, arg.len())),
        Ok(v) => Ok(v), 
    }
}

fn parse_xx(arg: &AsmArg) -> Result<u8, ErrInfo> {
    let v = match parse_hex(arg) {
        Err(_) => match arg.s.parse::<u16>() {
                Err(_) => return Err(ErrInfo::new(
                        "Inalid byte argument".to_string(),
                        arg.pos, arg.len())),
                Ok(v) => v,
        },
        Ok(v) => v,
    };

    // TODO: some out of range error, as opposed to not recognised
    if v > 0xff {
        return Err(ErrInfo::new("Byte argument larger than 0xFF".to_string(),
                    arg.pos, arg.len()));
    }
    Ok(v as u8)
}

fn parse_nnn(arg: &AsmArg) -> Result<u16, ErrInfo> {
    match parse_hex(arg) {
        Err(e) => Err(e),
        Ok(v) => {
            if v > 0xfff {
                return Err(ErrInfo::new(
                        "Address argument larger than 0xFFF".to_string(),
                        arg.pos, arg.len()));
            }
            Ok(v)
        }
    }
}

fn parse_extended_addr(arg: &AsmArg) -> Result<u16, ErrInfo> {
    match parse_hex(arg) {
        Err(e) => Err(e),
        Ok(v) => Ok(v),
    }
}

fn parse_nnn_or_symbol(arg: &AsmArg) -> AddressOrSymbol {
    match parse_nnn(arg) {
        Ok(v) => AddressOrSymbol::Address(v),
        // Try to lookup anything else as a symbol
        Err(_) => AddressOrSymbol::Symbol(arg.s.to_owned()),
    }
}

fn parse_n(arg: &AsmArg) -> Result<u8, ErrInfo> {
    match arg.s.parse::<u8>() {
        Err(msg) => Err(ErrInfo::new(msg.to_string(), arg.pos, arg.len())),
        Ok(v) => {
            if v > 15 {
                Err(ErrInfo::new("Nibble must be < 16".to_string(), arg.pos, arg.len()))
            } else {
                Ok(v)
            }
        }
    }
}

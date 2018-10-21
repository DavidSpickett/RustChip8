use system::instr::*;
use std::collections::HashMap;
mod test;

struct AsmError {
    line_no: usize,
    line: String,
    msg: String,
    char_no: usize,
}

impl AsmError {
    fn new(line_no: usize, line: String, msg: String, char_no: usize) -> AsmError {
        AsmError { line_no, line, msg, char_no}
    }
}

pub fn parse_asm(asm: &String) -> Result<Vec<Box<Instr>>, String> {
    let mut instrs: Vec<Box<Instr>> = vec![];
    let mut symbols: HashMap<String, u16> = HashMap::new();
    let mut addr: u16 = 0x0200;
    let mut errs: Vec<AsmError> = vec![];

    for (line_no, line) in asm.lines().enumerate() {
        match parse_line(&line, &mut symbols, addr) {
            Err((msg, pos)) => errs.push(AsmError::new(
                    line_no, line.to_string(),
                    msg, pos)),
            Ok(mut i) => {
                addr += 2*(i.len() as u16);
                instrs.append(&mut i);
            },
        }
    }

    if !errs.is_empty() {
        let mut err_msg = format!("Assembly failed with {} errors.\n", errs.len());
        for err in errs {
            let line_no_fmt = format!("{}:", err.line_no);
            let pointer = String::from(" ").repeat(line_no_fmt.len() + err.char_no + 1);
            err_msg += &format!("\n{} {}\n{}^\n{}", line_no_fmt, err.line, pointer, err.msg);
        }
        // In future we might want to keep these seperate
        return Err(err_msg);
    }

    // Patch up symbol addresses
    for ins in instrs.iter_mut() {
        if let Some(sym) = ins.get_symbol() {
            match symbols.get(&sym) {
                Some(addr) => ins.resolve_symbol(*addr),
                // TODO: add these to the result somehow
                None => panic!("Could not resolve symbol \"{}\"", sym),
            }
        }
    }

    Ok(instrs)
}


#[derive(PartialEq, Debug)]
struct AsmArg {
    s: String,
    pos: usize,
}

impl AsmArg {
    fn new(s: String, pos: usize) -> AsmArg {
        AsmArg{s, pos}
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

        if is_terminator || is_last {
            // TODO: this logic is a bit tortuous
            if is_last && !is_terminator {
                part.push(c);
            }

            if !part.is_empty() {
                parts.push(AsmArg::new(part.to_owned(), start));
                part.clear();
            }
        } else {
            if part.is_empty() {
                start = idx;
            }
            part.push(c);
        }
    }

    parts
}

pub fn parse_line(line: &str,
                  symbols: &mut HashMap<String, u16>, 
                  current_addr: u16)
                    -> Result<Vec<Box<Instr>>, (String, usize)> {
    // This function will add new symbols to the map and return an
    // instruction object if one was required.
    // That object may have an unresolved symbol in it, parse_asm
    // will take care of that.
    split_asm_line(line);
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

    let mut mnemonic = args.remove(0);

    // Check for labels
    if args.is_empty() {
        if mnemonic.s.ends_with(":") {
            // Add a symbol for this address
            symbols.insert(mnemonic.s[..mnemonic.len()-1].to_string(), current_addr);
            return Ok(instrs);
        }
    }

    // Now we know it's not a label we can normalise the case
    mnemonic.s = mnemonic.s.to_uppercase();

    if mnemonic.s == "JP" {
        // JP can have one or two args
        if (args.len() == 0) || (args.len() > 2) {
            return Err((
                format!("Expected 1 or 2 args for JP instruction, got {}", args.len()),
                mnemonic.pos));
        }
    } else {
        match check_num_args(&mnemonic, args.len()) {
            Ok(_) => {},
            Err((msg, pos)) => return Err((msg, pos)),
        }
    }

    match mnemonic.s.as_str() {
        // No arguments
        "CLS"   => instrs.push(Box::new(ClearDisplayInstr::create())),
        "RET"   => instrs.push(Box::new(RetInstr::create())),
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
                    return Err((
                            format!("Jump plus instruction can only use V0!"),
                            args[0].pos));
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
        "SHR"   => instrs.push(Box::new(ShrRegInstr::create(parse_vx(&args[0]).unwrap()))),
        "SHL"   => instrs.push(Box::new(ShlRegInstr::create(parse_vx(&args[0]).unwrap()))),
        "SKP"   => instrs.push(Box::new(SkipKeyIfPressedInstr::create(parse_vx(&args[0]).unwrap()))),
        "SKNP"  => instrs.push(Box::new(SkipKeyIfNotPressedInstr::create(parse_vx(&args[0]).unwrap()))),

        // Two arguments
        "OR"    => instrs.push(Box::new(OrRegInstr::create(
                    parse_vx(&args[0]).unwrap(),
                    parse_vx(&args[1]).unwrap()))),
        "XOR"    => instrs.push(Box::new(XORRegInstr::create(
                    parse_vx(&args[0]).unwrap(),
                    parse_vx(&args[1]).unwrap()))),
        "AND"    => instrs.push(Box::new(AndRegInstr::create(
                    parse_vx(&args[0]).unwrap(),
                    parse_vx(&args[1]).unwrap()))),
        "SUB"    => instrs.push(Box::new(SubRegInstr::create(
                    parse_vx(&args[0]).unwrap(),
                    parse_vx(&args[1]).unwrap()))),
        "SUBN"   => instrs.push(Box::new(SubNRegInstr::create(
                    parse_vx(&args[0]).unwrap(),
                    parse_vx(&args[1]).unwrap()))),
        "RND"    => instrs.push(Box::new(RandomInstr::create(
                    parse_vx(&args[0]).unwrap(),
                    parse_xx(&args[1]).unwrap()))),

        "SE"     => {
            let vx = parse_vx(&args[0]).unwrap();
            // Byte or register versions
            if let Ok(a) = parse_vx(&args[1]) {
                instrs.push(Box::new(SkipIfRegsEqualInstr::create(vx, a)))
            } else if let Ok(a) = parse_xx(&args[1]) {
                instrs.push(Box::new(SkipEqualInstr::create(vx, a)))
            } else {
                return Err((
                        format!("Invalid argument 2 for SE instruction"),
                        args[1].pos));
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
                return Err((
                        format!("Invalid argument 2 for SNE instruction"),
                        args[1].pos));
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
                    return Err((
                            format!("Invalid arguments for ADD instruction"),
                            args[1].pos));
                }
            // I, Vx
            } else if args[0].str_cmp("I") {
                instrs.push(Box::new(AddIVInstr::create(parse_vx(&args[1]).unwrap())));
            } else {
                return Err((
                        format!("Invalid args for ADD instruction"),
                        args[1].pos));
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
                    return Err((
                            format!("Invalid args to LD instruction"),
                            args[1].pos));
                }
            } else if args[0].str_cmp("I") {
                // Special 16 bit address sequence
                if let Ok(addr) = parse_extended_addr(&args[1]) {
                    // TODO: this check should go elsewhere, checking number
                    // of digits isn't a great way to go
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
                return Err((
                        format!("Invalid args to LD instruction"),
                        args[0].pos));
            }
        }

        // Only draw has 3
        "DRW"   => instrs.push(Box::new(DrawSpriteInstr::create(
                    parse_vx(&args[0]).unwrap(),
                    parse_vx(&args[1]).unwrap(),
                    parse_n(&args[2]).unwrap()))),
        //TODO: this will print it normalised, not as you typed it
        _ => return Err((
                format!("Unrecognised mnemonic: {}", mnemonic.s),
                mnemonic.pos)),
    }

    Ok(instrs)
}

fn check_num_args(mnemonic: &AsmArg, num: usize) -> Result<usize, (String, usize)> {
    let expected: usize = match &mnemonic.s[..] {
        "CLS" | "RET" => 0,
        "SYS" | "CALL" | "SHR" | "SHL" | "SKP" | "SKNP" | ".WORD" => 1,
        // Some variants of LD only have 1 variable arg, but for asm
        // purposes they all have two
        "LD" | "ADD" | "SE" | "SNE" | "OR" | "AND" | "XOR" | "SUB" | "SUBN" | "RND" => 2,
        "DRW" => 3,
        _ => return Err((
                format!("Can't get number of args for mnemonic: {}", mnemonic.s),
                mnemonic.pos)),
    };
    if expected != num {
        return Err((
                format!("Expected {} args for {}, got {}", expected, mnemonic.s, num),
                mnemonic.pos));
    }
    Ok(num)
}

fn parse_vx(arg: &AsmArg) -> Result<u8, (String, usize)> {
    let c1 = arg.s.chars().nth(0).unwrap();
    if (c1 != 'V') && (c1 != 'v') {
        return Err(("Does not begin with \"V\"".to_string(), arg.pos)); 
    }

    let num = &arg.s[1..];
    let idx: u8;

    match num.parse::<u8>() {
        Err(_) => {
            match u8::from_str_radix(&arg.s[1..], 16) {
                Err(_) => return Err((format!("Invalid V register: \"{}\"", arg.s), arg.pos)),
                Ok(v) => idx = v,
            }
        }
        Ok(v) => idx = v,
    }

    if idx > 0xF {
        return Err(("V register index cannot be > 0xF".to_string(), arg.pos));
    }

    Ok(idx)
}

fn parse_hex(arg: &AsmArg) -> Result<u16, (&str, usize)> {
    if arg.len() < 2 {
        return Err(("Arg too short to be a hex number", arg.pos));
    }
    if &arg.s[..2] != "0x" {
        return Err(("Hex number must start with \"0x\"", arg.pos));
    }
    match u16::from_str_radix(&arg.s[2..], 16) {
        //TODO: we're masking range errors here
        Err(_) => Err(("Invalid hex number", arg.pos)),
        Ok(v) => Ok(v), 
    }
}

fn parse_xx(arg: &AsmArg) -> Result<u8, (&str, usize)> {
    match parse_hex(arg) {
        Err((msg, _)) => Err((msg, arg.pos)),
        Ok(v) => {
            if v > 0xff {
                return Err(("Byte argument larger than 0xFF", arg.pos));
            }
            Ok(v as u8)
        }
    }
}

fn parse_nnn(arg: &AsmArg) -> Result<u16, (&str, usize)> {
    match parse_hex(arg) {
        Err((msg, _)) => Err((msg, arg.pos)),
        Ok(v) => {
            if v > 0xfff {
                return Err(("Address argument larger than 0xFFF", arg.pos));
            }
            Ok(v)
        }
    }
}

fn parse_extended_addr(arg: &AsmArg) -> Result<u16, (&str, usize)> {
    match parse_hex(arg) {
        Err((msg, _)) => Err((msg, arg.pos)),
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

fn parse_n(arg: &AsmArg) -> Result<u8, (String, usize)> {
    match arg.s.parse::<u8>() {
        Err(msg) => Err((msg.to_string(), arg.pos)),
        Ok(v) => {
            if v > 15 {
                return Err(("Nibble must be < 16".to_string(), arg.pos));
            } else {
                return Ok(v);
            }
        }
    }
}

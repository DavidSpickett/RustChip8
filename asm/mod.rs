use system::instr::*;
use std::collections::HashMap;
mod test;

pub fn parse_asm(asm: &String) -> Vec<Box<Instr>> {
    let mut instrs: Vec<Box<Instr>> = vec![];
    let mut symbols: HashMap<String, u16> = HashMap::new();
    let mut addr: u16 = 0x0200;

    for line in asm.lines() {
        let mut new_instrs = parse_line(&line, &mut symbols, addr);
        addr += 2*(new_instrs.len() as u16);
        instrs.append(&mut new_instrs);
    }

    // Patch up symbol addresses
    for ins in instrs.iter_mut() {
        if let Some(sym) = ins.get_symbol() {
            match symbols.get(&sym) {
                Some(addr) => ins.resolve_symbol(*addr),
                None => panic!("Could not resolve symbol \"{}\"", sym),
            }
        }
    }

    instrs
}

pub fn parse_line(line: &str, symbols: &mut HashMap<String, u16>, current_addr: u16) -> Vec<Box<Instr>> {
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

    let parts = no_comments_line.split_whitespace();
    let mut args = parts.map(|x| x.replace(",", "")).collect::<Vec<String>>();

    // Lines consisting of only whitespace
    if args.is_empty() {
        return instrs;
    }

    let mnemonic = args.remove(0);

    // Check for labels
    if args.is_empty() {
        if mnemonic.ends_with(":") {
            // Add a symbol for this address
            symbols.insert(mnemonic[..mnemonic.len()-1].to_string(), current_addr);
            return instrs;
        }
    }

    if mnemonic == "JP" {
        // JP can have one or two args
        if (args.len() == 0) || (args.len() > 2) {
            panic!("Expected 1 or 2 args for JP instruction, got {}", args.len());
        }
    } else {
        check_num_args(&mnemonic, args.len());
    }

    match mnemonic.as_str() {
        // No arguments
        "CLS"   => instrs.push(Box::new(ClearDisplayInstr::create())),
        "RET"   => instrs.push(Box::new(RetInstr::create())),
        // Single argument
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
                if args[0] != "V0" {
                    panic!("Jump plus instruction can only use V0!");
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
                panic!("Invalid argument 2 for SE instruction");
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
                panic!("Invalid argument 2 for SNE instruction");
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
                    panic!("Invalid arguments for ADD instruction");
                }
            // I, Vx
            } else if args[0] == "I" {
                instrs.push(Box::new(AddIVInstr::create(parse_vx(&args[1]).unwrap())));
            } else {
                panic!("Invalid args to ADD instruction");
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
                } else if args[1] == "DT" {
                    // LD V, DT
                    instrs.push(Box::new(GetDelayTimerInstr::create(a)));
                } else if args[1] == "K" {
                    // LD V, K
                    instrs.push(Box::new(WaitForKeyInstr::create(a)));
                } else if args[1] == "[I]" {
                    // LD V, [I]
                    instrs.push(Box::new(ReadRegsFromMemInstr::create(a)));
                } else {
                    panic!("Invalid args to LD instruction");
                }
            } else if args[0] == "I" {
                // Special 16 bit address sequence
                if let Ok(addr) = parse_extended_addr(&args[1]) {
                    // TODO: this check should go elsewhere, checking number
                    // of digits isn't a great way to go
                    if addr <= 0xFFF {
                        instrs.push(Box::new(LoadIInstr::create(addr)));
                    }

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
            } else if args[0] == "DT" {
                // LD DT, V
                instrs.push(Box::new(SetDelayTimerInstr::create(parse_vx(&args[1]).unwrap())));
            } else if args[0] == "ST" {
                // LD ST, V
                instrs.push(Box::new(SetSoundTimerInstr::create(parse_vx(&args[1]).unwrap())));
            } else if args[0] == "F" {
                // LD F, V
                instrs.push(Box::new(GetDigitAddrInstr::create(parse_vx(&args[1]).unwrap())));
            } else if args[0] == "B" {
                // LD B, V
                instrs.push(Box::new(StoreBCDInstr::create(parse_vx(&args[1]).unwrap())));
            } else if args[0] == "[I]" {
                // LD [I], V
                instrs.push(Box::new(WriteRegsToMemInstr::create(parse_vx(&args[1]).unwrap())));
            } else {
                panic!("Invalid args to LD instruction");
            }
        }

        // Only draw has 3
        "DRW"   => instrs.push(Box::new(DrawSpriteInstr::create(
                    parse_vx(&args[0]).unwrap(),
                    parse_vx(&args[1]).unwrap(),
                    parse_n(&args[2]).unwrap()))),
        _ => panic!("Unrecognised mnemonic: {}", mnemonic),
    }

    instrs
}

fn check_num_args(mnemonic: &str, num: usize) {
    let expected = match mnemonic {
        "CLS" | "RET" => 0,
        "SYS" | "CALL" | "SHR" | "SHL" | "SKP" | "SKNP" => 1,
        // Some variants of LD only have 1 variable arg, but for asm
        // purposes they all have two
        "LD" | "ADD" | "SE" | "SNE" | "OR" | "AND" | "XOR" | "SUB" | "SUBN" | "RND" => 2,
        "DRW" => 3,
        _ => panic!("Can't verify number of args for mnemonic: {}", mnemonic),
    };
    if expected != num {
        panic!("Expected {} args for {}, got {}", expected, mnemonic, num);
    }
}

fn parse_vx(arg: &String) -> Result<u8, String> {
    if arg.chars().nth(0).unwrap() != 'V' {
        return Err("Does not begin with \"V\"".to_string()); 
    }
    let num = &arg[1..];
    match num.parse::<u8>() {
        Err(msg) => Err(format!("Invalid index {}: {}", num, msg)),
        Ok(v) => {
            if v > 15 {
                return Err("V register number must be < 16".to_string());
            } else
            {
                return Ok(v);
            }
        }
    }
}

fn parse_xx(arg: &String) -> Result<u8, String> {
    if arg.len() < 2 {
        return Err("Arg too short to be a byte".to_string());
    }
    if &arg[..2] != "0x" {
        return Err("Byte must start with \"0x\"".to_string());
    }
    if arg.len() != 4 {
        return Err("Byte must be 2 hex chars".to_string());
    }
    match u8::from_str_radix(&arg[2..], 16) {
        Err(_) => Err("Invalid hex number".to_string()),
        Ok(v) => Ok(v),
    }
}

fn parse_nnn(arg: &String) -> Result<u16, String> {
    if arg.len() < 5 {
        return Err("Arg is too short to be an address.".to_string());
    }
    if &arg[..2] != "0x" {
        return Err("Address must start with \"0x\"".to_string());
    }
    if arg.len() != 5 {
        return Err("Address must be 3 hex chars".to_string());
    }
    match u16::from_str_radix(&arg[2..], 16) {
        Err(_) => Err("Invalid hex number".to_string()),
        Ok(v) => Ok(v),
    }
}

fn parse_extended_addr(arg: &String) -> Result<u16, String> {
    // Basically a 16 bit hex number
    if arg.len() != 6 {
        return Err("Incorrect length for extended address.".to_string());
    }
    if &arg[..2] != "0x" {
        return Err("Extended address must start with \"0x\"".to_string());
    }
    match u16::from_str_radix(&arg[2..], 16) {
        Err(_) => Err("Invalid hex number for extended address.".to_string()),
        Ok(v) => Ok(v),
    }
}


fn parse_nnn_or_symbol(arg: &String) -> AddressOrSymbol {
    match parse_nnn(arg) {
        Ok(v) => AddressOrSymbol::Address(v),
        // Try to lookup anything else as a symbol
        Err(_) => AddressOrSymbol::Symbol(arg.to_owned()),
    }
}

fn parse_n(arg: &String) -> Result<u8, String> {
    match arg.parse::<u8>() {
        Err(msg) => Err(msg.to_string()),
        Ok(v) => {
            if v > 15 {
                return Err("Nibble must be < 16".to_string());
            } else {
                return Ok(v);
            }
        }
    }
}

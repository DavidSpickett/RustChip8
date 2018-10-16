use system::instr::*;
mod test;

pub fn parse_asm(asm: &String) -> Vec<Box<Instr>> {
    let mut instrs: Vec<Box<Instr>> = vec![];

    for line in asm.lines() {
        match parse_line(&line) {
            Some(i) => instrs.push(i),
            None => {} // Blank lines
        }
    }

    instrs
}

pub fn parse_line(line: &str) -> Option<Box<Instr>> {
    let mut instr: Option<Box<Instr>> = None;

    let comment_chars = "//";
    let mut no_comments_line = line;
    if let Some(idx) = no_comments_line.find(comment_chars) {
        no_comments_line = no_comments_line.split_at(idx).0;
    }

    let parts = no_comments_line.split_whitespace();
    let mut args = parts.map(|x| x.replace(",", "")).collect::<Vec<String>>();

    // Lines consisting of only whitespace
    if args.is_empty() {
        return instr;
    }

    let mnemonic = args.remove(0);

    check_num_args(&mnemonic, args.len());

    match mnemonic.as_str() {
        // No arguments
        "CLS"   => instr = Some(Box::new(ClearDisplayInstr::create())),
        "RET"   => instr = Some(Box::new(RetInstr::create())),
        // Single argument
        "SYS"   => instr = Some(Box::new(SysInstr::create(parse_nnn(&args[0]).unwrap()))),
        "JP"    => instr = Some(Box::new(JumpInstr::create(parse_nnn(&args[0]).unwrap()))),
        "CALL"  => instr = Some(Box::new(CallInstr::create(parse_nnn(&args[0]).unwrap()))),
        "SHR"   => instr = Some(Box::new(ShrRegInstr::create(parse_vx(&args[0]).unwrap()))),
        "SHL"   => instr = Some(Box::new(ShlRegInstr::create(parse_vx(&args[0]).unwrap()))),
        "SKP"   => instr = Some(Box::new(SkipKeyIfPressedInstr::create(parse_vx(&args[0]).unwrap()))),
        "SKNP"  => instr = Some(Box::new(SkipKeyIfNotPressedInstr::create(parse_vx(&args[0]).unwrap()))),

        // Two arguments
        "OR"    => instr = Some(Box::new(OrRegInstr::create(
                    parse_vx(&args[0]).unwrap(),
                    parse_vx(&args[1]).unwrap()))),
        "XOR"    => instr = Some(Box::new(XORRegInstr::create(
                    parse_vx(&args[0]).unwrap(),
                    parse_vx(&args[1]).unwrap()))),
        "AND"    => instr = Some(Box::new(AndRegInstr::create(
                    parse_vx(&args[0]).unwrap(),
                    parse_vx(&args[1]).unwrap()))),
        "SUB"    => instr = Some(Box::new(SubRegInstr::create(
                    parse_vx(&args[0]).unwrap(),
                    parse_vx(&args[1]).unwrap()))),
        "SUBN"   => instr = Some(Box::new(SubNRegInstr::create(
                    parse_vx(&args[0]).unwrap(),
                    parse_vx(&args[1]).unwrap()))),
        "RND"    => instr = Some(Box::new(RandomInstr::create(
                    parse_vx(&args[0]).unwrap(),
                    parse_xx(&args[1]).unwrap()))),

        "SE"     => {
            let vx = parse_vx(&args[0]).unwrap();
            // Byte or register versions
            if let Ok(a) = parse_vx(&args[1]) {
                instr = Some(Box::new(SkipIfRegsEqualInstr::create(vx, a)))
            } else if let Ok(a) = parse_xx(&args[1]) {
                instr = Some(Box::new(SkipEqualInstr::create(vx, a)))
            } else {
                panic!("Invalid argument 2 for SE instruction");
            }
        },

        "SNE"   => {
            let vx = parse_vx(&args[0]).unwrap();
            // Byte or register versions
            if let Ok(a) = parse_vx(&args[1]) {
                instr = Some(Box::new(SkipIfRegsNotEqualInstr::create(vx, a)))
            } else if let Ok(a) = parse_xx(&args[1]) {
                instr = Some(Box::new(SkipNotEqualInstr::create(vx, a)))
            } else {
                panic!("Invalid argument 2 for SNE instruction");
            }
        }

        "ADD"   => {
            if let Ok(a) = parse_vx(&args[0]) {
                // Vx, byte
                if let Ok(b) = parse_vx(&args[1]) {
                    instr = Some(Box::new(AddRegInstr::create(a, b)));
                // Vx, Vy
                } else if let Ok(b) = parse_xx(&args[1]) {
                    instr = Some(Box::new(AddByteInstr::create(a, b)));
                } else {
                    panic!("Invalid arguments for ADD instruction");
                }
            // I, Vx
            } else if args[0] == "I" {
                instr = Some(Box::new(AddIVInstr::create(parse_vx(&args[1]).unwrap())));
            } else {
                panic!("Invalid args to ADD instruction");
            }
        }

        "LD"    => {
            if let Ok(a) = parse_vx(&args[0]) {
                if let Ok(b) = parse_xx(&args[1]) {
                    // LD V, byte
                    instr = Some(Box::new(LoadByteInstr::create(a, b)));
                } else if let Ok(b) = parse_vx(&args[1]) {
                    // LD V, V
                    instr = Some(Box::new(MovRegInstr::create(a, b)));
                } else if args[1] == "DT" {
                    // LD V, DT
                    instr = Some(Box::new(GetDelayTimerInstr::create(a)));
                } else if args[1] == "K" {
                    // LD V, K
                    instr = Some(Box::new(WaitForKeyInstr::create(a)));
                } else if args[1] == "[I]" {
                    // LD V, [I]
                    instr = Some(Box::new(ReadRegsFromMemInstr::create(a)));
                } else {
                    panic!("Invalid args to LD instruction");
                }
            } else if args[0] == "I" {
                // LD I, nnn
                instr = Some(Box::new(LoadIInstr::create(parse_nnn(&args[1]).unwrap())));
            } else if args[0] == "DT" {
                // LD DT, V
                instr = Some(Box::new(SetDelayTimerInstr::create(parse_vx(&args[1]).unwrap())));
            } else if args[0] == "ST" {
                // LD ST, V
                instr = Some(Box::new(SetSoundTimerInstr::create(parse_vx(&args[1]).unwrap())));
            } else if args[0] == "F" {
                // LD F, V
                instr = Some(Box::new(GetDigitAddrInstr::create(parse_vx(&args[1]).unwrap())));
            } else if args[0] == "B" {
                // LD B, V
                instr = Some(Box::new(StoreBCDInstr::create(parse_vx(&args[1]).unwrap())));
            } else if args[0] == "[I]" {
                // LD [I], V
                instr = Some(Box::new(WriteRegsToMemInstr::create(parse_vx(&args[1]).unwrap())));
            } else {
                panic!("Invalid args to LD instruction");
            }
        }

        // Only draw has 3
        "DRW"   => instr = Some(Box::new(DrawSpriteInstr::create(
                    parse_vx(&args[0]).unwrap(),
                    parse_vx(&args[1]).unwrap(),
                    parse_n(&args[2]).unwrap()))),
        _ => panic!("Unrecognised mnemonic: {}", mnemonic),
    }

    match instr {
        Some(_) => instr,
        // If we get here we know the line had content so it's right to fail
        None => panic!("Failed to assemble instruction!"),
    }
}

fn check_num_args(mnemonic: &str, num: usize) {
    let expected = match mnemonic {
        "CLS" | "RET" => 0,
        "SYS" | "JP" | "CALL" | "SHR" | "SHL" | "SKP" | "SKNP" => 1,
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

use system::instr::*;
mod test;

pub fn parse_asm(lines: &[String]) -> Vec<Box<Instr>> {
    let mut instrs: Vec<Box<Instr>> = vec![];

    for line in lines {
        // Split line into menemonic <space> arg, arg, arg, etc
        let mut parts = line.split_whitespace();
        let mnemonic = parts.next().unwrap();
        let args = parts.map(|x| x.replace(",", "")).collect::<Vec<String>>();

        check_num_args(mnemonic, args.len());

        match mnemonic {
            // No arguments
            "CLS"   => instrs.push(Box::new(ClearDisplayInstr::create())),
            "RET"   => instrs.push(Box::new(RetInstr::create())),
            // Single argument
            "SYS"   => instrs.push(Box::new(SysInstr::create(parse_nnn(&args[0]).unwrap()))),
            "JP"    => instrs.push(Box::new(JumpInstr::create(parse_nnn(&args[0]).unwrap()))),
            "CALL"  => instrs.push(Box::new(CallInstr::create(parse_nnn(&args[0]).unwrap()))),
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

            // Only draw has 3
            "DRW"   => instrs.push(Box::new(DrawSpriteInstr::create(
                        parse_vx(&args[0]).unwrap(),
                        parse_vx(&args[1]).unwrap(),
                        parse_n(&args[2]).unwrap()))),
            _ => panic!("Unrecognised mnemonic: {}", mnemonic),
        }
    }
    

    instrs
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

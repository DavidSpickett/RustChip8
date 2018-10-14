use system::instr::*;
mod test;

pub fn parse_asm(lines: &[String]) -> Vec<Box<Instr>> {
    //TODO: make all these args 0 because why not
    let chip8_instrs = [
        Box::new(SysInstr::create(0x736)) as Box<Instr>,            
        Box::new(ClearDisplayInstr::create()) as Box<Instr>,
        Box::new(RetInstr::create()) as Box<Instr>,       
        Box::new(JumpInstr::create(0x123)) as Box<Instr>,
        Box::new(CallInstr::create(0x321)) as Box<Instr>,          
        Box::new(SkipEqualInstr::create(9, 0x45)) as Box<Instr>,
        Box::new(SkipNotEqualInstr::create(3, 0x89)) as Box<Instr>,
        Box::new(SkipIfRegsEqualInstr::create(1, 2)) as Box<Instr>,
        Box::new(LoadByteInstr::create(7, 0x63)) as Box<Instr>,
        Box::new(AddByteInstr::create(3, 0x68)) as Box<Instr>,    
        Box::new(MovRegInstr::create(0, 4)) as Box<Instr>,     
        Box::new(OrRegInstr::create(5, 8)) as Box<Instr>,         
        Box::new(AndRegInstr::create(10, 12)) as Box<Instr>,
        Box::new(XORRegInstr::create(13, 2)) as Box<Instr>,       
        Box::new(AddRegInstr::create(7, 14)) as Box<Instr>,        
        Box::new(SubRegInstr::create(6, 13)) as Box<Instr>,        
        Box::new(ShrRegInstr::create(5)) as Box<Instr>,        
        Box::new(SubNRegInstr::create(2, 9)) as Box<Instr>,
        Box::new(ShlRegInstr::create(11)) as Box<Instr>,        
        Box::new(SkipIfRegsNotEqualInstr::create(10, 3)) as Box<Instr>,
        Box::new(LoadIInstr::create(0x847)) as Box<Instr>, 
        Box::new(JumpPlusVZeroInstr::create(0x734)) as Box<Instr>,
        Box::new(RandomInstr::create(8, 0x39)) as Box<Instr>,     
        Box::new(DrawSpriteInstr::create(5, 7, 10)) as Box<Instr>,
        Box::new(SkipKeyIfPressedInstr::create(9)) as Box<Instr>,     
        Box::new(SkipKeyIfNotPressedInstr::create(3)) as Box<Instr>,
        Box::new(GetDelayTimerInstr::create(5)) as Box<Instr>,   
        Box::new(WaitForKeyInstr::create(11)) as Box<Instr>,         
        Box::new(SetDelayTimerInstr::create(6)) as Box<Instr>,        
        Box::new(SetSoundTimerInstr::create(12)) as Box<Instr>,     
        Box::new(AddIVInstr::create(7)) as Box<Instr>,        
        Box::new(GetDigitAddrInstr::create(13)) as Box<Instr>,
        Box::new(StoreBCDInstr::create(6)) as Box<Instr>,
        Box::new(WriteRegsToMemInstr::create(15)) as Box<Instr>,
        Box::new(ReadRegsFromMemInstr::create(2)) as Box<Instr>,
    ];

    let mut instrs: Vec<Box<Instr>> = vec![];

    for line in lines {
        // Split line into menemonic <space> arg, arg, arg, etc
        let mut parts = line.split_whitespace();
        let mnemonic = parts.next().unwrap();
        let args = parts.map(|x| x.replace(",", "")).collect::<Vec<String>>();

        // Get potential instruction objects by menmonic
        let potential_instrs = chip8_instrs.iter().filter(
            |i| i.get_mnemonic() == mnemonic ).collect::<Vec<&Box<Instr>>>();

        if potential_instrs.len() == 0 {
            panic!("Unrecognised mnemonic {}", mnemonic);
        }

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

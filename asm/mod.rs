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
        let args = parts.collect::<Vec<&str>>();
        // Get potential instruction objects by menmonic
        let potential_instrs = chip8_instrs.iter().filter(
            |i| i.get_mnemonic() == mnemonic ).collect::<Vec<&Box<Instr>>>();

        if potential_instrs.len() != 0 {
            check_num_args(mnemonic, args.len());

            // Now parse the arguments to check type
            // TODO: standard checkers for the differnt argument pairs e.g. VX, VY etc.
            // like we did for the create() methods
            match mnemonic {
                _ => panic!("Unrecognised mnemonic: {}", mnemonic),
            }
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
        Ok(v) => Ok(v),
    }
}

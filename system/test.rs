#[cfg(test)]
mod test {
    use system::*;
    use std::path::PathBuf;
    use std::collections::HashSet;

    #[test]
    fn run_bc_test_rom() {
        let mut rom_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        rom_path.push("roms/BC_test.ch8");
        let rom_path_str = String::from(rom_path.to_str().expect("bad path!"));
        let mut c8 = make_system(&read_rom(&rom_path_str));

        let expected = "\
        ----------------------------------------------------------------\n\
        ----------------------------------------------------------------\n\
        ----------------------------------------------------------------\n\
        ----------------------------------------------------------------\n\
        ----------------------------------------------------------------\n\
        ----------------------------------------------------------------\n\
        ----------------------------------------------------------------\n\
        ----------------------------------------------------------------\n\
        ----------------------------------------------------------------\n\
        ----------------------------------------------------------------\n\
        ----------------------------------------------------------------\n\
        ---------------------@@@@-----@@@@---@----@---------------------\n\
        ---------------------@---@---@----@--@@---@---------------------\n\
        ---------------------@---@---@----@--@-@--@---------------------\n\
        ---------------------@@@@----@----@--@--@-@---------------------\n\
        ---------------------@---@---@----@--@---@@---------------------\n\
        ---------------------@---@---@----@--@----@---------------------\n\
        ---------------------@---@---@----@--@----@---------------------\n\
        ---------------------@@@@-----@@@@---@----@---------------------\n\
        ----------------------------------------------------------------\n\
        ----------------------------------------------------------------\n\
        ----------------------------------------------------------------\n\
        ----------------------------------------------------------------\n\
        ----------------------------------------------------------------\n\
        --@@-------------@@-------------@----@@@---------@--------------\n\
        --@-@------------@-@------------@----@-----------@--------------\n\
        --@-@--@-@-------@-@---@@---@@--@@---@-----@-----@---@@---------\n\
        --@@---@-@-------@@---@-@--@----@----@----@-@---@@--@-@---@@----\n\
        --@-@--@@@-------@-@--@@----@---@----@----@-@--@-@--@@----@-----\n\
        --@-@----@-------@-@--@------@--@----@----@-@--@-@--@-----@-----\n\
        --@@-----@-------@@----@@--@@----@@--@@@---@----@@---@@---@-@---\n";
       
        'running: loop {
            let instr = c8.fetch_and_decode();
            // Jump to self
            if instr.get_opcode() == 0x130e {
                break 'running;
            }
            c8.execute(&instr);
        }
        let got = c8.screen_to_str();
        assert_eq!(expected, got);
    }

    fn all_valid_chip8_instrs() -> Vec<u16> {
        // Generate all bitpatterns that should decode to valid Chip8 instructions
        let mut instrs: Vec<u16> = vec![];

        let alls: [u16; 11] = [0x0, 0x1, 0x2, 0x3, 0x4, 0x6, 0x7, 0xA, 0xB, 0xC, 0xD];
        for n in alls.iter() {
            let base = n << 12;
            for i in 0..0x1000_u16 {
                instrs.push(base+i);
            }
        }

        let bases: [u16; 2] = [0x5000, 0x9000];
        for base in bases.iter() {
            for n in 0..0x100_u16 {
                instrs.push(base+(n << 4));
            }
        }

        let eight_ends: [u16; 9] = [0x0, 0x1, 0x2, 0x3, 0x4, 0x5, 0x6, 0x7, 0xE];
        for end in eight_ends.iter() {
            let base = 0x8000 | *end;
            for n in 0..0x100_u16 {
                instrs.push(base+(n << 4));
            }
        }

        let e_ends: [u16; 2] = [0x9E, 0xA1];
        for end in e_ends.iter() {
            let base = 0xE000 | *end;
            for n in 0..0x10_u16 {
                instrs.push(base+(n<<8));
            }
        }

        let f_ends: [u16; 9] = [0x07, 0x0A, 0x15, 0x18, 0x1E, 0x29, 0x33, 0x55, 0x65];
        for end in f_ends.iter() {
            let base = 0xF000 | *end;
            for n in 0..0x10_u16 {
                instrs.push(base+(n<<8));
            }
        }

        instrs
    }

    #[test]
    fn do_all_valid_chip8_instr() {
        let instrs = all_valid_chip8_instrs();
        let dummy: Vec<u8> = vec![];
        let mut c8 = make_system(&dummy);

        // Execute all instructions at 0x200, reset in between
        // So that something like JP to self doesn't hold up
        // the whole process
        for i in instrs.iter() {
            let decode = c8.get_opcode_obj(*i);
            match decode {
                Err(msg)  => panic!(msg),
                Ok(instr) => c8.execute(&instr),
            }

            // Prevent trying to call with a full stack etc.
            c8.reset_regs();
            // This allows us to do a ret without having a corresponding call
            c8.stack_ptr = 1;
        }
    }

    #[test]
    fn all_invalid_chip8_should_panic() {
        let all_instrs: HashSet<u16> = all_valid_chip8_instrs().into_iter().collect();
        let all_encodings: HashSet<u16> = (0..0xFFFF_u16).collect();
        let invalid_instrs = all_encodings.difference(&all_instrs);

        let dummy: Vec<u8> = vec![];
        let c8 = make_system(&dummy);
        for i in invalid_instrs {
            let decode = c8.get_opcode_obj(*i);
            assert!(decode.is_err());
        }
    }

}

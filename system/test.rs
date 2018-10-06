#[cfg(test)]
mod test {
    use system::*;
    use std::path::PathBuf;
    use std::collections::HashSet;
    extern crate rand;
    use system::test::test::rand::Rng;

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
            // This allows us to do a ret without having a corresponding call
            c8.stack.push(0x200);

            let decode = c8.get_opcode_obj(*i);
            match decode {
                Err(msg)  => panic!(msg),
                Ok(instr) => c8.execute(&instr),
            }

            // Prevent trying to call with a full stack etc.
            c8.reset_regs();
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

    fn setup_max_gp_regs(c8: &mut Chip8System) {
        for r in c8.v_regs.iter_mut() {
            *r = <u8>::max_value();
        }
    }

    #[test]
    fn all_chip8_instr_max_gp_reg_values() {
        // This is just GP regs, we'll look at PC/stack_ptr/I seperatley
        let valid_instrs = all_valid_chip8_instrs();

        let dummy: Vec<u8> = vec![];
        let mut c8 = make_system(&dummy);

        for i in valid_instrs {
            c8.reset_regs();
            setup_max_gp_regs(&mut c8);
            c8.stack.push(0x200);

            let instr = c8.get_opcode_obj(i).unwrap();
            // Again we need to handle key index > 16 somehow, just not now
            if instr.get_flags() != InstrFlags::Keys {
                c8.execute(&instr);
            }
        }
    }

    fn setup_nested_call_test() -> Chip8System {
        // CALL 0x200
        // call to self so we only need one
        let instrs: Vec<u8> = vec![0x22, 0x00];
        let mut c8 = make_system(&instrs);

        // 1 off of the limit
        for _ in 0..16 {
            let ins = c8.fetch_and_decode();
            c8.execute(&ins);
        }
        c8
    }

    // This is a weird way of making sure the setup functon doesn't
    // panic. The test below will then check that the stack limit is enforced.
    #[test]
    fn call_to_full_stack() {
        let _ = setup_nested_call_test();
    }

    #[test]
    fn ret_to_empty_stack() {
        let mut c8 = setup_nested_call_test();
        let pc = 0x200;
        // RET, since we just CALL 0x200 a lot, we'll ret to 0x200 each time
        c8.memory[pc] = 0x00; c8.memory[pc+1] = 0xEE;

        for _ in 0..16 {
            let ins = c8.fetch_and_decode();
            c8.execute(&ins);
        }
    }

    #[test]
    #[should_panic(expected = "Stack is full!")]
    fn stack_nested_calls_panics() {
        let mut c8 = setup_nested_call_test();
        // do the last CALL
        let ins = c8.fetch_and_decode();
        c8.execute(&ins);
    }

    #[test]
    #[should_panic(expected = "Stack is empty!")]
    fn ret_empty_stack_panics() {
        // RET
        let instrs: Vec<u8> = vec![0x00, 0xEE];
        let mut c8 = make_system(&instrs);

        let ins = c8.fetch_and_decode();
        c8.execute(&ins);
    }

    #[test]
    fn valid_key_indexes() {
        // Key numbers 0-15 should not panic

        let dummy: Vec<u8> = vec![];
        let mut c8 = make_system(&dummy);
        // Both using V0
        let instrs: Vec<u16> = vec![0xE09E, 0xE0A1];

        for key in 0..16_u8 {
            c8.v_regs[0] = key;
            for instr in instrs.iter() {
                let decode = c8.get_opcode_obj(*instr);
                match decode {
                    Err(msg)  => panic!(msg),
                    Ok(instr) => c8.execute(&instr),
                }
            }
        }
    }

    fn setup_invalid_key_test() -> Chip8System {
        let rom: Vec<u8> = vec![0xE0, 0x9E, 0xE0, 0xA1];
        make_system(&rom)
    }

    #[test]
    #[should_panic(expected="Key number 16 out of range!")]
    fn invalid_key_index_if_pressed() {
        // Key numbers >=16 should panic
        let mut c8 = setup_invalid_key_test();
        c8.v_regs[0] = 16; // TODO: check > 16 too
        let ins = c8.fetch_and_decode();
        c8.execute(&ins);
    }

    #[test]
    #[should_panic(expected="Key number 16 out of range!")]
    fn invalid_key_index_if_not_pressed() {
        // Key numbers >=16 should panic
        let mut c8 = setup_invalid_key_test();
        c8.v_regs[0] = 16; // TODO: check > 16 too
        c8.pc += 2; // Skip to if not instr
        let ins = c8.fetch_and_decode();
        c8.execute(&ins);
    }

    fn randomise_regs(c8: &mut Chip8System) {
        let mut rng = rand::thread_rng();
        for r in c8.v_regs.iter_mut() {
            *r = rng.gen::<u8>();
        }
        c8.i_reg = rng.gen::<u16>();
    }

    #[test]
    #[ignore]
    fn fuzz_test () {
        let valid_instrs = all_valid_chip8_instrs();
        let dummy: Vec<u8> = vec![];
        let mut c8 = make_system(&dummy);

        let exceptions: Vec<u16> = vec![
            0x00EE, // RET
            0xE09E, // SKP
            0xE0A1, // SKNP
            // 0xF055, // LD [I], VX
            // 0xF065, // LD VX, [I]
            // 0xF033, // LD B, VX
        ];

        'running: loop {
            for i in valid_instrs.iter() {
                if (exceptions.contains(&(*i & 0xF0FF)) ||
                    (*i >> 12) == 0x2) || // CALL
                   ((*i >> 12) == 0xD)    // DRW
                   {
                       continue;
                   }
                println!("0x{:04x}", *i);
                randomise_regs(&mut c8);
                let decode = c8.get_opcode_obj(*i);
                match decode {
                    Err(msg)  => panic!(msg),
                    Ok(instr) => c8.execute(&instr),
                }
            }
        }
    }
}

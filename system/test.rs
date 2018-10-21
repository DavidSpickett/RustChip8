#[cfg(test)]
mod test {
    use system::*;
    use std::path::PathBuf;
    use asm::parse_asm;
    use std::collections::HashSet;
    extern crate rand;
    use system::test::test::rand::{Rng, thread_rng};
    extern crate itertools;
    use self::itertools::Itertools;

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
        --@@-----@-------@@----@@--@@----@@--@@@---@----@@---@@---@-@---\n\
        -------@@@------------------------------------------------------";
       
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
        //SKP V0; SKNP V0
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

    fn setup_out_of_bounds_i_reg() -> Chip8System {
        let rom: Vec<u8> = vec![
            0xF0, 0x33, // Store BCD
            0xF2, 0x55, // Store registers
            0xF2, 0x65, // Load registers
        ];
        let mut c8 = make_system(&rom);
        c8.i_reg = 0xFFFD;
        c8
    }

    #[test]
    #[should_panic(expected="I register memory access at 0xfffd with length 3 is out of bounds!")]
    fn out_of_bounds_i_reg_bcd() {
        let mut c8 = setup_out_of_bounds_i_reg();
        let ins = c8.fetch_and_decode();
        c8.execute(&ins);
    }

    #[test]
    #[should_panic(expected="I register memory access at 0xfffd with length 3 is out of bounds!")]
    fn out_of_bounds_i_reg_store_regs() {
        let mut c8 = setup_out_of_bounds_i_reg();
        c8.pc = 0x202;
        let ins = c8.fetch_and_decode();
        c8.execute(&ins);
    }

    #[test]
    #[should_panic(expected="I register memory access at 0xfffd with length 3 is out of bounds!")]
    fn out_of_bounds_i_reg_load_regs() {
        let mut c8 = setup_out_of_bounds_i_reg();
        c8.pc = 0x204;
        let ins = c8.fetch_and_decode();
        c8.execute(&ins);
    }

    fn instr_to_data(instrs: &[Box<Instr>]) -> Vec<u8> {
        let mut rom: Vec<u8> = vec![];
        for i in instrs {
            let opc = i.get_opcode();
            rom.push((opc >> 8) as u8);
            rom.push(opc as u8);
        }
        rom
    }

    static PROG_EXPECTED: &'static str = "\
        ----------------------------------------------------------------\n\
        ----------------------------------------------------------------\n\
        ----------------------------------------------------------------\n\
        ----------------------------------------------------------------\n\
        ----------------------------------------------------------------\n\
        ----------------------------------------------------------------\n\
        ----------------------------------------------------------------\n\
        --@@@@------@-----@@@@----@@@@----@--@----@@@@----@@@@----@@@@--\n\
        --@--@-----@@--------@-------@----@--@----@-------@----------@--\n\
        --@--@------@-----@@@@----@@@@----@@@@----@@@@----@@@@------@---\n\
        --@--@------@-----@----------@-------@-------@----@--@-----@----\n\
        --@@@@-----@@@----@@@@----@@@@-------@----@@@@----@@@@-----@----\n\
        ----------------------------------------------------------------\n\
        ----------------------------------------------------------------\n\
        ----------------------------------------------------------------\n\
        ----------------------------------------------------------------\n\
        ----------------------------------------------------------------\n\
        ----------------------------------------------------------------\n\
        ----------------------------------------------------------------\n\
        ----------------------------------------------------------------\n\
        --@@@@----@@@@----@@@@----@@@-----@@@@----@@@-----@@@@----@@@@--\n\
        --@--@----@--@----@--@----@--@----@-------@--@----@-------@-----\n\
        --@@@@----@@@@----@@@@----@@@-----@-------@--@----@@@@----@@@@--\n\
        --@--@-------@----@--@----@--@----@-------@--@----@-------@-----\n\
        --@@@@----@@@@----@--@----@@@-----@@@@----@@@-----@@@@----@-----\n\
        ----------------------------------------------------------------\n\
        ----------------------------------------------------------------\n\
        ----------------------------------------------------------------\n\
        ----------------------------------------------------------------\n\
        ----------------------------------------------------------------\n\
        ----------------------------------------------------------------\n\
        ----------------------------------------------------------------";

    #[test]
    fn basic_instr_building() {
        let mut instrs: Vec<Box<Instr>> = vec![
            // V0 = digit = 0 already
            // V1 = x = 2
            Box::new(AddByteInstr::create(1, 2)),
            // V2 = y = 7
            Box::new(AddByteInstr::create(2, 7)),

            //Draw char to screen
            Box::new(GetDigitAddrInstr::create(0)),
            Box::new(DrawSpriteInstr::create(1, 2, 5)),

            // Increment x
            Box::new(AddByteInstr::create(1, 8)),
            // If we're at 7, move one row down
            Box::new(SkipNotEqualInstr::create(0, 7)),
            // +5 for the height of the char
            Box::new(AddByteInstr::create(2, 8+5)),
            // Note that we don't need to reset X as drawing wraps after x=64

            // Increment digit (after checking for the new row)
            Box::new(AddByteInstr::create(0, 1)),

            // If we're not at 0x10 (not drawn F), jump back
            Box::new(SkipEqualInstr::create(0, 0x10)),
            // Note that we jump *over* the setup instr for V1/V2 (X/Y)
            Box::new(JumpInstr::create(0x204)),
        ];

        // Add a jump to self to finish off
        let target: u16 = (0x200 + (instrs.len() as u16))*2;
        instrs.push(Box::new(JumpInstr::create(target)));

        let rom = instr_to_data(&instrs);
        let mut c8 = make_system(&rom);

        while c8.pc != target {
            let ins = c8.fetch_and_decode();
            c8.execute(&ins);
        }

        assert_eq!(PROG_EXPECTED, c8.screen_to_str());
    }

    #[test]
    fn assembling_example_program() {
        let asm = "\
        // Setup X and Y
        ADD V1, 0x02\n\
        ADD V2, 0x07\n\
        loop:\n\
        LD F, V0 // Load address of digit\n\
        // Draw it
        DRW V1, V2, 5 // Increment X\n\
        ADD V1, 0x08\n\
        // If we're about to draw char 8, move down a row
        SNE V0, 0x07\n\
        ADD V2, 0x0d\n\
        ADD V0, 0x01 // Increment digit\n\
        // If we just drew 'F' end the program\n\
        SE V0, 0x10\n\
        // Otherwise draw the next char\n\
        JP loop\n\
        self:\n\
        JP self".to_string();

        let instrs = parse_asm(&asm).unwrap();
        let rom = instr_to_data(&instrs);

        let target: u16 = 0x214;
        let mut c8 = make_system(&rom);

        while c8.pc != target {
            let ins = c8.fetch_and_decode();
            c8.execute(&ins);
        }

        assert_eq!(PROG_EXPECTED, c8.screen_to_str());
    }

    #[test]
    fn instrs_have_create() {
        let data = [
            (Box::new(SysInstr::create(0x736))                as Box<Instr>, "SYS 0x736"),
            (Box::new(ClearDisplayInstr::create())            as Box<Instr>, "CLS"),
            (Box::new(RetInstr::create())                     as Box<Instr>, "RET"),
            (Box::new(JumpInstr::create(0x123))               as Box<Instr>, "JP 0x123"),
            (Box::new(CallInstr::create(0x321))               as Box<Instr>, "CALL 0x321"),
            (Box::new(SkipEqualInstr::create(9, 0x45))        as Box<Instr>, "SE V9, 0x45"),
            (Box::new(SkipNotEqualInstr::create(3, 0x89))     as Box<Instr>, "SNE V3, 0x89"),
            (Box::new(SkipIfRegsEqualInstr::create(1, 2))     as Box<Instr>, "SE V1, V2"),
            (Box::new(LoadByteInstr::create(7, 0x63))         as Box<Instr>, "LD V7, 0x63"),
            (Box::new(AddByteInstr::create(3, 0x68))          as Box<Instr>, "ADD V3, 0x68"),
            (Box::new(MovRegInstr::create(0, 4))              as Box<Instr>, "LD V0, V4"),
            (Box::new(OrRegInstr::create(5, 8))               as Box<Instr>, "OR V5, V8"),
            (Box::new(AndRegInstr::create(10, 12))            as Box<Instr>, "AND V10, V12"),
            (Box::new(XORRegInstr::create(13, 2))             as Box<Instr>, "XOR V13, V2"),
            (Box::new(AddRegInstr::create(7, 14))             as Box<Instr>, "ADD V7, V14"),
            (Box::new(SubRegInstr::create(6, 13))             as Box<Instr>, "SUB V6, V13"),
            (Box::new(ShrRegInstr::create(5))                 as Box<Instr>, "SHR V5"),
            (Box::new(SubNRegInstr::create(2, 9))             as Box<Instr>, "SUBN V2, V9"),
            (Box::new(ShlRegInstr::create(11))                as Box<Instr>, "SHL V11"),
            (Box::new(SkipIfRegsNotEqualInstr::create(10, 3)) as Box<Instr>, "SNE V10, V3"),
            (Box::new(LoadIInstr::create(0x847))              as Box<Instr>, "LD I, 0x847"),
            (Box::new(JumpPlusVZeroInstr::create(0x734))      as Box<Instr>, "JP V0, 0x734"),
            (Box::new(RandomInstr::create(8, 0x39))           as Box<Instr>, "RND V8, 0x39"),
            (Box::new(DrawSpriteInstr::create(5, 7, 10))      as Box<Instr>, "DRW V5, V7, 10"),
            (Box::new(SkipKeyIfPressedInstr::create(9))       as Box<Instr>, "SKP V9"),
            (Box::new(SkipKeyIfNotPressedInstr::create(3))    as Box<Instr>, "SKNP V3"),
            (Box::new(GetDelayTimerInstr::create(5))          as Box<Instr>, "LD V5, DT"),
            (Box::new(WaitForKeyInstr::create(11))            as Box<Instr>, "LD V11, K"),
            (Box::new(SetDelayTimerInstr::create(6))          as Box<Instr>, "LD DT, V6"),
            (Box::new(SetSoundTimerInstr::create(12))         as Box<Instr>, "LD ST, V12"),
            (Box::new(AddIVInstr::create(7))                  as Box<Instr>, "ADD I, V7"),
            (Box::new(GetDigitAddrInstr::create(13))          as Box<Instr>, "LD F, V13"),
            (Box::new(StoreBCDInstr::create(6))               as Box<Instr>, "LD B, V6"),
            (Box::new(WriteRegsToMemInstr::create(15))        as Box<Instr>, "LD [I], V15"),
            (Box::new(ReadRegsFromMemInstr::create(2))        as Box<Instr>, "LD V2, [I]"),
        ];

        for (ins, expected) in data.iter() {
            assert_eq!(String::from(*expected), ins.repr());
        }
    }

    // TODO: test same thing for all address based instrs
    // Need to deal with unwind boundaries
    #[test]
    #[should_panic(expected="Cannot get address for unresolved symbol \"xyz\"")]
    fn exec_on_unresolved_symbol_panics() {
        let dummy: Vec<u8> = vec![];
        let mut c8 = make_system(&dummy);

        let ins = Box::new(SysInstr::create_with_symbol("xyz".to_string())) as Box<Instr>;
        c8.execute(&ins);
    }

    fn make_sprite_asm(sprite: &String) -> Vec<String> {
        let mut sprite_data: [u8; 8*16] = [0; 8*16];

        for (ln, line) in sprite.lines().enumerate() {
            for (p, c) in line.chars().enumerate() {
                let sprite_idx = ((ln/8)*4) + (p/8);
                let sprite_line = ln % 8;
                let char_idx = p % 8;
                if c == '@' {
                    sprite_data[(sprite_idx*8)+sprite_line] |= 1 << (7-char_idx);
                }
            }
        }

        let mut data: Vec<u16> = vec![];
        for mut bytes in &sprite_data.into_iter().chunks(2) {
            data.push(((*bytes.next().unwrap() as u16) << 8) | (*bytes.next().unwrap() as u16));
        }

        data.iter().map(|v| format!(".word 0x{:04x}", v) ).collect()
    }

    #[test]
    fn draw_sprite_from_rom() {
        let sprite = "\
            @@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@\n\
            @------------------------------@\n\
            @-@@@@@@@@@@@@@@@@@@@@@@@@@@@@-@\n\
            @-@--------------------------@-@\n\
            @-@-@@@@@@@@@@@@@@@@@@@@@@@@-@-@\n\
            @-@-@----------------------@-@-@\n\
            @-@-@-@@@@@@@@@@@@@@@@@@@@-@-@-@\n\
            @-@-@-@------------------@-@-@-@\n\
            @-@-@-@-@@@@@@@@@@@@@@@@-@-@-@-@\n\
            @-@-@-@-@--------------@-@-@-@-@\n\
            @-@-@-@-@-@@@@@@@@@@@@-@-@-@-@-@\n\
            @-@-@-@-@-@---------@--@-@-@-@-@\n\
            @-@-@-@-@-@-@@@@@@@-@--@-@-@-@-@\n\
            @-@-@-@-@-@-@-----@-@--@-@-@-@-@\n\
            @-@-@-@-@-@-@-@@@-@-@--@-@-@-@-@\n\
            @-@-@-@-@-@-@-@-@-@-@--@-@-@-@-@\n\
            @-@-@-@-@-@-@-@-@-@-@--@-@-@-@-@\n\
            @-@-@-@-@-@-@-@@@-@-@--@-@-@-@-@\n\
            @-@-@-@-@-@-@-----@-@--@-@-@-@-@\n\
            @-@-@-@-@-@-@@@@@@@-@--@-@-@-@-@\n\
            @-@-@-@-@-@---------@--@-@-@-@-@\n\
            @-@-@-@-@-@@@@@@@@@@@@-@-@-@-@-@\n\
            @-@-@-@-@--------------@-@-@-@-@\n\
            @-@-@-@-@@@@@@@@@@@@@@@@-@-@-@-@\n\
            @-@-@-@------------------@-@-@-@\n\
            @-@-@-@@@@@@@@@@@@@@@@@@@@-@-@-@\n\
            @-@-@----------------------@-@-@\n\
            @-@-@@@@@@@@@@@@@@@@@@@@@@@@-@-@\n\
            @-@--------------------------@-@\n\
            @-@@@@@@@@@@@@@@@@@@@@@@@@@@@@-@\n\
            @------------------------------@\n\
            @@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@".to_string();


        let mut asm = make_sprite_asm(&sprite).iter().fold("".to_string(), |acc, l| acc + &"\n".to_string() + l);
        asm = "\
            JP start
            sprite_data:".to_string() + &asm;
        asm += &"
        start:
            LD V0, 0x10 // X
            LD V1, 0x00 // Y
            LD I, sprite_data
            LD V2, 0x00 // Sprite counter
            LD V3, 0x08 // I increment
        loop:
            DRW V0, V1, 8
            ADD V0, 0x08 // Inc X
            ADD V2, 0x01
            ADD I, V3 // Point to new sprite
            // If we've drawn all 16 sprites, end
            SNE V2, 0x10
            JP end
            // If we haven't drawn the last sprite on the row...
            SE V0, 0x30
            // Continue to draw this row
            JP loop
            // Otherwise we need to increment Y and reset X
            ADD V1, 0x08
            LD V0, 0x10
            JP loop
        end:
            JP end".to_string();

        let instrs = parse_asm(&asm).unwrap();
        let rom = instr_to_data(&instrs);
        let mut c8 = make_system(&rom);
        let mut old_pc: u16 = 0xffff;

        while c8.pc != old_pc {
            old_pc = c8.pc;
            let ins = c8.fetch_and_decode();
            c8.execute(&ins);
        }

        // Convert to a screen dump for comparison
        let pad = "----------------";
        let expected = sprite.lines().map(|x| pad.to_owned() + &x.to_owned() + pad).join("\n");

        assert_eq!(expected, c8.screen_to_str());
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
        let mut valid_instrs = all_valid_chip8_instrs();
        let dummy: Vec<u8> = vec![];
        let mut c8 = make_system(&dummy);

        let exceptions: Vec<u16> = vec![
            0x00EE, // RET
            0xE09E, // SKP
            0xE0A1, // SKNP
            0xF055, // LD [I], VX
            0xF065, // LD VX, [I]
            0xF033, // LD B, VX
        ];

        'running: loop {
            thread_rng().shuffle(&mut valid_instrs);
            for i in valid_instrs.iter() {
                if (exceptions.contains(&(*i & 0xF0FF)) ||
                    (*i >> 12) == 0x2) || // CALL
                   ((*i >> 12) == 0xD)    // DRW
                   {
                       continue;
                   }
                randomise_regs(&mut c8);
                let decode = c8.get_opcode_obj(*i);
                print!("0x{:04x}", *i);
                match decode {
                    Err(msg)  => panic!(msg),
                    Ok(instr) => {
                        println!(" -- {}", instr.repr());
                        c8.execute(&instr);
                    },
                }
            }
        }
    }
}

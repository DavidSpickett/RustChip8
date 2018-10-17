#[cfg(test)]
mod test {
    use asm::*;

    #[test]
    fn expected_parse_vx() {
        assert_eq!(u8::from(1), parse_vx(&String::from("V1")).unwrap());
        match parse_vx(&String::from("B1")) {
            Ok(_) => panic!(),
            Err(msg) => assert_eq!("Does not begin with \"V\"", msg),
        }
    }

    #[test]
    fn expected_parse_xx() {
        assert_eq!(u8::from(0x12), parse_xx(&String::from("0x12")).unwrap());
    }

    #[test]
    fn basic_assembly_test() {
        let expected = [
            "SYS 0x123",
            "CLS",
            "RET",
            "JP 0x123",
            "JP V0, 0x123",
            "CALL 0x123",
            "SHR V0",
            "SHL V0",
            "SKP V0",
            "SKNP V0",
            "DRW V0, V1, 12",
            "AND V0, V1",
            "XOR V0, V1",
            "OR V0, V1",
            "SUB V0, V1",
            "SUBN V0, V1",
            "RND V0, 0x12",
            "SE V0, 0x12",
            "SE V0, V1",
            "SNE V0, 0x12",
            "SNE V0, V1",
            "ADD I, V0",
            "ADD V0, V1",
            "ADD V0, 0x12",
            "LD V0, 0x12",
            "LD V0, V1",
            "LD I, 0x123",
            "LD V0, DT",
            "LD V0, K",
            "LD V0, [I]",
            "LD DT, V0",
            "LD ST, V0",
            "LD F, V0",
            "LD B, V0",
            "LD [I], V0",
        ].iter().map(|x| x.to_string()).collect::<Vec<String>>();
        let asm_str = expected.iter().fold(String::from(""), |acc, n| acc + "\n" + n);
        let got = parse_asm(&asm_str);
        for (e, g) in expected.iter().zip(got.iter()) {
            assert_eq!(*e, g.repr());
        }
    }

    fn assert_asm_bitpatterns(asm: &String, expected: &[u16]) {
        for (instr, exp) in parse_asm(&asm).iter().zip(expected.iter()) {
            assert_eq!(*exp, instr.get_opcode());
        }
    }

    #[test]
    fn test_blank_lines_ignored() {
        let asm = "\
        \t\n\
        DRW V0, V1, 6
             \n\
        ADD V0, 0x34".to_string();
        let expected: [u16; 2] = [0xD016, 0x7034];

        assert_asm_bitpatterns(&asm, &expected);
    }

    #[test]
    fn test_single_line_comments_ignored() {
        let asm = "\
        // This is a single line comment\n\
        DRW V5, V2, 7\n\
        LD F, V9 // This is one the end of a line\n\
        LD ST, V2// This one has no space after the arg\n\
        CLS//This one LD ST, V7 includes an instr".to_string();
        let expected: Vec<u16> = vec![0xD527, 0xF929, 0xF218, 0x00E0];

        assert_asm_bitpatterns(&asm, &expected);
    }

    #[test]
    fn branch_back_to_label() {
        let asm = "\
        SYS 0x000\n\
        // Comment lines don't increment address\n\
        foo:\n\
        SYS 0x111\n\
        bar:: // Should be refered to as 'bar:' just fine
        JP foo\n\
        JP bar:".to_string();
        let expected: Vec<u16> = vec![0x0000, 0x0111, 0x1202, 0x1204];

        assert_asm_bitpatterns(&asm, &expected);
    }

    #[test]
    fn branch_forward_to_label() {
        let asm = "\
        JP start
        CALL start
        SYS start
        JP V0, start
        LD I, start
        start:
        self:
        JP self".to_string();
        let expected: Vec<u16> = vec![0x120a, 0x220a, 0x020a, 0xB20a, 0xA20a, 0x120a];

        assert_asm_bitpatterns(&asm, &expected);
    }

    #[test]
    #[should_panic(expected="Could not resolve symbol \"aardvark\"")]
    fn unresolved_symbol_panics() {
        let asm = "\
        JP next\n\
        next:\n\
        CALL next\n\
        JP aardvark".to_string();

        parse_asm(&asm);
    }

    #[test]
    fn symbol_capable_instructions() {
        let asm ="\
        SYS 0x000\n\
        SYS 0x000\n\
        target:\n\
        SYS target\n\
        JP target\n\
        CALL target\n\
        LD I, target".to_string();
        let expected: Vec<u16> = vec![0x0000, 0x0000, 0x0204, 0x1204, 0x2204, 0xA204];

        assert_asm_bitpatterns(&asm, &expected);
    }

    #[test]
    fn set_i_extended_address() {
        // Standard jump with remainder
        let asm = "LD I, 0x1999".to_string();
        let expected: Vec<u16> = vec![
            0xAFFF,
            0x6EFF,
            0xFE1E, 0xFE1E, 0xFE1E, 0xFE1E, 0xFE1E, 0xFE1E, 0xFE1E, 0xFE1E, 0xFE1E,
            0x6Ea3,
            0xFE1E,
        ];

        assert_asm_bitpatterns(&asm, &expected);

        // No remainder here
        let asm2 = "LD I, 0x11FD".to_string();
        let expected2: Vec<u16> = vec![
            0xAFFF,
            0x6EFF,
            0xFE1E, 0xFE1E,
        ];
        assert_asm_bitpatterns(&asm2, &expected2);

        // Only remainder
        let asm3 = "LD I, 0x100B".to_string();
        let expected3: Vec<u16> = vec![
            0xAFFF,
            0x6E0C,
            0xFE1E,
        ];
        assert_asm_bitpatterns(&asm3, &expected3);
    }

}

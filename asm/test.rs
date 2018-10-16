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
}

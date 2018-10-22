#[cfg(test)]
mod test {
    use asm::*;

    #[test]
    fn expected_parse_vx() {
        assert_eq!(u8::from(1), parse_vx(&AsmArg::new(String::from("V1"), 0)).unwrap());
        match parse_vx(&AsmArg::new(String::from("B1"), 0)) {
            Ok(_) => panic!(),
            Err((msg, _, _)) => assert_eq!("VX arg does not begin with \"V\"", msg),
        }
    }

    #[test]
    fn expected_parse_xx() {
        assert_eq!(u8::from(0x12), parse_xx(&AsmArg::new(String::from("0x12"), 0)).unwrap());
    }

    #[test]
    fn expected_split_asm_line() {
        // Note that we expect comments to be gone by this point
        let tests: Vec<(&str, Vec<AsmArg>)> = vec![
            ("CLS", vec![
                AsmArg::new("CLS".to_string(), 0)]),
            ("JP 0x123", vec![
                AsmArg::new("JP".to_string(), 0),
                AsmArg::new("0x123".to_string(), 3)]),
            ("DRW ,V0  ,v1,   00012", vec![
                AsmArg::new("DRW".to_string(), 0),
                AsmArg::new("V0".to_string(), 5),
                AsmArg::new("v1".to_string(), 10),
                AsmArg::new("00012".to_string(), 16)]),
            ("JP F", vec![
                AsmArg::new("JP".to_string(), 0),
                AsmArg::new("F".to_string(), 3)]),
            ("AND V0, abc", vec![
                AsmArg::new("AND".to_string(), 0),
                AsmArg::new("V0".to_string(), 4),
                AsmArg::new("abc".to_string(), 8)]),
        ];
        for (input, expected) in tests {
            assert_eq!(expected, split_asm_line(input));
        }
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
            ".word 0x1234",
        ].iter().map(|x| x.to_string()).collect::<Vec<String>>();
        let asm_str = expected.iter().fold(String::from(""), |acc, n| acc + "\n" + n);
        let got = parse_asm_str(&asm_str).unwrap();
        for (e, g) in expected.iter().zip(got.iter()) {
            assert_eq!(*e, g.repr());
        }
    }

    fn assert_asm_roundtrip(tests: &[(&str, &str)]) {
        for (input, expected) in tests.iter() {
            println!("{}", input);
            let in_str = input.to_string();
            assert_eq!(*expected, parse_asm_str(&in_str).unwrap()[0].repr());
        }
    }

    #[test]
    fn either_case_mnemonics() {
        let asm_tests = [
            ("cls",          "CLS"),
            ("jp 0x123",     "JP 0x123"),
            ("add V0, V1",   "ADD V0, V1"),
            (".WORD 0x1234", ".word 0x1234"),
        ];
        assert_asm_roundtrip(&asm_tests);
    }

    #[test]
    fn either_case_and_hex_v_regs() {
        let asm_tests = [
            ("ADD V0, V1",       "ADD V0, V1"),
            ("ADD v0, V1",       "ADD V0, V1"),
            ("ADD V0, v1",       "ADD V0, V1"),
            //A bit silly but they work so lets check them
            ("ADD V0000000, v1", "ADD V0, V1"),
            ("ADD V00F, v1",     "ADD V15, V1"),

            ("ADD VA, V10",      "ADD V10, V10"),
            ("ADD V02, VF",      "ADD V2, V15"),

            // This instr always has V0, but it should verify it the same way
            ("JP V0, 0x123",     "JP V0, 0x123"),
            ("JP v0, 0x456",     "JP V0, 0x456"),
            ("JP V000, 0x789",   "JP V0, 0x789"),
        ];
        assert_asm_roundtrip(&asm_tests);
    }
    
    #[test]
    fn hex_number_formatting_accepted() {
        let asm_tests = [
            // Lets assume that all instrs use standard parser
            // functions and just check one of each

            // 12 bit addresses
            ("SYS 0x123",        "SYS 0x123"),
            ("SYS 0x23",         "SYS 0x023"),
            ("SYS 0x0000000023", "SYS 0x023"),
            ("SYS 0x0000000323", "SYS 0x323"),

            // Bytes
            ("RND V0, 0x12",       "RND V0, 0x12"),
            ("RND V0, 0x2",        "RND V0, 0x02"),
            ("RND V0, 0x00000002", "RND V0, 0x02"),

            // 16 bit values
            (".word 0x1234",     ".word 0x1234"),
            (".word 0x234",      ".word 0x0234"),
            (".word 0x34",       ".word 0x0034"),
            (".word 0x4",        ".word 0x0004"),
            (".word 0x00000004", ".word 0x0004"),
            (".word 0x00001234", ".word 0x1234"),

            //Mixed case letters
            (".word 0xFaBc", ".word 0xFABC"),
        ];
        assert_asm_roundtrip(&asm_tests);
    }

    #[test]
    fn nibble_formatting_accepted() {
        let asm_tests = [
            ("DRW V0, V1, 0001", "DRW V0, V1, 1"),
            ("DRW V0, V1, 012",  "DRW V0, V1, 12"),
        ];
        assert_asm_roundtrip(&asm_tests);
    }

    fn assert_asm_bitpatterns(asm: &String, expected: &[u16]) {
        for (instr, exp) in parse_asm_str(&asm).unwrap().iter().zip(expected.iter()) {
            assert_eq!(*exp, instr.get_opcode());
        }
    }

    #[test]
    fn test_blank_lines_ignored() {
        let asm = "
        JP t
        \t
        t:
        DRW V0, V1, 6
             \n\
        ADD V0, 0x34".to_string();
        let expected: Vec<u16> = vec![0x1202, 0xD016, 0x7034];

        assert_asm_bitpatterns(&asm, &expected);
    }

    #[test]
    fn test_single_line_comments_ignored() {
        let asm = "
        // This is a single line comment
        DRW V5, V2, 7
        LD F, V9 // This is one the end of a line
        LD ST, V2// This one has no space after the arg
        CLS//This one LD ST, V7 includes an instr".to_string();
        let expected: Vec<u16> = vec![0xD527, 0xF929, 0xF218, 0x00E0];

        assert_asm_bitpatterns(&asm, &expected);
    }

    #[test]
    fn branch_back_to_label() {
        let asm = "
            SYS 0x000
            // Comment lines don't increment address
        foo:
            SYS 0x111
        bar:: // Should be refered to as 'bar:' just fine
            JP foo
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
        let asm = "
            JP next
        next:
            CALL next
            JP aardvark".to_string();

        let _ = parse_asm_str(&asm).unwrap();
    }

    #[test]
    fn symbol_capable_instructions() {
        let asm ="
            SYS 0x000
            SYS 0x000
        target:
            SYS target
            JP target
            CALL target
            LD I, target".to_string();
        let expected: Vec<u16> = vec![0x0000, 0x0000, 0x0204, 0x1204, 0x2204, 0xA204];

        assert_asm_bitpatterns(&asm, &expected);
    }
    
    #[test]
    #[should_panic(expected="Cannot get address for unresolved symbol \"xyz\"")]
    fn get_opcode_on_unresolved_symbol_panics() {
        let ins = JumpInstr::create_with_symbol("xyz".to_string());
        ins.get_opcode();
    }

    #[test]
    fn handling_symbol_instrs() {
        // First make sure they all have symbol create
        let sym = "foo".to_string();
        let instrs = [
            Box::new(          SysInstr::create_with_symbol(sym.to_owned())) as Box<Instr>,
            Box::new(         JumpInstr::create_with_symbol(sym.to_owned())) as Box<Instr>,
            Box::new(         CallInstr::create_with_symbol(sym.to_owned())) as Box<Instr>,
            Box::new(        LoadIInstr::create_with_symbol(sym.to_owned())) as Box<Instr>,
            Box::new(JumpPlusVZeroInstr::create_with_symbol(sym.to_owned())) as Box<Instr>,
        ];

        // They can all repr with the symbol name
        let expected_repr = [
            "SYS foo",
            "JP foo",
            "CALL foo",
            "LD I, foo",
            "JP V0, foo",
        ];
        for (ins, expected) in instrs.iter().zip(expected_repr.iter()) {
            assert_eq!(String::from(*expected), ins.repr());
        }
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

        // Addr < 0xfff, just uses a single instr
        let asm4 = "LD I, 0x123".to_string();
        let expected4: Vec<u16> = vec![0xA123];
        assert_asm_bitpatterns(&asm4, &expected4);
    }

    #[test]
    fn word_directive() {
        let asm = "
            CALL end // Jump to check that .word does increment the address
            .word 0x1234
            ADD V0, V1
            .word 0x0000
            .word 0x2111 // Value can overlap real instructions
        end:
            ADD V2, V3".to_string();
        let expected: Vec<u16> = vec![0x220A, 0x1234, 0x8014, 0x0000, 0x2111, 0x8234];
        assert_asm_bitpatterns(&asm, &expected);
    }

    #[test]
    fn asm_err_messages() {
        let tests: Vec<(&str, &str)> = vec![
// I know this indentation is weird, but I'm sick of typing slash n
("FOOD", 
"\
<str>:0:0: error: Can't get number of args for mnemonic: FOOD
FOOD
^~~~"),
("CLS V0",
"\
<str>:0:0: error: Expected 0 args for CLS, got 1
CLS V0
^~~"),
("SHR z0",
"\
<str>:0:4: error: VX arg does not begin with \"V\"
SHR z0
    ^~"),
("SHL V21",
"\
<str>:0:4: error: V register index cannot be > 0xF
SHL V21
    ^~~"),
("SKP Vfood",
"\
<str>:0:4: error: Invalid V register: \"Vfood\"
SKP Vfood
    ^~~~~"),
("SKP food",
"\
<str>:0:4: error: VX arg does not begin with \"V\"
SKP food
    ^~~~"),
("SKP f",
"\
<str>:0:4: error: VX arg does not begin with \"V\"
SKP f
    ^"),
("SKNP V1F",
"\
<str>:0:5: error: V register index cannot be > 0xF
SKNP V1F
     ^~~"),
("SUB f0, V2",
"\
<str>:0:4: error: VX arg does not begin with \"V\"
SUB f0, V2
    ^~"),
("SUBN V0, Z0",
"\
<str>:0:9: error: VX arg does not begin with \"V\"
SUBN V0, Z0
         ^~"),
("XOR V21, V0",
"\
<str>:0:4: error: V register index cannot be > 0xF
XOR V21, V0
    ^~~"),
("XOR V1, V33",
"\
<str>:0:8: error: V register index cannot be > 0xF
XOR V1, V33
        ^~~"),
("AND 0x12, V0",
"\
<str>:0:4: error: VX arg does not begin with \"V\"
AND 0x12, V0
    ^~~~"),
("AND V0, 32",
"\
<str>:0:8: error: VX arg does not begin with \"V\"
AND V0, 32
        ^~"),
// Had an issue with single char args
("OR V0, 3",
"\
<str>:0:7: error: VX arg does not begin with \"V\"
OR V0, 3
       ^"),
("OR 1, vf",
"\
<str>:0:3: error: VX arg does not begin with \"V\"
OR 1, vf
   ^"),
("ADD I, nonsense",
"\
<str>:0:7: error: VX arg does not begin with \"V\"
ADD I, nonsense
       ^~~~~~~~"),
("ADD stuff, things",
"\
<str>:0:4: error: Invalid args for ADD instruction
ADD stuff, things
    ^~~~~~~~~~~~~"),
        ];
        for (input, expected_err) in tests {
            match parse_asm_str(&String::from(input)) {
                Err(msg) => {
                    assert_eq!(expected_err, msg);
                }
                Ok(_) => panic!("Expected an error here!"),
            }
        }
    }
}

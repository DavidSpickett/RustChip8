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
            "SYS 0x123".to_string(),
            "CLS".to_string(),
            "RET".to_string(),
            "JP 0x123".to_string(),
            "CALL 0x123".to_string(),
            "SHR V0".to_string(),
            "SHL V0".to_string(),
            "SKP V0".to_string(),
            "SKNP V0".to_string(),
            "DRW V0, V1, 12".to_string(),
            "AND V0, V1".to_string(),
            "XOR V0, V1".to_string(),
            "OR V0, V1".to_string(),
            "SUB V0, V1".to_string(),
            "SUBN V0, V1".to_string(),
            "RND V0, 0x12".to_string(),
        ];
        let got = parse_asm(&expected);
        for (e, g) in expected.iter().zip(got.iter()) {
            assert_eq!(*e, g.repr());
        }
    }
}

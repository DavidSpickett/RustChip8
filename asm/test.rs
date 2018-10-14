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
        ].iter().map(|x| x.to_string()).collect::<Vec<String>>();
        let got = parse_asm(&expected);
        for (e, g) in expected.iter().zip(got.iter()) {
            assert_eq!(*e, g.repr());
        }
    }
}

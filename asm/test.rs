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
}

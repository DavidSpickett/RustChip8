mod system;

fn main() {
    let mut c8 = system::make_system(String::from("INVADERS"));

    loop {
        c8.do_opcode();
    }
}

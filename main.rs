mod system;
mod sdl;
use system::make_system;
use system::read_rom;
use sdl::{sdl_init, process_events, draw_screen, read_keys, wait_on_key};

pub fn main() {
    let pixel_size: i32 = 10;
    let (mut canvas, mut event_pump) = sdl_init(pixel_size);

    let rom_name = String::from("roms/INVADERS");
    let mut c8 = make_system(&read_rom(&rom_name));

    'running: loop {
        if process_events(&mut event_pump) {
            break 'running
        }
 
        let instr = c8.fetch_and_decode();
        let flags = instr.get_flags();
        match flags {
            system::InstrFlags::Keys => c8.update_keys(read_keys(&mut event_pump)),
            system::InstrFlags::WaitKey => {
                c8.pressed_key = wait_on_key(&mut event_pump);
                if c8.pressed_key == 16 {
                    break 'running
                }
            }
            _ => {},
        }

        c8.execute(&instr);

        match flags {
            system::InstrFlags::Screen => draw_screen(pixel_size, &mut canvas, &c8.screen),
            _ => {},
        }
    }
}

extern crate sdl2;
use std::io::ErrorKind;

use chiprs::Chip8;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use std::env;
use std::time::Duration;

const INSTRUCTIONS_PER_SECOND: u32 = 700;

fn main() {
    let args: Vec<String> = env::args().collect();
    let file_path = match &args[..] {
        [_, path] => path,
        _ => {
            eprintln!("USAGE: <script> <program.ch8>");
            std::process::exit(1);
        }
    };
    let program = match std::fs::read(file_path) {
        Ok(program) => program,
        Err(e) => {
            match e.kind() {
                ErrorKind::NotFound => eprintln!("{file_path} does not exist."),
                ErrorKind::PermissionDenied => eprintln!("no read permissions for {file_path}"),
                _ => eprintln!("Unexpected error: {e}"),
            }
            std::process::exit(1);
        }
    };
    let mut emulator = Chip8::load_program(&program);

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let window = video_subsystem
        .window("chip-8", 640, 320)
        .position_centered()
        .build()
        .expect("Unable to build sdl2 window");

    let mut canvas = window.into_canvas().build().unwrap();

    canvas.set_draw_color(Color::BLACK);
    canvas.clear();
    canvas.present();
    let mut event_pump = sdl_context.event_pump().unwrap();
    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                Event::KeyDown {
                    keycode: Some(code),
                    ..
                } 
                    /*
                    1 2 3 4
                    q w e r
                    a s d f
                    z x c v
                    */
                => {
                    todo!();
                },
                _ => {}
            }
        }

        emulator.step();
        canvas.set_draw_color(Color::BLACK);
        canvas.clear();
        canvas.set_draw_color(Color::WHITE);
        for (y, row) in emulator.display.iter().enumerate() {
            for (x, pixel) in row.iter().enumerate() {
                if *pixel {
                    let white_box = sdl2::rect::Rect::new(x as i32 * 10, y as i32 * 10, 10, 10);
                    canvas.fill_rect(white_box).unwrap();
                }
            }
        }
        canvas.present();
        std::thread::sleep(Duration::new(0, 1_000_000_000u32 / INSTRUCTIONS_PER_SECOND));
    }
}

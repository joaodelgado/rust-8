extern crate itertools;
extern crate rand;
extern crate sdl2;

use std::env;
use std::fs::File;
use std::time::Duration;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;

mod instr;
mod cpu;
mod display;
mod spec;

fn main() {

    let file_name = env::args().nth(1).expect("Provide a rom as the first argument.");

    // Read rom file
    println!("Reading from {}", file_name);
    let file = File::open(file_name).unwrap();

    // Initialize SDL
    let sdl_context = sdl2::init().unwrap();
    let mut event_pump = sdl_context.event_pump().unwrap();

    // Initialize VM
    let mut cpu = cpu::Cpu::new(&sdl_context, &file);
    println!("Initial state: {}", cpu);

    let mut running = true;
    let mut stepping = true;
    let mut execute = false;
    while running {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } |
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    running = false;
                }
                Event::KeyDown { keycode: Some(Keycode::S), .. } => {
                    stepping = !stepping;
                    println!("Stepping: {}", stepping);
                    execute = !stepping
                }
                Event::KeyDown { keycode: Some(Keycode::Space), .. } => {
                    execute = true;
                }
                _ => {}
            }
        }

        if execute {
            let instr = cpu.read_instr();

            let cmd = instr::parse(instr);
            println!("Read: {}", cmd);

            instr::execute(cmd, &mut cpu);
            println!("Current state: {}", cpu);

            execute = !stepping;
        }

        cpu.get_display().flush();

        std::thread::sleep(Duration::from_millis(10));
    }

    // let mut running = true;
    // while running {
    // let instr = cpu.read_instr();

    // let cmd = instr::parse(instr);
    // println!("Read: {}", cmd);
    // instr::execute(cmd, &mut cpu);
    // println!("Current state: {}", cpu);


    // let mut waiting = true;
    // while waiting {
    // let mut input = String::new();
    // io::stdin().read_line(&mut input).unwrap();
    // input = input.trim().to_string();

    // if input.starts_with('p') {
    // println!("Current state: {}", cpu);
    // } else if input.is_empty() {
    // waiting = false;
    // } else {
    // waiting = false;
    // running = false;
    // }
    // }
    // }

    // println!("Final state: {}", cpu);
}

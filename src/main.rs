extern crate itertools;
extern crate rand;
extern crate sdl2;
extern crate time;

use std::env;
use std::fs::File;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;

mod instr;
mod cpu;
mod display;
mod spec;

fn tick(cpu: &mut cpu::Cpu, debug: bool) {
    let instr = cpu.read_instr();
    let cmd = instr::parse(instr);

    if debug {
        println!("Read: {}", cmd);
    }

    cpu.dec_dt();
    instr::execute(cmd, cpu);

    if debug {
        println!("Current state: {}", cpu);
    }
}

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
                    execute = !stepping;
                    cpu.reset_sync();
                    println!("Stepping: {}", stepping);
                    if stepping {
                        println!("Current state: {}", cpu);
                    }
                }
                Event::KeyDown { keycode: Some(Keycode::Space), .. } => {
                    execute = true;
                }
                _ => {}
            }
        }

        if execute {
            tick(&mut cpu, stepping);
            execute = !stepping;
        }

        cpu.get_display().flush();

        cpu.sync();
    }
}

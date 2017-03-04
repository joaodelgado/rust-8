extern crate itertools;
extern crate rand;
extern crate sdl2;
extern crate time;

use std::env;
use std::fs::File;

mod cpu;
mod display;
mod instr;
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

    while cpu.is_running() {
        cpu.tick(&mut event_pump);
    }
}

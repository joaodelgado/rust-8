use std::cmp::max;
use std::fmt;
use std::fs::File;
use std::io::Read;
use std::io::Seek;
use std::io::SeekFrom;
use std::thread;
use std::time::Duration;

use itertools::join;

use sdl2::Sdl;

use time::PreciseTime;

use display::Display;
use spec;

pub struct Cpu<'a> {
    display: Display<'a>,
    cur_instr: u16,
    last_sync: PreciseTime,

    // Registers
    r_vx: [u8; 16],
    r_i: u16,
    r_dt: u8,
    r_st: u8,
    r_pc: u16,
    r_sp: u8,
    stack: [u16; 16],
    mem: [u8; 4096],
}

impl<'a> Cpu<'a> {
    /// Initialize the CPU with all registers at 0
    pub fn new(sdl_context: &Sdl, rom_file: &'a File) -> Cpu<'a> {
        let mut mem = [0u8; spec::MEM_SIZE];

        Cpu::load_sprites(&mut mem);
        Cpu::load_rom(&mut mem, rom_file);

        Cpu {
            display: Display::new(sdl_context),
            cur_instr: 0,
            last_sync: PreciseTime::now(),

            r_vx: [0; 16],
            r_i: 0,
            r_dt: 0,
            r_st: 0,
            r_pc: spec::PROGRAM_START as u16,
            r_sp: 0,
            stack: [0; 16],
            mem: mem,
        }
    }

    /// Load the built in font sprites
    #[cfg_attr(rustfmt, rustfmt_skip)]
    fn load_sprites(mem: &mut [u8]) {
        let sprites = [
            0b11110000, 0b00100000, 0b11110000, 0b11110000, 0b10010000, 0b11110000, 0b11110000,
            0b10010000, 0b01100000, 0b00010000, 0b00010000, 0b10010000, 0b10000000, 0b10000000,
            0b10010000, 0b00100000, 0b11110000, 0b11110000, 0b11110000, 0b11110000, 0b11110000,
            0b10010000, 0b00100000, 0b10000000, 0b00010000, 0b00010000, 0b00010000, 0b10010000,
            0b11110000, 0b01110000, 0b11110000, 0b11110000, 0b00010000, 0b11110000, 0b11110000,

            0b11110000, 0b11110000, 0b11110000, 0b11110000, 0b11100000, 0b11110000, 0b11110000,
            0b00010000, 0b10010000, 0b10010000, 0b10000000, 0b10010000, 0b10000000, 0b10000000,
            0b00100000, 0b11110000, 0b11110000, 0b10000000, 0b10010000, 0b11110000, 0b11110000,
            0b01000000, 0b10010000, 0b00010000, 0b10000000, 0b10010000, 0b10000000, 0b10000000,
            0b01000000, 0b11110000, 0b11110000, 0b11110000, 0b11100000, 0b11110000, 0b10000000,
        ];

        for i in 0 .. sprites.len() {
            mem[i] = sprites[i];
        }
    }

    /// Read the whole rom file and dumps it into memory
    fn load_rom(mem: &mut [u8], mut rom_file: &File) {
        let mut buf: Vec<u8> = Vec::new();

        // Ensure that we are reading from the beginning of the file
        rom_file.seek(SeekFrom::Start(0)).unwrap();
        rom_file.read_to_end(&mut buf).unwrap();

        for i in 0..buf.len() {
            mem[i + spec::PROGRAM_START] = buf[i];
        }
    }

    /// Reads the next instruction on the rom.
    /// The position is set by the current value of PC
    pub fn read_instr(&mut self) -> u16 {
        let instr = ((self.mem[self.r_pc as usize] as u16) << 8) |
                    self.mem[self.r_pc as usize + 1] as u16;

        self.inc_pc();
        self.cur_instr = instr;
        instr
    }

    /// Read n bytes from memory, starting at addr
    pub fn read_mem(&mut self, addr: usize, n: usize) -> Vec<u8> {
        self.mem[addr..(addr + n)].to_vec()
    }

    /// Read n bytes from memory, starting at addr
    pub fn put_mem(&mut self, addr: usize, value: u8) {
        self.mem[addr] = value;
    }

    /// Sets the PC register to a given address.
    pub fn get_pc(&self) -> u16 {
        self.r_pc
    }

    /// Increments the PC to the next instruction
    pub fn inc_pc(&mut self) {
        let cur_pc = self.r_pc;
        self.set_pc(cur_pc + 2);
    }

    /// Sets the PC register to a given address.
    pub fn set_pc(&mut self, addr: u16) {
        self.r_pc = addr;
    }

    /// Increment the stack pointer and put value in the top of the stack
    pub fn push_stack(&mut self, value: u16) {
        self.r_sp += 1;
        self.stack[self.r_sp as usize] = value;
    }

    /// Gets the value at the top of the stack and then decrements the stack pointer
    pub fn pop_stack(&mut self) -> u16 {
        let value = self.stack[self.r_sp as usize];
        self.r_sp -= 1;

        value
    }

    /// Gets the value of the Vx register.
    pub fn get_vx(&self, reg: usize) -> u8 {
        self.r_vx[reg]
    }

    /// Sets the Vx register to a given value, where x in the given index.
    pub fn set_vx(&mut self, reg: usize, value: u8) {
        self.r_vx[reg] = value;
    }

    /// Gets the value of the Vx register.
    pub fn get_i(&self) -> u16 {
        self.r_i
    }

    /// Sets the i register to a given value.
    pub fn set_i(&mut self, value: u16) {
        self.r_i = value;
    }

    /// Get a mutable reference to the display
    pub fn get_display(&mut self) -> &mut Display<'a> {
        &mut self.display
    }

    /// Reset the last sync time to the current time
    pub fn reset_sync(&mut self) {
        self.last_sync = PreciseTime::now();
    }

    /// Sleep for the necessary time to sync to the desired FPS
    pub fn sync(&mut self) {
        let now = PreciseTime::now();

        let ellapsed = spec::MILLI_PER_FRAME as i64 - self.last_sync.to(now).num_milliseconds();
        let sleep = max(ellapsed, 0) as u64;

        self.reset_sync();
        thread::sleep(Duration::from_millis(sleep));
    }
}


impl<'a> fmt::Display for Cpu<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let r_vx = join(self.r_vx.into_iter().map(|v| format!("{:02x}", v)), ", ");
        let stack = join(self.stack.into_iter().map(|v| format!("{:04x}", v)), ", ");


        return write!(f,
                      "CPU [
    r_vx: {},
    r_i: {:04x},
    r_dt: {:02x},
    r_st: {:02x},
    r_pc: {:04x},
    r_sp: {:02x},
    stack: {},
]",
                      r_vx,
                      self.r_i,
                      self.r_dt,
                      self.r_st,
                      self.r_pc,
                      self.r_sp,
                      stack);
    }
}

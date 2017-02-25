use std::fmt;
use std::fs::File;
use std::io::Read;
use std::io::Seek;
use std::io::SeekFrom;

use itertools::join;

use sdl2::Sdl;

use display::Display;

pub struct Cpu<'a> {
    display: Display<'a>,
    rom: &'a File,
    cur_instr: u16,
    base: u16,

    // Registers
    r_vx: [u8; 16],
    r_i: u16,
    r_dt: u8,
    r_st: u8,
    r_pc: u16,
    r_sp: u8,
    stack: [u16; 16],
}

impl<'a> Cpu<'a> {
    /// Initialize the CPU with all registers at 0
    pub fn new(sdl_context: &Sdl, mut rom: &'a File) -> Cpu<'a> {
        rom.seek(SeekFrom::Start(0)).unwrap();
        let base = 0x200;

        Cpu {
            rom: rom,
            display: Display::new(sdl_context),
            cur_instr: 0,
            base: base,

            r_vx: [0; 16],
            r_i: 0,
            r_dt: 0,
            r_st: 0,
            r_pc: base,
            r_sp: 0,
            stack: [0; 16],
        }
    }

    /// Reads the next instruction on the rom.
    /// The position is set by the current value of PC
    pub fn read_instr(&mut self) -> u16 {
        let mut buf = [0u8; 2];

        self.rom.seek(SeekFrom::Start((self.r_pc - self.base) as u64)).unwrap();
        self.rom.read(&mut buf).unwrap();

        let instr = ((buf[0] as u16) << 8) | buf[1] as u16;

        self.inc_pc();
        self.cur_instr = instr;
        instr
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

    /// Increments the SP register by 1.
    pub fn inc_sp(&mut self) {
        self.r_sp += 1;
    }

    /// Sets a value on the stack where SP is currently pointing.
    pub fn set_stack(&mut self, value: u16) {
        self.stack[self.r_sp as usize] = value;
    }

    pub fn get_display(&mut self) -> &mut Display<'a> {
        &mut self.display
    }

    /// Read n bytes from memory, starting at addr
    pub fn read_mem(&mut self, addr: u16, n: u8) -> Vec<u8> {
        if addr < 0x200 {
            return vec![0; n as usize];
        }
        let mut buf = vec![0u8; n as usize];
        self.rom.seek(SeekFrom::Start((addr - self.base) as u64)).unwrap();
        self.rom.read(&mut buf).unwrap();

        buf.to_vec()
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

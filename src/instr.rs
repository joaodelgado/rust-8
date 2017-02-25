use std::fmt;
use std::boxed::Box;

use rand;
use rand::Rng;

use cpu::Cpu;
use display::Pixel;
use spec;

pub trait Instr: fmt::Display {
    fn parse(&mut self, instr: u16);
    fn execute(&self, cpu: &mut Cpu);
}

/// *1nnn - JP addr* :: Jump to location nnn.
///
/// The interpreter sets the program counter to nnn.
#[derive(Default)]
struct Jp {
    raw: u16,
    addr: u16,
}


impl Instr for Jp {
    fn parse(&mut self, instr: u16) {
        self.raw = instr;
        self.addr = instr & 0x0fff;
    }

    fn execute(&self, cpu: &mut Cpu) {
        cpu.set_pc(self.addr);
    }
}

impl fmt::Display for Jp {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:04x} - JP {:03x}", self.raw, self.addr)
    }
}

/// *2nnn - CALL addr* :: Call subroutine at nnn.
///
/// The interpreter increments the stack pointer, then puts the current PC on the
/// top of the stack. The PC is then set to nnn.
#[derive(Default)]
struct Call {
    raw: u16,
    addr: u16,
}

impl Instr for Call {
    fn parse(&mut self, instr: u16) {
        self.raw = instr;
        self.addr = instr & 0x0fff;
    }

    fn execute(&self, cpu: &mut Cpu) {
        // Increment SP and store the current PC
        let cur_pc = cpu.get_pc();
        cpu.inc_sp();
        cpu.set_stack(cur_pc);

        // Set the PC to the new address
        cpu.set_pc(self.addr / 4);
    }
}

impl fmt::Display for Call {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:04x} - CALL {:03x}", self.raw, self.addr)
    }
}

/// *3xkk - SE Vx, byte* :: Skip next instruction if Vx = kk.
///
/// The interpreter compares register Vx to kk, and if they are equal, increments
/// the program counter by 2.
#[derive(Default)]
struct Se {
    raw: u16,
    reg: usize,
    value: u8,
}

impl Instr for Se {
    fn parse(&mut self, instr: u16) {
        self.raw = instr;
        self.reg = ((instr & 0x0f00) >> 8) as usize;
        self.value = (instr & 0x00ff) as u8;
    }

    fn execute(&self, cpu: &mut Cpu) {
        if cpu.get_vx(self.reg) == self.value {
            cpu.inc_pc();
        }
    }
}

impl fmt::Display for Se {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,
               "{:04x} - SE V{:x}, {:02x}",
               self.raw,
               self.reg,
               self.value)
    }
}

/// *6xkk - LD Vx, byte* :: Set Vx = kk.
///
/// The interpreter puts the value kk into register Vx.
#[derive(Default)]
struct Ld {
    raw: u16,
    reg: usize,
    value: u8,
}

impl Instr for Ld {
    fn parse(&mut self, instr: u16) {
        self.raw = instr;
        self.reg = ((instr & 0x0f00) >> 8) as usize;
        self.value = (instr & 0x00ff) as u8;
    }

    fn execute(&self, cpu: &mut Cpu) {
        cpu.set_vx(self.reg, self.value);
    }
}

impl fmt::Display for Ld {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,
               "{:04x} - LD V{:x}, {:02x}",
               self.raw,
               self.reg,
               self.value)
    }
}

/// *7xkk - ADD Vx, byte* :: Set Vx = Vx + kk.
///
/// Adds the value kk to the value of register Vx, then stores the result in Vx.
#[derive(Default)]
struct Add {
    raw: u16,
    reg: usize,
    value: u8,
}

impl Instr for Add {
    fn parse(&mut self, instr: u16) {
        self.raw = instr;
        self.reg = ((instr & 0x0f00) >> 8) as usize;
        self.value = (instr & 0x00ff) as u8;
    }

    fn execute(&self, cpu: &mut Cpu) {
        let new_value = cpu.get_vx(self.reg) + self.value;
        cpu.set_vx(self.reg, new_value);
    }
}

impl fmt::Display for Add {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,
               "{:04x} - ADD V{:x}, {:02x}",
               self.raw,
               self.reg,
               self.value)
    }
}


/// *Annn - LD I, addr* :: Set I = nnn.
///
/// The value of register I is set to nnn.
#[derive(Default)]
struct LdI {
    raw: u16,
    addr: u16,
}

impl Instr for LdI {
    fn parse(&mut self, instr: u16) {
        self.raw = instr;
        self.addr = instr & 0x0fff;
    }

    fn execute(&self, cpu: &mut Cpu) {
        cpu.set_i(self.addr);
    }
}

impl fmt::Display for LdI {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:04x} - LD I, {:03x}", self.raw, self.addr)
    }
}

/// *Cxkk - RND Vx, byte* :: Set Vx = random byte AND kk.
///
/// The interpreter generates a random number from 0 to 255, which is then ANDed
/// with the value kk. The results are stored in Vx. See instruction 8xy2 for more
/// information on AND.
#[derive(Default)]
struct Rnd {
    raw: u16,
    reg: usize,
    value: u8,
}

impl Instr for Rnd {
    fn parse(&mut self, instr: u16) {
        self.raw = instr;
        self.reg = ((instr & 0x0f00) >> 8) as usize;
        self.value = (instr & 0x00ff) as u8;
    }

    fn execute(&self, cpu: &mut Cpu) {
        let rnd_byte = rand::thread_rng().gen::<u8>();
        cpu.set_vx(self.reg, rnd_byte & self.value);
    }
}

impl fmt::Display for Rnd {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,
               "{:04x} - RND V{:x}, {:02x}",
               self.raw,
               self.reg,
               self.value)
    }
}

/// *Dxyn - DRW Vx, Vy, nibble* :: Display n-byte sprite starting at memory location I at (Vx, Vy), set VF = collision.
///
/// The interpreter reads n bytes from memory, starting at the address stored in I.
/// These bytes are then displayed as sprites on screen at coordinates (Vx, Vy).
/// Sprites are XORed onto the existing screen. If this causes any pixels to be
/// erased, VF is set to 1, otherwise it is set to 0. If the sprite is positioned
/// so part of it is outside the coordinates of the display, it wraps around to the
/// opposite side of the screen. See instruction 8xy3 for more information on XOR,
/// and section 2.4, Display, for more information on the Chip-8 screen and
/// sprites.
#[derive(Default)]
struct Drw {
    raw: u16,
    x: usize,
    y: usize,
    n: u8,
}

impl Instr for Drw {
    fn parse(&mut self, instr: u16) {
        self.raw = instr;
        self.x = ((instr & 0x0f00) >> 8) as usize;
        self.y = ((instr & 0x00f0) >> 4) as usize;
        self.n = (instr & 0x000f) as u8;
    }

    #[allow(unused_variables)]
    fn execute(&self, cpu: &mut Cpu) {
        let x = cpu.get_vx(self.x);
        let y = cpu.get_vx(self.y);
        let i = cpu.get_i();
        let n = self.n;

        // Set VF as 0 by default.
        let mut vf = 0;

        // Read data to be drawn
        let raw_bytes = cpu.read_mem(i, n);

        let mut pixels: Vec<Pixel> = vec![];
        for (iter_y, byte) in raw_bytes.iter().enumerate() {
            // Get the wrapped y coord
            let dy = (y as u32 + iter_y as u32) % spec::DISPLAY_HEIGHT;
            for iter_x in 0..8 {
                // Get the wrapped x coord
                let dx = (x as u32 + iter_x as u32) % spec::DISPLAY_WIDTH;

                // Get the new and old bit value for the current pixel
                let px = byte >> (7 - iter_x) & 0x01u8;
                let old_px = cpu.get_display().get_pixel(dx as usize, dy as usize);

                // Calculate the new pixel value
                // and store any collision in VF
                let new_px = old_px ^ px;
                if old_px == 1 && new_px == 0 {
                    vf = 1
                }

                // Push the pixel to the pixels to be drawn
                let pixel = Pixel::new(dx as usize, dy as usize, new_px);
                pixels.push(pixel);
            }
        }

        cpu.set_vx(0xf, vf);
        cpu.get_display().draw(pixels);
    }
}

impl fmt::Display for Drw {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,
               "{:04x} - DRW V{:x}, V{:x} {:x}",
               self.raw,
               self.x,
               self.y,
               self.n)
    }
}

///
///
///
///

pub fn parse(raw: u16) -> Box<Instr> {
    let mut instr: Box<Instr> = match raw & 0xf000 {
        0x1000 => Box::new(Jp::default()),
        0x2000 => Box::new(Call::default()),
        0x3000 => Box::new(Se::default()),
        0x6000 => Box::new(Ld::default()),
        0x7000 => Box::new(Add::default()),
        0xa000 => Box::new(LdI::default()),
        0xc000 => Box::new(Rnd::default()),
        0xd000 => Box::new(Drw::default()),
        _ => panic!("unsupported instruction: {:04x}", raw),
    };

    instr.parse(raw);
    instr
}

pub fn execute(inst: Box<Instr>, cpu: &mut Cpu) {
    inst.execute(cpu)
}

///
///
///
///

/// *0nnn - SYS addr* :: Jump to a machine code routine at nnn.
///
/// This instruction is only used on the old computers on which Chip-8 was
/// originally implemented. It is ignored by modern interpreters.
#[allow(dead_code, unused_variables)]
pub fn sys_addr(cpu: &mut Cpu, instr: u16) {
    // TODO
}

/// *00E0 - CLS* :: Clear the display.
#[allow(dead_code, unused_variables)]
pub fn cls(cpu: &mut Cpu, instr: u16) {
    // TODO
}

/// *00EE - RET* :: Return from a subroutine.
///
/// The interpreter sets the program counter to the address at the top of the
/// stack, then subtracts 1 from the stack pointer.
#[allow(dead_code, unused_variables)]
pub fn ret(cpu: &mut Cpu, instr: u16) {
    // TODO
}

/// *4xkk - SNE Vx, byte* :: Skip next instruction if Vx != kk.
///
/// The interpreter compares register Vx to kk, and if they are not equal,
/// increments the program counter by 2.
#[allow(dead_code, unused_variables)]
pub fn sne_vx_byte(cpu: &mut Cpu, instr: u16) {
    // TODO
}

/// *5xy0 - SE Vx, Vy* :: Skip next instruction if Vx = Vy.
///
/// The interpreter compares register Vx to register Vy, and if they are equal,
/// increments the program counter by 2.
#[allow(dead_code, unused_variables)]
pub fn se_vx_vy(cpu: &mut Cpu, instr: u16) {
    // TODO
}

/// *8xy0 - LD Vx, Vy* :: Set Vx = Vy.
///
/// Stores the value of register Vy in register Vx.
#[allow(dead_code, unused_variables)]
pub fn ld_vx_vy(cpu: &mut Cpu, instr: u16) {
    // TODO
}

/// *8xy1 - OR Vx, Vy* :: Set Vx = Vx OR Vy.
///
/// Performs a bitwise OR on the values of Vx and Vy, then stores the result in Vx.
/// A bitwise OR compares the corrseponding bits from two values, and if either bit
/// is 1, then the same bit in the result is also 1. Otherwise, it is 0.
#[allow(dead_code, unused_variables)]
pub fn or_vx_vy(cpu: &mut Cpu, instr: u16) {
    // TODO
}

/// *8xy2 - AND Vx, Vy* :: Set Vx = Vx AND Vy.
///
/// Performs a bitwise AND on the values of Vx and Vy, then stores the result in
/// Vx. A bitwise AND compares the corrseponding bits from two values, and if both
/// bits are 1, then the same bit in the result is also 1. Otherwise, it is 0.
#[allow(dead_code, unused_variables)]
pub fn and_vx_vy(cpu: &mut Cpu, instr: u16) {
    // TODO
}

/// *8xy3 - XOR Vx, Vy* :: Set Vx = Vx XOR Vy.
///
/// Performs a bitwise exclusive OR on the values of Vx and Vy, then stores the
/// result in Vx. An exclusive OR compares the corrseponding bits from two values,
/// and if the bits are not both the same, then the corresponding bit in the result
/// is set to 1. Otherwise, it is 0.
#[allow(dead_code, unused_variables)]
pub fn xor_vx_vy(cpu: &mut Cpu, instr: u16) {
    // TODO
}

/// *8xy4 - ADD Vx, Vy* :: Set Vx = Vx + Vy, set VF = carry.
///
/// The values of Vx and Vy are added together. If the result is greater than
/// 8 bits (i.e., > 255,) VF is set to 1, otherwise 0. Only the lowest 8 bits of
/// the result are kept, and stored in Vx.
#[allow(dead_code, unused_variables)]
pub fn add_vx_vy(cpu: &mut Cpu, instr: u16) {
    // TODO
}

/// *8xy5 - SUB Vx, Vy* :: Set Vx = Vx - Vy, set VF = NOT borrow.
///
/// If Vx > Vy, then VF is set to 1, otherwise 0. Then Vy is subtracted from Vx,
/// and the results stored in Vx.
#[allow(dead_code, unused_variables)]
pub fn sub_vx_vy(cpu: &mut Cpu, instr: u16) {
    // TODO
}

/// *8xy6 - SHR Vx {, Vy}* :: Set Vx = Vx SHR 1.
///
/// If the least-significant bit of Vx is 1, then VF is set to 1, otherwise 0. Then
/// Vx is divided by 2.
#[allow(dead_code, unused_variables)]
pub fn shr_vx_vy(cpu: &mut Cpu, instr: u16) {
    // TODO
}

/// *8xy7 - SUBN Vx, Vy* :: Set Vx = Vy - Vx, set VF = NOT borrow.
///
/// If Vy > Vx, then VF is set to 1, otherwise 0. Then Vx is subtracted from Vy,
/// and the results stored in Vx.
#[allow(dead_code, unused_variables)]
pub fn subn_vx_vy(cpu: &mut Cpu, instr: u16) {
    // TODO
}

/// *8xyE - SHL Vx {, Vy}* :: Set Vx = Vx SHL 1.
///
/// If the most-significant bit of Vx is 1, then VF is set to 1, otherwise to 0.
/// Then Vx is multiplied by 2.
#[allow(dead_code, unused_variables)]
pub fn shl_vx_vy(cpu: &mut Cpu, instr: u16) {
    // TODO
}

/// *9xy0 - SNE Vx, Vy* :: Skip next instruction if Vx != Vy.
///
/// The values of Vx and Vy are compared, and if they are not equal, the program
/// counter is increased by 2.
#[allow(dead_code, unused_variables)]
pub fn sne_vx_vy(cpu: &mut Cpu, instr: u16) {
    // TODO
}

/// *Bnnn - JP V0, addr* :: Jump to location nnn + V0.
///
/// The program counter is set to nnn plus the value of V0.
#[allow(dead_code, unused_variables)]
pub fn jp_v0_addr(cpu: &mut Cpu, instr: u16) {
    // TODO
}

/// *Ex9E - SKP Vx* :: Skip next instruction if key with the value of Vx is pressed.
///
/// Checks the keyboard, and if the key corresponding to the value of Vx is
/// currently in the down position, PC is increased by 2.
#[allow(dead_code, unused_variables)]
pub fn skp_vx(cpu: &mut Cpu, instr: u16) {
    // TODO
}

/// *ExA1 - SKNP Vx* :: Skip next instruction if key with the value of Vx is not pressed.
///
/// Checks the keyboard, and if the key corresponding to the value of Vx is
/// currently in the up position, PC is increased by 2.
#[allow(dead_code, unused_variables)]
pub fn sknp_vx(cpu: &mut Cpu, instr: u16) {
    // TODO
}

/// *Fx07 - LD Vx, DT* :: Set Vx = delay timer value.
///
/// The value of DT is placed into Vx.
#[allow(dead_code, unused_variables)]
pub fn ld_vx_dt(cpu: &mut Cpu, instr: u16) {
    // TODO
}

/// *Fx0A - LD Vx, K* :: Wait for a key press, store the value of the key in Vx.
///
/// All execution stops until a key is pressed, then the value of that key is
/// stored in Vx.
#[allow(dead_code, unused_variables)]
pub fn ld_vx_k(cpu: &mut Cpu, instr: u16) {
    // TODO
}

/// *Fx15 - LD DT, Vx* :: Set delay timer = Vx.
///
/// DT is set equal to the value of Vx.
#[allow(dead_code, unused_variables)]
pub fn ld_dt_vx(cpu: &mut Cpu, instr: u16) {
    // TODO
}

/// *Fx18 - LD ST, Vx* :: Set sound timer = Vx.
///
/// ST is set equal to the value of Vx.
#[allow(dead_code, unused_variables)]
pub fn ld_st_vx(cpu: &mut Cpu, instr: u16) {
    // TODO
}

/// *Fx1E - ADD I, Vx* :: Set I = I + Vx.
///
/// The values of I and Vx are added, and the results are stored in I.
#[allow(dead_code, unused_variables)]
pub fn add_i_vx(cpu: &mut Cpu, instr: u16) {
    // TODO
}

/// *Fx29 - LD F, Vx* :: Set I = location of sprite for digit Vx.
///
/// The value of I is set to the location for the hexadecimal sprite corresponding
/// to the value of Vx. See section 2.4, Display, for more information on the
/// Chip-8 hexadecimal font.
#[allow(dead_code, unused_variables)]
pub fn ld_f_vx(cpu: &mut Cpu, instr: u16) {
    // TODO
}

/// *Fx33 - LD B, Vx* :: Store BCD representation of Vx in memory locations I, I+1, and I+2.
///
/// The interpreter takes the decimal value of Vx, and places the hundreds digit in
/// memory at location in I, the tens digit at location I+1, and the ones digit at
/// location I+2.
#[allow(dead_code, unused_variables)]
pub fn ld_b_vx(cpu: &mut Cpu, instr: u16) {
    // TODO
}

/// *Fx55 - LD [I], Vx* :: Store registers V0 through Vx in memory starting at location I.
///
/// The interpreter copies the values of registers V0 through Vx into memory,
/// starting at the address in I.
#[allow(dead_code, unused_variables)]
pub fn ld_i_vx(cpu: &mut Cpu, instr: u16) {
    // TODO
}

/// *Fx65 - LD Vx, [I]* :: Read registers V0 through Vx from memory starting at location I.
///
/// The interpreter reads values from memory starting at location I into registers
/// V0 through Vx.
#[allow(dead_code, unused_variables)]
pub fn ld_vx_i(cpu: &mut Cpu, instr: u16) {
    // TODO
}

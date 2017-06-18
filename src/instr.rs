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

/// *00E0 - CLS* :: Clear the display.
#[derive(Default)]
struct Cls {
    raw: u16,
}


impl Instr for Cls {
    fn parse(&mut self, instr: u16) {
        self.raw = instr;
    }

    fn execute(&self, cpu: &mut Cpu) {
        cpu.get_display().clear();
    }
}

impl fmt::Display for Cls {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:04x} - CLS", self.raw)
    }
}


/// *00EE - RET* :: Return from a subroutine.
///
/// The interpreter sets the program counter to the address at the top of the
/// stack, then subtracts 1 from the stack pointer.
#[derive(Default)]
struct Ret {
    raw: u16,
}


impl Instr for Ret {
    fn parse(&mut self, instr: u16) {
        self.raw = instr;
    }

    fn execute(&self, cpu: &mut Cpu) {
        let new_pc = cpu.pop_stack();
        cpu.set_pc(new_pc);
    }
}

impl fmt::Display for Ret {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:04x} - RET", self.raw)
    }
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
        // Store the current PC in the stack
        let cur_pc = cpu.get_pc();
        cpu.push_stack(cur_pc);

        // Set the PC to the new address
        cpu.set_pc(self.addr);
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
struct SeB {
    raw: u16,
    reg: usize,
    value: u8,
}

impl Instr for SeB {
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

impl fmt::Display for SeB {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,
               "{:04x} - SE V{:x}, {:02x}",
               self.raw,
               self.reg,
               self.value)
    }
}

/// *4xkk - SNE Vx, byte* :: Skip next instruction if Vx != kk.
///
/// The interpreter compares register Vx to kk, and if they are not equal,
/// increments the program counter by 2.
#[derive(Default)]
struct Sne {
    raw: u16,
    reg: usize,
    value: u8,
}

impl Instr for Sne {
    fn parse(&mut self, instr: u16) {
        self.raw = instr;
        self.reg = ((instr & 0x0f00) >> 8) as usize;
        self.value = (instr & 0x00ff) as u8;
    }

    fn execute(&self, cpu: &mut Cpu) {
        if cpu.get_vx(self.reg) != self.value {
            cpu.inc_pc();
        }
    }
}

impl fmt::Display for Sne {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,
               "{:04x} - SNE V{:x}, {:02x}",
               self.raw,
               self.reg,
               self.value)
    }
}

/// *5xy0 - SE Vx, Vy* :: Skip next instruction if Vx = Vy.
///
/// The interpreter compares register Vx to register Vy, and if they are equal,
/// increments the program counter by 2.
#[derive(Default)]
struct SeV {
    raw: u16,
    x: usize,
    y: usize,
}

impl Instr for SeV {
    fn parse(&mut self, instr: u16) {
        self.raw = instr;
        self.x = ((instr & 0x0f00) >> 8) as usize;
        self.y = ((instr & 0x00f0) >> 4) as usize;
    }

    fn execute(&self, cpu: &mut Cpu) {
        if cpu.get_vx(self.x) == cpu.get_vx(self.y) {
            cpu.inc_pc();
        }
    }
}

impl fmt::Display for SeV {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:04x} - SE V{:x}, V{:x}", self.raw, self.x, self.y)
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
        let new_value = cpu.get_vx(self.reg).wrapping_add(self.value);
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

/// *8xy0 - LD Vx, Vy* :: Set Vx = Vy.
///
/// Stores the value of register Vy in register Vx.
#[derive(Default)]
struct LdReg {
    raw: u16,
    x: usize,
    y: usize,
}

impl Instr for LdReg {
    fn parse(&mut self, instr: u16) {
        self.raw = instr;
        self.x = ((instr & 0x0f00) >> 8) as usize;
        self.y = ((instr & 0x00f0) >> 4) as usize;
    }

    fn execute(&self, cpu: &mut Cpu) {
        let new_value = cpu.get_vx(self.y);
        cpu.set_vx(self.x, new_value);
    }
}

impl fmt::Display for LdReg {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:04x} - LD V{:x}, V{:x}", self.raw, self.x, self.y)
    }
}

/// *8xy2 - AND Vx, Vy* :: Set Vx = Vx AND Vy.
///
/// Performs a bitwise AND on the values of Vx and Vy, then stores the result in
/// Vx. A bitwise AND compares the corrseponding bits from two values, and if both
/// bits are 1, then the same bit in the result is also 1. Otherwise, it is 0.
#[derive(Default)]
struct And {
    raw: u16,
    x: usize,
    y: usize,
}

impl Instr for And {
    fn parse(&mut self, instr: u16) {
        self.raw = instr;
        self.x = ((instr & 0x0f00) >> 8) as usize;
        self.y = ((instr & 0x00f0) >> 4) as usize;
    }

    fn execute(&self, cpu: &mut Cpu) {
        let new_value = cpu.get_vx(self.x) & cpu.get_vx(self.y);
        cpu.set_vx(self.x, new_value);
    }
}

impl fmt::Display for And {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:04x} - AND V{:x}, V{:x}", self.raw, self.x, self.y)
    }
}

/// *8xy3 - XOR Vx, Vy* :: Set Vx = Vx XOR Vy.
///
/// Performs a bitwise exclusive OR on the values of Vx and Vy, then stores the
/// result in Vx. An exclusive OR compares the corrseponding bits from two values,
/// and if the bits are not both the same, then the corresponding bit in the result
/// is set to 1. Otherwise, it is 0.
#[derive(Default)]
struct Xor {
    raw: u16,
    x: usize,
    y: usize,
}

impl Instr for Xor {
    fn parse(&mut self, instr: u16) {
        self.raw = instr;
        self.x = ((instr & 0x0f00) >> 8) as usize;
        self.y = ((instr & 0x00f0) >> 4) as usize;
    }

    fn execute(&self, cpu: &mut Cpu) {
        let new_value = cpu.get_vx(self.x) ^ cpu.get_vx(self.y);
        cpu.set_vx(self.x, new_value);
    }
}

impl fmt::Display for Xor {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:04x} - XOR V{:x}, V{:x}", self.raw, self.x, self.y)
    }
}

/// *8xy5 - SUB Vx, Vy* :: Set Vx = Vx - Vy, set VF = NOT borrow.
///
/// If Vx > Vy, then VF is set to 1, otherwise 0. Then Vy is subtracted from Vx,
/// and the results stored in Vx.
#[derive(Default)]
struct Sub {
    raw: u16,
    x: usize,
    y: usize,
}

impl Instr for Sub {
    fn parse(&mut self, instr: u16) {
        self.raw = instr;
        self.x = ((instr & 0x0f00) >> 8) as usize;
        self.y = ((instr & 0x00f0) >> 4) as usize;
    }

    fn execute(&self, cpu: &mut Cpu) {
        let vx = cpu.get_vx(self.x);
        let vy = cpu.get_vx(self.y);

        if vx > vy {
            cpu.set_vx(0xf, 1);
        } else {
            cpu.set_vx(0xf, 0);
        }

        let new_value = cpu.get_vx(self.x).wrapping_sub(cpu.get_vx(self.y));
        cpu.set_vx(self.x, new_value);
    }
}

impl fmt::Display for Sub {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:04x} - SUB V{:x}, V{:x}", self.raw, self.x, self.y)
    }
}

/// *8xy6 - SHR Vx {, Vy}* :: Set Vx = Vx SHR 1.
///
/// If the least-significant bit of Vx is 1, then VF is set to 1, otherwise 0. Then
/// Vx is divided by 2.
#[derive(Default)]
struct Shr {
    raw: u16,
    x: usize,
}

impl Instr for Shr {
    fn parse(&mut self, instr: u16) {
        self.raw = instr;
        self.x = ((instr & 0x0f00) >> 8) as usize;
    }

    fn execute(&self, cpu: &mut Cpu) {
        let vx = cpu.get_vx(self.x);

        if vx & 0x01 == 0x01 {
            cpu.set_vx(0xf, 1);
        } else {
            cpu.set_vx(0xf, 0);
        }

        cpu.set_vx(self.x, vx / 2);
    }
}

impl fmt::Display for Shr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:04x} - SHR V{:x}", self.raw, self.x)
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
        let raw_bytes = cpu.read_mem(i as usize, n as usize);

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

/// *Ex9E - SKP Vx* :: Skip next instruction if key with the value of Vx is pressed.
///
/// Checks the keyboard, and if the key corresponding to the value of Vx is
/// currently in the down position, PC is increased by 2.
#[derive(Default)]
struct SkpVx {
    raw: u16,
    reg: usize,
}

impl Instr for SkpVx {
    fn parse(&mut self, instr: u16) {
        self.raw = instr;
        self.reg = ((instr & 0x0f00) >> 8) as usize;
    }

    fn execute(&self, cpu: &mut Cpu) {
        let value = cpu.get_vx(self.reg) as usize;
        if cpu.get_keyboard().pressed(value) {
            cpu.inc_pc();
        }
    }
}

impl fmt::Display for SkpVx {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:04x} - SKP V{:x}", self.raw, self.reg)
    }
}

/// *ExA1 - SKNP Vx* :: Skip next instruction if key with the value of Vx is not pressed.
///
/// Checks the keyboard, and if the key corresponding to the value of Vx is
/// currently in the up position, PC is increased by 2.
#[derive(Default)]
struct SknpVx {
    raw: u16,
    reg: usize,
}

impl Instr for SknpVx {
    fn parse(&mut self, instr: u16) {
        self.raw = instr;
        self.reg = ((instr & 0x0f00) >> 8) as usize;
    }

    fn execute(&self, cpu: &mut Cpu) {
        let value = cpu.get_vx(self.reg) as usize;
        if !cpu.get_keyboard().pressed(value) {
            cpu.inc_pc();
        }
    }
}

impl fmt::Display for SknpVx {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:04x} - SKNP V{:x}", self.raw, self.reg)
    }
}


/// *Fx07 - LD Vx, DT* :: Set Vx = delay timer value.
///
/// The value of DT is placed into Vx.
#[derive(Default)]
struct LdVxDt {
    raw: u16,
    reg: usize,
}

impl Instr for LdVxDt {
    fn parse(&mut self, instr: u16) {
        self.raw = instr;
        self.reg = ((instr & 0x0f00) >> 8) as usize;
    }

    fn execute(&self, cpu: &mut Cpu) {
        let value = cpu.get_dt();
        cpu.set_vx(self.reg, value);
    }
}

impl fmt::Display for LdVxDt {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:04x} - LD V{:x}, DT", self.raw, self.reg)
    }
}

/// *Fx0A - LD Vx, K* :: Wait for a key press, store the value of the key in Vx.
///
/// All execution stops until a key is pressed, then the value of that key is
/// stored in Vx.
#[derive(Default)]
struct LdVxK {
    raw: u16,
    reg: usize,
}

impl Instr for LdVxK {
    fn parse(&mut self, instr: u16) {
        self.raw = instr;
        self.reg = ((instr & 0x0f00) >> 8) as usize;
    }

    fn execute(&self, cpu: &mut Cpu) {
        cpu.wait_for_input(self.reg);
    }
}

impl fmt::Display for LdVxK {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:04x} - LD V{:x}, K", self.raw, self.reg)
    }
}

/// *Fx15 - LD DT, Vx* :: Set delay timer = Vx.
///
/// DT is set equal to the value of Vx.
#[derive(Default)]
struct LdDt {
    raw: u16,
    reg: usize,
}

impl Instr for LdDt {
    fn parse(&mut self, instr: u16) {
        self.raw = instr;
        self.reg = ((instr & 0x0f00) >> 8) as usize;
    }

    fn execute(&self, cpu: &mut Cpu) {
        let value = cpu.get_vx(self.reg);
        cpu.set_dt(value);
    }
}

impl fmt::Display for LdDt {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:04x} - LD DT, V{:x}", self.raw, self.reg)
    }
}

/// *Fx1E - ADD I, Vx* :: Set I = I + Vx.
///
/// The values of I and Vx are added, and the results are stored in I.
#[derive(Default)]
struct AddI {
    raw: u16,
    reg: usize,
}

impl Instr for AddI {
    fn parse(&mut self, instr: u16) {
        self.raw = instr;
        self.reg = ((instr & 0x0f00) >> 8) as usize;
    }

    fn execute(&self, cpu: &mut Cpu) {
        let result = cpu.get_vx(self.reg) as u16 + cpu.get_i();
        cpu.set_i(result);
    }
}

impl fmt::Display for AddI {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:04x} - Add I, V{:x}", self.raw, self.reg)
    }
}

/// *Fx29 - LD F, Vx* :: Set I = location of sprite for digit Vx.
///
/// The value of I is set to the location for the hexadecimal sprite corresponding
/// to the value of Vx. See section 2.4, Display, for more information on the
/// Chip-8 hexadecimal font.
#[derive(Default)]
struct LdSprite {
    raw: u16,
    x: usize,
}

impl Instr for LdSprite {
    fn parse(&mut self, instr: u16) {
        self.raw = instr;
        self.x = ((instr & 0x0f00) >> 8) as usize;
    }

    fn execute(&self, cpu: &mut Cpu) {
        let value = cpu.get_vx(self.x) as u16;
        cpu.set_i(value * 5);
    }
}

impl fmt::Display for LdSprite {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:04x} - Ld F, V{:x}", self.raw, self.x)
    }
}

/// *Fx33 - LD B, Vx* :: Store BCD representation of Vx in memory locations I, I+1, and I+2.
///
/// The interpreter takes the decimal value of Vx, and places the hundreds digit in
/// memory at location in I, the tens digit at location I+1, and the ones digit at
/// location I+2.
#[derive(Default)]
struct LdBCD {
    raw: u16,
    x: usize,
}

impl Instr for LdBCD {
    fn parse(&mut self, instr: u16) {
        self.raw = instr;
        self.x = ((instr & 0x0f00) >> 8) as usize;
    }

    fn execute(&self, cpu: &mut Cpu) {
        let mut value = cpu.get_vx(self.x);
        let mem_idx = cpu.get_i() as usize;
        cpu.set_mem(mem_idx + 2, value % 10);
        value /= 10;
        cpu.set_mem(mem_idx + 1, value % 10);
        value /= 10;
        cpu.set_mem(mem_idx, value % 10);
    }
}

impl fmt::Display for LdBCD {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:04x} - Ld B, V{:x}", self.raw, self.x)
    }
}

/// *Fx55 - LD [I], Vx* :: Store registers V0 through Vx in memory starting at location I.
///
/// The interpreter copies the values of registers V0 through Vx into memory,
/// starting at the address in I.
#[derive(Default)]
struct SaveRegs {
    raw: u16,
    max_reg: usize,
}

impl Instr for SaveRegs {
    fn parse(&mut self, instr: u16) {
        self.raw = instr;
        self.max_reg = ((instr & 0x0f00) >> 8) as usize;
    }

    fn execute(&self, cpu: &mut Cpu) {
        for i in 0..self.max_reg {
            let addr = cpu.get_i() as usize + i;
            let value = cpu.get_vx(i);
            cpu.put_mem(addr, value);
        }
    }
}

impl fmt::Display for SaveRegs {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:04x} - Ld [I], V{:x}", self.raw, self.max_reg)
    }
}

/// *Fx65 - LD Vx, [I]* :: Read registers V0 through Vx from memory starting at location I.
///
/// The interpreter reads values from memory starting at location I into registers
/// V0 through Vx.
#[derive(Default)]
struct RestoreRegs {
    raw: u16,
    max_reg: usize,
}

impl Instr for RestoreRegs {
    fn parse(&mut self, instr: u16) {
        self.raw = instr;
        self.max_reg = ((instr & 0x0f00) >> 8) as usize;
    }

    fn execute(&self, cpu: &mut Cpu) {
        for i in 0..self.max_reg {
            let addr = cpu.get_i() as usize + i;
            let value = cpu.read_mem(addr, 1)[0];
            cpu.set_vx(i, value)
        }
    }
}

impl fmt::Display for RestoreRegs {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:04x} - Ld V{:x}, [I]", self.raw, self.max_reg)
    }
}

/// Dummy instruction. Does nothing
#[derive(Default)]
struct Dummy {
    raw: u16,
}

impl Instr for Dummy {
    fn parse(&mut self, instr: u16) {
        self.raw = instr;
    }

    #[allow(unused_variables)]
    fn execute(&self, cpu: &mut Cpu) {
        // Do nothing
    }
}

impl fmt::Display for Dummy {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:04x} - Dummy", self.raw)
    }
}

///
///
///
///

pub fn parse(raw: u16) -> Box<Instr> {
    let mut instr: Box<Instr> = match raw & 0xf000 {
        0x0000 => {
            match raw {
                0x00e0 => Box::new(Cls::default()),
                0x00ee => Box::new(Ret::default()),
                _ => panic!("unsupported instruction: {:04x}", raw),
            }
        }
        0x1000 => Box::new(Jp::default()),
        0x2000 => Box::new(Call::default()),
        0x3000 => Box::new(SeB::default()),
        0x4000 => Box::new(Sne::default()),
        0x5000 => Box::new(SeV::default()),
        0x6000 => Box::new(Ld::default()),
        0x7000 => Box::new(Add::default()),
        0x8000 => {
            match raw & 0x000f {
                0x0000 => Box::new(LdReg::default()),
                0x0002 => Box::new(And::default()),
                0x0003 => Box::new(Xor::default()),
                0x0005 => Box::new(Sub::default()),
                0x0006 => Box::new(Shr::default()),
                _ => panic!("unsupported instruction: {:04x}", raw),
            }
        }
        0xa000 => Box::new(LdI::default()),
        0xc000 => Box::new(Rnd::default()),
        0xd000 => Box::new(Drw::default()),
        0xe000 => {
            match raw & 0x00ff {
                0x009e => Box::new(SkpVx::default()),
                0x00a1 => Box::new(SknpVx::default()),
                _ => panic!("unsupported instruction: {:04x}", raw),
            }
        }
        0xf000 => {
            match raw & 0x00ff {
                0x0007 => Box::new(LdVxDt::default()),
                0x000a => Box::new(LdVxK::default()),
                0x0015 => Box::new(LdDt::default()),
                0x001e => Box::new(AddI::default()),
                0x0029 => Box::new(LdSprite::default()),
                0x0033 => Box::new(LdBCD::default()),
                0x0055 => Box::new(SaveRegs::default()),
                0x0065 => Box::new(RestoreRegs::default()),
                _ => panic!("unsupported instruction: {:04x}", raw),
            }
        }
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

/// *8xy1 - OR Vx, Vy* :: Set Vx = Vx OR Vy.
///
/// Performs a bitwise OR on the values of Vx and Vy, then stores the result in Vx.
/// A bitwise OR compares the corrseponding bits from two values, and if either bit
/// is 1, then the same bit in the result is also 1. Otherwise, it is 0.
#[allow(dead_code, unused_variables)]
pub fn or_vx_vy(cpu: &mut Cpu, instr: u16) {
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

/// *Fx18 - LD ST, Vx* :: Set sound timer = Vx.
///
/// ST is set equal to the value of Vx.
#[allow(dead_code, unused_variables)]
pub fn ld_st_vx(cpu: &mut Cpu, instr: u16) {
    // TODO
}

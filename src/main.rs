use rand::Rng;
use rand::rngs::ThreadRng;

const STACK_SIZE: usize = 64;
const DISPLAY_WIDTH: usize = 64;
const DISPLAY_HEIGHT: usize = 32;
const CHAR_ON: char = '█';
const CHAR_OFF: char = ' ';

struct CPU {
    ram: [u8; 4096], // Main memory
    pc: u16, // Program counter
    ir: u16, // Index register
    stack: [u16; STACK_SIZE], // Reserved out side the main memory
    regs: [u8; 16], // General purpose registers
    vbuf: [u8; DISPLAY_WIDTH * DISPLAY_HEIGHT / 8], // Video buffer,
    rng: ThreadRng,
}

const font_set: [[u8; 5]; 16] = [
[0xF0, 0x90, 0x90, 0x90, 0xF0], // 0
[0x20, 0x60, 0x20, 0x20, 0x70], // 1
[0xF0, 0x10, 0xF0, 0x80, 0xF0], // 2
[0xF0, 0x10, 0xF0, 0x10, 0xF0], // 3
[0x90, 0x90, 0xF0, 0x10, 0x10], // 4
[0xF0, 0x80, 0xF0, 0x10, 0xF0], // 5
[0xF0, 0x80, 0xF0, 0x90, 0xF0], // 6
[0xF0, 0x10, 0x20, 0x40, 0x40], // 7
[0xF0, 0x90, 0xF0, 0x90, 0xF0], // 8
[0xF0, 0x90, 0xF0, 0x10, 0xF0], // 9
[0xF0, 0x90, 0xF0, 0x90, 0x90], // A
[0xE0, 0x90, 0xE0, 0x90, 0xE0], // B
[0xF0, 0x80, 0x80, 0x80, 0xF0], // C
[0xE0, 0x90, 0x90, 0x90, 0xE0], // D
[0xF0, 0x80, 0xF0, 0x80, 0xF0], // E
[0xF0, 0x80, 0xF0, 0x80, 0x80]  // F
];

impl CPU {
    fn new() -> CPU {
       CPU {
           ram: [0; 4096],
           pc: 0,
           ir: 0,
           stack: [0; STACK_SIZE],
           regs: [0; 16],
           vbuf: [0x00; DISPLAY_WIDTH * DISPLAY_HEIGHT / 8],
           rng : rand::thread_rng(),
       }
    }

    fn clear_display(&mut self) {
        self.vbuf =  [0; DISPLAY_WIDTH * DISPLAY_HEIGHT / 8]
    }

    // Return from sub routine
    fn retsub(&mut self) {
        return;
    }

    fn goto(&mut self, address: u16) {
        self.pc = address;
    }

    // Call subroutine at address
    fn call(&mut self, address: u16) {
        return;
    }

    // Conditionals skip next instruction if true, skip is done by incrementing PC by 2
    fn se(&mut self, reg:u8, value:u8) {
        if self.regs[reg as usize] == value {
            self.pc = self.pc + 2;
        }
        return;
    }

    fn sne(&mut self, reg:u8, value:u8) {
        if self.regs[reg as usize] != value {
            self.pc = self.pc + 2;
        }
        return;
    }

    fn sre(&mut self, reg1:u8, reg2:u8) {
        if self.regs[reg1 as usize] == self.regs[reg2 as usize] {
            self.pc = self.pc + 2;
        }
        return;
    }

    fn setreg(&mut self, reg: u8, value: u8) {
        self.regs[reg as usize] = value;
    }

    fn addc(&mut self, reg: u8, value: u8) {
        self.regs[reg as usize] = self.regs[reg as usize] + value
    }

    fn assignreg(&mut self, reg1:u8, reg2:u8) {
        self.regs[reg1 as usize] = self.regs[reg2 as usize]
    }

    fn bitor(&mut self, reg1:u8, reg2:u8) {
        self.regs[reg1 as usize] |= self.regs[reg2 as usize]
    }

    fn bitand(&mut self, reg1:u8, reg2:u8) {
        self.regs[reg1 as usize] &= self.regs[reg2 as usize]
    }

    fn bitxor(&mut self, reg1:u8, reg2:u8) {
        self.regs[reg1 as usize] ^= self.regs[reg2 as usize]
    }

    fn addreg(&mut self, reg1:u8, reg2:u8) {
        let sum: u16 = self.regs[reg1 as usize] as u16 + self.regs[reg2 as usize] as u16;
        self.regs[reg1 as usize] = (sum & 0x00ff) as u8;
        self.regs[0x0f] = ((sum & 0xff00) > 0) as u8;
    }

    fn subreg(&mut self, reg1:u8, reg2:u8) {
        let sub: u16 = self.regs[reg1 as usize] as u16 - self.regs[reg2 as usize] as u16;
        self.regs[reg1 as usize] = (sub & 0x00ff) as u8;
        self.regs[0x0f] = (!(sub & 0xff00) > 0) as u8;
    }

    // Subtract with reversed order
    fn subregrev(&mut self, reg1:u8, reg2:u8) {
        let sub: u16 = self.regs[reg2 as usize] as u16 - self.regs[reg1 as usize] as u16;
        self.regs[reg1 as usize] = (sub & 0x00ff) as u8;
        self.regs[0x0f] = (!(sub & 0xff00) > 0) as u8;
    }

    fn rshiftreg(&mut self, reg: u8) {
        // Store lsb to F
        self.regs[0x0f] = self.regs[reg as usize] & 0x01;
        self.regs[reg as usize] >>= 1;
    }

    fn lshiftreg(&mut self, reg: u8) {
        // Store msb to F
        self.regs[0x0f] = self.regs[reg as usize] & 0x80;
        self.regs[reg as usize] <<= 1;
    }

    fn snereg(&mut self, reg1:u8, reg2:u8) {
        if self.regs[reg1 as usize] != self.regs[reg2 as usize] {
            self.pc += 2;
        }
    }

    fn seti(&mut self, address: u16) {
        self.ir = address;
    }

    fn gotoreg(&mut self, address: u16) {
        self.pc = self.regs[0] as u16 + address;
    }

    fn rand(&mut self, reg: u8, address: u16) {
        let rng: u8 = self.rng.gen();
        self.regs[reg as usize] = rng & (address & 0x00ff) as u8
    }

    fn draw(&mut self, reg1: u8, reg2: u8, height: u8) {
        let x_px = self.regs[reg1 as usize]; // starting pixel x
        let y_px = self.regs[reg2 as usize]; // starting pixel y

        let vbuf_x:usize = (x_px / 8) as usize; // vbuf x element. vbuf y is same a pixel y
        let offset_x = x_px % 8;

        if vbuf_x >= DISPLAY_WIDTH {
            return;
        }

        // Get starting line of sprite from ram
        let sprite = self.ram[self.ir as usize];

        for line in 0..height {
            if line as usize >= DISPLAY_HEIGHT {
                return;
            }
            // First element
            let x = self.vbuf[vbuf_x + DISPLAY_WIDTH * (y_px + line) as usize];
            let bits1= x ^ sprite >> offset_x;

            // Check if any bits were turned off with AND
            if x & bits1 != x {
                self.regs[0x0F] = 1;
            }

            // Second element if screen not wrapping
            if vbuf_x < DISPLAY_WIDTH {
                let x = self.vbuf[vbuf_x + 1 + DISPLAY_WIDTH * (y_px + line) as usize];
                let bits2= x ^ sprite << (8-offset_x);

                // Check if any bits were turned off with AND
                if x & bits2 != x {
                    self.regs[0x0F] = 1;
                }
            }
        }
    }


    fn fetch(&mut self) -> u16 {
        let high: u16  = self.ram[self.pc as usize] as u16;
        let low: u16 = (self.ram[self.pc as usize + 1] as u16) << 8;
        self.pc = self.pc + 2;
        return high | low;
    }

    // TODO: Consider splitting u16 to 2 u8s before function call
    fn exec(&mut self, ins: u16) {
        // Decode command and execute matched
        match ins & 0xf000 {
            0x0000 => match ins & 0x00ff {
                0x00e0 => self.clear_display(),
                0x00ee => self.retsub(),
                _ => return,
            },
            0x1000 => self.goto(ins & 0x0fff),
            0x2000 => self.call(ins & 0x0fff),
            0x3000 => self.se((ins & 0x0f00 >> 8) as u8, (ins & 0x00ff) as u8),
            0x4000 => self.sne((ins & 0x0f00 >> 8) as u8, (ins & 0x00ff) as u8),
            0x5000 => self.sre((ins & 0x0f00 >> 8) as u8, (ins & 0x00f0 >> 4) as u8),
            0x6000 => self.setreg((ins & 0x0f00 >> 8) as u8, (ins & 0x00ff) as u8),
            0x7000 => self.addc((ins & 0x0f00 >> 8) as u8, (ins & 0x00ff) as u8),
            0x8000 => match ins & 0x000f {
                0x00 => self.assignreg((ins & 0x0f00 >> 8) as u8, (ins & 0x00f0 >> 4) as u8),
                0x01 => self.bitor((ins & 0x0f00 >> 8) as u8, (ins & 0x00f0 >> 4) as u8),
                0x02 => self.bitand((ins & 0x0f00 >> 8) as u8, (ins & 0x00f0 >> 4) as u8),
                0x03 => self.bitxor((ins & 0x0f00 >> 8) as u8, (ins & 0x00f0 >> 4) as u8),
                0x04 => self.addreg((ins & 0x0f00 >> 8) as u8, (ins & 0x00f0 >> 4) as u8),
                0x05 => self.subreg((ins & 0x0f00 >> 8) as u8, (ins & 0x00f0 >> 4) as u8),
                0x06 => self.rshiftreg((ins & 0x0f00 >> 8) as u8),
                0x07 => self.subreg((ins & 0x0f00 >> 8) as u8, (ins & 0x00f0 >> 4) as u8),
                0x0E => self.rshiftreg((ins & 0x0f00 >> 8) as u8),
                _ => return,
            }
            0x9000 => self.snereg((ins & 0x0f00 >> 8) as u8, (ins & 0x00f0 >> 4) as u8),
            0xA000 => self.seti(ins & 0x0fff),
            0xB000 => self.gotoreg(ins & 0x0fff),
            0xC000 => self.rand((ins & 0x0f00 >> 8) as u8, ins & 0x00ff),
            0xD000 => self.draw((ins & 0x0f00 >> 8) as u8, (ins & 0x00f0 >> 4) as u8, (ins & 0x000f) as u8),
            _ => return,
        }
    }

    fn run(&mut self) -> i32 {
        self.print_vbuf();
        return 0;
        loop {
            let instruction = self.fetch();
            println!("{}", format!("PC:{:04x} | {:08x}", self.pc, instruction));
            if self.pc >= 4094 {
                println!("Last instruction");
                return -1;
            }
        }
    }

    fn render_sprite_line(&self, sprite_line: &u8) -> String {
        let mut s: String = "".to_string();
        for px in [0x01, 0x02, 0x04, 0x08, 0x10, 0x20, 0x40, 0x80] {
            match sprite_line & px {
                0 => s.push(CHAR_OFF),
                px => s.push(CHAR_ON),
                _ => s.push('X'),
            }
        }
        return s;
    }

    pub fn print_vbuf(&mut self) {
        for (idx, sprite) in self.vbuf.iter().enumerate() {
            if idx * 8 % DISPLAY_WIDTH == 0 {
                print!("\n");
            }
            let s = self.render_sprite_line(sprite);
            print!("{}", s);
        }
    }

    pub fn load_bin(&mut self, binary: String) {
        return;
    }
}

fn main() {
    println!("Hello, world!");
    let mut cpu = CPU::new();
    cpu.run();
    return;
}

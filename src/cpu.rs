use rand::Rng;
use rand::rngs::ThreadRng;

//const STACK_SIZE: usize = 64;
const DISPLAY_WIDTH: usize = 64;
const DISPLAY_HEIGHT: usize = 32;
const PROGRAM_START: usize = 0x200;
const CHAR_ON: char = '█';
const CHAR_OFF: char = ' ';
const CLOCK_SPEED: u64 = 500; // Hz


pub struct CPU {
    ram: [u8; 4096], // Main memory
    pc: u16, // Program counter
    ir: u16, // Index register
    sp: u8, // Stack pointer
    dt: u8, // Delay timer
    st: u8, // Sound timer
    stack: Vec<u16>, // Stack
    regs: [u8; 16], // General purpose registers
    pub vbuf: [u8; DISPLAY_WIDTH * DISPLAY_HEIGHT / 8], // Video buffer,
    rng: ThreadRng,
}

const _FONT_SET: [[u8; 5]; 16] = [
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

impl Default for CPU {
    fn default() -> CPU {
        CPU {
            ram: [0; 4096],
            pc: PROGRAM_START as u16,
            ir: 0,
            sp: 0,
            dt: 0,
            st: 0,
            stack: Vec::new(),
            regs: [0; 16],
            vbuf: [0x00; DISPLAY_WIDTH * DISPLAY_HEIGHT / 8],
            rng : rand::thread_rng(),
        }
        // Preload sprites to 0x0000 - 0x01ff
    }
}

impl CPU {
    pub fn new() -> CPU {
        CPU {
            ram: [0; 4096],
            pc: PROGRAM_START as u16,
            ir: 0,
            sp: 0,
            dt: 0,
            st: 0,
            stack: Vec::new(),
            regs: [0; 16],
            vbuf: [0x00; DISPLAY_WIDTH * DISPLAY_HEIGHT / 8],
            rng : rand::thread_rng(),
        }
        // Preload sprites to 0x0000 - 0x01ff
    }

    fn clear_display(&mut self) {
        self.vbuf =  [0; DISPLAY_WIDTH * DISPLAY_HEIGHT / 8]
    }

    // Return from sub routine
    fn retsub(&mut self) {
        let addr = self.stack.pop();
        if addr.is_none() {
            return;  // If nothing to return from exit program
        }
        self.pc = addr.unwrap(); // Return to previous address
        self.sp -= 1; // Decrement sp
    }

    fn goto(&mut self, address: u16) {
        self.pc = address;
    }

    // Call subroutine at address
    fn call(&mut self, address: u16) {
        self.sp += 1; // Increment stack pointer
        self.stack.push(self.pc); // Save current pc to stack
        self.pc = address; // Jump to address
        return;
    }

    // Skip if register is equal to value
    fn se(&mut self, reg:u8, value:u8) {
        if self.regs[reg as usize] == value {
            self.pc = self.pc + 2;
        }
        return;
    }

    // Skip if register not equal to value
    fn sne(&mut self, reg:u8, value:u8) {
        if self.regs[reg as usize] != value {
            self.pc = self.pc + 2;
        }
        return;
    }


    // Skip if registers are equal
    fn sre(&mut self, reg1:u8, reg2:u8) {
        if self.regs[reg1 as usize] == self.regs[reg2 as usize] {
            self.pc = self.pc + 2;
        }
        return;
    }



    fn setreg(&mut self, reg: u8, value: u8) {
        println!("reg: {}, value: {}", reg, value);
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
        println!("Set I to {}", address);
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


        for line in 0..height {
            if line as usize >= DISPLAY_HEIGHT {
                return;
            }

            // Get starting line of sprite from ram
            let sprite = self.ram[self.ir as usize + line as usize];

            // First element
            let element = vbuf_x + DISPLAY_WIDTH / 8 * (y_px / 8 + line) as usize;

            println!("x_px: {}, y_px: {}, vbuf_x: {}, offset_x: {}, line: {}, element: {}, sprite: {:08b}", x_px, y_px, vbuf_x, offset_x, line, element, sprite);

            let x = self.vbuf[element];
            let bits1= x ^ sprite >> offset_x;
            self.vbuf[element] = bits1;

            // Check if any bits were turned off with AND
            if x & bits1 != x {
                self.regs[0x0F] = 1;
            }

            // Second element if screen not wrapping
            if vbuf_x < DISPLAY_WIDTH && offset_x > 0{
                let x = self.vbuf[element + 1];
                let bits2= x ^ sprite << (8-offset_x);
                self.vbuf[element + 1] = bits2;

                // Check if any bits were turned off with AND
                if x & bits2 != x {
                    self.regs[0x0F] = 1;
                }
            }
        }
        self.print_vbuf();
    }

    // Skip if key is pressed with value in register
    fn skp(&mut self, _reg: u8) {
        return
    }

    // Skip if key is not pressed with value in register
    fn sknp(&mut self, _reg: u8) {
        return
    }

    // Read value of dt to register
    fn getdt(&mut self, reg: u8) {
        self.regs[reg as usize] = self.dt;
    }


    // Set value of register to delay timer
    fn setdt(&mut self, reg: u8) {
        self.dt = self.regs[reg as usize];
    }

    // Wait for keypress and store to reg
    fn waitkp(&mut self, _reg: u8) {
        return;
    }

    // Set value of register to sound timer
    fn setst(&mut self, reg: u8) {
        self.st = self.regs[reg as usize];
    }

    // Add value of register to index register
    fn addi(&mut self, reg: u8) {
        self.ir += self.regs[reg as usize] as u16;
    }

    // Set location of sprite in private memory that matches value of register to index register
    fn setisprite(&mut self, _reg: u8) {
        return;
    }

    // Store BCD (Binary Coded Decimal) of value in register to three bytes starting from index register
    fn setbcd(&mut self, reg: u8) {
        let s: String = self.regs[reg as usize].to_string();
        self.ram[self.ir as usize] = s.chars().nth(0).unwrap_or('0') as u8;
        self.ram[self.ir as usize + 1] = s.chars().nth(1).unwrap_or('0') as u8;
        self.ram[self.ir as usize + 2] = s.chars().nth(2).unwrap_or('0') as u8;
    }

    // Store registers from 0 to register starting at address in index register
    fn regsstore(&mut self, reg: u8) {
        for r in 0..=reg {
            self.ram[self.ir as usize + r as usize] = self.regs[r as usize];
        }
    }

    // Load values starting from index register to registers from 0 to given register
    fn regsload(&mut self, reg: u8) {
        for r in 0..=reg {
            self.regs[r as usize] = self.ram[self.ir as usize + r as usize];
        }
    }


    fn fetch(&mut self) -> u16 {
        let high: u16  = (self.ram[self.pc as usize] as u16) << 8;
        let low: u16 = self.ram[self.pc as usize + 1] as u16;
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
            0x3000 => self.se((ins >> 8 & 0xf) as u8, (ins & 0x00ff) as u8),
            0x4000 => self.sne((ins >> 8 & 0xf) as u8, (ins & 0x00ff) as u8),
            0x5000 => self.sre((ins >> 8 & 0xf) as u8, (ins >> 4 & 0x00f0) as u8),
            0x6000 => {self.setreg((ins >> 8 & 0xf) as u8, (ins & 0x00ff) as u8)}, // TODO: this is the correct way to mask!!!!!
            0x7000 => self.addc((ins >> 8 & 0xf) as u8, (ins & 0x00ff) as u8),
            0x8000 => match ins & 0x000f {
                0x00 => self.assignreg((ins >> 8 & 0xf) as u8, (ins >> 4 & 0x00f0) as u8),
                0x01 => self.bitor((ins >> 8 & 0xf) as u8, (ins >> 4 & 0x00f0) as u8),
                0x02 => self.bitand((ins >> 8 & 0xf) as u8, (ins >> 4 & 0x00f0) as u8),
                0x03 => self.bitxor((ins >> 8 & 0xf) as u8, (ins >> 4 & 0x00f0) as u8),
                0x04 => self.addreg((ins >> 8 & 0xf) as u8, (ins >> 4 & 0x00f0) as u8),
                0x05 => self.subreg((ins >> 8 & 0xf) as u8, (ins >> 4 & 0x00f0) as u8),
                0x06 => self.rshiftreg((ins >> 8 & 0xf) as u8), // TODO: Handle ambiguity
                0x07 => self.subregrev((ins >> 8 & 0xf) as u8, (ins >> 4 & 0x00f0) as u8),
                0x0E => self.lshiftreg((ins >> 8 & 0xf) as u8),
                _ => return,
            }
            0x9000 => self.snereg((ins >> 8 & 0xf) as u8, (ins >> 4 & 0x00f0) as u8),
            0xA000 => self.seti(ins & 0x0fff),
            0xB000 => self.gotoreg(ins & 0x0fff),
            0xC000 => self.rand((ins >> 8 & 0xf) as u8, ins & 0x00ff),
            0xD000 => self.draw((ins >> 8 & 0xf) as u8, (ins >> 4 & 0x00f0) as u8, (ins & 0x000f) as u8),
            0xE000 => match ins & 0x00ff {
                0x9E => self.skp((ins & 0xf00 >> 8) as u8),
                0xA1 => self.sknp((ins & 0xf00 >> 8) as u8),
                _ => return,
            }
            0xF000 => match ins & 0x00ff {
                0x07 => self.getdt((ins >> 8 & 0x0f) as u8),
                0x0A => self.waitkp((ins >> 8 & 0x0f) as u8),
                0x15 => self.setdt((ins >> 8 & 0x0f) as u8),
                0x18 => self.setst((ins >> 8 & 0x0f) as u8),
                0x1E => self.addi((ins >> 8 & 0x0f) as u8),
                0x29 => self.setisprite((ins >> 8 & 0x0f) as u8),
                0x33 => self.setbcd((ins >> 8 & 0x0f) as u8),
                0x55 => self.regsstore((ins >> 8 & 0x0f) as u8),
                0x65 => self.regsload((ins >> 8 & 0x0f) as u8),
                _ => return,
            }

            _ => return,
        }
    }

    pub fn run(&mut self) -> i32 {
        loop {
            let instruction = self.fetch();
            // Print current pc and instruction
            println!("PC: {:04X} INS: {:04X}", self.pc - 2, instruction);
            self.exec(instruction);
            if self.pc >= 4095 {
                println!("Last instruction");
                return -1;
            }
        }
    }

    fn render_sprite_line(&self, sprite_line: &u8) -> String {
        let mut s: String = "".to_string();
        for px in [0x80, 0x40, 0x20, 0x10, 0x08, 0x04, 0x02, 0x01] {
            match sprite_line & px {
                0 => s.push(CHAR_OFF),
                _px => s.push(CHAR_ON),
            }
        }
        return s;
    }

    pub fn get_registers(&self) -> [u8; 16] {
        return self.regs;
    }

    pub fn print_registers(&mut self) {
        println!("PC: {:04X}", self.pc);
        println!("I: {:04X}", self.ir);
        println!("SP: {:04X}", self.sp);
        println!("DT: {:02X}", self.dt);
        println!("ST: {:02X}", self.st);
        println!("V0: {:02X} V1: {:02X} V2: {:02X} V3: {:02X}", self.regs[0], self.regs[1], self.regs[2], self.regs[3]);
        println!("V4: {:02X} V5: {:02X} V6: {:02X} V7: {:02X}", self.regs[4], self.regs[5], self.regs[6], self.regs[7]);
        println!("V8: {:02X} V9: {:02X} VA: {:02X} VB: {:02X}", self.regs[8], self.regs[9], self.regs[10], self.regs[11]);
        println!("VC: {:02X} VD: {:02X} VE: {:02X} VF: {:02X}", self.regs[12], self.regs[13], self.regs[14], self.regs[15]);
    }

    pub fn read_vbuf(&self, x: u8, y: u8) -> bool {
        let vbuf_x: usize = (x / 8) as usize;
        let vbuf_y: usize = (y / 8) as usize;
        let offset_x = x % 8;
        let element = vbuf_x + DISPLAY_WIDTH / 8 * vbuf_y;
        let b = self.vbuf[element];
        return b & (0x80 >> offset_x) != 0;
        // let vbuf_x: usize = (x / 8) as usize;
        // let element = vbuf_x + DISPLAY_WIDTH / 8 * (y) as usize;
        // let bits = self.vbuf[element];
        // let bit = x % 8;
        // return bits & (0x80 >> bit) != 0;
    }

    pub fn print_vbuf(&mut self) {
        for (idx, sprite) in self.vbuf.iter().enumerate() {
            if idx * 8 % DISPLAY_WIDTH == 0 {
                print!("\n");
            }
            let s = self.render_sprite_line(sprite);
            print!("{}", s);
        }
        println!("")
    }

    pub fn print_memory(&mut self) {
        for (idx, byte) in self.ram.iter().enumerate() {
            if idx % 16 == 0 {
                print!("\n{:04x} | ", idx);
            }
            print!("{:02x} ", byte);
        }
        print!("\n");
    }

    pub fn load_bin(&mut self, binary: Vec<u8>, override_ram: bool) {
        // Override is used to prevent writing over the preloaded ram from 0x00 to 0x1ff
        if override_ram {
            for (idx, byte) in binary.iter().enumerate() {
                self.ram[idx] = *byte;
            }
        } else {
            for (idx, byte) in binary[PROGRAM_START..binary.len()].iter().enumerate() {
                self.ram[idx + PROGRAM_START] = *byte;
            }
        }
        return;
    }
}

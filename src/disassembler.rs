pub fn decode(ins: u16) -> String {

    match ins & 0xf000 {
        0x0000 => match ins & 0x00ff {
            0x00e0 => "CLS".to_string(),
            0x00ee => "RET".to_string(),
            _ => panic!("Unknown instruction {:X}", ins)
        },
        0x1000 => format!("JP {:X}", ins & 0x0fff),
        0x2000 => format!("SYS {:X}", ins & 0x0fff),
        0x3000 => format!("SE V{:X}, {:X}", (ins >> 8 & 0xf) as u8, (ins & 0x00ff) as u8),
        0x4000 => format!("SNE V{:X}, {:X}", (ins >> 8 & 0xf) as u8, (ins & 0x00ff) as u8),
        0x5000 => format!("SE V{:X}, V{:X}", (ins >> 8 & 0xf) as u8, (ins >> 4 & 0x00f0) as u8),
        0x6000 => format!("LD V{:X}, {:X}", (ins >> 8 & 0xf) as u8, (ins & 0x00ff) as u8),
        0x7000 => format!("ADD V{:X}, {:X}", (ins >> 8 & 0xf) as u8, (ins & 0x00ff) as u8),
        0x8000 => match ins & 0x000f {
            0x00 => format!("LD V{:X}, V{:X}", (ins >> 8 & 0xf) as u8, (ins >> 4 & 0x00f0) as u8),
            0x01 => format!("OR V{:X}, V{:X}", (ins >> 8 & 0xf) as u8, (ins >> 4 & 0x00f0) as u8),
            0x02 => format!("AND V{:X}, V{:X}", (ins >> 8 & 0xf) as u8, (ins >> 4 & 0x00f0) as u8),
            0x03 => format!("XOR V{:X}, V{:X}", (ins >> 8 & 0xf) as u8, (ins >> 4 & 0x00f0) as u8),
            0x04 => format!("ADD V{:X}, V{:X}", (ins >> 8 & 0xf) as u8, (ins >> 4 & 0x00f0) as u8),
            0x05 => format!("SUB V{:X}, V{:X}", (ins >> 8 & 0xf) as u8, (ins >> 4 & 0x00f0) as u8),
            0x06 => format!("SHR V{:X}, V{:X}", (ins >> 8 & 0xf) as u8, (ins >> 4 & 0x00f0) as u8),
            0x07 => format!("SUBN V{:X}, V{:X}", (ins >> 8 & 0xf) as u8, (ins >> 4 & 0x00f0) as u8),
            0x0e => format!("SHL V{:X}, V{:X}", (ins >> 8 & 0xf) as u8, (ins >> 4 & 0x00f0) as u8),
            _ => panic!("Unknown instruction {:X}", ins)
        }
        0x9000 => format!("SNE V{:X}, V{:X}", (ins >> 8 & 0xf) as u8, (ins >> 4 & 0x00f0) as u8),
        0xA000 => format!("LD I, {:X}", ins & 0x0fff),
        0xB000 => format!("JP V0, {:X}", ins & 0x0fff),
        0xC000 => format!("RND V{:X}, {:X}", (ins >> 8 & 0xf) as u8, ins & 0x00ff),
        0xD000 => format!("DRW V{:X}, V{:X}, {:X}", (ins >> 8 & 0xf) as u8, (ins >> 4 & 0xf) as u8, (ins & 0x000f) as u8),
        0xE000 => match ins & 0x00ff {
            0x9E => format!("SKP V{:X}", (ins >> 8 & 0xf) as u8),
            0xA1 => format!("SKNP V{:X}", (ins >> 8 & 0xf) as u8),
            _ => panic!("Unknown instruction {:X}", ins)
        }
        0xF000 => match ins & 0x00ff {
            0x07 => format!("LD V{:X}, DT", (ins >> 8 & 0xf) as u8),
            0x0A => format!("LD V{:X}, K", (ins >> 8 & 0xf) as u8),
            0x15 => format!("LD DT, V{:X}", (ins >> 8 & 0xf) as u8),
            0x18 => format!("LD ST, V{:X}", (ins >> 8 & 0xf) as u8),
            0x1E => format!("ADD I, V{:X}", (ins >> 8 & 0xf) as u8),
            0x29 => format!("LD F, V{:X}", (ins >> 8 & 0xf) as u8),
            0x33 => format!("LD B, V{:X}", (ins >> 8 & 0xf) as u8),
            0x55 => format!("LD [I], V{:X}", (ins >> 8 & 0xf) as u8),
            0x65 => format!("LD V{:X}, [I]", (ins >> 8 & 0xf) as u8),
            _ => panic!("Unknown instruction {:X}", ins)
        }
        _ => panic!("Unknown instruction {:X}", ins)
    }
}

#[derive(Debug)]
pub struct Chip8Instr {
    pub bits: u16,
}

impl Chip8Instr {
    pub fn first(&self) -> u16 {
        (self.bits & 0xF000) >> 12
    }

    pub fn second(&self) -> u16 {
        (self.bits & 0x0F00) >> 8
    }

    pub fn third(&self) -> u16 {
        (self.bits & 0x00F0) >> 4
    }

    pub fn fourth(&self) -> u16 {
        self.bits & 0x000F
    }

    pub fn nn(&self) -> u8 {
        // nn = last 2 hex chars
        (self.bits & 0x00FF) as u8
    }

    pub fn nnn(&self) -> u16 {
        // nnn = last 3 hex chars
        self.bits & 0x0FFF
    }

    pub fn reg_x(&self) -> usize {
        // reg_x = 2nd hex char
        self.second() as usize
    }

    pub fn reg_y(&self) -> usize {
        // reg_y = 3rd hex char
        self.third() as usize
    }
}

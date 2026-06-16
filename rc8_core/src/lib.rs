// Real screen width and height in pixels
pub const SCREEN_WIDTH: usize = 64;
pub const SCREEN_HEIGHT: usize = 32;

// Size is in bytes
const RAM_SIZE: usize = 4096;
const STACK_SIZE: usize = 16;
const NUM_REGS: usize = 16;
const NUM_KEYS: usize = 16;
const PC_START_ADDR: u16 = 0x200;

pub struct Chip8 {
    // Each pixel can either be on/true (white) or off/false (black)
    pub screen: [bool; SCREEN_WIDTH * SCREEN_HEIGHT],

    // Registers
    pc: u16,
    v_reg: [u8; NUM_REGS], // 16 V registers
    i_reg: u16,            // index register

    // Stack holds return addresses, sp is an index into the stack
    sp: u8, // Holds the index of the top of the stack (last unused position)
    stack: [u16; STACK_SIZE],

    ram: [u8; RAM_SIZE],

    // Timers
    dt: u8,
    st: u8,

    // Input keys
    keys: [bool; NUM_KEYS],
}

impl Chip8 {
    // Constructor
    pub fn new() -> Self {
        Self {
            pc: PC_START_ADDR,
            v_reg: [0; NUM_REGS],
            i_reg: 0,
            sp: 0,
            stack: [0; STACK_SIZE],
            ram: [0; RAM_SIZE],
            screen: [false; SCREEN_WIDTH * SCREEN_HEIGHT],
            dt: 0,
            st: 0,
            keys: [false; NUM_KEYS],
        }
    }

    // Stack Methods
    fn stack_push(&mut self, addr: u16) {
        self.stack[self.sp as usize] = addr;
        self.sp += 1;
    }

    fn stack_pop(&mut self) -> u16 {
        self.sp -= 1;
        self.stack[self.sp as usize]
    }

    // Instructions

    // CLS
    fn OP_00E0(&mut self) {
        self.screen.fill(false);
    }

    // RET
    fn OP_00EE(&mut self) {
        self.pc = self.stack_pop();
    }

    // JUMP
    fn OP_1NNN(&mut self, instr: u16) {
        self.pc = instr & 0x0FFF;
    }

    // CALL
    fn OP_2NNN(&mut self, instr: u16) {
        // Save current PC to the stack before going there
        self.stack_push(self.pc);
        self.OP_1NNN(instr);
    }

    // Instructions for making decisions

    // SEQI
    fn OP_3XNN(&mut self, instr: u16) {
        // Skip following instruction if VX == NN
        let reg_x = ((instr & 0x0F00) >> 16) as usize;
        let nn = (instr & 0x00FF) as u8;

        if self.v_reg[reg_x] == nn {
            self.pc += 2;
        }
    }

    // SNEI
    fn OP_4XNN(&mut self, instr: u16) {
        // Skip following instruction if VX != NN
        let reg_x = ((instr & 0x0F00) >> 16) as usize;
        let nn = (instr & 0x00FF) as u8;

        if self.v_reg[reg_x] != nn {
            self.pc += 2;
        }
    }

    // SEQ
    fn OP_5XY0(&mut self, instr: u16) {
        // Skip following instruction if VX == VY
        let reg_x = ((instr & 0x0F00) >> 16) as usize;
        let reg_y = ((instr & 0x00F0) >> 8) as usize;

        if self.v_reg[reg_x] == self.v_reg[reg_y] {
            self.pc += 2;
        }
    }

    // Arithmetic Instructions

    // LI
    fn OP_6XNN(&mut self, instr: u16) {
        // Load NN into VX
        let reg_x = ((instr & 0x0F00) >> 16) as usize;
        let nn = (instr & 0x00FF) as u8;

        self.v_reg[reg_x] = nn;
    }

    // ADDI
    fn OP_7XNN(&mut self, instr: u16) {
        // Add NN to VX
        let reg_x = ((instr & 0x0F00) >> 16) as usize;
        let nn = (instr & 0x00FF) as u8;

        // Wrapping add to avoid panic on overflow
        self.v_reg[reg_x] = self.v_reg[reg_x].wrapping_add(nn);
    }

    // MV
    fn OP_8XY0(&mut self, instr: u16) {
        // Copy register VY into VX
        let reg_x = ((instr & 0x0F00) >> 16) as usize;
        let reg_y = ((instr & 0x00F0) >> 8) as usize;

        self.v_reg[reg_x] = self.v_reg[reg_y];
    }

    // SEOR
    fn OP_8XY1(&mut self, instr: u16) {
        // Set VX to VX | VY
        let reg_x = ((instr & 0x0F00) >> 16) as usize;
        let reg_y = ((instr & 0x00F0) >> 8) as usize;

        self.v_reg[reg_x] |= self.v_reg[reg_y];
    }

    // SEAND
    fn OP_8XY2(&mut self, instr: u16) {
        // Set VX to VX & VY
        let reg_x = ((instr & 0x0F00) >> 16) as usize;
        let reg_y = ((instr & 0x00F0) >> 8) as usize;

        self.v_reg[reg_x] &= self.v_reg[reg_y];
    }

    // SEXOR
    fn OP_8XY3(&mut self, instr: u16) {
        // Set VX to VX ^ VY
        let reg_x = ((instr & 0x0F00) >> 16) as usize;
        let reg_y = ((instr & 0x00F0) >> 8) as usize;

        self.v_reg[reg_x] ^= self.v_reg[reg_y];
    }

    // ADD
    fn OP_8XY4(&mut self, instr: u16) {
        // VX = VX + VY
        // VF set to 1 on overflow
        let reg_x = ((instr & 0x0F00) >> 16) as usize;
        let reg_y = ((instr & 0x00F0) >> 8) as usize;

        let (result, overflow) = self.v_reg[reg_x].overflowing_add(self.v_reg[reg_y]);

        self.v_reg[reg_x] = result;
        self.v_reg[0xF] = if overflow { 1 } else { 0 };
    }

    // SUB
    fn OP_8XY5(&mut self, instr: u16) {
        // VX = VX - VY
        // VF set to 1 on overflow
        let reg_x = ((instr & 0x0F00) >> 16) as usize;
        let reg_y = ((instr & 0x00F0) >> 8) as usize;

        let (result, overflow) = self.v_reg[reg_x].overflowing_sub(self.v_reg[reg_y]);

        self.v_reg[reg_x] = result;
        self.v_reg[0xF] = if overflow { 1 } else { 0 };
    }

    // Instruction Handling
    fn fetch(&mut self) -> u16 {
        // Chip-8 is a big-endian machine
        // Retrieve the byte at pc and pc+1 into the u16
        let instr =
            ((self.ram[self.pc as usize] as u16) << 8) | (self.ram[self.pc as usize + 1] as u16);
        self.pc += 2;
        instr
    }

    fn execute(&mut self, instr: u16) {
        // Separate each hex digit
        let first = (instr & 0xF000) >> 12;
        let second = (instr & 0x0F00) >> 8;
        let third = (instr & 0x00F0) >> 4;
        let fourth = instr & 0x000F;

        match (first, second, third, fourth) {
            (0, 0, 0, 0) => return,
            (0, 0, 0xE, 0) => self.OP_00E0(),
            (0, 0, 0xE, 0xE) => self.OP_00EE(),
            (0x1, _, _, _) => self.OP_1NNN(instr),
            (0x2, _, _, _) => self.OP_2NNN(instr),
            (0x3, _, _, _) => self.OP_3XNN(instr),
            (0x4, _, _, _) => self.OP_4XNN(instr),
            (0x5, _, _, 0) => self.OP_5XY0(instr),
            (0x6, _, _, _) => self.OP_6XNN(instr),
            (0x7, _, _, _) => self.OP_7XNN(instr),
            (0x8, _, _, 0) => self.OP_8XY0(instr),
            (0x8, _, _, 0x1) => self.OP_8XY1(instr),
            (0x8, _, _, 0x2) => self.OP_8XY2(instr),
            (0x8, _, _, 0x3) => self.OP_8XY3(instr),
            (0x8, _, _, 0x4) => self.OP_8XY4(instr),
            (0x8, _, _, 0x5) => self.OP_8XY5(instr),
            _ => unimplemented!("Unimplemented opcode: {instr}"),
        }
    }

    // Should be run at 60Hz?
    pub fn tick(&mut self) {
        // Fetch
        let instr = self.fetch();

        // Decode and Execute
        self.execute(instr);
    }

    // Load ROM
    pub fn load(&mut self, data: &[u8]) {
        let start = PC_START_ADDR as usize;
        let end = start + data.len() as usize;
        self.ram[start..end].copy_from_slice(data);
    }
}

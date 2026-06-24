use std::u8;

// Real screen width and height in pixels
pub const SCREEN_WIDTH: usize = 64;
pub const SCREEN_HEIGHT: usize = 32;

// Size is in bytes
const RAM_SIZE: usize = 4096;
const STACK_SIZE: usize = 16;
const NUM_REGS: usize = 16;
const NUM_KEYS: usize = 16;
const PC_START_ADDR: u16 = 0x200;

#[derive(Debug)]
struct Chip8Instr {
    bits: u16,
}

impl Chip8Instr {
    fn first(&self) -> u16 {
        (self.bits & 0xF000) >> 12
    }

    fn second(&self) -> u16 {
        (self.bits & 0x0F00) >> 8
    }

    fn third(&self) -> u16 {
        (self.bits & 0x00F0) >> 4
    }

    fn fourth(&self) -> u16 {
        self.bits & 0x000F
    }

    fn nn(&self) -> u8 {
        // nn = last 2 hex chars
        (self.bits & 0x00FF) as u8
    }

    fn nnn(&self) -> u16 {
        // nnn = last 3 hex chars
        self.bits & 0x0FFF
    }

    fn reg_x(&self) -> usize {
        // reg_x = 2nd hex char
        self.second() as usize
    }

    fn reg_y(&self) -> usize {
        // reg_y = 3rd hex char
        self.third() as usize
    }
}

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
    fn op_00e0(&mut self, _instr: Chip8Instr) {
        self.screen.fill(false);
    }

    // RET
    fn op_00ee(&mut self, _instr: Chip8Instr) {
        self.pc = self.stack_pop();
    }

    // JMP
    fn op_1nnn(&mut self, instr: Chip8Instr) {
        self.pc = instr.nnn() & 0x0FFF;
    }

    // JAL
    fn op_2nnn(&mut self, instr: Chip8Instr) {
        // Save current PC to the stack before going there
        self.stack_push(self.pc);
        self.op_1nnn(instr);
    }

    // SEQI
    fn op_3xnn(&mut self, instr: Chip8Instr) {
        // Skip following instruction if VX == NN
        if self.v_reg[instr.reg_x()] == instr.nn() {
            self.pc += 2;
        }
    }

    // SNEI
    fn op_4xnn(&mut self, instr: Chip8Instr) {
        // Skip following instruction if VX != NN
        if self.v_reg[instr.reg_x()] != instr.nn() {
            self.pc += 2;
        }
    }

    // SEQ
    fn op_5xy0(&mut self, instr: Chip8Instr) {
        // Skip following instruction if VX == VY
        if self.v_reg[instr.reg_x()] == self.v_reg[instr.reg_y()] {
            self.pc += 2;
        }
    }

    // LI
    fn op_6xnn(&mut self, instr: Chip8Instr) {
        // Load NN into VX
        self.v_reg[instr.reg_x()] = instr.nn();
    }

    // ADDI
    fn op_7xnn(&mut self, instr: Chip8Instr) {
        // Add NN to VX
        // Wrapping add to avoid panic on overflow
        self.v_reg[instr.reg_x()] = self.v_reg[instr.reg_x()].wrapping_add(instr.nn());
    }

    // MV
    fn op_8xy0(&mut self, instr: Chip8Instr) {
        // Copy register VY into VX
        self.v_reg[instr.reg_x()] = self.v_reg[instr.reg_y()];
    }

    // SEOR
    fn op_8xy1(&mut self, instr: Chip8Instr) {
        // Set VX to VX | VY
        self.v_reg[instr.reg_x()] |= self.v_reg[instr.reg_y()];
    }

    // SEAND
    fn op_8xy2(&mut self, instr: Chip8Instr) {
        // Set VX to VX & VY
        self.v_reg[instr.reg_x()] &= self.v_reg[instr.reg_y()];
    }

    // SEXOR
    fn op_8xy3(&mut self, instr: Chip8Instr) {
        // Set VX to VX ^ VY
        self.v_reg[instr.reg_x()] ^= self.v_reg[instr.reg_y()];
    }

    // ADD
    fn op_8xy4(&mut self, instr: Chip8Instr) {
        // VX = VX + VY
        // VF set to 1 on overflow
        let (result, overflow) = self.v_reg[instr.reg_x()].overflowing_add(self.v_reg[instr.reg_y()]);

        self.v_reg[instr.reg_x()] = result;
        self.v_reg[0xF] = if overflow { 1 } else { 0 };
    }

    // SUB
    fn op_8xy5(&mut self, instr: Chip8Instr) {
        // VX = VX - VY
        // VF set to 1 on overflow
        let (result, overflow) = self.v_reg[instr.reg_x()].overflowing_sub(self.v_reg[instr.reg_y()]);

        self.v_reg[instr.reg_x()] = result;
        self.v_reg[0xF] = if overflow { 1 } else { 0 };
    }

    // SRL
    // NOTE: Differing implementations based on reference.
    fn op_8xy6(&mut self, instr: Chip8Instr) {
        // VX = VY >> 1
        // VF = LSB of VY
        self.v_reg[instr.reg_x()] = self.v_reg[instr.reg_y()] >> 1;
        self.v_reg[0xF] = self.v_reg[instr.reg_y()] & 1;
    }

    // SUB2
    fn op_8xy7(&mut self, instr: Chip8Instr) {
        // VX = VY - VX
        // VF set to 1 on overflow
        let (result, overflow) = self.v_reg[instr.reg_y()].overflowing_sub(self.v_reg[instr.reg_x()]);

        self.v_reg[instr.reg_x()] = result;
        self.v_reg[0xF] = if overflow { 1 } else { 0 };
    }

    // SLL
    // NOTE: Differing implementations based on reference.
    fn op_8xye(&mut self, instr: Chip8Instr) {
        // VX = VY << 1
        // VF = LSB of VY
        self.v_reg[instr.reg_x()] = self.v_reg[instr.reg_y()] << 1;
        self.v_reg[0xF] = self.v_reg[instr.reg_y()] & 0x8;
    }

    // SNE
    fn op_9xy0(&mut self, instr: Chip8Instr) {
        // Skip following instruction if VX != VY
        if self.v_reg[instr.reg_x()] != self.v_reg[instr.reg_y()] {
            self.pc += 2;
        }
    }

    // SMI
    fn op_annn(&mut self, instr: Chip8Instr) {
        // Store NNN into i_reg
        self.i_reg = instr.nnn();
    }

    // LJMP
    fn op_bnnn(&mut self, instr: Chip8Instr) {
        // PC = NNN + V0
        self.pc = instr.nnn() + self.v_reg[0] as u16;
    }

    // SRND
    fn op_cxnn(&mut self, instr: Chip8Instr) {
        // reg_x = random number & 0xNN
        let random: u8 = rand::random();
        self.v_reg[instr.reg_x()] = random & instr.nn();
    }

    // DSPR
    fn op_dxyn(&mut self, instr: Chip8Instr) {
        // TODO
    }

    // SKP
    fn op_ex9e(&mut self, instr: Chip8Instr) {
        // Skip next instr if the key in VX is pressed
        let key_code = self.v_reg[instr.reg_x()] as usize;
        if self.keys.get(key_code).is_some_and(|x| *x) {
            self.pc += 2;
        }
    }

    // SKNP
    fn op_exa1(&mut self, instr: Chip8Instr) {
        // Skip next instr if the key in VX is not pressed
        let key_code = self.v_reg[instr.reg_x()] as usize;
        if self.keys.get(key_code).is_some_and(|x| !*x) {
            self.pc += 2;
        }
    }

    // SDT
    fn op_fx07(&mut self, instr: Chip8Instr) {
        // Store current delay timer value in VX
        self.v_reg[instr.reg_x()] = self.dt;
    }

    // WKP
    fn op_fx0a(&mut self, instr: Chip8Instr) {
        // Wait for a keypress and store into VX
        let pressed_key = self.keys.iter().position(|x| *x);

        if pressed_key.is_some() {
            self.v_reg[instr.reg_x()] = pressed_key.unwrap() as u8;
        } else {
            // Keep blocking until a key is pressed
            self.pc -= 2;
        }
    }

    // Instruction Handling
    fn fetch(&mut self) -> Chip8Instr {
        // Chip-8 is a big-endian machine
        // Retrieve the byte at pc and pc+1 into the u16
        let instr =
            ((self.ram[self.pc as usize] as u16) << 8) | (self.ram[self.pc as usize + 1] as u16);
        self.pc += 2;
        Chip8Instr { bits: instr }
    }

    fn execute(&mut self, instr: Chip8Instr) {
        match (instr.first(), instr.second(), instr.third(), instr.fourth()) {
            (0x0, 0x0, 0x0, 0x0) => return,
            (0x0, 0x0, 0xE, 0x0) => self.op_00e0(instr),
            (0x0, 0x0, 0xE, 0xE) => self.op_00ee(instr),
            (0x1, _, _, _) => self.op_1nnn(instr),
            (0x2, _, _, _) => self.op_2nnn(instr),
            (0x3, _, _, _) => self.op_3xnn(instr),
            (0x4, _, _, _) => self.op_4xnn(instr),
            (0x5, _, _, 0x0) => self.op_5xy0(instr),
            (0x6, _, _, _) => self.op_6xnn(instr),
            (0x7, _, _, _) => self.op_7xnn(instr),
            (0x8, _, _, 0x0) => self.op_8xy0(instr),
            (0x8, _, _, 0x1) => self.op_8xy1(instr),
            (0x8, _, _, 0x2) => self.op_8xy2(instr),
            (0x8, _, _, 0x3) => self.op_8xy3(instr),
            (0x8, _, _, 0x4) => self.op_8xy4(instr),
            (0x8, _, _, 0x5) => self.op_8xy5(instr),
            (0x8, _, _, 0x6) => self.op_8xy6(instr),
            (0x8, _, _, 0x7) => self.op_8xy7(instr),
            (0x8, _, _, 0xE) => self.op_8xye(instr),
            (0x9, _, _, 0) => self.op_9xy0(instr),
            (0xA, _, _, _) => self.op_annn(instr),
            (0xB, _, _, _) => self.op_bnnn(instr),
            (0xC, _, _, _) => self.op_cxnn(instr),
            (0xD, _, _, _) => self.op_dxyn(instr),
            (0xE, _, 0x9, 0xE) => self.op_ex9e(instr),
            (0xE, _, 0xA, 0x1) => self.op_exa1(instr),
            (0xF, _, 0x0, 0x7) => self.op_fx07(instr),
            (0xF, _, 0x0, 0xA) => self.op_fx0a(instr),
            _ => unimplemented!("Unimplemented opcode: {:?}", instr),
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

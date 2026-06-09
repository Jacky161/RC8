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
    v_reg: [u8; NUM_REGS],  // 16 V registers
    i_reg: u16,  // index register

    // Stack holds return addresses, sp is an index into the stack
    sp: u8,  // Holds the index of the top of the stack (last unused position)
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

    // Instruction Handling
    fn fetch(&mut self) -> u16 {
        // Chip-8 is a big-endian machine
        // Retrieve the byte at pc and pc+1 into the u16
        let instr = ((self.ram[self.pc as usize] as u16) << 8) | (self.ram[self.pc as usize + 1] as u16);
        self.pc += 2;
        instr
    }

    // Should be run at 60Hz?
    pub fn tick(&mut self) {
        // Fetch
        let instr = self.fetch();

        // Decode

        // Execute
    }

    // Load ROM
    pub fn load(&mut self, data: &[u8]) {
        let start = PC_START_ADDR as usize;
        let end = start + data.len() as usize;
        self.ram[start..end].copy_from_slice(data);
    }
}

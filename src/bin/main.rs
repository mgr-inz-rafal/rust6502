#![feature(llvm_asm, const_if_match)]

use volatile_register::RW;

const WSYNC: u16 = 0xD40A;
const COLBK: u16 = 0xD01A;
const SCREEN: u16 = 0xBC40;
const STRIG0: u16 = 0x284;

#[repr(C)]
pub struct ByteWrapper {
    pub v: RW<u8>,
}

pub struct Byte {
    b: *mut ByteWrapper,
}

#[repr(C)]
pub struct ScreenWrapper {
    pub screen: [RW<u8>; 40 * 20],
}

pub struct Screen {
    s: *mut ScreenWrapper,
}

impl Screen {
    fn new(addr: u16) -> Self {
        Screen {
            s: addr as *mut ScreenWrapper,
        }
    }

    const fn to_atari_char(c: char) -> u8 {
        match c {
            'H' => 40,
            'e' => 101,
            'l' => 108,
            'o' => 111,
            ' ' => 0,
            'R' => 50,
            'u' => 117,
            's' => 115,
            't' => 116,
            _ => 0,
        }
    }

    fn putchar(&self, x: u8, y: u8, c: char) {
        unsafe { (*self.s).screen[(x + y * 40u8) as usize].write(Screen::to_atari_char(c)) }
    }
}

trait Settable {
    fn set(&mut self, v: u8);
}

trait Gettable {
    fn get(&self) -> u8;
}

impl Byte {
    fn new(addr: u16) -> Self {
        Byte {
            b: addr as *mut ByteWrapper,
        }
    }
}

impl Settable for Byte {
    fn set(&mut self, v: u8) {
        unsafe { (*self.b).v.write(v) }
    }
}

impl Gettable for Byte {
    fn get(&self) -> u8 {
        unsafe { (*self.b).v.read() }
    }
}

pub fn black_box<T>(dummy: T) -> T {
    unsafe { llvm_asm!("" : : "r"(&dummy)) }
    dummy
}

#[inline(never)]
pub fn asm6502() {
    let mut wsync = Byte::new(WSYNC);
    let mut colbk = Byte::new(COLBK);
    let strig0 = Byte::new(STRIG0);
    let screen = Screen::new(SCREEN);

    screen.putchar(0, 0, 'H');
    screen.putchar(1, 0, 'e');
    screen.putchar(2, 0, 'l');
    screen.putchar(3, 0, 'l');
    screen.putchar(4, 0, 'o');
    screen.putchar(5, 0, ' ');
    screen.putchar(6, 0, 'R');
    screen.putchar(7, 0, 'u');
    screen.putchar(8, 0, 's');
    screen.putchar(9, 0, 't');

    let mut x: u8 = 0;
    loop {
        wsync.set(0);
        colbk.set(x);
        x += strig0.get();
    }
}



#[inline(never)]
pub fn asm6502_1() {
    let mut wsync = Byte::new(WSYNC);
    let mut colbk = Byte::new(COLBK);
    let strig0 = Byte::new(STRIG0);
    let screen = Screen::new(SCREEN);

    let mut pos = 1;
    loop {
        screen.putchar(pos + 0, 0, 'H');
        screen.putchar(pos + 1, 0, 'e');
        screen.putchar(pos + 2, 0, 'l');
        screen.putchar(pos + 3, 0, 'l');
        screen.putchar(pos + 4, 0, 'o');
        screen.putchar(pos + 5, 0, ' ');
        screen.putchar(pos + 6, 0, 'R');
        screen.putchar(pos + 7, 0, 'u');
        screen.putchar(pos + 8, 0, 's');
        screen.putchar(pos + 9, 0, 't');
        pos += 1;
    }

    let mut x: u8 = 0;
    loop {
        wsync.set(0);
        colbk.set(x);
        x += strig0.get();
    }
}


pub fn main() {
    let _ = asm6502();
}

#![feature(llvm_asm)]

use volatile_register::RW;

const WSYNC: u16 = 0xD40A;
const COLBK: u16 = 0xD01A;
const STRIG0: u16 = 0x284;

#[repr(C)]
pub struct ByteWrapper {
    pub v: RW<u8>,
}

pub struct Byte {
    b: *mut ByteWrapper,
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
    let mut strig0 = Byte::new(STRIG0);

    let mut x: u8 = 0;
    loop {
        wsync.set(0);
        colbk.set(x);
        x += 1;
        let a = strig0.get();
        strig0.set(a);
    }
}

pub fn main() {
    let _ = asm6502();
}

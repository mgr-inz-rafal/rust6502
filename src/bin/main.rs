#![feature(llvm_asm)]

use volatile_register::WO;
#[repr(C)]
pub struct BYTE {
    pub v: WO<u8>
}

pub fn black_box<T>(dummy: T) -> T {
    unsafe { llvm_asm!("" : : "r"(&dummy)) }
    dummy
}

#[inline(never)]
pub fn asm6502() {
    const WSYNC: u16 = 0xD40A;
    const COLPM0: u16 = 0xD012;

    let wsync = WSYNC as *const BYTE;
    let colpm0 = COLPM0 as *const BYTE;

    let mut x: u8 = 0;
    loop {
        unsafe {
            (*wsync).v.write(0);
            (*colpm0).v.write(x);
        }
        x += 1;
    }
}

pub fn main() {
    let _ = asm6502();
}

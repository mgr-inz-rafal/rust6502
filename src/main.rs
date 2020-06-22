#![feature(llvm_asm)]

use volatile_register::RW;
#[repr(C)]
pub struct COLOR {
    pub csr: RW<u8>
}

pub fn black_box<T>(dummy: T) -> T {
    unsafe { llvm_asm!("" : : "r"(&dummy)) }
    dummy
}

#[inline(never)]
pub fn asm6502() {
    let color = 764 as *const COLOR;

    for i in 0u8..150 {
        unsafe {
            (*color).csr.write(i);
        }
    }
}

pub fn main() {
    let _ = asm6502();
}

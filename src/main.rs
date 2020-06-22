use volatile_register::RW;
#[repr(C)]
pub struct COLOR {
    pub csr: RW<u8>
}

pub fn main() {
    let color = 764 as *const COLOR;

    for i in 0u8..10 {
        unsafe {
            (*color).csr.write(i);
        }
    }
}

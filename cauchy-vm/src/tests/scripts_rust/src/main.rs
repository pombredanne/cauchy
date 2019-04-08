#![no_std]
#![no_main]
#![feature(asm)]
use core::panic::PanicInfo;
use riscv;

/// This function is called on panic.
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

#[no_mangle]
pub extern "C" fn _start() -> ! {
    loop{
     unsafe {
        asm!("mv a7, 97\n\t"
        :
        :
        :
        : "riscv" );
    }
    }
}
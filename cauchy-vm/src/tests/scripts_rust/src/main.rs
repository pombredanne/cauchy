#![no_std]
#![no_main]
#![feature(asm)]
use core::panic::PanicInfo;

/// This function is called on panic.
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

#[no_mangle]
pub extern "C" fn _start() -> ! {
    loop{
     unsafe {
        asm!("
        li a0, 8        
        li a7, 93
        ecall"
        : /* no outputs */
        : 
        : "a7"
        :);
    }
    }
}
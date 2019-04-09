#![no_std]
#![no_main]
#![feature(asm)]
#![feature(type_ascription)]
use core::panic::PanicInfo;

/// This function is called on panic.
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

fn __vm_exit(code: u8) {
    unsafe {
        asm!("
        mv a0, $0     
        li a7, 93
        ecall"
        : /* no outputs */
        : "r"(code)
        : "a0", "a7" );
    }
}

fn __vm_send(txid : &[u8], msg: &[u8]) {
    let txid_sz = txid.len();
    let msg_sz = msg.len();

    unsafe {
        asm!("
        mv a3, $0
        li a4, $1
        mv a5, $2
        li a6, $3
        li a7, 0xCBFF
        ecall
        "
        : /* no outputs */
        : "r"(txid as *const _ as *const u8), "i"(txid_sz), "r"(msg as *const _ as *const u8), "i"(msg_sz)
        : "a3", "a4", "a5", "a6", "a7" );
    }
}

fn __vm_recv(txid : &mut [u8], msg: &mut [u8]) {
    let txid_sz = txid.len() as u32;
    let msg_sz = msg.len() as u32;

    unsafe {
        asm!("
        mv a3, $0
        li a4, $1
        mv a5, $2
        li a6, $3
        li a7, 0xCBFE
        ecall
        "
        : /* no outputs */
        : "r"(txid as *const _ as *const u8), "i"(txid_sz), "r"(msg as *const _ as *const u8), "i"(msg_sz)
        : "a3", "a4", "a5", "a6", "a7" );
    }
}

#[no_mangle]
pub extern "C" fn _start()  {
     let mut sender_txid = [0x48 : u8, 64];
     let mut data = [0x48 : u8, 64];
     __vm_recv(&mut sender_txid, &mut data);
     __vm_send(&sender_txid, &data);
     __vm_send(&b"RECVR_RUST"[..], &b"DEADBEEF is happyBEEF, especially RUSTED beef!"[..]);
     __vm_exit(8);
}
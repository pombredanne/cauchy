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
pub extern "C" fn abort() -> ! {
    loop{}
}

fn __vm_exit(code: u8) {
    unsafe {
        asm!("
        li a7, 0xCBF8
        ecall
        mv a0, $0     
        li a7, 93
        ecall"
        : /* no outputs */
        : "r"(code)
        : "a0", "a7" );
    }
}

fn __vm_send(txid : &[u8], msg: &[u8]) {
    let txid_sz = txid.len() as u64;
    let msg_sz = msg.len() as u64;

    unsafe {
        asm!("
        mv a3, $0
        mv a4, $1
        mv a5, $2
        mv a6, $3
        li a7, 0xCBFF
        ecall
        "
        : /* no outputs */
        : "r"(txid as *const _ as *const u8), "r"(txid_sz as u64), "r"(msg as *const _ as *const u8), "r"(msg_sz as u64)
        : "a3", "a4", "a5", "a6", "a7" );
    }
}

fn __vm_recv(txid : &mut [u8], msg: &mut [u8]) -> (usize, usize) {
    let mut txid_sz = txid.len();
    let mut msg_sz = msg.len();

    let txid_buff : [u8;256] = [0x41;256];
    let msg_buff : [u8;256] = [0x42;256];

    unsafe {
        asm!("
        mv a3, $2
        mv a4, $3
        mv a5, $4
        mv a6, $5
        li a7, 0xCBFE
        ecall
        mv $0, s1
        mv $1, s2"
        : "=r"(txid_sz), "=r"(msg_sz)
        : "r"(&txid_buff as *const _ as *const u8), "r"(txid_sz), "r"(&msg_buff as *const _ as *const u8), "r"(msg_sz)
        : "s0", "s1", "a3", "a4", "a5", "a6", "a7" );
    }
    let mut idx = 0;
    for b in txid_buff[..txid_sz].iter() {
        txid[idx] = *b;
        idx+=1;
    }

    let mut idx = 0;
    for b in msg_buff[..msg_sz].iter() {
        msg[idx] = *b;
        idx+=1;
    }
    (txid_sz, msg_sz)
}

#[no_mangle]
pub extern "C" fn _start()  {
     let mut sender_txid : [u8;32] = [0x48; 32];
     let mut data : [u8;32] = [0x48; 32];
     let (txid_sz, msg_sz) = __vm_recv(&mut sender_txid, &mut data);
     __vm_send(&sender_txid, &data);
     __vm_send(b"RECVR_RUST", b"DEADBEEF is happyBEEF, especially RUSTED beef!");
    // __vm_send(b"abcd", b"bazy");
     __vm_exit(8);
}
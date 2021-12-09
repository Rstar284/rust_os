#![feature(abi_x86_interrupt)]
#![no_std]
#![no_main]

mod vga_buffer;
mod gdt;
mod interrupts;

#[no_mangle]
pub extern "C" fn _start() -> ! {
    init();
    hlt_loop();
}

use core::panic::PanicInfo;
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    hlt_loop();
}


pub fn init() {
    gdt::init();
    interrupts::init_idt();
    unsafe { interrupts::PICS.lock().initialize() }
    x86_64::instructions::interrupts::enable();
    println!("Hi there! :)")
}


pub fn hlt_loop() -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}

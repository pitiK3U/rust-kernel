//! Hobby/learning simple kernel using rust.
#![feature(lang_items)]
#![feature(asm)]
#![no_std]
#![no_main]

#![warn(missing_docs)]


mod monitor;


// dev profile: easier to debug panics; can put a breakpoint on `rust_begin_unwind`
// #[cfg(debug_assertions)]
// use panic_halt as _;

// release profile: minimize the binary size of the application
// #[cfg(not(debug_assertions))]
// use panic_abort as _;


use core::panic::PanicInfo;

#[panic_handler]
fn panic(_panic: &PanicInfo<'_>) -> ! {
    loop {}
}

#[lang = "eh_personality"]
extern "C" fn eh_personality() {}

static HELLO: &[u8] = b"Hello\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\nasdf\rWorld";

/// Initial kernel function that gets called by `src/boot.s`.
#[no_mangle]
pub extern "C" fn _start() -> ! {
    monitor::VGA::Monitor::clear();

    monitor::VGA::Monitor::write_str(HELLO);

    loop {}
}


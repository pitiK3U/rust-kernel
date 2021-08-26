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

static HELLO: &[u8] = b"Hello\n\n\n\nWorld";

/// Initial kernel function that gets called by `src/boot.s`.
#[no_mangle]
pub extern "C" fn _start() -> ! {
    let mut buf: monitor::VGA::Monitor = unsafe { monitor::VGA::BUFFER.take() };

    buf.clear();

    for byte in HELLO.iter() {
        buf.write_byte(*byte);
    }

    loop {}
}


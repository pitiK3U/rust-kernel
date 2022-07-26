//! Hobby/learning simple kernel using rust.
#![feature(lang_items)]
#![no_std]
#![no_main]

#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]

#![feature(const_mut_refs)]
#![feature(const_raw_ptr_deref)]

#![warn(missing_docs)]

extern crate bit_field;


mod monitor;
mod essentials;
mod interrupts;
mod port;
mod test;

// dev profile: easier to debug panics; can put a breakpoint on `rust_begin_unwind`
// #[cfg(debug_assertions)]
// use panic_halt as _;

// release profile: minimize the binary size of the application
// #[cfg(not(debug_assertions))]
// use panic_abort as _;

use core::panic::PanicInfo;

#[panic_handler]
fn panic(info: &PanicInfo<'_>) -> ! {
    println!("{}", info);
    loop {}
}

#[lang = "eh_personality"]
extern "C" fn eh_personality() {}

static HELLO: &str = "Hello\tWöorld\n";

/// Initial kernel function that gets called by `src/boot.s`.
#[no_mangle]
pub extern "C" fn _start() -> ! {

    interrupts::IDT::init();

    // let mut writer = BUFFER.lock();

    // writer.set_background_color(&Color::Black);
    // writer.set_foreground_color(&Color::White);

    // writer.clear();

    // writer.write_str(HELLO);
    // println!("{}", HELLO);

    #[cfg(test)]
    test_main();

    loop {}
}

#[cfg(test)]
fn test_runner(tests: &[&dyn Fn()]) {
    use test::*;

    println!("Running {} tests", tests.len());
    for test in tests {
        test();
    }

    test::exit_qemu(QemuExitCode::Failed);
}

#[test_case]
fn trivial_assertion() {
    print!("trivial assertion... ");
    assert_eq!(1, 1);
    println!("[ok]");
}


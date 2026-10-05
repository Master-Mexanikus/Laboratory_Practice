#![no_std]
#![no_main]

use cortex_m as _;
use cortex_m_rt::entry;
use panic_rtt_target as _;
use rtt_target::{rprintln, rtt_init_print};
#[allow(unused_imports)]
use stm32f4::stm32f411 as _;

#[entry]
fn main() -> ! {
    rtt_init_print!();
    rprintln!("Hello, Rust!");

    loop {}
}
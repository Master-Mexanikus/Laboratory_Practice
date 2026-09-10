#![no_std]
#![no_main]

use cortex_m_rt::entry;
use panic_rtt_target as _;
use stm32f4::stm32f411;

#[entry]
fn main() -> ! {
    let _ = core::mem::size_of::<stm32f411::Peripherals>();
    loop {}
}

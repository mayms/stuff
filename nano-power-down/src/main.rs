#![no_std]
#![no_main]
#![feature(asm_experimental_arch)]
#![feature(abi_avr_interrupt)]

use arduino_hal::prelude::*;
use panic_halt as _;
use core::arch::asm;

#[arduino_hal::entry]
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);
    let mut serial = arduino_hal::default_serial!(dp, pins, 57600);

    ufmt::uwriteln!(&mut serial, "# startup").void_unwrap();
    ufmt::uwriteln!(&mut serial, "# setup watchdog").void_unwrap();
    avr_device::interrupt::free(|_| {
        dp.WDT.wdtcsr.write(|w| w
            .wdce().set_bit()
            .wde().set_bit());
        dp.WDT.wdtcsr.write(|w| w
            .wdce().clear_bit()
            .wdie().set_bit()
            .wde().clear_bit()
            .wdph().set_bit());
        dp.CPU.smcr.write(|w| w
            .sm().bits(0b010)// power-down sleep mode
            .se().set_bit());
    });
    let wdtcsr = dp.WDT.wdtcsr.read().bits();
    let smcr = dp.CPU.smcr.read().bits();
    unsafe { avr_device::interrupt::enable() };
    ufmt::uwriteln!(&mut serial, "## WDTCSR {}", wdtcsr).void_unwrap();
    ufmt::uwriteln!(&mut serial, "## SMCR {}", smcr).void_unwrap();

    let mcusr = dp.CPU.mcusr.read().bits();
    let mut sreg: u8 = 0;
    unsafe {
        asm!(
        "in {0}, 0x3f", // SREG
        out(reg) sreg
        );
    }
    ufmt::uwriteln!(&mut serial, "## MCUSR {}", mcusr).void_unwrap();
    ufmt::uwriteln!(&mut serial, "## SREG {}", sreg).void_unwrap();

    loop {
        ufmt::uwriteln!(&mut serial, "## loop").void_unwrap();
        arduino_hal::delay_ms(2_000);
        ufmt::uwriteln!(&mut serial, "## go to sleep").void_unwrap();
        arduino_hal::delay_ms(10);
        unsafe {
            asm!(
            "sleep"
            );
        }
        arduino_hal::delay_ms(10);
        ufmt::uwriteln!(&mut serial, "## returned from sleep").void_unwrap();
    }
}

#[avr_device::interrupt(atmega328p)]
fn WDT() {
    avr_device::interrupt::free(|_| {
        unsafe {
            asm!(
            "wdr"
            );
        }
    });
}

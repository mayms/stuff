#![no_std]
#![no_main]
#![feature(asm_experimental_arch)]
#![feature(abi_avr_interrupt)]

use arduino_hal::{hal, Usart};
use arduino_hal::prelude::*;
use panic_halt as _;
use arduino_hal::hal::wdt;
use core::arch::asm;

#[arduino_hal::entry]
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);
    let mut serial = arduino_hal::default_serial!(dp, pins, 57600);

    // avr_device::interrupt::free(|_| {
    //     dp.WDT.wdtcsr.modify(|_, w| w
    //         .wdce().set_bit()
    //         .wde().set_bit());
    //     dp.WDT.wdtcsr.modify(|_, w| w
    //         .wdce().clear_bit()
    //         .wdie().set_bit()
    //         .wde().clear_bit()
    //         .wdph().set_bit());
    //     dp.CPU.smcr.modify(|_, w| w
    //         .sm().bits(0b010)// power-down sleep mode
    //         .se().set_bit());
    // });
    // unsafe { avr_device::interrupt::enable() };

    ufmt::uwriteln!(&mut serial, "# startup").void_unwrap();
    ufmt::uwriteln!(&mut serial, "# setup watchdog").void_unwrap();
    let mut wdtcsr: u8 = 0; // read WDTCSR
    let mut smcr: u8 = 0; // read SMCR
    unsafe {
        asm!(
        "CLI",
        "WDR",
        "STS 0x60, {0}",    // WDTCSR
        "STS 0x60, {1}",    // WDTCSR
        "out 0x33, {2}",    // SMCR
        "SEI",
        "LDS {3}, 0x60",    // WDTCSR,
        "in {4}, 0x33",      // SMCR
        //========WWWW_WWWW
        //========DDDD_DDDD
        //========IIPC_EPPP
        //========FE3E__210
        in(reg) 0b0001_1000 as u8, // WDCE=1, WDE=1
        in(reg) 0b0110_0000 as u8, // WDIE 1, WDP3 1, WDE 0
        //--------XXXX_SSSS
        //--------XXXX_MMME
        //--------XXXX_210-
        in(reg) 0b0000_0101 as u8,  // Power down SM2=0, SM1=1, SM0=0, SE=1
        out(reg) wdtcsr,
        out(reg) smcr
        );
    }
    ufmt::uwriteln!(&mut serial, "## WDTCSR {}", wdtcsr).void_unwrap();
    ufmt::uwriteln!(&mut serial, "## SMCR {}", smcr).void_unwrap();

    let mut mcusr: u8 = 0;
    let mut sreg: u8 = 0;
    unsafe {
        asm!(
        "in {0}, 0x35", // MCUSR
        "in {1}, 0x3f", // SREG
        out(reg) mcusr,
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

// fn asm_add() {
//     let a: i8 = 3;
//     let b: i8 = 11;
//     let mut o: i8 = 0;
//     unsafe {
//         asm!(
//         "mov {0}, {1}",
//         "add {0}, {2}",
//         out(reg) o, in(reg) a, in(reg) b
//         );
//     }
//     ufmt::uwriteln!(&mut serial, "## asm {}", o).void_unwrap();
// }
//
// fn asm_read_io_port() {
//     let mut o: i8 = 0;
//     unsafe {
//         asm!(
//         "in {0}, 0x05",
//         out(reg) o
//         );
//     }
//     ufmt::uwriteln!(&mut serial, "## asm {}", o & 0b0010_0000 > 5).void_unwrap();
// }

#![no_std]
use core::arch::asm;
use core::ffi::c_void;
use cortex_m::interrupt::{InterruptNumber, Nr};
use cortex_m::peripheral::NVIC;
use cortex_m::peripheral::syst::SystClkSource;
use cortex_m_rt::{exception, heap_start};
use stm32f1xx_hal::flash::FlashExt;
use stm32f1xx_hal::pac;
use stm32f1xx_hal::rcc::RccExt;
use stm32f1xx_hal::time::Hertz;


/// Low level initialization. The low level initialization function will
/// perform basic low level initialization of the hardware.
pub trait LowLevelInit {
    /// The input is the number of ticks per second that ThreadX will be
    /// expecting. The output is a pointer to a slice that is used as the
    /// heap memory. This function is also exptected to set
    /// the system stack pointer. The variable is exposed by threadx_sys
    /// as threadx_sys::_tx_thread_system_stack_ptr 
    fn low_level_init(ticks_per_second: u32) -> Result<(),()>;
}

// cortexm-rt crate defines the _stack_start function. Due to the action of flip-link, the stack pointer 
// is moved lower down in memory after leaving space for the bss and data sections.
extern "C" {
    static _stack_start: u32;
}

pub struct BoardStm32f103c8BluePill;

impl LowLevelInit for BoardStm32f103c8BluePill {
    fn low_level_init(ticks_per_second: u32) -> Result<(),()> {

        unsafe {
        let stack_start = &_stack_start as *const u32 as u32;
            threadx_sys::_tx_thread_system_stack_ptr = stack_start as *mut c_void;
            defmt::println!("Low level init.  Stack at: {=u32:#x} Ticks per second:{}",stack_start, ticks_per_second);

            defmt::println!("Stack size {}",    stack_start - 0x2000_0000);
        }

        
            let p = pac::Peripherals::take().unwrap();
            let rcc = p.RCC.constrain();
            let mut flash = p.FLASH.constrain();

            let clocks = rcc
                .cfgr
                .use_hse(Hertz::MHz(8))
                .sysclk(Hertz::MHz(72))
                .hclk(Hertz::MHz(64))
                .pclk1(Hertz::MHz(36))
                .pclk2(Hertz::MHz(64))
                .freeze(&mut flash.acr);

            let cp = cortex_m::Peripherals::take().unwrap();
            let mut syst = cp.SYST;
            let mut nvic = cp.NVIC;
            let mut dcb = cp.DCB;
            dcb.enable_trace();
            let mut dbg = cp.DWT;
            // configures the system timer to trigger a SysTick exception every second
            dbg.enable_cycle_counter();

            syst.set_clock_source(SystClkSource::Core);
            syst.set_reload( (72_000_000 / ticks_per_second) - 1);
            syst.enable_counter();
            syst.enable_interrupt();


            defmt::println!("Low level init");

            //Set up the priorities for SysTick and PendSV and SVC
            unsafe {
                asm!(
                    "MOV     r0, #0xE000E000",
                    "LDR     r1, =0x00000000",
                    "STR     r1, [r0, #0xD18]",
                    "LDR     r1, =0xFF000000",
                    "STR     r1, [r0, #0xD1C]",
                    "LDR     r1, =0x40FF0000",
                    "STR     r1, [r0, #0xD20]",
                );
            }
            defmt::println!("Int prio set");
            Ok(())
            
    }
}


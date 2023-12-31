#![no_main]
#![no_std]

// ThreadX tests running on an stm32f103 bluepill board

use defmt_rtt as _;
use panic_probe as _;
use stm32f1xx_hal as _; // memory layout

use threadx_rs::Builder;

#[defmt_test::tests]
mod tests {
    use core::time::Duration;
    use defmt::{assert_eq, unwrap};
    use threadx_rs::Builder;

    struct Board{}


    #[init]
    fn init() -> Board {

        //let tx = Builder::new(low_level_init_cb, app_define_cb)

        Board {  }
    }

    #[test]
    fn confirm_firmware_version(board: &mut Board) {
        
    }

}
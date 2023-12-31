use core::time::Duration;

use threadx_sys::{TX_TIMER_TICKS_PER_SECOND, _tx_thread_sleep};

use crate::tx_checked_call;

use super::error::TxError;
use defmt::debug;
use defmt::error;
use num_traits::FromPrimitive;

pub struct TxTicks(u32);

/// `threadx_sys::TX_TIMER_TICKS_PER_SECOND` is a constant that is set by the
/// ThreadX build configuration. The default is 100 and it can be
/// changed by providing a user defined `tx_user.h` file.
const MILLIS_PER_TICK: u128 = 1000 / TX_TIMER_TICKS_PER_SECOND as u128;

impl From<Duration> for TxTicks {
    fn from(d: Duration) -> Self {
        TxTicks((d.as_millis() / MILLIS_PER_TICK) as u32)
    }
}

impl Into<u32> for TxTicks {
    fn into(self) -> u32 {
        self.0
    }
}


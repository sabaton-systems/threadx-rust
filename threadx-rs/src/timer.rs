/*
UINT        _tx_timer_activate(TX_TIMER *timer_ptr);
UINT        _tx_timer_change(TX_TIMER *timer_ptr, ULONG initial_ticks, ULONG reschedule_ticks);
UINT        _tx_timer_create(TX_TIMER *timer_ptr, CHAR *name_ptr,
                VOID (*expiration_function)(ULONG input), ULONG expiration_input,
                ULONG initial_ticks, ULONG reschedule_ticks, UINT auto_activate);
UINT        _tx_timer_deactivate(TX_TIMER *timer_ptr);
UINT        _tx_timer_delete(TX_TIMER *timer_ptr);
UINT        _tx_timer_info_get(TX_TIMER *timer_ptr, CHAR **name, UINT *active, ULONG *remaining_ticks,
                ULONG *reschedule_ticks, TX_TIMER **next_timer);
UINT        _tx_timer_performance_info_get(TX_TIMER *timer_ptr, ULONG *activates, ULONG *reactivates,
                ULONG *deactivates, ULONG *expirations, ULONG *expiration_adjusts);
UINT        _tx_timer_performance_system_info_get(ULONG *activates, ULONG *reactivates,
                ULONG *deactivates, ULONG *expirations, ULONG *expiration_adjusts);

ULONG       _tx_time_get(VOID);
VOID        _tx_time_set(ULONG new_time);
*/

use core::ffi::c_void;
use core::ffi::CStr;
use crate::time::TxTicks;
use crate::tx_checked_call;

use super::WaitOption;
use super::error::TxError;
use defmt::debug;
use defmt::error;
use num_traits::FromPrimitive;
use thiserror_no_std::Error;
use threadx_sys::_tx_timer_create;
use threadx_sys::ULONG;

use core::mem::MaybeUninit;
use threadx_sys::TX_TIMER;

type TimerCallbackType = unsafe extern "C" fn(ULONG);

unsafe extern "C" fn timer_callback_trampoline<F>(arg: ULONG)
where F: Fn(ULONG)
{
    let closure = &mut *(arg as *mut F);
    closure(arg);
}

fn get_trampoline<F>(closure_: &F) -> TimerCallbackType
where
    F: Fn(ULONG),
{
    timer_callback_trampoline::<F>
}

pub struct Timer(MaybeUninit<TX_TIMER>);

impl Timer {
    pub const fn new() -> Self {
        Timer(MaybeUninit::uninit())
    }

    pub fn initialize<F: Fn(ULONG)>(
        &'static mut self,
        name: &CStr,
        expiration_function: F,
        expiration_input: ULONG,
        initial_ticks: core::time::Duration,
        reschedule_ticks: core::time::Duration,
        auto_activate: bool,
    ) -> Result<(), TxError> {
        let timer = unsafe { &mut *self.0.as_mut_ptr() };
        
        //convert to a ULONG
        let trampoline = get_trampoline(&expiration_function);

        let initial_ticks = TxTicks::from(initial_ticks).into();
        let reschedule_ticks = TxTicks::from(reschedule_ticks).into();
        let auto_activate = if auto_activate { 1 } else { 0 };
        
        tx_checked_call!(_tx_timer_create(
                timer,
                name.as_ptr() as *mut i8,
                Some(trampoline),
                expiration_input,
                initial_ticks,
                reschedule_ticks,
                auto_activate
            )).map(|_| ())?;
        Ok(())
    }
}
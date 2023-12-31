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

use core::mem::MaybeUninit;

use threadx_sys::TX_TIMER;

pub struct Timer(MaybeUninit<TX_TIMER>);

impl Timer {
    pub const fn new() -> Self {
        Timer(MaybeUninit::uninit())
    }

    // pub fn initialize(
    //     &'static mut self,
    //     name: &CStr,
    //     expiration_function: Option<fn(ULONG)>,
    //     expiration_input: ULONG,
    //     initial_ticks: ULONG,
    //     reschedule_ticks: ULONG,
    //     auto_activate: bool,
    // ) -> Result<TimerHandle, TxError> {
    //     let timer = unsafe { &mut *self.0.as_mut_ptr() };
    //     let expiration_function = expiration_function.unwrap_or_else(|| {
    //         panic!("expiration_function must be provided");
    //     });
    //     let expiration_function = expiration_function as *mut _ as *mut c_void;
    //     let expiration_input = expiration_input as ULONG;
    //     let initial_ticks = initial_ticks as ULONG;
    //     let reschedule_ticks = reschedule_ticks as ULONG;
    //     let auto_activate = if auto_activate { 1 } else { 0 };
    //     let status = unsafe {
    //         _tx_timer_create(
    //             timer,
    //             name.as_ptr(),
    //             expiration_function,
    //             expiration_input,
    //             initial_ticks,
    //             reschedule_ticks,
    //             auto_activate,
    //         )
    //     };
    //     if status != 0 {
    //         return Err(TxError::from_u32(status));
    //     }
    //     Ok(TimerHandle { timer })
    // }
}
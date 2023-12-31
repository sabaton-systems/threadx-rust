use core::ffi::CStr;
use core::mem::MaybeUninit;

use threadx_sys::{_tx_event_flags_delete, _tx_event_flags_get, ULONG, _tx_event_flags_set, _tx_event_flags_set_notify};
use threadx_sys::{TX_EVENT_FLAGS_GROUP,_tx_event_flags_create};

use crate::tx_checked_call;

use super::WaitOption;
use super::error::TxError;
use defmt::debug;
use defmt::error;
use num_traits::FromPrimitive;

#[repr(u32)]
pub enum GetOption {
    WaitAll = threadx_sys::TX_AND,
    WaitAny = threadx_sys::TX_OR,
}

#[repr(u32)]
pub enum SetOption {
    SetAll = threadx_sys::TX_AND,
    SetAny = threadx_sys::TX_OR,
}
pub struct EventFlagsGroup(MaybeUninit<TX_EVENT_FLAGS_GROUP>);

impl EventFlagsGroup {
    pub const fn new() -> Self{
        EventFlagsGroup(core::mem::MaybeUninit::uninit())
    }

    pub fn create(&mut self, name: &CStr) -> Result<EventFlagsGroupHandle,TxError> {
        let group_ptr = self.0.as_mut_ptr();
        if group_ptr.is_null() {
            panic!("EventFlagsGroup ptr is null");
        }
        unsafe {
            if !(*group_ptr).tx_event_flags_group_name.is_null() {
                panic!("EventFlagsGroup is already initialized");
            }
        }
        tx_checked_call!(_tx_event_flags_create(
            group_ptr,
            name.as_ptr() as *mut i8
        ))
        .map(|_| EventFlagsGroupHandle::new(group_ptr))
    }
}

pub struct EventFlagsGroupHandle(*mut TX_EVENT_FLAGS_GROUP);

unsafe impl Sync for EventFlagsGroupHandle {}
unsafe impl Send for EventFlagsGroupHandle  {}
impl EventFlagsGroupHandle {
    fn new(group_ptr: *mut TX_EVENT_FLAGS_GROUP) -> Self {
        assert!(!group_ptr.is_null(),"EventFlagsGroupHandle::new group_ptr is null");
        EventFlagsGroupHandle(group_ptr)
    }
    pub fn delete(self) -> Result<(),TxError> {
        tx_checked_call!(_tx_event_flags_delete(self.0))
    }

    pub fn get(&mut self, requested_flags: u32, get_option: GetOption, wait_option: WaitOption) -> Result<u32,TxError> {
        let mut actual_flags = 0u32;
        tx_checked_call!(_tx_event_flags_get(self.0, requested_flags, get_option as ULONG, &mut actual_flags, wait_option as ULONG))?;
        Ok(actual_flags)
    }

    pub fn set(&mut self, flags_to_set: u32, set_option: SetOption) -> Result<(),TxError> {
        tx_checked_call!(_tx_event_flags_set(self.0, flags_to_set, set_option as ULONG))
    }

    pub fn on_notify(&mut self, mut notify: fn(EventFlagsGroupHandle)) -> Result<(),TxError> {
        let trampoline = get_notify_trampoline(&notify);
        tx_checked_call!(_tx_event_flags_set_notify(self.0, Some(trampoline)))
    }
}

type TxEventNotifyCallback = unsafe extern "C" fn(*mut TX_EVENT_FLAGS_GROUP);

unsafe extern "C" fn cb_trampoline<F>(arg: *mut TX_EVENT_FLAGS_GROUP)
where F: Fn(EventFlagsGroupHandle)
{
    let closure = &mut *(arg as *mut F);
    closure(EventFlagsGroupHandle::new(arg));
}

fn get_notify_trampoline<F>(_: &F) -> TxEventNotifyCallback
where
    F: Fn(EventFlagsGroupHandle),
{
    cb_trampoline::<F>
}

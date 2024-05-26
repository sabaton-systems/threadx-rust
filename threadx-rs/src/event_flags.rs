use core::ffi::CStr;
use core::marker::PhantomData;
use core::mem::MaybeUninit;
use core::ptr;

use threadx_sys::{_tx_event_flags_delete, _tx_event_flags_get, ULONG, _tx_event_flags_set, _tx_event_flags_set_notify};
use threadx_sys::{TX_EVENT_FLAGS_GROUP,_tx_event_flags_create};

use crate::tx_checked_call;

use super::WaitOption;
use super::error::TxError;
use defmt::{debug, println, trace};
use defmt::error;
use num_traits::FromPrimitive;

#[derive(Copy,Clone)]
#[repr(u32)]
pub enum GetOption {
    WaitAll = threadx_sys::TX_AND,
    WaitAllAndClear = threadx_sys::TX_AND_CLEAR,
    WaitAny = threadx_sys::TX_OR,
    WaitAnyAndClear = threadx_sys::TX_OR_CLEAR,
}

#[derive(Copy,Clone)]
#[repr(u32)]
pub enum SetOption {
    SetAndClear = threadx_sys::TX_AND,
    SetAny = threadx_sys::TX_OR,
}
pub struct EventFlagsGroup(pub MaybeUninit<TX_EVENT_FLAGS_GROUP>);

impl EventFlagsGroup {
    pub const fn new() -> Self{
        EventFlagsGroup(core::mem::MaybeUninit::uninit())
    }

    pub fn initialize(&'static mut self, name: &CStr) -> Result<(),TxError> {
        let group_ptr = self.0.as_mut_ptr();
        if group_ptr.is_null() {
            panic!("EventFlagsGroup ptr is null");
        }
        unsafe {
            if !(*group_ptr).tx_event_flags_group_name.is_null() {
                panic!("EventFlagsGroup is already initialized");
            }
        }
        trace!("EventFlagsGroup::initialize: ptr is: {}",group_ptr);
        tx_checked_call!(_tx_event_flags_create(
            group_ptr,
            name.as_ptr() as *mut i8
        ))?;
        Ok(())
    }

    pub fn publish(&'static self, flags_to_set: u32) -> Result<(),TxError> {
        let group_ptr = self.0.as_ptr();
        let group_ptr = group_ptr as *mut TX_EVENT_FLAGS_GROUP;
        if group_ptr.is_null() {
            return Err(TxError::Unknown);
        }
        tx_checked_call!(_tx_event_flags_set(group_ptr, flags_to_set,0))
    }

    pub fn get(&'static self, requested_flags: u32, get_option: GetOption, wait_option: WaitOption) -> Result<u32,TxError> {
        let group_ptr = self.0.as_ptr();
        let group_ptr = group_ptr as *mut TX_EVENT_FLAGS_GROUP;
        if group_ptr.is_null() {
            return Err(TxError::Unknown);
        }
        let mut actual_flags = 0u32;
        tx_checked_call!(_tx_event_flags_get(group_ptr, requested_flags, get_option as ULONG, &mut actual_flags, wait_option as ULONG))?;
        Ok(actual_flags)
    }
}




#[deprecated]
#[derive(Clone)]
pub struct EventFlagsGroupHandle<'a>(*mut TX_EVENT_FLAGS_GROUP,core::marker::PhantomData<&'a()>);

unsafe impl Sync for EventFlagsGroupHandle<'_> {}
unsafe impl Send for EventFlagsGroupHandle<'_>  {}
impl <'a>EventFlagsGroupHandle<'a> {
    fn new(group_ptr: *mut TX_EVENT_FLAGS_GROUP) -> Self {
        assert!(!group_ptr.is_null(),"EventFlagsGroupHandle::new group_ptr is null");
        EventFlagsGroupHandle(group_ptr,core::marker::PhantomData)
  
    }
    pub fn delete(self) -> Result<(),TxError> {
        // convert reference to pointer
        let self_ptr = self.0 as *const TX_EVENT_FLAGS_GROUP as *mut TX_EVENT_FLAGS_GROUP;
        tx_checked_call!(_tx_event_flags_delete(self_ptr))
    }

    pub fn get(&self, requested_flags: u32, get_option: GetOption, wait_option: WaitOption) -> Result<u32,TxError> {
        debug!("EventFlagsGroupHandle::get requested_flags: {:?}",requested_flags);
        let mut actual_flags = 0u32;
        
        println!("1");
        
        println!("EventFlagsGroupHandle::get self_ptr: {}",self.0);
        println!("Foo");
        tx_checked_call!(_tx_event_flags_get(self.0, requested_flags, get_option as ULONG, &mut actual_flags, wait_option as ULONG))?;
        Ok(actual_flags)
    }

    pub fn set(&self, flags_to_set: u32, set_option: SetOption) -> Result<(),TxError> {
        debug!("EventFlagsGroupHandle::set flags_to_set: {:?}",flags_to_set);

        
        //let self_ptr = self_ptr as *mut _;
        //println!("EventFlagsGroupHandle::get self_ptr: {}",self_ptr);
        println!("Bar");
        
        tx_checked_call!(_tx_event_flags_set(self.0, flags_to_set,2))
    }

    pub fn on_notify(&mut self, mut notify: fn(EventFlagsGroupHandle)) -> Result<(),TxError> {
        let trampoline = get_notify_trampoline(&notify);
        let self_ptr = self.0 as *const TX_EVENT_FLAGS_GROUP as *mut TX_EVENT_FLAGS_GROUP;
        tx_checked_call!(_tx_event_flags_set_notify(self_ptr, Some(trampoline)))
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

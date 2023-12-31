
use core::mem::size_of;
use core::{mem::MaybeUninit, ffi::CStr, marker::PhantomData};
use crate::tx_checked_call;
use super::{error::TxError, WaitOption};
use defmt::debug;
use defmt::error;
use num_traits::FromPrimitive;
use threadx_sys::{TX_SEMAPHORE, _tx_semaphore_create, _tx_semaphore_delete, _tx_semaphore_get, _tx_semaphore_put, _tx_semaphore_prioritize, _tx_semaphore_put_notify};

/*
#define tx_semaphore_ceiling_put                    _tx_semaphore_ceiling_put
#define tx_semaphore_create                         _tx_semaphore_create
#define tx_semaphore_delete                         _tx_semaphore_delete
#define tx_semaphore_get                            _tx_semaphore_get
#define tx_semaphore_info_get                       _tx_semaphore_info_get
#define tx_semaphore_performance_info_get           _tx_semaphore_performance_info_get
#define tx_semaphore_performance_system_info_get    _tx_semaphore_performance_system_info_get
#define tx_semaphore_prioritize                     _tx_semaphore_prioritize
#define tx_semaphore_put                            _tx_semaphore_put
#define tx_semaphore_put_notify                     _tx_semaphore_put_notify
*/

pub struct Semaphore(MaybeUninit<TX_SEMAPHORE>);

impl Semaphore {
    pub const fn new() -> Self {
        Semaphore(MaybeUninit::<TX_SEMAPHORE>::uninit())
    }

    pub fn initialize(
        &'static mut self,
        name: &CStr,
        initial_count: u32,
    ) -> Result<SemaphoreOwnerHandle, TxError> {
        let sem_ptr = self.0.as_mut_ptr();
        if sem_ptr.is_null() {
            panic!("Semaphore ptr is null");
        }
        unsafe {
            if !(*sem_ptr).tx_semaphore_name.is_null() {
                panic!("Semaphore is already initialized");
            }
        }
        tx_checked_call!(_tx_semaphore_create(
            sem_ptr,
            name.as_ptr() as *mut i8,
            initial_count as u32
        ))
        .map(|_| SemaphoreOwnerHandle::new(sem_ptr))
    }
}

#[derive(Clone,Copy)]
pub struct SemaphoreOwnerHandle(*mut TX_SEMAPHORE);
pub struct SemaphoreUserHandle(*mut TX_SEMAPHORE);

pub trait SemaphoreOwner {
    fn delete(self) -> Result<(),TxError> ;
    fn get_semaphore_user(&self) -> SemaphoreUserHandle;
}

pub trait SemaphoreUser {
    fn get(&self, wait: WaitOption) -> Result<(), TxError>;
    fn put(&self) -> Result<(), TxError>;
    fn prioritize(&self) -> Result<(), TxError>;
    fn semaphore_put_notify(&self, notify: fn(SemaphoreUserHandle)) -> Result<(), TxError>;
}

impl SemaphoreOwnerHandle {
    fn new(sem_ptr: *mut TX_SEMAPHORE) -> Self {
        assert!(!sem_ptr.is_null(),"SemaphoreOwnerHandle::new sem_ptr is null");
        SemaphoreOwnerHandle(sem_ptr)
    }
}

impl SemaphoreOwner for SemaphoreOwnerHandle {
    fn delete(self) -> Result<(),TxError> {
        tx_checked_call!(_tx_semaphore_delete(self.0))
    }
    fn get_semaphore_user(&self) -> SemaphoreUserHandle {
        SemaphoreUserHandle(self.0.clone())
    }
}

impl SemaphoreUser for SemaphoreUserHandle {
    fn get(&self, wait: WaitOption) -> Result<(), TxError> {
        tx_checked_call!(_tx_semaphore_get(
            self.0,
            wait as u32
        ))
    }
    fn put(&self) -> Result<(), TxError> {
        tx_checked_call!(_tx_semaphore_put(
            self.0
        ))
    }

    fn prioritize(&self) -> Result<(), TxError> {
        tx_checked_call!(_tx_semaphore_prioritize(
            self.0
        ))
    }

    fn semaphore_put_notify(&self, notify: fn(SemaphoreUserHandle)) -> Result<(), TxError> {
        let trampoline = get_notify_trampoline(&notify);
        tx_checked_call!(_tx_semaphore_put_notify(
            self.0,
            Some(trampoline)
        ))
    }
}

type SemaphoreNotifyCallback = unsafe extern "C" fn(*mut TX_SEMAPHORE);

unsafe extern "C" fn semaphore_cb_trampoline<F>(arg: *mut TX_SEMAPHORE)
where F: Fn(SemaphoreUserHandle)
{
    let closure = &mut *(arg as *mut F);
    closure(SemaphoreUserHandle(arg));
}

fn get_notify_trampoline<F>(_: &F) -> SemaphoreNotifyCallback
where
    F: Fn(SemaphoreUserHandle),
{
    semaphore_cb_trampoline::<F>
}

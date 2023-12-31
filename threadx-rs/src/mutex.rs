use core::borrow::Borrow;
use core::borrow::BorrowMut;
use core::cell::UnsafeCell;
use core::f32::consts::E;
use core::ffi::CStr;
use core::mem::MaybeUninit;
use core::ops::Deref;
use core::ops::DerefMut;

/*
UINT        _tx_mutex_create(TX_MUTEX *mutex_ptr, CHAR *name_ptr, UINT inherit);
UINT        _tx_mutex_delete(TX_MUTEX *mutex_ptr);
UINT        _tx_mutex_get(TX_MUTEX *mutex_ptr, ULONG wait_option);
UINT        _tx_mutex_info_get(TX_MUTEX *mutex_ptr, CHAR **name, ULONG *count, TX_THREAD **owner,
                    TX_THREAD **first_suspended, ULONG *suspended_count,
                    TX_MUTEX **next_mutex);
UINT        _tx_mutex_performance_info_get(TX_MUTEX *mutex_ptr, ULONG *puts, ULONG *gets,
                    ULONG *suspensions, ULONG *timeouts, ULONG *inversions, ULONG *inheritances);
UINT        _tx_mutex_performance_system_info_get(ULONG *puts, ULONG *gets, ULONG *suspensions, ULONG *timeouts,
                    ULONG *inversions, ULONG *inheritances);
UINT        _tx_mutex_prioritize(TX_MUTEX *mutex_ptr);
UINT        _tx_mutex_put(TX_MUTEX *mutex_ptr);

*/
use crate::tx_checked_call;

use super::WaitOption;
use super::error::TxError;
use defmt::debug;
use defmt::error;
use num_traits::FromPrimitive;
use thiserror_no_std::Error;
use threadx_sys::TX_MUTEX;
use threadx_sys::_tx_mutex_create;
use threadx_sys::_tx_mutex_delete;
use threadx_sys::_tx_mutex_get;
use threadx_sys::_tx_mutex_put;

pub struct Mutex<T> {
    inner : UnsafeCell<T>,
    mutex : UnsafeCell<MaybeUninit<TX_MUTEX>>,
}
//unsafe impl<T: Send> Send for Mutex<T> {}
unsafe impl<T: Send> Sync for Mutex<T> {}

pub struct MutexGuard<'a,T> {
    mutex : &'a Mutex<T>,
}

impl<T> Deref for MutexGuard<'_, T> {
    type Target = T;

    fn deref(&self) -> &T {
        unsafe { &*self.mutex.inner.get() }
    }
}

impl<T> DerefMut for MutexGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *self.mutex.inner.get() }
    }
}

impl<T> Drop for MutexGuard<'_, T> {
    fn drop(&mut self) {
        let mutex_ptr = self.mutex.mutex.get();
        if let Some(mutex_ptr) = unsafe{mutex_ptr.as_mut()} {
            if tx_checked_call!(_tx_mutex_put(mutex_ptr.as_mut_ptr())).is_err() {
                error!("MutexGuard::drop failed to put mutex");
            }
        } else {
            panic!("Mutex ptr is null");
        }
    }
}

#[derive(Error,Debug)]
pub enum MutexError {
    MutexError(TxError),
    PoisonError,
}

impl <T>Mutex<T> {
    pub const fn new(inner: T) -> Self {
        Self {
            inner : UnsafeCell::new(inner),
            mutex : UnsafeCell::new(MaybeUninit::<TX_MUTEX>::uninit()),
        }
    }

    pub fn initialize(&'static mut self, name: &CStr, inherit: bool) -> Result<(),TxError> {

        
        unsafe {
            if !self.mutex.get_mut().as_ptr().as_ref().unwrap().tx_mutex_name.is_null() {
                panic!("Mutex is already initialized");
            }
        }
        let mutex_ptr = self.mutex.get_mut().as_mut_ptr();

        tx_checked_call!(_tx_mutex_create(
            mutex_ptr,
            name.as_ptr() as *mut i8,
            inherit as u32
        ))
    }

    pub fn lock(&'static self, wait_option: WaitOption) -> Result<MutexGuard<T>,MutexError> {
        let mut mutex_ptr = self.mutex.get();
        
        if let Some(mutex_ptr) = unsafe{mutex_ptr.as_mut()} {
            let mutex_ptr = mutex_ptr.as_mut_ptr();
            unsafe {
                if (*mutex_ptr).tx_mutex_name.is_null() {
                    return Err(MutexError::PoisonError);
                }
            }
            let result = tx_checked_call!(_tx_mutex_get(mutex_ptr,wait_option as u32));
            match result {
                Ok(_) => Ok(MutexGuard{mutex:self}),
                Err(e) => Err(MutexError::MutexError(e))
            }
        } else {
            return Err(MutexError::PoisonError);
        }
    }
}

impl <T>Drop for Mutex<T> {
    fn drop(&mut self) {
        let mutex_ptr = self.mutex.get_mut().as_mut_ptr();
        if mutex_ptr.is_null() {
            panic!("Mutex ptr is null");
        }
        let _ = tx_checked_call!(_tx_mutex_delete(mutex_ptr));
    }
}




// unsafe impl Sync for MutexHandle {}
// unsafe impl Send for MutexHandle  {}
// pub struct MutexHandle(&mut TX_MUTEX);

// impl MutexHandle {
//     fn new(mutex_ptr: *mut TX_MUTEX) -> Self {
//         assert!(!mutex_ptr.is_null(),"MutexHandle::new mutex_ptr is null");
//         MutexHandle(mutex_ptr)
//     }

//     pub fn delete(self) -> Result<(),TxError> {
//         tx_checked_call!(_tx_mutex_delete(self.0))
//     }

//     pub fn get(&self, wait_option: WaitOption) -> Result<(),TxError> {
//         tx_checked_call!(_tx_mutex_get(self.0,wait_option as u32))
//     }

//     pub fn put(&self) -> Result<(),TxError> {
//         tx_checked_call!(_tx_mutex_put(self.0))
//     }
// }
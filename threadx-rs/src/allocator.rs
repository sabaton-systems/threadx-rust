use core::{alloc::{GlobalAlloc, Layout}, ffi::{c_void, CStr}, mem::MaybeUninit};
use crate::{tx_checked_call, tx_str};
use threadx_sys::{_tx_byte_allocate, TX_WAIT_FOREVER, ULONG, _tx_byte_release, TX_BYTE_POOL, _tx_byte_pool_create};
use defmt::{error, println};
use crate::error::TxError;
use num_traits::FromPrimitive;

/// ThreadX allocator for Rust. Instantiate this struct and use it as the global allocator.
/// 
///  `
///  #[global_allocator]
///  static mut GLOBAL: ThreadXAllocator = ThreadXAllocator::new();
///  unsafe{GLOBAL.initialize(bp1_mem).unwrap()};
///  `
pub struct ThreadXAllocator(MaybeUninit<TX_BYTE_POOL>);
unsafe impl Sync for ThreadXAllocator {}

impl ThreadXAllocator {
    pub const fn new() -> Self {
        ThreadXAllocator(MaybeUninit::<TX_BYTE_POOL>::uninit())
    }

    pub fn initialize(
        &'static mut self,
        pool_memory: &mut [u8],
    ) -> Result<(), TxError> {

        let pool_ptr = self.0.as_mut_ptr();
        if pool_ptr.is_null() {
            panic!("Pool ptr is null");
        }
        unsafe {
            if !(*pool_ptr).tx_byte_pool_start.is_null() {
                panic!("Pool is already initialized");
            }
        }
        tx_checked_call!(_tx_byte_pool_create(
            pool_ptr,
            tx_str!("global").as_ptr() as *mut i8,
            pool_memory.as_mut_ptr() as *mut core::ffi::c_void,
            pool_memory.len() as ULONG
        ))
    }
}

unsafe impl GlobalAlloc for ThreadXAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let mut ptr: *mut c_void = core::ptr::null_mut() as *mut c_void;
        tx_checked_call!(_tx_byte_allocate(
            self.0.as_ptr() as *mut _, 
            &mut ptr,
            layout.size() as ULONG,
            TX_WAIT_FOREVER
        ))
        .map(|_| ptr as *mut u8 ).unwrap()
    }

    unsafe fn dealloc(&self, ptr: *mut u8, _layout: Layout) {
        tx_checked_call!(_tx_byte_release(ptr as *mut c_void)).unwrap()
    }
}
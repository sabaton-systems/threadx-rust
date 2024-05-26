#![no_main]
#![no_std]


use core::cell::RefCell;
use core::iter::Once;

use board::{BoardStm32f103c8BluePill, LowLevelInit};

use defmt::{debug, println};
use stm32f1xx_hal::pac::can1::rx;
use threadx_rs::event_flags::{EventFlagsGroup};
use threadx_rs::timer::Timer;
use threadx_rs::{tx_checked_call, WaitOption};
use threadx_rs::allocator::ThreadXAllocator;
use threadx_rs::mutex::Mutex;
use threadx_rs::pool::{BlockPool, BytePool, BytePoolHandle};

use threadx_rs::queue::Queue;
use threadx_rs::semaphore::{Semaphore, SemaphoreOwner, SemaphoreUser};
use threadx_rs::thread::{Thread, sleep};
use threadx_rs::tx_str;

extern crate alloc;
use alloc::boxed::Box;
use threadx_sys::_tx_event_flags_get;



#[cortex_m_rt::entry]
fn main() -> ! {
    defmt::println!("Hello, world!");
    

    let tx = threadx_rs::Builder::new(
        // low level initialization
        |ticks_per_second| {
            BoardStm32f103c8BluePill::low_level_init(ticks_per_second).unwrap();
            static mut HEAP: [u8; 4096*3] = [0u8; 4096*3];
            unsafe { HEAP.as_mut_slice() }
        },
        // Start of Application definition
        |mem_start| {

            defmt::println!("Define application. Memory starts at: {} with length:{}", mem_start.as_ptr(), mem_start.len());
            static mut BP: BytePool = BytePool::new();
         
            let (bp_mem , next)= mem_start.split_at_mut(1024);
            
            let bp = unsafe{BP.initialize(tx_str!("pool1"), bp_mem).unwrap()};
            
            //allocate memory for the two tasks.
            let task1_mem = bp.allocate(256, true).unwrap();
            let task2_mem = bp.allocate(256, true).unwrap();
            let task3_mem = bp.allocate(256, true).unwrap();

            let (global_alloc_mem, next) = next.split_at_mut(1024);
            //let  heap_bytepool : BytePoolHandle = unsafe{BP1.initialize(tx_str!("pool2"), bp1_mem).unwrap()};
            #[global_allocator]
            static mut GLOBAL: ThreadXAllocator = ThreadXAllocator::new();
            unsafe{GLOBAL.initialize(global_alloc_mem).unwrap()};

            // create events flag group
            static mut EVENT_GROUP: EventFlagsGroup = EventFlagsGroup::new();
            unsafe {EVENT_GROUP.initialize(tx_str!("event_group")).unwrap()};

            
            
            // Create timer
            static mut TIMER: Timer = Timer::new();
            unsafe {
                TIMER.initialize(tx_str!("HB"),   |u|{
                    debug!("Timer expired {}",u);
                    EVENT_GROUP.publish(1).unwrap();

                }, 
                42, 
                core::time::Duration::from_secs(5),  // initial timeout is 5 seconds
                core::time::Duration::from_secs(1),  // periodic timeout is 1 second
                true                                 // start the timer immediately
            ).expect("Timer Init failed");
            };

            static mut thread : Thread = Thread::new();
            let thread_func = move || {

                let mut arg : u32 = 0;                
                
                println!("Thread:{}", arg);
                loop {

                    unsafe {
                        let event = EVENT_GROUP.get(1, threadx_rs::event_flags::GetOption::WaitAllAndClear, WaitOption::WaitForever).unwrap();
                        debug!("Thread1: Got Event 1 : {}", event);
                    }

                    
                    //sleep(core::time::Duration::from_millis(100)).unwrap();
                }
            };

            let th_handle = unsafe {
                thread.initialize(tx_str!("thread1"), thread_func, task1_mem, 1, 1, 0, true).unwrap()
            };

            let thread2_fn = move || {
                let arg : u32 = 1;    

                loop {
                    unsafe {
                        let event = EVENT_GROUP.get(1, threadx_rs::event_flags::GetOption::WaitAllAndClear, WaitOption::WaitForever).unwrap();
                        debug!("Thread2: Got Event 1 : {}", event);
                    }
                    //sleep(core::time::Duration::from_millis(100)).unwrap();
                    
                }
            };
            static mut thread2 : Thread = Thread::new();

            let th2_handle = unsafe {
                thread2.initialize(tx_str!("thread1"), thread2_fn, task2_mem, 1, 1, 0, true).unwrap()
            };


            let thread3_fn = move || {
                let arg : u32 = 1;    

                loop {
                    unsafe {
                        let event = EVENT_GROUP.get(1, threadx_rs::event_flags::GetOption::WaitAllAndClear, WaitOption::WaitForever).unwrap();
                        debug!("Thread3: Got Event 1 : {}", event);
                    }

                    
                }
            };

            static mut thread3 : Thread = Thread::new();

            let th3_handle = unsafe {
                thread3.initialize(tx_str!("thread2"), thread3_fn, task3_mem, 1, 1, 0, true).unwrap()
            };


            

        },
    );

    tx.initialize();
    println!("Exit");
    threadx_app::exit()
}






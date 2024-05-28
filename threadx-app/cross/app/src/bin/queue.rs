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


pub enum Event {
    Event,
    Info(u32),
}


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
         
            let (bp_mem , next)= mem_start.split_at_mut(2048);
            
            let bp = unsafe{BP.initialize(tx_str!("pool1"), bp_mem).unwrap()};
            
            //allocate memory for the two tasks.
            let task1_mem = bp.allocate(256, true).unwrap();
            let task2_mem = bp.allocate(256, true).unwrap();
            let queue_mem = bp.allocate(64, true).unwrap();

            static mut QUEUE : Queue<Event> = Queue::new();
            let (sender, receiver) = unsafe{QUEUE.initialize(tx_str!("queue"), queue_mem).unwrap()};


            static mut thread : Thread = Thread::new();
            let thread1_func = move || {

                let mut arg : u32 = 0;                
                
                println!("Thread 1:{}", arg);
                let mut count : u32 = 1;
                loop {
                    let message = Event::Info(count);
                    sender.send(message, WaitOption::WaitForever).unwrap();
                    count += 1;
                    sleep(core::time::Duration::from_millis(1000)).unwrap();
                }
            };

            let th_handle = unsafe {
                thread.initialize(tx_str!("thread1"), thread1_func, task1_mem, 1, 1, 0, true).unwrap()
            };

            let thread2_fn = move || {

                loop {
                    let msg = receiver.receive(WaitOption::WaitForever).unwrap();
                    match msg {
                        Event::Event => {
                            println!("Thread 2: RX Event");
                        },
                        Event::Info(info) => {
                            println!("Thread 2: RX Info:{}", info);
                        }
                    }
                    
                }
            };
            static mut thread2 : Thread = Thread::new();

            let th2_handle = unsafe {
                thread2.initialize(tx_str!("thread1"), thread2_fn, task2_mem, 1, 1, 0, true).unwrap()
            };



        },
    );

    tx.initialize();
    println!("Exit");
    threadx_app::exit()
}






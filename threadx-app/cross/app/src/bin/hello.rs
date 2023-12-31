#![no_main]
#![no_std]

use board::{BoardStm32f103c8BluePill, LowLevelInit};

use defmt::println;
use threadx_rs::WaitOption;
use threadx_rs::allocator::ThreadXAllocator;
use threadx_rs::mutex::Mutex;
use threadx_rs::pool::{BlockPool, BytePool, BytePoolHandle};

use threadx_rs::queue::Queue;
use threadx_rs::semaphore::{Semaphore, SemaphoreOwner, SemaphoreUser};
use threadx_rs::thread::{Thread, sleep};
use threadx_rs::tx_str;

extern crate alloc;
use alloc::boxed::Box;

#[cortex_m_rt::entry]
fn main() -> ! {
    defmt::println!("Hello, world!");

    let tx = threadx_rs::Builder::new(
        |ticks_per_second| {
            BoardStm32f103c8BluePill::low_level_init(ticks_per_second).unwrap();
            static mut HEAP: [u8; 4096*3] = [0u8; 4096*3];
            unsafe { HEAP.as_mut_slice() }
        },
        // Start of Application definition
        |mem_start| {
            defmt::println!("Define application. Memory starts at: {} with length:{}", mem_start.as_ptr(), mem_start.len());
            static mut BP: BytePool = BytePool::new();
            static mut BP1:BytePool = BytePool::new();
            static mut BP2:BytePool = BytePool::new();
         
            let (bp_mem , next)= mem_start.split_at_mut(1024);
            
            let mut bp = unsafe{BP.initialize(tx_str!("pool1"), bp_mem).unwrap()};
            let task_mem = bp.allocate(256, true).unwrap();
            let task2_mem = bp.allocate(256, true).unwrap();

            
            let (bp1_mem, next) = next.split_at_mut(1024);
            //let  heap_bytepool : BytePoolHandle = unsafe{BP1.initialize(tx_str!("pool2"), bp1_mem).unwrap()};
            #[global_allocator]
            static mut GLOBAL: ThreadXAllocator = ThreadXAllocator::new();
            unsafe{GLOBAL.initialize(bp1_mem).unwrap()};

            {
                let dyn_a = Box::new([10u8;10]);
            }


            let (bp2_mem, next) = next.split_at_mut(1024);
            let mut bp2 = unsafe{BP2.initialize(tx_str!("pool3"), bp2_mem).unwrap()};
            let mem = bp2.allocate(512, true).unwrap();

            static mut BLOCK_POOL: BlockPool = BlockPool::new();

            let mut block_pool_handle = unsafe {
                BLOCK_POOL.initialize(tx_str!("block_pool"), 16, next).unwrap()
            };

            let block1 = block_pool_handle.allocate(true).unwrap();

            println!("Allocate block 1 with length {}", block1.len());

            static mut thread : Thread = Thread::new();
            
            static mut MUTEX : Mutex<i32> = Mutex::new(0);
            unsafe{MUTEX.initialize(tx_str!("test"), true).unwrap()};

            static mut QUEUE : Queue<u32> = Queue::new();
            let (sender, receiver) = unsafe{QUEUE.initialize(tx_str!("queue"), mem).unwrap()};
            
            static mut SEM : Semaphore = Semaphore::new();
            let sem_owner = unsafe {
                SEM.initialize(tx_str!("sem"), 0).unwrap()
            };

            
            let thread_func = move || {

                let mut arg : u32 = 0;                
                let mut local_counter = 0;
                
                let sem_user = sem_owner.get_semaphore_user();
                
                println!("Thread:{}", arg);
                loop {
                    if sem_user.get(WaitOption::WaitForever).is_ok() {
                        println!("Semaphore acquired");
                    }
                    arg = arg + 1;
                    println!("Thread:{}", arg);
                    {
                        let mut v = unsafe{MUTEX.lock(WaitOption::WaitForever).unwrap()};
                        *v = *v + 1;
                        println!("Value is now:{}", *v);
                    }
                    local_counter = local_counter + 1;
                    sender.send(local_counter , WaitOption::WaitForever).unwrap();
                    sleep(core::time::Duration::from_millis(500)).unwrap();
                }
            };

            

            let th_handle = unsafe {
                thread.initialize(tx_str!("thread1"), thread_func, task_mem, 1, 1, 0, true).unwrap()
            };

            let thread2_fn = move || {
                let arg : u32 = 1;    
                let sem_user = sem_owner.get_semaphore_user();
                loop {
                    sem_user.put().unwrap();
                    println!("Thread:{:#08x}", arg);
                    {
                        let mut v = unsafe{MUTEX.lock(WaitOption::WaitForever).unwrap()};
                        *v = *v - 1;
                        println!("Value is now:{}", *v);
                    }
                    if let Ok(rx) = receiver.receive(WaitOption::NoWait) {
                        println!("Received:{}", rx);
                    } else {
                        println!("No message");
                    }
                    
                    sleep(core::time::Duration::from_millis(200)).unwrap();
                    
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






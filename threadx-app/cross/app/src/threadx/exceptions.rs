use cortex_m_rt::exception;

#[exception]
fn SysTick() {
    unsafe { threadx_rs::tx_timer_interrupt() };
}


#[exception]
fn PendSV() {
    unsafe { threadx_rs::tx_pendsv_handler() };    
}
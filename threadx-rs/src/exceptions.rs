use cortex_m_rt::exception;

#[exception]
fn SysTick() {
    unsafe { threadx_sys::_tx_timer_interrupt() };
}


#[exception]
fn PendSV() {
    unsafe { threadx_sys::__tx_PendSVHandler() };    
}
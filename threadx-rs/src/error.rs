use num_derive::FromPrimitive;
use thiserror_no_std::Error;


pub type TxResult = Result<(), TxError>;

#[repr(u32)]
#[derive(Error,FromPrimitive,Debug)]
pub enum TxError {
    Deleted = threadx_sys::TX_DELETED,
    PoolError = threadx_sys::TX_POOL_ERROR,
    PtrError = threadx_sys::TX_PTR_ERROR,
    WaitError = threadx_sys::TX_WAIT_ERROR,
    SizeError = threadx_sys::TX_SIZE_ERROR,
    GroupError = threadx_sys::TX_GROUP_ERROR,
    NoEvents = threadx_sys::TX_NO_EVENTS,
    OptionError = threadx_sys::TX_OPTION_ERROR,
    QueueError = threadx_sys::TX_QUEUE_ERROR,
    QueueEmpty = threadx_sys::TX_QUEUE_EMPTY,
    QueueFull = threadx_sys::TX_QUEUE_FULL,
    SemaphoreError = threadx_sys::TX_SEMAPHORE_ERROR,
    NoInstance = threadx_sys::TX_NO_INSTANCE,
    ThreadError = threadx_sys::TX_THREAD_ERROR,
    PriorityError = threadx_sys::TX_PRIORITY_ERROR,
    NoMemoryOrStartError = threadx_sys::TX_NO_MEMORY,
    //StartError = threadx_sys::TX_START_ERROR, // threadx has 0x10 defined twice with two different values
    DeleteError = threadx_sys::TX_DELETE_ERROR,
    ResumeError = threadx_sys::TX_RESUME_ERROR,
    CallerError = threadx_sys::TX_CALLER_ERROR,
    SuspendError = threadx_sys::TX_SUSPEND_ERROR,
    TimerError = threadx_sys::TX_TIMER_ERROR,
    TickError = threadx_sys::TX_TICK_ERROR,
    ActivateError = threadx_sys::TX_ACTIVATE_ERROR,
    ThreshError = threadx_sys::TX_THRESH_ERROR,
    SuspendLifted = threadx_sys::TX_SUSPEND_LIFTED,
    WaitAborted = threadx_sys::TX_WAIT_ABORTED,
    WaitAbortError = threadx_sys::TX_WAIT_ABORT_ERROR,
    MutexError = threadx_sys::TX_MUTEX_ERROR,
    NotAvailable = threadx_sys::TX_NOT_AVAILABLE,
    NotOwned = threadx_sys::TX_NOT_OWNED,
    InheritError = threadx_sys::TX_INHERIT_ERROR,
    NotDone = threadx_sys::TX_NOT_DONE,
    CeilingExceeded = threadx_sys::TX_CEILING_EXCEEDED,
    InvalidCeiling = threadx_sys::TX_INVALID_CEILING,
    FeatureNotEnabled = threadx_sys::TX_FEATURE_NOT_ENABLED,
    Unknown = 0xFE,
}
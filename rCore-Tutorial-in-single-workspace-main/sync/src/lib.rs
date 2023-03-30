//! 同步互斥模块
//Sync类型是可以安全的在线程间共享不可变引用，那么可以标记为Sync 
//实现Send的类型可以在线程间安全的传递其所有权
#![no_std]
//#![deny(warnings, missing_docs)]

mod condvar;
mod mutex;
mod semaphore;
mod up;

extern crate alloc;

pub use condvar::Condvar;
pub use mutex::{Mutex, MutexBlocking};
pub use semaphore::Semaphore;
pub use up::{UPIntrFreeCell, UPIntrRefMut};

pub use up::{UPSafeCellRaw, IntrMaskingInfo};
# [cfg(test)]
mod tests{
    use crate::Condvar;
    use crate::MutexBlocking;
    //use crate::mutex::Mutex;
    use crate::Semaphore;
    use crate::{UPSafeCellRaw, IntrMaskingInfo};
    use crate::{UPIntrFreeCell, UPIntrRefMut};
    use rcore_task_manage::ThreadId;

    use riscv::register::sstatus;

    pub struct SyscallContext;


    #[test]
    fn test_condvar() {
        let _a = Condvar::new();
        //(& _a).signal();
        let tid1 = ThreadId::new();
        let tid2 = ThreadId::from_usize(0);
        assert_eq!(tid1, tid2);
        //(& _a).wait_no_sched(tid2);
    }

    #[test]
    fn test_mutex() {
        let _a = MutexBlocking::new();
        let tid1 = ThreadId::new();
        let tid2 = ThreadId::from_usize(0);
        //assert_eq!(tid1, tid2);
        //(& _a).lock(tid2);
    }

    #[test]
    fn test_semaphore() {
        let _a = Semaphore::new(1);
        //(& _a).up();
    }

    #[test]
    fn test_up() {

        let value = 1;
        unsafe{
            let a = UPSafeCellRaw::new(value);
            let b = (& a).get_mut();
            assert_eq!(1,*b);
        }
        
        let mut _a = IntrMaskingInfo::new();
        //(&mut a).exit();
        //(&mut a).enter();
        unsafe{
            let upintr = UPIntrFreeCell::new(value);
            (& upintr).exclusive_access();
        }

    }
}
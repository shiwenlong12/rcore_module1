//! 同步互斥模块
//Sync类型是可以安全的在线程间共享不可变引用，那么可以标记为Sync 
//实现Send的类型可以在线程间安全的传递其所有权
#![no_std]
//#![deny(warnings, missing_docs)]

pub mod condvar;
pub mod mutex;
pub mod semaphore;
pub mod up;

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

    //use riscv::register::sstatus;
    use alloc::sync::Arc;
    use alloc::{alloc::alloc_zeroed, string::String, vec::Vec};
    use core::{ptr::NonNull};
    use rcore_task_manage::ProcId;
    use core::{alloc::Layout, mem::MaybeUninit};



    /// 线程
    pub struct Thread {
        /// 不可变
        pub tid: ThreadId,
        // /// 可变
        //pub context: ForeignContext,
    }

    impl Thread {
        pub fn new() -> Self {
            Self {
                tid: ThreadId::new(),
                //context: ForeignContext { context, satp },
            }
        }
    }

    /// 进程。
    pub struct Process {
        /// 不可变
        pub pid: ProcId,
        // /// 可变
        // pub address_space: AddressSpace<Sv39, Sv39Manager>,
        // /// 文件描述符表
        // pub fd_table: Vec<Option<Mutex<FileHandle>>>,
        // /// 信号模块
        //pub signal: Box<dyn Signal>,
        // /// 分配的锁以及信号量
        // pub semaphore_list: Vec<Option<Arc<Semaphore>>>,
        // pub mutex_list: Vec<Option<Arc<dyn MutexTrait>>>,
        // pub condvar_list: Vec<Option<Arc<Condvar>>>,
    }


    // impl syscall::Thread for SyscallContext {
    //     fn thread_create(&self, _caller: Caller, entry: usize, arg: usize) -> isize {
    //         // 主要的问题是用户栈怎么分配，这里不增加其他的数据结构，直接从规定的栈顶的位置从下搜索是否被映射
    //         let current_proc = unsafe { PROCESSOR.get_current_proc().unwrap() };
    //         // 第一个线程的用户栈栈底
    //         let mut vpn = VPN::<Sv39>::new((1 << 26) - 2);
    //         let addrspace = &mut current_proc.address_space;
    //         loop {
    //             let idx = vpn.index_in(Sv39::MAX_LEVEL);
    //             if !addrspace.root()[idx].is_valid() {
    //                 break;
    //             }
    //             vpn = VPN::<Sv39>::new(vpn.val() - 3);
    //         }
    //         let stack = unsafe {
    //             alloc_zeroed(Layout::from_size_align_unchecked(
    //                 2 << Sv39::PAGE_BITS,
    //                 1 << Sv39::PAGE_BITS,
    //             ))
    //         };
    //         addrspace.map_extern(
    //             vpn..vpn + 2,
    //             PPN::new(stack as usize >> Sv39::PAGE_BITS),
    //             VmFlags::build_from_str("U_WRV"),
    //         );
    //         let satp = (8 << 60) | addrspace.root_ppn().val();
    //         let mut context = kernel_context::LocalContext::user(entry);
    //         *context.sp_mut() = (vpn + 2).base().val();
    //         *context.a_mut(0) = arg;
    //         let thread = Thread::new(satp, context);
    //         let tid = thread.tid;
    //         unsafe {
    //             PROCESSOR.add(tid, thread, current_proc.pid);
    //         }
    //         tid.get_usize() as _
    //     }

    //     fn gettid(&self, _caller: Caller) -> isize {
    //         let current_thread = unsafe { PROCESSOR.current().unwrap() };
    //         current_thread.tid.get_usize() as _
    //     }

    //     fn waittid(&self, _caller: Caller, tid: usize) -> isize {
    //         let current_thread = unsafe { PROCESSOR.current().unwrap() };
    //         // 线程不能自己等待自己
    //         if tid == current_thread.tid.get_usize() {
    //             return -1;
    //         }
    //         // 在当前的进程中查找 tid 对应的线程
    //         if let Some(exit_code) = unsafe { PROCESSOR.waittid(ThreadId::from_usize(tid)) } {
    //             exit_code
    //         } else {
    //             -1
    //         }
    //     }
    // }

    // impl SyncMutex for SyscallContext {
    //     fn semaphore_create(&self, _caller: Caller, res_count: usize) -> isize {
    //         let current_proc = unsafe { PROCESSOR.get_current_proc().unwrap() };
    //         let id = if let Some(id) = current_proc
    //             .semaphore_list
    //             .iter()
    //             .enumerate()
    //             .find(|(_, item)| item.is_none())
    //             .map(|(id, _)| id)
    //         {
    //             current_proc.semaphore_list[id] = Some(Arc::new(Semaphore::new(res_count)));
    //             id
    //         } else {
    //             current_proc
    //                 .semaphore_list
    //                 .push(Some(Arc::new(Semaphore::new(res_count))));
    //             current_proc.semaphore_list.len() - 1
    //         };
    //         id as isize
    //     }

    //     fn semaphore_up(&self, _caller: Caller, sem_id: usize) -> isize {
    //         let current_proc = unsafe { PROCESSOR.get_current_proc().unwrap() };
    //         let sem = Arc::clone(current_proc.semaphore_list[sem_id].as_ref().unwrap());
    //         if let Some(tid) = sem.up() {
    //             // 释放锁之后，唤醒某个阻塞在此信号量上的线程
    //             unsafe {
    //                 PROCESSOR.re_enque(tid);
    //             }
    //         }
    //         0
    //     }

    //     fn semaphore_down(&self, _caller: Caller, sem_id: usize) -> isize {
    //         let current = unsafe { PROCESSOR.current().unwrap() };
    //         let tid = current.tid;
    //         let current_proc = unsafe { PROCESSOR.get_current_proc().unwrap() };
    //         let sem = Arc::clone(current_proc.semaphore_list[sem_id].as_ref().unwrap());
    //         if !sem.down(tid) {
    //             -1
    //         } else {
    //             0
    //         }
    //     }
    //     // 虽然提供了标志位来创建不同的锁，但是目前是不支持自旋锁的
    //     fn mutex_create(&self, _caller: Caller, blocking: bool) -> isize {
    //         let new_mutex: Option<Arc<dyn MutexTrait>> = if blocking {
    //             Some(Arc::new(MutexBlocking::new()))
    //         } else {
    //             // 本来应该是自旋锁，但是目前还不支持，所以先返回 None
    //             None
    //         };
    //         let current_proc = unsafe { PROCESSOR.get_current_proc().unwrap() };
    //         if let Some(id) = current_proc
    //             .mutex_list
    //             .iter()
    //             .enumerate()
    //             .find(|(_, item)| item.is_none())
    //             .map(|(id, _)| id)
    //         {
    //             current_proc.mutex_list[id] = new_mutex;
    //             id as isize
    //         } else {
    //             current_proc.mutex_list.push(new_mutex);
    //             current_proc.mutex_list.len() as isize - 1
    //         }
    //     }

    //     fn mutex_unlock(&self, _caller: Caller, mutex_id: usize) -> isize {
    //         let current_proc = unsafe { PROCESSOR.get_current_proc().unwrap() };
    //         let mutex = Arc::clone(current_proc.mutex_list[mutex_id].as_ref().unwrap());
    //         if let Some(tid) = mutex.unlock() {
    //             // 释放锁之后，唤醒某个阻塞在此信号量上的线程
    //             unsafe {
    //                 PROCESSOR.re_enque(tid);
    //             }
    //         }
    //         0
    //     }

    //     fn mutex_lock(&self, _caller: Caller, mutex_id: usize) -> isize {
    //         let current = unsafe { PROCESSOR.current().unwrap() };
    //         let tid = current.tid;
    //         let current_proc = unsafe { PROCESSOR.get_current_proc().unwrap() };
    //         let mutex = Arc::clone(current_proc.mutex_list[mutex_id].as_ref().unwrap());
    //         if !mutex.lock(tid) {
    //             -1
    //         } else {
    //             0
    //         }
    //     }

    //     fn condvar_create(&self, _caller: Caller, _arg: usize) -> isize {
    //         let current_proc = unsafe { PROCESSOR.get_current_proc().unwrap() };
    //         let id = if let Some(id) = current_proc
    //             .condvar_list
    //             .iter()
    //             .enumerate()
    //             .find(|(_, item)| item.is_none())
    //             .map(|(id, _)| id)
    //         {
    //             current_proc.condvar_list[id] = Some(Arc::new(Condvar::new()));
    //             id
    //         } else {
    //             current_proc
    //                 .condvar_list
    //                 .push(Some(Arc::new(Condvar::new())));
    //             current_proc.condvar_list.len() - 1
    //         };
    //         id as isize
    //     }

    //     fn condvar_signal(&self, _caller: Caller, condvar_id: usize) -> isize {
    //         let current_proc = unsafe { PROCESSOR.get_current_proc().unwrap() };
    //         let condvar = Arc::clone(current_proc.condvar_list[condvar_id].as_ref().unwrap());
    //         if let Some(tid) = condvar.signal() {
    //             // 释放锁之后，唤醒某个阻塞在此信号量上的线程
    //             unsafe {
    //                 PROCESSOR.re_enque(tid);
    //             }
    //         }
    //         0
    //     }

    //     fn condvar_wait(&self, _caller: Caller, condvar_id: usize, mutex_id: usize) -> isize {
    //         let current = unsafe { PROCESSOR.current().unwrap() };
    //         let tid = current.tid;
    //         let current_proc = unsafe { PROCESSOR.get_current_proc().unwrap() };
    //         let condvar = Arc::clone(current_proc.condvar_list[condvar_id].as_ref().unwrap());
    //         let mutex = Arc::clone(current_proc.mutex_list[mutex_id].as_ref().unwrap());
    //         let (flag, waking_tid) = condvar.wait_with_mutex(tid, mutex);
    //         if let Some(waking_tid) = waking_tid {
    //             unsafe {
    //                 PROCESSOR.re_enque(waking_tid);
    //             }
    //         }
    //         if !flag {
    //             -1
    //         } else {
    //             0
    //         }
    //     }
    // }


    #[test]
    fn test_condvar() {
        let condver1 = Condvar::new();

        //(& _a).signal();
        let tid1 = ThreadId::new();
        let tid2 = ThreadId::from_usize(0);

        //wait_queue.push_back(tid2);
        //assert_eq!(tid1, tid2);
        //condver1.inner.exclusive_session(tid2);
        (& condver1).wait_no_sched(tid2);
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

    use riscv::register::sstatus;
    use crate::up::INTR_MASKING_INFO;
    #[test]
    fn test_up() {

        let value = 1;
        unsafe{
            let a = UPSafeCellRaw::new(value);
            let b = (& a).get_mut();
            assert_eq!(1,*b);
        }
        
        let mut _a = IntrMaskingInfo::new();
        let a =INTR_MASKING_INFO.get_mut();
        unsafe{
            //let sie = sstatus::read();
            // let sie = sstatus::Sstatus {
            //     bits: 0x100,
            // };
        }
        // unsafe {
        //     sstatus::clear_sie();
        // }
        
        //(&mut _a).exit();
        (&mut _a).enter();
        unsafe{
            let upintr = UPIntrFreeCell::new(value);
            //(& upintr).exclusive_access();
        }

    }
}
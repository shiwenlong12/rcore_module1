//! 任务管理 lib

#![no_std]
#![feature(doc_cfg)]
//#![deny(warnings, missing_docs)]

extern crate alloc;

mod id;
mod manager;
mod scheduler;

pub use id::*;
pub use manager::Manage;
pub use scheduler::Schedule;

#[cfg(feature = "proc")]
mod proc_manage;
#[cfg(feature = "proc")]
mod proc_rel;
#[cfg(feature = "proc")]
pub use proc_manage::PManager;
#[cfg(feature = "proc")]
pub use proc_rel::ProcRel;

#[cfg(feature = "thread")]
mod proc_thread_rel;
#[cfg(feature = "thread")]
mod thread_manager;
#[cfg(feature = "thread")]
pub use proc_thread_rel::ProcThreadRel;
#[cfg(feature = "thread")]
pub use thread_manager::PThreadManager;


// mod proc_manage;
// mod proc_rel;
// pub use proc_manage::PManager;
// pub use proc_rel::ProcRel;
// mod proc_thread_rel;
// mod thread_manager;
// pub use proc_thread_rel::ProcThreadRel;
// pub use thread_manager::PThreadManager;
# [cfg(test)]
mod tests{


    use crate::id::ProcId;
    use crate::id::ThreadId;
    use crate::proc_manage::PManager;
    use crate::thread_manager::PThreadManager;
    use crate::proc_rel::ProcRel;
    use crate::proc_thread_rel::ProcThreadRel;

    use alloc::collections::BTreeMap;
    use alloc::collections::VecDeque;
    use crate::Manage;
    use crate::Schedule;
    //use core::marker::PhantomData;
    

    #[test]
    fn test_id() {
        //测试进程ID
        let id1 = ProcId::new();
        let _id2 = ProcId::from_usize(0);
        let _id3 = ProcId::get_usize(&id1);
        assert_eq!(id1, _id2);
        assert_eq!(0, _id3);
        let id4 = ProcId::from_usize(5);
        let id5 = ProcId::get_usize(&id4);
        assert_eq!(5, id5);
        //测试线程ID
        let tid1 = ThreadId::new();
        let _tid2 = ThreadId::from_usize(0);
        let _tid3 = ThreadId::get_usize(&tid1);
        assert_eq!(tid1, _tid2);
        assert_eq!(0, _tid3);
    }


    #[test]
    fn test_proc_manage(){
        /// 进程。
        #[derive(PartialEq)]
        #[derive(Debug)]
        pub struct Process {
            /// 不可变
            pub pid: ProcId,
            // 可变
            //pub context: ForeignContext,
            //pub address_space: AddressSpace<Sv39, Sv39Manager>,
        }

        impl Process {
            pub fn new() -> Self {
                Self { pid: ProcId::new() }
            }
        }

        pub struct ProcManager {
            tasks: BTreeMap<ProcId, Process>,
            ready_queue: VecDeque<ProcId>,
        }
        
        impl ProcManager {
            /// 新建任务管理器
            pub fn new() -> Self {
                Self {
                    tasks: BTreeMap::new(),
                    ready_queue: VecDeque::new(),
                }
            }
        }
        
        impl Manage<Process, ProcId> for ProcManager {
            /// 插入一个新任务
            #[inline]
            fn insert(&mut self, id: ProcId, task: Process) {
                self.tasks.insert(id, task);
            }
            /// 删除任务实体
            #[inline]
            fn delete(&mut self, id: ProcId) {
                self.tasks.remove(&id);
            }
            /// 根据 id 获取对应的任务
            #[inline]
            fn get_mut(&mut self, id: ProcId) -> Option<&mut Process> {
                self.tasks.get_mut(&id)
            }
        }
        
        impl Schedule<ProcId> for ProcManager {
            /// 添加 id 进入调度队列
            fn add(&mut self, id: ProcId) {
                self.ready_queue.push_back(id);
            }
            /// 从调度队列中取出 id
            fn fetch(&mut self) -> Option<ProcId> {
                self.ready_queue.pop_front()
            }
        }
        let procmanager = ProcManager::new();
        //新建 PManager
        let mut pmanager = PManager::<Process, ProcManager>::new();
        
        //设置manager
        PManager::set_manager(&mut pmanager, procmanager);

        //添加进程
        let parent =  ProcId::new();
        let id = ProcId::from_usize(3);
        let id1 = ProcId::from_usize(1);
        let task = Process::new();
        PManager::add(&mut pmanager, id, task, parent);
        //PManager::current(&mut pmanager);
        //获取指定进程
        PManager::get_task(&mut pmanager, id);
        let mut task1 = Process::new();
        task1.pid = id1;
        assert_eq!(Some(&mut task1),PManager::get_task(&mut pmanager, id));

        
        assert_eq!(Some(&mut task1), PManager::<Process, ProcManager>::find_next(&mut pmanager));
    }


    #[test]
    fn test_proc_thread_manage(){
        /// 进程。
        #[derive(PartialEq)]
        #[derive(Debug)]
        pub struct Process {
            /// 不可变
            pub pid: ProcId,
            // 可变
            //pub context: ForeignContext,
            //pub address_space: AddressSpace<Sv39, Sv39Manager>,
        }

        impl Process {
            // pub fn new() -> Self {
            //     Self { pid: ProcId::new() }
            // }
        }

        pub struct ProcManager {
            tasks: BTreeMap<ProcId, Process>,
            ready_queue: VecDeque<ProcId>,
        }
        
        impl ProcManager {
            /// 新建任务管理器
            pub fn new() -> Self {
                Self {
                    tasks: BTreeMap::new(),
                    ready_queue: VecDeque::new(),
                }
            }
        }
        
        impl Manage<Process, ProcId> for ProcManager {
            /// 插入一个新任务
            #[inline]
            fn insert(&mut self, id: ProcId, task: Process) {
                self.tasks.insert(id, task);
            }
            /// 删除任务实体
            #[inline]
            fn delete(&mut self, id: ProcId) {
                self.tasks.remove(&id);
            }
            /// 根据 id 获取对应的任务
            #[inline]
            fn get_mut(&mut self, id: ProcId) -> Option<&mut Process> {
                self.tasks.get_mut(&id)
            }
        }
        
        impl Schedule<ProcId> for ProcManager {
            /// 添加 id 进入调度队列
            fn add(&mut self, id: ProcId) {
                self.ready_queue.push_back(id);
            }
            /// 从调度队列中取出 id
            fn fetch(&mut self) -> Option<ProcId> {
                self.ready_queue.pop_front()
            }
        }

        /// 线程。
        #[derive(PartialEq)]
        #[derive(Debug)]
        pub struct Thread {
            /// 不可变
            pub pid: ThreadId,
            // 可变
            //pub context: ForeignContext,
            //pub address_space: AddressSpace<Sv39, Sv39Manager>,
        }

        impl Thread {
            pub fn new() -> Self {
                Self { pid: ThreadId::new() }
            }
        }

        pub struct ProcThreadManager {
            tasks: BTreeMap<ThreadId, Thread>,
            ready_queue: VecDeque<ThreadId>,
        }
        
        impl ProcThreadManager {
            /// 新建任务管理器
            pub fn new() -> Self {
                Self {
                    tasks: BTreeMap::new(),
                    ready_queue: VecDeque::new(),
                }
            }
        }
        
        impl Manage<Thread, ThreadId> for ProcThreadManager {
            /// 插入一个新任务
            #[inline]
            fn insert(&mut self, id: ThreadId, task: Thread) {
                self.tasks.insert(id, task);
            }
            /// 删除任务实体
            #[inline]
            fn delete(&mut self, id: ThreadId) {
                self.tasks.remove(&id);
            }
            /// 根据 id 获取对应的任务
            #[inline]
            fn get_mut(&mut self, id: ThreadId) -> Option<&mut Thread> {
                self.tasks.get_mut(&id)
            }
        }
        
        impl Schedule<ThreadId> for ProcThreadManager {
            /// 添加 id 进入调度队列
            fn add(&mut self, id: ThreadId) {
                self.ready_queue.push_back(id);
            }
            /// 从调度队列中取出 id
            fn fetch(&mut self) -> Option<ThreadId> {
                self.ready_queue.pop_front()
            }
        }

        
        //新建 PManager
        let mut pmanager = PThreadManager::<Process, Thread, ProcThreadManager, ProcManager>::new();
        //设置manager
        let procmanager = ProcManager::new();
        let threadmanager = ProcThreadManager::new();

        PThreadManager::set_manager(&mut pmanager, threadmanager);
        PThreadManager::set_proc_manager(&mut pmanager, procmanager);

        //添加进程
        let parent =  ProcId::new();
        let _id = ProcId::from_usize(3);
        let _id1 = ProcId::from_usize(1);
        let _tid = ThreadId::new();
        let tid1 = ThreadId::from_usize(5);
        let task = Thread::new();
        PThreadManager::add(&mut pmanager, tid1, task, parent);
        
        //PManager::current(&mut pmanager);
        //获取指定进程
        //PThreadManager::get_task(&mut pmanager, id);
        //let mut task1 = Process::new();
        //task1.pid = id1;
        //assert_eq!(Some(&mut task1),PThreadManager::get_task(&mut pmanager, id));

        
        //assert_eq!(Some(&mut task1), PThreadManager::<Process, ProcManager>::find_next(&mut pmanager));
    }




    #[test]
    fn test_proc_rel(){
        let parent_pid = ProcId::new();
        let child_pid = ProcId::from_usize(5);
        let exit_code = 1;
        //创建一个进程时同时创建进程关系
        let mut _procrel = ProcRel::new(parent_pid);
        //测试等待子进程结束的函数
        assert_eq!(None, ProcRel::wait_any_child(&mut _procrel));
        assert_eq!(None, ProcRel::wait_child(&mut _procrel, child_pid));
        //添加子进程
        ProcRel::add_child(&mut _procrel, child_pid);
        assert_eq!(_procrel.children[0], child_pid);
        assert_eq!(Some((ProcId::from_usize(-2 as _), -1)), ProcRel::wait_any_child(&mut _procrel));
        assert_eq!(Some((ProcId::from_usize(-2 as _), -1)), ProcRel::wait_child(&mut _procrel, child_pid));
        //子进程结束，子进程 Id 被移入到 dead_children 队列中
        ProcRel::del_child(&mut _procrel, child_pid,exit_code);
        assert_eq!(_procrel.children, []);
        ProcRel::wait_any_child(&mut _procrel);
        assert_eq!(_procrel.dead_children.pop(), ProcRel::wait_any_child(&mut _procrel));
        assert_eq!(_procrel.dead_children.pop(), None);
        //测试等待子进程结束的函数
        ProcRel::add_child(&mut _procrel, child_pid);
        ProcRel::del_child(&mut _procrel, child_pid,exit_code);
        assert_eq!(Some((child_pid, 1)), ProcRel::wait_child(&mut _procrel, child_pid));
    }

    #[test]
    fn test_proc_thread_rel(){
        let parent_pid = ProcId::new();
        let child_pid = ProcId::from_usize(5);
        let _tid1 = ThreadId::new();
        let thread_pid = ThreadId::from_usize(3);
        let exit_code = 1;
        //创建一个进程时同时创建进程关系
        let mut _procrel = ProcThreadRel::new(parent_pid);
        //测试等待子进程结束的函数
        assert_eq!(None, ProcThreadRel::wait_any_child(&mut _procrel));
        assert_eq!(None, ProcThreadRel::wait_child(&mut _procrel, child_pid));
        //添加子进程
        ProcThreadRel::add_child(&mut _procrel, child_pid);
        assert_eq!(_procrel.children[0], child_pid);
        assert_eq!(Some((ProcId::from_usize(-2 as _), -1)), ProcThreadRel::wait_any_child(&mut _procrel));
        assert_eq!(Some((ProcId::from_usize(-2 as _), -1)), ProcThreadRel::wait_child(&mut _procrel, child_pid));
        //子进程结束，子进程 Id 被移入到 dead_children 队列中
        ProcThreadRel::del_child(&mut _procrel, child_pid,exit_code);
        assert_eq!(_procrel.children, []);
        ProcThreadRel::wait_any_child(&mut _procrel);
        assert_eq!(_procrel.dead_children.pop(), ProcThreadRel::wait_any_child(&mut _procrel));
        assert_eq!(_procrel.dead_children.pop(), None);
        //测试等待子进程结束的函数
        ProcThreadRel::add_child(&mut _procrel, child_pid);
        ProcThreadRel::del_child(&mut _procrel, child_pid,exit_code);
        assert_eq!(Some((child_pid, 1)), ProcThreadRel::wait_child(&mut _procrel, child_pid));
        
        assert_eq!(None, ProcThreadRel::wait_thread(&mut _procrel, thread_pid));
        //添加线程
        ProcThreadRel::add_thread(&mut _procrel, thread_pid);
        assert_eq!(_procrel.threads[0], thread_pid);
        assert_eq!(Some(-2), ProcThreadRel::wait_thread(&mut _procrel, thread_pid));
        //线程结束，线程 Id 被移入到 dead_children 队列中
        ProcThreadRel::del_thread(&mut _procrel, thread_pid, exit_code);
        assert_eq!(_procrel.children, []);
        assert_eq!(Some(1), ProcThreadRel::wait_thread(&mut _procrel, thread_pid));
    }
}
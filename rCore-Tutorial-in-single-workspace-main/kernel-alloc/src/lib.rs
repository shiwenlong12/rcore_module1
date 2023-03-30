//! 内存分配。

#![no_std]
//#![deny(warnings, missing_docs)]

extern crate alloc;

use alloc::alloc::handle_alloc_error;
use core::{
    alloc::{GlobalAlloc, Layout},
    ptr::NonNull,
};
use customizable_buddy::{BuddyAllocator, LinkedListBuddy, UsizeBuddy};

/// 初始化内存分配。
///
/// 参数 `base_address` 表示动态内存区域的起始位置。
#[inline]
pub fn init(base_address: usize) {
    unsafe {
        HEAP.init(
            core::mem::size_of::<usize>().trailing_zeros() as _,
            NonNull::new(base_address as *mut u8).unwrap(),
        )
    };
}

/// 将一个内存块托管到内存分配器。
///
/// # Safety
///
/// `region` 内存块的所有权将转移到分配器，
/// 因此需要调用者确保这个内存块与已经转移到分配器的内存块都不重叠，且未被其他对象引用。
/// 并且这个内存块必须位于初始化时传入的起始位置之后。
#[inline]
pub unsafe fn transfer(region: &'static mut [u8]) {
    let ptr = NonNull::new(region.as_mut_ptr()).unwrap();
    HEAP.transfer(ptr, region.len());
}

/// 堆分配器。/伙伴分配器
///
/// 最大容量：6 + 21 + 3 = 30 -> 1 GiB。
/// 不考虑并发使用，因此没有加锁。
static mut HEAP: BuddyAllocator<21, UsizeBuddy, LinkedListBuddy> = BuddyAllocator::new();

struct Global;

#[global_allocator]
static GLOBAL: Global = Global;

unsafe impl GlobalAlloc for Global {
    #[inline]
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        if let Ok((ptr, _)) = HEAP.allocate_layout::<u8>(layout) {
            ptr.as_ptr()
        } else {
            handle_alloc_error(layout)
        }
    }

    #[inline]
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        HEAP.deallocate_layout(NonNull::new(ptr).unwrap(), layout)
    }
}

#[cfg(test)]
mod tests {
    
    use crate::{init,transfer};
    use crate::Global;
    use crate::Layout;
    use core::alloc::GlobalAlloc;
    // 物理内存容量
    const MEMORY :usize= 9000_0000;

    /// 内核地址信息。
    #[derive(Debug)]
    pub struct KernelLayout {
        text: usize,
        end: usize,
    }

    impl KernelLayout {
        /// 内核起始地址。
        #[inline]
        pub const fn start(&self) -> usize {
            self.text
        }

        /// 内核结尾地址。
        #[inline]
        pub const fn end(&self) -> usize {
            self.end
        }

        /// 内核静态二进制长度。
        #[inline]
        pub const fn len(&self) -> usize {
            self.end - self.text
        }
    }

    #[test]
    fn test_alloc() {  
        let _a = KernelLayout {
            text: 8000_1000,
            end: 8800_0000,
        };
        //测试内核起始地址，结尾地址，静态二进制长度。
        assert_eq!(8000_1000,_a.start());
        assert_eq!(8800_0000,_a.end());
        assert_eq!(799_9000,_a.len());

        // 初始化内存分配。
        // 参数 `base_address` 表示动态内存区域的起始位置。
        init(_a.start());
        assert_eq!(8200_1000,MEMORY - _a.len());
        assert_eq!(8800_0000,_a.end() as _);
        unsafe{
            core::slice::from_raw_parts_mut(_a.end() as *mut u8,MEMORY - _a.len(),);
        }
        
        // 将一个内存块托管到内存分配器。
        // unsafe {
        //     transfer(core::slice::from_raw_parts_mut(
        //         _a.end() as *mut u8,
        //         MEMORY - _a.len(),
        //     ))
        // };

        //实现page_alloc
        extern "Rust" {
            fn __rust_alloc_zeroed(size: usize, align: usize) -> *mut u8;
        }


        #[must_use = "losing the pointer will leak memory"]
        #[inline]
        pub unsafe fn alloc_zeroed(layout: Layout) -> *mut u8 {
            unsafe { __rust_alloc_zeroed(layout.size(), layout.align()) }
        }

        let layout = Layout::new::<u16>();
        
        unsafe {
            let ptr: *mut u8  =  alloc_zeroed(layout);
            assert_eq!(*(ptr as *mut u16), 0);
        }
        
        

        //let a = Alignment::new(1024).unwrap();
        //我们的需求是分配一块连续的、大小至少为 size 字节的虚拟内存，且对齐要求为 align
        // let layout = Layout {
        //     //size 表示要分配的字节数，
        //     size: 512,
        //     //align 则表示分配的虚拟地址的最小对齐要求，即分配的地址要求是 align 的倍数。
        //     //这里的 align 必须是2的幂次。
        //     align: 1024,
        // };
        let global = Global{};
        unsafe{
            let a = Global::alloc(&global,layout);
        }
    }
}

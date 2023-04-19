use kernel_alloc::{init, transfer};
use std::alloc::{Layout};
use core::{
    alloc::{GlobalAlloc},
    ptr::NonNull,
};
use std::alloc::handle_alloc_error;
use customizable_buddy::{BuddyAllocator, LinkedListBuddy, UsizeBuddy};
// 物理内存容量
const MEMORY :usize= 9000_0000;
static mut HEAP: BuddyAllocator<21, UsizeBuddy, LinkedListBuddy> = BuddyAllocator::new();
const PAGE: Layout =
        //size:8192,align:4096
        unsafe { 
            Layout::from_size_align_unchecked(
            2 << Sv39::PAGE_BITS, 
            1 << Sv39::PAGE_BITS) 
        };
    
struct Global1;


static GLOBAL: Global1 = Global1;

unsafe impl GlobalAlloc for Global1 {
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


use page_table::{MmuMeta, Pte, VAddr, VmFlags, PPN, VPN};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub(crate) struct Sv39;

impl MmuMeta for Sv39 {
    const P_ADDR_BITS: usize = 56;
    const PAGE_BITS: usize = 12;
    const LEVEL_BITS: &'static [usize] = &[9; 3];
    const PPN_POS: usize = 10;

    #[inline]
    fn is_leaf(value: usize) -> bool {
        const MASK: usize = 0b1110;
        value & MASK != 0
    }
}

#[test]
fn test_alloc() {  
    //let mut v = Vec::new();
    //v.push(1);
    let layout = Layout::new::<u16>();
    let global = Global1{};
    unsafe{
        let a = Global1::alloc(&global,layout);
    }
}
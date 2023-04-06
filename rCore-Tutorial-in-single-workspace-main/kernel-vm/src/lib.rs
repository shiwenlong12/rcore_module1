//! 内核虚存管理。

#![no_std]
//#![deny(warnings, missing_docs)]

mod space;

pub extern crate page_table;
pub use space::AddressSpace;

use core::ptr::NonNull;
use page_table::{Pte, VmFlags, VmMeta, PPN};

/// 物理页管理。
pub trait PageManager<Meta: VmMeta> {
    /// 新建根页表页。
    fn new_root() -> Self;

    /// 获取根页表。
    fn root_ptr(&self) -> NonNull<Pte<Meta>>;

    /// 获取根页表的物理页号。
    #[inline]
    fn root_ppn(&self) -> PPN<Meta> {
        self.v_to_p(self.root_ptr())
    }

    /// 计算当前地址空间上指向物理页的指针。
    fn p_to_v<T>(&self, ppn: PPN<Meta>) -> NonNull<T>;

    /// 计算当前地址空间上的指针指向的物理页。
    fn v_to_p<T>(&self, ptr: NonNull<T>) -> PPN<Meta>;

    /// 检查是否拥有一个页的所有权。
    fn check_owned(&self, pte: Pte<Meta>) -> bool;

    /// 为地址空间分配 `len` 个物理页。
    fn allocate(&mut self, len: usize, flags: &mut VmFlags<Meta>) -> NonNull<u8>;

    /// 从地址空间释放 `pte` 指示的 `len` 个物理页。
    fn deallocate(&mut self, pte: Pte<Meta>, len: usize) -> usize;

    /// 释放根页表。
    fn drop_root(&mut self);
}


# [cfg(test)]
mod tests{

    use crate::space::mapper;
    use crate::space::AddressSpace;
    use core::{alloc::Layout, ptr::NonNull};
    use crate::{
        page_table::{MmuMeta, Pte, VAddr, VmFlags, PPN, VPN},
        PageManager,
    };

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

    #[repr(transparent)]
    pub struct Sv39Manager(NonNull<Pte<Sv39>>);

    //实现page_alloc
    extern "Rust" {
        fn __rust_alloc_zeroed(size: usize, align: usize) -> *mut u8;
    }


    #[must_use = "losing the pointer will leak memory"]
    #[inline]
    pub unsafe fn alloc_zeroed(layout: Layout) -> *mut u8 {
        unsafe { __rust_alloc_zeroed(layout.size(), layout.align()) }
    }

    impl Sv39Manager {
        const OWNED: VmFlags<Sv39> = unsafe { VmFlags::from_raw(1 << 8) };

        #[inline]
        fn page_alloc<T>(count: usize) -> *mut T {
            unsafe {
                alloc_zeroed(Layout::from_size_align_unchecked(
                    count << Sv39::PAGE_BITS,
                    1 << Sv39::PAGE_BITS,
                ))
            }
            .cast()
        }
    }

    impl PageManager<Sv39> for Sv39Manager {
        #[inline]
        fn new_root() -> Self {
            Self(NonNull::new(Self::page_alloc(1)).unwrap())
        }

        #[inline]
        fn root_ppn(&self) -> PPN<Sv39> {
            PPN::new(self.0.as_ptr() as usize >> Sv39::PAGE_BITS)
        }

        #[inline]
        fn root_ptr(&self) -> NonNull<Pte<Sv39>> {
            self.0
        }

        #[inline]
        fn p_to_v<T>(&self, ppn: PPN<Sv39>) -> NonNull<T> {
            unsafe { NonNull::new_unchecked(VPN::<Sv39>::new(ppn.val()).base().as_mut_ptr()) }
        }

        #[inline]
        fn v_to_p<T>(&self, ptr: NonNull<T>) -> PPN<Sv39> {
            PPN::new(VAddr::<Sv39>::new(ptr.as_ptr() as _).floor().val())
        }

        #[inline]
        fn check_owned(&self, pte: Pte<Sv39>) -> bool {
            pte.flags().contains(Self::OWNED)
        }

        #[inline]
        fn allocate(&mut self, len: usize, flags: &mut VmFlags<Sv39>) -> NonNull<u8> {
            *flags |= Self::OWNED;
            NonNull::new(Self::page_alloc(len)).unwrap()
        }

        fn deallocate(&mut self, _pte: Pte<Sv39>, _len: usize) -> usize {
            todo!()
        }

        fn drop_root(&mut self) {
            todo!()
        }
    } 

    #[test]
    fn test_mapper() {
        let layout = KernelLayout {
            text: 8000_1000,
            rodata: 8000_2000,
            data: 8000_3000,
            sbss: 8000_4000,
            ebss: 8000_5000,
            boot: 8000_6000,
            end: 8000_8000,
        };
        let mut address1 = AddressSpace::<Sv39, Sv39Manager>::new();
        let memory = 1000_0000;
        let s = VAddr::<Sv39>::new(layout.end());
        let e = VAddr::<Sv39>::new(layout.start()+memory);
        let range1 = s.floor()..e.ceil();
        let flag1 = VmFlags::<Sv39>::VALID;
        //mapper::Mapper::new(&mut address1, range1, flag1);
    }

    /// 内核地址信息。
    #[derive(Debug)]
    pub struct KernelLayout {
        text: usize,
        rodata: usize,
        data: usize,
        sbss: usize,
        ebss: usize,
        boot: usize,
        end: usize,
    }

    impl KernelLayout {
        /// 非零初始化，避免 bss。
        pub const INIT: Self = Self {
            text: usize::MAX,
            rodata: usize::MAX,
            data: usize::MAX,
            sbss: usize::MAX,
            ebss: usize::MAX,
            boot: usize::MAX,
            end: usize::MAX,
        };

        /// 定位内核布局。
        #[inline]
        pub fn locate() -> Self {
            extern "C" {
                fn __start();
                fn __rodata();
                fn __data();
                fn __sbss();
                fn __ebss();
                fn __boot();
                fn __end();
            }

            Self {
                text: __start as _,
                rodata: __rodata as _,
                data: __data as _,
                sbss: __sbss as _,
                ebss: __ebss as _,
                boot: __boot as _,
                end: __end as _,
            }
        }

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

    use page_table::PageNumber;
    use page_table::Physical;
    #[test]
    fn test_space() {
        // 创建新地址空间。
        let addressspace = AddressSpace::<Sv39, Sv39Manager>::new();
        // 地址空间根页表的物理页号。
        let ppn = (& addressspace).root_ppn();
        // 地址空间根页表
        let root = (& addressspace).root();
        // 向地址空间增加映射关系。
        let mut address1 = AddressSpace::<Sv39, Sv39Manager>::new();
        let pages = 2;
        let layout = KernelLayout {
            text: 8000_1000,
            rodata: 8000_2000,
            data: 8000_3000,
            sbss: 8000_4000,
            ebss: 8000_5000,
            boot: 8000_6000,
            end: 8000_8000,
        };
        let memory = 1000_0000;
        let s = VAddr::<Sv39>::new(layout.end());
        let e = VAddr::<Sv39>::new(layout.start()+memory);
        let range1 = s.floor()..e.ceil();
        let pbase1: PageNumber<Sv39, Physical>= PPN::new(s.floor().val());
        let flag1 = VmFlags::<Sv39>::VALID;
        let map1 = (&mut address1).map_extern(range1, pbase1, flag1);
        
        // 分配新的物理页，拷贝数据并建立映射。

        // 检查 `flags` 的属性要求，然后将地址空间中的一个虚地址翻译成当前地址空间中的指针。

        // 遍历地址空间，将其中的地址映射添加进自己的地址空间中，重新分配物理页并拷贝所有数据及代码


    }




}


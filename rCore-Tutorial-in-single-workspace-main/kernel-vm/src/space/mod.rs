pub mod mapper;
mod visitor;

extern crate alloc;

use crate::PageManager;
use alloc::vec::Vec;
use core::{fmt, ops::Range, ptr::NonNull};
use mapper::Mapper;
use page_table::{PageTable, PageTableFormatter, Pos, VAddr, VmFlags, VmMeta, PPN, VPN};
use visitor::Visitor;

/// 地址空间。
pub struct AddressSpace<Meta: VmMeta, M: PageManager<Meta>> {
    /// 虚拟地址块
    pub areas: Vec<Range<VPN<Meta>>>,
    page_manager: M,
}

impl<Meta: VmMeta, M: PageManager<Meta>> AddressSpace<Meta, M> {
    /// 创建新地址空间。
    #[inline]
    pub fn new() -> Self {
        Self {
            areas: Vec::new(),
            page_manager: M::new_root(),
        }
    }

    /// 地址空间根页表的物理页号。
    #[inline]
    pub fn root_ppn(&self) -> PPN<Meta> {
        self.page_manager.root_ppn()
    }

    /// 地址空间根页表
    #[inline]
    pub fn root(&self) -> PageTable<Meta> {
        unsafe { PageTable::from_root(self.page_manager.root_ptr()) }
    }

    /// 向地址空间增加映射关系。
    pub fn map_extern(&mut self, range: Range<VPN<Meta>>, pbase: PPN<Meta>, flags: VmFlags<Meta>) {
        self.areas.push(range.start..range.end);
        let count = range.end.val() - range.start.val();
        let mut root = self.root();
        let mut mapper = Mapper::new(self, pbase..pbase + count, flags);
        root.walk_mut(Pos::new(range.start, 0), &mut mapper);
        if !mapper.ans() {
            // 映射失败，需要回滚吗？
            todo!()
        }
    }

    /// 分配新的物理页，拷贝数据并建立映射。
    pub fn map(
        &mut self,
        range: Range<VPN<Meta>>,
        data: &[u8],
        offset: usize,
        mut flags: VmFlags<Meta>,
    ) {
        let count = range.end.val() - range.start.val();
        let size = count << Meta::PAGE_BITS;
        assert!(size >= data.len() + offset);
        let page = self.page_manager.allocate(count, &mut flags);
        unsafe {
            use core::slice::from_raw_parts_mut as slice;
            let mut ptr = page.as_ptr();
            slice(ptr, offset).fill(0);
            ptr = ptr.add(offset);
            slice(ptr, data.len()).copy_from_slice(data);
            ptr = ptr.add(data.len());
            slice(ptr, page.as_ptr().add(size).offset_from(ptr) as _).fill(0);
        }
        self.map_extern(range, self.page_manager.v_to_p(page), flags)
    }

    /// 检查 `flags` 的属性要求，然后将地址空间中的一个虚地址翻译成当前地址空间中的指针。
    pub fn translate<T>(&self, addr: VAddr<Meta>, flags: VmFlags<Meta>) -> Option<NonNull<T>> {
        let mut visitor = Visitor::new(self);
        self.root().walk(Pos::new(addr.floor(), 0), &mut visitor);
        visitor
            .ans()
            .filter(|pte| pte.flags().contains(flags))
            .map(|pte| unsafe {
                NonNull::new_unchecked(
                    self.page_manager
                        .p_to_v::<u8>(pte.ppn())
                        .as_ptr()
                        .add(addr.offset())
                        .cast(),
                )
            })
    }

    /// 遍历地址空间，将其中的地址映射添加进自己的地址空间中，重新分配物理页并拷贝所有数据及代码
    pub fn cloneself(&self, new_addrspace: &mut AddressSpace<Meta, M>) {
        let root = self.root();
        let areas = &self.areas;
        for (_, range) in areas.iter().enumerate() {
            let mut visitor = Visitor::new(self);
            // 虚拟地址块的首地址的 vpn
            let vpn = range.start;
            // 利用 visitor 访问页表，并获取这个虚拟地址块的页属性
            root.walk(Pos::new(vpn, 0), &mut visitor);
            // 利用 visitor 获取这个虚拟地址块的页属性，以及起始地址
            let (mut flags, mut data_ptr) = visitor
                .ans()
                .filter(|pte| pte.is_valid())
                .map(|pte| {
                    (pte.flags(), unsafe {
                        NonNull::new_unchecked(self.page_manager.p_to_v::<u8>(pte.ppn()).as_ptr())
                    })
                })
                .unwrap();
            let vpn_range = range.start..range.end;
            // 虚拟地址块中页数量
            let count = range.end.val() - range.start.val();
            let size = count << Meta::PAGE_BITS;
            // 分配 count 个 flags 属性的物理页面
            let paddr = new_addrspace.page_manager.allocate(count, &mut flags);
            let ppn = new_addrspace.page_manager.v_to_p(paddr);
            unsafe {
                use core::slice::from_raw_parts_mut as slice;
                let data = slice(data_ptr.as_mut(), size);
                let ptr = paddr.as_ptr();
                slice(ptr, size).copy_from_slice(data);
            }
            new_addrspace.map_extern(vpn_range, ppn, flags);
        }
    }
}

impl<Meta: VmMeta, P: PageManager<Meta>> fmt::Debug for AddressSpace<Meta, P> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "root: {:#x}", self.root_ppn().val())?;
        write!(
            f,
            "{:?}",
            PageTableFormatter {
                pt: self.root(),
                f: |ppn| self.page_manager.p_to_v(ppn)
            }
        )
    }
}


# [cfg(test)]
mod tests{

    use crate::space::mapper::Mapper;
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
        //PPN::new(s.floor().val());
        let e = VAddr::<Sv39>::new(layout.start()+memory);
        //let range1 = s.floor()..e.ceil();
        let range1 = PPN::new(s.floor().val())..PPN::new(e.floor().val());
        let flag1 = VmFlags::<Sv39>::VALID;
        let mapper1 = Mapper::new(&mut address1, range1, flag1);
        //
        assert_eq!(false, mapper1.ans());
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
        (&mut address1).map_extern(range1, pbase1, flag1);
        
        // 分配新的物理页，拷贝数据并建立映射。
        //(&mut address1).map(range1, &[5],1, flag1);
        // 检查 `flags` 的属性要求，然后将地址空间中的一个虚地址翻译成当前地址空间中的指针。
        (& addressspace).translate::<Sv39>(s, flag1);
        // 遍历地址空间，将其中的地址映射添加进自己的地址空间中，重新分配物理页并拷贝所有数据及代码
        (& addressspace).cloneself(&mut address1);

    }




}

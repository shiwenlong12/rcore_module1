//!An easy file system isolated from the kernel
#![no_std]
// #![deny(missing_docs)]
extern crate alloc;
pub mod bitmap;
mod block_cache;
mod block_dev;
mod efs;
mod file;
mod layout;
mod vfs;
/// Use a block size of 512 bytes
pub const BLOCK_SZ: usize = 512;
use bitmap::Bitmap;
use block_cache::{block_cache_sync_all, get_block_cache};
pub use block_dev::BlockDevice;
pub use efs::EasyFileSystem;
pub use file::*;
use layout::*;
pub use vfs::Inode;


# [cfg(test)]
mod tests{


    use crate::Bitmap;
    //use crate::BlockDevice;
    use crate::file::{UserBuffer};

    use alloc::{
        alloc::{alloc_zeroed, dealloc},
        sync::Arc,
    };
    use core::{alloc::Layout, ptr::NonNull};
    use crate::BlockDevice;
    use spin::{Lazy, Mutex};
    use virtio_drivers::{Hal, VirtIOBlk, VirtIOHeader};
    use core::mem::MaybeUninit;

    use page_table::{MmuMeta, Pte, VAddr, VmFlags, PPN, VPN};

    use alloc::vec::Vec;
    use core::ops::Range;
    use page_table::VmMeta;
    use page_table::Pos;
    use page_table::PageTable;

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
    // extern "Rust" {
    //     fn __rust_alloc_zeroed(size: usize, align: usize) -> *mut u8;
    // }


    // #[must_use = "losing the pointer will leak memory"]
    // #[inline]
    // pub unsafe fn alloc_zeroed(layout: Layout) -> *mut u8 {
    //     unsafe { __rust_alloc_zeroed(layout.size(), layout.align()) }
    // }

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

    pub struct Visitor<'a, Meta: VmMeta, M: PageManager<Meta>> {
        space: &'a AddressSpace<Meta, M>,
        ans: Option<Pte<Meta>>,
    }
    
    impl<'a, Meta: VmMeta, M: PageManager<Meta>> Visitor<'a, Meta, M> {
        #[inline]
        pub const fn new(space: &'a AddressSpace<Meta, M>) -> Self {
            Self { space, ans: None }
        }
    
        #[inline]
        pub const fn ans(self) -> Option<Pte<Meta>> {
            self.ans
        }
    }

    impl<'a, Meta: VmMeta, M: PageManager<Meta>> page_table::Visitor<Meta> for Visitor<'a, Meta, M> {
        #[inline]
        fn arrive(&mut self, pte: Pte<Meta>, _target_hint: Pos<Meta>) -> Pos<Meta> {
            if pte.is_valid() {
                self.ans = Some(pte);
            }
            Pos::stop()
        }
    
        #[inline]
        fn meet(
            &mut self,
            _level: usize,
            pte: Pte<Meta>,
            _target_hint: Pos<Meta>,
        ) -> Option<NonNull<Pte<Meta>>> {
            Some(self.space.page_manager.p_to_v(pte.ppn()))
        }
    
        #[inline]
        fn block(&mut self, _level: usize, _pte: Pte<Meta>, _target: Pos<Meta>) -> Pos<Meta> {
            Pos::stop()
        }
    }
    


    /// 地址空间。
    pub struct AddressSpace<Meta: VmMeta, M: PageManager<Meta>> {
        /// 虚拟地址块
        pub areas: Vec<Range<VPN<Meta>>>,
        page_manager: M,
    }

    impl<Meta: VmMeta, M: PageManager<Meta>> AddressSpace<Meta, M> {
        /// 地址空间根页表
        #[inline]
        pub fn root(&self) -> PageTable<Meta> {
            unsafe { PageTable::from_root(self.page_manager.root_ptr()) }
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
    }


    static mut KERNEL_SPACE: MaybeUninit<AddressSpace<Sv39, Sv39Manager>> = MaybeUninit::uninit();
    const VIRTIO0: usize = 0x10001000;

    pub static BLOCK_DEVICE: Lazy<Arc<dyn BlockDevice>> = Lazy::new(|| {
        Arc::new(unsafe {
            VirtIOBlock(Mutex::new(
                VirtIOBlk::new(&mut *(VIRTIO0 as *mut VirtIOHeader)).unwrap(),
            ))
        })
    });


    struct VirtIOBlock(Mutex<VirtIOBlk<'static, VirtioHal>>);

    impl BlockDevice for VirtIOBlock {
        fn read_block(&self, block_id: usize, buf: &mut [u8]) {
            self.0
                .lock()
                .read_block(block_id, buf)
                .expect("Error when reading VirtIOBlk");
        }
        fn write_block(&self, block_id: usize, buf: &[u8]) {
            self.0
                .lock()
                .write_block(block_id, buf)
                .expect("Error when writing VirtIOBlk");
        }
    }
    struct VirtioHal;

    impl Hal for VirtioHal {
        fn dma_alloc(pages: usize) -> usize {
            // warn!("dma_alloc");
            unsafe {
                alloc_zeroed(Layout::from_size_align_unchecked(
                    pages << Sv39::PAGE_BITS,
                    1 << Sv39::PAGE_BITS,
                )) as _
            }
        }

        fn dma_dealloc(paddr: usize, pages: usize) -> i32 {
            // warn!("dma_dealloc");
            unsafe {
                dealloc(
                    paddr as _,
                    Layout::from_size_align_unchecked(pages << Sv39::PAGE_BITS, 1 << Sv39::PAGE_BITS),
                )
            }
            0
        }

        fn phys_to_virt(paddr: usize) -> usize {
            // warn!("p2v");
            paddr
        }

        fn virt_to_phys(vaddr: usize) -> usize {
            // warn!("v2p");
            // const VALID: VmFlags<Sv39> = VmFlags::build_from_str("__V");
            const VALID: VmFlags<Sv39> = VmFlags::<Sv39>::VALID;
            let ptr: NonNull<u8> = unsafe {
                KERNEL_SPACE
                    .assume_init_ref()
                    .translate(VAddr::new(vaddr), VALID)
                    .unwrap()
            };
            ptr.as_ptr() as usize
        }
    }

    use alloc::{string::String};
    use crate::{EasyFileSystem, FSManager, FileHandle, Inode, OpenFlags};
    //use spin::Lazy;
    
    pub static FS: Lazy<FileSystem> = Lazy::new(|| FileSystem {
        root: EasyFileSystem::root_inode(&EasyFileSystem::open(BLOCK_DEVICE.clone())),
    });
    
    pub struct FileSystem {
        root: Inode,
    }
    
    impl FSManager for FileSystem {
        fn open(&self, path: &str, flags: OpenFlags) -> Option<Arc<FileHandle>> {
            let (readable, writable) = flags.read_write();
            if flags.contains(OpenFlags::CREATE) {
                if let Some(inode) = self.find(path) {
                    // Clear size
                    inode.clear();
                    Some(Arc::new(FileHandle::new(readable, writable, inode)))
                } else {
                    // Create new file
                    self.root
                        .create(path)
                        .map(|new_inode| Arc::new(FileHandle::new(readable, writable, new_inode)))
                }
            } else {
                self.find(path).map(|inode| {
                    if flags.contains(OpenFlags::TRUNC) {
                        inode.clear();
                    }
                    Arc::new(FileHandle::new(readable, writable, inode))
                })
            }
        }
    
        fn find(&self, path: &str) -> Option<Arc<Inode>> {
            self.root.find(path)
        }
    
        fn readdir(&self, _path: &str) -> Option<alloc::vec::Vec<String>> {
            Some(self.root.readdir())
        }
    
        fn link(&self, _src: &str, _dst: &str) -> isize {
            unimplemented!()
        }
    
        fn unlink(&self, _path: &str) -> isize {
            unimplemented!()
        }
    }
    
    pub fn read_all(fd: Arc<FileHandle>) -> Vec<u8> {
        let mut offset = 0usize;
        let mut buffer = [0u8; 512];
        let mut v: Vec<u8> = Vec::new();
        if let Some(inode) = &fd.inode {
            loop {
                let len = inode.read_at(offset, &mut buffer);
                if len == 0 {
                    break;
                }
                offset += len;
                v.extend_from_slice(&buffer[..len]);
            }
        }
        v
    }
    



    #[test]
    fn test_bitmap() {
        let bitmap1 = Bitmap::new(VIRTIO0,100);
        let _a = &BLOCK_DEVICE.clone();
        //bitmap1.alloc(&BLOCK_DEVICE.clone());
        //(& bitmap1).dealloc(&block_device1, 300);
        //测试获取可分配块的最大数量
        assert_eq!(409600, (& bitmap1).maximum());
    }

    use crate::block_cache::*;
    use crate::BLOCK_SZ;
    #[test]
    fn test_block_cache() {
        let block_id = VIRTIO0+4096;
        //let _a = BLOCK_DEVICE.clone();
        let mut cache = [0u8; BLOCK_SZ];
        //block_device.read_block(block_id, &mut cache);
        // BlockCache {
        //     cache,
        //     block_id,
        //     block_device,
        //     modified: false,
        // };
        let block_device1: Lazy<Arc<dyn BlockDevice>> = Lazy::new(|| {
            Arc::new(unsafe {
                VirtIOBlock(Mutex::new(
                    VirtIOBlk::new(&mut *(VIRTIO0 as *mut VirtIOHeader)).unwrap(),
                ))
            })
        });
        //let _a = block_device1.clone();
        //BlockCache::new(VIRTIO0+4096, BLOCK_DEVICE.clone());
        BlockCacheManager::new();
    }


    #[test]
    fn test_efs() {
        //let initproc = read_all(FS.open("initproc", OpenFlags::RDONLY).unwrap());
        //EasyFileSystem::create(BLOCK_DEVICE.clone(), 4096*5, 4096);
        //EasyFileSystem::root_inode(&EasyFileSystem::open(BLOCK_DEVICE.clone()));
        EasyFileSystem::open(BLOCK_DEVICE.clone());
        
    }


    #[test]
    fn test_file() {
        // let mut points_buf : Vec<u8> = Vec::with_capacity(points.len() * point::POINT_SIZE);
        // for _ in (0..points_buf.capacity()) {
        //     points_buf.push(0);
        // }
        // file.read(&mut points_buf[..]).unwrap();

        // let mut buffer = [0u8; 512];
        // //let mut v: Vec<u8> = Vec![0; 512];
        // //let buffers = v.extend_from_slice(&buffer[..512]);
        // UserBuffer::new(v);
    }

    use crate::layout::{SuperBlock, DiskInode, DiskInodeType};
    /// Magic number for sanity check
    const EFS_MAGIC: u32 = 0x3b800001;
    /// The max number of direct inodes
    const INODE_DIRECT_COUNT: usize = 28;
    /// The max length of inode name
    const NAME_LENGTH_LIMIT: usize = 27;
    /// The max number of indirect1 inodes
    const INODE_INDIRECT1_COUNT: usize = BLOCK_SZ / 4;
    /// The max number of indirect2 inodes
    const INODE_INDIRECT2_COUNT: usize = INODE_INDIRECT1_COUNT * INODE_INDIRECT1_COUNT;
    /// The upper bound of direct inode index
    const DIRECT_BOUND: usize = INODE_DIRECT_COUNT;
    /// The upper bound of indirect1 inode index
    const INDIRECT1_BOUND: usize = DIRECT_BOUND + INODE_INDIRECT1_COUNT;
    /// The upper bound of indirect2 inode indexs
    #[allow(unused)]
    const INDIRECT2_BOUND: usize = INDIRECT1_BOUND + INODE_INDIRECT2_COUNT;
    #[test]
    fn test_layout() {
        let mut superblock = SuperBlock{
            magic: 0x3b800001,
            total_blocks: 512,
            inode_bitmap_blocks: 1,
            inode_area_blocks: 1,
            data_bitmap_blocks: 1,
            data_area_blocks: 1,
        };
        (&mut superblock).initialize(0x3b800001,10,10,10,10);
        assert_eq!(true, (&superblock).is_valid());

        let mut diskinode = DiskInode{
            size:0,
            direct: [0; INODE_DIRECT_COUNT],
            indirect1: 0,
            indirect2: 0,
            //目录还是文件
            type_: DiskInodeType::File
        };
        //测试初始化
        (&mut diskinode).initialize(DiskInodeType::Directory);
        //判断是否是目录
        assert_eq!(true, (&diskinode).is_dir());

        (&mut diskinode).initialize(DiskInodeType::File);
        //判断是否是文件
        assert_eq!(true, (&diskinode).is_file());
        let datablocks = (&diskinode).data_blocks();
        assert_eq!(0, datablocks);
        //分别调用直接索引，一级索引，二级索引时的数据块个数
        let tolal1 = DiskInode::total_blocks(4096);
        let tolal2 = DiskInode::total_blocks(13825);
        let tolal3 = DiskInode::total_blocks(79360);
        assert_eq!(8, tolal1);
        assert_eq!(28, tolal2);
        assert_eq!(156, tolal3);
        let needed1 = (&diskinode).blocks_num_needed(4096);
        assert_eq!(8, needed1);
    }

    #[test]
    fn test_vfs() {
        
    }
}


//!An easy file system isolated from the kernel
#![no_std]
// #![deny(missing_docs)]
extern crate alloc;
mod bitmap;
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

    // use alloc::{
    //     alloc::{alloc_zeroed, dealloc},
    //     sync::Arc,
    // };
    // use core::{alloc::Layout, ptr::NonNull};
    // use crate::BlockDevice;
    // use spin::{Lazy, Mutex};
    // use virtio_drivers::{Hal, VirtIOBlk, VirtIOHeader};
    // use core::mem::MaybeUninit;

    // use page_table::{MmuMeta, Pte, VAddr, VmFlags, PPN, VPN};

    // #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
    // pub(crate) struct Sv39;

    // impl MmuMeta for Sv39 {
    //     const P_ADDR_BITS: usize = 56;
    //     const PAGE_BITS: usize = 12;
    //     const LEVEL_BITS: &'static [usize] = &[9; 3];
    //     const PPN_POS: usize = 10;

    //     #[inline]
    //     fn is_leaf(value: usize) -> bool {
    //         const MASK: usize = 0b1110;
    //         value & MASK != 0
    //     }
    // }

    // #[repr(transparent)]
    // pub struct Sv39Manager(NonNull<Pte<Sv39>>);

    // //实现page_alloc
    // extern "Rust" {
    //     fn __rust_alloc_zeroed(size: usize, align: usize) -> *mut u8;
    // }


    // #[must_use = "losing the pointer will leak memory"]
    // #[inline]
    // pub unsafe fn alloc_zeroed(layout: Layout) -> *mut u8 {
    //     unsafe { __rust_alloc_zeroed(layout.size(), layout.align()) }
    // }

    // impl Sv39Manager {
    //     const OWNED: VmFlags<Sv39> = unsafe { VmFlags::from_raw(1 << 8) };

    //     #[inline]
    //     fn page_alloc<T>(count: usize) -> *mut T {
    //         unsafe {
    //             alloc_zeroed(Layout::from_size_align_unchecked(
    //                 count << Sv39::PAGE_BITS,
    //                 1 << Sv39::PAGE_BITS,
    //             ))
    //         }
    //         .cast()
    //     }
    // }

    // impl PageManager<Sv39> for Sv39Manager {
    //     #[inline]
    //     fn new_root() -> Self {
    //         Self(NonNull::new(Self::page_alloc(1)).unwrap())
    //     }

    //     #[inline]
    //     fn root_ppn(&self) -> PPN<Sv39> {
    //         PPN::new(self.0.as_ptr() as usize >> Sv39::PAGE_BITS)
    //     }

    //     #[inline]
    //     fn root_ptr(&self) -> NonNull<Pte<Sv39>> {
    //         self.0
    //     }

    //     #[inline]
    //     fn p_to_v<T>(&self, ppn: PPN<Sv39>) -> NonNull<T> {
    //         unsafe { NonNull::new_unchecked(VPN::<Sv39>::new(ppn.val()).base().as_mut_ptr()) }
    //     }

    //     #[inline]
    //     fn v_to_p<T>(&self, ptr: NonNull<T>) -> PPN<Sv39> {
    //         PPN::new(VAddr::<Sv39>::new(ptr.as_ptr() as _).floor().val())
    //     }

    //     #[inline]
    //     fn check_owned(&self, pte: Pte<Sv39>) -> bool {
    //         pte.flags().contains(Self::OWNED)
    //     }

    //     #[inline]
    //     fn allocate(&mut self, len: usize, flags: &mut VmFlags<Sv39>) -> NonNull<u8> {
    //         *flags |= Self::OWNED;
    //         NonNull::new(Self::page_alloc(len)).unwrap()
    //     }

    //     fn deallocate(&mut self, _pte: Pte<Sv39>, _len: usize) -> usize {
    //         todo!()
    //     }

    //     fn drop_root(&mut self) {
    //         todo!()
    //     }
    // } 




    // static mut KERNEL_SPACE: MaybeUninit<AddressSpace<Sv39, Sv39Manager>> = MaybeUninit::uninit();
    // const VIRTIO0: usize = 0x10001000;

    // pub static BLOCK_DEVICE: Lazy<Arc<dyn BlockDevice>> = Lazy::new(|| {
    //     Arc::new(unsafe {
    //         VirtIOBlock(Mutex::new(
    //             VirtIOBlk::new(&mut *(VIRTIO0 as *mut VirtIOHeader)).unwrap(),
    //         ))
    //     })
    // });
    // struct VirtIOBlock(Mutex<VirtIOBlk<'static, VirtioHal>>);

    // impl BlockDevice for VirtIOBlock {
    //     fn read_block(&self, block_id: usize, buf: &mut [u8]) {
    //         self.0
    //             .lock()
    //             .read_block(block_id, buf)
    //             .expect("Error when reading VirtIOBlk");
    //     }
    //     fn write_block(&self, block_id: usize, buf: &[u8]) {
    //         self.0
    //             .lock()
    //             .write_block(block_id, buf)
    //             .expect("Error when writing VirtIOBlk");
    //     }
    // }
    // struct VirtioHal;

    // impl Hal for VirtioHal {
    //     fn dma_alloc(pages: usize) -> usize {
    //         // warn!("dma_alloc");
    //         unsafe {
    //             alloc_zeroed(Layout::from_size_align_unchecked(
    //                 pages << Sv39::PAGE_BITS,
    //                 1 << Sv39::PAGE_BITS,
    //             )) as _
    //         }
    //     }

    //     fn dma_dealloc(paddr: usize, pages: usize) -> i32 {
    //         // warn!("dma_dealloc");
    //         unsafe {
    //             dealloc(
    //                 paddr as _,
    //                 Layout::from_size_align_unchecked(pages << Sv39::PAGE_BITS, 1 << Sv39::PAGE_BITS),
    //             )
    //         }
    //         0
    //     }

    //     fn phys_to_virt(paddr: usize) -> usize {
    //         // warn!("p2v");
    //         paddr
    //     }

    //     fn virt_to_phys(vaddr: usize) -> usize {
    //         // warn!("v2p");
    //         const VALID: VmFlags<Sv39> = VmFlags::build_from_str("__V");
    //         let ptr: NonNull<u8> = unsafe {
    //             KERNEL_SPACE
    //                 .assume_init_ref()
    //                 .translate(VAddr::new(vaddr), VALID)
    //                 .unwrap()
    //         };
    //         ptr.as_ptr() as usize
    //     }
    // }



    #[test]
    fn test_bitmap() {
        // let mut a = 10;
        // decomposition(a);
        let bitmap1 = Bitmap::new(0,10);
    }

    #[test]
    fn test_block_cache() {
        
    }

    #[test]
    fn test_efs() {
        
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

    use crate::layout::{SuperBlock};
    #[test]
    fn test_layout() {
        // let a = SuperBlock{
        //     magic: 0x3b800001,
        //     total_blocks: 512,
        //     inode_bitmap_blocks: 1,
        //     inode_area_blocks: 1,
        //     data_bitmap_blocks: 1,
        //     data_area_blocks: 1,
        // };
        //SuperBlock::initialize(a,EFS_MAGIC,);
    }

    #[test]
    fn test_vfs() {
        
    }
}
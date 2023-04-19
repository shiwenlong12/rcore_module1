// use easy_fs::{};

// use easy_fs::Bitmap;
// //use easy_fs::BlockDevice;
// use easy_fs::file::{UserBuffer};

// use alloc::{
//     alloc::{alloc_zeroed, dealloc},
//     sync::Arc,
// };
// use core::{alloc::Layout, ptr::NonNull};
// use easy_fs::BlockDevice;
// use spin::{Lazy, Mutex};
// use virtio_drivers::{Hal, VirtIOBlk, VirtIOHeader};
// use core::mem::MaybeUninit;

// use page_table::{MmuMeta, Pte, VAddr, VmFlags, PPN, VPN};

// use alloc::vec::Vec;
// use core::ops::Range;
// use page_table::VmMeta;
// use page_table::Pos;
// use page_table::PageTable;



// use alloc::{string::String};
// use easy_fs::{EasyFileSystem, FSManager, FileHandle, Inode, OpenFlags};
//use spin::Lazy;
use easy_fs::{bitmap::Bitmap};

/// 将应用打包到 easy-fs 镜像中放到磁盘中，
/// 当我们要执行应用的时候只需从文件系统中取出ELF 执行文件格式的应用 并加载到内存中执行即可，
//use clap::{App, Arg};
use easy_fs::{BlockDevice, EasyFileSystem};
use std::fs::{read_dir, File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::sync::Arc;
use std::sync::Mutex;

/// Use a block size of 512 bytes
const BLOCK_SZ: usize = 512;
const BLOCK_NUM: usize = 131072; //64*2048

/// Wrapper for turning a File into a BlockDevice
struct BlockFile(Mutex<File>);

impl BlockDevice for BlockFile {
    /// Read a block from file
    fn read_block(&self, block_id: usize, buf: &mut [u8]) {
        let mut file = self.0.lock().unwrap();
        file.seek(SeekFrom::Start((block_id * BLOCK_SZ) as u64))
            .expect("Error when seeking!");
        assert_eq!(file.read(buf).unwrap(), BLOCK_SZ, "Not a complete block!");
    }
    /// Write a block into file
    fn write_block(&self, block_id: usize, buf: &[u8]) {
        let mut file = self.0.lock().unwrap();
        file.seek(SeekFrom::Start((block_id * BLOCK_SZ) as u64))
            .expect("Error when seeking!");
        assert_eq!(file.write(buf).unwrap(), BLOCK_SZ, "Not a complete block!");
    }
}
//use easy_fs::{get_block_cache};
/// Use a block size of 512 bytes
//pub const BLOCK_SZ: usize = 512;
/// Number of bits in a block
const BLOCK_BITS: usize = BLOCK_SZ * 8;
const VIRTIO0: usize = 0x10001000;

#[test]
fn test_bitmap() {
    let block_file = Arc::new(BlockFile(Mutex::new({
        let f = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open("target/fs.img")?;
        f.set_len(BLOCK_NUM * BLOCK_SZ).unwrap();
        f
    })));
    let bitmap1 = Bitmap::new(VIRTIO0,100);
    // let _a = &BLOCK_DEVICE.clone();
    // //bitmap1.alloc(&BLOCK_DEVICE.clone());
    // //(& bitmap1).dealloc(&block_device1, 300);
    // //测试获取可分配块的最大数量
    // assert_eq!(409600, (& bitmap1).maximum());
}

// use easy_fs::block_cache::*;
// use easy_fs::BLOCK_SZ;
#[test]
fn test_block_cache() {
    //let block_id = VIRTIO0+4096;
    //let _a = BLOCK_DEVICE.clone();
    //let mut cache = [0u8; BLOCK_SZ];
    //block_device.read_block(block_id, &mut cache);
    // BlockCache {
    //     cache,
    //     block_id,
    //     block_device,
    //     modified: false,
    // };
    // let block_device1: Lazy<Arc<dyn BlockDevice>> = Lazy::new(|| {
    //     Arc::new(unsafe {
    //         VirtIOBlock(Mutex::new(
    //             VirtIOBlk::new(&mut *(VIRTIO0 as *mut VirtIOHeader)).unwrap(),
    //         ))
    //     })
    // });
    //let _a = block_device1.clone();
    //BlockCache::new(VIRTIO0+4096, BLOCK_DEVICE.clone());
    //BlockCacheManager::new();
}


#[test]
fn test_efs() {
    //let initproc = read_all(FS.open("initproc", OpenFlags::RDONLY).unwrap());
    //EasyFileSystem::create(BLOCK_DEVICE.clone(), 4096*5, 4096);
    //EasyFileSystem::root_inode(&EasyFileSystem::open(BLOCK_DEVICE.clone()));
    //EasyFileSystem::open(BLOCK_DEVICE.clone());
    
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


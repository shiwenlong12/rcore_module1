use super::{BlockDevice, BLOCK_SZ};
use alloc::{collections::VecDeque, sync::Arc};
use spin::{Lazy, Mutex};

/// Cached block inside memory
pub struct BlockCache {
    /// cached block data
    /// 表示位于内存中的缓冲区
    cache: [u8; BLOCK_SZ],
    /// underlying block id
    /// 潜在的区块编号
    block_id: usize,
    /// underlying block device
    /// 记录块所属的底层设备，这里只有一个磁盘
    block_device: Arc<dyn BlockDevice>,
    /// whether the block is dirty
    /// 记录自从这个块缓存从磁盘载入内存之后，它有没有被修改过
    modified: bool,
}

impl BlockCache {
    /// Load a new BlockCache from disk.
    /// 从磁盘加载新的 BlockCache。
    /// 创建 BlockCache 时，将一个块从磁盘读到缓冲区 cache
    pub fn new(block_id: usize, block_device: Arc<dyn BlockDevice>) -> Self {
        let mut cache = [0u8; BLOCK_SZ];
        block_device.read_block(block_id, &mut cache);
        Self {
            cache,
            block_id,
            block_device,
            modified: false,
        }
    }
    /// Get the address of an offset inside the cached block data
    /// 得到一个 BlockCache 内部的缓冲区中指定偏移量 offset 的字节地址
    fn addr_of_offset(&self, offset: usize) -> usize {
        &self.cache[offset] as *const _ as usize
    }

    /// 可以获取缓冲区中的位于偏移量 offset 的一个类型为 T 的磁盘上数据结构的不可变引用。
    pub fn get_ref<T>(&self, offset: usize) -> &T
    where
        T: Sized,
    {
        let type_size = core::mem::size_of::<T>();
        assert!(offset + type_size <= BLOCK_SZ);
        let addr = self.addr_of_offset(offset);
        unsafe { &*(addr as *const T) }
    }

    pub fn get_mut<T>(&mut self, offset: usize) -> &mut T
    where
        T: Sized,
    {
        let type_size = core::mem::size_of::<T>();
        assert!(offset + type_size <= BLOCK_SZ);
        self.modified = true;
        let addr = self.addr_of_offset(offset);
        unsafe { &mut *(addr as *mut T) }
    }

    //在 BlockCache 缓冲区偏移量为 offset 的位置，获取一个类型为 T 不可变/可变引用，
    //将闭包 f 作用于这个引用，返回 f 的返回值。
    pub fn read<T, V>(&self, offset: usize, f: impl FnOnce(&T) -> V) -> V {
        f(self.get_ref(offset))
    }

    pub fn modify<T, V>(&mut self, offset: usize, f: impl FnOnce(&mut T) -> V) -> V {
        f(self.get_mut(offset))
    }

    //同步即如果被修改了，写回磁盘里面去
    pub fn sync(&mut self) {
        if self.modified {
            self.modified = false;
            self.block_device.write_block(self.block_id, &self.cache);
        }
    }
}

impl Drop for BlockCache {
    fn drop(&mut self) {
        self.sync()
    }
}
/// Use a block cache of 16 blocks
const BLOCK_CACHE_SIZE: usize = 16;

pub struct BlockCacheManager {
    queue: VecDeque<(usize, Arc<Mutex<BlockCache>>)>,
}

impl BlockCacheManager {
    pub fn new() -> Self {
        Self {
            queue: VecDeque::new(),
        }
    }

 //尝试从块缓存管理器中获取一个编号为 block_id 的块缓存
    pub fn get_block_cache(
        &mut self,
        block_id: usize,
        block_device: Arc<dyn BlockDevice>,
    ) -> Arc<Mutex<BlockCache>> {
        //遍历整个队列试图找到一个编号相同的块缓存，如果找到，将块缓存管理器中保存的块缓存的引用复制一份并返回
        if let Some(pair) = self.queue.iter().find(|pair| pair.0 == block_id) {
            Arc::clone(&pair.1)
        } else {
            // substitute
            //判断已保存的块数量是否达到了上限
            if self.queue.len() == BLOCK_CACHE_SIZE {
                // from front to tail
                //替换的标准是其强引用计数 ，即除了块缓存管理器保留的一份副本之外，在外面没有副本正在使用。
                if let Some((idx, _)) = self
                    .queue
                    .iter()
                    .enumerate()
                    .find(|(_, pair)| Arc::strong_count(&pair.1) == 1)
                {
                    self.queue.drain(idx..=idx);
                } else {
                    panic!("Run out of BlockCache!");
                }
            }
            // load block into mem and push back
            //创建一个新的块缓存（会触发 read_block 进行块读取）并加入到队尾，最后返回给请求者。
            let block_cache = Arc::new(Mutex::new(BlockCache::new(
                block_id,
                Arc::clone(&block_device),
            )));
            self.queue.push_back((block_id, Arc::clone(&block_cache)));
            block_cache
        }
    }
}

/// The global block cache manager
pub static BLOCK_CACHE_MANAGER: Lazy<Mutex<BlockCacheManager>> =
    Lazy::new(|| Mutex::new(BlockCacheManager::new()));

/// Get the block cache corresponding to the given block id and block device
/// 获取与给定块id和块设备对应的块缓存 
pub fn get_block_cache(
    block_id: usize,
    block_device: Arc<dyn BlockDevice>,
) -> Arc<Mutex<BlockCache>> {
    BLOCK_CACHE_MANAGER
        .lock()
        .get_block_cache(block_id, block_device)
}
/// Sync all block cache to block device
/// /// 将所有块缓存同步到块设备
pub fn block_cache_sync_all() {
    let manager = BLOCK_CACHE_MANAGER.lock();
    for (_, cache) in manager.queue.iter() {
        cache.lock().sync();
    }
}

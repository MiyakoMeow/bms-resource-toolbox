use once_cell::sync::Lazy;
use smol::lock::Semaphore;
use std::{collections::HashMap, path::Path, sync::Arc};
use sysinfo::{DiskKind, Disks};

// 类型别名，简化复杂的类型定义
type DiskLocksMap = HashMap<String, Arc<Semaphore>>;
type DiskLocksMutex = Arc<smol::lock::Mutex<DiskLocksMap>>;

// 全局锁管理器，为每个磁盘维护一个信号量
static DISK_LOCKS: Lazy<DiskLocksMutex> =
    Lazy::new(|| Arc::new(smol::lock::Mutex::new(HashMap::new())));

/// 获取指定路径所在磁盘的锁
/// 返回一个可以被同时持有的锁，持有数量等于该磁盘类型允许的并发操作数
pub async fn get_disk_lock(path: &Path) -> Arc<Semaphore> {
    let disk_key = get_disk_key(path);
    let locks = DISK_LOCKS.clone();

    // 获取或创建磁盘锁
    {
        let mut locks_guard = locks.lock().await;
        locks_guard
            .entry(disk_key.clone())
            .or_insert_with(|| {
                let max_concurrent = compute_max_concurrent_for_disk(path);
                Arc::new(Semaphore::new(max_concurrent))
            })
            .clone()
    }
}

/// 计算指定路径所在磁盘的最大并发操作数
fn compute_max_concurrent_for_disk(path: &Path) -> usize {
    let disks = Disks::new_with_refreshed_list();
    disks
        .iter()
        .filter(|d| path.starts_with(d.mount_point()))
        .max_by_key(|d| d.mount_point().components().count())
        .map(|d| match d.kind() {
            DiskKind::SSD => num_cpus::get(),
            DiskKind::HDD => 2,
            DiskKind::Unknown(_) => 2,
        })
        .unwrap_or_else(|| num_cpus::get().min(4))
}

/// 获取磁盘的唯一标识符
fn get_disk_key(path: &Path) -> String {
    let disks = Disks::new_with_refreshed_list();
    disks
        .iter()
        .filter(|d| path.starts_with(d.mount_point()))
        .max_by_key(|d| d.mount_point().components().count())
        .map(|d| d.mount_point().to_string_lossy().to_string())
        .unwrap_or_else(|| "unknown".to_string())
}

/// 获取指定路径所在磁盘的锁并自动管理生命周期
/// 使用示例：
/// ```rust
/// let _guard = acquire_disk_lock(&path).await;
/// // 在这里进行文件操作
/// // 当 guard 被 drop 时，锁会自动释放
/// ```
pub async fn acquire_disk_lock(path: &Path) -> smol::lock::SemaphoreGuard<'static> {
    let semaphore = get_disk_lock(path).await;
    // 这里需要将 Arc<Semaphore> 转换为 'static 生命周期
    // 由于我们使用的是全局静态变量，这是安全的
    unsafe {
        std::mem::transmute::<smol::lock::SemaphoreGuard<'_>, smol::lock::SemaphoreGuard<'static>>(
            semaphore.acquire().await,
        )
    }
}

/// 智能获取多个路径的磁盘锁，避免在同一磁盘上重复获取锁
/// 使用示例：
/// ```rust
/// let _guards = acquire_disk_locks(&[&src_path, &dst_path]).await;
/// // 在这里进行文件操作
/// // 当 guards 被 drop 时，锁会自动释放
/// ```
pub async fn acquire_disk_locks(paths: &[&Path]) -> Vec<smol::lock::SemaphoreGuard<'static>> {
    let mut disk_keys = std::collections::HashSet::new();
    let mut guards = Vec::new();

    for path in paths {
        let disk_key = get_disk_key(path);
        if disk_keys.insert(disk_key) {
            // 只有在这个磁盘的锁还没有获取过时才获取
            guards.push(acquire_disk_lock(path).await);
        }
    }

    guards
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_compute_max_concurrent_for_disk() {
        let path = PathBuf::from("/tmp/test");
        let result = compute_max_concurrent_for_disk(&path);
        assert!(result > 0);
    }

    #[test]
    fn test_get_disk_key() {
        let path = PathBuf::from("/tmp/test");
        let key = get_disk_key(&path);
        assert!(!key.is_empty());
    }
}

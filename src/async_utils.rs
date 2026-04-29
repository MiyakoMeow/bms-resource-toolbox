//! Async utilities for concurrent IO operations.
//!
//! Provides semaphore-based concurrency control and async directory walking.

use std::sync::Arc;

use tokio::sync::Semaphore;

/// Create a semaphore for controlling concurrency.
#[must_use]
pub fn create_semaphore(permits: usize) -> Arc<Semaphore> {
    Arc::new(Semaphore::new(permits))
}

/// Concurrency controller for async file operations.
pub struct AsyncFileOps {
    semaphore: Arc<Semaphore>,
}

impl AsyncFileOps {
    /// Create a new async file operations controller.
    #[must_use]
    pub fn new(max_concurrent: usize) -> Self {
        Self {
            semaphore: create_semaphore(max_concurrent),
        }
    }

    /// Acquire a semaphore permit.
    ///
    /// # Panics
    ///
    /// Panics if the semaphore is closed.
    pub async fn acquire(&self) -> tokio::sync::OwnedSemaphorePermit {
        self.semaphore.clone().acquire_owned().await.unwrap()
    }
}

/// Async directory walker with concurrency control.
pub struct AsyncDirWalker {
    max_concurrent: usize,
}

impl AsyncDirWalker {
    /// Create a new async directory walker.
    #[must_use]
    pub fn new(max_concurrent: usize) -> Self {
        Self { max_concurrent }
    }

    /// Walk a directory asynchronously.
    ///
    /// # Panics
    ///
    /// Panics if a semaphore is closed.
    pub async fn walk_dir(&self, root: &std::path::Path) -> Vec<std::path::PathBuf> {
        let semaphore = create_semaphore(self.max_concurrent);
        let mut handles = Vec::new();

        if let Ok(entries) = tokio::fs::read_dir(root).await {
            let mut entries = entries;
            while let Ok(Some(entry)) = entries.next_entry().await {
                let path = entry.path();
                let sem = semaphore.clone();

                let handle = tokio::spawn(async move {
                    let _permit = sem.acquire().await.unwrap();
                    path
                });

                handles.push(handle);
            }
        }

        let mut results = Vec::new();
        for handle in handles {
            if let Ok(path) = handle.await {
                results.push(path);
            }
        }

        results
    }
}

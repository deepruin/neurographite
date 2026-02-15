use std::path::{Path, PathBuf};
use tokio::fs;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use anyhow::{Result, Context};
use bincode;

use crate::hypergraph::HyperGraph;

/// Storage engine for persisting hypergraph data
pub struct StorageEngine {
    data_dir: PathBuf,
    graph_file: PathBuf,
    backup_dir: PathBuf,
}

impl StorageEngine {
    pub async fn new<P: AsRef<Path>>(data_dir: P) -> Result<Self> {
        let data_dir = data_dir.as_ref().to_path_buf();
        let graph_file = data_dir.join("graph.bin");
        let backup_dir = data_dir.join("backups");
        
        // Create directories if they don't exist
        fs::create_dir_all(&data_dir).await
            .context("Failed to create data directory")?;
        fs::create_dir_all(&backup_dir).await
            .context("Failed to create backup directory")?;
        
        Ok(Self {
            data_dir,
            graph_file,
            backup_dir,
        })
    }
    
    /// Save the hypergraph to persistent storage
    pub async fn save_graph(&self, graph: &HyperGraph) -> Result<()> {
        // Serialize the graph
        let serialized = bincode::serialize(graph)
            .context("Failed to serialize hypergraph")?;
        
        // Write to a temporary file first
        let temp_file = self.graph_file.with_extension("tmp");
        let mut file = fs::File::create(&temp_file).await
            .context("Failed to create temporary file")?;
        
        file.write_all(&serialized).await
            .context("Failed to write graph data")?;
        
        file.sync_all().await
            .context("Failed to sync file to disk")?;
        
        // Atomically replace the old file
        fs::rename(&temp_file, &self.graph_file).await
            .context("Failed to replace graph file")?;
        
        Ok(())
    }
    
    /// Load the hypergraph from persistent storage
    pub async fn load_graph(&self) -> Result<HyperGraph> {
        if !self.graph_file.exists() {
            return Ok(HyperGraph::new()); // Return empty graph if no file exists
        }
        
        let mut file = fs::File::open(&self.graph_file).await
            .context("Failed to open graph file")?;
        
        let mut contents = Vec::new();
        file.read_to_end(&mut contents).await
            .context("Failed to read graph file")?;
        
        let graph = bincode::deserialize(&contents)
            .context("Failed to deserialize hypergraph")?;
        
        Ok(graph)
    }
    
    /// Create a backup of the current graph
    pub async fn backup_graph(&self) -> Result<PathBuf> {
        if !self.graph_file.exists() {
            return Err(anyhow::anyhow!("No graph file to backup"));
        }
        
        let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
        let backup_file = self.backup_dir.join(format!("graph_{}.bin", timestamp));
        
        fs::copy(&self.graph_file, &backup_file).await
            .context("Failed to create backup")?;
        
        Ok(backup_file)
    }
    
    /// List all available backups
    pub async fn list_backups(&self) -> Result<Vec<PathBuf>> {
        let mut backups = Vec::new();
        let mut entries = fs::read_dir(&self.backup_dir).await
            .context("Failed to read backup directory")?;
        
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if path.extension().map_or(false, |ext| ext == "bin") {
                backups.push(path);
            }
        }
        
        backups.sort();
        Ok(backups)
    }
    
    /// Restore graph from a backup
    pub async fn restore_from_backup<P: AsRef<Path>>(&self, backup_path: P) -> Result<HyperGraph> {
        let backup_path = backup_path.as_ref();
        
        let mut file = fs::File::open(backup_path).await
            .with_context(|| format!("Failed to open backup file: {:?}", backup_path))?;
        
        let mut contents = Vec::new();
        file.read_to_end(&mut contents).await
            .context("Failed to read backup file")?;
        
        let graph = bincode::deserialize(&contents)
            .context("Failed to deserialize backup")?;
        
        Ok(graph)
    }
    
    /// Get storage statistics
    pub async fn stats(&self) -> Result<StorageStats> {
        let graph_size = if self.graph_file.exists() {
            fs::metadata(&self.graph_file).await?.len()
        } else {
            0
        };
        
        let backups = self.list_backups().await?;
        let backup_count = backups.len();
        
        let mut total_backup_size = 0;
        for backup in backups {
            if let Ok(metadata) = fs::metadata(&backup).await {
                total_backup_size += metadata.len();
            }
        }
        
        Ok(StorageStats {
            graph_file_size: graph_size,
            backup_count,
            total_backup_size,
        })
    }
    
    /// Cleanup old backups (keep only the last N)
    pub async fn cleanup_backups(&self, keep_count: usize) -> Result<usize> {
        let mut backups = self.list_backups().await?;
        
        if backups.len() <= keep_count {
            return Ok(0);
        }
        
        // Sort by modification time (oldest first)
        backups.sort_by_key(|path| {
            std::fs::metadata(path)
                .ok()
                .and_then(|m| m.modified().ok())
                .unwrap_or(std::time::UNIX_EPOCH)
        });
        
        let to_remove = backups.len() - keep_count;
        let mut removed_count = 0;
        
        for backup_path in backups.iter().take(to_remove) {
            if fs::remove_file(backup_path).await.is_ok() {
                removed_count += 1;
            }
        }
        
        Ok(removed_count)
    }
}

#[derive(Debug, Clone)]
pub struct StorageStats {
    pub graph_file_size: u64,
    pub backup_count: usize,
    pub total_backup_size: u64,
}
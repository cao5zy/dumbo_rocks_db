use anyhow::{Context, Result};
use rocksdb::{Options, DB};
use std::path::Path;
use std::sync::OnceLock; // 移除了Arc的引入

fn open_db_with_column_families(db_path: &Path, column_families: &[&str]) -> Result<DB> {
    std::fs::create_dir_all(db_path).context("Failed to create db directory")?;

    let mut opts = Options::default();
    opts.create_if_missing(true);
    opts.create_missing_column_families(true);

    DB::open_cf(&opts, db_path, column_families)
        .context("Failed to open RocksDB with column families")
}

pub struct DbContext {
    pub db: DB,
}

static INSTANCE: OnceLock<DbContext> = OnceLock::new();

impl DbContext {
    pub fn initialize(db_path: &Path, column_families: &[&str]) -> Result<()> {
        let db = open_db_with_column_families(db_path, column_families)?;
        INSTANCE
            .set(DbContext { db }) // 直接存储DB实例，无需Arc
            .map_err(|_| anyhow::anyhow!("DbContext already initialized")) // 更新错误消息
    }

    pub fn get_instance() -> &'static Self {
        INSTANCE.get().expect("DbContext not initialized")
    }
}

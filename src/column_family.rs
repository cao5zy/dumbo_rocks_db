use crate::DbContext;
use anyhow::{Context, Result};
use rocksdb::IteratorMode;
use serde::{de::DeserializeOwned, Serialize};

fn serialize_to_bytes<T: Serialize>(value: &T) -> Result<Vec<u8>> {
    serde_json::to_vec(value).context("Failed to serialize value")
}

fn deserialize_from_bytes<T: DeserializeOwned>(bytes: &[u8]) -> Result<T> {
    serde_json::from_slice(bytes).context("Failed to deserialize value")
}

pub trait Keyable: serde::Serialize + serde::de::DeserializeOwned {
    fn key(&self) -> String;
    fn column_family() -> &'static str;
}

/// 表示RocksDB中的一个列族(column family)
///
/// 泛型参数`T`需要实现`Keyable`特性，用于定义:
/// 1. 数据的主键生成方式(`key()`)
/// 2. 所属列族名称(`column_family()`)
///
/// 提供基本的CRUD操作接口，包括：
/// - 获取所有记录(`get_all`)
/// - 按主键查询(`get`)
/// - 删除记录(`del`)
/// - 插入/更新记录(`set`)
pub struct ColumnFamily<T: Keyable> {
    _phantom: std::marker::PhantomData<T>,
}

impl<T: Keyable> Default for ColumnFamily<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Keyable> ColumnFamily<T> {
    /// 创建指定类型的列族实例
    ///
    /// 该实例不包含实际数据，仅作为操作指定列族的接口
    pub fn new() -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }

    /// 获取当前列族中的所有记录
    ///
    /// # 返回值
    /// - `Ok(Vec<T>)`: 包含所有反序列化后的记录
    /// - `Err`: 当发生以下情况时返回错误：
    ///   - 无法获取列族句柄
    ///   - 数据库迭代失败
    ///   - 数据反序列化失败
    pub fn get_all(&self) -> Result<Vec<T>> {
        let cf_handle = DbContext::get_instance()
            .db
            .cf_handle(T::column_family())
            .context(format!(
                "Failed to get {} column family handle",
                T::column_family()
            ))?;

        let mut items = Vec::new();
        let iter = DbContext::get_instance()
            .db
            .iterator_cf(&cf_handle, IteratorMode::Start);

        for item in iter {
            let (_key, value) = item.context("Failed to read database entry")?;
            let item: T = deserialize_from_bytes(&value)?;
            items.push(item);
        }

        Ok(items)
    }

    /// 根据主键查询单条记录
    ///
    /// # 参数
    /// - `key`: 要查询记录的主键
    ///
    /// # 返回值
    /// - `Ok(Some(T))`: 找到对应记录并成功反序列化
    /// - `Ok(None)`: 未找到对应记录
    /// - `Err`: 当发生以下情况时返回错误：
    ///   - 无法获取列族句柄
    ///   - 数据库读取失败
    ///   - 数据反序列化失败
    pub fn get(&self, key: &str) -> Result<Option<T>> {
        let cf_handle = DbContext::get_instance()
            .db
            .cf_handle(T::column_family())
            .context(format!(
                "Failed to get {} column family handle",
                T::column_family()
            ))?;

        match DbContext::get_instance()
            .db
            .get_cf(&cf_handle, key)
            .context("Failed to read database entry")?
        {
            Some(value) => {
                let item: T = deserialize_from_bytes(&value)?;
                Ok(Some(item))
            }
            None => Ok(None),
        }
    }

    /// 根据主键删除记录
    ///
    /// # 参数
    /// - `key`: 要删除记录的主键
    ///
    /// # 返回值
    /// - `Ok(())`: 删除成功
    /// - `Err`: 当发生以下情况时返回错误：
    ///   - 无法获取列族句柄
    ///   - 数据库删除操作失败
    pub fn del(&self, key: &str) -> Result<()> {
        let cf_handle = DbContext::get_instance()
            .db
            .cf_handle(T::column_family())
            .context(format!(
                "Failed to get {} column family handle",
                T::column_family()
            ))?;

        DbContext::get_instance()
            .db
            .delete_cf(&cf_handle, key)
            .context("Failed to delete item")
    }

    /// 插入或更新记录
    ///
    /// 如果主键已存在则更新，否则插入新记录
    ///
    /// # 参数
    /// - `item`: 要存储的记录对象
    ///
    /// # 返回值
    /// - `Ok(())`: 操作成功
    /// - `Err`: 当发生以下情况时返回错误：
    ///   - 无法获取列族句柄
    ///   - 数据序列化失败
    ///   - 数据库写入失败
    pub fn set(&self, item: &T) -> Result<()> {
        let cf_handle = DbContext::get_instance()
            .db
            .cf_handle(T::column_family())
            .context(format!(
                "Failed to get {} column family handle",
                T::column_family()
            ))?;

        let key = item.key();
        let value = serialize_to_bytes(item)?;

        DbContext::get_instance()
            .db
            .put_cf(&cf_handle, key, value)
            .context("Failed to write item to database")
    }
}
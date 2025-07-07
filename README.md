# dumbo_rocks_db

RocksDB数据库操作模块，提供类型安全的列族(Column Family)抽象。

## 概述
本模块通过`DbContext`管理数据库连接，使用`ColumnFamily<T>`提供类型安全的数据操作。每个列族对应一个类型`T`，该类型需实现`Keyable` trait。

## 快速开始

### 1. 初始化数据库
在系统启动时初始化全局数据库实例：
```rust
use dumbo_rocks_db::DbContext;

// 参数说明：
// - db_path: 数据库存储路径
// - column_families: 所有列族名称列表
DbContext::initialize(db_path, column_families)?;
```

### 2. 定义数据模型
为每个列族创建实现`Keyable`的类型：
```rust
use serde::{Serialize, Deserialize};
use dumbo_rocks_db::Keyable;

#[derive(Serialize, Deserialize)]
struct User {
    id: String,
    name: String,
}

impl Keyable for User {
    fn key(&self) -> String {
        self.id.clone()
    }
    
    fn column_family() -> &'static str {
        "users"  // 对应初始化时传入的列族名称
    }
}
```

### 3. 使用列族操作数据
```rust
use dumbo_rocks_db::ColumnFamily;

// 创建列族访问对象
let user_cf = ColumnFamily::<User>::new();

// 插入数据
user_cf.set(&User {
    id: "001".into(),
    name: "Alice".into()
})?;

// 查询数据
let user = user_cf.get("001")?;

// 获取全部数据
let all_users = user_cf.get_all()?;

// 删除数据
user_cf.del("001")?;
```

## 核心组件说明

### `DbContext`
- `initialize(db_path: &str, column_families: &[&str])`：初始化数据库连接
- 单例模式管理数据库实例

### `ColumnFamily<T>`
要求 `T: Keyable`，提供以下操作：
- `get(key: &str) -> Result<Option<T>>`：按键查询
- `set(item: &T) -> Result<()>`：插入/更新数据
- `del(key: &str) -> Result<()>`：删除数据
- `get_all() -> Result<Vec<T>>`：获取列族全部数据
- `count_all() -> Result<usize>`: 获取总的记录数

### `Keyable` trait
数据模型必须实现的特征：
```rust
pub trait Keyable: Serialize + DeserializeOwned {
    /// 获取数据主键
    fn key(&self) -> String;
    
    /// 返回对应的列族名称
    fn column_family() -> &'static str;
}
```


use super::*;
use crate::{ColumnFamily, DbContext, Keyable};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::sync::OnceLock;
use tempfile::TempDir;

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone)]
struct TestUser {
    id: String,
    name: String,
}

impl Keyable for TestUser {
    fn key(&self) -> String {
        self.id.clone()
    }

    fn column_family() -> &'static str {
        "test_users"
    }
}

// 模块级共享临时目录对象
static GLOBAL_TEMP_DIR: OnceLock<TempDir> = OnceLock::new();

fn get_test_tempdir() -> &'static TempDir {
    GLOBAL_TEMP_DIR.get_or_init(|| TempDir::new().expect("Failed to create global temp directory"))
}

#[test]
fn test_crud_operations() -> Result<()> {
    let global_temp = get_test_tempdir();
    let db_path = global_temp.path();
    let column_families = vec!["test_users"];

    DbContext::initialize(db_path, &column_families)?;

    let user_cf = ColumnFamily::<TestUser>::new();

    // Create
    let user1 = TestUser {
        id: "001".to_string(),
        name: "Alice".to_string(),
    };
    user_cf.set(&user1)?;

    // Read single
    let retrieved = user_cf.get("001")?.unwrap();
    assert_eq!(retrieved.name, "Alice");

    // Read all
    let user2 = TestUser {
        id: "002".to_string(),
        name: "Bob".to_string(),
    };
    user_cf.set(&user2)?;

    let all_users = user_cf.get_all()?;
    assert_eq!(all_users.len(), 2);
    assert!(all_users.contains(&user1));
    assert!(all_users.contains(&user2));

    // Count all
    assert_eq!(user_cf.count_all().unwrap(), 2);

    // Update
    let updated_user = TestUser {
        id: "001".to_string(),
        name: "Alicia".to_string(),
    };
    user_cf.set(&updated_user)?;
    assert_eq!(user_cf.get("001")?.unwrap().name, "Alicia");

    // Delete
    user_cf.del("001")?;
    assert!(user_cf.get("001")?.is_none());

    // Verify remaining data
    let remaining = user_cf.get_all()?;
    assert_eq!(remaining.len(), 1);
    assert_eq!(remaining[0], user2);

    assert!(user_cf.get("non_existent")?.is_none());

    Ok(())
}

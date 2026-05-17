use std::time::Duration;

use sea_orm::{ConnectOptions, Database, DatabaseConnection};
use sea_orm_migration::MigratorTrait;
use tauri::AppHandle;

use crate::db::migration::Migrator;
use crate::score::{self, AppError};

pub async fn connect(app: &AppHandle) -> Result<DatabaseConnection, AppError> {
    let path = score::db_path(app)?;
    let path = path
        .to_str()
        .ok_or_else(|| AppError::new("数据库路径包含无效字符"))?
        .replace('\\', "/");
    let url = format!("sqlite://{path}?mode=rwc");

    let mut options = ConnectOptions::new(url);
    options
        .max_connections(5)
        .min_connections(1)
        .connect_timeout(Duration::from_secs(8))
        .acquire_timeout(Duration::from_secs(8))
        .sqlx_logging(false);

    let db = Database::connect(options).await?;
    Migrator::up(&db, None).await?;
    Ok(db)
}

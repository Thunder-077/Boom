use std::fs::create_dir_all;
use std::path::PathBuf;
use std::time::Duration;

use sea_orm::{ConnectOptions, Database, DatabaseConnection};
use sea_orm_migration::MigratorTrait;
use tauri::{AppHandle, Manager};

use crate::db::migration::Migrator;
use crate::score::AppError;

pub async fn connect(app: &AppHandle) -> Result<DatabaseConnection, AppError> {
    let path = db_path(app)?;
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

fn db_path(app: &AppHandle) -> Result<PathBuf, AppError> {
    let mut dir = app
        .path()
        .app_data_dir()
        .map_err(|e| AppError::new(format!("获取应用数据目录失败: {e}")))?;
    create_dir_all(&dir).map_err(|e| AppError::new(format!("创建应用数据目录失败: {e}")))?;
    dir.push("scores.sqlite3");
    Ok(dir)
}

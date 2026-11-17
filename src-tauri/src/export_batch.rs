use crate::score::AppError;
use std::{
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

static NEXT_BATCH: AtomicU64 = AtomicU64::new(1);
static NEXT_VERSION: AtomicU64 = AtomicU64::new(1);

static ACTIVE_ROOTS: std::sync::LazyLock<std::sync::Mutex<std::collections::HashSet<PathBuf>>> =
    std::sync::LazyLock::new(|| std::sync::Mutex::new(std::collections::HashSet::new()));

// 只保存目录键，不持有跨 await 的 MutexGuard；同一输出根目录的整次操作互斥。
pub(crate) struct BatchOperationGuard(PathBuf);

impl BatchOperationGuard {
    pub fn acquire(root: &Path) -> Result<Self, AppError> {
        fs::create_dir_all(root).map_err(|e| AppError::new(e.to_string()))?;
        let key = fs::canonicalize(root).map_err(|e| AppError::new(e.to_string()))?;
        let mut active = ACTIVE_ROOTS
            .lock()
            .map_err(|_| AppError::new("导出操作锁异常"))?;
        if !active.insert(key.clone()) {
            return Err(AppError::new("正在生成或导出本批次，请等待完成后重试"));
        }
        Ok(Self(key))
    }
}

impl Drop for BatchOperationGuard {
    fn drop(&mut self) {
        if let Ok(mut active) = ACTIVE_ROOTS.lock() {
            active.remove(&self.0);
        }
    }
}

pub(crate) struct PreparedExportBatch {
    pub staging_dir: PathBuf,
    pub final_dir: PathBuf,
    backup_dir: PathBuf,
    replace_existing: bool,
}

pub(crate) struct PublishedExportBatch {
    final_dir: PathBuf,
    backup_dir: PathBuf,
    staging_dir: PathBuf,
    had_previous: bool,
}

pub(crate) fn prepare_export_batch(
    root: &Path,
    final_name: &str,
) -> Result<PreparedExportBatch, AppError> {
    if final_name.is_empty()
        || !matches!(
            Path::new(final_name).components().next(),
            Some(std::path::Component::Normal(_))
        )
        || Path::new(final_name).components().count() != 1
        || final_name == "."
        || final_name == ".."
    {
        return Err(AppError::new("导出批次目录名无效"));
    }
    fs::create_dir_all(root).map_err(|e| AppError::new(format!("创建导出根目录失败: {e}")))?;
    let nonce = format!(
        "{}-{}-{}",
        std::process::id(),
        chrono::Utc::now().timestamp_nanos_opt().unwrap_or_default(),
        NEXT_BATCH.fetch_add(1, Ordering::Relaxed)
    );
    let staging_dir = root.join(format!(".staging-{nonce}"));
    fs::create_dir(&staging_dir)
        .map_err(|e| AppError::new(format!("创建临时导出目录失败: {e}")))?;
    Ok(PreparedExportBatch {
        staging_dir,
        final_dir: root.join(final_name),
        backup_dir: root.join(format!(".backup-{nonce}")),
        replace_existing: true,
    })
}

// 每次发布使用独立目录，避免旧目录被资源管理器或其他程序占用。
pub(crate) fn prepare_versioned_batch(
    root: &Path,
    final_name: &str,
) -> Result<PreparedExportBatch, AppError> {
    let timestamp = chrono::Utc::now().timestamp();
    loop {
        let version = NEXT_VERSION.fetch_add(1, Ordering::Relaxed);
        let name = format!("{final_name}-{timestamp}-{version}");
        // 已有路径必须保留；即使另一个进程同时发布，也不能覆盖旧结果。
        match fs::symlink_metadata(root.join(&name)) {
            Ok(_) => continue,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(AppError::new(format!("检查导出目录失败: {error}"))),
        }
        let mut batch = prepare_export_batch(root, &name)?;
        batch.replace_existing = false;
        return Ok(batch);
    }
}

pub(crate) async fn current_batch_dir<C: sea_orm::ConnectionTrait>(
    db: &C,
    root: &Path,
    title: &str,
) -> Result<PathBuf, AppError> {
    let Some(name) = crate::db::repos::export_bundle::current_export_directory(db).await? else {
        return Ok(crate::export_bundle::export_batch_dir(root, title));
    };
    if !matches!(
        Path::new(&name).components().next(),
        Some(std::path::Component::Normal(_))
    ) || Path::new(&name).components().count() != 1
    {
        return Err(AppError::new("导出目录记录无效"));
    }
    Ok(root.join(name))
}

pub(crate) async fn select_prepared_batch<C: sea_orm::ConnectionTrait>(
    db: &C,
    batch: &PreparedExportBatch,
) -> Result<Option<PathBuf>, AppError> {
    let root = batch
        .final_dir
        .parent()
        .ok_or_else(|| AppError::new("导出目录无效"))?;
    let previous = match crate::db::repos::export_bundle::current_export_directory(db).await? {
        Some(name) => {
            let path = Path::new(&name);
            if !matches!(
                path.components().next(),
                Some(std::path::Component::Normal(_))
            ) || path.components().count() != 1
            {
                return Err(AppError::new("导出目录记录无效"));
            }
            Some(root.join(path))
        }
        None => None,
    };
    let name = batch
        .final_dir
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| AppError::new("导出目录名无效"))?;
    crate::db::repos::export_bundle::set_current_export_directory(db, name).await?;
    Ok(previous)
}

// 清理旧版本只是尽力而为；Windows 占用旧文件时不能影响已经提交的新结果。
pub(crate) async fn complete_versioned_batch(
    published: PublishedExportBatch,
    commit: impl std::future::Future<Output = Result<(), sea_orm::DbErr>>,
    previous: Option<PathBuf>,
) -> Result<Vec<String>, AppError> {
    let current = published.final_dir.clone();
    let warnings = complete_published_batch(published, commit).await?;
    if let Some(old) = previous.filter(|path| path != &current) {
        let _ = fs::remove_dir_all(old);
    }
    Ok(warnings)
}

pub(crate) fn publish_prepared_batch(
    batch: &PreparedExportBatch,
) -> Result<PublishedExportBatch, AppError> {
    let had_previous = batch.final_dir.exists();
    if had_previous && !batch.replace_existing {
        return Err(AppError::new("导出目录名称已被占用，请重试"));
    }
    if had_previous {
        if fs::symlink_metadata(&batch.final_dir)
            .map_err(|e| AppError::new(e.to_string()))?
            .file_type()
            .is_symlink()
        {
            return Err(AppError::new("导出目标不能是符号链接"));
        }
        if !batch.final_dir.is_dir() {
            return Err(AppError::new("导出目标被文件占用，原结果未更改"));
        }
        fs::rename(&batch.final_dir, &batch.backup_dir)
            .map_err(|e| AppError::new(format!("备份旧导出目录失败，原结果未更改: {e}")))?;
    }
    if let Err(error) = fs::rename(&batch.staging_dir, &batch.final_dir) {
        if had_previous {
            fs::rename(&batch.backup_dir, &batch.final_dir).map_err(|rollback| {
                AppError::new(format!(
                    "发布失败: {error}；恢复失败: {rollback}；备份保留在 {}",
                    batch.backup_dir.display()
                ))
            })?;
        }
        return Err(AppError::new(format!("发布导出目录失败: {error}")));
    }
    Ok(PublishedExportBatch {
        final_dir: batch.final_dir.clone(),
        backup_dir: batch.backup_dir.clone(),
        staging_dir: batch.staging_dir.clone(),
        had_previous,
    })
}

impl PublishedExportBatch {
    pub fn rollback(&mut self) -> Result<(), AppError> {
        // 失败的新批次改回临时目录，绝不递归删除最终目录或吞掉恢复错误。
        fs::rename(&self.final_dir, &self.staging_dir).map_err(|e| {
            AppError::new(format!(
                "撤回新导出失败: {e}；旧备份: {}",
                self.backup_dir.display()
            ))
        })?;
        if self.had_previous {
            fs::rename(&self.backup_dir, &self.final_dir).map_err(|e| {
                AppError::new(format!(
                    "恢复旧导出失败: {e}；备份保留在 {}",
                    self.backup_dir.display()
                ))
            })?;
        }
        Ok(())
    }

    pub fn finish(self) -> Result<(), AppError> {
        if self.had_previous {
            fs::remove_dir_all(&self.backup_dir).map_err(|e| {
                AppError::new(format!(
                    "新批次已发布，旧备份清理失败: {e}；备份: {}",
                    self.backup_dir.display()
                ))
            })?;
        }
        Ok(())
    }
}

#[cfg(test)]
fn copy_student_artifacts(source: &Path, target: &Path) -> Result<(), AppError> {
    if !source.is_dir() {
        return Err(AppError::new(
            "本次考场资料目录不存在，请先重新生成考场安排",
        ));
    }
    for entry in fs::read_dir(source).map_err(|e| AppError::new(e.to_string()))? {
        let entry = entry.map_err(|e| AppError::new(e.to_string()))?;
        let kind = entry
            .file_type()
            .map_err(|e| AppError::new(e.to_string()))?;
        if kind.is_symlink() {
            return Err(AppError::new("导出目录不能包含符号链接"));
        }
        let dest = target.join(entry.file_name());
        if kind.is_dir() {
            fs::create_dir(&dest).map_err(|e| AppError::new(e.to_string()))?;
            copy_student_artifacts(&entry.path(), &dest)?;
        } else {
            fs::copy(entry.path(), &dest)
                .map_err(|e| AppError::new(format!("保留考场资料失败: {e}")))?;
        }
    }
    Ok(())
}

pub(crate) async fn complete_published_batch(
    mut published: PublishedExportBatch,
    commit: impl std::future::Future<Output = Result<(), sea_orm::DbErr>>,
) -> Result<Vec<String>, AppError> {
    if let Err(error) = commit.await {
        published.rollback()?;
        return Err(error.into());
    }
    // 提交是成功边界；之后的备份清理只能产生警告，不能把已完成的发布报成失败。
    Ok(match published.finish() {
        Ok(()) => Vec::new(),
        Err(error) => vec![error.to_string()],
    })
}

pub(crate) fn write_batch_identity(
    dir: &Path,
    generated_at: &str,
    revision: &(Option<String>, Option<String>),
) -> Result<(), AppError> {
    let bytes =
        serde_json::to_vec(&(generated_at, revision)).map_err(|e| AppError::new(e.to_string()))?;
    fs::write(dir.join(".boom-batch.json"), bytes)
        .map_err(|e| AppError::new(format!("写入批次标识失败: {e}")))
}

pub(crate) fn verify_batch_identity(
    dir: &Path,
    generated_at: &str,
    score_revision: &Option<String>,
) -> Result<(), AppError> {
    let bytes = fs::read(dir.join(".boom-batch.json"))
        .map_err(|_| AppError::new("考场资料与当前快照不匹配，请重新生成考场安排"))?;
    let identity: (String, (Option<String>, Option<String>)) = serde_json::from_slice(&bytes)
        .map_err(|_| AppError::new("考场资料批次标识无效，请重新生成"))?;
    if identity.0 != generated_at || identity.1 .0 != *score_revision {
        return Err(AppError::new("考场资料已过期，请重新生成考场安排"));
    }
    Ok(())
}

pub(crate) fn write_staff_source(
    dir: &Path,
    revision: &(Option<String>, Option<String>),
    staff_revision: &str,
) -> Result<(), AppError> {
    fs::write(
        dir.join(".boom-staff-source.json"),
        serde_json::to_vec(&(revision, staff_revision))
            .map_err(|e| AppError::new(e.to_string()))?,
    )
    .map_err(|e| AppError::new(format!("写入监考源版本失败: {e}")))
}

pub(crate) fn verify_staff_source(
    dir: &Path,
    revision: &(Option<String>, Option<String>),
    staff_revision: &str,
) -> Result<(), AppError> {
    let bytes = fs::read(dir.join(".boom-staff-source.json"))
        .map_err(|_| AppError::new("监考资料缺少源版本，请重新分配监考教师"))?;
    let saved: ((Option<String>, Option<String>), String) =
        serde_json::from_slice(&bytes).map_err(|_| AppError::new("监考源版本无效，请重新分配"))?;
    if saved.0 != *revision || saved.1 != staff_revision {
        return Err(AppError::new("导入数据已变化，请重新分配监考教师"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        db::repos::export_bundle::{ExamRow, SessionInfo, StudentBase},
        export_bundle::GradeBundleInput,
        export_invigilation::{
            AccountingConfig, AccountingTeacherRow, GradeStaffExportInput, TaskExportRow,
        },
        score::Subject,
    };
    use std::collections::{HashMap, HashSet};

    struct TestDirectory(PathBuf);
    impl TestDirectory {
        fn new() -> Self {
            let path = std::env::temp_dir().join(format!(
                "boom-batch-test-{}-{}",
                std::process::id(),
                NEXT_BATCH.fetch_add(1, Ordering::Relaxed)
            ));
            fs::create_dir_all(&path).unwrap();
            Self(path)
        }
        fn path(&self) -> &Path {
            &self.0
        }
    }
    impl Drop for TestDirectory {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    #[cfg(windows)]
    #[test]
    fn versioned_publication_succeeds_with_old_directory_and_file_locked() {
        use sea_orm::{Database, TransactionTrait};
        use sea_orm_migration::MigratorTrait;
        use std::os::windows::fs::OpenOptionsExt;
        tauri::async_runtime::block_on(async {
            let root = TestDirectory::new();
            let db = Database::connect("sqlite::memory:").await.unwrap();
            crate::db::migration::Migrator::up(&db, None).await.unwrap();
            let legacy = crate::export_bundle::export_batch_dir(root.path(), "考试");
            fs::create_dir(&legacy).unwrap();
            fs::write(legacy.join("old.xlsx"), b"old").unwrap();
            assert_eq!(
                current_batch_dir(&db, root.path(), "考试").await.unwrap(),
                legacy
            );
            let locked_dir = fs::OpenOptions::new()
                .read(true)
                .share_mode(3)
                .custom_flags(0x02000000)
                .open(&legacy)
                .unwrap();
            let locked_file = fs::OpenOptions::new()
                .read(true)
                .share_mode(3)
                .open(legacy.join("old.xlsx"))
                .unwrap();
            let batch = prepare_versioned_batch(root.path(), "考试考场安排").unwrap();
            fs::write(batch.staging_dir.join("new.xlsx"), b"new").unwrap();
            let tx = db.begin().await.unwrap();
            select_prepared_batch(&tx, &batch).await.unwrap();
            let published = publish_prepared_batch(&batch).unwrap();
            complete_published_batch(published, tx.commit())
                .await
                .unwrap();
            assert_eq!(
                current_batch_dir(&db, root.path(), "考试").await.unwrap(),
                batch.final_dir
            );
            assert_eq!(fs::read(batch.final_dir.join("new.xlsx")).unwrap(), b"new");
            drop(locked_file);
            drop(locked_dir);
        });
    }

    #[cfg(windows)]
    #[test]
    fn repeated_publication_ignores_locked_old_version_cleanup() {
        use sea_orm::{Database, TransactionTrait};
        use sea_orm_migration::MigratorTrait;
        use std::os::windows::fs::OpenOptionsExt;
        tauri::async_runtime::block_on(async {
            let root = TestDirectory::new();
            let db = Database::connect("sqlite::memory:").await.unwrap();
            crate::db::migration::Migrator::up(&db, None).await.unwrap();
            let first = prepare_versioned_batch(root.path(), "batch").unwrap();
            fs::write(first.staging_dir.join("old.xlsx"), b"old").unwrap();
            let tx = db.begin().await.unwrap();
            let previous = select_prepared_batch(&tx, &first).await.unwrap();
            let published = publish_prepared_batch(&first).unwrap();
            complete_versioned_batch(published, tx.commit(), previous)
                .await
                .unwrap();
            let locked = fs::OpenOptions::new()
                .read(true)
                .share_mode(3)
                .open(first.final_dir.join("old.xlsx"))
                .unwrap();
            let second = prepare_versioned_batch(root.path(), "batch").unwrap();
            fs::write(second.staging_dir.join("new.xlsx"), b"new").unwrap();
            let tx = db.begin().await.unwrap();
            let previous = select_prepared_batch(&tx, &second).await.unwrap();
            let published = publish_prepared_batch(&second).unwrap();
            complete_versioned_batch(published, tx.commit(), previous)
                .await
                .unwrap();
            assert_eq!(
                current_batch_dir(&db, root.path(), "考试").await.unwrap(),
                second.final_dir
            );
            assert_eq!(fs::read(second.final_dir.join("new.xlsx")).unwrap(), b"new");
            drop(locked);
        });
    }

    #[test]
    fn failed_transaction_keeps_current_directory() {
        use sea_orm::{Database, TransactionTrait};
        use sea_orm_migration::MigratorTrait;
        tauri::async_runtime::block_on(async {
            let root = TestDirectory::new();
            let db = Database::connect("sqlite::memory:").await.unwrap();
            crate::db::migration::Migrator::up(&db, None).await.unwrap();
            let batch = prepare_versioned_batch(root.path(), "batch").unwrap();
            let tx = db.begin().await.unwrap();
            select_prepared_batch(&tx, &batch).await.unwrap();
            let published = publish_prepared_batch(&batch).unwrap();
            tx.rollback().await.unwrap();
            let result = complete_published_batch(published, async {
                Err(sea_orm::DbErr::Custom("injected commit failure".into()))
            })
            .await;
            assert!(result.is_err());
            assert_eq!(
                current_batch_dir(&db, root.path(), "考试").await.unwrap(),
                crate::export_bundle::export_batch_dir(root.path(), "考试")
            );
            assert!(!batch.final_dir.exists());
        });
    }

    #[test]
    fn versioned_directory_uses_seconds_timestamp_and_does_not_replace_existing() {
        let root = TestDirectory::new();
        let before = chrono::Utc::now().timestamp();
        let batch = prepare_versioned_batch(root.path(), "期中考试考场安排").unwrap();
        let after = chrono::Utc::now().timestamp();
        let name = batch.final_dir.file_name().unwrap().to_str().unwrap();
        let suffix = name.strip_prefix("期中考试考场安排-").unwrap();
        let (timestamp, version) = suffix.rsplit_once('-').unwrap();
        let timestamp = timestamp.parse::<i64>().unwrap();
        assert!((before..=after).contains(&timestamp));
        assert!(version.parse::<u64>().unwrap() > 0);
        fs::create_dir(&batch.final_dir).unwrap();
        fs::write(batch.final_dir.join("old"), b"old").unwrap();
        assert!(publish_prepared_batch(&batch).is_err());
        assert_eq!(fs::read(batch.final_dir.join("old")).unwrap(), b"old");
    }

    #[test]
    fn test_batch_operations_serialize_same_root_only() {
        let root = TestDirectory::new();
        let other = TestDirectory::new();
        let guard = BatchOperationGuard::acquire(root.path()).unwrap();
        assert!(BatchOperationGuard::acquire(&root.path().join(".")).is_err());
        assert!(BatchOperationGuard::acquire(other.path()).is_ok());
        drop(guard);
        assert!(BatchOperationGuard::acquire(root.path()).is_ok());
    }

    #[cfg(windows)]
    #[test]
    fn test_committed_batch_cleanup_failure_is_success_with_warning() {
        use std::os::windows::fs::OpenOptionsExt;
        let root = TestDirectory::new();
        fs::create_dir(root.path().join("batch")).unwrap();
        fs::write(root.path().join("batch/old"), b"old").unwrap();
        let batch = prepare_export_batch(root.path(), "batch").unwrap();
        fs::write(batch.staging_dir.join("new"), b"new").unwrap();
        let published = publish_prepared_batch(&batch).unwrap();
        // Windows 文件占用允许读写但禁止删除，真实复现提交后的清理失败。
        let locked = fs::OpenOptions::new()
            .read(true)
            .share_mode(3)
            .open(batch.backup_dir.join("old"))
            .unwrap();
        let result =
            tauri::async_runtime::block_on(complete_published_batch(published, async { Ok(()) }));
        assert!(result.is_ok(), "数据库已提交，清理失败应返回成功及警告");
        let warnings = result.unwrap();
        assert_eq!(warnings.len(), 1);
        assert!(warnings[0].contains(&batch.backup_dir.display().to_string()));
        assert_eq!(fs::read(batch.final_dir.join("new")).unwrap(), b"new");
        assert_eq!(fs::read(batch.backup_dir.join("old")).unwrap(), b"old");
        drop(locked);
    }

    #[test]
    fn test_failed_commit_restores_previous_export() {
        let root = TestDirectory::new();
        fs::create_dir(root.path().join("batch")).unwrap();
        fs::write(root.path().join("batch/old"), b"old").unwrap();
        let batch = prepare_export_batch(root.path(), "batch").unwrap();
        fs::write(batch.staging_dir.join("new"), b"new").unwrap();
        let published = publish_prepared_batch(&batch).unwrap();
        let result = tauri::async_runtime::block_on(complete_published_batch(published, async {
            Err(sea_orm::DbErr::Custom("injected commit failure".into()))
        }));
        assert!(result.is_err());
        assert_eq!(fs::read(batch.final_dir.join("old")).unwrap(), b"old");
        assert!(!batch.final_dir.join("new").exists());
    }

    #[cfg(windows)]
    #[test]
    fn test_locked_previous_directory_preserves_completed_batch() {
        use std::os::windows::fs::OpenOptionsExt;
        let root = TestDirectory::new();
        fs::create_dir(root.path().join("batch")).unwrap();
        fs::write(root.path().join("batch/old"), b"old").unwrap();
        let batch = prepare_export_batch(root.path(), "batch").unwrap();
        fs::write(batch.staging_dir.join("new"), b"new").unwrap();
        // 打开目录但不共享删除权限，触发 Windows 真实的 rename 失败。
        let locked = fs::OpenOptions::new()
            .read(true)
            .share_mode(3)
            .custom_flags(0x02000000)
            .open(&batch.final_dir)
            .unwrap();
        assert!(publish_prepared_batch(&batch).is_err());
        assert_eq!(fs::read(batch.final_dir.join("old")).unwrap(), b"old");
        assert_eq!(fs::read(batch.staging_dir.join("new")).unwrap(), b"new");
        assert!(!batch.backup_dir.exists());
        drop(locked);
    }

    #[test]
    fn test_export_rejects_duplicate_room_cells() {
        let row = TaskExportRow {
            session_id: Some(1),
            space_id: Some(1),
            task_source: "exam".into(),
            role: "exam_room_invigilator".into(),
            grade_name: "高三".into(),
            subject: Subject::Math,
            space_name: "高三11场".into(),
            floor: "一楼".into(),
            start_at: "2026-11-18T08:00".into(),
            end_at: "2026-11-18T10:00".into(),
            start_ts: 0,
            duration_minutes: 120,
            recommended_self_study_topic_label: None,
            teacher_name: None,
        };
        assert!(
            crate::export_invigilation::validate_room_cells(&[row.clone(), row.clone()]).is_ok()
        );
        let duplicate = TaskExportRow {
            space_id: Some(2),
            space_name: "高三11班".into(),
            ..row.clone()
        };
        assert!(crate::export_invigilation::validate_room_cells(&[row, duplicate]).is_err());
    }

    #[test]
    fn test_export_database_failure_rolls_back_published_batch() {
        use sea_orm::{
            ActiveModelTrait, ActiveValue::Set, Database, EntityTrait, TransactionTrait,
        };
        use sea_orm_migration::MigratorTrait;
        tauri::async_runtime::block_on(async {
            let db = Database::connect("sqlite::memory:").await.unwrap();
            crate::db::migration::Migrator::up(&db, None).await.unwrap();
            crate::entity::latest_teachers_v2::ActiveModel {
                teacher_name: Set("原数据".into()),
                is_middle_manager: Set(0),
                ..Default::default()
            }
            .insert(&db)
            .await
            .unwrap();
            let root = std::env::temp_dir().join(format!(
                "boom-export-{}",
                chrono::Utc::now().timestamp_nanos_opt().unwrap()
            ));
            fs::create_dir_all(root.join("batch")).unwrap();
            fs::write(root.join("batch/old"), b"old").unwrap();
            let batch = prepare_export_batch(&root, "batch").unwrap();
            fs::write(batch.staging_dir.join("new"), b"new").unwrap();
            let tx = db.begin().await.unwrap();
            crate::entity::latest_teachers_v2::Entity::delete_many()
                .exec(&tx)
                .await
                .unwrap();
            let mut published = publish_prepared_batch(&batch).unwrap();
            // 模拟写入后提交失败：应用服务走相同的显式双回退路径。
            tx.rollback().await.unwrap();
            published.rollback().unwrap();
            assert_eq!(
                crate::entity::latest_teachers_v2::Entity::find()
                    .all(&db)
                    .await
                    .unwrap()[0]
                    .teacher_name,
                "原数据"
            );
            assert_eq!(fs::read(root.join("batch/old")).unwrap(), b"old");
            fs::remove_dir_all(root).unwrap();
        });
    }

    #[test]
    fn test_export_separates_grade_workbooks_and_group_labels() {
        use calamine::{Reader, Xlsx};
        let root = std::env::temp_dir().join(format!(
            "boom-export-{}",
            chrono::Utc::now().timestamp_nanos_opt().unwrap()
        ));
        let batch = prepare_export_batch(&root, "batch").unwrap();
        let mut bundles = Vec::new();
        let mut staff = Vec::new();
        let mut counts = HashMap::new();
        for (index, grade) in ["高一", "高三"].into_iter().enumerate() {
            let id = index as i64 + 1;
            let name = format!("{grade}学生");
            let teacher_name = format!("{grade}教师");
            bundles.push(GradeBundleInput {
                grade: grade.into(),
                sessions: HashMap::from([(
                    Subject::Math,
                    SessionInfo {
                        subject_label: "数学",
                        start_at: Some("2026-11-18T08:00".into()),
                        end_at: Some("2026-11-18T10:00".into()),
                        start_ts: Some(1794988800000),
                    },
                )]),
                rows: vec![ExamRow {
                    admission_no: id.to_string(),
                    student_name: name.clone(),
                    class_name: format!("{grade}1班"),
                    subject: Subject::Math,
                    subject_label: "数学",
                    space_name: format!("{grade}1场"),
                    seat_no: 1,
                }],
                students: vec![StudentBase {
                    admission_no: id.to_string(),
                    student_name: name,
                    class_name: format!("{grade}1班"),
                    class_rank: 1,
                    exam_group: if grade == "高三" {
                        "comprehensive".into()
                    } else {
                        "regular".into()
                    },
                }],
            });
            staff.push(GradeStaffExportInput {
                grade: grade.into(),
                rows: vec![TaskExportRow {
                    session_id: Some(id),
                    space_id: Some(id),
                    task_source: "exam".into(),
                    role: "exam_room_invigilator".into(),
                    grade_name: grade.into(),
                    subject: Subject::Math,
                    space_name: format!("{grade}1场"),
                    floor: "一楼".into(),
                    start_at: "2026-11-18T08:00".into(),
                    end_at: "2026-11-18T10:00".into(),
                    start_ts: 1794988800000,
                    duration_minutes: 120,
                    recommended_self_study_topic_label: None,
                    teacher_name: Some(teacher_name.clone()),
                }],
                teachers: vec![AccountingTeacherRow {
                    teacher_id: id,
                    teacher_name,
                    group_subject: "数学".into(),
                    is_middle_manager: false,
                }],
                config: AccountingConfig {
                    indoor_allowance_per_minute: 0.5,
                    outdoor_allowance_per_minute: 0.3,
                    middle_manager_default_enabled: false,
                    middle_manager_exception_teacher_ids: HashSet::new(),
                    total_exam_and_self_study_minutes: 120,
                },
            });
            counts.insert((id, id), 1);
        }
        let high3 = bundles.last_mut().unwrap();
        high3.students.push(StudentBase {
            admission_no: "3".into(),
            student_name: "高三普通学生".into(),
            class_name: "高三2班".into(),
            class_rank: 1,
            exam_group: "regular".into(),
        });
        high3.rows.push(ExamRow {
            admission_no: "3".into(),
            student_name: "高三普通学生".into(),
            class_name: "高三2班".into(),
            subject: Subject::Math,
            subject_label: "数学",
            space_name: "高三2场".into(),
            seat_no: 1,
        });
        crate::export_bundle::write_exam_bundle_data(
            &batch.staging_dir,
            "测试考试",
            &[],
            &bundles,
            |_, _, _| {},
        )
        .unwrap();
        let student_file = batch.staging_dir.join("高三/考场分配结果汇总.xlsx");
        let original = fs::read(&student_file).unwrap();
        let mut published = publish_prepared_batch(&batch).unwrap();
        published.rollback().unwrap();
        let staff_batch = prepare_export_batch(&root, "next").unwrap();
        copy_student_artifacts(&batch.staging_dir, &staff_batch.staging_dir).unwrap();
        let files = crate::export_invigilation::write_grade_staff_workbooks(
            &staff_batch.staging_dir,
            "测试考试",
            staff,
            &counts,
        )
        .unwrap();
        assert_eq!(files.len(), 2);
        assert_eq!(
            fs::read(staff_batch.staging_dir.join("高三/考场分配结果汇总.xlsx")).unwrap(),
            original
        );
        assert!(!staff_batch.staging_dir.join("测试考试监考表.xlsx").exists());
        for file in files {
            let mut book: Xlsx<_> = calamine::open_workbook(&file.file_path).unwrap();
            let text = book
                .worksheets()
                .into_iter()
                .flat_map(|(_, sheet)| {
                    sheet
                        .cells()
                        .map(|(_, _, cell)| cell.to_string())
                        .collect::<Vec<_>>()
                })
                .collect::<Vec<_>>()
                .join("|");
            assert!(text.contains(&format!("{}教师", file.grade_name)));
            let other = if file.grade_name == "高一" {
                "高三"
            } else {
                "高一"
            };
            assert!(!text.contains(&format!("{other}教师")));
        }
        let mut summary: Xlsx<_> = calamine::open_workbook(&student_file).unwrap();
        assert!(summary.worksheets().into_iter().any(|(_, sheet)| sheet
            .cells()
            .any(|(_, _, cell)| cell.to_string() == "综评班")));
        assert!(summary.worksheets().into_iter().any(|(_, sheet)| sheet
            .cells()
            .any(|(_, _, cell)| cell.to_string() == "普通班")));
        fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn test_export_publish_rollback_preserves_previous_and_siblings() {
        let root = std::env::temp_dir().join(format!(
            "boom-export-{}",
            chrono::Utc::now().timestamp_nanos_opt().unwrap()
        ));
        fs::create_dir_all(root.join("batch")).unwrap();
        fs::write(root.join("batch/old"), b"old").unwrap();
        fs::write(root.join("sibling"), b"keep").unwrap();
        let batch = prepare_export_batch(&root, "batch").unwrap();
        fs::write(batch.staging_dir.join("new"), b"new").unwrap();
        let mut published = publish_prepared_batch(&batch).unwrap();
        published.rollback().unwrap();
        assert_eq!(fs::read(root.join("batch/old")).unwrap(), b"old");
        assert_eq!(fs::read(root.join("sibling")).unwrap(), b"keep");
        fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn test_export_occupied_target_and_invalid_names_preserve_files() {
        let root = std::env::temp_dir().join(format!(
            "boom-export-{}",
            chrono::Utc::now().timestamp_nanos_opt().unwrap()
        ));
        let batch = prepare_export_batch(&root, "batch").unwrap();
        fs::write(&batch.final_dir, b"occupied").unwrap();
        assert!(publish_prepared_batch(&batch).is_err());
        assert_eq!(fs::read(&batch.final_dir).unwrap(), b"occupied");
        assert!(prepare_export_batch(&root, "../escape").is_err());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn test_export_later_grade_write_failure_preserves_completed_batch() {
        let root = std::env::temp_dir().join(format!(
            "boom-export-{}",
            chrono::Utc::now().timestamp_nanos_opt().unwrap()
        ));
        fs::create_dir_all(root.join("batch")).unwrap();
        fs::write(root.join("batch/old"), b"old").unwrap();
        let batch = prepare_export_batch(&root, "batch").unwrap();
        // 第二个年级目录被文件占用，前一个年级的临时文件不能成为部分发布结果。
        fs::write(batch.staging_dir.join("高三"), b"occupied").unwrap();
        let inputs: Vec<_> = ["高一", "高三"]
            .into_iter()
            .map(|grade| GradeBundleInput {
                grade: grade.into(),
                rows: vec![],
                students: vec![],
                sessions: HashMap::new(),
            })
            .collect();
        assert!(crate::export_bundle::write_exam_bundle_data(
            &batch.staging_dir,
            "测试",
            &[],
            &inputs,
            |_, _, _| {}
        )
        .is_err());
        assert!(batch
            .staging_dir
            .join("高一/考场分配结果汇总.xlsx")
            .exists());
        assert_eq!(fs::read(root.join("batch/old")).unwrap(), b"old");
        fs::remove_dir_all(root).unwrap();
    }
}

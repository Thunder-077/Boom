use crate::db::repos::exam_allocation::{
    SessionInsertRow, SpaceInsertRow, StudentAllocationInsertRow,
};
use crate::score::AppError;
use std::collections::{HashMap, HashSet};

/// 草稿ID只在本轮内使用，持久化时统一映射真实ID，计算失败不触碰旧快照。
#[derive(Debug, Default, Clone)]
pub(crate) struct ExamPlanDraft {
    pub sessions: Vec<SessionInsertRow>,
    pub spaces: Vec<SpaceInsertRow>,
    pub allocations: Vec<StudentAllocationInsertRow>,
    pub class_groups: HashMap<String, String>,
    pub max_capacities: HashMap<String, i64>,
    pub windows: HashMap<i64, (i64, i64)>,
}

impl ExamPlanDraft {
    pub fn add_session(&mut self, row: SessionInsertRow) -> i64 {
        self.sessions.push(row);
        self.sessions.len() as i64
    }
    pub fn add_space(&mut self, row: SpaceInsertRow) -> i64 {
        self.spaces.push(row);
        self.spaces.len() as i64
    }
}

pub(crate) fn validate_exam_plan_draft(draft: &ExamPlanDraft) -> Result<(), AppError> {
    let mut names = HashSet::new();
    for space in &draft.spaces {
        let name = space
            .space_name
            .strip_suffix('班')
            .map(|s| format!("{s}场"))
            .unwrap_or_else(|| space.space_name.clone());
        if !names.insert((space.session_id, name.clone())) {
            return Err(AppError::new(format!(
                "{}考场{name}同名冲突",
                space.grade_name
            )));
        }
        if space
            .capacity
            .is_some_and(|cap| cap > *draft.max_capacities.get(&space.grade_name).unwrap_or(&200))
        {
            return Err(AppError::new(format!("考场{name}人数超过年级上限")));
        }
    }
    for (index, left) in draft.spaces.iter().enumerate() {
        for right in &draft.spaces[index + 1..] {
            let normalize = |s: &str| {
                s.strip_suffix('班')
                    .map(|s| format!("{s}场"))
                    .unwrap_or_else(|| s.to_string())
            };
            if left.session_id != right.session_id
                && normalize(&left.space_name) == normalize(&right.space_name)
            {
                if let (Some(a), Some(b)) = (
                    draft.windows.get(&left.session_id),
                    draft.windows.get(&right.session_id),
                ) {
                    if a.0 < b.1 && b.0 < a.1 {
                        return Err(AppError::new(format!(
                            "考场{}在重叠时间重复占用",
                            left.space_name
                        )));
                    }
                }
            }
        }
    }
    let mut seats = HashSet::new();
    let mut students = HashSet::new();
    let mut counts = HashMap::new();
    for allocation in &draft.allocations {
        if !students.insert((allocation.session_id, &allocation.admission_no)) {
            return Err(AppError::new("同一场次学生重复分配"));
        }
        let space_id = allocation
            .space_id
            .ok_or_else(|| AppError::new("学生未关联空间"))?;
        let space = draft
            .spaces
            .get((space_id - 1) as usize)
            .ok_or_else(|| AppError::new("学生空间索引无效"))?;
        if space.session_id != allocation.session_id {
            return Err(AppError::new("学生空间与场次不一致"));
        }
        if allocation.allocation_type == "exam" {
            if !allocation
                .seat_no
                .is_some_and(|seat| seat > 0 && seat <= space.capacity.unwrap_or(0))
            {
                return Err(AppError::new("考场座位超出容量"));
            }
            if space.exam_group.as_ref() != draft.class_groups.get(&allocation.class_name) {
                return Err(AppError::new("考场包含不同分组学生"));
            }
            if !seats.insert((space_id, allocation.seat_no)) {
                return Err(AppError::new("考场座位重复"));
            }
            *counts.entry(space_id).or_insert(0_i64) += 1;
        }
    }
    for (index, space) in draft.spaces.iter().enumerate() {
        if space.space_type == "exam_room"
            && space.capacity != Some(*counts.get(&(index as i64 + 1)).unwrap_or(&0))
        {
            return Err(AppError::new("考场容量与实际分配人数不一致"));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn space(session: i64, name: &str) -> SpaceInsertRow {
        SpaceInsertRow {
            session_id: session,
            space_type: "exam_room".into(),
            space_source: "exam_room".into(),
            grade_name: "高三".into(),
            subject: "chinese".into(),
            space_name: name.into(),
            original_class_name: None,
            self_study_topic_kind: None,
            self_study_topic_subjects_json: None,
            self_study_topic_label: None,
            building: "教学楼".into(),
            floor: "一楼".into(),
            capacity: Some(1),
            sort_index: 1,
            exam_group: Some("regular".into()),
        }
    }
    fn valid_draft() -> ExamPlanDraft {
        let mut draft = ExamPlanDraft::default();
        draft.spaces.push(space(1, "高三11场"));
        draft
            .class_groups
            .insert("高三1班".into(), "regular".into());
        draft.max_capacities.insert("高三".into(), 30);
        draft.allocations.push(StudentAllocationInsertRow {
            session_id: 1,
            admission_no: "anonymous-1".into(),
            student_name: "匿名学生".into(),
            class_name: "高三1班".into(),
            allocation_type: "exam".into(),
            space_id: Some(1),
            seat_no: Some(1),
            subject_score: Some(10.0),
        });
        draft
    }
    #[test]
    fn test_conflicting_room_names_fail_validation() {
        let mut draft = valid_draft();
        validate_exam_plan_draft(&draft).unwrap();
        draft.spaces.push(space(1, "高三11场"));
        assert!(validate_exam_plan_draft(&draft)
            .unwrap_err()
            .to_string()
            .contains("同名"));
    }
    #[test]
    fn test_draft_checks_groups_counts_and_overlapping_windows() {
        let mut draft = valid_draft();
        draft.allocations[0].class_name = "高三11班".into();
        draft
            .class_groups
            .insert("高三11班".into(), "comprehensive".into());
        assert!(validate_exam_plan_draft(&draft)
            .unwrap_err()
            .to_string()
            .contains("分组"));
        let mut draft = valid_draft();
        draft.spaces[0].capacity = Some(44);
        assert!(validate_exam_plan_draft(&draft)
            .unwrap_err()
            .to_string()
            .contains("上限"));
        let mut draft = valid_draft();
        draft.spaces.push(space(2, "高三11场"));
        draft.windows.insert(1, (100, 200));
        draft.windows.insert(2, (150, 250));
        assert!(validate_exam_plan_draft(&draft)
            .unwrap_err()
            .to_string()
            .contains("重叠"));
    }
    #[test]
    fn test_invalid_batch_preserves_previous_snapshot() {
        tauri::async_runtime::block_on(async {
            use sea_orm::{Database, EntityTrait, TransactionTrait};
            use sea_orm_migration::MigratorTrait;
            let db = Database::connect("sqlite::memory:").await.unwrap();
            crate::db::migration::Migrator::up(&db, None).await.unwrap();
            let tx = db.begin().await.unwrap();
            let id = crate::db::repos::exam_allocation::insert_session(
                &tx,
                SessionInsertRow {
                    grade_name: "高一".into(),
                    subject: "math".into(),
                    is_foreign_group: 0,
                    foreign_order: None,
                    participant_count: 1,
                    exam_room_count: 1,
                    self_study_room_count: 0,
                },
            )
            .await
            .unwrap();
            tx.commit().await.unwrap();
            let mut invalid = valid_draft();
            invalid.spaces.push(space(1, "高三11场"));
            let tx = db.begin().await.unwrap();
            assert!(
                crate::db::repos::exam_allocation::replace_plan_snapshot(&tx, &invalid)
                    .await
                    .is_err()
            );
            tx.rollback().await.unwrap();
            let rows = crate::entity::latest_exam_plan_sessions::Entity::find()
                .all(&db)
                .await
                .unwrap();
            assert_eq!(rows.len(), 1);
            assert_eq!(rows[0].id, id);
            assert_eq!(rows[0].grade_name, "高一");
        });
    }
}

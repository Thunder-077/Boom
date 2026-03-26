-- =============================================================================
-- Academic Administration System - 统一数据库 Schema
-- 所有表结构集中定义于此，项目启动时自动执行
-- =============================================================================

-- ---------------------------------------------------------------------------
-- 成绩模块 (score)
-- ---------------------------------------------------------------------------

CREATE TABLE IF NOT EXISTS latest_import_meta (
    id INTEGER PRIMARY KEY,
    imported_at TEXT NOT NULL,
    source_file TEXT NOT NULL,
    row_count INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS latest_student_scores (
    admission_no TEXT PRIMARY KEY,
    class_name TEXT NOT NULL,
    grade_name TEXT NOT NULL,
    student_name TEXT NOT NULL,
    subject_combination TEXT NOT NULL DEFAULT '',
    language TEXT NOT NULL DEFAULT '',
    total_score REAL NOT NULL,
    class_rank INTEGER NOT NULL,
    grade_rank INTEGER NOT NULL,
    selected_subject_count INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS latest_subject_scores (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    admission_no TEXT NOT NULL,
    subject TEXT NOT NULL,
    score REAL,
    is_selected INTEGER NOT NULL,
    is_absent INTEGER NOT NULL,
    FOREIGN KEY(admission_no) REFERENCES latest_student_scores(admission_no)
);

CREATE INDEX IF NOT EXISTS idx_latest_student_class_name ON latest_student_scores(class_name);
CREATE INDEX IF NOT EXISTS idx_latest_student_grade_name ON latest_student_scores(grade_name);
CREATE INDEX IF NOT EXISTS idx_latest_student_name ON latest_student_scores(student_name);
CREATE INDEX IF NOT EXISTS idx_latest_student_admission ON latest_student_scores(admission_no);
CREATE INDEX IF NOT EXISTS idx_latest_subject_admission ON latest_subject_scores(admission_no);

-- ---------------------------------------------------------------------------
-- 教师模块 (teacher)
-- ---------------------------------------------------------------------------

CREATE TABLE IF NOT EXISTS latest_teacher_import_meta (
    id INTEGER PRIMARY KEY,
    imported_at TEXT NOT NULL,
    source_file TEXT NOT NULL,
    row_count INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS latest_teachers_v2 (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    teacher_name TEXT NOT NULL UNIQUE,
    remark TEXT,
    is_middle_manager INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS latest_teacher_homerooms_v2 (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    teacher_id INTEGER NOT NULL,
    class_name TEXT NOT NULL,
    UNIQUE(teacher_id, class_name),
    FOREIGN KEY(teacher_id) REFERENCES latest_teachers_v2(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS latest_teacher_assignments_v2 (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    teacher_id INTEGER NOT NULL,
    subject TEXT NOT NULL,
    class_name TEXT NOT NULL,
    UNIQUE(teacher_id, subject, class_name),
    FOREIGN KEY(teacher_id) REFERENCES latest_teachers_v2(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_latest_teachers_v2_name ON latest_teachers_v2(teacher_name);
CREATE INDEX IF NOT EXISTS idx_latest_teacher_assignments_v2_teacher_id ON latest_teacher_assignments_v2(teacher_id);
CREATE INDEX IF NOT EXISTS idx_latest_teacher_assignments_v2_subject ON latest_teacher_assignments_v2(subject);
CREATE INDEX IF NOT EXISTS idx_latest_teacher_assignments_v2_class_name ON latest_teacher_assignments_v2(class_name);
CREATE INDEX IF NOT EXISTS idx_latest_teacher_homerooms_v2_teacher_id ON latest_teacher_homerooms_v2(teacher_id);
CREATE INDEX IF NOT EXISTS idx_latest_teacher_homerooms_v2_class_name ON latest_teacher_homerooms_v2(class_name);

-- ---------------------------------------------------------------------------
-- 班级配置模块 (class_config)
-- ---------------------------------------------------------------------------

CREATE TABLE IF NOT EXISTS class_configs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    config_type TEXT NOT NULL,
    grade_name TEXT NOT NULL,
    class_name TEXT NOT NULL,
    building TEXT NOT NULL,
    floor TEXT NOT NULL,
    room_label TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    UNIQUE(config_type, class_name)
);

CREATE TABLE IF NOT EXISTS class_config_subjects (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    config_id INTEGER NOT NULL,
    subject TEXT NOT NULL,
    UNIQUE(config_id, subject),
    FOREIGN KEY(config_id) REFERENCES class_configs(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_class_configs_type_grade ON class_configs(config_type, grade_name);
CREATE INDEX IF NOT EXISTS idx_class_configs_class_name ON class_configs(class_name);
CREATE INDEX IF NOT EXISTS idx_class_config_subjects_config_id ON class_config_subjects(config_id);

-- ---------------------------------------------------------------------------
-- 考场分配模块 (exam_allocation)
-- ---------------------------------------------------------------------------

CREATE TABLE IF NOT EXISTS exam_allocation_settings (
    id INTEGER PRIMARY KEY,
    default_capacity INTEGER NOT NULL,
    max_capacity INTEGER NOT NULL,
    exam_title TEXT NOT NULL DEFAULT '',
    exam_notices_json TEXT NOT NULL DEFAULT '[]',
    updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS latest_exam_plan_meta (
    id INTEGER PRIMARY KEY,
    generated_at TEXT NOT NULL,
    default_capacity INTEGER NOT NULL,
    max_capacity INTEGER NOT NULL,
    grade_count INTEGER NOT NULL,
    session_count INTEGER NOT NULL,
    warning_count INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS exam_generation_progress (
    id INTEGER PRIMARY KEY,
    status TEXT NOT NULL,
    stage TEXT NOT NULL,
    stage_label TEXT NOT NULL,
    percent INTEGER NOT NULL,
    message TEXT NOT NULL,
    current_grade TEXT,
    total_grades INTEGER NOT NULL DEFAULT 0,
    completed_grades INTEGER NOT NULL DEFAULT 0,
    updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS latest_exam_plan_sessions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    grade_name TEXT NOT NULL,
    subject TEXT NOT NULL,
    is_foreign_group INTEGER NOT NULL,
    foreign_order INTEGER,
    participant_count INTEGER NOT NULL,
    exam_room_count INTEGER NOT NULL,
    self_study_room_count INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS latest_exam_plan_spaces (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    session_id INTEGER NOT NULL,
    space_type TEXT NOT NULL,
    space_source TEXT NOT NULL,
    grade_name TEXT NOT NULL,
    subject TEXT NOT NULL,
    space_name TEXT NOT NULL,
    original_class_name TEXT,
    self_study_subject TEXT,
    building TEXT NOT NULL,
    floor TEXT NOT NULL,
    capacity INTEGER,
    sort_index INTEGER NOT NULL,
    FOREIGN KEY(session_id) REFERENCES latest_exam_plan_sessions(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS latest_exam_plan_student_allocations (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    session_id INTEGER NOT NULL,
    admission_no TEXT NOT NULL,
    student_name TEXT NOT NULL,
    class_name TEXT NOT NULL,
    allocation_type TEXT NOT NULL,
    space_id INTEGER,
    seat_no INTEGER,
    subject_score REAL,
    FOREIGN KEY(session_id) REFERENCES latest_exam_plan_sessions(id) ON DELETE CASCADE,
    FOREIGN KEY(space_id) REFERENCES latest_exam_plan_spaces(id) ON DELETE SET NULL
);

CREATE TABLE IF NOT EXISTS latest_exam_plan_staff_assignments (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    session_id INTEGER NOT NULL,
    space_id INTEGER NOT NULL,
    teacher_name TEXT NOT NULL,
    assignment_type TEXT NOT NULL,
    note TEXT,
    FOREIGN KEY(session_id) REFERENCES latest_exam_plan_sessions(id) ON DELETE CASCADE,
    FOREIGN KEY(space_id) REFERENCES latest_exam_plan_spaces(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS exam_session_times (
    session_id INTEGER PRIMARY KEY,
    subject TEXT NOT NULL,
    start_at TEXT NOT NULL,
    end_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    FOREIGN KEY(session_id) REFERENCES latest_exam_plan_sessions(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS exam_subject_time_templates (
    subject TEXT PRIMARY KEY,
    start_at TEXT NOT NULL,
    end_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

-- ---------------------------------------------------------------------------
-- 监考人员分配模块 (exam_staff / invigilation)
-- ---------------------------------------------------------------------------

CREATE TABLE IF NOT EXISTS latest_exam_staff_plan_meta (
    id INTEGER PRIMARY KEY,
    generated_at TEXT NOT NULL,
    session_count INTEGER NOT NULL,
    task_count INTEGER NOT NULL,
    assigned_count INTEGER NOT NULL,
    unassigned_count INTEGER NOT NULL,
    warning_count INTEGER NOT NULL,
    imbalance_minutes INTEGER NOT NULL,
    solver_engine TEXT NOT NULL DEFAULT 'greedy',
    optimality_status TEXT NOT NULL DEFAULT 'fallback',
    solve_duration_ms INTEGER NOT NULL DEFAULT 0,
    fallback_reason TEXT,
    fallback_pool_assignments INTEGER NOT NULL DEFAULT 0,
    baseline_dominated INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS latest_exam_staff_tasks (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    session_id INTEGER,
    space_id INTEGER,
    task_source TEXT NOT NULL DEFAULT 'exam',
    role TEXT NOT NULL,
    grade_name TEXT NOT NULL,
    subject TEXT NOT NULL,
    space_name TEXT NOT NULL,
    floor TEXT NOT NULL,
    start_at TEXT NOT NULL,
    end_at TEXT NOT NULL,
    duration_minutes INTEGER NOT NULL,
    recommended_subject TEXT,
    priority_subject_chain TEXT,
    assignment_tier TEXT,
    status TEXT NOT NULL,
    reason TEXT,
    allowance_amount REAL NOT NULL DEFAULT 0,
    FOREIGN KEY(session_id) REFERENCES latest_exam_plan_sessions(id) ON DELETE CASCADE,
    FOREIGN KEY(space_id) REFERENCES latest_exam_plan_spaces(id) ON DELETE SET NULL
);

CREATE TABLE IF NOT EXISTS latest_exam_staff_assignments (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    task_id INTEGER NOT NULL,
    teacher_id INTEGER NOT NULL,
    teacher_name TEXT NOT NULL,
    assigned_at TEXT NOT NULL,
    FOREIGN KEY(task_id) REFERENCES latest_exam_staff_tasks(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS latest_teacher_duty_stats (
    teacher_id INTEGER PRIMARY KEY,
    teacher_name TEXT NOT NULL,
    indoor_minutes INTEGER NOT NULL,
    outdoor_minutes INTEGER NOT NULL,
    total_minutes INTEGER NOT NULL,
    task_count INTEGER NOT NULL,
    exam_room_task_count INTEGER NOT NULL DEFAULT 0,
    self_study_task_count INTEGER NOT NULL DEFAULT 0,
    floor_rover_task_count INTEGER NOT NULL DEFAULT 0,
    is_middle_manager INTEGER NOT NULL DEFAULT 0,
    allowance_total REAL NOT NULL DEFAULT 0,
    indoor_allowance_total REAL NOT NULL DEFAULT 0,
    outdoor_allowance_total REAL NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS invigilation_config_settings (
    id INTEGER PRIMARY KEY,
    default_exam_room_required_count INTEGER NOT NULL DEFAULT 1,
    indoor_allowance_per_minute REAL NOT NULL DEFAULT 0.5,
    outdoor_allowance_per_minute REAL NOT NULL DEFAULT 0.3,
    middle_manager_default_enabled INTEGER NOT NULL DEFAULT 0,
    middle_manager_exception_teacher_ids_json TEXT NOT NULL DEFAULT '[]',
    self_study_date TEXT NOT NULL DEFAULT '',
    self_study_start_time TEXT NOT NULL DEFAULT '12:10',
    self_study_end_time TEXT NOT NULL DEFAULT '13:40',
    self_study_class_subjects_json TEXT NOT NULL DEFAULT '[]',
    updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS invigilation_staff_exclusions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    teacher_id INTEGER NOT NULL,
    teacher_name TEXT NOT NULL,
    session_id INTEGER NOT NULL,
    session_label TEXT NOT NULL,
    created_at TEXT NOT NULL,
    UNIQUE(teacher_id, session_id)
);

-- ---------------------------------------------------------------------------
-- 索引 (监考相关)
-- ---------------------------------------------------------------------------

CREATE INDEX IF NOT EXISTS idx_exam_session_times_subject ON exam_session_times(subject);
CREATE INDEX IF NOT EXISTS idx_exam_subject_time_templates_subject ON exam_subject_time_templates(subject);
CREATE INDEX IF NOT EXISTS idx_latest_exam_staff_tasks_session ON latest_exam_staff_tasks(session_id);
CREATE INDEX IF NOT EXISTS idx_latest_exam_staff_tasks_role_status ON latest_exam_staff_tasks(role, status);
CREATE INDEX IF NOT EXISTS idx_latest_exam_staff_assignments_task ON latest_exam_staff_assignments(task_id);
CREATE INDEX IF NOT EXISTS idx_latest_teacher_duty_stats_total ON latest_teacher_duty_stats(total_minutes);
CREATE INDEX IF NOT EXISTS idx_invigilation_staff_exclusions_session ON invigilation_staff_exclusions(session_id);

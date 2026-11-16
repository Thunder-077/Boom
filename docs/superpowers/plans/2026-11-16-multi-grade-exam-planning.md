# 多年级排考与监考隔离 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 支持多Sheet整批导入、综评隔离排考、按教师Sheet归属和同学科覆盖优先分配监考，并安全输出全部年级资料。

**Architecture:** 沿用统一数据库，显式携带年级与分配组，先计算整批内存草稿，再通过应用服务统一控制短事务替换结果。监考求解按年级拆分，年级内部使用考场级覆盖目标；文件先生成到临时目录，再通过可回退发布流程替换完整输出。

**Tech Stack:** Rust、Tauri、SeaORM/SQLite、calamine、rust_xlsxwriter、CP-SAT、React/TypeScript、Vitest。

**Spec:** `docs/superpowers/specs/2026-11-16-multi-grade-exam-planning-design.md`（用户已确认）。

## Global Constraints

- “教师可监考年级以教师工作簿所在 Sheet 为准，任教班级编码不改变其监考归属。”
- “同名教师跨年级 Sheet 出现时，提示具体 Sheet 和 Excel 行号，阻止整次教师导入，保留原数据库数据。”
- “高三综评班与非综评班学生不能混场；先安排非综评组，再安排综评组，备用考场也由非综评组优先启用。”
- “默认人数不是硬上限，最大人数才是单场上限”；保留年级容量配置。
- “没有合格教师则保留‘待分配’并报告缺口，不能为凑数跨年级借人或安排冲突。”
- “旧教师无法可靠恢复原 Sheet，新增归属字段允许为空，保留原数据”；生成前要求重新导入。
- “所有 Sheet 先在内存中完成解析和校验，再在应用服务控制的事务中一次替换同类最新数据。”
- “不将用户原始文件、姓名、准考证号或成绩明细加入版本控制。”
- Migration版本化；SQL只在Repository内并参数化，常规CRUD使用ORM；保留用户工作区修改；必要的中文业务注释。
- 不新增学科映射配置、不增加手动逐年级生成入口，不自动推送或发布版本。

## Review Focus

1. 后面的Sheet出错不能使前面的Sheet已入库；Task 2以原有快照不变为断言。
2. 课表同步不能清空或推断教师Sheet归属；Task 1检查已有归属保留、新教师归属为空。
3. 某组无人、没有成绩的配置班、普通组耗尽备用房时不能跨组借房；Task 3覆盖这三种输入。
4. 某名跨年级任课教师、缺归属教师以及跨年级重叠楼层任务不能突破年级边界；Task 4覆盖。
5. Windows目标文件被Excel占用、磁盘写入失败或提交失败时不能损坏旧数据库和旧导出；Task 6注入故障验证回退。

## 文件与依赖边界

新增模块只承担独立职责，不把现有大模块整体重写：

- `src-tauri/src/exam_group.rs`：普通/综评枚举及序列化。
- `src-tauri/src/excel_import.rs`：Sheet年级名与来源定位，不承载学生成绩或教师业务解析。
- `src-tauri/src/exam_plan_snapshot.rs`：无副作用的考场草稿结构和校验。
- `src-tauri/src/export_batch.rs`：临时目录、完整发布与失败回退，不包含Excel表格内容生成。
- 现有`score.rs`、`teacher.rs`继续各自解析业务行；所有Repository只负责查询/写入。
- 现有`exam_allocation.rs`协调资源与整批草稿；`exam_staff.rs`协调任务、候选人和求解。
- 现有`export_bundle.rs`、`export_invigilation.rs`保留工作簿构建函数，接受显式快照/年级输入。

顺序：Task 1 → Task 2 → Task 3 → Task 4 → Task 5 → Task 6 → Task 7。先不要并行修改共有模型和大模块。

## Task 1：迁移、模型与事务访问边界

**Files:**
- Create: `src-tauri/src/exam_group.rs`
- Modify: `src-tauri/src/lib.rs`, `src-tauri/src/db/migration.rs`
- Modify: `src-tauri/src/entity/latest_student_scores.rs`, `latest_teachers_v2.rs`, `latest_exam_plan_spaces.rs`
- Modify: `src-tauri/src/db/repos/score.rs`, `teacher.rs`, `exam_allocation.rs`, `exam_staff.rs`, `export_bundle.rs`
- Modify: 对应`score.rs`、`teacher.rs`的Parsed/DTO类型和所有受影响实体构造器。
- Modify: `src/entities/score/model.ts`、`src/entities/teacher/model.ts`，与新增响应字段保持一致，不强制增加新的列表列。
- Test: `db/migration.rs`及各Repository的`#[cfg(test)] mod tests`。

**Interfaces:**
- Produces: `ExamGroup::{Regular, Comprehensive}`；`as_key(self) -> &'static str`返回`regular`/`comprehensive`；`from_key(&str) -> Result<ExamGroup, AppError>`；派生Clone/Copy/Eq/Hash/Serialize/Deserialize。
- 学生新增`remark: Option<String>`、`exam_group: String`（非空，默认`regular`）；教师新增`invigilation_grade_name: Option<String>`；空间新增`exam_group: Option<String>`（旧空间NULL，自习空间NULL）。
- `ParsedStudent`增加`remark: Option<String>, exam_group: ExamGroup`；`AggregatedTeacher`增加`invigilation_grade_name: String`。
- `persist_latest_snapshot<C: ConnectionTrait>(db: &C, source_file: &str, imported_at: &str, students: &[ParsedStudent]) -> Result<(), AppError>`与`replace_all<C: ConnectionTrait>(db: &C, imported_at: &str, source_file: &str, teachers: &[AggregatedTeacher]) -> Result<(), AppError>`不自行开/提交事务；应用服务负责begin/commit。
- `exam_staff::TeacherRow`与`TeacherInfo`均增加`invigilation_grade_name: Option<String>`；所有读取链路携带而非推断该字段。

- [ ] 写失败测试`test_multigrade_migration_preserves_legacy_data`：执行原有migration、插入旧学生/教师/设置，再升级两次，断言旧记录仍在、学生组为`regular`、备注/教师归属/旧空间组为NULL。
- [ ] 写`test_schedule_sync_preserves_invigilation_grade`：已归属高一的教师同步含高二任课记录后仍为高一；课表新增教师归属为NULL，不能自动推断。
- [ ] 运行`cargo test --manifest-path src-tauri/Cargo.toml --lib multigrade_migration`和`... --lib schedule_sync_preserves`，确认失败。
- [ ] 追加唯一命名migration，不修改旧migration；补齐模型、数据访问映射与构造器。事务重构同时更新现有命令和测试调用方，禁止在事务中另起连接读取未提交结果。
- [ ] 重跑上述测试及`cargo test --manifest-path src-tauri/Cargo.toml --lib`，全部非手工测试通过。
- [ ] 按明确路径暂存本任务修改，提交`feat(db): 增加学生分组与教师监考年级`。

## Task 2：多Sheet整批解析和安全导入

**Files:**
- Create: `src-tauri/src/excel_import.rs`
- Modify: `src-tauri/src/lib.rs`, `src-tauri/src/score.rs`, `src-tauri/src/teacher.rs`
- Modify: `src-tauri/src/db/repos/score.rs`, `teacher.rs`（仅测试和必要映射）
- Test: `score.rs::tests`、`teacher.rs::tests`；以rust_xlsxwriter创建临时合成工作簿，由calamine读回，不提交真实附件。

**Interfaces:**
- Consumes: Task 1字段及泛型Repository写入方法。
- Produces: `ImportLocation { sheet_name: String, row_number: usize }`、`normalize_sheet_grade(name: &str) -> Result<String, AppError>`，仅接受去首尾空白后的高一/高二/高三。
- 保持`parse_excel_rows(file_path: &str) -> Result<Vec<ParsedStudent>, AppError>`与`parse_teacher_excel(file_path: &str) -> Result<Vec<ParsedTeacherRow>, AppError>`入口；内部遍历全部Sheet。
- `ParsedTeacherRow`增加`invigilation_grade_name: String, location: ImportLocation`；学生解析中的重复标识定位表留在内存，不持久化真实源文件内容。

- [ ] 写`test_score_import_reads_all_grade_sheets`：三个Sheet合成2/3/4名学生，断言总数9、各年级独立排名；高三备注含空值、`综评班`，对应组正确。
- [ ] 写一组表驱动错误测试：末Sheet错表头/缺必填、跨Sheet重复准考证号、班级年级矛盾、同班综评混杂；断言错误包含Sheet及真实行号，数据库旧快照/导入元数据逐项不变。
- [ ] 写教师测试：三Sheet共3名教师全部保留；同Sheet同名两行仅1实体；同名跨Sheet拒绝并列出两处行号；高一Sheet含`204`的教师仍归高一。补空Sheet、无备注旧表头、教师首行表头/首行标题两种格式。
- [ ] 运行`cargo test --manifest-path src-tauri/Cargo.toml --lib score::tests`和`... --lib teacher::tests`，确认新测试失败。
- [ ] calamine逐Sheet解析，先校验所有数据再聚合/排名，然后由各import命令begin一次事务并写一次完整快照。未知非空备注原样保留且按普通处理，高三`综评班`精确匹配；非空非年级Sheet报错。
- [ ] 重跑两个测试模块与导入Repository回退测试，确认通过。
- [ ] 提交`feat(import): 支持多Sheet年级归属与综评备注校验`。

## Task 3：整批考场草稿、两组隔离及资源判重

**Files:**
- Create: `src-tauri/src/exam_plan_snapshot.rs`
- Modify: `src-tauri/src/lib.rs`, `src-tauri/src/exam_allocation.rs`
- Modify: `src-tauri/src/db/repos/exam_allocation.rs`, `export_bundle.rs`
- Test: `exam_allocation.rs::tests`、`exam_plan_snapshot.rs::tests`、Repository快照替换测试。

**Interfaces:**
- Consumes: `ExamGroup`、带组别的学生、现有`GradeContext`和年级人数设置。
- Produces: `ExamPlanDraft { sessions: Vec<SessionDraft> }`；`SessionDraft`承载年级/实际科目/场次时间、`Vec<SpaceDraft>`和学生座位；`SpaceDraft`承载来源、规范名称、位置、`Option<ExamGroup>`及容量。草稿内以局部索引引用空间，不提前依赖数据库自增ID。
- `build_session_draft(...) -> Result<SessionDraft, AppError>`替代原`build_session`的即时写库职责，参数显式接收只读来源与占用表；容量、轮转算法继续复用。
- `validate_exam_plan_draft(draft: &ExamPlanDraft) -> Result<(), AppError>`校验不混组、空间唯一与容量；`replace_plan_snapshot(tx: &DatabaseTransaction, draft: &ExamPlanDraft) -> Result<(), AppError>`只写库并完成草稿索引到真实ID映射，不自行提交。
- 教室分组只读取已导入班级属性；配置存在但全班没有成绩、无法确定组别的教学教室不自动纳入任一组，给出来源缺失诊断。显式备用考场仍通过备用资源池分配。

- [ ] 写`test_regular_group_claims_reserves_before_comprehensive`：普通教室1—10、综评11—12、备用13—16，两组各需2个备用，断言普通占13/14、综评占15/16；两组所有考场学生属性一致。
- [ ] 写`test_group_boundaries_survive_empty_groups_and_shortages`：综评无人则无空场；缺成绩的配置班不被猜为综评或借用为另一组资源；普通已占备用后综评不足明确失败，不借普通教室、不超容量。
- [ ] 写`test_same_named_classroom_and_reserve_are_not_two_resources`及`test_conflicting_room_names_fail_validation`：教学班`高三11班`派生名称与备用`高三11场`不能累计成44人/4监考，异物理位置同导出名称提示配置冲突。
- [ ] 写整批失败测试：前两个年级草稿可建、末年级不足时，旧考场/学生座位/监考快照全部不变。补普通与综评尾场分别计算、最大30时任何场不超30、不同开始时间但区间重叠的同名空间不可重复占用。
- [ ] 运行`cargo test --manifest-path src-tauri/Cargo.toml --lib exam_allocation::tests`，确认新测试失败。
- [ ] 先加载所有来源、计算全部年级内存草稿、完成预检，再进入短事务写入；进度更新发生在草稿计算阶段，不在持有SQLite写锁时用另一连接写进度。规范房间名后统一占用，备用按自然编号排序，组内普通优先、综评在后。
- [ ] 命令暂保留原返回契约，最终发布流程由Task 6接管；本任务测试以草稿与事务Repository为边界，不将半成品路径标为生成完成。
- [ ] 重跑考场相关测试和完整Rust测试，通过后提交`feat(exam): 隔离综评考场并构建整批排考草稿`。

## Task 4：教师年级硬边界及按年级建立任务

**Files:**
- Modify: `src-tauri/src/exam_staff.rs`, `src-tauri/src/db/repos/exam_staff.rs`
- Modify: `src-tauri/src/invigilation.rs`（仅需要透传错误/DTO时）
- Test: `exam_staff.rs::tests`。

**Interfaces:**
- Consumes: Task 1的`TeacherInfo.invigilation_grade_name`、Task 3已验证场次空间。
- Produces: `validate_teacher_grade_ownership(teachers: &[TeacherInfo]) -> Result<(), AppError>`；有NULL归属时在清理旧结果前报“请重新导入教师表以补齐监考年级”。
- 保留`build_task_candidate_summary(...) -> TaskCandidateSummary`入口；全部角色先按`teacher.invigilation_grade_name == task.grade_name`过滤，再应用原角色规则。
- `FloorRoverCoverage`归属键改为`(grade_name, floor)`，合并必须同年级；所有`covered_session_ids`同年级。`generate_latest_exam_staff_plan_internal`按年级依次建立/求解，不在每个年级清空全局表，最终合并`SolvedPlan`后持久化一次。

- [ ] 写`test_sheet_grade_overrides_cross_grade_teaching_assignments`：高一归属教师任课`204`也不出现在高二考试/自习/楼层候选中；新过滤不能被自定义指定教师规则绕过。
- [ ] 写`test_missing_teacher_grade_preserves_previous_staff_snapshot`及`test_floor_rover_merge_stays_inside_grade`，断言缺归属不开始替换、同楼层重叠跨年级产生两条各自覆盖任务，同年级仍合并。
- [ ] 运行`cargo test --manifest-path src-tauri/Cargo.toml --lib exam_staff::tests`，确认新测试失败。
- [ ] 移除考试监考员路径中与新需求冲突的本人学科回避，包括候选过滤和求解模型中的师生同考禁排；保留显式禁排、停用、时间冲突，以及自习/流动的原专业选择规则。硬约束仍优先于学科偏好。
- [ ] 更新旧回避测试为新规则断言，不简单删除测试；重跑所有角色和时间冲突测试。
- [ ] 提交`feat(staff): 按教师Sheet年级隔离监考任务与候选人`。

## Task 5：同学科覆盖的分层求解目标

**Files:**
- Modify: `src-tauri/src/exam_staff.rs`（`solve_with_cp_sat`、阶段预算/诊断及相关指标）
- Test: `exam_staff.rs::tests`，保持每个测试围绕一个完整可验证场景，不按单个教师拆测试。

**Interfaces:**
- Consumes: Task 4已按年级过滤的任务/教师/候选表；保持`solve_with_cp_sat`与`SolvedPlan`外部接口。
- Produces: 以`(session_id, space_id)`标识的考试考场覆盖布尔变量，只在考试监考员任务计算；同科关系来自`task.subject`与`teacher.subjects`。
- 指标顺序严格为：原合法岗位填充/考试岗位优先 → 至少1名同科的考场数 → 同科监考位总数 → 原均衡目标。各阶段固定上一阶段已取得值，再优化下一阶段；复用剩余总预算，不为每阶段重置完整超时。

- [ ] 写`test_same_subject_teachers_cover_rooms_before_second_seats`：10场各2人，10名同科+10名其他、均合法，断言填20位、覆盖10场、每场恰1同科；重复运行不检查具体姓名顺序。
- [ ] 写6名同科+足量其他时覆盖6个不同场、20名同科时全20位同科、0名时其他老师正常补齐；每场1人与3人也遵守先覆盖原则。
- [ ] 写`test_same_subject_preference_respects_constraints_and_languages`：停用/时间冲突/显式禁排不因同科被突破，英语考试不把俄语当同科；自习和流动不计入考场覆盖收益。
- [ ] 用可控阶段超时测试断言：后阶段超时保留已求得的合法解与正确最优性状态，不能返回违反硬约束的贪心结果，也不能声称最优。
- [ ] 运行`cargo test --manifest-path src-tauri/Cargo.toml --lib same_subject`，确认失败；实现上述阶段目标和诊断，重跑同科、原均衡、岗位不足测试及完整Rust测试。
- [ ] 提交`feat(staff): 优先覆盖各考场的同学科监考员`。

## Task 6：分年级导出、整批发布与前端路径契约

**Files:**
- Create: `src-tauri/src/export_batch.rs`
- Modify: `src-tauri/src/lib.rs`, `src-tauri/src/export_bundle.rs`, `export_invigilation.rs`
- Modify: `src-tauri/src/db/repos/export_bundle.rs`, `export_invigilation.rs`, `exam_allocation.rs`, `exam_staff.rs`
- Modify: `src/entities/exam-plan/model.ts`, `src/features/dashboard/store.ts`, `service.ts`
- Modify: `src/features/invigilation/ui/MonitorConfigPanel.tsx`, `src/features/invigilation/service.ts`
- Test: 各导出模块Rust测试，`src/features/dashboard/__tests__/examAllocationStore.spec.ts`。

**Interfaces:**
- Consumes: `ExamPlanDraft`、按年级`SolvedPlan`、现有工作簿构建函数。
- Produces: `PreparedExportBatch { staging_dir: PathBuf, final_dir: PathBuf }`；`prepare_export_batch(root: &Path, final_name: &str) -> Result<PreparedExportBatch, AppError>`只创建唯一临时目录，不删旧目录。
- `publish_prepared_batch(batch: &PreparedExportBatch) -> Result<PublishedExportBatch, AppError>`将旧目标改为本次备份，再将已准备目录发布；`PublishedExportBatch::rollback(&mut self) -> Result<(), AppError>`恢复旧输出，`finish(self) -> Result<(), AppError>`清理本次备份。Drop不执行不可报告的破坏性清理。
- `write_exam_bundle_from_draft(draft: &ExamPlanDraft, output_dir: &Path, ...) -> Result<(), AppError>`读取显式数据，不重新连接去读尚未提交的最新表；监考工作簿按明确年级参数过滤输入。
- `ExportLatestInvigilationScheduleResult`改为camelCase `{ folderPath: string, files: Array<{gradeName: string, filePath: string}>, exportedAt: string }`；所有调用方和mock同步升级，打开目录使用folderPath，不把目录伪装为旧filePath。

- [ ] 写`test_export_separates_grade_workbooks_and_group_labels`：两个年级、一个年级有两组，逐文件用calamine读回，断言只含对应年级，组别、人数、监考/核算教师集合正确，根目录无合并跨年级监考表。
- [ ] 写`test_staff_export_preserves_same_batch_student_artifacts`：先完成考场资料，再生成/再次导出监考资料，断言名册、考场汇总和准考证字节不变；不能把只有监考文件的临时目录覆盖为整批目录。
- [ ] 写`test_export_rejects_duplicate_room_cells`：相同年级时间同名但不同空间直接报冲突，不能把30和14累加为44或输出4位置；同一空间的合法两条监考任务不误报。
- [ ] 写发布故障测试：第二年级写文件失败、目标目录被占用导致rename失败、发布后DB提交失败，分别断言旧文件字节及旧DB完整快照保留，失败路径未返回成功。清理只能涉及staging/本次backup，其他兄弟目录不变。
- [ ] 前端写mock响应多files的测试，断言成功反馈打开folderPath，各年级文件信息保留，服务异常不覆盖旧结果路径。
- [ ] 运行`cargo test --manifest-path src-tauri/Cargo.toml --lib export`与`pnpm test`，确认新增测试失败。
- [ ] 年级Repository查询增加显式过滤，核算教师按归属年级而非仅按有任务过滤；空任务但有效归属教师是否出现在核算表沿用原表逻辑。目录名校验为标准年级，防止路径越界。
- [ ] 接通整批发布：预检并计算全草稿/全部求解结果 → 写齐临时输出 → 应用服务开始短事务替换全部结果 → 发布目录且保留旧备份 → 提交DB → 清理本次备份。失败则回滚DB与目录；回退失败明确报告并保留备份路径，禁止吞错或递归删最终根目录。
- [ ] 共用已清洗考试标题对应的批次根目录；监考阶段以当前考场完成批次为基础，在临时目录保留其学生资料，再更新各年级监考文件。只有导出调用时不替换DB。考场重排成功后旧监考快照和旧监考文件同时失效，不能把旧监考文件复制进新考场批次。
- [ ] 显式导出已有完成快照时使用一致性读取事务捕获全部数据后生成文件；生成快照与文件中的源导入时间戳在发布前核验，若期间已重新导入则拒绝发布过期结果。进程被强制终止后的备份保留供恢复，不将临时目录当成成功结果。
- [ ] 重跑导出/发布测试、`pnpm test`和`pnpm build`；提交`feat(export): 按年级安全发布排考与监考资料`。

## Task 7：全链路回归与交付检查

**Files:**
- Create: `src-tauri/src/multi_grade_tests.rs`，仅在`lib.rs`通过`#[cfg(test)] mod multi_grade_tests;`注册。
- Modify: 需要供兄弟测试调用的内部入口仅扩大为`pub(crate)`，不暴露为新Tauri命令。
- Modify: 本计划中的完成勾选与必要维护说明；不复制用户附件。

**Interfaces:**
- Consumes: 前述导入解析/事务写入、草稿、求解、导出和发布接口；测试使用临时目录及内存SQLite，不启动AppHandle，不连接真实用户数据库。
- Produces: 一个可重复运行的匿名多Sheet→数据库→分组→监考→年级文件夹回归套件。

- [ ] 编写`test_multigrade_import_to_export_pipeline`：合成高一/高二/高三Sheet，包含同Sheet教师多行、跨年级任课、高三普通/综评及不同语种，验证整批保留、教师归属、无混场、容量、监考覆盖和目录隔离。
- [ ] 编写`test_multigrade_failure_preserves_completed_batch`，依次覆盖输入失败、资源不足、教师缺归属、导出失败；各断言前次完整数据库快照与文件不变。
- [ ] 运行`cargo test --manifest-path src-tauri/Cargo.toml --lib multi_grade_tests`并验证新断言能检出故意破坏的年级过滤/组别合并（只在测试轮临时撤回修复，随后恢复）。
- [ ] 用户提供文件只能做本地只读验收：确认851名学生、62名教师及高三94名综评学生；如需运行真实生成，使用复制的隔离数据库和明确配置，不改用户原文件或真实DB，不输出身份明细。
- [ ] 运行`pnpm version:check`、`pnpm test`、`pnpm build`、`cargo test --manifest-path src-tauri/Cargo.toml --lib`和`git diff --check`。允许现有手工真实DB集成测试继续ignored，其他失败均处理后再交付。
- [ ] 桌面验证导入总数、缺归属提示、整批进度、同科不足警告、打开年级目录；不得只凭jsdom声称Excel/桌面已验证。资源或运行环境不足时明确列出未验证部分。
- [ ] 检查`git diff --name-only`及暂存清单不包含`.codex-remote-attachments/`、真实数据库、真实Excel或临时导出；提交`test(exam): 覆盖多年级导入与分组排考全流程`。
- [ ] 进行整分支独立审查，修复重要问题后重跑受影响验证。交付本地分支、提交与测试证据，未经用户另行要求不推送、不打标签、不发布版本。

## 计划自审记录

- 设计第1—4节→Task 2；第5节→Task 1；第6节→Task 3；第7节→Task 4/5；第8节→Task 6；第9—10节→Global Constraints与Task 7。
- Review Focus的5类故障均已写入对应任务测试；同名空间防护覆盖生成与导出两层。
- `ExamGroup`、教师归属字段、草稿和导出目录契约在首次出现处定义；后续任务复用相同命名。
- 发布流程采用短事务加可回退文件替换，避免SQLite写锁期间另一连接写进度、旧目录先删除及跨年级部分提交。
- 当前仅计划与设计文档发生变化。实施需用户审阅此计划并选择执行方式后开始。

# Comprehensive Exam Rooms Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 高三综评组仅使用按数字排序的“综评x场”考试教室，并禁止普通组占用。

**Architecture:** 在现有考场资源选择边界内识别专用考试教室；普通组和综评组分别筛选候选资源，共享原有占用检查。不修改数据模型。

**Tech Stack:** Rust、现有单元测试、Cargo。

**Spec:** `docs/superpowers/specs/2026-11-17-comprehensive-exam-rooms-design.md`

## Global Constraints

- 其他年级及普通组的现有分配规则保持不变。
- 数据库访问仍集中在 Repository，不为本次名称筛选引入 SQL。

## Review Focus

- 非标准名称不能误判为综评场；由名称识别单元测试覆盖。
- 普通组不能提前占走综评场；由分组分配测试覆盖。
- 综评场被占用或不足不能回退到教学班；由失败测试覆盖。

### Task 1: 专用考场筛选及回归测试

**Files:**
- Modify: `src-tauri/src/exam_allocation.rs`
- Modify: `docs/superpowers/specs/2026-11-16-multi-grade-exam-planning-design.md`
- Modify: `docs/superpowers/plans/2026-11-16-multi-grade-exam-planning.md`
- Modify: `docs/superpowers/acceptance/2026-11-16-multi-grade-exam-planning.md`

**Interfaces:**
- Produces: `comprehensive_room_number(room_name: &str) -> Option<u64>`，只匹配“综评x场”。

- [x] 更新分组测试，断言综评场独占、按数字排序、无考场或占用时失败。
- [x] 运行考场测试，确认旧固定编号实现不满足新断言。
- [x] 实现专用资源筛选并保留现有占用校验与错误边界。
- [x] 在高三考试教室录入框提示“综评x场”命名规则。
- [x] 运行考场及完整 Rust 测试，确认无回归。
- [x] 同步更新旧规则文档，不提交代码，保留用户工作区。

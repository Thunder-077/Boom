use rusqlite::Connection;

use crate::score::AppError;

const SCHEMA_SQL: &str = include_str!("schema.sql");

pub fn ensure_schema(conn: &Connection) -> Result<(), AppError> {
    conn.execute_batch(SCHEMA_SQL)?;
    ensure_column(conn, "latest_exam_staff_tasks", "assignment_tier", "TEXT")?;
    ensure_column(
        conn,
        "latest_exam_staff_plan_meta",
        "solver_engine",
        "TEXT NOT NULL DEFAULT 'greedy'",
    )?;
    ensure_column(
        conn,
        "latest_exam_staff_plan_meta",
        "optimality_status",
        "TEXT NOT NULL DEFAULT 'fallback'",
    )?;
    ensure_column(
        conn,
        "latest_exam_staff_plan_meta",
        "solve_duration_ms",
        "INTEGER NOT NULL DEFAULT 0",
    )?;
    ensure_column(
        conn,
        "latest_exam_staff_plan_meta",
        "fallback_reason",
        "TEXT",
    )?;
    ensure_column(
        conn,
        "latest_exam_staff_plan_meta",
        "fallback_pool_assignments",
        "INTEGER NOT NULL DEFAULT 0",
    )?;
    ensure_column(
        conn,
        "latest_exam_staff_plan_meta",
        "baseline_dominated",
        "INTEGER NOT NULL DEFAULT 0",
    )?;
    Ok(())
}

fn ensure_column(
    conn: &Connection,
    table_name: &str,
    column_name: &str,
    definition: &str,
) -> Result<(), AppError> {
    if has_column(conn, table_name, column_name)? {
        return Ok(());
    }
    conn.execute(
        &format!("ALTER TABLE {table_name} ADD COLUMN {column_name} {definition}"),
        [],
    )?;
    Ok(())
}

fn has_column(conn: &Connection, table_name: &str, column_name: &str) -> Result<bool, AppError> {
    let mut stmt = conn.prepare(&format!("PRAGMA table_info({table_name})"))?;
    let rows = stmt.query_map([], |row| row.get::<_, String>(1))?;
    for row in rows {
        if row? == column_name {
            return Ok(true);
        }
    }
    Ok(false)
}

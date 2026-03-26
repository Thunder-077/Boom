use rusqlite::Connection;

use crate::score::AppError;

const SCHEMA_SQL: &str = include_str!("schema.sql");

pub fn ensure_schema(conn: &Connection) -> Result<(), AppError> {
    conn.execute_batch(SCHEMA_SQL)?;
    Ok(())
}

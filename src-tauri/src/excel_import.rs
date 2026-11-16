use crate::score::AppError;

#[derive(Debug, Clone)]
pub(crate) struct ImportLocation {
    pub sheet_name: String,
    pub row_number: usize,
}

impl std::fmt::Display for ImportLocation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Sheet《{}》第{}行", self.sheet_name, self.row_number)
    }
}

pub(crate) fn normalize_sheet_grade(name: &str) -> Result<String, AppError> {
    let grade = name.trim();
    match grade {
        "高一" | "高二" | "高三" => Ok(grade.to_string()),
        _ => Err(AppError::new(format!(
            "Sheet《{name}》不是有效年级，请使用高一、高二或高三"
        ))),
    }
}

use crate::score::AppError;
use serde::{Deserialize, Serialize};

/// 考生分组是排考硬边界，不能通过尾场合并跨组调剂。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExamGroup {
    Regular,
    Comprehensive,
}

impl ExamGroup {
    pub fn as_key(self) -> &'static str {
        match self {
            Self::Regular => "regular",
            Self::Comprehensive => "comprehensive",
        }
    }
    pub fn from_key(value: &str) -> Result<Self, AppError> {
        match value {
            "regular" => Ok(Self::Regular),
            "comprehensive" => Ok(Self::Comprehensive),
            _ => Err(AppError::new(format!("无效的考生分组：{value}"))),
        }
    }
}

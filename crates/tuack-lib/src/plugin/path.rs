//! 工作区路径解析宿主函数的返回契约。

use crate::prelude::*;

/// 路径解析未能完成的可恢复错误（`host_get_path` payload 的 `Err` 侧）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, thiserror::Error)]
pub enum PathError {
    /// 路径非法（`..` 穿越、越出工作区、经软链逃逸或非 UTF-8）
    #[error("路径不合法：{0}")]
    Invalid(String),
    /// 协议/内部错误（通常不可恢复）
    #[error("内部错误：{0}")]
    Internal(String),
}

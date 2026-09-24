//! 资产访问宿主函数的返回契约。

use crate::prelude::*;

/// 一段资产内容（`asset_read` payload 的成功侧）：空块表示已读到 EOF，
/// 字节按原样过边界（`serde_bytes`）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AssetChunk(#[serde(with = "serde_bytes")] pub Vec<u8>);

/// 资产访问未能完成的可恢复错误（`asset_*` payload 的 `Err` 侧）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, thiserror::Error)]
pub enum AssetError {
    /// 当前插件调用未提供题目资源
    #[error("该插件不支持访问题目资源")]
    Unavailable,
    /// 资产句柄无效
    #[error("无效的资产句柄：{0}")]
    InvalidHandle(u64),
    /// 资源逻辑路径非法（穿越/越界）
    #[error("资源路径不合法：{0}")]
    InvalidPath(String),
    /// 打开资源失败（含 `AssetProvider` 的整条上下文链）
    #[error("{0}")]
    Load(String),
    /// 读写资源失败
    #[error("读写资源失败：{0}")]
    Io(String),
    /// 协议/内部错误（通常不可恢复，如两侧 payload 编码不一致）
    #[error("内部错误：{0}")]
    Internal(String),
}

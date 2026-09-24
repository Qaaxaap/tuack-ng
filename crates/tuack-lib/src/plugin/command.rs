//! `run_command` 宿主函数的返回契约。

use crate::prelude::*;

/// 外部命令执行结果（宿主暴露给插件的 `run_command` 返回值）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandResult {
    pub exit_code: i32,
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
}

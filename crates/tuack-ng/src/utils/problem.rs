//! 题目的派生运行参数。

use crate::prelude::*;

/// 题目运行时的 IO 模式：`file_io` 未配置时按文件 IO（输入输出文件名为 `<name>.in` / `<name>.out`）。
pub fn io_mode(problem: &ProblemConfig) -> IoMode {
    if problem.file_io.unwrap_or(true) {
        IoMode::File {
            input_name: format!("{}.in", problem.name),
            output_name: format!("{}.out", problem.name),
        }
    } else {
        IoMode::Stdio
    }
}

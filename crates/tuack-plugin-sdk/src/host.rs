//! 宿主函数导入与封装。

use std::io::Read;

use extism_pdk::{Json, Msgpack, host_fn};
use tuack_lib::plugin::{AssetChunk, AssetError, CommandError, CommandResult, PathError};

/// 宿主提供的 host 函数（由 extism 的 `host_context` 承载状态）。
///
/// # Errors
///
/// 契约内的失败（资源不可用/打不开、句柄非法、路径越界、命令不在白名单或未启动等）在 payload
/// 的 `Err` 侧返回；内存读写或宿主上下文层面的错误以 `extism_pdk::Error` 结束调用。
#[host_fn("extism:host/user")]
unsafe extern "ExtismHost" {
    pub fn asset_open(problem_idx: u64, url: String) -> Json<Result<u64, AssetError>>;
    /// 资产块按 msgpack 字节串通道传回。
    pub fn asset_read(asset_id: u64, len: u64) -> Msgpack<Result<AssetChunk, AssetError>>;
    pub fn asset_close(asset_id: u64);
    pub fn asset_copy(asset_id: u64, dest: String) -> Json<Result<(), AssetError>>;
    /// 命令的 `stdout`/`stderr` 按 msgpack 字节串通道传回。
    pub fn run_command(
        program: String,
        args: Json<Vec<String>>,
        cwd: Json<String>,
    ) -> Msgpack<Result<CommandResult, CommandError>>;
    pub fn plugin_log(level: i32, msg: String);
    pub fn host_get_path(path: String) -> Json<Result<String, PathError>>;
}

/// 资产流：把宿主的 `asset_open/read/close` 封装为 `Read` 对象。
///
/// 注意：此流与宿主持有的流共享当前位置。若先 `read` 到中间或末尾，再把本句柄
/// 放进 `OutputFile` 返回，宿主取流落盘时会从当前位置继续拷贝，产物将只含剩余
/// 部分（甚至为空）。要整份拷贝，请勿提前 `read`。
pub struct AssetReader {
    id: u64,
}

impl AssetReader {
    /// 打开第 `problem_idx` 题的资产 `url`，返回可读流。
    ///
    /// # Errors
    ///
    /// 该次调用未提供题目资源或打开失败时返回 [`AssetError`]。
    pub fn open(problem_idx: u64, url: &str) -> Result<Self, AssetError> {
        let outer = unsafe { asset_open(problem_idx, url.to_string()) }
            .map_err(|e| AssetError::Internal(format!("{e:#}")))?;
        Ok(Self { id: outer.0? })
    }

    /// 请求宿主把资产流从当前位置直接拷贝到目标 WASI 路径（如 `/tmp/xxx`），避免跨 wasm 逐块读取。
    ///
    /// 只写 WASI 文件系统，不参与产物回传；作为产物请直接把本流放进 `OutputFile`
    /// （SDK 会转成 `OutputSpec::Asset`）。
    ///
    /// # Errors
    ///
    /// 目标路径越出工作区、指向只读资源目录、句柄无效或写入失败时返回 [`AssetError`]。
    pub fn copy_to_host(&self, dest: &str) -> Result<(), AssetError> {
        let outer = unsafe { asset_copy(self.id, dest.to_string()) }
            .map_err(|e| AssetError::Internal(format!("{e:#}")))?;
        outer.0
    }

    /// 消耗自身，交出资产句柄（不再于 Drop 时关闭），供回传 `OutputSpec::Asset` 使用。
    pub(crate) fn into_id(self) -> u64 {
        std::mem::ManuallyDrop::new(self).id
    }
}

impl Read for AssetReader {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let outer = unsafe { asset_read(self.id, buf.len() as u64) }
            .map_err(|e| std::io::Error::other(format!("{e:#}")))?;
        let data = outer.0.map_err(|e| std::io::Error::other(e.to_string()))?.0;
        if data.is_empty() {
            return Ok(0);
        }
        let n = data.len().min(buf.len());
        buf[..n].copy_from_slice(&data[..n]);
        Ok(n)
    }
}

impl Drop for AssetReader {
    fn drop(&mut self) {
        unsafe {
            let _ = asset_close(self.id);
        }
    }
}

/// 执行外部命令（宿主侧执行，插件拿回退出码与输出）。
///
/// `program` 为可执行文件名/路径，`args` 为其余参数。`cwd` 为工作目录，须传 [`get_path`] 返回的
/// 宿主路径（工作区根为 `get_path("/")`）。
///
/// 不建议使用此命令获取大输出，优先使用命令自带的重定向方法。
///
/// # Errors
///
/// 命令未能执行时返回 [`CommandError`]：不在白名单、`cwd` 非法或进程未启动；
/// 命令完成执行时返回 `Ok(CommandResult)`，无论退出码为何。
pub fn command(program: &str, args: &[&str], cwd: &str) -> Result<CommandResult, CommandError> {
    let args = Json(args.iter().map(|s| s.to_string()).collect::<Vec<String>>());
    let cwd = Json(cwd.to_string());
    let outer = unsafe { run_command(program.to_string(), args, cwd) }
        .map_err(|e| CommandError::Internal(format!("{e:#}")))?;
    outer.0
}

/// 获取 WASI 工作区路径对应的宿主真实文件系统路径（如 `/out` -> 宿主产物目录）。
///
/// 仅覆盖可写工作区（`/` / `/tmp` / `/out`）；只读资源目录 `/assets` 只能在插件内
/// 用 WASI 文件 API 直接读取，不经此函数。
///
/// # Errors
///
/// 路径越出工作区或指向只读资源目录时返回 [`PathError::Invalid`]。
pub fn get_path(path: &str) -> Result<String, PathError> {
    let outer = unsafe { host_get_path(path.to_string()) }
        .map_err(|e| PathError::Internal(format!("{e:#}")))?;
    outer.0
}

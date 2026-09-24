//! 插件跨 wasm 边界的 wire 契约。
//!
//! 只放需要序列化过关的类型：host 函数返回（command/asset/path）与插件回传产物；
//! 共享的业务契约（`RenderDocument`/`DumpDocument`/`OutputFile` 等）留在
//! `ren`/`dump`/`utils`，由 host 与插件共同使用。

mod asset;
mod command;
mod output;
mod path;

pub use asset::{AssetChunk, AssetError};
pub use command::{CommandError, CommandResult};
pub use output::{DumperOutput, OutputSpec, ProcessorOutput, RendererOutput};
pub use path::PathError;

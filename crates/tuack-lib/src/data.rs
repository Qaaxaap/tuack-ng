use std::any::Any;
use std::io::{self, Read};

use crate::prelude::*;
use crate::utils::testlib::Arg;

/// 统一的可读流抽象：`Read + Send + 'static` 的便捷 trait（可 downcast）。
pub trait Reader: Read + Send + Any {}

impl<T: Read + Send + 'static> Reader for T {}

/// 可读数据源
pub trait Data: Send {
    /// 数据点输入
    fn input(&self) -> io::Result<Box<dyn Reader>>;

    /// 数据点输出
    fn answer(&self) -> io::Result<Box<dyn Reader>>;
}

/// 可写数据源
pub trait DataMut: Data {
    /// 写入输入文件
    fn write_input(&self, input: Box<dyn Reader>) -> Result<()>;

    /// 写入输出文件
    fn write_output(&self, output: Box<dyn Reader>) -> Result<()>;
}

/// 数据生成中的数据点：可写，并携带生成参数
pub trait DmkItem: DataMut {
    /// 生成参数
    fn args(&self) -> &IndexMap<String, Arg>;
}

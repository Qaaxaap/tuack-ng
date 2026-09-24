use crate::data::{DataMut, DmkItem};
use crate::prelude::*;
use crate::utils::compiler::{IoMode, ResourceLimits, RunSpec, RunStatus, Runner};
use crate::utils::testlib::{Generator, Validator, ValidatorResult};

/// 数据生成会话
pub struct DmkSession<'a> {
    /// 已 prepare 的标程运行器。
    runner: &'a mut dyn Runner,
    /// 已 prepare 的数据生成器。
    generator: &'a mut dyn Generator,
    /// 已 prepare 的输入校验器（可为空）。
    validator: Option<&'a dyn Validator>,
    /// 标程运行的 IO 模式。
    io_mode: IoMode,
}

impl<'a> DmkSession<'a> {
    /// 创建会话。需保证 `runner` / `generator` / `validator` 已完成 prepare。
    pub fn new(
        runner: &'a mut dyn Runner,
        generator: &'a mut dyn Generator,
        validator: Option<&'a dyn Validator>,
        io_mode: IoMode,
    ) -> Self {
        Self {
            runner,
            generator,
            validator,
            io_mode,
        }
    }

    /// 生成单点输入
    pub fn gen_input(&self, item: &dyn DmkItem, seed: u64) -> Result<()> {
        let stream = self.generator.run(item.args().clone(), seed)?;
        item.write_input(stream)?;

        let Some(validator) = self.validator else {
            return Ok(());
        };
        let mut input = item.input()?;
        match validator.validate(&mut *input)? {
            ValidatorResult::Ok => Ok(()),
            ValidatorResult::Invalid(message) => bail!("输入校验失败：{}", message),
        }
    }

    /// 用标程生成单点输出
    pub fn gen_output(&mut self, item: &dyn DataMut) -> Result<()> {
        let input = item.input()?;

        let result = self.runner.execute(RunSpec {
            limits: ResourceLimits::unlimited(),
            io_mode: self.io_mode.clone(),
            input,
        })?;

        match result.status {
            RunStatus::Success => {}
            _ if !result.stderr.is_empty() => {
                bail!(
                    "标程运行失败\n标准错误输出：{}",
                    String::from_utf8_lossy(&result.stderr)
                );
            }
            _ => bail!("标程运行失败"),
        }

        let output = match result.output {
            Some(out) => out,
            None => bail!("标程未生成输出"),
        };
        item.write_output(output)
    }
}

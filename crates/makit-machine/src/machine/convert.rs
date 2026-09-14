//! 将上下文所有权转换为对应初始化方式的状态机。

use std::marker::PhantomData;

use crate::MachineContext;

use super::{Machine, StrictMachine, Uninitialized, inner::Inner};

/// 为所有 [`MachineContext`] 提供消费所有权的状态机转换。
///
/// 两种转换均使用上下文声明的初始状态，不执行状态进入钩子，
/// 也不要求初始状态已经实现 [`State`](crate::State)。
/// 实际初始化与输入推进的方法会单独要求该状态实现对应契约。
pub trait IntoMachineState: MachineContext {
    /// 消费上下文，创建延时初始化的普通状态机。
    ///
    /// 构造时调用 [`MachineContext::initial_state`]，
    /// 初始进入钩子由显式 [`Machine::init`] 或首次 [`Machine::dispatch`] 执行。
    #[must_use]
    fn into_machine(self) -> Machine<Self>;

    /// 消费上下文，创建尚未初始化的严格状态机。
    ///
    /// 构造时调用 [`MachineContext::initial_state`]，
    /// 初始进入钩子由 [`StrictMachine::init`] 显式执行。
    #[must_use]
    fn into_strict_machine(self) -> StrictMachine<Self, Uninitialized>;
}

impl<M> IntoMachineState for M
where
    M: MachineContext,
{
    fn into_machine(self) -> Machine<Self> {
        Machine {
            inner: Inner::new(self),
            initialized: false,
        }
    }

    fn into_strict_machine(self) -> StrictMachine<Self, Uninitialized> {
        StrictMachine {
            inner: Inner::new(self),
            _mark: PhantomData,
        }
    }
}

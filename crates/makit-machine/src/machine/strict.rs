//! 使用类型阶段限制初始化与输入推进的状态机。

use std::{fmt, marker::PhantomData, ops::Deref};

use crate::{DispatchResult, MachineContext, State, machine::inner::Inner};

/// [`StrictMachine`] 已完成初始化的类型标记。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Initialized;

/// [`StrictMachine`] 尚未完成初始化的类型标记。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Uninitialized;

/// 使用类型标记管理初始化的状态机。
///
/// [`IntoMachineState::into_strict_machine`](crate::IntoMachineState::into_strict_machine)
/// 创建 [`Uninitialized`] 阶段，
/// [`init`](Self::init) 消费该机器并返回 [`Initialized`] 阶段。
/// 仅已初始化阶段提供 [`dispatch`](Self::dispatch)。
/// 每个实例独立持有上下文与状态，父状态可持有自己的子机器。
///
/// 通过 [`Deref`] 共享访问上下文，同时保留显式的上下文与状态查询。
/// 克隆保留类型阶段，不重放状态钩子。
pub struct StrictMachine<M, T>
where
    M: MachineContext,
{
    pub(super) inner: Inner<M>,
    pub(super) _mark: PhantomData<T>,
}

impl<M, T> StrictMachine<M, T>
where
    M: MachineContext,
{
    /// 借用当前机器独立持有的上下文。
    #[must_use]
    pub fn context(&self) -> &M {
        self.inner.context()
    }

    /// 借用当前状态。
    #[must_use]
    pub fn state(&self) -> &M::State {
        self.inner.state()
    }
}

impl<M> StrictMachine<M, Uninitialized>
where
    M: MachineContext,
    M::State: State<M>,
{
    /// 消费未初始化机器，执行进入钩子并返回已初始化机器。
    #[must_use]
    pub fn init(mut self) -> StrictMachine<M, Initialized> {
        self.inner.init();
        StrictMachine {
            inner: self.inner,
            _mark: PhantomData,
        }
    }
}

impl<M> StrictMachine<M, Initialized>
where
    M: MachineContext,
    M::State: State<M>,
{
    /// 检查 Guard 并推进一次输入，返回执行结果。
    ///
    /// [`DispatchResult::GuardBlocked`] 表示没有调用推进逻辑或转换钩子。
    /// [`DispatchResult::Executed`] 表示本次推进与状态切换均已完成，
    /// 即使事件为空也属于已执行；状态转换不会交给调用方再次执行。
    pub fn dispatch(&mut self, input: &M::Input<'_>) -> DispatchResult<M::Event> {
        self.inner.dispatch(input)
    }
}

impl<M, T> Clone for StrictMachine<M, T>
where
    M: MachineContext + Clone,
    M::State: Clone,
{
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            _mark: PhantomData,
        }
    }
}

impl<M, T> Deref for StrictMachine<M, T>
where
    M: MachineContext,
{
    type Target = M;

    fn deref(&self) -> &Self::Target {
        self.inner.context()
    }
}

impl<M, T> fmt::Debug for StrictMachine<M, T>
where
    M: MachineContext + fmt::Debug,
    M::State: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("StrictMachine")
            .field("context", self.inner.context())
            .field("state", self.inner.state())
            .field("stage", &std::any::type_name::<T>())
            .finish()
    }
}

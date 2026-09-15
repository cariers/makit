//! 使用类型阶段限制初始化与输入推进的状态机。

use std::{fmt, marker::PhantomData, ops::Deref};

use crate::{Error, MachineContext, State, machine::inner::Inner};

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
/// 克隆保留类型阶段，不重放状态或上下文钩子。
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
    ///
    /// 初始化仅执行初始状态进入，不调用上下文的派发或转换钩子。
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
    /// 派发一次输入，成功时返回有序事件。
    ///
    /// 先执行 [`MachineContext::before_dispatch`] 和输入推进，成功时完成所需转换及其钩子，
    /// 最后调用 [`MachineContext::after_dispatch`]，再原样返回事件。
    /// `Ok` 表示本次处理、状态切换与成功收尾均已完成，即使事件为空也属于已处理；
    /// 状态转换不会交给调用方再次执行。
    ///
    /// # Errors
    ///
    /// 原样返回 [`State::advance`] 的 [`Error::Unhandled`] 或 [`Error::Custom`]。
    /// 前者没有结果回调，后者先调用 [`MachineContext::dispatch_error`] 观察具体业务错误；
    /// 两者均不调用 [`MachineContext::after_dispatch`]。
    /// 错误不执行转换钩子；状态实现应保持推进方法入口的状态与上下文，驱动不提供回滚。
    ///
    /// # Panics
    ///
    /// 推进、钩子或状态析构中的 panic 会直接传播，后续钩子不保证执行，
    /// 也不会转为业务错误通知或自动回滚。
    pub fn dispatch(&mut self, input: &M::Input<'_>) -> Result<Box<[M::Event]>, Error<M::Error>> {
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

//! 使用运行时标记或类型标记管理初始化的状态机包装器。

mod convert;
mod inner;
mod strict;

pub use convert::IntoMachineState;
pub use strict::{Initialized, StrictMachine, Uninitialized};

use std::{fmt, ops::Deref};

use crate::{Error, MachineContext, State, machine::inner::Inner};

/// 使用运行时标记管理初始化的状态机。
///
/// 每个实例独立持有上下文与当前状态；父状态可以持有另一台机器。
/// 通过 [`IntoMachineState::into_machine`] 创建，可以显式调用 [`init`](Self::init)
/// 提前初始化，也可以交由首次 [`dispatch`](Self::dispatch) 自动完成。
/// 初始化仅执行一次，后续初始化或派发不会重复执行初始状态的进入钩子。
/// 初始化阶段若能由类型表达，可以使用 [`StrictMachine`]。
///
/// 通过 [`Deref`] 共享访问上下文，同时保留显式的上下文与状态查询。
/// 克隆保留初始化标记，不重放状态钩子。
pub struct Machine<M>
where
    M: MachineContext,
{
    inner: Inner<M>,
    initialized: bool,
}

impl<M> Machine<M>
where
    M: MachineContext,
{
    /// 返回机器是否已完成初始化。
    #[must_use]
    pub fn is_initialized(&self) -> bool {
        self.initialized
    }

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

impl<M> Machine<M>
where
    M: MachineContext,
    M::State: State<M>,
{
    /// 按需初始化机器，执行初始状态的进入钩子。
    ///
    /// 进入钩子正常结束后才记录为已初始化，重复调用不会重放进入钩子。
    /// 可以显式提前调用，也可以由 [`dispatch`](Self::dispatch) 自动调用。
    pub fn init(&mut self) {
        if !self.initialized {
            self.inner.init();
            self.initialized = true;
        }
    }

    /// 按需完成初始化并派发一次输入，成功时返回有序事件。
    ///
    /// 每次调用先通过 [`init`](Self::init) 确保机器已初始化。
    /// `Ok` 表示本次处理与状态切换均已完成，即使事件为空也属于已处理；
    /// 状态转换不会交给调用方再次执行。
    ///
    /// # Errors
    ///
    /// 原样返回 [`State::advance`] 的 [`Error::Unhandled`] 或 [`Error::Custom`]。
    /// 错误不执行转换钩子；状态实现应保持推进方法入口的状态与上下文，驱动不提供回滚。
    /// 首次派发在推进前完成的初始化会保留，后续派发也不会重放初始进入钩子。
    pub fn dispatch(&mut self, input: &M::Input<'_>) -> Result<Box<[M::Event]>, Error<M::Error>> {
        // 初始化属于机器生命周期，后续推进的任一错误都不会撤销它。
        self.init();
        self.inner.dispatch(input)
    }
}

impl<M> Deref for Machine<M>
where
    M: MachineContext,
{
    type Target = M;

    fn deref(&self) -> &Self::Target {
        self.inner.context()
    }
}

impl<M> Clone for Machine<M>
where
    M: MachineContext + Clone,
    M::State: Clone,
{
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            initialized: self.initialized,
        }
    }
}

impl<M> fmt::Debug for Machine<M>
where
    M: MachineContext + fmt::Debug,
    M::State: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Machine")
            .field("context", self.inner.context())
            .field("state", self.inner.state())
            .field("initialized", &self.initialized)
            .finish()
    }
}

use crate::Transition;

/// 状态实现产生的单次输入成功结果，由机器驱动消费。
///
/// 事件保持实现给出的顺序，允许为空；状态转换尚未由驱动应用。
/// [`crate::Machine::dispatch`] 和 [`crate::StrictMachine::dispatch`] 完成转换后，
/// 只向调用方返回事件；未处理与业务失败通过独立的 [`crate::Error`] 返回。
#[must_use]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Outcome<S, E> {
    /// 本次处理产生的有序事件，允许为空。
    pub events: Box<[E]>,
    /// 本次处理要求驱动应用的状态转换。
    pub transition: Transition<S>,
}

impl<S, E> Outcome<S, E> {
    /// 构造成功且保留当前状态的结果，事件可由数组、`Vec` 或装箱切片传入。
    pub fn stay(events: impl Into<Box<[E]>>) -> Self {
        Self {
            events: events.into(),
            transition: Transition::Stay,
        }
    }

    /// 构造成功且切换到指定状态的结果，事件可由数组、`Vec` 或装箱切片传入。
    pub fn to(state: S, events: impl Into<Box<[E]>>) -> Self {
        Self {
            events: events.into(),
            transition: Transition::To(state),
        }
    }
}

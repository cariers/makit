use crate::Transition;

/// 状态实现产生的单次输入处理结果，由机器驱动消费。
///
/// 已处理结果中的事件保持实现给出的顺序，允许为空；状态转换尚未由驱动应用，
/// 因而该值不能直接视为对外的 [`crate::DispatchResult`]。
/// 未处理结果不包含事件或转换，实现必须遵守 [`crate::State::advance`] 的无修改契约。
#[must_use]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Outcome<S, E> {
    /// 本次输入已处理，包含有序事件和待应用的状态转换。
    Handled {
        /// 本次处理产生的有序事件，允许为空。
        events: Box<[E]>,
        /// 本次处理要求驱动应用的状态转换。
        transition: Transition<S>,
    },
    /// 当前状态未处理本次输入，不附带事件或状态转换。
    ///
    /// 驱动不会执行转换钩子，也不会回滚实现已做的修改。
    /// 实现须在修改领域数据之前决定返回该结果。
    Unhandled,
}

impl<S, E> Outcome<S, E> {
    /// 构造已处理且保留当前状态的结果，事件可由数组、`Vec` 或装箱切片传入。
    pub fn stay(events: impl Into<Box<[E]>>) -> Self {
        Self::Handled {
            events: events.into(),
            transition: Transition::Stay,
        }
    }

    /// 构造已处理且切换到指定状态的结果，事件可由数组、`Vec` 或装箱切片传入。
    pub fn to(state: S, events: impl Into<Box<[E]>>) -> Self {
        Self::Handled {
            events: events.into(),
            transition: Transition::To(state),
        }
    }
}

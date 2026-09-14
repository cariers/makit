/// 一次状态推进要求机器应用的转换。
///
/// 该类型属于 [`crate::State::advance`] 的实现协议，外部输入处理只获得
/// 已完成转换后的 [`crate::DispatchResult`]。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Transition<S> {
    /// 保留当前状态，不调用退出或进入钩子。
    ///
    /// 推进仍可以修改当前状态内部的数据和上下文。
    Stay,
    /// 退出当前状态并进入指定的新状态。
    ///
    /// 即使新旧状态在业务上相等，也按显式转换执行钩子。
    To(S),
}

/// 状态实现产生的单次推进描述，由机器驱动消费。
///
/// 事件保持实现给出的顺序，允许为空；状态转换尚未由驱动应用，
/// 因而该值不能直接视为对外的执行结果。
#[must_use]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Step<S, E> {
    /// 本次推进产生的有序事件。
    pub events: Box<[E]>,
    /// 本次推进要求驱动应用的状态转换。
    pub transition: Transition<S>,
}

impl<S, E> Step<S, E> {
    /// 构造保留当前状态的推进描述，事件可由数组、`Vec` 或装箱切片传入。
    pub fn stay(events: impl Into<Box<[E]>>) -> Self {
        Self {
            events: events.into(),
            transition: Transition::Stay,
        }
    }

    /// 构造切换到指定状态的推进描述，事件可由数组、`Vec` 或装箱切片传入。
    pub fn to(state: S, events: impl Into<Box<[E]>>) -> Self {
        Self {
            events: events.into(),
            transition: Transition::To(state),
        }
    }
}

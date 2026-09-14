/// 一次输入的实际执行结果，不包含待执行的状态切换。
///
/// 已执行的输入即使没有事件，也与未通过 Guard 的输入不同：推进可能已经修改数据。
/// 调用方应根据结果变体判断是否执行，不能通过事件是否为空决定是否重投输入。
/// 结果只描述本次输入是否推进；普通机器派发时先调用幂等的 [`crate::Machine::init`]。
/// 若此次派发实际完成了初始化，即使 Guard 未通过，初始化对状态和上下文的修改也不会撤销。
#[must_use]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DispatchResult<E> {
    /// 本次输入已经执行，状态转换及其钩子均已处理完毕。
    Executed {
        /// 本次执行产生的有序事件，允许为空。
        events: Box<[E]>,
    },
    /// Guard 未放行，本次输入未推进，也未执行由状态切换触发的钩子。
    ///
    /// 若普通机器此次派发实际完成了初始化，初始化效果会保留，不能将该结果解释为整次派发无修改。
    /// 该结果不携带拒绝原因或领域事件，也不自动向父机冒泡。
    GuardBlocked,
}

impl<E> DispatchResult<E> {
    /// 判断本次输入是否已经执行，与事件数量无关。
    ///
    /// 该判断仅反映是否推进输入，不反映本次派发是否执行了初始化。
    pub const fn is_executed(&self) -> bool {
        matches!(self, Self::Executed { .. })
    }

    /// 借用本次执行产生的事件；Guard 未通过时返回 `None`。
    ///
    /// `Some(&[])` 表示已执行但没有事件，不能视为 Guard 未通过。
    pub fn events(&self) -> Option<&[E]> {
        match self {
            Self::Executed { events } => Some(events),
            Self::GuardBlocked => None,
        }
    }
}

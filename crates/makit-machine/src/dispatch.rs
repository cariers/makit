/// 一次输入的实际处理结果，不包含待执行的状态切换。
///
/// 已处理的输入即使没有事件，也可能已经修改数据。
/// 调用方应根据结果变体判断是否处理，不能通过事件是否为空决定是否重投输入。
/// 结果只描述本次输入的处理情况；普通机器派发时先调用幂等的 [`crate::Machine::init`]。
/// 若此次派发实际完成了初始化，即使输入未处理，初始化对状态和上下文的修改也不会撤销。
#[must_use]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DispatchResult<E> {
    /// 本次输入已经处理，状态转换及其钩子均已处理完毕。
    Handled {
        /// 本次处理产生的有序事件，允许为空。
        events: Box<[E]>,
    },
    /// 当前状态未处理本次输入，驱动未执行由状态切换触发的钩子。
    ///
    /// 若普通机器此次派发实际完成了初始化，初始化效果会保留，不能将该结果解释为整次派发无修改。
    /// 状态实现须保证本次输入处理没有产生领域修改；驱动不自动回滚。
    /// 该结果不携带拒绝原因或领域事件，也不自动向父机冒泡。
    Unhandled,
}

impl<E> DispatchResult<E> {
    /// 判断本次输入是否已经处理，与事件数量无关。
    ///
    /// 该判断仅反映输入的处理结果，不反映本次派发是否执行了初始化。
    pub const fn is_handled(&self) -> bool {
        matches!(self, Self::Handled { .. })
    }

    /// 借用本次处理产生的事件；未处理时返回 `None`。
    ///
    /// `Some(&[])` 表示已处理但没有事件，不能视为未处理。
    pub fn events(&self) -> Option<&[E]> {
        match self {
            Self::Handled { events } => Some(events),
            Self::Unhandled => None,
        }
    }
}

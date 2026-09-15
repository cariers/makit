//! 阶段单次输入处理与整手结束的结果类型。

use makit_machine::Outcome;

/// 一次阶段输入成功处理的结果，由引擎适配为基础状态机的推进结果。
///
/// [`Self::Continue`] 和 [`Self::Finished`] 均表示输入已处理，事件保持原顺序且允许为空。
/// 继续推进时直接使用基础 [`Outcome`] 保存事件与阶段转换；整手结束时另行交付最终结果。
/// 未处理与业务错误由 [`makit_machine::Error`] 表达，不包含在本类型中。
/// `P` 指定阶段转换的目标类型；[`crate::PhaseResult<V>`] 将其固定为 `V::Phase`，
/// 不取决于具体的 [`crate::Phase`] 实现类型。
/// 该值本身不应用状态转换；交由引擎与基础驱动消费后，派发成功只向调用者返回事件，
/// 整手最终结果由引擎终态持有。
#[must_use]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PhaseOutcome<P, E, O> {
    /// 本次输入已处理，整手流程继续，可保持当前阶段或进入下一阶段。
    ///
    /// 内部 [`Outcome`] 保存有序事件及待应用的阶段转换，不包含整手结束。
    Continue(Outcome<P, E>),
    /// 本次输入已处理，整手流程结束并交付最终结果。
    Finished {
        /// 本次有序事件，包括需要向调用方返回的最后一批业务事实，允许为空。
        events: Box<[E]>,
        /// 移入引擎终态的最终业务结果，不需要克隆。
        output: O,
    },
}

impl<P, E, O> PhaseOutcome<P, E, O> {
    /// 构造已处理且继续当前阶段的结果，事件可由数组、`Vec` 或装箱切片传入。
    pub fn stay(events: impl Into<Box<[E]>>) -> Self {
        Self::Continue(Outcome::stay(events))
    }

    /// 构造已处理且进入指定阶段的结果，事件可由数组、`Vec` 或装箱切片传入。
    pub fn to(phase: P, events: impl Into<Box<[E]>>) -> Self {
        Self::Continue(Outcome::to(phase, events))
    }

    /// 构造已处理且结束整手的结果，同时交付最终结果和最后一批事件。
    ///
    /// 事件可由数组、`Vec` 或装箱切片传入；空事件不改变本次已处理和整手结束的含义。
    pub fn finished(output: O, events: impl Into<Box<[E]>>) -> Self {
        Self::Finished {
            events: events.into(),
            output,
        }
    }
}

//! 阶段单次输入处理与整手结束的结果类型。

use makit_machine::Transition;

/// 一次阶段输入的处理结果，由引擎适配为基础状态机的推进结果。
///
/// [`Self::Handled`] 和 [`Self::Finished`] 均表示输入已处理，事件保持原顺序且允许为空。
/// 只有 [`Self::Unhandled`] 表示未处理，不能因结束整手而遗漏输入记录或最后一批事件。
/// 状态转换尚未由基础驱动应用，该值不是对外的 [`makit_machine::DispatchResult`]。
#[must_use]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PhaseOutcome<P, E, O> {
    /// 本次输入未处理，不携带事件或转换。
    ///
    /// 实现必须在修改阶段、局上下文或外部领域事实前作出该决定，引擎不会回滚已有修改。
    Unhandled,
    /// 本次输入已处理，整手流程继续，可保持当前阶段或进入下一阶段。
    Handled {
        /// 本次处理产生的有序事件，允许为空。
        events: Box<[E]>,
        /// 本次要求应用的阶段转换，不包含整手结束。
        transition: Transition<P>,
    },
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
        Self::Handled {
            events: events.into(),
            transition: Transition::Stay,
        }
    }

    /// 构造已处理且进入指定阶段的结果，事件可由数组、`Vec` 或装箱切片传入。
    pub fn to(phase: P, events: impl Into<Box<[E]>>) -> Self {
        Self::Handled {
            events: events.into(),
            transition: Transition::To(phase),
        }
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

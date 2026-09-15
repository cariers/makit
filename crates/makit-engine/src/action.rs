//! 可包含多次交互的行动协议及其执行进度。

/// 启动准备阶段的行动模块，具体行动不重导出到 crate 根部。
pub mod setup;

use makit_context::{Context, Variant};

/// 可独立编排的麻将行动。
///
/// 行动实例可以保存收集中的玩家选择等多次交互数据；阶段实现负责决定何时启动行动、
/// 转交后续输入及处理行动完成，不代替行动内部的业务状态。启动和后续输入均可访问局上下文。
///
/// 启动不提供拒绝结果，调用方应先满足具体行动的前置条件。后续输入返回
/// [`ActionOutcome::Unhandled`] 前必须完成只读判断，且不得修改行动自身、局上下文、
/// 随机进度或外部领域事实。该要求依赖实现遵守，不提供自动回滚。
pub trait Action<V>
where
    V: Variant,
{
    /// 行动接受的输入，`'ipt` 表示输入内部可借用的数据的生命周期。
    ///
    /// 输入仅借用至本次处理结束；多次交互中需要保留的数据应由行动取得所有权。
    type Input<'ipt>;

    /// 行动处理产生的事件，业务含义和面向客户端的投影由上层定义。
    type Event;

    /// 使用局上下文启动行动，返回本次处理的进度及事件。
    ///
    /// 可以直接返回 [`Progress::Complete`] 完成洗牌等单次操作，也可以返回
    /// [`Progress::Running`] 等待后续输入。启动没有未处理分支，前置条件由调用方保证。
    fn start(&mut self, ctx: &mut Context<V>) -> Progress<Self::Event>;

    /// 在行动运行期间处理一次输入，返回是否处理及已处理时的进度和事件。
    ///
    /// [`ActionOutcome::Handled`] 包含 [`Progress::Running`] 或 [`Progress::Complete`]，
    /// 分别表示行动继续运行或完成，均不要求自动执行下一步。
    /// 返回 [`ActionOutcome::Unhandled`] 表示未处理，必须在任何领域修改发生前作出该决定。
    fn handle(
        &mut self,
        ctx: &mut Context<V>,
        input: &Self::Input<'_>,
    ) -> ActionOutcome<Self::Event>;
}

/// 行动在启动或一次已处理输入后的执行进度。
///
/// 两种进度均携带有序事件，允许为空；未处理输入由 [`ActionOutcome::Unhandled`] 表达。
/// 本类型不指示通用状态机自动循环、切换阶段或构造后续输入。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Progress<T> {
    /// 本次处理已完成，行动继续运行，等待后续输入。
    Running(Box<[T]>),
    /// 本次处理已完成，行动结束。
    Complete(Box<[T]>),
}

/// 行动对一次后续输入的处理结果。
///
/// 已处理时返回执行进度及事件，未处理时不附带事件；该结果不直接指定阶段转换。
/// 行动启动使用 [`Progress`]，不会返回本类型的未处理分支。
#[must_use]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ActionOutcome<T> {
    /// 本次输入已处理，包含行动进度与有序事件，是否切换阶段由上层决定。
    Handled(Progress<T>),
    /// 行动未处理本次输入，不附带事件或执行进度。
    ///
    /// 不会自动回滚实现已做的修改；实现须在修改行动或领域数据之前决定返回该结果。
    Unhandled,
}

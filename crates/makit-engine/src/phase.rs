//! 具体玩法的阶段执行契约及整手结果协议。

mod outcome;

use makit_context::{Context, Variant};

pub use outcome::PhaseOutcome;

/// 由具体玩法定义的阶段状态及其输入处理。
///
/// 各业务阶段由实现类型的枚举分支或内部结构表达，共用 [`PhaseVariant`] 声明的协议类型。
/// 行动完成后的阶段编排由该实现决定；只有整手玩法结束时才交付最终结果。
pub trait Phase<V>
where
    Self: Sized,
    V: PhaseVariant,
{
    /// 处理一次输入，返回未处理、继续推进或整手结束。
    ///
    /// 返回 [`PhaseOutcome::Unhandled`] 前必须完成只读判断，不得修改阶段、局上下文、
    /// 随机进度或外部领域事实；这是实现契约，引擎不提供自动回滚。
    /// [`PhaseOutcome::Handled`] 和 [`PhaseOutcome::Finished`] 均表示本次输入已处理，
    /// 即使事件为空也会记录输入。
    ///
    /// 返回 [`PhaseOutcome::Finished`] 时必须完成本次业务处理，准备好最终结果和最后一批事件。
    /// 阶段实例随后会被丢弃，结果不得借用该实例或仅在本次调用中有效的数据。
    fn handle(
        &mut self,
        ctx: &mut Context<V>,
        input: &V::Input,
    ) -> PhaseOutcome<Self, V::Event, V::Output>;
}

/// 定义玩法的输入、事件、整手结果及主阶段入口。
///
/// 输入、事件及整手结果由本 trait 统一声明，[`Phase`] 使用这些类型处理阶段逻辑。
pub trait PhaseVariant: Variant {
    /// 玩法主阶段的实现类型。
    type Phase: Phase<Self>;

    /// 阶段输入；引擎保存每次已处理输入的副本。
    type Input: Clone;

    /// 本次处理产生的事件；引擎同时留存事件副本并向调用方返回原始事件。
    type Event: Clone;

    /// 整手玩法结束后交付的业务结果，由具体变体定义。
    ///
    /// 结果移动到引擎终态，不要求实现 `Clone`、`Default` 或序列化能力。
    /// 没有专属结果的玩法可以使用 `()`。
    type Output;

    /// 根据既有局上下文创建初始阶段，不执行阶段输入。
    fn initial_phase(ctx: &Context<Self>) -> Self::Phase;
}

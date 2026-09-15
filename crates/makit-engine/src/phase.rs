//! 具体玩法的阶段执行契约及整手结果协议。

mod outcome;

use makit_context::{Context, Variant};
use makit_machine::Error;

pub use outcome::PhaseOutcome;

/// 使用变体协议的阶段执行结果。
///
/// `PhaseResult<V>` 使用 [`PhaseVariant`] 声明的主阶段、事件、整手结果与业务错误。
/// 其中 [`makit_machine::Transition::To`] 的目标固定为 `V::Phase`，与调用
/// [`Phase::handle`] 的具体实现类型是否为主阶段无关。
/// 这只是 [`Result`] 的类型别名，不改变 [`PhaseOutcome`] 的成功语义，
/// 也不改变 [`Error`] 对未处理与业务失败的区分。
pub type PhaseResult<V> = Result<
    PhaseOutcome<
        <V as PhaseVariant>::Phase,
        <V as PhaseVariant>::Event,
        <V as PhaseVariant>::Output,
    >,
    Error<<V as PhaseVariant>::Error>,
>;

/// 由具体玩法定义的阶段状态及其输入处理。
///
/// 各业务阶段由实现类型的枚举分支或内部结构表达，共用 [`PhaseVariant`] 声明的协议类型。
/// 行动完成后的阶段编排由该实现决定；只有整手玩法结束时才交付最终结果。
/// 实现类型可以是主阶段，也可以是主阶段持有的其他处理者；
/// 所有实现统一返回 [`PhaseResult<V>`]，阶段转换目标始终是 `V::Phase`。
pub trait Phase<V>
where
    Self: Sized,
    V: PhaseVariant,
{
    /// 处理一次输入，成功时返回继续推进或整手结束的结果。
    ///
    /// `Ok` 内的 [`PhaseOutcome::Continue`] 和 [`PhaseOutcome::Finished`] 均表示
    /// 本次输入已处理；交由引擎消费时，即使事件为空也会记录输入。
    /// [`PhaseOutcome::Continue`] 使用 [`makit_machine::Outcome`] 保存事件与阶段转换。
    /// [`makit_machine::Transition::Stay`] 请求保留引擎当前主阶段，
    /// [`makit_machine::Transition::To`] 交付下一主阶段 `V::Phase`，由引擎适配后交给机器驱动应用。
    /// 其他实现类型也使用这一主阶段转换协议，本方法本身不执行实例替换。
    ///
    /// 返回 [`PhaseOutcome::Finished`] 时必须完成本次业务处理，准备好最终结果和最后一批事件。
    /// 当引擎消费此结果并由机器驱动完成终态转换时，原主阶段实例才被丢弃；
    /// 直接调用本方法不会自动丢弃实现者。结果不得借用该实例或仅在本次调用中有效的数据。
    ///
    /// # Errors
    ///
    /// 输入不适用于当前阶段时返回 [`Error::Unhandled`]；已识别输入的业务校验失败时
    /// 返回 [`Error::Custom`]，具体错误由 [`PhaseVariant::Error`] 定义。
    /// 行动直接返回自身业务错误，由阶段映射为变体错误并包装到 [`Error::Custom`]。
    /// [`Error::map_custom`] 用于转换已经包装的子阶段或子机错误，保留未处理的原始含义。
    ///
    /// 返回任一错误时，必须保持调用前的阶段、局上下文与领域事实。实现应先检查或准备
    /// 候选结果，再提交修改；不能先推进子机、消耗随机进度或成功执行行动后再返回错误。
    /// 引擎不会回滚已有修改，错误不会产生成功输入记录、事件记录或阶段切换。
    fn handle(&mut self, ctx: &mut Context<V>, input: &V::Input) -> PhaseResult<V>;
}

/// 定义玩法的输入、事件、整手结果、业务错误及主阶段入口。
///
/// 执行协议由本 trait 统一声明，[`Phase`] 使用这些类型处理阶段逻辑。
/// 主阶段在这里仅声明类型；`V::Phase: Phase<V>` 的执行约束由引擎的
/// [`State`](makit_machine::State) 实现要求。构造引擎或尚未初始化的机器时
/// 不需要这一执行约束，初始化和派发时才需要。
pub trait PhaseVariant: Variant {
    /// 玩法主阶段的数据类型，也是 [`PhaseResult<Self>`] 中的阶段转换目标。
    ///
    /// 此处不要求实现 [`Phase<Self>`]；该约束在引擎状态获得执行能力时检查。
    type Phase;

    /// 阶段输入；引擎保存每次已处理输入的副本。
    type Input: Clone;

    /// 本次处理产生的事件；引擎同时留存事件副本并向调用方返回原始事件。
    type Event: Clone;

    /// 整手玩法结束后交付的业务结果，由具体变体定义。
    ///
    /// 结果移动到引擎终态，不要求实现 `Clone`、`Default` 或序列化能力。
    /// 没有专属结果的玩法可以使用 `()`。
    type Output;

    /// 阶段识别输入后可能返回的业务错误，由 [`Error::Custom`] 承载。
    ///
    /// 无业务错误时可以使用 [`std::convert::Infallible`]，仍可返回 [`Error::Unhandled`]。
    /// 不要求错误支持克隆、线程共享或序列化。
    type Error: std::error::Error;

    /// 根据既有局上下文创建初始阶段数据，不执行阶段输入。
    fn initial_phase(ctx: &Context<Self>) -> Self::Phase;
}

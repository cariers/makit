//! 麻将行动、阶段协议及其通用状态机适配。
//!
//! [`Action`] 实现启动和后续输入的业务处理，可在自身保存多次交互所需的数据。
//! 启动和后续输入均返回 `Result`，成功值 [`Progress`] 区分继续运行与行动完成。
//! 失败时直接返回 [`Action::Error`] 定义的业务错误，行动不包含未处理分支。
//! 输入路由、调用行动、解释完成进度及安排后续输入由阶段实现负责。
//!
//! [`PhaseVariant`] 声明输入、事件、业务错误及整手结果 [`PhaseVariant::Output`]，并指定主阶段。
//! [`Phase`] 使用这些类型处理阶段逻辑；[`PhaseOutcome::Continue`] 复用
//! [`makit_machine::Outcome`] 保存事件与阶段转换，
//! [`PhaseOutcome::Finished`] 同时交付整手最终结果与最后一批事件，
//! 行动的 [`Progress::Complete`] 则仅表示该行动结束。
//! 阶段使用 [`makit_machine::Error`] 区分输入未处理与业务失败，并将行动错误映射到
//! [`PhaseVariant::Error`] 后包装为 [`makit_machine::Error::Custom`]。
//! [`PhaseResult<V>`] 固定使用变体声明的主阶段、事件、整手结果与业务错误。
//! 其他类型也可以实现 [`Phase<V>`]，但返回的阶段转换目标统一为 `V::Phase`。
//! [`PhaseVariant::Phase`] 仅声明数据类型；引擎状态在提供执行能力时要求
//! `V::Phase: Phase<V>`。构造引擎或尚未初始化的机器只依赖数据契约，初始化和派发才需要执行约束。
//!
//! [`Engine`] 持有局上下文及已处理输入、事件的记录，[`EngineInput`] 区分启动和阶段输入。
//! `EngineState<P, O>::Finished(O)` 独立持有最终结果；派发完成后可通过
//! `machine.state().output()` 借用结果，不要求结果可克隆，也不在引擎上下文中重复保存。
//! 事件的客户端可见范围与投影由上层定义，运行进度不会自动产生下一次输入。
//! 错误不追加成功记录或执行状态切换；实现必须先校验再修改，引擎不提供自动回滚。
//!
//! 具体行动按 [`action`] 的子模块组织，crate 根部只重导出通用协议与适配类型。

mod engine;
mod phase;

/// 行动协议、执行进度及按领域组织的行动模块。
pub mod action;

pub use action::{Action, Progress};
pub use engine::{Engine, EngineInput, EngineState};
pub use phase::{Phase, PhaseOutcome, PhaseResult, PhaseVariant};

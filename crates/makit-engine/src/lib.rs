//! 麻将行动、阶段协议及其通用状态机适配。
//!
//! [`Action`] 实现启动和后续输入的业务处理，可在自身保存多次交互所需的数据。
//! 启动返回 [`Progress`]，区分继续运行与行动完成；后续输入返回 [`ActionOutcome`]，
//! 还可以表达未处理。调用行动、解释完成进度及安排后续输入由阶段实现负责。
//!
//! [`Phase`] 关联输入、事件及整手结果 [`Phase::Output`]。阶段处理通过 [`PhaseOutcome`]
//! 表达未处理、继续推进或整手结束；[`PhaseOutcome::Finished`] 同时交付最终结果与
//! 最后一批事件，行动的 [`Progress::Complete`] 则仅表示该行动结束。
//!
//! [`Engine`] 持有局上下文及已处理输入、事件的记录，[`EngineInput`] 区分启动和阶段输入。
//! `EngineState<P, O>::Finished(O)` 独立持有最终结果；派发完成后可通过
//! `machine.state().output()` 借用结果，不要求结果可克隆，也不在引擎上下文中重复保存。
//! 事件的客户端可见范围与投影由上层定义，运行进度不会自动产生下一次输入。
//!
//! 具体行动按 [`action`] 的子模块组织，crate 根部只重导出通用协议与适配类型。

mod engine;
mod phase;

/// 行动协议、执行进度及按领域组织的行动模块。
pub mod action;

pub use action::{Action, ActionOutcome, Progress};
pub use engine::{Engine, EngineInput, EngineState};
pub use phase::{Phase, PhaseOutcome, PhaseVariant};

//! 面向麻将变体开发的统一公共入口。
//!
//! 根 crate 显式导出常用领域类型、上下文、状态机与引擎协议；各层完整 API 分别位于
//! [`core`]、[`context`]、[`machine`] 和 [`engine`]。这些入口直接重导出现有类型，
//! 不创建包装类型，也不改变底层 crate 的职责。
//!
//! ```mermaid
//! flowchart TD
//!     Makit[makit：统一入口] --> Core[makit-core：领域类型]
//!     Makit --> Context[makit-context：上下文与组合操作]
//!     Makit --> Machine[makit-machine：通用状态机]
//!     Makit --> Engine[makit-engine：行动与阶段协议]
//!     Context --> Core
//!     Engine --> Core
//!     Engine --> Context
//!     Engine --> Machine
//! ```
//!
//! [`Tile`] 表示牌面，同牌副本没有独立身份；[`Context`] 维护通用牌区与变体扩展。
//! [`PhaseVariant`] 声明玩法的输入、事件、错误、最终结果和主阶段，具体 [`Phase`] 决定行动
//! 如何启动及切换。客户端可见信息与具体玩法的合法性由使用方定义。
//!
//! 细分错误、操作结果及具体行动保留在所属命名空间，例如
//! [`context::ContextError`]、[`context::WallEnd`] 和 [`engine::action::setup`]。
//! [`Outcome`] 是通用状态机的成功推进结果，[`PhaseOutcome::Continue`] 复用该结果，
//! [`PhaseOutcome::Finished`] 交付整手终态输出。
//! [`Progress`] 描述单个行动的执行进度。行动通过 `Result` 直接返回业务错误；
//! 阶段和机器使用 [`Error`] 区分未处理与业务失败。
//! [`PhaseResult<V>`] 使用变体声明的阶段、事件、最终结果与业务错误，转换目标固定为 `V::Phase`。
//! [`PhaseVariant`] 只声明阶段类型；引擎初始化和派发时要求 `V::Phase: Phase<V>`。

/// 通用上下文、组合操作与数据层变体契约。
pub use makit_context as context;
/// 牌面、座位、牌区与其他基础领域类型。
pub use makit_core as core;
/// 行动、阶段与麻将引擎协议。
pub use makit_engine as engine;
/// 独立上下文驱动的通用状态机。
pub use makit_machine as machine;

pub use makit_context::{Context, SeatContext, Variant};
pub use makit_core::{
    Discard, DiscardId, EastSeat, Hand, Meld, Melds, River, Seat, SeatMap, SeatMask, Seed, Tile,
    TileCounts, TileMask, Wall, Wildcard, Wind,
};
pub use makit_engine::{
    Action, Engine, EngineInput, EngineState, Phase, PhaseOutcome, PhaseResult, PhaseVariant,
    Progress,
};
pub use makit_machine::{
    Error, Initialized, IntoMachineState, Machine, MachineContext, Outcome, State, StrictMachine,
    Transition, Uninitialized,
};

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
//! [`PhaseVariant`] 声明玩法的输入、事件、最终结果和主阶段，具体 [`Phase`] 决定行动
//! 如何启动及切换。客户端可见信息与具体玩法的合法性由使用方定义。
//!
//! 细分错误、操作结果及具体行动保留在所属命名空间，例如
//! [`context::ContextError`]、[`context::WallEnd`] 和 [`engine::action::setup`]。
//! [`Outcome`] 是通用状态机的推进结果，[`PhaseOutcome`] 可以交付整手终态输出，
//! [`ActionOutcome`] 则描述单个行动对后续输入的处理结果。

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
    Action, ActionOutcome, Engine, EngineInput, EngineState, Phase, PhaseOutcome, PhaseVariant,
    Progress,
};
pub use makit_machine::{
    DispatchResult, Initialized, IntoMachineState, Machine, MachineContext, Outcome, State,
    StrictMachine, Transition, Uninitialized,
};

//! 使用独立上下文、只读 Guard 与无错误推进契约的通用状态机。
//!
//! [`MachineContext`] 定义单个机器持有的上下文及其状态、输入和事件类型。
//! [`State`] 实现输入准入、推进及进入和退出钩子；[`Step`] 与 [`Transition`]
//! 仅供状态实现描述本次推进，由机器驱动消费。
//!
//! [`IntoMachineState`] 为实现 [`MachineContext`] 的上下文提供构造入口：
//! [`into_machine`](IntoMachineState::into_machine) 创建延时初始化的 [`Machine`]，
//! [`into_strict_machine`](IntoMachineState::into_strict_machine) 创建处于
//! [`Uninitialized`] 阶段的 [`StrictMachine`]。
//! 普通机器提供幂等的 [`Machine::init`]，每次派发输入也会先调用该方法，
//! 由它判断是否需要执行初始进入钩子。严格机器通过 `init` 显式转换到 [`Initialized`]
//! 阶段后才能派发。两种包装器共用内部驱动，并通过只读解引用访问上下文。
//!
//! [`DispatchResult`] 表达已经完成的执行结果：Guard 未通过，或已执行及其有序事件。
//! 事件为空仍可能表示已执行，不应据此重新投递输入。状态切换完成后才返回执行结果。
//! 普通机器派发输入时先调用 `init` 再检查 Guard；若本次调用实际完成了初始化，
//! 即使结果为 Guard 未通过，初始化及其对状态和上下文的修改也会保留。
//!
//! ```mermaid
//! flowchart TD
//!     Input[Machine 接收输入] --> Ensure[调用幂等 init]
//!     Ensure --> Initialized{已经初始化}
//!     Initialized -->|是| Guard{只读 Guard}
//!     Initialized -->|否| Init[执行初始进入钩子]
//!     Init --> Mark[记录已初始化]
//!     Mark --> Guard
//!     Strict[已初始化 StrictMachine 接收输入] --> Guard
//!     Guard -->|未通过| Blocked[返回 GuardBlocked]
//!     Guard -->|通过| Advance[advance 产生 Step]
//!     Advance --> Change{Transition}
//!     Change -->|Stay| Executed[返回 Executed 和事件]
//!     Change -->|To| Exit[旧状态退出钩子]
//!     Exit --> Replace[替换状态]
//!     Replace --> Entry[新状态进入钩子]
//!     Entry --> Executed
//! ```
//!
//! 父状态可以持有拥有另一种上下文的完整子机器。子机器不隐式共享父上下文，
//! 输入路由、事件汇总、等待条件及流程完成语义均由父状态或具体业务定义。
//! Guard 是实现契约，不提供跨机器事务、panic 回滚或外部副作用恢复。

mod dispatch;
mod machine;
mod state;
mod transition;

pub use dispatch::DispatchResult;
pub use machine::{Initialized, IntoMachineState, Machine, StrictMachine, Uninitialized};
pub use state::State;
pub use transition::{Step, Transition};

/// 单个机器独立持有的上下文及其关联类型。
///
/// 上下文本身就是 `Self`，由机器取得所有权；嵌套机器可以使用不同的上下文类型。
/// 该 trait 不要求上下文或关联类型实现克隆、序列化及线程共享能力。
pub trait MachineContext
where
    Self: Sized,
{
    /// 当前机器使用的状态类型。
    ///
    /// 执行初始化和输入推进时，该类型需要实现 [`State<Self>`]。
    type State;

    /// 当前机器接受的输入，`'ipt` 表示输入内部允许借用的数据的生命周期。
    ///
    /// 同一次输入以共享引用交给 Guard 和推进方法。事件类型独立于 `'ipt`，
    /// 因而不能仅借用本次调用期间有效的输入数据；需要留存的内容应由实现取得所有权。
    type Input<'ipt>;

    /// 输入推进产生的事件类型，其业务含义与可见范围由上层定义。
    type Event;

    /// 构造初始状态值，不执行进入钩子。
    ///
    /// 需要根据上下文设置的初始数据，可在 [`State::handle_entry_action`] 中完成。
    fn initial_state() -> Self::State;
}

//! 使用独立上下文与输入处理结果驱动的通用状态机。
//!
//! [`MachineContext`] 定义单个机器持有的上下文及其状态、输入、事件和业务错误类型。
//! [`State`] 实现输入处理及进入和退出钩子；成功值 [`Outcome`] 携带事件与
//! [`Transition`]，由机器驱动消费。错误值 [`Error`] 区分未处理与具体业务错误。
//!
//! [`IntoMachineState`] 为实现 [`MachineContext`] 的上下文提供构造入口：
//! [`into_machine`](IntoMachineState::into_machine) 创建延时初始化的 [`Machine`]，
//! [`into_strict_machine`](IntoMachineState::into_strict_machine) 创建处于
//! [`Uninitialized`] 阶段的 [`StrictMachine`]。
//! 普通机器提供幂等的 [`Machine::init`]，每次派发输入也会先调用该方法，
//! 由它判断是否需要执行初始进入钩子。严格机器通过 `init` 显式转换到 [`Initialized`]
//! 阶段后才能派发。两种包装器共用内部驱动，并通过只读解引用访问上下文。
//!
//! [`Machine::dispatch`] 和 [`StrictMachine::dispatch`] 使用 [`Result`] 返回有序事件
//! 或执行错误。成功时事件允许为空，不应据此重新投递输入；状态切换完成后才返回事件。
//! 普通机器派发输入时先调用 `init` 再调用 [`State::advance`]；
//! 若本次调用实际完成了初始化，即使后续返回错误，初始化对状态和上下文的修改也会保留。
//!
//! ```mermaid
//! flowchart TD
//!     Input[Machine 接收输入] --> Ensure[调用幂等 init]
//!     Ensure --> Initialized{已经初始化}
//!     Initialized -->|是| Advance[调用 advance]
//!     Initialized -->|否| Init[执行初始进入钩子]
//!     Init --> Mark[记录已初始化]
//!     Mark --> Advance
//!     Strict[已初始化 StrictMachine 接收输入] --> Advance
//!     Advance --> Result{处理结果}
//!     Result -->|Err| Error[返回 Unhandled 或 Custom 错误]
//!     Result -->|Ok| Change{Outcome 中的 Transition}
//!     Change -->|Stay| Events[返回 Ok 和事件]
//!     Change -->|To| Exit[旧状态退出钩子]
//!     Exit --> Replace[替换状态]
//!     Replace --> Entry[新状态进入钩子]
//!     Entry --> Events
//! ```
//!
//! 父状态可以持有拥有另一种上下文的完整子机器。子机器不隐式共享父上下文，
//! 输入路由、事件汇总、等待条件及流程完成语义均由父状态或具体业务定义。
//! 返回任一错误前保持方法入口的状态与上下文属于实现契约。
//! 错误不会触发状态转换钩子，驱动不提供跨机器事务、数据回滚或外部副作用恢复。

mod error;
mod machine;
mod outcome;
mod state;
mod transition;

pub use error::Error;
pub use machine::{Initialized, IntoMachineState, Machine, StrictMachine, Uninitialized};
pub use outcome::Outcome;
pub use state::State;
pub use transition::Transition;

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
    /// 输入以共享引用交给推进方法。事件类型独立于 `'ipt`，
    /// 因而不能仅借用本次调用期间有效的输入数据；需要留存的内容应由实现取得所有权。
    type Input<'ipt>;

    /// 输入推进产生的事件类型，其业务含义与可见范围由上层定义。
    type Event;

    /// 输入执行失败时携带的具体业务错误。
    ///
    /// 当前状态不处理输入时使用 [`Error::Unhandled`]；业务错误由 [`Error::Custom`]
    /// 包装。该类型不额外要求克隆、线程共享或静态生命周期。
    type Error: std::error::Error;

    /// 构造初始状态值，不执行进入钩子。
    ///
    /// 需要根据上下文设置的初始数据，可在 [`State::handle_entry_action`] 中完成。
    fn initial_state() -> Self::State;
}

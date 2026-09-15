//! 使用独立上下文与输入处理结果驱动的通用状态机。
//!
//! [`MachineContext`] 定义单个机器持有的上下文及其状态、输入、事件和业务错误类型。
//! 上下文提供派发前观察、成功收尾、业务错误观察及转换前后维护的默认空钩子。
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
//! 或执行错误。成功时事件允许为空，不应据此重新投递输入；状态切换与成功收尾完成后才返回事件。
//! 普通机器派发输入时先调用 `init`，初始化仅执行初始状态进入钩子，
//! 随后的 [`MachineContext::before_dispatch`] 观察初始化后的状态，再由 [`State::advance`] 推进。
//! 若本次调用实际完成了初始化，即使后续返回错误，初始化对状态和上下文的修改也会保留。
//!
//! ```mermaid
//! flowchart TD
//!     Input[Machine 接收输入] --> Ensure[调用幂等 init]
//!     Ensure --> Initialized{已经初始化}
//!     Initialized -->|是| BeforeDispatch[context.before_dispatch]
//!     Initialized -->|否| Init[执行初始进入钩子]
//!     Init --> Mark[记录已初始化]
//!     Mark --> BeforeDispatch
//!     Strict[已初始化 StrictMachine 接收输入] --> BeforeDispatch
//!     BeforeDispatch --> Advance[调用 state.advance]
//!     Advance --> Result{处理结果}
//!     Result -->|Unhandled| Error[返回原始 Err]
//!     Result -->|Custom| DispatchError[context.dispatch_error 借用业务错误]
//!     DispatchError --> Error
//!     Result -->|Ok| Change{Outcome 中的 Transition}
//!     Change -->|Stay| AfterDispatch[context.after_dispatch 借用事件]
//!     Change -->|To| BeforeTransition[context.before_transition]
//!     BeforeTransition --> Exit[旧状态退出钩子]
//!     Exit --> Replace[替换状态并丢弃旧值]
//!     Replace --> Entry[新状态进入钩子]
//!     Entry --> AfterTransition[context.after_transition]
//!     AfterTransition --> AfterDispatch
//!     AfterDispatch --> Events[返回原始 Ok 事件]
//! ```
//!
//! 父状态可以持有拥有另一种上下文的完整子机器。子机器不隐式共享父上下文，
//! 仅在实际派发时调用自己的钩子；输入路由、事件汇总、等待条件及流程完成语义均由父状态
//! 或具体业务定义。钩子不会自动产生客户端事件，也不承担持久化或恢复协议。
//! 返回任一错误前保持方法入口的状态与上下文属于实现契约。
//! 错误不会触发状态转换钩子，驱动不提供跨机器事务、数据回滚或外部副作用恢复。
//! [`Error::Unhandled`] 没有结果回调；[`Error::Custom`] 仅通知业务错误钩子。
//! 初始化、推进、钩子或析构中的 panic 均不被捕获，后续钩子不保证执行。

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

/// 单个机器独立持有的上下文、关联类型与执行边界钩子。
///
/// 上下文本身就是 `Self`，由机器取得所有权；嵌套机器可以使用不同的上下文类型。
/// 该 trait 不要求上下文或关联类型实现克隆、序列化及线程共享能力。
/// 五个执行钩子均有默认空实现且返回 `()`；它们不能拒绝输入、指定另一转换，
/// 也不能追加本次派发返回的事件。可能失败的业务计算仍由 [`State::advance`] 完成。
/// 派发前和业务错误钩子仅用于观察，成功收尾与转换钩子用于保证能完成的上下文维护。
/// 所有钩子都只借用状态，正式状态替换由驱动应用 [`Transition::To`]。
///
/// 初始化、克隆或丢弃机器时不调用这些执行钩子。钩子和状态方法中的 panic 不被捕获，
/// 后续钩子不保证执行；成功收尾不承担 `finally` 职责。
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

    /// 初始化完成后、推进输入前，观察当前状态和本次输入。
    ///
    /// 每次正常进入派发驱动时调用一次；`state` 已包含初始进入钩子的修改。
    /// 此时尚未确定输入是否成功，不能通过内部可变性或外部副作用修改影响后续规则、
    /// 事件或重放结果的领域数据。诊断信息不应作为业务成功记录或恢复依据。
    /// 返回未处理或发生 panic 时，不保证有对应的结果回调。
    #[allow(unused_variables)]
    fn before_dispatch(&self, state: &Self::State, input: &Self::Input<'_>) {}

    /// 推进成功且所需转换完成后，维护成功记录并收尾。
    ///
    /// `state` 是最终当前状态：保留状态时已包含本次推进修改，转换时已完成进入钩子
    /// 和 [`after_transition`](Self::after_transition)。`events` 是即将原样返回的有序
    /// 事件切片，允许为空；可按需保留记录，但不能改写事件的业务含义。
    /// 本方法可修改上下文，不能补做可能失败的业务校验；两种错误均不调用本方法。
    #[allow(unused_variables)]
    fn after_dispatch(
        &mut self,
        state: &Self::State,
        input: &Self::Input<'_>,
        events: &[Self::Event],
    ) {
    }

    /// 推进返回业务错误时，观察当前状态、输入与具体错误。
    ///
    /// 仅对 [`Error::Custom`] 调用，`error` 直接借用其中的原始业务错误。
    /// 回调完成后驱动原样返回完整错误，不调用转换钩子或成功收尾；
    /// [`Error::Unhandled`] 直接返回，不调用本方法。
    /// `state` 是推进返回后的当前值；只有推进遵守错误契约时，它才与推进入口一致，
    /// 驱动不创建快照或回滚。观察时不得修改影响规则、事件或重放结果的领域数据。
    /// 本方法不接管错误，不自动重试、转发或映射错误，也不接收 panic。
    #[allow(unused_variables)]
    fn dispatch_error(&self, state: &Self::State, input: &Self::Input<'_>, error: &Self::Error) {}

    /// 推进成功且明确要求转换时，在旧状态退出前维护上下文。
    ///
    /// `from` 与上下文已包含本次推进的成功修改，并非输入处理前的快照；
    /// `to` 是即将进入的目标状态，尚未执行进入钩子。
    /// 仅 [`Transition::To`] 调用，即使目标与当前状态业务上相同也不省略；
    /// [`Transition::Stay`] 和初始化不调用。维护必须保证能够完成，不补做业务校验。
    #[allow(unused_variables)]
    fn before_transition(&mut self, from: &Self::State, to: &Self::State) {}

    /// 新状态完成进入钩子后，维护与当前状态对应的上下文。
    ///
    /// `state` 是完成进入钩子后的新状态，上下文已包含转换前维护及旧状态退出、新状态
    /// 进入的修改。旧状态已在新状态进入前丢弃，不为本方法额外保留。
    /// 仅 [`Transition::To`] 调用；正常结束后才执行 [`after_dispatch`](Self::after_dispatch)。
    /// 维护必须保证能够完成，不补做业务校验。
    #[allow(unused_variables)]
    fn after_transition(&mut self, state: &Self::State) {}
}

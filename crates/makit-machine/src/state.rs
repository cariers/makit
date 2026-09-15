use crate::{Error, MachineContext, Outcome};

/// 单个状态的输入处理与转换钩子。
///
/// 状态实现自行检查输入，成功时返回 [`Outcome`]，失败时返回 [`Error`]。
/// 两种错误都要求保持方法入口的状态与上下文；输入检查应在实际修改前完成。
/// 进入和退出钩子用于保证能够完成的状态维护，不返回错误或事件。
/// [`MachineContext`] 的钩子维护整台机器的执行边界，本 trait 的钩子维护当前状态自身。
/// 该契约不提供 panic 捕获、数据回滚或外部副作用撤销；panic 后不保证执行后续钩子。
pub trait State<M>
where
    Self: Sized,
    M: MachineContext,
{
    /// 处理本次输入，成功时返回有序事件及待应用的转换。
    ///
    /// 实现必须先检查输入是否能够处理，再修改当前状态和上下文。
    /// 返回任一错误时，必须保持调用本方法时的状态与上下文，不得留下子机推进、
    /// 随机进度消耗、领域事件写入或外部领域副作用。
    /// 驱动不自动回滚已发生的修改，也不自动要求父机转发或重试输入。
    ///
    /// 返回 `Ok` 表示本次输入已经处理，可以修改状态与上下文。
    /// 底层受检操作的前提或候选结果由实现预先确认，不能在部分修改后返回错误。
    /// 当前状态的局部数据可以直接修改；正式切换状态应通过 [`crate::Transition::To`]
    /// 表达，不应直接替换 `self` 后返回 [`crate::Transition::Stay`] 绕过转换钩子。
    ///
    /// 驱动在调用本方法前执行 [`MachineContext::before_dispatch`]。
    /// 已处理结果携带 [`crate::Transition::Stay`] 时不执行转换钩子；
    /// 携带 [`crate::Transition::To`] 时由驱动负责应用转换前维护、退出、替换、进入及转换后维护。
    /// 所有成功最后调用 [`MachineContext::after_dispatch`]，事件允许为空。
    /// 错误不执行转换或成功收尾；业务错误通知 [`MachineContext::dispatch_error`]，
    /// 未处理直接返回且没有结果回调。
    ///
    /// [`crate::Machine`] 每次派发会先调用幂等的 [`crate::Machine::init`]。
    /// 若此次派发实际完成了初始化，错误不会撤销进入本方法前已经完成的初始化。
    ///
    /// # Errors
    ///
    /// 当前状态不处理输入时返回 [`Error::Unhandled`]；
    /// 业务条件不满足时返回携带具体原因的 [`Error::Custom`]。
    fn advance(
        &mut self,
        ctx: &mut M,
        input: &M::Input<'_>,
    ) -> Result<Outcome<Self, M::Event>, Error<M::Error>>;

    /// 执行进入当前状态时保证能够完成的维护。
    ///
    /// 首次初始化以及切换到该状态时调用。普通机器可显式调用幂等的 [`crate::Machine::init`]，
    /// 每次派发也会先调用该方法，确保输入处理前已完成初始化。
    /// 严格机器由未初始化阶段的 `init` 完成初始化。
    /// 初始化不调用上下文派发或转换钩子；正常状态转换时，旧状态已经退出并丢弃，
    /// 本方法正常结束后才调用 [`MachineContext::after_transition`]。
    /// 默认实现不执行任何操作。
    #[allow(unused_variables)]
    fn handle_entry_action(&mut self, ctx: &mut M) {}

    /// 执行退出当前状态时保证能够完成的维护。
    ///
    /// 仅在推进要求切换状态时调用，此时能够看到本次推进对旧状态的修改，
    /// 上下文还包含 [`MachineContext::before_transition`] 已完成的维护。
    /// 保持当前状态或直接丢弃机器时不调用。
    /// 默认实现不执行任何操作。
    #[allow(unused_variables)]
    fn handle_exit_action(&mut self, ctx: &mut M) {}
}

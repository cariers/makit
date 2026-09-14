use crate::{MachineContext, Step};

/// 单个状态的输入准入、推进与转换钩子。
///
/// Guard 放行的输入必须能够完成推进，业务拒绝条件应在 Guard 或上层输入校验中判断。
/// 进入和退出钩子用于保证能够完成的状态维护，不返回错误或事件。
/// 该契约不提供 panic 捕获、数据回滚或外部副作用撤销。
pub trait State<M>
where
    Self: Sized,
    M: MachineContext,
{
    /// 逻辑只读地判断当前状态是否允许推进本次输入。
    ///
    /// 返回 `false` 时，驱动不会调用 [`Self::advance`] 或由状态切换触发的钩子。
    /// [`crate::Machine`] 每次派发会先调用幂等的 [`crate::Machine::init`]，再调用 Guard；
    /// 若此次派发实际完成了初始化，拒绝输入不会撤销初始化对状态和上下文的修改。
    /// 实现不得通过内部可变性推进本机或子机、消耗随机进度或写入领域事件。
    /// 这是实现契约，共享引用本身不能阻止内部可变性或外部副作用。
    /// 返回值不携带拒绝原因，也不自动要求父机转发或重试输入。
    fn guard(&self, ctx: &M, input: &M::Input<'_>) -> bool;

    /// 推进刚通过当前 Guard 的同一次输入，返回事件和待应用的状态转换。
    ///
    /// 调用方必须保证 Guard 与推进之间的状态、上下文和输入保持一致；
    /// 机器驱动会在同一次独占调用中完成这两个操作，不缓存 Guard 结果。
    /// 实现可修改当前状态和上下文，但必须完成本次推进，不能在部分修改后中止。
    /// 底层受检操作的前提由 Guard 和推进逻辑共同保证。
    /// 当前状态的局部数据可以直接修改；正式切换状态应通过 [`crate::Transition::To`]
    /// 表达，不应直接替换 `self` 后返回 [`crate::Transition::Stay`] 绕过转换钩子。
    ///
    /// 返回 [`crate::Transition::Stay`] 时不执行转换钩子；
    /// 返回 [`crate::Transition::To`] 时由驱动负责退出旧状态并进入新状态。
    fn advance(&mut self, ctx: &mut M, input: &M::Input<'_>) -> Step<Self, M::Event>;

    /// 执行进入当前状态时保证能够完成的维护。
    ///
    /// 首次初始化以及切换到该状态时调用。普通机器可显式调用幂等的 [`crate::Machine::init`]，
    /// 每次派发也会先调用该方法，确保 Guard 检查前已完成初始化。
    /// 严格机器由未初始化阶段的 `init` 完成初始化。
    fn handle_entry_action(&mut self, ctx: &mut M);

    /// 执行退出当前状态时保证能够完成的维护。
    ///
    /// 仅在推进要求切换状态时调用，此时能够看到本次推进对旧状态和上下文的修改。
    /// 保持当前状态或直接丢弃机器时不调用。
    fn handle_exit_action(&mut self, ctx: &mut M);
}

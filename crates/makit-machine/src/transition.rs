/// 一次状态推进要求机器应用的转换。
///
/// 该类型属于 [`crate::State::advance`] 的成功协议；[`crate::Machine::dispatch`]
/// 和 [`crate::StrictMachine::dispatch`] 负责完成转换，成功时只向调用方返回事件。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Transition<S> {
    /// 保留当前状态，不调用退出或进入钩子。
    ///
    /// 推进仍可以修改当前状态内部的数据和上下文。
    Stay,
    /// 退出当前状态并进入指定的新状态。
    ///
    /// 即使新旧状态在业务上相等，也按显式转换执行钩子。
    To(S),
}

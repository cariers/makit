use crate::{DispatchResult, MachineContext, State, Step, Transition};

/// 两种包装器共用的状态存储与单次推进逻辑。
pub(super) struct Inner<M>
where
    M: MachineContext,
{
    context: M,
    state: M::State,
}

impl<M> Inner<M>
where
    M: MachineContext,
{
    pub(super) fn new(context: M) -> Self {
        Self {
            context,
            state: M::initial_state(),
        }
    }

    pub(super) fn context(&self) -> &M {
        &self.context
    }

    pub(super) fn state(&self) -> &M::State {
        &self.state
    }
}

impl<M> Inner<M>
where
    M: MachineContext,
    M::State: State<M>,
{
    pub(super) fn init(&mut self) {
        // 初始化阶段由包装器管理，内部驱动只负责执行一次进入钩子。
        self.state.handle_entry_action(&mut self.context);
    }

    pub(super) fn dispatch(&mut self, input: &M::Input<'_>) -> DispatchResult<M::Event> {
        if !self.state.guard(&self.context, input) {
            return DispatchResult::GuardBlocked;
        }

        // 检查后立即使用同一份输入推进，不缓存 Guard 结论或暴露中间状态。
        let Step { events, transition } = self.state.advance(&mut self.context, input);

        if let Transition::To(next) = transition {
            self.state.handle_exit_action(&mut self.context);
            self.state = next;
            self.state.handle_entry_action(&mut self.context);
        }

        DispatchResult::Executed { events }
    }
}

impl<M> Clone for Inner<M>
where
    M: MachineContext + Clone,
    M::State: Clone,
{
    fn clone(&self) -> Self {
        Self {
            context: self.context.clone(),
            state: self.state.clone(),
        }
    }
}

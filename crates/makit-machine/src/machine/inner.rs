use crate::{Error, MachineContext, Outcome, State, Transition};

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
        // 初始化没有旧状态，仅执行初始进入，不触发派发或转换钩子。
        self.state.handle_entry_action(&mut self.context);
    }

    pub(super) fn dispatch(
        &mut self,
        input: &M::Input<'_>,
    ) -> Result<Box<[M::Event]>, Error<M::Error>> {
        self.context.before_dispatch(&self.state, input);

        // 仅业务错误触发通知；借用结束后原样返回错误，不执行成功收尾。
        let Outcome { events, transition } = match self.state.advance(&mut self.context, input) {
            Ok(outcome) => outcome,
            Err(error) => {
                if let Error::Custom(source) = &error {
                    self.context.dispatch_error(&self.state, input, source);
                }
                return Err(error);
            }
        };

        if let Transition::To(next) = transition {
            self.context.before_transition(&self.state, &next);
            self.state.handle_exit_action(&mut self.context);
            // 保持原有赋值析构时机，不将旧状态延长到新状态进入或转换后维护。
            self.state = next;
            self.state.handle_entry_action(&mut self.context);
            self.context.after_transition(&self.state);
        }

        self.context.after_dispatch(&self.state, input, &events);
        Ok(events)
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

//! 局上下文、阶段输入记录与事件记录组成的引擎状态机。

use std::collections::VecDeque;

use makit_context::Context;
use makit_machine::{Error, MachineContext, Outcome as MachineOutcome, State, Transition};

use crate::{Phase, PhaseOutcome, PhaseVariant};

/// 独立持有局上下文及已处理阶段输入、事件记录的引擎。
///
/// 当前阶段由 [`EngineState`] 持有；阶段处理产生的状态转换交由机器驱动应用。
/// 输入记录仅包含阶段成功处理的输入，不包含启动指令或返回错误的输入。
/// 事件按产生顺序留存副本，同时作为本次派发结果返回。
/// 整手结束的输入与事件同样记录，最终结果由 [`EngineState::Finished`] 持有。
/// 阶段错误原样返回，不追加记录；阶段实现负责遵守错误前不修改数据的契约。
///
/// 引擎的数据声明、[`new`](Self::new) 和 [`MachineContext`] 实现仅要求 [`PhaseVariant`]。
/// 通过 [`IntoMachineState`](makit_machine::IntoMachineState) 转换为尚未初始化的普通或
/// 严格机器也不要求主阶段可执行；初始化和派发时，才通过引擎状态的 [`State`] 实现
/// 要求 `V::Phase: Phase<V>`。
///
/// ```mermaid
/// flowchart TD
///     Protocol[PhaseVariant：类型与初始数据声明] --> Engine[构造 Engine]
///     Engine --> Machine[转换为尚未初始化的机器]
///     Bound[主阶段实现 Phase] --> State[引擎状态获得 State 执行能力]
///     Machine --> Execute[初始化与输入派发]
///     State --> Execute
/// ```
pub struct Engine<V>
where
    V: PhaseVariant,
{
    /// 当前一局的领域上下文。
    context: Context<V>,
    /// 已处理阶段输入的顺序记录。
    inputs: VecDeque<V::Input>,
    /// 已处理阶段事件的顺序记录，与派发结果分别持有事件值。
    events: VecDeque<V::Event>,
}

impl<V> Engine<V>
where
    V: PhaseVariant,
{
    /// 委托变体使用规则构造初始局上下文，并创建空的输入与事件记录。
    ///
    /// 本方法不创建初始阶段；初始阶段在准备状态接收 [`EngineInput::Start`] 时创建。
    /// 构造只依赖变体的数据声明，不要求 `V::Phase` 已实现 [`Phase<V>`]。
    pub fn new(rule: V::Rule) -> Self {
        let context = V::initial_context(rule);
        Self {
            context,
            inputs: VecDeque::new(),
            events: VecDeque::new(),
        }
    }
}

/// 引擎接受的生命周期指令或阶段输入。
pub enum EngineInput<P> {
    /// 请求准备中的引擎创建初始阶段。
    Start,
    /// 将输入交给当前运行的阶段处理。
    Phase(P),
}

/// 引擎的外层生命周期，以及当前阶段或整手最终结果。
///
/// 阶段之间的切换保持为运行状态；只有 [`PhaseOutcome::Finished`] 才结束整手流程。
/// 最终结果按所有权移入结束状态，不要求其能够克隆。
/// 状态值本身只保存 `P` 和 `O`；适配为可执行引擎状态时，主阶段才需要实现 [`Phase`]。
pub enum EngineState<P, O> {
    /// 尚未创建初始阶段，等待启动指令。
    Preparing,
    /// 持有并运行当前阶段。
    Running(P),
    /// 整手已经结束，持有最终结果，不再处理输入。
    Finished(O),
}

impl<P, O> EngineState<P, O> {
    /// 借用整手最终结果；尚未结束时返回 `None`。
    ///
    /// 派发完成后可通过机器当前状态读取，结果由结束状态持有，不复制到引擎上下文。
    #[must_use]
    pub const fn output(&self) -> Option<&O> {
        match self {
            Self::Finished(output) => Some(output),
            Self::Preparing | Self::Running(_) => None,
        }
    }
}

impl<V> MachineContext for Engine<V>
where
    V: PhaseVariant,
{
    type State = EngineState<V::Phase, V::Output>;
    type Input<'ipt> = EngineInput<V::Input>;
    type Event = V::Event;
    type Error = V::Error;

    fn initial_state() -> Self::State {
        EngineState::Preparing
    }
}

// 执行约束放在状态适配处，使构造引擎和未初始化机器只依赖数据声明。
impl<V> State<Engine<V>> for EngineState<V::Phase, V::Output>
where
    V: PhaseVariant,
    V::Phase: Phase<V>,
{
    fn advance(
        &mut self,
        ctx: &mut Engine<V>,
        input: &EngineInput<V::Input>,
    ) -> Result<MachineOutcome<Self, V::Event>, Error<V::Error>> {
        match (self, input) {
            (EngineState::Preparing, EngineInput::Start) => {
                let phase = V::initial_phase(&ctx.context);
                Ok(MachineOutcome::to(EngineState::Running(phase), []))
            }
            (EngineState::Running(phase), EngineInput::Phase(p)) => {
                // 错误在记录与转换之前传播，避免失败输入被记为成功处理。
                let (events, transition) = match phase.handle(&mut ctx.context, p)? {
                    PhaseOutcome::Continue(MachineOutcome { events, transition }) => {
                        let transition = match transition {
                            Transition::Stay => Transition::Stay,
                            Transition::To(next) => Transition::To(EngineState::Running(next)),
                        };
                        (events, transition)
                    }
                    PhaseOutcome::Finished { events, output } => {
                        (events, Transition::To(EngineState::Finished(output)))
                    }
                };

                // 普通推进与整手结束都记录一次输入，避免遗漏最后一次处理。
                ctx.inputs.push_back(p.clone());
                // 留存副本而不消费本批原始事件，使记录与派发结果各自拥有事件。
                ctx.events.extend(events.iter().cloned());

                Ok(MachineOutcome { events, transition })
            }
            _ => Err(Error::Unhandled),
        }
    }
}

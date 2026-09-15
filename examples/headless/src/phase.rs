//! 将公共准备行动组装为演示变体的主阶段。

use makit::engine::action::setup::{DetermineDealer, Shuffle};
use makit::{Action, Context, Error, Phase, PhaseOutcome, PhaseResult, Progress};

use crate::{DemoError, DemoEvent, DemoInput, DemoOutput, DemoVariant};

/// 演示运行中的阶段，持有该阶段唯一需要启动的行动。
///
/// 不匹配当前阶段的输入返回 [`Error::Unhandled`]，避免重复启动已完成行动。
/// 具体行动的业务错误映射到 [`DemoError`]，不吞掉错误或伪造完成事件。
/// 准备结果交给引擎终态保存，本枚举不重复持有完成状态。
#[derive(Debug)]
pub enum DemoPhase {
    /// 等待洗牌输入，持有尚未启动的洗牌行动。
    Shuffling(Shuffle),
    /// 等待定庄输入，持有尚未启动的定庄行动。
    DeterminingDealer(DetermineDealer),
}

impl Phase<DemoVariant> for DemoPhase {
    fn handle(
        &mut self,
        ctx: &mut Context<DemoVariant>,
        input: &DemoInput,
    ) -> PhaseResult<DemoVariant> {
        match (self, input) {
            (Self::Shuffling(action), DemoInput::RunShuffle) => {
                // 具体洗牌行动只会在启动时完成；契约变化时必须调整编排，不能提前切换。
                let progress = action.start(ctx).unwrap_or_else(|never| match never {});
                let Progress::Complete(events) = progress else {
                    panic!("shuffle must complete during start");
                };
                let events: Box<[DemoEvent]> =
                    events.into_iter().map(DemoEvent::Shuffled).collect();
                Ok(PhaseOutcome::to(
                    Self::DeterminingDealer(DetermineDealer::new(ctx.rule().align_east())),
                    events,
                ))
            }
            (Self::DeterminingDealer(action), DemoInput::DetermineDealer) => {
                // 固定上下文已保证东风位置加入；定庄行动一次启动便完成并产生唯一结果事件。
                // 行动只返回业务错误；先归入变体错误，再由 ? 包装为 Error::Custom。
                let progress = action.start(ctx).map_err(DemoError::DetermineDealer)?;
                let Progress::Complete(events) = progress else {
                    panic!("dealer determination must complete during start");
                };
                let [determined] = events.as_ref() else {
                    panic!("dealer determination must emit exactly one result");
                };
                let output = DemoOutput {
                    dice: determined.dice,
                    dealer: determined.dealer,
                    east: determined.east,
                    wall: ctx.wall().as_slice().into(),
                };
                let events: Box<[DemoEvent]> = events
                    .into_iter()
                    .map(DemoEvent::DealerDetermined)
                    .collect();
                Ok(PhaseOutcome::finished(output, events))
            }
            // 在任何行动调用之前完成路由判断，未处理输入不会留下领域修改。
            _ => Err(Error::Unhandled),
        }
    }
}

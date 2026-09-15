//! 使用相同种子与稀疏座位，展示定庄时保留或调整东风锚点的结果。
//!
//! 运行方式：`cargo run -p makit-engine --example determine_dealer`。

use makit_context::{Context, JoinError, Variant};
use makit_core::{Seat, Seed};
use makit_engine::{Action, Progress, action::setup::DetermineDealer};

/// 本示例不需要额外的规则或变体状态。
struct DemoVariant;

impl Variant for DemoVariant {
    type Rule = ();
    type Context = ();
    type SeatContext = ();

    fn initial_context(rule: Self::Rule) -> Context<Self> {
        // 固定字节仅用于可重复示例，不读取随机源。
        let seed = Seed::from_bytes([4; Seed::BYTE_LEN]);
        Context::new(rule, seed, ())
    }
}

fn main() -> Result<(), JoinError<()>> {
    // 两种配置各自使用相同固定种子创建全新的上下文。
    for align_east in [false, true] {
        let mut ctx = DemoVariant::initial_context(());
        // 编号为 2 的位置缺席；默认东风位置为已加入的 1 号位置。
        for seat in [Seat::ALL[0], Seat::ALL[2], Seat::ALL[3]] {
            ctx.join(seat, ())?;
        }

        let participants: Vec<_> = ctx.participants().iter().map(Seat::code).collect();
        println!("Align east: {align_east}");
        println!("Participants: {participants:?}");
        println!("Initial east: {}", ctx.east().seat().code());

        let mut action = DetermineDealer::new(align_east);
        let (status, events) = match Action::start(&mut action, &mut ctx) {
            Progress::Running(events) => ("Running", events),
            Progress::Complete(events) => ("Complete", events),
        };
        println!("Progress: {status}");
        println!("Events: {}", events.len());
        for event in events {
            println!("Dice: {:?}", event.dice);
            println!("Event dealer: {}", event.dealer.code());
            println!("Event east: {}", event.east.seat().code());
        }

        println!("Context dealer: {:?}", ctx.dealer().map(Seat::code));
        println!("Context east: {}", ctx.east().seat().code());
        println!();
    }
    Ok(())
}

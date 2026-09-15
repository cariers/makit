//! 使用固定种子直接启动洗牌行动，展示处理结果与牌墙变化。
//!
//! 运行方式：`cargo run -p makit-engine --example shuffle`。

use makit_context::{Context, Variant};
use makit_core::{Seed, Tile, Wall};
use makit_engine::{Action, Progress, action::setup::Shuffle};

/// 本示例不需要额外的规则或变体状态。
struct DemoVariant;

impl Variant for DemoVariant {
    type Rule = ();
    type Context = ();
    type SeatContext = ();

    fn initial_context(rule: Self::Rule) -> Context<Self> {
        // 固定字节仅用于可重复示例，不读取随机源。
        let seed = Seed::from_bytes([7; Seed::BYTE_LEN]);
        Context::new(rule, seed, ())
    }
}

fn main() {
    let mut ctx = DemoVariant::initial_context(());
    ctx.replace_wall(Wall::from_tiles([
        Tile::MAN_1,
        Tile::MAN_1,
        Tile::MAN_2,
        Tile::MAN_2,
        Tile::PIN_1,
        Tile::PIN_1,
        Tile::EAST,
        Tile::EAST,
    ]));

    let before: Vec<_> = ctx.wall().iter().map(Tile::code).collect();
    println!("Before: {before:?}");

    let mut shuffle = Shuffle;
    let (status, events) = match Action::start(&mut shuffle, &mut ctx) {
        Progress::Running(events) => ("Running", events),
        Progress::Complete(events) => ("Complete", events),
    };
    println!("Progress: {status}");
    println!("Events: {}", events.len());
    for event in events {
        println!("Shuffled tiles: {}", event.tile_count);
    }

    let after: Vec<_> = ctx.wall().iter().map(Tile::code).collect();
    println!("After: {after:?}");
}

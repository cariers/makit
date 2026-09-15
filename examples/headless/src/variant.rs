//! 演示规则及固定领域数据的装配。

use makit::engine::action::setup::Shuffle;
use makit::{Context, PhaseVariant, Seat, Seed, Tile, Variant, Wall};

use crate::{DemoEvent, DemoInput, DemoOutput, DemoPhase};

/// 演示使用的显式种子和东风切换配置。
///
/// 牌集与参与座位由 [`DemoVariant`] 固定，不引入通用玩法的默认配置。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DemoRule {
    seed: Seed,
    align_east: bool,
}

impl DemoRule {
    /// 创建演示规则；`align_east` 为 `true` 时将东风切换到本次选出的庄家。
    pub const fn new(seed: Seed, align_east: bool) -> Self {
        Self { seed, align_east }
    }

    /// 借用用于本次演示的完整种子。
    pub const fn seed(&self) -> &Seed {
        &self.seed
    }

    /// 返回是否在定庄后将东风切换到庄家。
    pub const fn align_east(&self) -> bool {
        self.align_east
    }
}

/// 只覆盖麻将准备操作的演示变体，不提供行牌或结算规则。
///
/// 初始牌墙包含一万、二万、一筒和东风各两张，参与座位固定为 `1`、`3`、`4`。
/// 整手与逐座扩展均为 `()`；座位在上下文创建时加入，不模拟加入输入的交互。
#[derive(Debug)]
pub struct DemoVariant;

impl Variant for DemoVariant {
    type Rule = DemoRule;
    type Context = ();
    type SeatContext = ();

    fn initial_context(rule: Self::Rule) -> Context<Self> {
        let seed = *rule.seed();
        let mut ctx = Context::<Self>::new(rule, seed, ());
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

        // 固定集合没有重复位置，且包含默认东风位置，满足定庄启动的前置条件。
        for seat in [Seat::ALL[0], Seat::ALL[2], Seat::ALL[3]] {
            ctx.join(seat, ())
                .expect("fixed demo seats must be distinct");
        }
        ctx
    }
}

impl PhaseVariant for DemoVariant {
    type Phase = DemoPhase;
    type Input = DemoInput;
    type Event = DemoEvent;
    type Output = DemoOutput;

    fn initial_phase(_: &Context<Self>) -> Self::Phase {
        DemoPhase::Shuffling(Shuffle)
    }
}

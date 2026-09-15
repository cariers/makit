//! 从当前东风起逆时针数已加入座位的定庄行动。

use std::convert::Infallible;

use makit_context::{Context, Variant};
use makit_core::{EastSeat, Seat};
use rand::{RngExt, SeedableRng};
use rand_chacha::ChaCha20Rng;

use crate::{Action, ActionOutcome, Progress};

/// 使用两枚骰子确定庄家，并按配置决定是否将东风切换到庄家的行动。
///
/// 从当前东风起逆时针数已加入座位，起点算作 `1`，跳过未加入的位置。
/// 调用方必须在启动前保证当前东风位置已加入；本行动不检查此前置条件，
/// 启动结果也不提供拒绝分支。不要求固定参与人数，也不要求原庄家为空。
///
/// 每次启动使用局种子初始化 `ChaCha20` 的流 `1`，与洗牌使用的默认流 `0` 分开。
/// 相同种子、输入上下文、依赖版本和特性配置下结果可复现，不承诺跨依赖版本保持相同结果。
/// 本行动没有后续业务输入，也不记录是否已启动；阶段或调用方负责限制重复启动。
///
/// # Panics
///
/// 启动时如果没有任何已加入座位，会在计算步数时因对零取余而 panic。
#[derive(Clone, Copy, Debug)]
pub struct DetermineDealer {
    /// 是否将最终东风位置切换到新庄家。
    align_east: bool,
}

impl DetermineDealer {
    /// 创建定庄行动，显式选择是否将东风切换到新庄家。
    ///
    /// `align_east` 为 `false` 时保留原东风位置；两种配置均更新庄家。
    pub const fn new(align_east: bool) -> Self {
        Self { align_east }
    }
}

/// 定庄完成事件，包含两枚骰子点数、庄家位置和执行后的东风。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DealerDetermined {
    /// 本次生成的两枚骰子点数，每枚范围为 `1..=6`。
    pub dice: [u8; 2],
    /// 按两枚骰子的总点数选出的已加入座位。
    pub dealer: Seat,
    /// 执行后的东风；关闭东风切换时保持原值。
    pub east: EastSeat,
}

impl<V> Action<V> for DetermineDealer
where
    V: Variant,
{
    type Input<'ipt> = Infallible;

    type Event = DealerDetermined;

    /// 在调用方已保证当前东风位置加入的条件下掷骰并提交定庄结果。
    ///
    /// # Panics
    ///
    /// 已加入座位集合为空时，计算步数会因对零取余而 panic。
    fn start(&mut self, ctx: &mut Context<V>) -> Progress<Self::Event> {
        let participants = ctx.participants();
        let original_east = ctx.east();
        let mut dealer = original_east.seat();

        let mut rng = ChaCha20Rng::from_seed(*ctx.seed().as_bytes());
        // 洗牌使用默认流 0，定庄固定使用流 1，两个行动不消费彼此的随机序列。
        rng.set_stream(1);
        let dice = [rng.random_range(1_u8..=6), rng.random_range(1_u8..=6)];

        // 已加入的起点算 1，因此只需移动总点数减 1 个参与座位。
        // 依赖调用方保证起点已加入，使人数非零且跳过空位的循环每圈都能遇到参与座位。
        let mut steps = (usize::from(dice[0]) + usize::from(dice[1]) - 1) % participants.len();
        while steps > 0 {
            dealer = dealer.counter_clockwise_next();
            if participants.contains(dealer) {
                steps -= 1;
            }
        }

        let east = if self.align_east {
            EastSeat::new(dealer)
        } else {
            original_east
        };
        // 完成计算和事件分配后再提交庄家与东风，准备阶段不修改局上下文。
        let events: Box<[DealerDetermined]> = Box::new([DealerDetermined { dice, dealer, east }]);
        ctx.set_dealer(Some(dealer));
        if self.align_east {
            ctx.set_east(east);
        }

        Progress::Complete(events)
    }

    fn handle(
        &mut self,
        _: &mut Context<V>,
        input: &Self::Input<'_>,
    ) -> ActionOutcome<Self::Event> {
        match *input {}
    }
}

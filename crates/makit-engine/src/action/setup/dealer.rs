//! 从当前东风起逆时针数已加入座位的定庄行动。

use std::convert::Infallible;

use makit_context::{Context, Variant};
use makit_core::{EastSeat, Seat};
use rand::{RngExt, SeedableRng};
use rand_chacha::ChaCha20Rng;

use crate::{Action, Progress};

/// 使用两枚骰子确定庄家，并按配置决定是否将东风切换到庄家的行动。
///
/// 从当前东风起逆时针数已加入座位，起点算作 `1`，跳过未加入的位置。
/// 启动时检查当前东风位置已经加入；缺失时返回
/// [`DetermineDealerError::EastNotJoined`]，不生成骰子或修改局上下文。
/// 不要求固定参与人数，也不要求原庄家为空。
///
/// 每次启动使用局种子初始化 `ChaCha20` 的流 `1`，与洗牌使用的默认流 `0` 分开。
/// 相同种子、输入上下文、依赖版本和特性配置下结果可复现，不承诺跨依赖版本保持相同结果。
/// 本行动没有后续业务输入，也不记录是否已启动；阶段或调用方负责限制重复启动。
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

/// 定庄无法启动的结构化业务错误。
///
/// 由 [`Action::start`] 直接返回；错误发生时不产生事件，不改变行动或局上下文。
#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
pub enum DetermineDealerError {
    /// 当前东风锚点未加入，不能以该位置作为参与座位计数的起点。
    #[error("east seat {} has not joined; dealer cannot be determined", .east.code())]
    EastNotJoined {
        /// 尚未加入的当前东风逻辑位置。
        east: Seat,
    },
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

    type Error = DetermineDealerError;

    /// 检查东风位置已经加入后掷骰，提交庄家及最终东风。
    ///
    /// # Errors
    ///
    /// 当前东风位置尚未加入（包括参与集合为空）时直接返回
    /// [`DetermineDealerError::EastNotJoined`]；
    /// 此时未生成随机数，也不修改行动、庄家、东风或其他局数据。
    fn start(&mut self, ctx: &mut Context<V>) -> Result<Progress<Self::Event>, Self::Error> {
        let participants = ctx.participants();
        let original_east = ctx.east();
        // 校验起点同时保证参与人数非零，避免空集合取余或把缺席位置选为庄家。
        if !participants.contains(original_east.seat()) {
            return Err(DetermineDealerError::EastNotJoined {
                east: original_east.seat(),
            });
        }
        let mut dealer = original_east.seat();

        let mut rng = ChaCha20Rng::from_seed(*ctx.seed().as_bytes());
        // 洗牌使用默认流 0，定庄固定使用流 1，两个行动不消费彼此的随机序列。
        rng.set_stream(1);
        let dice = [rng.random_range(1_u8..=6), rng.random_range(1_u8..=6)];

        // 已加入的起点算 1，因此只需移动总点数减 1 个参与座位。
        // 起点已确认加入，跳过空位的循环每圈都能遇到参与座位。
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

        Ok(Progress::Complete(events))
    }

    fn handle(
        &mut self,
        _: &mut Context<V>,
        input: &Self::Input<'_>,
    ) -> Result<Progress<Self::Event>, Self::Error> {
        match *input {}
    }
}

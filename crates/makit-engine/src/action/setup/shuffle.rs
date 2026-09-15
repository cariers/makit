//! 使用局种子打乱当前剩余牌墙的单次行动。

use std::convert::Infallible;

use makit_context::{Context, Variant};
use makit_core::Wall;
use rand::{SeedableRng, seq::SliceRandom};
use rand_chacha::ChaCha20Rng;

use crate::{Action, ActionOutcome, Progress};

/// 使用局种子打乱当前剩余牌墙的无配置行动。
///
/// 调用方先按具体玩法装配牌墙；本行动只重排当前剩余牌，保留重复牌面的张数，
/// 不恢复已取走的牌。空牌墙和单张牌也正常完成，并产生一个 [`Shuffled`] 事件。
///
/// 每次启动都从局种子初始化独立的 `ChaCha20` 随机发生器；相同种子、相同初始剩余牌序
/// 以及相同依赖版本和特性可复现结果，不承诺跨依赖版本保持相同牌序。
/// 本行动没有后续业务输入，也不记录是否已启动；阶段或调用方负责限制重复启动。
#[derive(Clone, Copy, Debug, Default)]
pub struct Shuffle;

/// 一次洗牌完成后产生的事件，只包含本次参与洗牌的精确张数。
///
/// 不携带种子或完整牌序，事件的可见范围及客户端投影由上层决定。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Shuffled {
    /// 本次完成洗牌的剩余牌张数，可能为零。
    pub tile_count: usize,
}

impl<V> Action<V> for Shuffle
where
    V: Variant,
{
    type Input<'ipt> = Infallible;

    type Event = Shuffled;

    fn start(&mut self, ctx: &mut Context<V>) -> Progress<Self::Event> {
        // 只复制剩余区间，避免替换牌墙时将已经取出的牌重新放回。
        let mut tiles = ctx.wall().as_slice().to_vec();
        let mut rng = ChaCha20Rng::from_seed(*ctx.seed().as_bytes());
        tiles.shuffle(&mut rng);

        // 先准备完整候选牌墙和事件，再一次提交，准备阶段不修改局上下文。
        let wall = Wall::from_tiles(tiles);
        let events: Box<[Shuffled]> = Box::new([Shuffled {
            tile_count: wall.len(),
        }]);
        ctx.replace_wall(wall);

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

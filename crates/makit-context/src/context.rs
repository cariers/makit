//! 一手牌的通用数据聚合、独立准备操作与跨容器操作。

mod error;
mod meld;
mod seat;
mod transfer;

use std::{fmt, iter::FusedIterator, mem, ops};

use makit_core::{
    EastSeat, Hand, Melds, River, Seat, SeatMap, SeatMask, Seed, Tile, TileCounts, TileMask, Wall,
    Wind,
};

use crate::Variant;

pub use error::{ContextError, JoinError};
pub use seat::SeatContext;
pub use transfer::{DealOutcome, DealRequest, ExchangeSelection, ExchangeTransfer, WallEnd};

/// 从创建、座位加入和准备持续到一手牌结束的权威上下文。
///
/// 各数据从自身有效的空值或默认值独立形成，不设置统一就绪屏障。
/// 通用组合操作维护核心容器的一致性；规则合法性、行动顺序、交互和结束判断由上层负责。
///
/// 核心容器仅提供只读访问，扩展状态可单独修改。扩展修改不自动参与通用操作的失败回滚。
/// 本类型包含全部手牌和随机种子等信息，不能直接作为客户端公开视图。
pub struct Context<V: Variant> {
    /// 当前一手显式传入的固定规则配置。
    rule: V::Rule,
    /// 当前一手固定的初始随机输入，不代表随机发生器的当前状态。
    seed: Seed,
    /// 逐步加入的座位以及随其建立的通用和变体状态。
    seats: SeatMap<SeatContext<V>>,
    /// 独立记录的庄家指定，不隐含目标座位已经加入。
    dealer: Option<Seat>,
    /// 始终存在的坐风锚点，与庄家指定相互独立。
    east: EastSeat,
    /// 当前剩余牌墙，空墙不表示某个特定流程阶段。
    wall: Wall,
    /// 当前一手的弃牌历史和留存状态。
    river: River,
    /// 当前一手共享的变体状态。
    variant: V::Context,
}

impl<V: Variant> Context<V> {
    /// 从空座位、空牌墙和空牌河创建上下文。
    ///
    /// 庄家初始未指定，坐风采用 [`EastSeat::default`]；规则、种子和整手扩展必须显式提供。
    /// 构造不会生成随机结果，也不会调用变体初始化逻辑。
    pub fn new(rule: V::Rule, seed: Seed, variant: V::Context) -> Self {
        Self {
            rule,
            seed,
            seats: SeatMap::new(),
            dealer: None,
            east: EastSeat::default(),
            wall: Wall::new(),
            river: River::new(),
            variant,
        }
    }

    /// 借用当前一手固定的规则配置。
    pub const fn rule(&self) -> &V::Rule {
        &self.rule
    }

    /// 借用当前一手固定的初始随机种子。
    ///
    /// 本方法供权威逻辑显式读取；随机算法、连续消费状态及向客户端的可见性由上层管理。
    pub const fn seed(&self) -> &Seed {
        &self.seed
    }

    /// 借用所有已加入座位的固定槽映射，不提供对成员或核心牌区的直接可变访问。
    pub const fn seats(&self) -> &SeatMap<SeatContext<V>> {
        &self.seats
    }

    /// 借用指定座位的状态；该位置尚未加入时返回 `None`。
    pub const fn seat(&self, seat: Seat) -> Option<&SeatContext<V>> {
        self.seats.get(seat)
    }

    /// 返回已加入的逻辑位置集合，不表示合法人数或行动顺序。
    pub fn participants(&self) -> SeatMask {
        self.seats.mask()
    }

    /// 返回已加入的座位数量，不重复保存人数状态。
    pub const fn seat_count(&self) -> usize {
        self.seats.len()
    }

    /// 判断指定逻辑位置是否已经加入。
    pub const fn is_joined(&self, seat: Seat) -> bool {
        self.seats.contains_key(seat)
    }

    /// 返回当前庄家指定，未指定时为 `None`。
    ///
    /// 返回的位置不一定已加入；依赖参与状态的操作仍需分别检查。
    pub const fn dealer(&self) -> Option<Seat> {
        self.dealer
    }

    /// 返回始终存在的坐风锚点，不检查对应位置是否已加入。
    pub const fn east(&self) -> EastSeat {
        self.east
    }

    /// 按当前锚点计算指定逻辑位置的坐风，即使该位置尚未加入也可计算。
    pub const fn wind_at(&self, seat: Seat) -> Wind {
        self.east.wind_at(seat)
    }

    /// 按当前锚点计算指定坐风对应的逻辑位置，不保证该位置已加入。
    pub const fn seat_at(&self, wind: Wind) -> Seat {
        self.east.seat_at(wind)
    }

    /// 借用当前牌墙；空墙不区分尚未装牌和已经取完。
    pub const fn wall(&self) -> &Wall {
        &self.wall
    }

    /// 借用当前一手的弃牌历史及留存状态。
    pub const fn river(&self) -> &River {
        &self.river
    }

    /// 借用当前一手共享的变体状态。
    pub const fn variant(&self) -> &V::Context {
        &self.variant
    }

    /// 可变借用当前一手共享的变体状态。
    ///
    /// 经此引用完成的修改不会因后续通用操作失败而自动回滚。
    pub fn variant_mut(&mut self) -> &mut V::Context {
        &mut self.variant
    }

    /// 借用指定座位的变体扩展；该位置尚未加入时返回 `None`。
    pub fn seat_variant(&self, seat: Seat) -> Option<&V::SeatContext> {
        self.seats.get(seat).map(SeatContext::variant)
    }

    /// 可变借用指定座位的变体扩展；该位置尚未加入时返回 `None`。
    ///
    /// 不暴露该座位的可变核心牌区，扩展修改也不会因后续通用操作失败而自动回滚。
    pub fn seat_variant_mut(&mut self, seat: Seat) -> Option<&mut V::SeatContext> {
        self.seats.get_mut(seat).map(|state| &mut state.variant)
    }

    /// 借用指定座位的手牌；该位置尚未加入时返回 `None`。
    pub fn hand(&self, seat: Seat) -> Option<&Hand> {
        self.seats.get(seat).map(SeatContext::hand)
    }

    /// 借用指定座位按提交顺序存储的固定组；该位置尚未加入时返回 `None`。
    pub fn melds(&self, seat: Seat) -> Option<&Melds> {
        self.seats.get(seat).map(SeatContext::melds)
    }

    /// 为尚未加入的位置建立空手牌、空固定组和显式传入的座位扩展。
    ///
    /// 本方法仅检查成员不重复；合法人数、身份认证和允许加入的时机由上层决定。
    /// 不要求庄家已指定或牌墙已有牌，不推进任何流程。
    ///
    /// # Errors
    ///
    /// 该位置已有状态时返回 [`JoinError::SeatOccupied`]，保留原状态并归还传入的扩展。
    pub fn join(
        &mut self,
        seat: Seat,
        variant: V::SeatContext,
    ) -> Result<(), JoinError<V::SeatContext>> {
        if self.seats.contains_key(seat) {
            return Err(JoinError::SeatOccupied { seat, variant });
        }
        let _ = self.seats.insert(seat, SeatContext::new(variant));
        Ok(())
    }

    /// 独立更新庄家指定，`None` 表示取消指定。
    ///
    /// 不要求目标已加入，不改变坐风锚点或推进流程；后续操作自行检查所需的参与状态。
    pub fn set_dealer(&mut self, dealer: Option<Seat>) {
        self.dealer = dealer;
    }

    /// 更新坐风锚点，不改变庄家、座位成员或任何牌区。
    pub fn set_east(&mut self, east: EastSeat) {
        self.east = east;
    }

    /// 装配已经准备好的牌墙，并将原牌墙的所有权返回给调用者。
    ///
    /// 不改变手牌或弃牌历史；是否允许替换由上层决定。此操作引入外部牌源，
    /// 不属于通用区域内部的守恒转移，也不会自动更新随机种子。
    pub fn replace_wall(&mut self, wall: Wall) -> Wall {
        mem::replace(&mut self.wall, wall)
    }

    /// 逐张迭代通用区域当前持有的原始牌面，保留重复牌面。
    ///
    /// 范围仅包括剩余牌墙、各已加入座位的手牌与固定组、牌河当前留存牌。
    /// 不包含已取走弃牌的历史副本，也不读取任何变体扩展中的专属牌区。
    /// 迭代顺序是牌墙、按座位编码排列的各座位牌区、按弃牌编号排列的当前留存牌，
    /// 不代表事件或行动顺序。
    pub fn common_tiles(&self) -> impl DoubleEndedIterator<Item = Tile> + FusedIterator + '_ {
        self.wall
            .iter()
            .chain(self.seats.values().flat_map(SeatContext::tiles))
            .chain(self.river.present().map(|(_, discard)| discard.tile()))
    }

    /// 返回 [`Self::common_tiles`] 范围内的精确总张数，不使用饱和计数摘要。
    pub fn common_tile_count(&self) -> usize {
        self.common_tiles().count()
    }

    /// 返回 [`Self::common_tiles`] 范围内指定原始牌面的精确张数，不执行通配解释。
    pub fn common_count(&self, tile: Tile) -> usize {
        self.common_tiles()
            .filter(|&candidate| candidate == tile)
            .count()
    }

    /// 返回 [`Self::common_tiles`] 范围内的牌面集合，不保留张数。
    pub fn common_mask(&self) -> TileMask {
        self.common_tiles().collect()
    }

    /// 返回 [`Self::common_tiles`] 范围内每牌面最多 255 张的饱和计数摘要。
    ///
    /// 精确校验使用 [`Self::common_count`] 或 [`Self::common_tile_count`]，不能以摘要总数替代。
    pub fn common_counts(&self) -> TileCounts {
        self.common_tiles().collect()
    }
}

/// 将自动解引用限定到整手变体扩展，不暴露核心容器的可变访问。
///
/// 扩展与上下文存在同名方法时优先使用上下文的固有方法；通过 [`Context::variant`]
/// 可以明确访问扩展自身的接口。
impl<V: Variant> ops::Deref for Context<V> {
    type Target = V::Context;

    fn deref(&self) -> &Self::Target {
        &self.variant
    }
}

/// 可变自动解引用仅作用于整手变体扩展，不构成跨通用状态与扩展的事务。
///
/// 与上下文固有方法同名的扩展方法可以通过 [`Context::variant_mut`] 明确调用。
impl<V: Variant> ops::DerefMut for Context<V> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.variant
    }
}

/// 仅在实际存储的三个扩展作用域均可复制时复制上下文，不要求 `V: Clone`。
///
/// 核心容器独立复制；扩展若包含共享内部可变状态，其 `Clone` 不会自动获得隔离语义。
impl<V: Variant> Clone for Context<V>
where
    V::Rule: Clone,
    V::Context: Clone,
    V::SeatContext: Clone,
{
    fn clone(&self) -> Self {
        Self {
            rule: self.rule.clone(),
            seed: self.seed,
            seats: self.seats.clone(),
            dealer: self.dealer,
            east: self.east,
            wall: self.wall.clone(),
            river: self.river.clone(),
            variant: self.variant.clone(),
        }
    }
}

/// 仅要求实际存储的扩展数据可调试，不要求 `V: Debug`。
///
/// 种子沿用 [`Seed`] 的隐藏输出；手牌及扩展仍会展开，因此此输出不适合直接公开。
impl<V: Variant> fmt::Debug for Context<V>
where
    V::Rule: fmt::Debug,
    V::Context: fmt::Debug,
    V::SeatContext: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Context")
            .field("rule", &self.rule)
            .field("seed", &self.seed)
            .field("seats", &self.seats)
            .field("dealer", &self.dealer)
            .field("east", &self.east)
            .field("wall", &self.wall)
            .field("river", &self.river)
            .field("variant", &self.variant)
            .finish()
    }
}

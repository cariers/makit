//! 单个已加入座位的通用牌区与变体扩展。

use std::{fmt, iter::FusedIterator, ops};

use makit_core::{Hand, Melds, Tile, TileCounts, TileMask};

use crate::Variant;

/// 一个已加入座位的手牌、固定组及专属变体状态。
///
/// 由 [`super::Context::join`] 同时创建三个部分，避免通用状态和变体状态拥有不同的成员集合。
/// 手牌和固定组仅提供只读借用，修改通过上下文的组合操作完成。
pub struct SeatContext<V: Variant> {
    /// 包含旧牌区和摸牌区的当前手牌。
    pub(super) hand: Hand,
    /// 按提交顺序保存的固定组，加杠在原碰牌位置替换。
    pub(super) melds: Melds,
    /// 随该座位建立的专属扩展，可能持有通用统计范围外的牌。
    pub(super) variant: V::SeatContext,
}

impl<V: Variant> SeatContext<V> {
    /// 仅由上下文加入操作建立空通用牌区，不隐式构造变体扩展。
    pub(super) fn new(variant: V::SeatContext) -> Self {
        Self {
            hand: Hand::new(),
            melds: Melds::new(),
            variant,
        }
    }

    /// 借用手牌及其旧牌、摸牌分区。
    pub const fn hand(&self) -> &Hand {
        &self.hand
    }

    /// 借用按提交顺序存储的固定组，不保证组内牌面已经通过玩法合法性校验。
    pub const fn melds(&self) -> &Melds {
        &self.melds
    }

    /// 借用该座位的变体扩展。
    pub const fn variant(&self) -> &V::SeatContext {
        &self.variant
    }

    /// 可变借用该座位的变体扩展，不修改核心手牌和固定组。
    ///
    /// 此修改不自动参与后续通用操作的失败回滚。上下文不提供整个座位的可变借用，
    /// 通过 [`super::Context::seat_variant_mut`] 可直接取得已加入座位的扩展。
    pub fn variant_mut(&mut self) -> &mut V::SeatContext {
        &mut self.variant
    }

    /// 先按手牌排列、再按固定组及组内排列逐张迭代原始牌面。
    ///
    /// 范围仅包含手牌和固定组，不读取变体扩展的专属牌区；顺序不代表行动顺序。
    pub fn tiles(&self) -> impl DoubleEndedIterator<Item = Tile> + FusedIterator + '_ {
        self.hand.iter().chain(self.melds.tiles())
    }

    /// 返回手牌与固定组的精确总张数，不包含变体专属牌区。
    pub fn tile_count(&self) -> usize {
        self.tiles().count()
    }

    /// 返回手牌与固定组中指定原始牌面的精确张数，不执行通配解释。
    pub fn count(&self, tile: Tile) -> usize {
        self.tiles().filter(|&candidate| candidate == tile).count()
    }

    /// 返回手牌与固定组的牌面集合，不包含变体专属牌区，也不保留张数。
    pub fn mask(&self) -> TileMask {
        self.tiles().collect()
    }

    /// 返回手牌与固定组中每牌面最多 255 张的饱和计数摘要。
    ///
    /// 不包含变体专属牌区；精确数量使用 [`Self::count`] 或 [`Self::tile_count`]。
    pub fn counts(&self) -> TileCounts {
        self.tiles().collect()
    }
}

/// 将自动解引用限定到座位变体扩展，不暴露核心牌区的可变访问。
///
/// 同名方法优先使用座位上下文的固有方法；[`SeatContext::variant`] 提供明确的扩展入口。
impl<V: Variant> ops::Deref for SeatContext<V> {
    type Target = V::SeatContext;

    fn deref(&self) -> &Self::Target {
        &self.variant
    }
}

/// 可变自动解引用仅作用于座位扩展，不自动回滚扩展修改。
///
/// 同名扩展方法可以通过 [`SeatContext::variant_mut`] 明确调用。
impl<V: Variant> ops::DerefMut for SeatContext<V> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.variant
    }
}

/// 仅要求座位扩展可复制，不要求 `V` 或其他作用域的关联类型实现 `Clone`。
impl<V: Variant> Clone for SeatContext<V>
where
    V::SeatContext: Clone,
{
    fn clone(&self) -> Self {
        Self {
            hand: self.hand.clone(),
            melds: self.melds.clone(),
            variant: self.variant.clone(),
        }
    }
}

/// 仅要求座位扩展可调试，不要求 `V` 或其他作用域的关联类型实现 `Debug`。
impl<V: Variant> fmt::Debug for SeatContext<V>
where
    V::SeatContext: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SeatContext")
            .field("hand", &self.hand)
            .field("melds", &self.melds)
            .field("variant", &self.variant)
            .finish()
    }
}

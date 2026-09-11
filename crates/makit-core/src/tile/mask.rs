//! 使用 64 位集合表达牌面成员关系及集合运算。

use std::{iter::FusedIterator, ops};

use crate::Tile;

/// 表示最多 64 种牌面集合。
///
/// bit `n` 对应 [`Tile::index`] 为 `n` 的牌面。位集合只保存成员关系，不保存牌面数量。
/// 集合允许全部 64 个编码槽；当前玩法启用哪些牌面，应由规则另外限定。
/// [`Default::default`] 返回空集合。
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct TileMask(u64);

impl TileMask {
    /// 不包含任何牌面的集合。
    pub const EMPTY: Self = Self(0);
    /// 包含全部 64 个编码槽的集合。
    pub const ALL: Self = Self(u64::MAX);

    /// 包含本 crate 固定定义的牌编码 `1..=42` 标准牌面。
    pub const STANDARD: Self = Self((1_u64 << Tile::STANDARD_MAX_CODE) - 1);
    /// 包含由规则解释的牌编码 `43..=64` 扩展牌面槽。
    pub const EXTENSIONS: Self = Self(!Self::STANDARD.0);

    /// 包含一万至九万。
    pub const MAN: Self = Self((1_u64 << 9) - 1);
    /// 包含一筒至九筒。
    pub const PIN: Self = Self(Self::MAN.0 << 9);
    /// 包含一条至九条。
    pub const SOU: Self = Self(Self::MAN.0 << 18);
    /// 包含万、筒、条三门序数牌。
    pub const NUMBERED: Self = Self(Self::MAN.0 | Self::PIN.0 | Self::SOU.0);

    /// 包含东、南、西、北四种风牌。
    pub const WINDS: Self = Self(((1_u64 << 4) - 1) << 27);
    /// 包含中、发、白三种箭牌。
    pub const DRAGONS: Self = Self(((1_u64 << 3) - 1) << 31);
    /// 包含全部风牌和箭牌。
    pub const HONORS: Self = Self(Self::WINDS.0 | Self::DRAGONS.0);

    /// 包含万、筒、条三门序数牌、全部风牌和箭牌。
    pub const STANDARD_NO_FLOWER: Self = Self(Self::NUMBERED.0 | Self::HONORS.0);

    /// 包含春、夏、秋、冬、梅、兰、菊、竹八种花牌。
    pub const FLOWERS: Self = Self(((1_u64 << 8) - 1) << 34);

    /// 包含三门序数牌中的一和九。
    pub const TERMINALS: Self = Self(
        (1_u64 << Tile::MAN_1.index())
            | (1_u64 << Tile::MAN_9.index())
            | (1_u64 << Tile::PIN_1.index())
            | (1_u64 << Tile::PIN_9.index())
            | (1_u64 << Tile::SOU_1.index())
            | (1_u64 << Tile::SOU_9.index()),
    );
    /// 包含三门序数牌中的二至八。
    pub const SIMPLES: Self = Self(Self::NUMBERED.0 & !Self::TERMINALS.0);
    /// 包含十三幺所需的六种幺九牌和七种字牌。
    pub const ORPHANS: Self = Self(Self::TERMINALS.0 | Self::HONORS.0);

    /// 从原始 bit 创建牌面集合。
    ///
    /// 64 个 bit 都对应结构上合法的 [`Tile`]。某个扩展 bit 是否被当前规则启用，
    /// 以及它表示什么印刷牌面，由规则配置另行校验。
    pub const fn from_bits(bits: u64) -> Self {
        Self(bits)
    }

    /// 返回底层 64 个 bit。
    pub const fn bits(self) -> u64 {
        self.0
    }

    /// 创建只包含指定牌面的集合。
    pub const fn from_tile(tile: Tile) -> Self {
        Self(1_u64 << tile.index())
    }

    /// 从牌面切片创建集合，重复牌面只保留一次。
    pub const fn from_tiles(tiles: &[Tile]) -> Self {
        let mut mask = Self::EMPTY;
        let mut index = 0;
        while index < tiles.len() {
            mask.insert(tiles[index]);
            index += 1;
        }
        mask
    }

    /// 判断集合是否包含指定牌面。
    pub const fn contains(self, tile: Tile) -> bool {
        self.0 & Self::from_tile(tile).0 != 0
    }

    /// 返回集合包含的不同牌面数量，范围为 `0..=64`。
    pub const fn len(self) -> usize {
        self.0.count_ones() as usize
    }

    /// 判断集合是否不包含任何牌面。
    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }

    /// 判断自身是否为 `other` 的子集；相等和空集合也满足条件。
    pub const fn is_subset(self, other: Self) -> bool {
        self.difference(other).is_empty()
    }

    /// 判断两个集合是否没有共同牌面。
    pub const fn is_disjoint(self, other: Self) -> bool {
        self.intersection(other).is_empty()
    }

    /// 返回两个集合的并集。
    pub const fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }

    /// 返回两个集合的交集。
    pub const fn intersection(self, other: Self) -> Self {
        Self(self.0 & other.0)
    }

    /// 返回自身包含而 `other` 不包含的牌面集合。
    pub const fn difference(self, other: Self) -> Self {
        Self(self.0 & !other.0)
    }

    /// 返回只在两个集合之一出现的牌面集合。
    pub const fn symmetric_difference(self, other: Self) -> Self {
        Self(self.0 ^ other.0)
    }

    /// 返回相对于全部 64 个编码槽的补集，包含未启用的扩展牌面槽。
    ///
    /// 计算某玩法牌面域 `domain` 内的补集时，应使用 `domain.difference(mask)`。
    pub const fn complement(self) -> Self {
        Self(!self.0)
    }

    /// 插入指定牌面；此前不存在时返回 `true`，已存在时返回 `false`。
    pub const fn insert(&mut self, tile: Tile) -> bool {
        let inserted = !self.contains(tile);
        *self = self.union(Self::from_tile(tile));
        inserted
    }

    /// 移除指定牌面；此前存在时返回 `true`，不存在时返回 `false`。
    pub const fn remove(&mut self, tile: Tile) -> bool {
        let removed = self.contains(tile);
        *self = self.difference(Self::from_tile(tile));
        removed
    }

    /// 按牌面编码升序遍历成员，每个牌面只返回一次；反向遍历时按降序返回。
    pub fn iter(self) -> impl DoubleEndedIterator<Item = Tile> + FusedIterator + Clone {
        (0..Tile::CAPACITY).filter_map(move |index| {
            if self.0 & (1_u64 << index) != 0 {
                Tile::from_index(index)
            } else {
                None
            }
        })
    }
}

impl From<Tile> for TileMask {
    /// 将单个牌面转换为只包含该牌面的集合。
    fn from(tile: Tile) -> Self {
        Self::from_tile(tile)
    }
}

impl FromIterator<Tile> for TileMask {
    /// 收集牌面为集合，重复牌面只保留一次。
    fn from_iter<T: IntoIterator<Item = Tile>>(iter: T) -> Self {
        let mut mask = Self::EMPTY;
        mask.extend(iter);
        mask
    }
}

impl Extend<Tile> for TileMask {
    /// 将迭代器中的牌面加入集合，重复牌面不改变集合。
    fn extend<T: IntoIterator<Item = Tile>>(&mut self, iter: T) {
        for tile in iter {
            self.insert(tile);
        }
    }
}

impl ops::Not for TileMask {
    type Output = Self;

    /// 返回全部 64 个编码槽内的补集；玩法域内的补集应使用 [`Self::difference`]。
    fn not(self) -> Self::Output {
        self.complement()
    }
}

impl ops::BitOr for TileMask {
    type Output = Self;

    /// 返回两个集合的并集。
    fn bitor(self, rhs: Self) -> Self::Output {
        self.union(rhs)
    }
}

impl ops::BitOrAssign for TileMask {
    /// 将自身更新为两个集合的并集。
    fn bitor_assign(&mut self, rhs: Self) {
        *self = self.union(rhs);
    }
}

impl ops::BitAnd for TileMask {
    type Output = Self;

    /// 返回两个集合的交集。
    fn bitand(self, rhs: Self) -> Self::Output {
        self.intersection(rhs)
    }
}

impl ops::BitAndAssign for TileMask {
    /// 将自身更新为两个集合的交集。
    fn bitand_assign(&mut self, rhs: Self) {
        *self = self.intersection(rhs);
    }
}

impl ops::BitXor for TileMask {
    type Output = Self;

    /// 返回两个集合的对称差集。
    fn bitxor(self, rhs: Self) -> Self::Output {
        self.symmetric_difference(rhs)
    }
}

impl ops::BitXorAssign for TileMask {
    /// 将自身更新为两个集合的对称差集。
    fn bitxor_assign(&mut self, rhs: Self) {
        *self = self.symmetric_difference(rhs);
    }
}

impl ops::Sub for TileMask {
    type Output = Self;

    /// 返回自身包含而右侧集合不包含的牌面集合。
    fn sub(self, rhs: Self) -> Self::Output {
        self.difference(rhs)
    }
}

impl ops::SubAssign for TileMask {
    /// 从自身移除右侧集合包含的所有牌面。
    fn sub_assign(&mut self, rhs: Self) {
        *self = self.difference(rhs);
    }
}

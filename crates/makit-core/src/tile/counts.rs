//! 以固定长度计数向量保存各牌面的张数。

use std::ops;

use crate::{Tile, TileMask};

/// 对单个牌面的张数执行饱和加法。
const fn add_count(lhs: u8, rhs: u8) -> u8 {
    lhs.saturating_add(rhs)
}

/// 按 [`Tile::index`] 定位的 64 槽牌面计数向量。
///
/// 槽 `n` 对应紧凑索引为 `n` 的牌面，每槽使用 `u8` 表示，上限为 255。
/// 该上限独立作用于每种牌面，具体玩法应另行校验更小的实体数量上限。
/// 本类型提供的计数累加超过单槽上限时饱和为 255，与构建配置无关。
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct TileCounts([u8; Tile::CAPACITY]);

impl TileCounts {
    /// 所有槽均为零的计数向量。
    pub const EMPTY: Self = Self([0; Tile::CAPACITY]);

    /// 只含万、筒、条三门序数牌的标准 108 张牌库。
    pub const STANDARD_108: Self = Self::from_mask(TileMask::NUMBERED, 4);

    /// 不含花牌的标准 136 张牌库。
    pub const STANDARD_136: Self = Self::from_mask(TileMask::STANDARD_NO_FLOWER, 4);

    /// 每种花牌各一张的标准八张花牌。
    pub const STANDARD_FLOWER: Self = Self::from_mask(TileMask::FLOWERS, 1);

    /// 含花牌的标准 144 张牌库。
    pub const STANDARD_144: Self =
        Self::from_groups(&[(TileMask::STANDARD_NO_FLOWER, 4), (TileMask::FLOWERS, 1)]);

    /// 从按 [`Tile::index`] 排列的完整计数数组创建向量。
    ///
    /// 每槽均可使用完整的 `u8` 范围，不校验具体玩法的牌面启用情况或数量限制。
    pub const fn from_array(counts: [u8; Tile::CAPACITY]) -> Self {
        Self(counts)
    }

    /// 借用按 [`Tile::index`] 排列的完整计数数组，包含数量为零的槽。
    pub const fn as_array(&self) -> &[u8; Tile::CAPACITY] {
        &self.0
    }

    /// 取出按 [`Tile::index`] 排列的完整计数数组，包含数量为零的槽。
    pub const fn into_array(self) -> [u8; Tile::CAPACITY] {
        self.0
    }

    /// 将位集合中的每种牌初始化为相同数量。
    ///
    /// `mask` 中存在的牌数量设为 `count`，其余牌数量保持为零。
    pub const fn from_mask(mask: TileMask, count: u8) -> Self {
        Self::from_groups(&[(mask, count)])
    }

    /// 按多组牌面集合与数量构建计数向量。
    ///
    /// 每组 `(mask, count)` 为集合中的每种牌面增加 `count` 张；
    /// 不同组包含相同牌面时累加数量，未包含的牌面数量为零。
    ///
    /// 任一牌面的累计数量超过 255 时饱和为 255。
    pub const fn from_groups(groups: &[(TileMask, u8)]) -> Self {
        let mut counts = [0_u8; Tile::CAPACITY];
        let mut group_index = 0;
        while group_index < groups.len() {
            let (mask, count) = groups[group_index];
            let bits = mask.bits();
            let mut index = 0;
            while index < Tile::CAPACITY {
                if bits & (1_u64 << index) != 0 {
                    counts[index] = add_count(counts[index], count);
                }
                index += 1;
            }
            group_index += 1;
        }
        Self(counts)
    }

    /// 返回指定牌面的张数。
    pub const fn get(&self, tile: Tile) -> u8 {
        self.0[tile.index()]
    }

    /// 设置指定牌面的张数并返回原张数。
    ///
    /// 设置为零会清空该牌面的计数。
    pub const fn set(&mut self, tile: Tile, count: u8) -> u8 {
        let previous = self.0[tile.index()];
        self.0[tile.index()] = count;
        previous
    }

    /// 返回全部牌面的总张数。
    ///
    /// 总张数与单槽上限不同，最大为 `64 * 255 = 16320`，因此使用 `u16`。
    pub const fn total(&self) -> u16 {
        let mut total = 0_u16;
        let mut index = 0;
        while index < Tile::CAPACITY {
            total += self.0[index] as u16;
            index += 1;
        }
        total
    }

    /// 判断是否所有牌面的张数均为零。
    pub const fn is_empty(&self) -> bool {
        let mut index = 0;
        while index < Tile::CAPACITY {
            if self.0[index] != 0 {
                return false;
            }
            index += 1;
        }
        true
    }

    /// 判断指定牌面是否至少有一张。
    pub const fn contains(&self, tile: Tile) -> bool {
        self.get(tile) != 0
    }

    /// 返回所有非零计数对应的牌面集合。
    ///
    /// 集合只保留牌面是否存在，不保留张数。
    pub const fn mask(&self) -> TileMask {
        let mut bits = 0_u64;
        let mut index = 0;
        while index < Tile::CAPACITY {
            if self.0[index] != 0 {
                bits |= 1_u64 << index;
            }
            index += 1;
        }
        TileMask::from_bits(bits)
    }

    /// 按牌编码升序迭代非零计数，返回牌面与其张数。
    ///
    /// 每种牌面最多返回一次，不按张数重复牌面；反向迭代时按牌编码降序返回。
    pub fn iter(&self) -> impl DoubleEndedIterator<Item = (Tile, u8)> + Clone + '_ {
        TileMask::ALL.iter().filter_map(|tile| {
            let count = self.get(tile);
            (count != 0).then_some((tile, count))
        })
    }

    /// 增加指定牌面的张数，超过 255 时饱和为 255。
    pub const fn saturating_add_tile(&mut self, tile: Tile, count: u8) {
        let index = tile.index();
        self.0[index] = add_count(self.0[index], count);
    }

    /// 尝试移除指定牌面的若干张牌。
    ///
    /// 数量足够时移除并返回 `true`；不足时返回 `false`，保持原计数不变。
    /// 移除零张始终成功，包括当前没有该牌面的情况。
    pub const fn checked_remove(&mut self, tile: Tile, count: u8) -> bool {
        let index = tile.index();
        match self.0[index].checked_sub(count) {
            Some(remaining) => {
                self.0[index] = remaining;
                true
            }
            None => false,
        }
    }

    /// 尝试按牌面逐槽减去另一组计数。
    ///
    /// 所有牌面数量均足够时返回完整结果；任一槽不足时返回 `None`。
    /// 该方法按值计算，不修改原计数。
    pub const fn checked_sub(mut self, rhs: Self) -> Option<Self> {
        let mut index = 0;
        while index < Tile::CAPACITY {
            self.0[index] = match self.0[index].checked_sub(rhs.0[index]) {
                Some(remaining) => remaining,
                None => return None,
            };
            index += 1;
        }
        Some(self)
    }
}

impl Default for TileCounts {
    /// 创建所有牌面张数均为零的计数向量。
    fn default() -> Self {
        Self::EMPTY
    }
}

impl ops::Index<Tile> for TileCounts {
    type Output = u8;

    /// 借用指定牌面的张数。
    fn index(&self, tile: Tile) -> &Self::Output {
        &self.0[tile.index()]
    }
}

impl FromIterator<Tile> for TileCounts {
    /// 从逐张牌面的序列构建计数，重复牌面累加，超过单槽上限时饱和为 255。
    fn from_iter<T: IntoIterator<Item = Tile>>(iter: T) -> Self {
        let mut counts = Self::EMPTY;
        counts.extend(iter);
        counts
    }
}

impl Extend<Tile> for TileCounts {
    /// 将逐张牌面的序列累加到当前计数，超过单槽上限时饱和为 255。
    fn extend<T: IntoIterator<Item = Tile>>(&mut self, iter: T) {
        for tile in iter {
            self.saturating_add_tile(tile, 1);
        }
    }
}

impl ops::Add for TileCounts {
    type Output = Self;

    /// 将两组计数按牌面逐槽相加。
    ///
    /// 任一牌面的累计数量超过 255 时饱和为 255。
    fn add(mut self, rhs: Self) -> Self::Output {
        for (count, rhs_count) in self.0.iter_mut().zip(rhs.0) {
            *count = add_count(*count, rhs_count);
        }
        self
    }
}

impl ops::AddAssign for TileCounts {
    /// 将另一组计数按牌面逐槽累加。
    ///
    /// 任一牌面的累计数量超过 255 时饱和为 255。
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

//! 使用低四位表达固定逻辑位置的成员集合，不推断玩家行动顺序。

use std::{iter::FusedIterator, ops};

use crate::Seat;

/// 固定四个逻辑位置的成员集合。
///
/// bit `n` 对应 [`Seat::index`] 为 `n` 的位置，只有低四位有效，高四位始终为零。
/// 本类型不关联玩家或坐风，也不判断位置是否参与当前牌局；规则自行限定启用位置。
/// 成员关系及遍历顺序均不推断行动顺序。默认值为空集合。
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct SeatMask(u8);

impl SeatMask {
    /// 不包含任何位置的集合。
    pub const EMPTY: Self = Self(0);
    /// 包含全部四个逻辑位置的集合。
    pub const ALL: Self = Self(0b1111);

    /// 从原始位创建集合；高四位包含任何置位时返回 `None`。
    pub const fn from_bits(bits: u8) -> Option<Self> {
        if bits & !Self::ALL.0 != 0 {
            None
        } else {
            Some(Self(bits))
        }
    }

    /// 从原始位创建集合，显式丢弃高四位。
    pub const fn from_bits_truncate(bits: u8) -> Self {
        Self(bits & Self::ALL.0)
    }

    /// 返回底层位表示，只有低四位可能置位。
    pub const fn bits(self) -> u8 {
        self.0
    }

    /// 创建只包含指定位置的集合。
    pub const fn from_seat(seat: Seat) -> Self {
        Self(1_u8 << seat.index())
    }

    /// 从位置切片创建集合，重复位置只保留一次。
    pub const fn from_seats(seats: &[Seat]) -> Self {
        let mut mask = Self::EMPTY;
        let mut index = 0;
        while index < seats.len() {
            mask.insert(seats[index]);
            index += 1;
        }
        mask
    }

    /// 判断集合是否包含指定位置。
    pub const fn contains(self, seat: Seat) -> bool {
        self.0 & Self::from_seat(seat).0 != 0
    }

    /// 返回集合包含的不同位置数量，范围为 `0..=4`。
    pub const fn len(self) -> usize {
        self.0.count_ones() as usize
    }

    /// 判断集合是否不包含任何位置。
    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }

    /// 判断自身是否为 `other` 的子集；相等和空集合也满足条件。
    pub const fn is_subset(self, other: Self) -> bool {
        self.difference(other).is_empty()
    }

    /// 判断两个集合是否没有共同位置。
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

    /// 返回自身包含而 `other` 不包含的位置集合。
    pub const fn difference(self, other: Self) -> Self {
        Self(self.0 & !other.0)
    }

    /// 返回只在两个集合之一出现的位置集合。
    pub const fn symmetric_difference(self, other: Self) -> Self {
        Self(self.0 ^ other.0)
    }

    /// 返回相对于全部四个逻辑位置的补集，高四位始终为零。
    ///
    /// 计算某玩法位置域 `domain` 内的补集时，应使用 `domain.difference(mask)`。
    pub const fn complement(self) -> Self {
        // 按位取反会置位高四位，必须截断以保持集合的有效位不变量。
        Self::from_bits_truncate(!self.0)
    }

    /// 插入指定位置；此前不存在时返回 `true`，已存在时返回 `false`。
    pub const fn insert(&mut self, seat: Seat) -> bool {
        let inserted = !self.contains(seat);
        *self = self.union(Self::from_seat(seat));
        inserted
    }

    /// 移除指定位置；此前存在时返回 `true`，不存在时返回 `false`。
    pub const fn remove(&mut self, seat: Seat) -> bool {
        let removed = self.contains(seat);
        *self = self.difference(Self::from_seat(seat));
        removed
    }

    /// 清空所有成员。
    pub const fn clear(&mut self) {
        *self = Self::EMPTY;
    }

    /// 按位置编码升序遍历成员，每个位置只返回一次；反向遍历时按降序返回。
    ///
    /// 此顺序只保证确定性，不代表玩家行动顺序。
    pub fn iter(self) -> impl DoubleEndedIterator<Item = Seat> + FusedIterator + Clone {
        Seat::ALL
            .into_iter()
            .filter(move |seat| self.contains(*seat))
    }
}

impl From<Seat> for SeatMask {
    /// 将单个位置转换为只包含该位置的集合。
    fn from(seat: Seat) -> Self {
        Self::from_seat(seat)
    }
}

impl FromIterator<Seat> for SeatMask {
    /// 收集位置为集合，重复位置只保留一次。
    fn from_iter<T: IntoIterator<Item = Seat>>(iter: T) -> Self {
        let mut mask = Self::EMPTY;
        mask.extend(iter);
        mask
    }
}

impl Extend<Seat> for SeatMask {
    /// 将迭代器中的位置加入集合，重复位置不改变集合。
    fn extend<T: IntoIterator<Item = Seat>>(&mut self, iter: T) {
        for seat in iter {
            self.insert(seat);
        }
    }
}

impl ops::Not for SeatMask {
    type Output = Self;

    /// 返回全部四个逻辑位置内的补集；玩法域内的补集应使用 [`Self::difference`]。
    fn not(self) -> Self::Output {
        self.complement()
    }
}

impl ops::BitOr for SeatMask {
    type Output = Self;

    /// 返回两个集合的并集。
    fn bitor(self, rhs: Self) -> Self::Output {
        self.union(rhs)
    }
}

impl ops::BitOrAssign for SeatMask {
    /// 将自身更新为两个集合的并集。
    fn bitor_assign(&mut self, rhs: Self) {
        *self = self.union(rhs);
    }
}

impl ops::BitAnd for SeatMask {
    type Output = Self;

    /// 返回两个集合的交集。
    fn bitand(self, rhs: Self) -> Self::Output {
        self.intersection(rhs)
    }
}

impl ops::BitAndAssign for SeatMask {
    /// 将自身更新为两个集合的交集。
    fn bitand_assign(&mut self, rhs: Self) {
        *self = self.intersection(rhs);
    }
}

impl ops::BitXor for SeatMask {
    type Output = Self;

    /// 返回两个集合的对称差集。
    fn bitxor(self, rhs: Self) -> Self::Output {
        self.symmetric_difference(rhs)
    }
}

impl ops::BitXorAssign for SeatMask {
    /// 将自身更新为两个集合的对称差集。
    fn bitxor_assign(&mut self, rhs: Self) {
        *self = self.symmetric_difference(rhs);
    }
}

impl ops::Sub for SeatMask {
    type Output = Self;

    /// 返回自身包含而右侧集合不包含的位置集合。
    fn sub(self, rhs: Self) -> Self::Output {
        self.difference(rhs)
    }
}

impl ops::SubAssign for SeatMask {
    /// 从自身移除右侧集合包含的所有位置。
    fn sub_assign(&mut self, rhs: Self) {
        *self = self.difference(rhs);
    }
}

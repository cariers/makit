//! 以稳定逻辑位置索引的固定四槽可缺席映射。

use std::{iter::FusedIterator, ops};

use crate::{Seat, SeatMask};

/// 以 [`Seat`] 为键的固定四槽映射，每个位置可以独立保存一个值。
///
/// 槽位按座位编码排列，`None` 表示该位置没有存储值，不会移动其余位置的编号。
/// 是否存有值不决定规则是否启用该位置，也不决定玩家行动顺序。
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct SeatMap<T> {
    /// 按 [`Seat::index`] 排列的全部槽位，包括未存储值的位置。
    slots: [Option<T>; Seat::CAPACITY],
}

impl<T> SeatMap<T> {
    /// 创建全部四个位置均未存储值的映射，不要求 `T` 实现 `Default` 或 `Copy`。
    pub const fn new() -> Self {
        Self {
            slots: [const { None }; Seat::CAPACITY],
        }
    }

    /// 从按 [`Seat::index`] 排列的完整槽位数组创建映射。
    pub const fn from_array(slots: [Option<T>; Seat::CAPACITY]) -> Self {
        Self { slots }
    }

    /// 借用完整槽位数组，保留所有空槽。
    pub const fn as_array(&self) -> &[Option<T>; Seat::CAPACITY] {
        &self.slots
    }

    /// 消耗映射并取出完整槽位数组，保留所有空槽。
    pub fn into_array(self) -> [Option<T>; Seat::CAPACITY] {
        self.slots
    }

    /// 返回已存储值的位置数量，范围为 `0..=4`。
    pub const fn len(&self) -> usize {
        let mut len = 0;
        let mut index = 0;
        while index < Seat::CAPACITY {
            if self.slots[index].is_some() {
                len += 1;
            }
            index += 1;
        }
        len
    }

    /// 判断是否所有位置均未存储值。
    pub const fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// 判断指定位置是否已存储值，不判断该位置是否被规则启用。
    pub const fn contains_key(&self, seat: Seat) -> bool {
        self.slots[seat.index()].is_some()
    }

    /// 借用指定位置的值，空槽返回 `None`。
    pub const fn get(&self, seat: Seat) -> Option<&T> {
        self.slots[seat.index()].as_ref()
    }

    /// 可变借用指定位置的值，空槽返回 `None`。
    pub fn get_mut(&mut self, seat: Seat) -> Option<&mut T> {
        self.slots[seat.index()].as_mut()
    }

    /// 在指定位置存储值并返回原值，原为空槽时返回 `None`。
    ///
    /// 原值的所有权交给调用方，不会移动其他槽位的值。
    pub fn insert(&mut self, seat: Seat, value: T) -> Option<T> {
        self.slots[seat.index()].replace(value)
    }

    /// 取出指定位置的值并将该槽设为 `None`，原为空槽时返回 `None`。
    pub fn remove(&mut self, seat: Seat) -> Option<T> {
        self.slots[seat.index()].take()
    }

    /// 移除并释放全部已存储值，保留四个固定槽位。
    pub fn clear(&mut self) {
        for slot in &mut self.slots {
            *slot = None;
        }
    }

    /// 返回已存储值的位置集合，不代表规则启用的位置集合。
    pub fn mask(&self) -> SeatMask {
        let mut mask = SeatMask::EMPTY;
        for seat in self.keys() {
            mask.insert(seat);
        }
        mask
    }

    /// 按座位编码升序迭代已存储的键值对，跳过空槽。
    ///
    /// 反向迭代按座位编码降序返回；此顺序不表示玩家行动顺序。
    pub fn iter(&self) -> impl DoubleEndedIterator<Item = (Seat, &T)> + FusedIterator + '_ {
        Seat::ALL
            .into_iter()
            .zip(self.slots.iter())
            .filter_map(|(seat, slot)| slot.as_ref().map(|value| (seat, value)))
    }

    /// 按座位编码升序迭代已存储的键与值的可变引用，跳过空槽。
    ///
    /// 反向迭代按座位编码降序返回；修改值不会改变其对应的座位编号。
    pub fn iter_mut(
        &mut self,
    ) -> impl DoubleEndedIterator<Item = (Seat, &mut T)> + FusedIterator + '_ {
        Seat::ALL
            .into_iter()
            .zip(self.slots.iter_mut())
            .filter_map(|(seat, slot)| slot.as_mut().map(|value| (seat, value)))
    }

    /// 按座位编码升序迭代已存储值的键，跳过空槽，支持反向迭代。
    pub fn keys(&self) -> impl DoubleEndedIterator<Item = Seat> + FusedIterator + '_ {
        self.iter().map(|(seat, _)| seat)
    }

    /// 按座位编码升序迭代已存储值的引用，跳过空槽，支持反向迭代。
    pub fn values(&self) -> impl DoubleEndedIterator<Item = &T> + FusedIterator + '_ {
        self.iter().map(|(_, value)| value)
    }

    /// 按座位编码升序迭代已存储值的可变引用，跳过空槽，支持反向迭代。
    pub fn values_mut(&mut self) -> impl DoubleEndedIterator<Item = &mut T> + FusedIterator + '_ {
        self.iter_mut().map(|(_, value)| value)
    }
}

/// 不依赖值类型的默认构造。
impl<T> Default for SeatMap<T> {
    /// 创建全部四个位置均未存储值的映射，不要求 `T` 实现 `Default`。
    fn default() -> Self {
        Self::new()
    }
}

/// 直接读取指定逻辑位置的可缺席槽位。
impl<T> ops::Index<Seat> for SeatMap<T> {
    type Output = Option<T>;

    /// 借用指定槽位；空槽的值是 `None`，不会因缺席触发 panic。
    fn index(&self, seat: Seat) -> &Self::Output {
        &self.slots[seat.index()]
    }
}

/// 直接修改指定逻辑位置的可缺席槽位。
impl<T> ops::IndexMut<Seat> for SeatMap<T> {
    /// 可变借用指定槽位，可将其设为 `Some` 或 `None`，不改变其他位置的编号。
    ///
    /// 空槽的值是 `None`，不会因缺席触发 panic。
    fn index_mut(&mut self, seat: Seat) -> &mut Self::Output {
        &mut self.slots[seat.index()]
    }
}

/// 从键值序列构造固定槽位映射。
impl<T> FromIterator<(Seat, T)> for SeatMap<T> {
    /// 按输入顺序插入键值对，同一座位的后值覆盖前值，被覆盖的值会被释放。
    fn from_iter<I: IntoIterator<Item = (Seat, T)>>(iter: I) -> Self {
        let mut map = Self::new();
        map.extend(iter);
        map
    }
}

/// 将键值序列合并到现有映射。
impl<T> Extend<(Seat, T)> for SeatMap<T> {
    /// 按输入顺序插入键值对，同一座位的后值覆盖前值，被覆盖的值会被释放。
    fn extend<I: IntoIterator<Item = (Seat, T)>>(&mut self, iter: I) {
        for (seat, value) in iter {
            let _ = self.insert(seat, value);
        }
    }
}

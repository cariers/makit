//! 一手牌内稳定追加的弃牌历史与当前留存状态。
//!
//! 历史记录不会因取牌而删除或重新编号；只有仍留存的牌参与牌河持有量统计。
//! 编号作用域、响应窗口、取牌权限与跨容器提交由上层管理。

use std::iter::FusedIterator;

use crate::{Discard, Seat, Tile};

/// 一个 [`River`] 实例所属的一手牌内，从零开始的弃牌记录编号。
///
/// 编号标识一次弃牌事实，不是实体牌身份，也不跨牌局或容器实例唯一。
/// 从原始数值构造编号不证明记录存在；上层应先确认作用域，再交给容器查询或取牌。
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct DiscardId(u32);

impl DiscardId {
    /// 从零基原始编号构造引用，不检查记录是否存在或属于当前作用域。
    pub const fn from_value(value: u32) -> Self {
        Self(value)
    }

    /// 返回零基原始编号。
    pub const fn value(self) -> u32 {
        self.0
    }
}

/// 弃牌历史及每条记录对应的牌是否仍留在牌河中。
///
/// 所有座位的弃牌共用按发生顺序追加的记录；取牌只改变留存状态，不修改原始事实或
/// 复用编号。历史数量与当前持有牌数分别通过 [`Self::len`] 和 [`Self::present_len`]
/// 查询，不能将完整历史重复计入实体持有量。
///
/// 开始新一手牌时创建新容器；容器不能自行识别来自其他实例的编号。
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct River {
    /// 仅追加的历史条目，索引对应零基弃牌编号。
    entries: Vec<RiverEntry>,
}

/// 一次不可变弃牌事实与它的当前留存状态。
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct RiverEntry {
    /// 已经发生的原始弃牌事实。
    discard: Discard,
    /// 该次弃牌对应的牌是否仍由此牌河持有。
    present: bool,
}

/// 将已受编号容量约束的内部索引转换为弃牌编号。
fn discard_id(index: usize) -> DiscardId {
    // 追加前检查最后一个索引，因此每个已存条目的索引都必定能表示为 u32。
    DiscardId::from_value(u32::try_from(index).expect("stored discard index must fit in u32"))
}

impl River {
    /// 创建不包含历史记录的空牌河。
    pub const fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    /// 返回历史记录总数，包括对应牌已经被取走的记录。
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// 判断是否从未追加过弃牌记录，不表示当前是否仍持有牌。
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// 追加一次弃牌事实，初始状态为仍留在牌河中，返回其稳定编号。
    ///
    /// # Errors
    ///
    /// 无法分配下一个编号时返回 [`RiverError::IdExhausted`]，不修改历史。
    pub fn append(&mut self, discard: Discard) -> Result<DiscardId, RiverError> {
        let mut ids = self.append_many(vec![discard])?;
        // 单条追加成功后，批量结果必定包含唯一一个新编号。
        Ok(ids
            .pop()
            .expect("a successful single append must return one discard id"))
    }

    /// 按输入顺序整批追加弃牌事实，全部初始留存，返回对应的稳定编号。
    ///
    /// 追加前检查整批编号容量，不回绕或复用编号；空批次成功返回空向量，
    /// 即使已无可分配编号也不报错。
    ///
    /// # Errors
    ///
    /// 当前长度加请求数量无法表示，或任一新编号超出 `u32` 范围时返回
    /// [`RiverError::IdExhausted`]，整批不修改历史。
    pub fn append_many(&mut self, discards: Vec<Discard>) -> Result<Vec<DiscardId>, RiverError> {
        let requested = discards.len();
        if requested == 0 {
            return Ok(Vec::new());
        }
        let start = self.entries.len();
        let end = start
            .checked_add(requested)
            .ok_or(RiverError::IdExhausted { requested })?;
        // 检查最后一个索引而非最大编号加一，避免在 32 位平台计算容量时溢出。
        if u32::try_from(end - 1).is_err() {
            return Err(RiverError::IdExhausted { requested });
        }
        let ids = (start..end).map(discard_id).collect();
        self.entries
            .extend(discards.into_iter().map(|discard| RiverEntry {
                discard,
                present: true,
            }));
        Ok(ids)
    }

    /// 查询原始弃牌事实，牌被取走后仍可读取；编号不存在时返回 `None`。
    pub fn get(&self, id: DiscardId) -> Option<&Discard> {
        self.entry(id).map(|entry| &entry.discard)
    }

    /// 按编号升序迭代全部历史，包含对应牌已被取走的记录，支持反向迭代。
    pub fn iter(
        &self,
    ) -> impl DoubleEndedIterator<Item = (DiscardId, &Discard)> + ExactSizeIterator + FusedIterator + '_
    {
        self.entries
            .iter()
            .enumerate()
            .map(|(index, entry)| (discard_id(index), &entry.discard))
    }

    /// 按编号升序迭代指定座位的全部弃牌历史，包含已取走的牌，支持反向迭代。
    pub fn iter_for(
        &self,
        seat: Seat,
    ) -> impl DoubleEndedIterator<Item = (DiscardId, &Discard)> + FusedIterator + '_ {
        self.iter()
            .filter(move |(_, discard)| discard.seat() == seat)
    }

    /// 查询该条记录对应的牌是否仍留存，编号不存在时返回 `None`。
    pub fn is_present(&self, id: DiscardId) -> Option<bool> {
        self.entry(id).map(|entry| entry.present)
    }

    /// 按编号升序迭代仍由牌河持有的牌及其弃牌事实，支持反向迭代。
    ///
    /// 已经取走的牌不出现在结果中，但仍保留在完整历史中。
    pub fn present(
        &self,
    ) -> impl DoubleEndedIterator<Item = (DiscardId, &Discard)> + FusedIterator + '_ {
        self.entries
            .iter()
            .enumerate()
            .filter(|(_, entry)| entry.present)
            .map(|(index, entry)| (discard_id(index), &entry.discard))
    }

    /// 返回当前仍由牌河持有的精确牌数，不包含已经取走的牌。
    pub fn present_len(&self) -> usize {
        self.entries.iter().filter(|entry| entry.present).count()
    }

    /// 取走仍留存的牌并返回原始牌面，保留对应历史与编号。
    ///
    /// 不检查该弃牌是否仍在响应窗口中，也不决定取牌目的地或业务权限。
    ///
    /// # Errors
    ///
    /// 编号不存在时返回 [`RiverError::UnknownDiscard`]；对应牌已取走时返回
    /// [`RiverError::AlreadyTaken`]。失败不改变任何留存状态。
    pub fn take(&mut self, id: DiscardId) -> Result<Tile, RiverError> {
        let mut tiles = self.take_many([id])?;
        // 单编号取牌成功后，批量结果必定包含唯一一张牌。
        Ok(tiles
            .pop()
            .expect("a successful single take must return one tile"))
    }

    /// 全部校验后批量取走仍留存的牌，返回顺序与请求编号顺序一致。
    ///
    /// 接受数组、切片或向量，空批次成功且不修改状态。所有原始历史和编号始终保留。
    /// 本方法只保证单容器操作原子性，跨容器转移与当前响应权限由上层校验。
    ///
    /// # Errors
    ///
    /// 编号不存在时返回 [`RiverError::UnknownDiscard`]；对应牌已被取走时返回
    /// [`RiverError::AlreadyTaken`]；批内重复引用时返回 [`RiverError::DuplicateDiscard`]。
    /// 任一校验失败都不会修改任何留存状态。
    pub fn take_many(&mut self, ids: impl AsRef<[DiscardId]>) -> Result<Vec<Tile>, RiverError> {
        let ids = ids.as_ref();
        let mut indices = Vec::with_capacity(ids.len());
        let mut tiles = Vec::with_capacity(ids.len());
        // 先收集并验证全部索引和结果，再修改留存状态，避免中途失败造成部分消费。
        for (position, &id) in ids.iter().enumerate() {
            let index =
                usize::try_from(id.value()).map_err(|_| RiverError::UnknownDiscard { id })?;
            let entry = self
                .entries
                .get(index)
                .ok_or(RiverError::UnknownDiscard { id })?;
            if !entry.present {
                return Err(RiverError::AlreadyTaken { id });
            }
            if ids[..position].contains(&id) {
                return Err(RiverError::DuplicateDiscard { id });
            }
            indices.push(index);
            tiles.push(entry.discard.tile());
        }
        for index in indices {
            self.entries[index].present = false;
        }
        Ok(tiles)
    }

    /// 将外部编号受检转换为平台索引，再查询内部条目。
    fn entry(&self, id: DiscardId) -> Option<&RiverEntry> {
        let index = usize::try_from(id.value()).ok()?;
        self.entries.get(index)
    }
}

/// 弃牌编号分配或留存牌消费失败的结构化错误。
///
/// 对外消息使用英文，保留弃牌编号或请求数量；不包含底层来源错误。
#[derive(Clone, Debug, Eq, PartialEq, thiserror::Error)]
pub enum RiverError {
    /// 当前容器中不存在请求编号对应的历史记录。
    #[error("unknown discard id {}", .id.value())]
    UnknownDiscard {
        /// 请求查询或消费的弃牌编号。
        id: DiscardId,
    },
    /// 历史记录存在，但对应的牌已被取走。
    #[error("discard id {} has already been taken", .id.value())]
    AlreadyTaken {
        /// 对应牌已不再留存的弃牌编号。
        id: DiscardId,
    },
    /// 同一批请求重复引用了一次弃牌。
    #[error("duplicate discard id {}", .id.value())]
    DuplicateDiscard {
        /// 在批内重复出现的弃牌编号。
        id: DiscardId,
    },
    /// 当前编号或平台索引容量无法容纳完整追加批次。
    #[error("discard id capacity exhausted for {requested} new records")]
    IdExhausted {
        /// 本次请求追加的记录总数。
        requested: usize,
    },
}

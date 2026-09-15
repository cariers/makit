//! 保留原始牌面、旧牌区与本次摸牌区的手牌容器。
//!
//! 本模块维护分区和取牌的结构约束，不决定牌局张数上限、牌面启用情况或行动合法性，
//! 也不存储已声明固定组。按牌面取牌使用 [`Hand::take_tiles`]、[`Hand::take_tiles_many`]，
//! 按牌面出牌使用 [`Hand::discard_tile`]、[`Hand::discard_tiles`]。相等的 [`Tile`]
//! 不标识具体副本，重复请求表示所需张数；匹配沿原排列选取不同副本，优先使用旧牌区。
//!
//! 数组索引相关方法仅操作当前排列中的位置，不是实体牌 ID；上层收到索引操作时应校验
//! 相应状态版本，避免变更或排序后继续使用旧索引。

use std::{iter::FusedIterator, ops};

use crate::{Tile, TileCounts, TileMask};

/// 按旧牌区与本次摸牌区连续存储的原始牌面容器。
///
/// `[0, draw_cursor)` 为旧牌区，`[draw_cursor, len)` 为摸牌区；摸牌区可以包含多张牌。
/// 普通取牌保留剩余分区，成功的非空出牌操作则将余牌全部归入旧牌区；牌局是否允许这些操作由
/// 玩法层判断。相同牌面的副本可以重复存在，不为每张副本分配独立身份。
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Hand {
    /// 按旧牌在前、摸牌在后的顺序保存全部原始牌面。
    tiles: Vec<Tile>,
    /// 旧牌区长度，同时是摸牌区起点，始终不超过牌向量长度。
    draw_cursor: usize,
}

impl Hand {
    /// 创建旧牌区和摸牌区均为空的手牌。
    pub const fn new() -> Self {
        Self {
            tiles: Vec::new(),
            draw_cursor: 0,
        }
    }

    /// 将给定原始牌面全部作为初始旧牌，保留输入顺序。
    pub fn from_tiles(tiles: Vec<Tile>) -> Self {
        let draw_cursor = tiles.len();
        Self { tiles, draw_cursor }
    }

    /// 从完整牌向量与摸牌区起点重建手牌，保留输入顺序。
    ///
    /// # Errors
    ///
    /// `draw_cursor` 大于牌向量长度时返回 [`HandError::InvalidDrawCursor`]。
    pub fn from_parts(tiles: Vec<Tile>, draw_cursor: usize) -> Result<Self, HandError> {
        if draw_cursor > tiles.len() {
            return Err(HandError::InvalidDrawCursor {
                cursor: draw_cursor,
                len: tiles.len(),
            });
        }
        Ok(Self { tiles, draw_cursor })
    }

    /// 消耗手牌，返回原始牌向量与摸牌区起点。
    pub fn into_parts(self) -> (Vec<Tile>, usize) {
        (self.tiles, self.draw_cursor)
    }

    /// 消耗手牌并返回原始牌向量，丢弃旧牌区与摸牌区的边界信息。
    pub fn into_tiles(self) -> Vec<Tile> {
        self.tiles
    }

    /// 返回完整原始牌序列，保留重复牌面与分区顺序。
    pub fn tiles(&self) -> &[Tile] {
        &self.tiles
    }

    /// 返回摸牌区起点之前的全部旧牌。
    pub fn held_tiles(&self) -> &[Tile] {
        &self.tiles[..self.draw_cursor]
    }

    /// 返回本次摸入且尚未归并的全部牌，可能为空或包含多张。
    pub fn drawn_tiles(&self) -> &[Tile] {
        &self.tiles[self.draw_cursor..]
    }

    /// 返回摸牌区起点，同时也是旧牌区长度。
    pub fn draw_cursor(&self) -> usize {
        self.draw_cursor
    }

    /// 返回全部原始牌的精确张数，不限制具体玩法的手牌上限。
    pub fn len(&self) -> usize {
        self.tiles.len()
    }

    /// 判断手牌是否为空。
    pub fn is_empty(&self) -> bool {
        self.tiles.is_empty()
    }

    /// 判断摸牌区是否至少有一张尚未归并的牌。
    pub fn has_drawn(&self) -> bool {
        self.draw_cursor < self.tiles.len()
    }

    /// 借用当前排列中指定索引处的原始牌面，越界时返回 `None`。
    pub fn get(&self, index: usize) -> Option<&Tile> {
        self.tiles.get(index)
    }

    /// 返回指定原始牌面的精确张数，不执行通配替换或饱和截断。
    pub fn count(&self, tile: Tile) -> usize {
        self.tiles
            .iter()
            .filter(|&&candidate| candidate == tile)
            .count()
    }

    /// 按当前存储顺序逐张迭代原始牌面，保留重复牌面，支持反向迭代。
    pub fn iter(
        &self,
    ) -> impl DoubleEndedIterator<Item = Tile> + ExactSizeIterator + FusedIterator + Clone + '_
    {
        self.tiles.iter().copied()
    }

    /// 返回所有原始牌面的集合，不保留重复张数或分区信息。
    pub fn mask(&self) -> TileMask {
        TileMask::from_tiles(&self.tiles)
    }

    /// 返回原始牌面的饱和计数摘要，不保留分区信息。
    ///
    /// 每种牌面最多记录 255 张，因此不能用于校验超过该上限的批量扣牌请求；
    /// 需要精确数量时使用 [`Self::count`] 或 [`Self::take_tiles`]。
    pub fn counts(&self) -> TileCounts {
        self.iter().collect()
    }

    /// 将一张摸牌追加到摸牌区末尾，并返回它在当前排列中的索引。
    ///
    /// 保留原摸牌区起点，不自动归并先前摸入的牌。
    pub fn draw(&mut self, tile: Tile) -> usize {
        let index = self.tiles.len();
        self.tiles.push(tile);
        index
    }

    /// 按输入顺序将多张摸牌追加到摸牌区，保留原摸牌区起点。
    pub fn draw_many(&mut self, tiles: impl IntoIterator<Item = Tile>) {
        self.tiles.extend(tiles);
    }

    /// 将非摸牌来源的收牌按输入顺序插入旧牌区末尾，原摸牌区仍保留。
    ///
    /// 可用于交换收牌等场景；新增牌归入旧牌区，不会自动获得摸牌来源语义。
    pub fn receive(&mut self, tiles: impl IntoIterator<Item = Tile>) {
        let received: Vec<Tile> = tiles.into_iter().collect();
        let count = received.len();
        // 插在分区边界并同步推进游标，使原摸牌区只移动位置、不改变来源。
        self.tiles
            .splice(self.draw_cursor..self.draw_cursor, received);
        self.draw_cursor += count;
    }

    /// 将当前全部摸牌归入旧牌区，不调整牌的存储顺序。
    pub fn merge_drawn(&mut self) {
        self.draw_cursor = self.tiles.len();
    }

    /// 清空全部牌，并将摸牌区起点重置为零。
    pub fn clear(&mut self) {
        self.tiles.clear();
        self.draw_cursor = 0;
    }

    /// 仅按原始牌编码排序旧牌区，保留摸牌区顺序与分区边界。
    ///
    /// 排序后旧牌区索引可能改变，上层应同步更新状态版本。
    pub fn sort_held(&mut self) {
        self.tiles[..self.draw_cursor].sort();
    }

    /// 仅按给定键稳定排序旧牌区，可由玩法为扩展牌定义展示次序。
    ///
    /// 保留摸牌区顺序与分区边界；排序后旧牌区索引可能改变，上层应同步更新状态版本。
    pub fn sort_held_by_key<K: Ord, F: FnMut(&Tile) -> K>(&mut self, key: F) {
        self.tiles[..self.draw_cursor].sort_by_key(key);
    }

    /// 按原始牌面打出一张牌，记录移除前的分区来源，并将余牌全部归入旧牌区。
    ///
    /// 选取当前排列中最早的匹配牌，优先使用旧牌区，不执行通配替换。
    /// 相同牌面不具有独立身份；返回的来源标记仅反映实际选中副本的原分区。
    ///
    /// # Errors
    ///
    /// 手牌中不存在该原始牌面时返回 [`HandError::InsufficientTiles`]，手牌及分区保持不变。
    pub fn discard_tile(&mut self, tile: Tile) -> Result<HandDiscard, HandError> {
        let mut discarded = self.discard_tiles([tile])?;
        // 单张请求成功时必定产生唯一结果，来源由批量接口按移除前的分区计算。
        Ok(discarded
            .pop()
            .expect("a successful single discard must return one tile"))
    }

    /// 按原始牌面原子打出多张牌，重复请求匹配不同副本，结果按请求顺序返回。
    ///
    /// 按操作前的排列选取尚未选中的匹配牌，优先使用旧牌区，不执行通配替换。
    /// 全部出牌的来源均按操作前的同一分区边界判定，余牌保留相对顺序。
    /// 非空批次成功后将全部余牌归入旧牌区；空请求返回空向量，不改变手牌或分区。
    /// 支持数组、切片和向量。
    ///
    /// 一次打出多张牌应使用本方法；循环调用 [`Self::discard_tile`] 会在第一次出牌后
    /// 归并摸牌区，无法保留同一批操作前的来源语义。
    ///
    /// # Errors
    ///
    /// 任一原始牌面的副本不足时返回 [`HandError::InsufficientTiles`]，包含该牌面的
    /// 整批请求总张数与操作前的精确可用张数。任何校验失败都不会修改手牌或分区。
    pub fn discard_tiles(
        &mut self,
        tiles: impl AsRef<[Tile]>,
    ) -> Result<Vec<HandDiscard>, HandError> {
        let indices = self.select_tiles(tiles.as_ref())?;
        self.discard_many(indices)
    }

    /// 移除指定索引的牌并记录它在移除前是否属于摸牌区，然后将余牌全部归入旧牌区。
    ///
    /// 保留剩余牌的相对顺序；摸牌区允许多张牌，因此来源标记是否符合玩法的摸切定义
    /// 仍由玩法层解释。
    ///
    /// # Errors
    ///
    /// 索引越界时返回 [`HandError::IndexOutOfBounds`]，手牌及分区保持不变。
    pub fn discard(&mut self, index: usize) -> Result<HandDiscard, HandError> {
        let mut discarded = self.discard_many([index])?;
        // 批量接口为每个请求索引返回一项，单索引成功时必定得到唯一结果。
        Ok(discarded
            .pop()
            .expect("a successful single discard must return one tile"))
    }

    /// 原子打出多个索引处的牌，全部来源均按操作前的同一分区边界判定。
    ///
    /// 结果按请求索引顺序返回，余牌保留相对顺序。非空批量成功后才将全部余牌归入
    /// 旧牌区；空索引列表成功返回空向量，不改变手牌或分区。
    /// 索引数量可以在运行时确定，支持数组、切片和向量。
    ///
    /// 一次打出多张牌应使用本方法；循环调用单张 [`Self::discard`] 会在第一次出牌后
    /// 归并摸牌区，无法保留同一批操作前的来源语义，后续索引也会随删除而变化。
    ///
    /// # Errors
    ///
    /// 索引越界时返回 [`HandError::IndexOutOfBounds`]；同一索引出现多次时返回
    /// [`HandError::DuplicateIndex`]。任何校验失败都不会修改手牌或分区。
    pub fn discard_many(
        &mut self,
        indices: impl AsRef<[usize]>,
    ) -> Result<Vec<HandDiscard>, HandError> {
        let indices = indices.as_ref();
        let old_cursor = self.draw_cursor;
        let tiles = self.take_many(indices)?;
        if !indices.is_empty() {
            self.draw_cursor = self.tiles.len();
        }
        // 取牌会维护剩余分区，出牌来源仍必须以整批操作前的边界计算。
        Ok(tiles
            .into_iter()
            .zip(indices)
            .map(|(tile, &index)| HandDiscard {
                tile,
                from_draw: index >= old_cursor,
            })
            .collect())
    }

    /// 原子取出多个索引处的牌，返回顺序与输入索引顺序一致。
    ///
    /// 全部索引相对操作前的排列，剩余牌保留相对顺序和所属分区；只缩短旧牌区中被
    /// 移除的部分，不自动归并摸牌。`N` 为零时成功返回空数组且不改变手牌。
    ///
    /// # Errors
    ///
    /// 索引越界时返回 [`HandError::IndexOutOfBounds`]；同一索引出现多次时返回
    /// [`HandError::DuplicateIndex`]。任何校验失败都不会修改手牌或分区。
    pub fn take<const N: usize>(&mut self, indices: [usize; N]) -> Result<[Tile; N], HandError> {
        let taken = self.take_many(indices)?;
        // 成功的批量取牌与输入索引一一对应，结果长度必定等于数组长度 N。
        Ok(std::array::from_fn(|index| taken[index]))
    }

    /// 原子取出运行时确定的多个索引处的牌，返回顺序与输入索引顺序一致。
    ///
    /// 支持数组、切片和向量，全部索引相对操作前的排列。剩余牌保留相对顺序和所属
    /// 分区，只缩短旧牌区中被移除的部分，不自动归并摸牌。
    /// 空索引列表成功返回空向量，不改变手牌或分区。
    ///
    /// # Errors
    ///
    /// 索引越界时返回 [`HandError::IndexOutOfBounds`]；同一索引出现多次时返回
    /// [`HandError::DuplicateIndex`]。任何校验失败都不会修改手牌或分区。
    pub fn take_many(&mut self, indices: impl AsRef<[usize]>) -> Result<Vec<Tile>, HandError> {
        let indices = indices.as_ref();
        // 全部校验和返回值收集先于删除，防止批量操作中途失败留下部分取牌。
        for (position, &index) in indices.iter().enumerate() {
            if index >= self.tiles.len() {
                return Err(HandError::IndexOutOfBounds {
                    index,
                    len: self.tiles.len(),
                });
            }
            if indices[..position].contains(&index) {
                return Err(HandError::DuplicateIndex { index });
            }
        }
        let taken = indices.iter().map(|&index| self.tiles[index]).collect();
        let removed_held = indices
            .iter()
            .filter(|&&index| index < self.draw_cursor)
            .count();
        let mut sorted = indices.to_vec();
        sorted.sort_unstable();
        // 从后向前删除使尚未处理的原索引保持有效，Vec::remove 保留余牌相对顺序。
        for index in sorted.into_iter().rev() {
            self.tiles.remove(index);
        }
        self.draw_cursor -= removed_held;
        Ok(taken)
    }

    /// 按原始牌面原子取牌，重复请求会匹配不同副本，返回顺序与请求顺序一致。
    ///
    /// 每次选取当前排列中最早且尚未选中的匹配位置，因此优先使用旧牌区。
    /// 不执行通配替换，不使用饱和计数校验；剩余牌保留相对顺序和所属分区。
    /// `N` 为零时成功返回空数组且不改变手牌。
    ///
    /// # Errors
    ///
    /// 任一原始牌面的副本不足时返回 [`HandError::InsufficientTiles`]，包含该牌面的
    /// 请求总张数与精确可用张数，手牌及分区保持不变。
    pub fn take_tiles<const N: usize>(&mut self, tiles: [Tile; N]) -> Result<[Tile; N], HandError> {
        let taken = self.take_tiles_many(tiles)?;
        // 成功的批量取牌与请求牌面一一对应，结果长度必定等于数组长度 N。
        Ok(std::array::from_fn(|index| taken[index]))
    }

    /// 按原始牌面原子取出运行时确定的多张牌，返回顺序与请求顺序一致。
    ///
    /// 支持数组、切片和向量，重复请求匹配不同副本。按操作前的排列选取尚未选中的
    /// 匹配牌，优先使用旧牌区，不执行通配替换，也不使用饱和计数校验。
    /// 剩余牌保留相对顺序和所属分区，不自动归并摸牌；空请求返回空向量且不改变手牌。
    ///
    /// # Errors
    ///
    /// 任一原始牌面的副本不足时返回 [`HandError::InsufficientTiles`]，包含该牌面的
    /// 整批请求总张数与操作前的精确可用张数。任何校验失败都不会修改手牌或分区。
    pub fn take_tiles_many(&mut self, tiles: impl AsRef<[Tile]>) -> Result<Vec<Tile>, HandError> {
        let indices = self.select_tiles(tiles.as_ref())?;
        self.take_many(indices)
    }

    /// 在原排列中匹配整批牌面，得到仅供当前操作使用的内部位置。
    fn select_tiles(&self, tiles: &[Tile]) -> Result<Vec<usize>, HandError> {
        let mut indices = Vec::with_capacity(tiles.len());
        // 匹配完成前不移除牌，重复请求必须使用不同副本，出牌来源也保留原分区语义。
        for &tile in tiles {
            let matching = self
                .tiles
                .iter()
                .enumerate()
                .find(|(index, candidate)| **candidate == tile && !indices.contains(index));
            match matching {
                Some((index, _)) => indices.push(index),
                None => {
                    return Err(HandError::InsufficientTiles {
                        tile,
                        required: tiles.iter().filter(|&&candidate| candidate == tile).count(),
                        available: self.count(tile),
                    });
                }
            }
        }
        Ok(indices)
    }
}

/// 一次成功出牌取得的原始牌面与移除前的分区来源。
///
/// 来源仅区分旧牌区与摸牌区，不自动证明摸切；多张摸牌场景的业务解释由玩法决定。
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct HandDiscard {
    /// 实际从手牌移除的原始牌面。
    tile: Tile,
    /// 移除前该位置是否位于摸牌区。
    from_draw: bool,
}

impl HandDiscard {
    /// 返回实际出牌的原始牌面。
    pub const fn tile(&self) -> Tile {
        self.tile
    }

    /// 判断出牌在移除前是否属于摸牌区，不自动推导玩法中的摸切判定。
    pub const fn is_from_draw(&self) -> bool {
        self.from_draw
    }
}

/// 手牌构造或取牌操作未满足结构约束时的错误。
///
/// 对外消息使用英文，保留索引、张数或牌编码；不包含底层来源错误。
#[derive(Clone, Debug, Eq, PartialEq, thiserror::Error)]
pub enum HandError {
    /// 摸牌区起点超出当前牌向量长度。
    #[error("draw cursor {cursor} exceeds hand length {len}")]
    InvalidDrawCursor {
        /// 请求使用的摸牌区起点。
        cursor: usize,
        /// 当前牌向量长度。
        len: usize,
    },
    /// 请求访问的索引不在当前手牌内。
    #[error("hand index {index} is out of bounds for length {len}")]
    IndexOutOfBounds {
        /// 越界索引。
        index: usize,
        /// 当前牌向量长度。
        len: usize,
    },
    /// 同一批取牌重复使用了一个索引。
    #[error("duplicate hand index {index}")]
    DuplicateIndex {
        /// 重复出现的索引。
        index: usize,
    },
    /// 指定原始牌面的精确副本数不足。
    #[error(
        "insufficient copies of tile code {}: required {required}, available {available}",
        .tile.code()
    )]
    InsufficientTiles {
        /// 缺少的原始牌面，不执行通配解释。
        tile: Tile,
        /// 本批请求中该牌面的总张数。
        required: usize,
        /// 手牌中该牌面的精确可用张数。
        available: usize,
    },
}

/// 将原始牌向量作为初始旧牌构造手牌。
impl From<Vec<Tile>> for Hand {
    /// 保留输入顺序，将全部牌归入旧牌区。
    fn from(tiles: Vec<Tile>) -> Self {
        Self::from_tiles(tiles)
    }
}

/// 从逐张原始牌面的序列构造初始手牌。
impl FromIterator<Tile> for Hand {
    /// 保留迭代顺序，将全部牌归入旧牌区。
    fn from_iter<T: IntoIterator<Item = Tile>>(iter: T) -> Self {
        Self::from_tiles(iter.into_iter().collect())
    }
}

/// 只读访问当前排列中的原始牌面。
impl ops::Index<usize> for Hand {
    type Output = Tile;

    /// 借用当前索引处的原始牌面；索引不表示稳定实体身份。
    ///
    /// # Panics
    ///
    /// 索引不小于当前手牌长度时 panic；需要处理越界时使用 [`Hand::get`]。
    fn index(&self, index: usize) -> &Self::Output {
        &self.tiles[index]
    }
}

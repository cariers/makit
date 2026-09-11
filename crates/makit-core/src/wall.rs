//! 固定牌序与头尾游标组成的牌墙，仅管理当前剩余区间的双端取牌。
//!
//! 构造时保留输入顺序，不洗牌；开门、补牌及其他玩法约束由上层决定。

use std::{error::Error, fmt, iter::FusedIterator};

use crate::{Tile, TileCounts, TileMask};

/// 以固定牌序和头尾游标管理剩余牌的牌墙。
///
/// 剩余牌始终为连续区间，前后端仅表示输入顺序，不预设桌面方位、牌墩或补牌规则。
/// 取牌只移动游标，不修改底层牌序；已取牌仍占据底层存储，但不参与剩余牌查询。
/// 不提供复位或插入接口，默认值为空牌墙。
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Wall {
    /// 初始化后不再修改的完整牌序。
    tiles: Box<[Tile]>,
    /// 剩余区间的起点，包含此位置。
    head: usize,
    /// 剩余区间的终点，不包含此位置；始终满足 head <= tail <= tiles.len()。
    tail: usize,
}

impl Wall {
    /// 创建空牌墙。
    pub fn new() -> Self {
        Self::default()
    }

    /// 按输入顺序收集牌面，保留重复副本，不洗牌或校验玩法牌库。
    pub fn from_tiles(tiles: impl IntoIterator<Item = Tile>) -> Self {
        let tiles: Box<[Tile]> = tiles.into_iter().collect();
        let tail = tiles.len();
        Self {
            tiles,
            head: 0,
            tail,
        }
    }

    /// 按牌编码升序展开计数，创建有序牌墙。
    ///
    /// 结果只代表传入摘要的张数，不恢复原始牌序，也不能恢复饱和计数丢失的张数。
    pub fn from_counts(counts: &TileCounts) -> Self {
        Self::from_tiles(
            counts
                .iter()
                .flat_map(|(tile, count)| std::iter::repeat_n(tile, usize::from(count))),
        )
    }

    /// 返回精确剩余张数，不使用饱和计数摘要。
    pub const fn len(&self) -> usize {
        self.tail - self.head
    }

    /// 判断是否没有剩余牌。
    pub const fn is_empty(&self) -> bool {
        self.head == self.tail
    }

    /// 按从前到后的顺序借用全部剩余牌，不包含已取牌。
    pub fn as_slice(&self) -> &[Tile] {
        &self.tiles[self.head..self.tail]
    }

    /// 借用前端牌面；空牌墙返回 `None`。
    pub fn front(&self) -> Option<&Tile> {
        self.as_slice().first()
    }

    /// 借用后端牌面；空牌墙返回 `None`。
    pub fn back(&self) -> Option<&Tile> {
        self.as_slice().last()
    }

    /// 按从当前前端开始的零基索引借用牌面；越界返回 `None`。
    ///
    /// 索引相对于剩余区间，前端取牌后会改变，不表示初始牌序的稳定位置。
    pub fn get(&self, index: usize) -> Option<&Tile> {
        self.as_slice().get(index)
    }

    /// 从前向后逐张遍历剩余牌，支持反向遍历并保留重复牌面。
    pub fn iter(
        &self,
    ) -> impl DoubleEndedIterator<Item = Tile> + ExactSizeIterator + FusedIterator + Clone + '_
    {
        self.as_slice().iter().copied()
    }

    /// 从前端取走一张牌；空牌墙返回 `None`，不移动游标。
    pub fn draw_front(&mut self) -> Option<Tile> {
        let tile = *self.front()?;
        self.head += 1;
        Some(tile)
    }

    /// 从后端取走一张牌；空牌墙返回 `None`，不移动游标。
    pub fn draw_back(&mut self) -> Option<Tile> {
        let tile = *self.back()?;
        self.tail -= 1;
        Some(tile)
    }

    /// 从前端取走 `n` 张牌，结果顺序与连续调用 [`Self::draw_front`] 一致。
    ///
    /// 取零张成功并保持原状态。
    ///
    /// # Errors
    ///
    /// 剩余数量不足时返回 [`WallError::InsufficientTiles`]，不移动任何游标。
    pub fn draw_front_many(&mut self, n: usize) -> Result<Vec<Tile>, WallError> {
        self.check_available(n)?;
        // 完整验证并准备结果后再移动游标，避免批次失败时消耗部分牌。
        let tiles = self.as_slice()[..n].to_vec();
        self.head += n;
        Ok(tiles)
    }

    /// 从后端取走 `n` 张牌，结果顺序与连续调用 [`Self::draw_back`] 一致。
    ///
    /// 例如原顺序为 `[A, B, C, D]` 时，取两张返回 `[D, C]`。
    /// 取零张成功并保持原状态。
    ///
    /// # Errors
    ///
    /// 剩余数量不足时返回 [`WallError::InsufficientTiles`]，不移动任何游标。
    pub fn draw_back_many(&mut self, n: usize) -> Result<Vec<Tile>, WallError> {
        self.check_available(n)?;
        let start = self.tail - n;
        // 后端结果按实际消费顺序收集；收集完成后才提交新的尾游标。
        let tiles = self.tiles[start..self.tail].iter().rev().copied().collect();
        self.tail = start;
        Ok(tiles)
    }

    /// 消费牌墙，按从前到后的当前顺序返回全部剩余牌，丢弃已取牌。
    pub fn into_tiles(self) -> Vec<Tile> {
        let Self { tiles, head, tail } = self;
        let mut tiles = tiles.into_vec();
        // 复用原有分配，先截去已取后缀，再移除已取前缀。
        tiles.truncate(tail);
        drop(tiles.drain(..head));
        tiles
    }

    /// 返回指定原始牌面的精确剩余张数，不受计数摘要的单槽上限影响。
    pub fn count(&self, tile: Tile) -> usize {
        self.iter().filter(|&candidate| candidate == tile).count()
    }

    /// 返回剩余牌的成员集合，只保留牌面是否存在，不保留张数或顺序。
    pub fn mask(&self) -> TileMask {
        self.iter().collect()
    }

    /// 返回剩余牌的饱和计数摘要，不保留顺序。
    ///
    /// 同一牌面超过 255 张时该槽饱和为 255，不能用此摘要核对任意长牌墙的精确数量。
    /// 需要精确张数时使用 [`Self::len`] 或 [`Self::count`]。
    pub fn counts(&self) -> TileCounts {
        self.iter().collect()
    }

    /// 修改游标前校验完整批次，零张请求始终成功。
    fn check_available(&self, requested: usize) -> Result<(), WallError> {
        if requested > self.len() {
            Err(WallError::InsufficientTiles {
                requested,
                remaining: self.len(),
            })
        } else {
            Ok(())
        }
    }
}

/// 支持按迭代器顺序收集牌面建立牌墙。
impl FromIterator<Tile> for Wall {
    /// 保留重复副本，不洗牌或校验玩法牌库。
    fn from_iter<T: IntoIterator<Item = Tile>>(iter: T) -> Self {
        Self::from_tiles(iter)
    }
}

/// 牌墙批量取牌时的结构化错误。
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WallError {
    /// 剩余牌不足以满足完整批次的取牌请求。
    InsufficientTiles {
        /// 本次请求取走的张数。
        requested: usize,
        /// 操作前的精确剩余张数。
        remaining: usize,
    },
}

impl fmt::Display for WallError {
    /// 输出包含请求值与实际数量的英文错误消息。
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InsufficientTiles {
                requested,
                remaining,
            } => write!(
                f,
                "insufficient tiles: requested {requested}, remaining {remaining}"
            ),
        }
    }
}

impl Error for WallError {}

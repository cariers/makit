//! 按声明顺序保存固定组，并提供吃、碰及三种杠的独立操作。
//!
//! 本模块维护固定组集合的结构，不管理手牌、牌河或未决交互，也不校验玩法牌形。

use std::{error::Error, fmt, iter::FusedIterator, ops::Index};

use crate::{
    ChowMeld, ClaimedKongMeld, ConcealedKongMeld, Meld, MeldKind, PongMeld, Seat, Tile, TileCounts,
    TileMask,
};

/// 按声明顺序保存的固定组集合。
///
/// 新声明追加到末尾，加杠在原碰组位置替换，已有组索引不会因这些操作改变。
/// 索引只引用当前集合中的固定组记录，不表示某张手牌的副本身份，也不能跨集合使用。
/// 不提供任意覆盖、删除或可变切片接口；不预设组数上限，不校验顺子、同牌、通配解释
/// 或声明权限，这些条件由玩法层决定。
///
/// `Eq`、`Hash` 比较有序的完整记录，包含每组的形成方式、原始牌面顺序和来源。
#[derive(Clone, Debug, Default, Eq, Hash, PartialEq)]
pub struct Melds {
    /// 按声明顺序保存的固定组；加杠保留原碰组的位置。
    melds: Vec<Meld>,
}

impl Melds {
    /// 创建空固定组集合，不预分配空间。
    pub const fn new() -> Self {
        Self { melds: Vec::new() }
    }

    /// 按输入顺序收集已有固定组记录，不排序、不合并或校验玩法合法性。
    ///
    /// 允许导入已经成立的加杠记录，不要求其历史曾在当前集合执行。
    pub fn from_melds(melds: impl IntoIterator<Item = Meld>) -> Self {
        Self {
            melds: melds.into_iter().collect(),
        }
    }

    /// 消费集合，按保存顺序返回全部固定组记录。
    pub fn into_melds(self) -> Vec<Meld> {
        self.melds
    }

    /// 返回固定组数量，不是组内牌的总张数。
    pub const fn len(&self) -> usize {
        self.melds.len()
    }

    /// 判断是否没有固定组。
    pub const fn is_empty(&self) -> bool {
        self.melds.is_empty()
    }

    /// 按保存顺序借用全部固定组，不允许通过切片修改集合。
    pub fn as_slice(&self) -> &[Meld] {
        &self.melds
    }

    /// 按当前集合中的零基组索引借用固定组；越界返回 `None`。
    pub fn get(&self, index: usize) -> Option<&Meld> {
        self.melds.get(index)
    }

    /// 按保存顺序借用每个固定组，支持反向迭代并提供精确剩余组数。
    pub fn iter(
        &self,
    ) -> impl DoubleEndedIterator<Item = &Meld> + ExactSizeIterator + FusedIterator + Clone + '_
    {
        self.melds.iter()
    }

    /// 依次遍历每个固定组的原始牌面，保留组间顺序、组内顺序及重复牌面。
    ///
    /// 支持反向迭代，不执行通配解释或牌面替换。
    pub fn tiles(&self) -> impl DoubleEndedIterator<Item = Tile> + FusedIterator + Clone + '_ {
        self.melds
            .iter()
            .flat_map(|meld| meld.tiles().iter().copied())
    }

    /// 返回全部固定组包含的精确牌数；杠按四张计数。
    pub fn tile_count(&self) -> usize {
        self.melds.iter().map(Meld::tile_count).sum()
    }

    /// 返回指定原始牌面的精确张数，不受计数摘要的单槽上限影响。
    pub fn count(&self, tile: Tile) -> usize {
        self.tiles().filter(|&candidate| candidate == tile).count()
    }

    /// 返回指定形成方式的固定组数量，不是组内牌数。
    pub fn kind_count(&self, kind: MeldKind) -> usize {
        self.melds.iter().filter(|meld| meld.kind() == kind).count()
    }

    /// 返回全部固定组原始牌面的成员集合，不保留张数、组边界或顺序。
    pub fn mask(&self) -> TileMask {
        self.tiles().collect()
    }

    /// 返回全部固定组原始牌面的饱和计数摘要，不保留组边界或顺序。
    ///
    /// 同一牌面超过 255 张时该槽饱和为 255，不能以此摘要核对任意长集合的精确数量。
    /// 需要精确张数时使用 [`Self::tile_count`] 或 [`Self::count`]。
    pub fn counts(&self) -> TileCounts {
        self.tiles().collect()
    }

    /// 用本家两张牌、鸣取牌和来源追加吃牌声明，返回新组索引。
    ///
    /// 保留输入顺序，不校验顺子、通配解释或鸣取来源是否合法。
    pub fn chow(&mut self, owned: [Tile; 2], claimed: Tile, source: Seat) -> usize {
        self.append(ChowMeld::new(owned, claimed, source).into())
    }

    /// 用本家两张牌、鸣取牌和来源追加碰牌声明，返回新组索引。
    ///
    /// 保留输入顺序，不要求原始牌面直接相等；通配解释及声明权限由玩法层校验。
    pub fn pong(&mut self, owned: [Tile; 2], claimed: Tile, source: Seat) -> usize {
        self.append(PongMeld::new(owned, claimed, source).into())
    }

    /// 用本家三张牌、鸣取牌和来源追加直接明杠声明，返回新组索引。
    ///
    /// 保留输入顺序，不校验杠牌形、通配解释或鸣取来源是否合法。
    pub fn claimed_kong(&mut self, owned: [Tile; 3], claimed: Tile, source: Seat) -> usize {
        self.append(ClaimedKongMeld::new(owned, claimed, source).into())
    }

    /// 用本家四张牌追加暗杠声明，返回新组索引。
    ///
    /// 保留输入顺序和原始牌面，不校验杠牌形或声明条件；牌面隐藏由投影层处理。
    pub fn concealed_kong(&mut self, tiles: [Tile; 4]) -> usize {
        self.append(ConcealedKongMeld::new(tiles).into())
    }

    /// 将指定位置的碰组原位升级为加杠，保留原碰牌顺序及鸣取来源。
    ///
    /// 新增牌追加在组内第四张，集合长度及其他组的位置不变。
    /// 不校验新增牌与原碰牌的规则等价关系，也不处理抢杠等未决交互。
    ///
    /// # Errors
    ///
    /// 索引越界时返回 [`MeldsError::IndexOutOfBounds`]；目标不是碰组时返回
    /// [`MeldsError::ExpectedPong`]。失败时集合保持不变。
    pub fn add_kong(&mut self, index: usize, tile: Tile) -> Result<(), MeldsError> {
        let len = self.len();
        let meld = self
            .melds
            .get_mut(index)
            .ok_or(MeldsError::IndexOutOfBounds { index, len })?;
        let Meld::Pong(pong) = *meld else {
            return Err(MeldsError::ExpectedPong {
                index,
                actual: meld.kind(),
            });
        };
        // 完成全部结构校验后才替换原组，避免失败时破坏原碰牌记录。
        *meld = pong.with_added_tile(tile).into();
        Ok(())
    }

    /// 统一追加已构造的声明，既有组的位置保持不变。
    fn append(&mut self, meld: Meld) -> usize {
        let index = self.len();
        self.melds.push(meld);
        index
    }
}

/// 按迭代器顺序收集已有固定组记录。
impl FromIterator<Meld> for Melds {
    /// 不排序、不合并或校验已有记录的玩法合法性。
    fn from_iter<T: IntoIterator<Item = Meld>>(iter: T) -> Self {
        Self::from_melds(iter)
    }
}

/// 将固定组集合借用为只读切片。
impl AsRef<[Meld]> for Melds {
    /// 按保存顺序返回全部固定组。
    fn as_ref(&self) -> &[Meld] {
        self.as_slice()
    }
}

/// 使用当前集合中的零基组索引只读访问固定组。
impl Index<usize> for Melds {
    type Output = Meld;

    /// 返回指定位置的固定组；需要处理越界时应使用 [`Melds::get`]。
    ///
    /// # Panics
    ///
    /// `index >= self.len()` 时 panic。
    fn index(&self, index: usize) -> &Self::Output {
        &self.melds[index]
    }
}

/// 消费集合并按保存顺序移出固定组。
impl IntoIterator for Melds {
    type Item = Meld;
    type IntoIter = std::vec::IntoIter<Meld>;

    /// 返回支持双端遍历的固定组所有权迭代器。
    fn into_iter(self) -> Self::IntoIter {
        self.melds.into_iter()
    }
}

/// 借用集合并按保存顺序访问固定组。
impl<'a> IntoIterator for &'a Melds {
    type Item = &'a Meld;
    type IntoIter = std::slice::Iter<'a, Meld>;

    /// 返回支持双端遍历的固定组共享借用迭代器。
    fn into_iter(self) -> Self::IntoIter {
        self.melds.iter()
    }
}

/// 固定组集合操作中的结构化错误。
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MeldsError {
    /// 请求的组索引超出当前集合范围。
    IndexOutOfBounds {
        /// 本次请求的零基组索引。
        index: usize,
        /// 操作前的固定组数量。
        len: usize,
    },
    /// 加杠目标不是尚未升级的碰组。
    ExpectedPong {
        /// 本次请求升级的组索引。
        index: usize,
        /// 目标记录实际的形成方式。
        actual: MeldKind,
    },
}

impl fmt::Display for MeldsError {
    /// 输出包含组索引及实际状态的英文错误消息。
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::IndexOutOfBounds { index, len } => {
                write!(f, "meld index {index} is out of bounds for length {len}")
            }
            Self::ExpectedPong { index, actual } => {
                write!(f, "expected pong at meld index {index}, found {actual:?}")
            }
        }
    }
}

impl Error for MeldsError {}

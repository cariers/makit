//! 已声明固定组的结构、原始牌面与鸣取来源。
//!
//! 固定数组约束组内张数，组类别和原始牌面一并保留；牌形、通配解释、鸣取权限及
//! 来源是否为他家由玩法层校验。这里不对牌排序，不擦除牌面，也不保存未决交互。

use std::iter::FusedIterator;

use crate::{Seat, Tile, TileCounts, TileMask};

/// 已声明固定组的形成方式。
///
/// 本类型区分吃、碰及三种杠，只描述固定组本身，不判断具体玩法是否合法。
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum MeldKind {
    /// 吃牌形成的顺子。
    Chow,
    /// 碰牌形成的刻子。
    Pong,
    /// 取得他家弃牌后直接形成的明杠。
    ClaimedKong,
    /// 使用本家四张牌形成的暗杠。
    ConcealedKong,
    /// 在已有碰牌上增加第四张牌形成的加杠。
    AddedKong,
}

impl MeldKind {
    /// 返回该类固定组包含的牌数，吃和碰为三张，杠为四张。
    pub const fn tile_count(self) -> usize {
        match self {
            Self::Chow | Self::Pong => 3,
            Self::ClaimedKong | Self::ConcealedKong | Self::AddedKong => 4,
        }
    }

    /// 判断该组是否属于明杠、暗杠或加杠。
    pub const fn is_kong(self) -> bool {
        matches!(
            self,
            Self::ClaimedKong | Self::ConcealedKong | Self::AddedKong
        )
    }

    /// 判断该组本身是否为明组，暗杠除外的四类均返回 `true`。
    ///
    /// 此属性不判断整手牌是否门清，也不决定牌面向各受众的展示方式。
    pub const fn is_open(self) -> bool {
        !self.is_concealed()
    }

    /// 判断该组是否为暗杠，不判断整手牌是否门清。
    pub const fn is_concealed(self) -> bool {
        matches!(self, Self::ConcealedKong)
    }
}

/// 吃牌声明的三张原始牌面与鸣取来源。
///
/// 前两张为本家提供的牌，最后一张为鸣取牌；本类型不证明原始牌面构成合法顺子。
/// 数组顺序保留构造输入与鸣取位置，`Eq`、`Hash` 比较包含顺序和来源的完整记录。
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct ChowMeld {
    /// 按本家两张、鸣取一张排列的原始牌面。
    tiles: [Tile; 3],
    /// 鸣取牌的来源位置，不表示该组拥有者。
    source: Seat,
}

impl ChowMeld {
    /// 用本家两张牌、鸣取牌及来源构造吃牌声明，保留输入顺序。
    ///
    /// 只保证结构与张数，不校验顺子牌形、通配解释或来源是否为他家；这些由玩法层校验。
    pub const fn new(owned: [Tile; 2], claimed: Tile, source: Seat) -> Self {
        Self {
            tiles: [owned[0], owned[1], claimed],
            source,
        }
    }

    /// 返回全部原始牌面，鸣取牌位于最后，不进行排序或牌面替换。
    pub const fn tiles(&self) -> &[Tile; 3] {
        &self.tiles
    }

    /// 返回吃牌类别。
    pub const fn kind(&self) -> MeldKind {
        MeldKind::Chow
    }

    /// 返回鸣取牌的来源位置，不验证该位置与拥有者的关系。
    pub const fn source(&self) -> Seat {
        self.source
    }

    /// 返回鸣取牌的原始牌面。
    pub const fn claimed_tile(&self) -> Tile {
        self.tiles[2]
    }
}

/// 碰牌声明的三张原始牌面与鸣取来源。
///
/// 前两张为本家提供的牌，最后一张为鸣取牌；考虑通配解释，原始牌面不要求直接相等。
/// 数组顺序保留构造输入与鸣取位置，`Eq`、`Hash` 比较包含顺序和来源的完整记录。
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct PongMeld {
    /// 按本家两张、鸣取一张排列的原始牌面。
    tiles: [Tile; 3],
    /// 鸣取牌的来源位置，不表示该组拥有者。
    source: Seat,
}

impl PongMeld {
    /// 用本家两张牌、鸣取牌及来源构造碰牌声明，保留输入顺序。
    ///
    /// 只保证结构与张数，不校验刻子牌形、通配解释或来源是否为他家；这些由玩法层校验。
    pub const fn new(owned: [Tile; 2], claimed: Tile, source: Seat) -> Self {
        Self {
            tiles: [owned[0], owned[1], claimed],
            source,
        }
    }

    /// 返回全部原始牌面，鸣取牌位于最后，不进行排序或牌面替换。
    pub const fn tiles(&self) -> &[Tile; 3] {
        &self.tiles
    }

    /// 返回碰牌类别。
    pub const fn kind(&self) -> MeldKind {
        MeldKind::Pong
    }

    /// 返回鸣取牌的来源位置，不验证该位置与拥有者的关系。
    pub const fn source(&self) -> Seat {
        self.source
    }

    /// 返回鸣取牌的原始牌面。
    pub const fn claimed_tile(&self) -> Tile {
        self.tiles[2]
    }

    /// 在本碰牌声明后追加一张原始牌面，构造加杠声明。
    ///
    /// 保留原碰牌及其来源，仅约束结构与张数；加杠牌形与执行条件由玩法层校验。
    pub const fn with_added_tile(self, tile: Tile) -> AddedKongMeld {
        AddedKongMeld::new(self, tile)
    }
}

/// 直接取得鸣取牌形成的明杠声明。
///
/// 前三张为本家提供的牌，最后一张为鸣取牌；原始牌面的规则等价关系由玩法层解释。
/// 数组顺序保留构造输入与鸣取位置，`Eq`、`Hash` 比较包含顺序和来源的完整记录。
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct ClaimedKongMeld {
    /// 按本家三张、鸣取一张排列的原始牌面。
    tiles: [Tile; 4],
    /// 鸣取牌的来源位置，不表示该组拥有者。
    source: Seat,
}

impl ClaimedKongMeld {
    /// 用本家三张牌、鸣取牌及来源构造明杠声明，保留输入顺序。
    ///
    /// 只保证结构与张数，不校验杠牌形、通配解释或来源是否为他家；这些由玩法层校验。
    pub const fn new(owned: [Tile; 3], claimed: Tile, source: Seat) -> Self {
        Self {
            tiles: [owned[0], owned[1], owned[2], claimed],
            source,
        }
    }

    /// 返回全部原始牌面，鸣取牌位于最后，不进行排序或牌面替换。
    pub const fn tiles(&self) -> &[Tile; 4] {
        &self.tiles
    }

    /// 返回直接鸣取形成的明杠类别。
    pub const fn kind(&self) -> MeldKind {
        MeldKind::ClaimedKong
    }

    /// 返回鸣取牌的来源位置，不验证该位置与拥有者的关系。
    pub const fn source(&self) -> Seat {
        self.source
    }

    /// 返回鸣取牌的原始牌面。
    pub const fn claimed_tile(&self) -> Tile {
        self.tiles[3]
    }
}

/// 本家四张牌形成的暗杠声明，不包含鸣取来源。
///
/// 原始牌面与输入顺序全部保留，向受众隐藏牌面的方式由投影层决定。
/// `Eq`、`Hash` 比较包含数组顺序的完整记录，不只比较牌面多重集合。
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct ConcealedKongMeld {
    /// 按输入顺序保存的四张原始牌面。
    tiles: [Tile; 4],
}

impl ConcealedKongMeld {
    /// 用本家四张牌构造暗杠声明，保留输入顺序。
    ///
    /// 只保证结构与张数，不校验杠牌形、通配解释或声明条件；这些由玩法层校验。
    pub const fn new(tiles: [Tile; 4]) -> Self {
        Self { tiles }
    }

    /// 返回按输入顺序保存的全部原始牌面，不进行排序或牌面擦除。
    pub const fn tiles(&self) -> &[Tile; 4] {
        &self.tiles
    }

    /// 返回暗杠类别。
    pub const fn kind(&self) -> MeldKind {
        MeldKind::ConcealedKong
    }
}

/// 在已有碰牌声明上追加一张牌形成的加杠声明。
///
/// 前三张保留原碰牌的顺序，第四张为新增牌；来源仍是原碰牌的鸣取来源。
/// `Eq`、`Hash` 比较包含数组顺序和来源的完整记录，不只比较牌面多重集合。
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct AddedKongMeld {
    /// 原碰牌的三张原始牌面与追加的第四张牌。
    tiles: [Tile; 4],
    /// 原碰牌的鸣取来源，不表示追加牌的来源或该组拥有者。
    source: Seat,
}

impl AddedKongMeld {
    /// 用原碰牌声明及新增牌构造加杠声明，保留原碰牌与鸣取来源。
    ///
    /// 只保证结构与张数，不校验杠牌形、通配解释或加杠时机；这些由玩法层校验。
    pub const fn new(pong: PongMeld, added: Tile) -> Self {
        // 原碰牌前缀与来源只存一份，避免加杠结果和重复保存的原碰信息发生分歧。
        Self {
            tiles: [pong.tiles[0], pong.tiles[1], pong.tiles[2], added],
            source: pong.source,
        }
    }

    /// 返回原碰牌三张与新增一张原始牌面，不进行排序或牌面替换。
    pub const fn tiles(&self) -> &[Tile; 4] {
        &self.tiles
    }

    /// 返回加杠类别。
    pub const fn kind(&self) -> MeldKind {
        MeldKind::AddedKong
    }

    /// 返回原碰牌的鸣取来源，不是新增牌的来源。
    pub const fn source(&self) -> Seat {
        self.source
    }

    /// 返回原碰牌中鸣取牌的原始牌面，即第三张牌。
    pub const fn claimed_tile(&self) -> Tile {
        self.tiles[2]
    }

    /// 返回加杠时追加的第四张原始牌面。
    pub const fn added_tile(&self) -> Tile {
        self.tiles[3]
    }

    /// 按保存的前三张牌与鸣取来源重建原碰牌声明。
    ///
    /// 保留原碰牌的顺序和来源，不重新解释或校验牌形。
    pub const fn pong(&self) -> PongMeld {
        PongMeld::new([self.tiles[0], self.tiles[1]], self.tiles[2], self.source)
    }
}

/// 已声明固定组的完整结构与原始牌面。
///
/// 各变体的数组长度约束牌数，但本类型不构成玩法合法性的证明，也不保存未决的鸣牌
/// 或抢杠交互。原始牌面的规则等价关系、来源合法性及声明时机由玩法层校验。
/// 数组顺序用于保留构造输入和鸣取位置，`Eq`、`Hash` 比较包括类别、顺序及来源的
/// 完整记录，不只比较牌面多重集合。
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Meld {
    /// 吃牌形成的顺子。
    Chow(ChowMeld),
    /// 碰牌形成的刻子。
    Pong(PongMeld),
    /// 取得他家弃牌后直接形成的明杠。
    ClaimedKong(ClaimedKongMeld),
    /// 使用本家四张牌形成的暗杠。
    ConcealedKong(ConcealedKongMeld),
    /// 在已有碰牌上增加第四张牌形成的加杠。
    AddedKong(AddedKongMeld),
}

impl Meld {
    /// 返回该声明固定组的形成方式。
    pub const fn kind(&self) -> MeldKind {
        match self {
            Self::Chow(meld) => meld.kind(),
            Self::Pong(meld) => meld.kind(),
            Self::ClaimedKong(meld) => meld.kind(),
            Self::ConcealedKong(meld) => meld.kind(),
            Self::AddedKong(meld) => meld.kind(),
        }
    }

    /// 返回组内牌数，吃和碰为三张，杠为四张。
    pub const fn tile_count(&self) -> usize {
        self.kind().tile_count()
    }

    /// 判断该组是否为明杠、暗杠或加杠。
    pub const fn is_kong(&self) -> bool {
        self.kind().is_kong()
    }

    /// 判断该组本身是否为明组，不判断整手牌是否门清或牌面如何向受众展示。
    pub const fn is_open(&self) -> bool {
        self.kind().is_open()
    }

    /// 判断该组是否为暗杠，不判断整手牌是否门清。
    pub const fn is_concealed(&self) -> bool {
        self.kind().is_concealed()
    }

    /// 返回该组全部原始牌面，不排序、替换或擦除牌面。
    ///
    /// 吃、碰与直接明杠的鸣取牌位于最后；加杠保留原碰牌的前三张，新增牌位于第四张。
    pub const fn tiles(&self) -> &[Tile] {
        match self {
            Self::Chow(meld) => meld.tiles(),
            Self::Pong(meld) => meld.tiles(),
            Self::ClaimedKong(meld) => meld.tiles(),
            Self::ConcealedKong(meld) => meld.tiles(),
            Self::AddedKong(meld) => meld.tiles(),
        }
    }

    /// 返回鸣取来源，暗杠返回 `None`，加杠保留原碰牌的鸣取来源。
    ///
    /// 此位置不表示组拥有者，是否为他家由玩法层校验。
    pub const fn source(&self) -> Option<Seat> {
        match self {
            Self::Chow(meld) => Some(meld.source()),
            Self::Pong(meld) => Some(meld.source()),
            Self::ClaimedKong(meld) => Some(meld.source()),
            Self::ConcealedKong(_) => None,
            Self::AddedKong(meld) => Some(meld.source()),
        }
    }

    /// 返回鸣取牌的原始牌面，暗杠返回 `None`，加杠返回原碰牌的鸣取牌。
    pub const fn claimed_tile(&self) -> Option<Tile> {
        match self {
            Self::Chow(meld) => Some(meld.claimed_tile()),
            Self::Pong(meld) => Some(meld.claimed_tile()),
            Self::ClaimedKong(meld) => Some(meld.claimed_tile()),
            Self::ConcealedKong(_) => None,
            Self::AddedKong(meld) => Some(meld.claimed_tile()),
        }
    }

    /// 返回加杠时追加的原始牌面，其余四类返回 `None`。
    pub const fn added_tile(&self) -> Option<Tile> {
        match self {
            Self::AddedKong(meld) => Some(meld.added_tile()),
            _ => None,
        }
    }

    /// 返回组内原始牌面的集合，只保留成员关系，不保留重复张数或通配解释。
    pub const fn mask(&self) -> TileMask {
        TileMask::from_tiles(self.tiles())
    }

    /// 返回组内原始牌面的计数，重复牌面累加，不执行通配替换。
    ///
    /// 组内最多四张牌，任何单槽计数均不会达到饱和上限。
    pub const fn counts(&self) -> TileCounts {
        let tiles = self.tiles();
        let mut counts = TileCounts::EMPTY;
        let mut index = 0;
        while index < tiles.len() {
            counts.saturating_add_tile(tiles[index], 1);
            index += 1;
        }
        counts
    }

    /// 按存储顺序逐张迭代原始牌面，保留重复牌面，支持反向迭代。
    pub fn iter(
        &self,
    ) -> impl DoubleEndedIterator<Item = Tile> + ExactSizeIterator + FusedIterator + Clone + '_
    {
        self.tiles().iter().copied()
    }
}

/// 将吃牌声明包装为统一固定组。
impl From<ChowMeld> for Meld {
    /// 保留吃牌声明的原始牌面与来源。
    fn from(meld: ChowMeld) -> Self {
        Self::Chow(meld)
    }
}

/// 将碰牌声明包装为统一固定组。
impl From<PongMeld> for Meld {
    /// 保留碰牌声明的原始牌面与来源。
    fn from(meld: PongMeld) -> Self {
        Self::Pong(meld)
    }
}

/// 将直接明杠声明包装为统一固定组。
impl From<ClaimedKongMeld> for Meld {
    /// 保留明杠声明的原始牌面与来源。
    fn from(meld: ClaimedKongMeld) -> Self {
        Self::ClaimedKong(meld)
    }
}

/// 将暗杠声明包装为统一固定组。
impl From<ConcealedKongMeld> for Meld {
    /// 保留暗杠声明的全部原始牌面，不擦除牌面。
    fn from(meld: ConcealedKongMeld) -> Self {
        Self::ConcealedKong(meld)
    }
}

/// 将加杠声明包装为统一固定组。
impl From<AddedKongMeld> for Meld {
    /// 保留加杠声明中的原碰牌信息与新增牌。
    fn from(meld: AddedKongMeld) -> Self {
        Self::AddedKong(meld)
    }
}

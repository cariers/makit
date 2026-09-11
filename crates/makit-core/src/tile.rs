//! 固定牌面与扩展牌面的紧凑编码及配套集合、计数类型。

use std::num::NonZeroU8;

mod counts;
mod mask;

pub use counts::TileCounts;
pub use mask::TileMask;

/// 为内部定义的牌面常量构造合法编码。
///
/// # Panics
///
/// 内部常量使用的 `code` 不在 `1..=64` 时触发 panic。
const fn tile(code: u8) -> Tile {
    match Tile::from_code(code) {
        Some(tile) => tile,
        None => panic!("tile code must be in 1..=64"),
    }
}

/// 以稳定牌编码表示的一种麻将牌面。
///
/// 牌编码范围为 `1..=64`，其中 `1..=42` 是本 crate 固定定义的标准牌，
/// `43..=64` 是由规则配置解释的扩展牌槽。与牌编码对应的紧凑索引范围为
/// `0..=63`，只用于内存中的数组槽与 bit 定位。
///
/// 相同牌面的多个副本具有相同的 `Tile` 值，本类型不表示实体牌身份。
/// 相等性、排序与哈希均依据牌编码；通配替换不会改变原始牌面的编码。
/// 合法编码不代表当前规则启用该牌面，扩展槽的含义由规则自行约定。
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Tile(NonZeroU8);

impl Tile {
    /// 可用的牌编码槽数量。
    pub const CAPACITY: usize = 64;
    /// 最小合法牌编码。
    pub const MIN_CODE: u8 = 1;
    /// 最大合法牌编码。
    pub const MAX_CODE: u8 = 64;

    /// 一万。
    pub const MAN_1: Self = tile(1);
    /// 二万。
    pub const MAN_2: Self = tile(2);
    /// 三万。
    pub const MAN_3: Self = tile(3);
    /// 四万。
    pub const MAN_4: Self = tile(4);
    /// 五万。
    pub const MAN_5: Self = tile(5);
    /// 六万。
    pub const MAN_6: Self = tile(6);
    /// 七万。
    pub const MAN_7: Self = tile(7);
    /// 八万。
    pub const MAN_8: Self = tile(8);
    /// 九万。
    pub const MAN_9: Self = tile(9);

    /// 一筒。
    pub const PIN_1: Self = tile(10);
    /// 二筒。
    pub const PIN_2: Self = tile(11);
    /// 三筒。
    pub const PIN_3: Self = tile(12);
    /// 四筒。
    pub const PIN_4: Self = tile(13);
    /// 五筒。
    pub const PIN_5: Self = tile(14);
    /// 六筒。
    pub const PIN_6: Self = tile(15);
    /// 七筒。
    pub const PIN_7: Self = tile(16);
    /// 八筒。
    pub const PIN_8: Self = tile(17);
    /// 九筒。
    pub const PIN_9: Self = tile(18);

    /// 一条。
    pub const SOU_1: Self = tile(19);
    /// 二条。
    pub const SOU_2: Self = tile(20);
    /// 三条。
    pub const SOU_3: Self = tile(21);
    /// 四条。
    pub const SOU_4: Self = tile(22);
    /// 五条。
    pub const SOU_5: Self = tile(23);
    /// 六条。
    pub const SOU_6: Self = tile(24);
    /// 七条。
    pub const SOU_7: Self = tile(25);
    /// 八条。
    pub const SOU_8: Self = tile(26);
    /// 九条。
    pub const SOU_9: Self = tile(27);

    /// 东风。
    pub const EAST: Self = tile(28);
    /// 南风。
    pub const SOUTH: Self = tile(29);
    /// 西风。
    pub const WEST: Self = tile(30);
    /// 北风。
    pub const NORTH: Self = tile(31);
    /// 红中。
    pub const RED_DRAGON: Self = tile(32);
    /// 发财。
    pub const GREEN_DRAGON: Self = tile(33);
    /// 白板。
    pub const WHITE_DRAGON: Self = tile(34);

    /// 春。
    pub const SPRING: Self = tile(35);
    /// 夏。
    pub const SUMMER: Self = tile(36);
    /// 秋。
    pub const AUTUMN: Self = tile(37);
    /// 冬。
    pub const WINTER: Self = tile(38);
    /// 梅。
    pub const PLUM: Self = tile(39);
    /// 兰。
    pub const ORCHID: Self = tile(40);
    /// 菊。
    pub const CHRYSANTHEMUM: Self = tile(41);
    /// 竹。
    pub const BAMBOO: Self = tile(42);

    /// 标准牌的最大编码。
    pub const STANDARD_MAX_CODE: u8 = 42;
    /// 扩展牌的最小编码。
    pub const EXTENSION_MIN_CODE: u8 = 43;
    /// 扩展牌的最大编码。
    pub const EXTENSION_MAX_CODE: u8 = Self::MAX_CODE;

    /// 从一基牌编码创建牌。
    ///
    /// `code` 不在 `1..=64` 时返回 `None`。
    pub const fn from_code(code: u8) -> Option<Self> {
        if code > Self::MAX_CODE || code < Self::MIN_CODE {
            return None;
        }

        match NonZeroU8::new(code) {
            Some(code) => Some(Self(code)),
            None => None,
        }
    }

    /// 返回一基牌编码。
    pub const fn code(self) -> u8 {
        self.0.get()
    }

    /// 返回是否属于本 crate 固定定义的 `1..=42` 标准牌面。
    ///
    /// 此方法只检查编码分类，不判断当前规则是否使用该牌面。
    pub const fn is_standard(self) -> bool {
        self.code() <= Self::STANDARD_MAX_CODE
    }

    /// 返回是否属于由规则解释的 `43..=64` 扩展牌面槽。
    ///
    /// 此方法不判断扩展槽是否已被当前规则赋予含义。
    pub const fn is_extension(self) -> bool {
        self.code() >= Self::EXTENSION_MIN_CODE
    }

    /// 返回仅包含当前牌面的集合。
    pub const fn to_mask(self) -> TileMask {
        TileMask::from_tile(self)
    }

    /// 返回供 [`TileMask`] 和 [`TileCounts`] 定位槽位的零基紧凑索引。
    pub const fn index(self) -> usize {
        (self.code() - 1) as usize
    }

    /// 从零基紧凑索引创建牌。
    ///
    /// `index` 不在 `0..=63` 时返回 `None`。
    /// 紧凑索引只用于内存中的 bit 和数组定位，不应作为业务标识持久化或交换。
    pub const fn from_index(index: usize) -> Option<Self> {
        // 先检查边界再窄化，避免过大的 usize 截断后被误认作合法牌面。
        if index >= Self::CAPACITY {
            return None;
        }

        Self::from_code((index + 1) as u8)
    }
}

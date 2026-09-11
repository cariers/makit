//! 固定四位置逻辑环、独立风位及以东风位置为锚点的坐风映射。

mod map;
mod mask;

use std::num::NonZeroU8;

pub use map::SeatMap;
pub use mask::SeatMask;

/// 为内部定义的座位常量构造合法编码。
///
/// # Panics
///
/// 内部常量使用的 `code` 不在 `1..=4` 时触发 panic。
const fn seat(code: u8) -> Seat {
    match Seat::from_code(code) {
        Some(seat) => seat,
        None => panic!("seat code must be in 1..=4"),
    }
}

/// 牌桌中编号为 `1..=4` 的稳定逻辑位置。
///
/// 编号按顺时针方向排列，但不表示玩家、当前坐风或行动顺序。三人或二人规则只选择
/// 本类型的部分值，不改变四位置循环，也不会让相邻位置跳过未使用的编号。
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Seat(NonZeroU8);

impl Seat {
    /// 固定逻辑位置数量。
    pub const CAPACITY: usize = 4;
    /// 最小合法座位 ID。
    pub const MIN_ID: u8 = 1;
    /// 最大合法座位 ID。
    pub const MAX_ID: u8 = 4;

    /// 全部逻辑位置，按座位 ID `1、2、3、4` 排列。
    pub const ALL: [Self; Self::CAPACITY] = [seat(1), seat(2), seat(3), seat(4)];

    /// 返回 `1..=4` 范围内的一基座位编码。
    pub const fn code(self) -> u8 {
        self.0.get()
    }

    /// 返回 `0..=3` 范围内的零基紧凑索引。
    pub const fn index(self) -> usize {
        (self.code() - 1) as usize
    }

    /// 返回仅包含当前位置的集合。
    pub const fn to_mask(self) -> SeatMask {
        SeatMask::from_seat(self)
    }

    /// 从一基座位编码创建逻辑位置。
    ///
    /// `code` 不在 `1..=4` 时返回 `None`，不检查该位置是否被当前规则启用。
    pub const fn from_code(code: u8) -> Option<Self> {
        if code > Self::MAX_ID || code < Self::MIN_ID {
            return None;
        }
        match NonZeroU8::new(code) {
            Some(code) => Some(Self(code)),
            None => None,
        }
    }

    /// 从零基紧凑索引创建逻辑位置。
    ///
    /// `index` 不在 `0..=3` 时返回 `None`，不检查该位置是否被当前规则启用。
    pub const fn from_index(index: usize) -> Option<Self> {
        // 先检查边界再窄化，避免过大的 usize 截断后被误认作合法座位编码。
        if index >= Self::CAPACITY {
            return None;
        }
        Self::from_code((index + 1) as u8)
    }

    /// 返回沿固定四位置循环顺时针移动 `steps` 步后的位置。
    ///
    /// 支持任意 `usize` 步数，按四取模，每四步回到原位，不跳过未启用的位置。
    pub const fn clockwise(self, steps: usize) -> Self {
        // 先将步数约束在一个周期内，避免与索引相加时发生溢出。
        let steps = steps % Self::CAPACITY;
        Self::ALL[(self.index() + steps) % Self::CAPACITY]
    }

    /// 返回沿固定四位置循环逆时针移动 `steps` 步后的位置。
    ///
    /// 支持任意 `usize` 步数，按四取模，每四步回到原位，不跳过未启用的位置。
    pub const fn counter_clockwise(self, steps: usize) -> Self {
        let steps = steps % Self::CAPACITY;
        Self::ALL[(self.index() + Self::CAPACITY - steps) % Self::CAPACITY]
    }

    /// 返回从当前位置顺时针到达 `other` 的最少步数，范围为 `0..=3`。
    ///
    /// 相同位置的距离为零，距离计算不会跳过未启用的位置。
    pub const fn clockwise_distance_to(self, other: Self) -> usize {
        (other.index() + Self::CAPACITY - self.index()) % Self::CAPACITY
    }

    /// 返回固定四位置循环中相隔两步的对面位置。
    pub const fn opposite(self) -> Self {
        self.clockwise(2)
    }

    /// 返回固定四位置循环中的下一个顺时针位置。
    pub const fn clockwise_next(self) -> Self {
        self.clockwise(1)
    }

    /// 返回固定四位置循环中的下一个逆时针位置。
    pub const fn counter_clockwise_next(self) -> Self {
        self.counter_clockwise(1)
    }
}

/// 牌桌中的逻辑方向。
///
/// 顺时针方向固定为东、南、西、北。本类型不关联玩家、庄家、行动顺序、圈风状态或
/// 任何牌面。
#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Wind {
    /// 东方。
    East = 1,
    /// 南方。
    South = 2,
    /// 西方。
    West = 3,
    /// 北方。
    North = 4,
}

impl Wind {
    /// 固定逻辑方向数量。
    pub const CAPACITY: usize = 4;
    /// 最小合法方向编码，对应东。
    pub const MIN_CODE: u8 = 1;
    /// 最大合法方向编码，对应北。
    pub const MAX_CODE: u8 = 4;

    /// 全部四个逻辑方向，按东、南、西、北排列。
    ///
    /// 该数组表达方向编码的稳定循环顺序，不代表玩家行动或庄家轮转顺序。
    pub const ALL: [Self; Self::CAPACITY] = [Self::East, Self::South, Self::West, Self::North];

    /// 返回 `1..=4` 范围内的一基方向编码，依次对应东、南、西、北。
    pub const fn code(self) -> u8 {
        self as u8
    }

    /// 返回 `0..=3` 范围内的零基紧凑索引，依次对应东、南、西、北。
    pub const fn index(self) -> usize {
        (self.code() - 1) as usize
    }

    /// 从一基方向编码创建逻辑方向。
    ///
    /// `code` 不在 `1..=4` 时返回 `None`。
    pub const fn from_code(code: u8) -> Option<Self> {
        if code < Self::MIN_CODE || code > Self::MAX_CODE {
            return None;
        }
        Some(Self::ALL[(code - 1) as usize])
    }

    /// 从零基紧凑索引创建逻辑方向。
    ///
    /// `index` 不在 `0..=3` 时返回 `None`。
    pub const fn from_index(index: usize) -> Option<Self> {
        if index >= Self::CAPACITY {
            return None;
        }
        Some(Self::ALL[index])
    }

    /// 返回沿东、南、西、北循环顺时针移动 `steps` 步后的方向。
    ///
    /// 支持任意 `usize` 步数，按四取模，每四步回到原方向。
    pub const fn clockwise(self, steps: usize) -> Self {
        // 先取模再与索引相加，使最大 usize 步数也能安全归入固定周期。
        let steps = steps % Self::CAPACITY;
        Self::ALL[(self.index() + steps) % Self::CAPACITY]
    }

    /// 返回沿东、北、西、南循环逆时针移动 `steps` 步后的方向。
    ///
    /// 支持任意 `usize` 步数，按四取模，每四步回到原方向。
    pub const fn counter_clockwise(self, steps: usize) -> Self {
        let steps = steps % Self::CAPACITY;
        Self::ALL[(self.index() + Self::CAPACITY - steps) % Self::CAPACITY]
    }

    /// 返回从当前方向顺时针到达 `other` 的最少步数，范围为 `0..=3`。
    ///
    /// 相同方向的距离为零。
    pub const fn clockwise_distance_to(self, other: Self) -> usize {
        (other.index() + Self::CAPACITY - self.index()) % Self::CAPACITY
    }

    /// 返回固定四方向循环中相隔两步的相反方向。
    pub const fn opposite(self) -> Self {
        self.clockwise(2)
    }

    /// 返回按东、南、西、北循环排列的下一个方向。
    pub const fn clockwise_next(self) -> Self {
        self.clockwise(1)
    }

    /// 返回按东、北、西、南循环排列的下一个方向。
    pub const fn counter_clockwise_next(self) -> Self {
        self.counter_clockwise(1)
    }
}

/// 当前承担东风的逻辑位置。
///
/// 本类型只表达坐风映射的锚点角色，不验证该位置是否被当前规则启用。
/// 默认以编码为 `1` 的位置作为初始东风锚点，不代表已完成定庄或验证该位置参与牌局，
/// 也不隐含任何行动顺序规则。
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct EastSeat(Seat);

/// 使用编码为 `1` 的逻辑位置作为默认东风锚点。
///
/// 此默认值只用于初始坐风映射，不代表已完成定庄或验证该位置参与牌局。
impl Default for EastSeat {
    /// 返回以编码为 `1` 的位置作为东风锚点的映射。
    fn default() -> Self {
        Self(seat(1))
    }
}

impl EastSeat {
    /// 将一个逻辑位置指定为当前东风位置。
    pub const fn new(seat: Seat) -> Self {
        Self(seat)
    }

    /// 返回承担东风的逻辑位置。
    pub const fn seat(self) -> Seat {
        self.0
    }

    /// 返回目标位置当前对应的坐风。
    ///
    /// 本方法只计算固定四位置循环中的相对方向，不读取或推进任何牌局状态。
    pub const fn wind_at(self, seat: Seat) -> Wind {
        // 东风锚点对应零偏移，其余坐风沿同一顺时针逻辑环确定。
        let offset = self.0.clockwise_distance_to(seat);
        Wind::ALL[offset]
    }

    /// 返回当前承担 `wind` 坐风的逻辑位置，是 [`Self::wind_at`] 的逆映射。
    ///
    /// 本方法只计算固定四位置循环中的相对位置，不检查位置是否启用，也不推进牌局状态。
    pub const fn seat_at(self, wind: Wind) -> Seat {
        self.0.clockwise(wind.index())
    }
}

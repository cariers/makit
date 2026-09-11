//! 已完成出牌的内容记录，组合出牌位置与手牌移除结果。

use crate::{HandDiscard, Seat, Tile};

/// 一次已完成出牌的原始内容，不包含响应窗口或行动权限状态。
///
/// 直接组合 [`HandDiscard`]，保留原始牌面与移除前的手牌分区来源，避免重复保存
/// 同一事实。构造时不校验该座位是否参与牌局或有权出牌，这些由上层行动负责。
///
/// 相等性与哈希比较记录内容，不代表相同内容的两条记录是同一次出牌；
/// 对某次弃牌的引用由 [`crate::DiscardId`] 在所属牌河及一手牌作用域内区分。
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Discard {
    /// 实际出牌的逻辑位置。
    seat: Seat,
    /// 从手牌移除的原始牌面及当时的分区来源。
    hand_discard: HandDiscard,
}

impl Discard {
    /// 为成功的手牌出牌结果附加出牌位置，不验证行动权限或记录身份。
    pub const fn new(seat: Seat, hand_discard: HandDiscard) -> Self {
        Self { seat, hand_discard }
    }

    /// 返回记录中的出牌位置，不判断该位置是否被当前规则启用。
    pub const fn seat(&self) -> Seat {
        self.seat
    }

    /// 返回实际打出的原始牌面，不进行通配替换。
    pub const fn tile(&self) -> Tile {
        self.hand_discard.tile()
    }

    /// 判断移除前是否属于摸牌区，不自动推导玩法中的摸切判定。
    ///
    /// 批量出牌时保留各自 [`HandDiscard`] 的来源，不重新解释批次或回合边界。
    pub const fn is_from_draw(&self) -> bool {
        self.hand_discard.is_from_draw()
    }

    /// 返回保存的手牌移除结果副本，保留原始牌面与分区来源。
    pub const fn hand_discard(&self) -> HandDiscard {
        self.hand_discard
    }
}

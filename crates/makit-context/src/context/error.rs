//! 通用组合操作的结构化错误，以及归还加入输入的错误。

use std::fmt;

use makit_core::{DiscardId, HandError, MeldsError, RiverError, Seat, WallError};

/// 通用组合操作未满足参与状态或核心容器结构约束。
///
/// 本类型不表达具体变体的规则拒绝；底层错误保留完整结构并可通过
/// [`std::error::Error::source`] 查询。
#[derive(Clone, Debug, Eq, PartialEq, thiserror::Error)]
pub enum ContextError {
    /// 请求操作的逻辑位置尚未加入。
    #[error("seat {} has not joined", .seat.code())]
    MissingSeat {
        /// 缺席的逻辑位置。
        seat: Seat,
    },
    /// 指定座位的手牌操作失败。
    #[error("hand operation failed for seat {}: {source}", .seat.code())]
    Hand {
        /// 手牌所属座位。
        seat: Seat,
        /// 手牌返回的结构化错误。
        source: HandError,
    },
    /// 当前牌墙不能满足取牌请求。
    #[error("wall operation failed: {source}")]
    Wall {
        /// 牌墙返回的结构化错误。
        source: WallError,
    },
    /// 弃牌追加或留存牌消费失败。
    #[error("river operation failed: {source}")]
    River {
        /// 牌河返回的结构化错误。
        source: RiverError,
    },
    /// 指定座位的固定组集合操作失败。
    #[error("melds operation failed for seat {}: {source}", .seat.code())]
    Melds {
        /// 固定组所属座位。
        seat: Seat,
        /// 集合返回的结构化错误，保留目标组索引及实际类别。
        source: MeldsError,
    },
    /// 一条座位间转移的来源和目标相同。
    #[error("transfer source and destination are both seat {}", .seat.code())]
    SelfTransfer {
        /// 同时作为来源和目标的位置。
        seat: Seat,
    },
    /// 鸣牌请求引用了本家弃牌，无法形成来自他家的鸣取来源。
    #[error("seat {} cannot claim its own discard {}", .seat.code(), .id.value())]
    OwnDiscard {
        /// 提交鸣牌且同时是弃牌来源的座位。
        seat: Seat,
        /// 被引用的本家弃牌编号。
        id: DiscardId,
    },
}

/// 座位加入失败时保留被拒绝的扩展值，不要求扩展可复制或调试。
///
/// 英文错误消息只包含座位；扩展值不参与格式化，也不作为底层错误来源，
/// 因而不要求 `S: Display + Error` 或 `'static`，允许保存借用的数据。
#[derive(thiserror::Error)]
pub enum JoinError<S> {
    /// 目标座位已有状态，不能被本次加入覆盖。
    #[error("seat {} is already occupied", .seat.code())]
    SeatOccupied {
        /// 已占用的位置。
        seat: Seat,
        /// 本次传入且尚未使用的座位扩展。
        variant: S,
    },
}

impl<S> JoinError<S> {
    /// 返回本次加入被拒绝的逻辑位置。
    pub const fn seat(&self) -> Seat {
        match self {
            Self::SeatOccupied { seat, .. } => *seat,
        }
    }

    /// 借用未被上下文接收的扩展值。
    pub const fn variant(&self) -> &S {
        match self {
            Self::SeatOccupied { variant, .. } => variant,
        }
    }

    /// 消耗错误并归还逻辑位置与尚未使用的扩展值。
    pub fn into_parts(self) -> (Seat, S) {
        match self {
            Self::SeatOccupied { seat, variant } => (seat, variant),
        }
    }
}

/// 调试输出保留错误类别和座位，隐藏扩展内容以避免要求 `S: Debug`。
impl<S> fmt::Debug for JoinError<S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SeatOccupied { seat, .. } => f
                .debug_struct("SeatOccupied")
                .field("seat", seat)
                .finish_non_exhaustive(),
        }
    }
}

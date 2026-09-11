//! 通用组合操作的结构化错误，以及归还加入输入的错误。

use std::{error::Error, fmt};

use makit_core::{DiscardId, HandError, MeldsError, RiverError, Seat, WallError};

/// 通用组合操作未满足参与状态或核心容器结构约束。
///
/// 本类型不表达具体变体的规则拒绝；底层错误保留完整结构并可通过 [`Error::source`] 查询。
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ContextError {
    /// 请求操作的逻辑位置尚未加入。
    MissingSeat {
        /// 缺席的逻辑位置。
        seat: Seat,
    },
    /// 指定座位的手牌操作失败。
    Hand {
        /// 手牌所属座位。
        seat: Seat,
        /// 手牌返回的结构化错误。
        source: HandError,
    },
    /// 当前牌墙不能满足取牌请求。
    Wall {
        /// 牌墙返回的结构化错误。
        source: WallError,
    },
    /// 弃牌追加或留存牌消费失败。
    River {
        /// 牌河返回的结构化错误。
        source: RiverError,
    },
    /// 指定座位的固定组集合操作失败。
    Melds {
        /// 固定组所属座位。
        seat: Seat,
        /// 集合返回的结构化错误，保留目标组索引及实际类别。
        source: MeldsError,
    },
    /// 一条座位间转移的来源和目标相同。
    SelfTransfer {
        /// 同时作为来源和目标的位置。
        seat: Seat,
    },
    /// 鸣牌请求引用了本家弃牌，无法形成来自他家的鸣取来源。
    OwnDiscard {
        /// 提交鸣牌且同时是弃牌来源的座位。
        seat: Seat,
        /// 被引用的本家弃牌编号。
        id: DiscardId,
    },
}

/// 将结构化失败原因格式化为英文，不丢弃底层错误信息。
impl fmt::Display for ContextError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingSeat { seat } => write!(f, "seat {} has not joined", seat.code()),
            Self::Hand { seat, source } => write!(
                f,
                "hand operation failed for seat {}: {source}",
                seat.code()
            ),
            Self::Wall { source } => write!(f, "wall operation failed: {source}"),
            Self::River { source } => write!(f, "river operation failed: {source}"),
            Self::Melds { seat, source } => write!(
                f,
                "melds operation failed for seat {}: {source}",
                seat.code()
            ),
            Self::SelfTransfer { seat } => write!(
                f,
                "transfer source and destination are both seat {}",
                seat.code()
            ),
            Self::OwnDiscard { seat, id } => write!(
                f,
                "seat {} cannot claim its own discard {}",
                seat.code(),
                id.value()
            ),
        }
    }
}

/// 将核心容器的原始错误作为来源保留，其他结构错误没有来源。
impl Error for ContextError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Hand { source, .. } => Some(source),
            Self::Wall { source } => Some(source),
            Self::River { source } => Some(source),
            Self::Melds { source, .. } => Some(source),
            _ => None,
        }
    }
}

/// 座位加入失败时保留被拒绝的扩展值，不要求扩展可复制或调试。
pub enum JoinError<S> {
    /// 目标座位已有状态，不能被本次加入覆盖。
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

/// 加入错误只展示英文失败原因，不格式化扩展数据。
impl<S> fmt::Display for JoinError<S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "seat {} is already occupied", self.seat().code())
    }
}

/// 加入错误不包含底层来源，也不要求扩展类型实现 `Error`。
impl<S> Error for JoinError<S> {}

//! 独立于牌面编码的单步通配替换解释。

use crate::{Tile, TileMask};

/// 为一个原始牌面定义单步通配替换规则。
///
/// 通配解释独立于 [`Tile`] 的编码与相等性；展示、计数和计分仍可保留原始牌面。
/// 查询替换结果时必须显式提供规则允许的目标集合，不会默认允许全部编码槽。
/// 替换只解释当前规则一次，不递归查找目标牌面上的其他通配规则。
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum Wildcard {
    /// 可替换为调用方允许集合中的任意牌面。
    Any {
        /// 具有通配能力的原始牌面。
        tile: Tile,
    },
    /// 只能替换为指定牌面，且该目标仍须被调用方允许。
    One {
        /// 具有通配能力的原始牌面。
        tile: Tile,
        /// 单步替换后的目标牌面。
        target: Tile,
    },
}

impl Wildcard {
    /// 返回具有通配能力的原始牌面。
    pub const fn tile(self) -> Tile {
        match self {
            Self::Any { tile } | Self::One { tile, .. } => tile,
        }
    }

    /// 返回固定替换目标；任意通配 [`Self::Any`] 返回 `None`。
    ///
    /// 此方法仅读取配置，不判断该目标是否在当前规则允许的集合中。
    pub const fn target(self) -> Option<Tile> {
        match self {
            Self::Any { .. } => None,
            Self::One { target, .. } => Some(target),
        }
    }

    /// 返回在 `allowed` 限定范围内可替换到的全部牌面。
    ///
    /// [`Self::Any`] 返回 `allowed`；[`Self::One`] 仅在固定目标被允许时返回
    /// 包含该目标的单元素集合，否则返回空集合。`allowed` 应由具体规则提供，
    /// 可随当前计算场景进一步收窄，不会自动包含原始牌面。
    pub const fn targets(self, allowed: TileMask) -> TileMask {
        match self {
            Self::Any { .. } => allowed,
            Self::One { target, .. } => target.to_mask().intersection(allowed),
        }
    }

    /// 返回在 `allowed` 限定范围内是否允许替换为 `target`。
    ///
    /// 查询与 [`Self::targets`] 使用相同的单步替换语义；即使固定目标匹配，
    /// 只要 `target` 不在 `allowed` 中也会返回 `false`。
    pub const fn allows(self, target: Tile, allowed: TileMask) -> bool {
        self.targets(allowed).contains(target)
    }
}

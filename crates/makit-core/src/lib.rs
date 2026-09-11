//! 麻将规则共用的基础牌面、牌面集合、牌面计数、逻辑位置、手牌与已声明固定组类型。
//!
//! [`Tile`] 表示牌面，同一牌面的多个副本共享相同编码。
//! [`TileMask`] 保存牌面成员关系，忽略张数；[`TileCounts`] 保存每种牌面的张数，
//! 每个牌面的计数独立饱和于 `255`，不限制全部牌面的合计张数为 `255`。
//!
//! [`Wildcard`] 在牌面编码之外表达通配替换，不改变原始牌面的相等性或身份。
//! 固定牌和扩展牌使用相同的 [`Tile`] 类型；扩展牌的含义、启用范围以及通配的
//! 合法目标集合均由具体规则定义，本 crate 不预设玩法的完整牌集。
//!
//! [`Seat`] 表示固定四位置循环中的稳定位置，[`Wind`] 表示独立的逻辑方向。
//! [`EastSeat`] 以东风位置为锚点提供坐风与位置的双向映射；这些类型不决定启用座位、
//! 玩家归属、庄家、行动顺序或圈风状态，也不将逻辑方向与牌面绑定。
//! [`SeatMask`] 保存固定四个位置的成员关系；[`SeatMap<T>`] 使用四个稳定的
//! `Option<T>` 槽位保存各位置的可选值，不会因空槽而压缩或重编号座位，
//! 也不决定玩家行动顺序。
//!
//! [`Meld`] 记录已声明固定组及其原始牌面与来源，[`MeldKind`] 标识固定组种类。
//! [`Melds`] 保存已成立固定组的顺序，提供独立吃、碰、杠声明与原位加杠；
//! 集合不预设组数上限，声明权限、响应及玩法合法性由上层管理。
//! [`Hand`] 保存原始牌面的顺序与摸牌分区，已声明固定组独立存储。
//! 手牌索引只表示当前存储位置，不是实体牌身份；具体玩法的手牌张数上限由上层确定。
//!
//! [`Seed`] 保存显式种子字节，不生成随机值或指定洗牌算法。
//! [`Discard`] 组合出牌位置与手牌移除结果，只记录出牌内容；[`DiscardId`] 在一手牌
//! 所属的 [`River`] 实例内引用某次弃牌。牌河保留历史并单独标记当前留存，已被取走的
//! 记录不能再计入牌河持有量。编号作用域与响应权限由上层校验。
//! [`Wall`] 使用固定牌序与头尾游标保存剩余区间，提供双端取牌；所有查询仅统计
//! 当前剩余牌。它不预设洗牌算法、保留区、公开状态或具体玩法的取牌权限。
//! 各容器独立维护结构约束，跨容器转移与行动提交的一致性由上层负责。

mod discard;
mod hand;
mod meld;
mod melds;
mod river;
mod seat;
mod seed;
mod tile;
mod wall;
mod wildcard;

pub use discard::Discard;
pub use hand::{Hand, HandDiscard, HandError};
pub use meld::{
    AddedKongMeld, ChowMeld, ClaimedKongMeld, ConcealedKongMeld, Meld, MeldKind, PongMeld,
};
pub use melds::{Melds, MeldsError};
pub use river::{DiscardId, River, RiverError};
pub use seat::{EastSeat, Seat, SeatMap, SeatMask, Wind};
pub use seed::Seed;
pub use tile::{Tile, TileCounts, TileMask};
pub use wall::{Wall, WallError};
pub use wildcard::Wildcard;

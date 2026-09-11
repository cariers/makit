//! 麻将一手牌从创建、加入到结束的通用上下文与组合操作。
//!
//! [`Context<V>`] 聚合牌墙、牌河及各参与座位的手牌与固定组；[`Variant`] 指定规则、
//! 全局扩展和逐座扩展的类型。规则与种子显式传入，通用容器初始为空，坐风锚点使用
//! [`makit_core::EastSeat`] 的默认值。数据可独立形成，不要求统一初始化或固定阶段。
//!
//! [`Context::join`] 同时建立座位的通用状态与变体扩展；发牌、摸牌、弃牌、换牌和
//! 固定组操作维护跨容器的一致性，返回错误时不会部分修改通用状态。具体玩法的合法性、
//! 响应优先级、多次交互和事件投影由上层负责，操作结果不应直接作为广播消息。
//! 手牌选择直接使用原始 [`makit_core::Tile`]，同牌重复表示请求多张；按原排列优先
//! 匹配旧牌区，不指定单张牌的存储位置，也不执行通配替换。固定组索引与弃牌编号
//! 引用已形成的记录，不能用牌面值代替其来源信息。
//! 固定组集合使用 [`makit_core::Melds`]；吃、碰、明杠分别由 [`Context::chow`]、
//! [`Context::pong`]、[`Context::claimed_kong`] 提交，暗杠与加杠也使用各自独立方法。
//!
//! [`Context`] 与 [`SeatContext`] 通过 [`std::ops::Deref`] 和 [`std::ops::DerefMut`]
//! 访问各自的变体扩展，不暴露核心容器的可变借用。同名成员可通过显式的
//! `variant()` / `variant_mut()` 访问。扩展的直接修改不包含在通用操作的回滚范围内。
//!
//! 计数查询仅覆盖通用区域，并排除已不留存的弃牌历史；变体自有牌区另行统计。
//! [`makit_core::TileCounts`] 是每种牌面最多 `255` 的饱和摘要，精确统计使用 `usize`。
//! 一个上下文对应一手牌；跨手信息迁移、随机算法、种子公开时机及网络身份由上层决定。

mod context;
mod variant;

pub use context::{
    Context, ContextError, DealOutcome, DealRequest, ExchangeSelection, ExchangeTransfer,
    JoinError, SeatContext, WallEnd,
};
pub use variant::Variant;

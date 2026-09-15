//! 洗牌、定庄等启动准备阶段的行动实现。
//!
//! [`Shuffle`](crate::action::setup::Shuffle) 打乱调用方预先装配的剩余牌墙，
//! 并通过 [`Shuffled`](crate::action::setup::Shuffled) 报告完成张数。
//! [`DetermineDealer`](crate::action::setup::DetermineDealer) 按两枚骰子的总点数确定庄家，
//! 并通过 [`DealerDetermined`](crate::action::setup::DealerDetermined) 报告骰子、庄家和最终东风。
//! 当前东风位置尚未加入时，定庄返回
//! [`DetermineDealerError`](crate::action::setup::DetermineDealerError)，不掷骰或修改局数据。
//! 具体玩法的牌库、行动顺序及组装方式由上层决定。

mod dealer;
mod shuffle;

pub use dealer::{DealerDetermined, DetermineDealer, DetermineDealerError};
pub use shuffle::{Shuffle, Shuffled};

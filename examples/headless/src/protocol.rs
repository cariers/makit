//! 演示变体统一使用的输入、事件与准备结果。

use makit::engine::action::setup::{DealerDetermined, Shuffled};
use makit::{EastSeat, Seat, Tile};

/// 请求当前阶段启动对应准备行动的输入。
///
/// 阶段不匹配时不会执行行动；整体启动使用 [`makit::EngineInput::Start`]。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DemoInput {
    /// 请求洗牌阶段启动洗牌。
    RunShuffle,
    /// 请求定庄阶段掷骰并确定庄家。
    DetermineDealer,
}

/// 演示阶段返回的有序事件，保留公共行动原始事件的结构。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DemoEvent {
    /// 洗牌完成，包含参与洗牌的精确张数。
    Shuffled(Shuffled),
    /// 定庄完成，包含骰子、庄家和最终东风。
    DealerDetermined(DealerDetermined),
}

/// 演示准备完成后移入引擎终态的结果。
///
/// 牌序是本地验证使用的剩余牌墙快照，不适合直接广播给在线玩家。
/// 本类型不实现 `Clone`，调用方可通过机器终态只读借用结果。
#[derive(Debug, PartialEq, Eq)]
pub struct DemoOutput {
    /// 定庄实际使用的两枚骰子点数，每枚范围为 `1..=6`。
    pub dice: [u8; 2],
    /// 定庄选出的已加入座位。
    pub dealer: Seat,
    /// 根据规则保留原位置或切换后的最终东风。
    pub east: EastSeat,
    /// 准备完成时，从前端到后端排列的全部剩余牌，保留重复牌面。
    pub wall: Box<[Tile]>,
}

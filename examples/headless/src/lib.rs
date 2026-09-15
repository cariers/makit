//! 可由命令行宿主或测试调用方复用的麻将准备演示。
//!
//! 演示变体使用固定牌集与已加入座位，显式配置种子和是否调整东风。
//! 本库只定义规则、阶段与协议；输入派发及结果展示由宿主负责，不执行外部 I/O。
//! 最终结果包含完整剩余牌序，仅用于本地演示，不作为在线玩家的广播协议。

mod error;
mod phase;
mod protocol;
mod variant;

pub use error::DemoError;
pub use phase::DemoPhase;
pub use protocol::{DemoEvent, DemoInput, DemoOutput};
pub use variant::{DemoRule, DemoVariant};

/// 演示使用的延时初始化机器，保留原生输入派发与状态查询接口。
///
/// 通过 `Engine::<DemoVariant>::new(rule).into_machine()` 创建，转换方法由
/// [`makit::IntoMachineState`] 提供。
pub type DemoMachine = makit::Machine<makit::Engine<DemoVariant>>;

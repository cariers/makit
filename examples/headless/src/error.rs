//! 演示变体对具体行动业务错误的汇总。

use makit::engine::action::setup::DetermineDealerError;

/// 演示已识别输入后可能产生的业务错误。
///
/// 保留具体行动错误以供宿主匹配；不适用的输入直接使用 [`makit::Error::Unhandled`]。
#[derive(Debug, thiserror::Error)]
pub enum DemoError {
    /// 定庄的领域前置条件不满足，保留具体原因与座位。
    #[error("dealer determination failed: {0}")]
    DetermineDealer(#[source] DetermineDealerError),
}

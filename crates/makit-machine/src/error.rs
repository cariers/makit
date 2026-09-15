/// 输入执行失败的通用分类，保留未处理与具体业务错误的区别。
///
/// [`Unhandled`](Self::Unhandled) 不携带业务原因；[`Custom`](Self::Custom) 保存原始业务
/// 错误，并沿用其对外错误描述。不同执行层可以通过 [`map_custom`](Self::map_custom)
/// 汇总业务错误类型，不改变未处理的含义。
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum Error<E> {
    /// 当前状态不处理本次输入。
    #[error("input was not handled")]
    Unhandled,
    /// 输入执行因具体业务条件失败。
    #[error(transparent)]
    Custom(#[from] E),
}

impl<E> Error<E> {
    /// 消费错误并转换其中的业务错误，保留未处理分类。
    ///
    /// 仅遇到 [`Custom`](Self::Custom) 时调用 `map`；
    /// [`Unhandled`](Self::Unhandled) 原样映射到目标错误类型。
    #[must_use]
    pub fn map_custom<F>(self, map: impl FnOnce(E) -> F) -> Error<F> {
        match self {
            Self::Unhandled => Error::Unhandled,
            Self::Custom(error) => Error::Custom(map(error)),
        }
    }
}

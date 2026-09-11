//! 随机过程的显式种子字节，不指定或执行随机算法。

use std::fmt;

/// 固定为 32 字节的显式随机种子值。
///
/// 所有字节组合均合法，包括全零值；字节宽度不代表输入具有相同位数的熵。
/// 本类型不生成种子，不指定随机算法，也不单独保证相同种子产生相同牌墙。
/// 可复现初始化还需要上层固定初始牌序、随机算法、洗牌步骤及相关版本。
///
/// 调试输出始终为 `Seed(..)`；需要读取原始内容时使用显式字节访问方法。
#[repr(transparent)]
#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub struct Seed([u8; 32]);

impl Seed {
    /// 种子值固定占用的字节数。
    pub const BYTE_LEN: usize = 32;

    /// 从完整字节数组构造种子，不变换内容或生成随机值。
    pub const fn from_bytes(bytes: [u8; Self::BYTE_LEN]) -> Self {
        Self(bytes)
    }

    /// 借用全部原始种子字节，供上层显式读取或适配随机算法。
    pub const fn as_bytes(&self) -> &[u8; Self::BYTE_LEN] {
        &self.0
    }

    /// 消耗种子值并返回完整原始字节数组。
    pub const fn into_bytes(self) -> [u8; Self::BYTE_LEN] {
        self.0
    }
}

/// 将完整字节数组无损转换为种子。
impl From<[u8; Seed::BYTE_LEN]> for Seed {
    /// 保留全部输入字节，不生成或派生额外随机值。
    fn from(bytes: [u8; Seed::BYTE_LEN]) -> Self {
        Self::from_bytes(bytes)
    }
}

/// 调试输出不展开种子内容。
impl fmt::Debug for Seed {
    /// 始终输出 `Seed(..)`，包括使用备用调试格式时。
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // 种子可能推导出未公开的牌序，普通状态日志不应隐式展开其内容。
        f.write_str("Seed(..)")
    }
}

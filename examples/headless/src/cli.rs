//! 命令行参数与结构化解析错误，不参与演示阶段的推进。

use std::ffi::OsString;

use makit::Seed;

/// 固定帮助文本，运行时使用英文。
pub(super) const HELP: &str = "Usage: makit-headless [--seed HEX] [--align-east] [--help]

Run the shuffle and dealer preparation demo without a UI or interactive input.

Options:
  --seed HEX    Exactly 64 ASCII hexadecimal digits; defaults to 04 repeated 32 times.
  --align-east  Move east to the selected dealer; disabled by default.
  --help        Show this help and exit.
";

/// 参数解析后选择的宿主命令。
pub(super) enum Command {
    /// 打印帮助，不创建演示机器。
    Help,
    /// 使用显式参数运行演示。
    Run(Options),
}

/// 只影响本次演示构造的命令行参数。
pub(super) struct Options {
    /// 完整的 32 字节随机种子。
    pub(super) seed: Seed,
    /// 是否将东风切换到庄家。
    pub(super) align_east: bool,
}

/// 保留可区分原因的命令行错误。
#[derive(Debug, thiserror::Error)]
pub(super) enum CliError {
    /// 参数不属于公开命令，包括非 Unicode 的操作系统参数。
    #[error("unknown argument: {argument:?}")]
    UnknownArgument { argument: OsString },
    /// 同一选项出现多次。
    #[error("option {option} must not be repeated")]
    DuplicateOption { option: &'static str },
    /// 需要值的选项没有提供值。
    #[error("option {option} requires a value")]
    MissingValue { option: &'static str },
    /// 种子值不是 Unicode 文本，因而不可能是 ASCII 十六进制。
    #[error("seed must contain only ASCII hexadecimal digits")]
    NonUnicodeSeed,
    /// 种子字节长度不等于要求的十六进制长度。
    #[error("seed must contain exactly 64 ASCII hexadecimal digits; received {actual} bytes")]
    SeedLength { actual: usize },
    /// 种子中的字节不是 ASCII 十六进制字符。
    #[error("seed contains a non-hexadecimal byte 0x{byte:02X} at byte offset {index}")]
    SeedDigit { index: usize, byte: u8 },
}

/// 完整解析参数，即使请求帮助，也不吞掉未知或重复参数。
pub(super) fn parse(arguments: impl IntoIterator<Item = OsString>) -> Result<Command, CliError> {
    let mut arguments = arguments.into_iter().peekable();
    let mut seed = None;
    let mut align_east = false;
    let mut help = false;

    while let Some(argument) = arguments.next() {
        match argument.to_str() {
            Some("--seed") => {
                if seed.is_some() {
                    return Err(CliError::DuplicateOption { option: "--seed" });
                }
                // 后续选项不能被当成种子值，缺值时仍报告原选项的错误。
                if arguments
                    .peek()
                    .and_then(|value| value.to_str())
                    .is_some_and(|value| value.starts_with("--"))
                {
                    return Err(CliError::MissingValue { option: "--seed" });
                }
                let value = arguments
                    .next()
                    .ok_or(CliError::MissingValue { option: "--seed" })?;
                let value = value.to_str().ok_or(CliError::NonUnicodeSeed)?;
                seed = Some(parse_seed(value)?);
            }
            Some("--align-east") => {
                if align_east {
                    return Err(CliError::DuplicateOption {
                        option: "--align-east",
                    });
                }
                align_east = true;
            }
            Some("--help") => {
                if help {
                    return Err(CliError::DuplicateOption { option: "--help" });
                }
                help = true;
            }
            _ => return Err(CliError::UnknownArgument { argument }),
        }
    }

    if help {
        Ok(Command::Help)
    } else {
        Ok(Command::Run(Options {
            seed: seed.unwrap_or_else(|| Seed::from_bytes([4; Seed::BYTE_LEN])),
            align_east,
        }))
    }
}

fn parse_seed(value: &str) -> Result<Seed, CliError> {
    if value.len() != Seed::BYTE_LEN * 2 {
        return Err(CliError::SeedLength {
            actual: value.len(),
        });
    }

    let mut bytes = [0; Seed::BYTE_LEN];
    // 长度已保证为偶数；逐字节解码，避免在任意 Unicode 字符内部切片。
    let (pairs, _) = value.as_bytes().as_chunks::<2>();
    for (index, &[high, low]) in pairs.iter().enumerate() {
        let high = hex_digit(high, index * 2)?;
        let low = hex_digit(low, index * 2 + 1)?;
        bytes[index] = (high << 4) | low;
    }
    Ok(Seed::from_bytes(bytes))
}

fn hex_digit(byte: u8, index: usize) -> Result<u8, CliError> {
    match byte {
        b'0'..=b'9' => Ok(byte - b'0'),
        b'a'..=b'f' => Ok(byte - b'a' + 10),
        b'A'..=b'F' => Ok(byte - b'A' + 10),
        _ => Err(CliError::SeedDigit { index, byte }),
    }
}

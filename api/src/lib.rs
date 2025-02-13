use meta::utils::{Arch, OsName};

pub mod meta;
pub const OS: OsName = if cfg!(target_os = "windows") {
    OsName::Windows
} else if cfg!(target_os = "macos") {
    OsName::Osx
} else if cfg!(target_os = "linux") {
    OsName::Linux
} else {
    panic!("unsupported OS")
};

pub const ARCH: Arch = if cfg!(target_arch = "x86") {
    Arch::X86
} else if cfg!(target_arch = "x86_64") {
    Arch::X86_64
} else if cfg!(target_arch = "aarch64") {
    Arch::ARM64
} else {
    panic!("unsupported arch")
};

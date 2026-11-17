//! Clang `@available` helpers for the macOS link.
//!
//! ONNX Runtime's CoreML objects call `__isPlatformVersionAtLeast`. rustc
//! links with `-nodefaultlibs`, which omits `libclang_rt.osx.a`, so the
//! symbol is missing unless we define it.

/// Pack a Darwin version the way dyld's `dyld_build_version_t` does.
#[cfg_attr(not(any(test, target_os = "macos")), allow(dead_code))]
pub(crate) fn dyld_version(major: u32, minor: u32, subminor: u32) -> u32 {
    ((major & 0xffff) << 16) | ((minor & 0xff) << 8) | (subminor & 0xff)
}

#[cfg(target_os = "macos")]
const PLATFORM_MACOS: u32 = 1;

#[cfg(target_os = "macos")]
#[repr(C)]
struct DyldBuildVersion {
    platform: u32,
    version: u32,
}

#[cfg(target_os = "macos")]
extern "C" {
    fn _availability_version_check(count: u32, versions: *const DyldBuildVersion) -> bool;
}

#[cfg(target_os = "macos")]
#[no_mangle]
pub extern "C" fn __isPlatformVersionAtLeast(
    platform: u32,
    major: u32,
    minor: u32,
    subminor: u32,
) -> i32 {
    let versions = [DyldBuildVersion {
        platform,
        version: dyld_version(major, minor, subminor),
    }];
    // `_availability_version_check` is in libSystem on macOS 11 and newer.
    i32::from(unsafe { _availability_version_check(1, versions.as_ptr()) })
}

#[cfg(target_os = "macos")]
#[no_mangle]
pub extern "C" fn __isOSVersionAtLeast(major: i32, minor: i32, subminor: i32) -> i32 {
    __isPlatformVersionAtLeast(PLATFORM_MACOS, major as u32, minor as u32, subminor as u32)
}

/// Keep the helpers in the object file even though no Rust caller uses them.
/// The Mach-O objects in ONNX Runtime are what reference them.
#[cfg(target_os = "macos")]
#[used]
static KEEP_APPLE_VERSION_SYMBOLS: [usize; 2] = [
    __isPlatformVersionAtLeast as usize,
    __isOSVersionAtLeast as usize,
];

#[cfg(test)]
mod tests {
    use super::dyld_version;

    #[test]
    fn packs_a_dyld_os_version() {
        assert_eq!(dyld_version(13, 4, 0), 0x000d_0400);
        assert_eq!(dyld_version(13, 4, 1), 0x000d_0401);
    }
}

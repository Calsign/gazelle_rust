#[cfg(target_os = "linux")]
mod x {
    use some_linux_lib::A;
}

#[cfg(target_os = "windows")]
mod y {
    use some_windows_lib::A;
}

use some_common_lib::B;

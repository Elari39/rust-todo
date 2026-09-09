#[cfg(target_os = "windows")]
pub fn shutting_down() -> bool {
    use windows_sys::Win32::UI::WindowsAndMessaging::{GetSystemMetrics, SM_SHUTTINGDOWN};
    unsafe { GetSystemMetrics(SM_SHUTTINGDOWN) != 0 }
}

#[cfg(not(target_os = "windows"))]
pub fn shutting_down() -> bool {
    false
}

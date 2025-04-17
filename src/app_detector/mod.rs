pub trait AppDetector {
    fn get_active_application(&self) -> String;
}

#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "macos")]
pub use macos::MacOSAppDetector as PlatformAppDetector;

#[cfg(target_os = "windows")]
mod windows;
#[cfg(target_os = "windows")]
pub use windows::WindowsAppDetector as PlatformAppDetector;

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "linux")]
pub use linux::LinuxAppDetector as PlatformAppDetector;

#[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
mod dummy;
#[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
pub use dummy::DummyAppDetector as PlatformAppDetector;

// Dummy implementation for unsupported platforms
#[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
mod dummy {
    use super::AppDetector;

    pub struct DummyAppDetector;

    impl DummyAppDetector {
        pub fn new() -> Self {
            DummyAppDetector
        }
    }

    impl AppDetector for DummyAppDetector {
        fn get_active_application(&self) -> String {
            "Unknown".to_string()
        }
    }
} 
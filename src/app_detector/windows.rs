use super::AppDetector;
use std::time::Duration;
use std::thread;
use std::sync::{Arc, Mutex};
use windows::Win32::Foundation::{HWND, FALSE};
use windows::Win32::UI::WindowsAndMessaging::{GetWindowTextW, GetWindowThreadProcessId, GetForegroundWindow};
use windows::Win32::System::Threading::{OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION};
use windows::Win32::System::ProcessStatus::K32GetModuleFileNameExW;

pub struct WindowsAppDetector {
    active_window: Arc<Mutex<String>>,
    active_process: Arc<Mutex<String>>,
    stop_signal: Arc<Mutex<bool>>,
}

impl WindowsAppDetector {
    pub fn new() -> Self {
        let active_window = Arc::new(Mutex::new(String::new()));
        let active_process = Arc::new(Mutex::new(String::new()));
        let stop_signal = Arc::new(Mutex::new(false));
        
        let detector = WindowsAppDetector {
            active_window: active_window.clone(),
            active_process: active_process.clone(),
            stop_signal: stop_signal.clone(),
        };
        
        // Start the detection thread
        let thread_active_window = active_window.clone();
        let thread_active_process = active_process.clone();
        let thread_stop_signal = stop_signal.clone();
        let detector_clone = detector.clone();
        
        std::thread::spawn(move || {
            detector_clone.run_detection_loop(
                thread_active_window,
                thread_active_process,
                thread_stop_signal,
            );
        });
        
        detector
    }
    
    fn run_detection_loop(
        &self,
        active_window: Arc<Mutex<String>>,
        active_process: Arc<Mutex<String>>,
        stop_signal: Arc<Mutex<bool>>,
    ) {
        while !*stop_signal.lock().unwrap() {
            if let Some((window_title, process_name)) = self.get_active_window_info() {
                *active_window.lock().unwrap() = window_title;
                *active_process.lock().unwrap() = process_name;
            }
            
            thread::sleep(Duration::from_millis(500));
        }
    }
    
    fn get_active_window_info(&self) -> Option<(String, String)> {
        unsafe {
            let hwnd = GetForegroundWindow();
            if hwnd.0 == 0 {
                return None;
            }
            
            // Get window title
            let mut buffer = [0u16; 256];
            let len = GetWindowTextW(hwnd, &mut buffer);
            if len == 0 {
                return None;
            }
            
            let window_title = String::from_utf16_lossy(&buffer[..len as usize]);
            
            // Get process ID
            let mut process_id: u32 = 0;
            GetWindowThreadProcessId(hwnd, Some(&mut process_id));
            
            if process_id == 0 {
                return Some((window_title, String::new()));
            }
            
            // Get process handle
            let process_handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, FALSE, process_id);
            if process_handle.is_invalid() {
                return Some((window_title, String::new()));
            }
            
            // Get process name
            let mut buffer = [0u16; 260];
            let len = K32GetModuleFileNameExW(process_handle, None, &mut buffer);
            
            let process_path = if len > 0 {
                String::from_utf16_lossy(&buffer[..len as usize])
            } else {
                String::new()
            };
            
            // Extract just the filename from the path
            let process_name = if !process_path.is_empty() {
                process_path.split('\\').last().unwrap_or("").to_string()
            } else {
                String::new()
            };
            
            Some((window_title, process_name))
        }
    }
    
    pub fn stop(&self) {
        *self.stop_signal.lock().unwrap() = true;
    }
}

impl Clone for WindowsAppDetector {
    fn clone(&self) -> Self {
        WindowsAppDetector {
            active_window: self.active_window.clone(),
            active_process: self.active_process.clone(),
            stop_signal: self.stop_signal.clone(),
        }
    }
}

impl AppDetector for WindowsAppDetector {
    fn get_active_application(&self) -> String {
        self.active_process.lock().unwrap().clone()
    }
}

impl Drop for WindowsAppDetector {
    fn drop(&mut self) {
        self.stop();
    }
} 
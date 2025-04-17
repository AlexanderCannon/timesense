use super::AppDetector;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::thread;
use std::process::Command;

pub struct LinuxAppDetector {
    active_app: Arc<Mutex<String>>,
    stop_signal: Arc<Mutex<bool>>,
}

impl LinuxAppDetector {
    pub fn new() -> Self {
        let active_app = Arc::new(Mutex::new(String::new()));
        let stop_signal = Arc::new(Mutex::new(false));
        
        let detector = LinuxAppDetector {
            active_app: active_app.clone(),
            stop_signal: stop_signal.clone(),
        };
        
        let thread_active_app = active_app.clone();
        let thread_stop_signal = stop_signal.clone();
        
        std::thread::spawn(move || {
            while !*thread_stop_signal.lock().unwrap() {
                // Try to get the active window using xdotool
                if let Ok(output) = Command::new("xdotool")
                    .args(&["getactivewindow", "getwindowpid"])
                    .output() {
                    
                    if output.status.success() {
                        if let Ok(pid_str) = String::from_utf8(output.stdout) {
                            if let Ok(pid) = pid_str.trim().parse::<u32>() {
                                // Get the process name from the PID
                                if let Ok(output) = Command::new("ps")
                                    .args(&["-p", &pid.to_string(), "-o", "comm="])
                                    .output() {
                                    
                                    if output.status.success() {
                                        if let Ok(process_name) = String::from_utf8(output.stdout) {
                                            let process_name = process_name.trim().to_string();
                                            *thread_active_app.lock().unwrap() = process_name;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                
                thread::sleep(Duration::from_millis(500));
            }
        });
        
        detector
    }
    
    pub fn stop(&self) {
        *self.stop_signal.lock().unwrap() = true;
    }
}

impl AppDetector for LinuxAppDetector {
    fn get_active_application(&self) -> String {
        self.active_app.lock().unwrap().clone()
    }
}

impl Drop for LinuxAppDetector {
    fn drop(&mut self) {
        self.stop();
    }
} 
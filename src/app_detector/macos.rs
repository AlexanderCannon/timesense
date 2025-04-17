use super::AppDetector;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::thread;
use std::process::Command;

pub struct MacOSAppDetector {
    active_app: Arc<Mutex<String>>,
    stop_signal: Arc<Mutex<bool>>,
}

impl MacOSAppDetector {
    pub fn new() -> Self {
        let active_app = Arc::new(Mutex::new(String::new()));
        let stop_signal = Arc::new(Mutex::new(false));
        
        let detector = MacOSAppDetector {
            active_app: active_app.clone(),
            stop_signal: stop_signal.clone(),
        };
        
        let thread_active_app = active_app.clone();
        let thread_stop_signal = stop_signal.clone();
        
        std::thread::spawn(move || {
            while !*thread_stop_signal.lock().unwrap() {
                // Use the 'osascript' command to get the frontmost application
                if let Ok(output) = Command::new("osascript")
                    .args(&["-e", "tell application \"System Events\" to get name of first application process whose frontmost is true"])
                    .output() {
                    
                    if output.status.success() {
                        if let Ok(app_name) = String::from_utf8(output.stdout) {
                            let app_name = app_name.trim().to_string();
                            *thread_active_app.lock().unwrap() = app_name;
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

impl AppDetector for MacOSAppDetector {
    fn get_active_application(&self) -> String {
        self.active_app.lock().unwrap().clone()
    }
}

impl Drop for MacOSAppDetector {
    fn drop(&mut self) {
        self.stop();
    }
} 
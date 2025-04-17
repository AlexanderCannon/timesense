use cocoa::base::id;
use objc::{class, msg_send, sel, sel_impl};
use std::ffi::CStr;

pub struct MacOSAppDetector;

impl MacOSAppDetector {
    pub fn new() -> Self {
        MacOSAppDetector
    }

    pub fn get_active_application(&self) -> String {
        unsafe {
            // Get the shared workspace
            let workspace: id = msg_send![class!(NSWorkspace), sharedWorkspace];
            
            // Get the frontmost application
            let app: id = msg_send![workspace, frontmostApplication];
            
            if app.is_null() {
                return "Unknown".to_string();
            }
            
            // Get the application name
            let app_name: id = msg_send![app, localizedName];
            
            if app_name.is_null() {
                return "Unknown".to_string();
            }
            
            // Convert to Rust string
            let c_str: *const i8 = msg_send![app_name, UTF8String];
            let rust_str = CStr::from_ptr(c_str).to_string_lossy().into_owned();
            
            rust_str
        }
    }
} 
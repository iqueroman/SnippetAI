// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::{Arc, Mutex};
use tauri::Window;
use base64::prelude::*;

#[cfg(target_os = "windows")]
use windows::{
    Win32::Foundation::*,
    Win32::System::DataExchange::*,
    Win32::System::Memory::*,
    Win32::UI::WindowsAndMessaging::*,
};

#[cfg(target_os = "windows")]
const CF_TEXT: u32 = 1;

#[cfg(target_os = "windows")]
const CF_UNICODETEXT: u32 = 13;

#[cfg(target_os = "windows")]
struct ClipboardMonitor {
    window: Option<Arc<Window>>,
}

#[cfg(target_os = "windows")]
impl ClipboardMonitor {
    fn set_window(&mut self, window: Window) {
        self.window = Some(Arc::new(window));
    }

    fn start_monitoring(&self, hwnd: HWND) -> windows::core::Result<()> {
        unsafe {
            // Register this window to receive clipboard update messages
            AddClipboardFormatListener(hwnd)?;
            Ok(())
        }
    }

    fn stop_monitoring(&self, hwnd: HWND) {
        unsafe {
            let _ = RemoveClipboardFormatListener(hwnd);
        }
    }

    fn handle_clipboard_update(&self) {
        println!("Clipboard update detected in Rust backend");
        if let Some(window) = &self.window {
            // Try to read clipboard image first
            match self.read_clipboard_image() {
                Ok(Some(image_data)) => {
                    println!("Successfully read image from clipboard, size: {}", image_data.len());
                    // Emit event with image data
                    let result = window.emit("clipboard-image", image_data);
                    match result {
                        Ok(_) => println!("Successfully emitted clipboard-image event"),
                        Err(e) => println!("Failed to emit clipboard-image event: {}", e),
                    }
                    return; // Don't check for text if we found an image
                }
                Ok(None) => {
                    // No image found, try to read text
                    match self.read_clipboard_text() {
                        Ok(Some(text_data)) => {
                            if !text_data.trim().is_empty() && text_data.len() > 5 {
                                println!("Successfully read text from clipboard, length: {}", text_data.len());
                                // Emit event with text data
                                let result = window.emit("clipboard-text", text_data);
                                match result {
                                    Ok(_) => println!("Successfully emitted clipboard-text event"),
                                    Err(e) => println!("Failed to emit clipboard-text event: {}", e),
                                }
                            } else {
                                println!("Text too short or empty, ignoring");
                                let _ = window.emit("clipboard-updated", ());
                            }
                        }
                        Ok(None) => {
                            println!("No text found in clipboard");
                            let _ = window.emit("clipboard-updated", ());
                        }
                        Err(e) => {
                            println!("Error reading clipboard text: {}", e);
                            let _ = window.emit("clipboard-updated", ());
                        }
                    }
                }
                Err(e) => {
                    println!("Error reading clipboard: {}", e);
                    let _ = window.emit("clipboard-updated", ());
                }
            }
        } else {
            println!("No window available to emit event");
        }
    }

    fn read_clipboard_image(&self) -> windows::core::Result<Option<String>> {
        unsafe {
            if OpenClipboard(HWND::default()).is_err() {
                return Ok(None);
            }

            let mut result = Ok(None);

            // Only look for PNG format which AI APIs can process
            let cf_png = RegisterClipboardFormatA(windows::core::s!("PNG"));
            match GetClipboardData(cf_png) {
                Ok(handle) if !handle.is_invalid() => {
                    if let Ok(data) = self.extract_clipboard_data(handle) {
                        result = Ok(Some(BASE64_STANDARD.encode(data)));
                    }
                }
                _ => {
                    // No PNG data found - Windows Snipping Tool should provide PNG
                    println!("No PNG format found in clipboard");
                }
            }

            let _ = CloseClipboard();
            result
        }
    }

    fn extract_clipboard_data(&self, handle: HANDLE) -> windows::core::Result<Vec<u8>> {
        unsafe {
            let hglobal = HGLOBAL(handle.0);
            let ptr = GlobalLock(hglobal);
            if ptr.is_null() {
                return Err(windows::core::Error::from_win32());
            }

            let size = GlobalSize(hglobal);
            let slice = std::slice::from_raw_parts(ptr as *const u8, size);
            let data = slice.to_vec();

            let _ = GlobalUnlock(hglobal);
            Ok(data)
        }
    }

    fn read_clipboard_text(&self) -> windows::core::Result<Option<String>> {
        unsafe {
            if OpenClipboard(HWND::default()).is_err() {
                return Ok(None);
            }

            let mut result = Ok(None);

            // Try to read Unicode text (CF_UNICODETEXT)
            match GetClipboardData(CF_UNICODETEXT) {
                Ok(handle) if !handle.is_invalid() => {
                    if let Ok(data) = self.extract_clipboard_text_data(handle) {
                        result = Ok(Some(data));
                    }
                }
                _ => {
                    // Fall back to ANSI text (CF_TEXT)
                    match GetClipboardData(CF_TEXT) {
                        Ok(handle) if !handle.is_invalid() => {
                            if let Ok(data) = self.extract_clipboard_ansi_text_data(handle) {
                                result = Ok(Some(data));
                            }
                        }
                        _ => {
                            println!("No text format found in clipboard");
                        }
                    }
                }
            }

            let _ = CloseClipboard();
            result
        }
    }

    fn extract_clipboard_text_data(&self, handle: HANDLE) -> windows::core::Result<String> {
        unsafe {
            let hglobal = HGLOBAL(handle.0);
            let ptr = GlobalLock(hglobal) as *const u16;
            if ptr.is_null() {
                return Err(windows::core::Error::from_win32());
            }

            // Find the length of the null-terminated wide string
            let mut len = 0;
            while *ptr.offset(len) != 0 {
                len += 1;
            }

            // Convert to Rust String
            let slice = std::slice::from_raw_parts(ptr, len as usize);
            let string = String::from_utf16_lossy(slice);

            let _ = GlobalUnlock(hglobal);
            Ok(string)
        }
    }

    fn extract_clipboard_ansi_text_data(&self, handle: HANDLE) -> windows::core::Result<String> {
        unsafe {
            let hglobal = HGLOBAL(handle.0);
            let ptr = GlobalLock(hglobal) as *const u8;
            if ptr.is_null() {
                return Err(windows::core::Error::from_win32());
            }

            // Find the length of the null-terminated string
            let mut len = 0;
            while *ptr.offset(len) != 0 {
                len += 1;
            }

            // Convert to Rust String
            let slice = std::slice::from_raw_parts(ptr, len as usize);
            let string = String::from_utf8_lossy(slice).to_string();

            let _ = GlobalUnlock(hglobal);
            Ok(string)
        }
    }
}

#[cfg(target_os = "windows")]
static CLIPBOARD_MONITOR: Mutex<ClipboardMonitor> = Mutex::new(ClipboardMonitor { window: None });

#[cfg(target_os = "windows")]
static mut ORIGINAL_WINDOW_PROC: Option<isize> = None;

#[cfg(target_os = "windows")]
static mut TARGET_HWND: Option<HWND> = None;

#[tauri::command]
fn start_clipboard_monitoring(window: Window) -> std::result::Result<String, String> {
    #[cfg(target_os = "windows")]
    {
        use std::ffi::c_void;

        let hwnd = window.hwnd().map_err(|e| e.to_string())?;
        let hwnd = HWND(hwnd.0 as *mut c_void);

        // Set up window in clipboard monitor
        {
            let mut monitor = CLIPBOARD_MONITOR.lock().unwrap();
            monitor.set_window(window);
        }

        unsafe {
            // Store the original window procedure
            let original_proc_ptr = GetWindowLongPtrW(hwnd, GWLP_WNDPROC);
            ORIGINAL_WINDOW_PROC = Some(original_proc_ptr);
            TARGET_HWND = Some(hwnd);
            
            // Custom window procedure to handle WM_CLIPBOARDUPDATE
            unsafe extern "system" fn window_proc(
                hwnd: HWND,
                msg: u32,
                wparam: WPARAM,
                lparam: LPARAM,
            ) -> LRESULT {
                const WM_CLIPBOARDUPDATE: u32 = 0x031D;
                
                if msg == WM_CLIPBOARDUPDATE {
                    println!("WM_CLIPBOARDUPDATE message received");
                    // Handle clipboard update
                    if let Ok(monitor) = CLIPBOARD_MONITOR.lock() {
                        monitor.handle_clipboard_update();
                    } else {
                        println!("Failed to acquire clipboard monitor lock");
                    }
                    return LRESULT(0);
                }
                
                // Call original window procedure for other messages
                if let Some(original_proc_ptr) = ORIGINAL_WINDOW_PROC {
                    let original_proc: WNDPROC = Some(std::mem::transmute(original_proc_ptr));
                    CallWindowProcW(original_proc, hwnd, msg, wparam, lparam)
                } else {
                    DefWindowProcW(hwnd, msg, wparam, lparam)
                }
            }

            // Store original procedure in user data and set new procedure
            SetWindowLongPtrW(hwnd, GWLP_USERDATA, original_proc_ptr);
            SetWindowLongPtrW(hwnd, GWLP_WNDPROC, window_proc as isize);
        }

        // Start clipboard monitoring
        let monitor = CLIPBOARD_MONITOR.lock().unwrap();
        monitor.start_monitoring(hwnd)
            .map_err(|e| format!("Failed to start clipboard monitoring: {}", e))?;

        Ok("Clipboard monitoring started".to_string())
    }
    
    #[cfg(not(target_os = "windows"))]
    {
        Err("Clipboard monitoring is only supported on Windows".to_string())
    }
}

#[tauri::command]
fn stop_clipboard_monitoring(window: Window) -> std::result::Result<String, String> {
    #[cfg(target_os = "windows")]
    {
        use std::ffi::c_void;
        
        let hwnd = window.hwnd().map_err(|e| e.to_string())?;
        let hwnd = HWND(hwnd.0 as *mut c_void);

        // Stop clipboard monitoring
        let monitor = CLIPBOARD_MONITOR.lock().unwrap();
        monitor.stop_monitoring(hwnd);

        // Restore original window procedure
        unsafe {
            let original_proc_ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA);
            if original_proc_ptr != 0 {
                SetWindowLongPtrW(hwnd, GWLP_WNDPROC, original_proc_ptr);
                SetWindowLongPtrW(hwnd, GWLP_USERDATA, 0);
            }
        }

        Ok("Clipboard monitoring stopped".to_string())
    }
    
    #[cfg(not(target_os = "windows"))]
    {
        Err("Clipboard monitoring is only supported on Windows".to_string())
    }
}

#[cfg(target_os = "windows")]
fn cleanup_and_exit() {
    // Silent cleanup
    unsafe {
        // Restore original window procedure if we have it
        if let (Some(hwnd), Some(original_proc)) = (TARGET_HWND, ORIGINAL_WINDOW_PROC) {
            let _ = SetWindowLongPtrW(hwnd, GWLP_WNDPROC, original_proc);
            let _ = SetWindowLongPtrW(hwnd, GWLP_USERDATA, 0);
            
            // Remove clipboard format listener
            let _ = RemoveClipboardFormatListener(hwnd);
        }
        
        // Clear static state
        ORIGINAL_WINDOW_PROC = None;
        TARGET_HWND = None;
    }
    
    // Clear clipboard monitor state
    if let Ok(mut monitor) = CLIPBOARD_MONITOR.lock() {
        monitor.window = None;
    }
    
    // Kill any child processes silently
    kill_child_processes();
    
    // Force immediate termination
    std::process::exit(0);
}

#[cfg(target_os = "windows")]
fn kill_child_processes() {
    use std::process::Command;
    use std::os::windows::process::CommandExt;
    
    // Use a more targeted approach - only kill processes that are not the current one
    let current_pid = std::process::id();
    
    // Run tasklist silently (no console window)
    let output = Command::new("tasklist")
        .args(&["/FI", "IMAGENAME eq snippet-ai.exe", "/FO", "CSV"])
        .creation_flags(0x08000000) // CREATE_NO_WINDOW
        .output();
    
    if let Ok(output) = output {
        let output_str = String::from_utf8_lossy(&output.stdout);
        
        // Parse and kill only other instances silently
        for line in output_str.lines().skip(1) { // Skip header
            if line.contains("snippet-ai.exe") {
                // Extract PID (should be in the second column)
                let parts: Vec<&str> = line.split(',').collect();
                if parts.len() >= 2 {
                    let pid_str = parts[1].trim_matches('"');
                    if let Ok(pid) = pid_str.parse::<u32>() {
                        if pid != current_pid {
                            // Kill silently without showing console
                            let _ = Command::new("taskkill")
                                .args(&["/F", "/PID", &pid.to_string()])
                                .creation_flags(0x08000000) // CREATE_NO_WINDOW
                                .output();
                        }
                    }
                }
            }
        }
    }
}

#[cfg(not(target_os = "windows"))]
fn kill_child_processes() {
    // No-op for non-Windows platforms
}

#[cfg(not(target_os = "windows"))]
fn cleanup_and_exit() {
    println!("Performing complete cleanup before exit");
    kill_child_processes();
    std::process::exit(0);
}

fn main() {
    // Set up panic handler to ensure cleanup on panic
    std::panic::set_hook(Box::new(|panic_info| {
        println!("Application panicked: {}", panic_info);
        cleanup_and_exit();
    }));

    // Set up exit handler
    extern "C" fn exit_handler() {
        #[cfg(target_os = "windows")]
        unsafe {
            // Emergency cleanup without going through normal cleanup
            if let (Some(hwnd), Some(original_proc)) = (TARGET_HWND, ORIGINAL_WINDOW_PROC) {
                let _ = RemoveClipboardFormatListener(hwnd);
                let _ = SetWindowLongPtrW(hwnd, GWLP_WNDPROC, original_proc);
            }
        }
    }

    // Register exit handler (Windows-specific approach since libc might not be available)
    #[cfg(target_os = "windows")]
    {
        extern "system" {
            fn atexit(func: unsafe extern "C" fn()) -> i32;
        }
        
        unsafe {
            atexit(exit_handler);
        }

        // Set up CTRL+C handler
        extern "system" fn ctrl_handler(_ctrl_type: u32) -> i32 {
            cleanup_and_exit();
            1 // Return 1 to indicate we handled it
        }

        extern "system" {
            fn SetConsoleCtrlHandler(handler: Option<unsafe extern "system" fn(u32) -> i32>, add: i32) -> i32;
        }

        unsafe {
            SetConsoleCtrlHandler(Some(ctrl_handler), 1);
        }
    }

    let app = tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            start_clipboard_monitoring,
            stop_clipboard_monitoring
        ])
        .on_window_event(|event| match event.event() {
            tauri::WindowEvent::CloseRequested { .. } => {
                // Hide window immediately to prevent flashing
                let window = event.window();
                let _ = window.hide();
                
                // Stop clipboard monitoring first
                let _ = stop_clipboard_monitoring(window.clone());
                
                // Perform complete cleanup and exit
                cleanup_and_exit();
            }
            tauri::WindowEvent::Destroyed => {
                cleanup_and_exit();
            }
            _ => {}
        })
        .build(tauri::generate_context!())
        .expect("error while building tauri application");
    
    app.run(|_app_handle, event| match event {
        tauri::RunEvent::ExitRequested { .. } => {
            cleanup_and_exit();
        }
        tauri::RunEvent::WindowEvent { event, .. } => {
            match event {
                tauri::WindowEvent::CloseRequested { .. } => {
                    cleanup_and_exit();
                }
                _ => {}
            }
        }
        _ => {}
    });
}
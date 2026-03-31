use windows::Win32::UI::WindowsAndMessaging::{GetForegroundWindow, GetWindowTextW};

pub fn get_active_window_title() -> String {
    unsafe {
        let hwnd = GetForegroundWindow();

        let mut buffer = vec![0u16; 512];

        let len = GetWindowTextW(hwnd, &mut buffer);

        String::from_utf16_lossy(&buffer[..len as usize])
    }
}
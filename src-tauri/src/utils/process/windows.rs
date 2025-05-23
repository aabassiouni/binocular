use windows::Win32::System::{
    ProcessStatus::K32GetModuleFileNameExW,
    Threading::{OpenProcess, PROCESS_QUERY_INFORMATION},
};

pub unsafe fn get_process_name(process_id: u32) -> Option<String> {
    let process_handle = OpenProcess(PROCESS_QUERY_INFORMATION, false, process_id);

    if let Ok(handle) = process_handle {
        let mut buffer = [0u16; 260]; // MAX_PATH
        let len = K32GetModuleFileNameExW(handle, None, &mut buffer);

        if len > 0 {
            let path = String::from_utf16_lossy(&buffer[..len as usize]);
            // Extract just the file name from the path
            return Some(path.split('\\').last().unwrap_or("Unknown").to_string());
        }
    }
    None
}

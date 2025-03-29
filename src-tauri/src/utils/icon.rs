use base64::{engine::general_purpose, Engine as _};
use windows::Win32::Foundation::{HWND, LPARAM, WPARAM};
use windows::Win32::Graphics::Gdi::{
    CreateCompatibleBitmap, CreateCompatibleDC, DeleteDC, DeleteObject, GetDC, SelectObject,
};
use windows::Win32::UI::WindowsAndMessaging::{
    GetClassLongPtrW, SendMessageW, GCLP_HICON, GCLP_HICONSM, HICON, ICON_BIG, ICON_SMALL,
    WM_GETICON,
};

pub unsafe fn get_window_icon(hwnd: HWND) -> Option<String> {
    let mut h_icon =
        HICON(SendMessageW(hwnd, WM_GETICON, WPARAM(ICON_SMALL as usize), LPARAM(0)).0);
    if h_icon.0 == 0 {
        h_icon = HICON(SendMessageW(hwnd, WM_GETICON, WPARAM(ICON_BIG as usize), LPARAM(0)).0);
    }

    if h_icon.0 == 0 {
        h_icon = HICON(GetClassLongPtrW(hwnd, GCLP_HICONSM) as isize);
    }

    if h_icon.0 == 0 {
        h_icon = HICON(GetClassLongPtrW(hwnd, GCLP_HICON) as isize);
    }

    if h_icon.0 == 0 {
        return None;
    }

    // Convert icon to bitmap and then to base64
    let icon_size = 16; // Small icon size
    let hdc = GetDC(hwnd);
    let hdc_mem = CreateCompatibleDC(hdc);
    let h_bitmap = CreateCompatibleBitmap(hdc, icon_size, icon_size);
    let old_bitmap = SelectObject(hdc_mem, h_bitmap);

    // Draw icon to bitmap
    let result = windows::Win32::UI::WindowsAndMessaging::DrawIconEx(
        hdc_mem,
        0,
        0,
        h_icon,
        icon_size,
        icon_size,
        0,
        None,
        windows::Win32::UI::WindowsAndMessaging::DI_NORMAL,
    );

    if !result.is_ok() {
        // Clean up and return None if drawing failed
        SelectObject(hdc_mem, old_bitmap);
        if !DeleteObject(h_bitmap).as_bool() {
            println!("Failed to delete object");
        };
        if !DeleteDC(hdc_mem).as_bool() {
            println!("Failed to delete dc");
        };
        return None;
    }

    // Create buffer for bitmap data
    let buffer_size = (icon_size * icon_size * 4) as usize;
    let mut buffer = vec![0u8; buffer_size];

    // Get bitmap data
    let mut bitmap_info = windows::Win32::Graphics::Gdi::BITMAPINFO {
        bmiHeader: windows::Win32::Graphics::Gdi::BITMAPINFOHEADER {
            biSize: std::mem::size_of::<windows::Win32::Graphics::Gdi::BITMAPINFOHEADER>() as u32,
            biWidth: icon_size,
            biHeight: -icon_size, // Negative for top-down
            biPlanes: 1,
            biBitCount: 32,
            biCompression: windows::Win32::Graphics::Gdi::BI_RGB.0,
            biSizeImage: 0,
            biXPelsPerMeter: 0,
            biYPelsPerMeter: 0,
            biClrUsed: 0,
            biClrImportant: 0,
        },
        bmiColors: [windows::Win32::Graphics::Gdi::RGBQUAD::default()],
    };

    let result = windows::Win32::Graphics::Gdi::GetDIBits(
        hdc_mem,
        h_bitmap,
        0,
        icon_size as u32,
        Some(buffer.as_mut_ptr() as *mut std::ffi::c_void),
        &mut bitmap_info as *mut _,
        windows::Win32::Graphics::Gdi::DIB_RGB_COLORS,
    );

    // Clean up
    SelectObject(hdc_mem, old_bitmap);
    let _ = DeleteObject(h_bitmap);
    let _ = DeleteDC(hdc_mem);

    if result == 0 {
        return None;
    }

    // Convert BGRA to RGBA (Windows stores colors as BGRA)
    for pixel in buffer.chunks_exact_mut(4) {
        // Swap Blue and Red channels
        pixel.swap(0, 2);
    }

    // Encode as PNG and convert to base64
    let png_data = match image::RgbaImage::from_raw(icon_size as u32, icon_size as u32, buffer) {
        Some(img) => {
            let mut cursor = std::io::Cursor::new(Vec::new());
            if let Ok(_) = img.write_to(&mut cursor, image::ImageOutputFormat::Png) {
                cursor.into_inner()
            } else {
                return None;
            }
        }
        None => return None,
    };

    // Convert to base64
    let base64_string = general_purpose::STANDARD.encode(png_data);
    Some(format!("data:image/png;base64,{}", base64_string))
}

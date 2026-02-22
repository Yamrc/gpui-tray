use log::debug;
use std::mem;
use std::sync::Arc;
use windows::Win32::Foundation::HICON;
use windows::Win32::UI::WindowsAndMessaging::*;

#[repr(C)]
struct Pixel {
    r: u8,
    g: u8,
    b: u8,
    a: u8,
}

const PIXEL_SIZE: usize = mem::size_of::<Pixel>();

pub struct Icon {
    hicon: Arc<HICON>,
}

impl Icon {
    pub fn from_rgba(rgba: &[u8], width: u32, height: u32) -> Result<Self, &'static str> {
        if rgba.len() != (width * height * 4) as usize {
            return Err("Invalid image size");
        }

        debug!("Creating icon from {}x{} RGBA data", width, height);

        let mut rgba_data = rgba.to_vec();
        let pixel_count = rgba_data.len() / PIXEL_SIZE;
        let mut and_mask = Vec::with_capacity(pixel_count);
        let pixels = unsafe {
            std::slice::from_raw_parts_mut(rgba_data.as_ptr() as *mut Pixel, pixel_count)
        };

        for pixel in pixels {
            and_mask.push(pixel.a.wrapping_sub(u8::MAX));
        }

        unsafe {
            let hicon_result = CreateIcon(
                None,
                width as i32,
                height as i32,
                1,
                (PIXEL_SIZE * 8) as u8,
                and_mask.as_ptr(),
                rgba_data.as_ptr(),
            );

            let hicon = match hicon_result {
                Ok(h) => h,
                Err(_) => return Err("Failed to create icon"),
            };

            if hicon.is_invalid() {
                return Err("Failed to create icon");
            }

            Ok(Self {
                hicon: Arc::new(hicon),
            })
        }
    }

    pub fn as_hicon(&self) -> HICON {
        *self.hicon
    }
}

impl Drop for Icon {
    fn drop(&mut self) {
        if Arc::strong_count(&self.hicon) == 1 {
            debug!("Destroying icon");
            unsafe {
                let _ = DestroyIcon(*self.hicon);
            }
        }
    }
}

impl Clone for Icon {
    fn clone(&self) -> Self {
        Self {
            hicon: self.hicon.clone(),
        }
    }
}

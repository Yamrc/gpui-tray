/// Tray icon types
#[derive(Clone, Debug)]
pub enum TrayIcon {
    /// Icon from raw image data
    Image { format: ImageFormat, data: Vec<u8> },
    /// Icon from name (Linux-specific, uses theme icons)
    Name(String),
}

impl TrayIcon {
    /// Create icon from theme name (Linux only)
    pub fn from_name(name: impl Into<String>) -> Self {
        Self::Name(name.into())
    }

    /// Create icon from image data
    pub fn from_data(format: ImageFormat, data: Vec<u8>) -> Self {
        Self::Image { format, data }
    }
}

/// Image format for tray icons
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ImageFormat {
    Png,
    Jpeg,
    Svg,
    RawRgba,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_icon_from_name() {
        let icon = TrayIcon::from_name("test-icon");
        match icon {
            TrayIcon::Name(name) => assert_eq!(name, "test-icon"),
            _ => panic!("Expected Name variant"),
        }
    }

    #[test]
    fn test_icon_from_data() {
        let data = vec![0, 1, 2, 3];
        let icon = TrayIcon::from_data(ImageFormat::Png, data.clone());
        match icon {
            TrayIcon::Image { format, data: d } => {
                assert_eq!(format, ImageFormat::Png);
                assert_eq!(d, data);
            }
            _ => panic!("Expected Image variant"),
        }
    }
}

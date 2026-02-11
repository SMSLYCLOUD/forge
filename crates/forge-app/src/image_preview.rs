use std::path::Path;
use anyhow::Result;

#[derive(Debug, Clone, PartialEq)]
pub enum ImageFormat {
    Png,
    Jpeg,
    Gif,
    Svg,
    Unknown,
}

#[derive(Debug, Clone)]
pub struct ImageInfo {
    pub format: ImageFormat,
    pub width: u32,
    pub height: u32,
    pub size_bytes: u64,
}

pub fn detect_format(path: &Path) -> ImageFormat {
    if let Some(ext) = path.extension() {
        if let Some(ext_str) = ext.to_str() {
            match ext_str.to_lowercase().as_str() {
                "png" => return ImageFormat::Png,
                "jpg" | "jpeg" => return ImageFormat::Jpeg,
                "gif" => return ImageFormat::Gif,
                "svg" => return ImageFormat::Svg,
                _ => {}
            }
        }
    }
    ImageFormat::Unknown
}

pub fn get_info(path: &Path) -> Result<ImageInfo> {
    let format = detect_format(path);
    let metadata = std::fs::metadata(path)?;
    let size_bytes = metadata.len();

    let mut width = 0;
    let mut height = 0;

    if let Ok(mut file) = std::fs::File::open(path) {
        use std::io::Read;

        // Read enough for PNG header (24 bytes is enough for IHDR)
        let mut buffer = [0u8; 32];
        if file.read(&mut buffer).is_ok() {
            match format {
                ImageFormat::Png => {
                    // PNG signature: 89 50 4E 47 0D 0A 1A 0A
                    if buffer.starts_with(&[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]) {
                         // Width at offset 16, big endian
                         width = u32::from_be_bytes([buffer[16], buffer[17], buffer[18], buffer[19]]);
                         height = u32::from_be_bytes([buffer[20], buffer[21], buffer[22], buffer[23]]);
                    }
                }
                _ => {
                    // Placeholder for other formats
                }
            }
        }
    }

    Ok(ImageInfo {
        format,
        width,
        height,
        size_bytes,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn test_detect_format_from_extension() {
        assert_eq!(detect_format(Path::new("test.png")), ImageFormat::Png);
        assert_eq!(detect_format(Path::new("test.jpg")), ImageFormat::Jpeg);
        assert_eq!(detect_format(Path::new("test.txt")), ImageFormat::Unknown);
    }

    #[test]
    fn test_get_info_png() {
        // Create a dummy PNG file
        let path = Path::new("test_image_preview.png");
        let mut file = std::fs::File::create(path).unwrap();
        // Signature
        file.write_all(&[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]).unwrap();
        // IHDR chunk
        // Length (13 bytes): 00 00 00 0D
        file.write_all(&[0x00, 0x00, 0x00, 0x0D]).unwrap();
        // Type (IHDR): 49 48 44 52
        file.write_all(&[0x49, 0x48, 0x44, 0x52]).unwrap();
        // Width (100): 00 00 00 64
        file.write_all(&[0x00, 0x00, 0x00, 0x64]).unwrap();
        // Height (200): 00 00 00 C8
        file.write_all(&[0x00, 0x00, 0x00, 0xC8]).unwrap();

        let info = get_info(path).unwrap();
        assert_eq!(info.format, ImageFormat::Png);
        assert_eq!(info.width, 100);
        assert_eq!(info.height, 200);

        let _ = std::fs::remove_file(path);
    }
}

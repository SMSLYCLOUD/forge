use image::{RgbaImage, Pixel};

pub trait ScreenshotTarget {
    fn capture_screenshot(&mut self) -> RgbaImage;
}

pub fn capture_frame<T: ScreenshotTarget>(target: &mut T) -> RgbaImage {
    target.capture_screenshot()
}

pub struct DiffResult {
    pub diff_pixels: usize,
    pub total_pixels: usize,
    pub match_percent: f32,
    pub diff_image: RgbaImage,
}

pub fn diff_images(a: &RgbaImage, b: &RgbaImage) -> DiffResult {
    let (w, h) = a.dimensions();
    let (w2, h2) = b.dimensions();
    if w != w2 || h != h2 {
        panic!("Images dimensions do not match: {}x{} vs {}x{}", w, h, w2, h2);
    }

    let mut diff = RgbaImage::new(w, h);
    let mut diff_count = 0;

    for y in 0..h {
        for x in 0..w {
            let p1 = a.get_pixel(x, y);
            let p2 = b.get_pixel(x, y);
            if p1 != p2 {
                diff_count += 1;
                diff.put_pixel(x, y, image::Rgba([255, 0, 0, 255]));
            } else {
                // Dim the matching pixels
                let image::Rgba([r, g, b, a]) = *p1;
                diff.put_pixel(x, y, image::Rgba([r / 4, g / 4, b / 4, a]));
            }
        }
    }

    let total = (w * h) as usize;
    DiffResult {
        diff_pixels: diff_count,
        total_pixels: total,
        match_percent: 100.0 * (1.0 - (diff_count as f32 / total as f32)),
        diff_image: diff,
    }
}

pub fn assert_visual_match(actual: &RgbaImage, expected_path: &str, tolerance: f32) {
    let expected = image::open(expected_path).expect("Failed to open expected image").to_rgba8();
    let result = diff_images(actual, &expected);
    if result.match_percent < tolerance {
        let diff_path = format!("{}_diff.png", expected_path);
        result.diff_image.save(&diff_path).unwrap();
        panic!(
            "Visual match failed! {}% match (threshold {}%). Diff saved to {}",
            result.match_percent, tolerance, diff_path
        );
    }
}

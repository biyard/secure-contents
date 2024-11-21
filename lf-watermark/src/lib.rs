use std::error::Error;

use image::{DynamicImage, GenericImageView, Rgb, RgbImage};
use rustdct::DctPlanner;

type Result<T> = std::result::Result<T, Box<dyn Error>>;

pub fn get_watermark_from_str(words: &str) -> Result<f32> {
    let char_map =
        "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789!@#$%^&*(),.<>/?; ";

    let mut ret = 0.0;

    for i in 0..words.len() {
        let c = words.chars().nth(i).ok_or("invalid index")?;
        let char_idx = char_map
            .find(c)
            .ok_or(format!("Invalid character; {}", c))?;
        ret += char_idx as f32;
    }

    Ok(ret
        * option_env!("WATERMARK_STRENGTH")
            .unwrap_or("0.01")
            .parse::<f32>()?)
}

pub fn embed_watermark_color(image: &DynamicImage, watermark: &str) -> Result<RgbImage> {
    let watermark = get_watermark_from_str(watermark)?;

    let (width, height) = image.dimensions();
    let len = (width * height) as usize;
    let mut cbcr_channel = vec![(0, 0); len];
    let mut y_channel = vec![0.0; len];
    let idx_fn = |x: u32, y: u32| (y * width + x) as usize;
    let normalization_factor = (2.0 / len as f32).sqrt();

    let image = image.to_rgb8();

    for (x, y, pixel) in image.enumerate_pixels() {
        let idx = idx_fn(x, y);

        let (y, u, v) = rgb_to_ycbcr(&pixel);
        cbcr_channel[idx] = (u, v);
        y_channel[idx] = y as f32 + watermark;
    }

    let mut dct_planner: DctPlanner<f32> = DctPlanner::new();
    let dct = dct_planner.plan_dct2(len);
    dct.process_dct2(&mut y_channel);

    for y in y_channel.iter_mut() {
        *y *= normalization_factor;
    }

    let idct = dct_planner.plan_dct3(len);
    idct.process_dct3(&mut y_channel);
    for y in y_channel.iter_mut() {
        *y *= normalization_factor;
    }

    let mut img_buffer = RgbImage::new(width, height);
    for (x, y, pixel) in img_buffer.enumerate_pixels_mut() {
        let index = idx_fn(x, y);
        let y_ch = y_channel[index];
        let (cb, cr) = cbcr_channel[index];

        *pixel = ycbcr_to_rgb(y_ch, cb as f32, cr as f32);
    }

    Ok(img_buffer)
}

fn rgb_to_ycbcr(pixel: &Rgb<u8>) -> (u8, u8, u8) {
    let r = pixel[0] as f64;
    let g = pixel[1] as f64;
    let b = pixel[2] as f64;

    let y = (0.299 * r + 0.587 * g + 0.114 * b).round() as u8;
    let cb = (-0.169 * r - 0.331 * g + 0.5 * b + 128.0).round() as u8;
    let cr = (0.5 * r - 0.419 * g - 0.081 * b + 128.0).round() as u8;

    (y, cb, cr)
}

fn ycbcr_to_rgb(y: f32, cb: f32, cr: f32) -> Rgb<u8> {
    let r = (y + 1.402 * (cr as f32 - 128.0)).round() as u8;
    let g = (y - 0.34414 * (cb as f32 - 128.0) - 0.71414 * (cr as f32 - 128.0)).round() as u8;
    let b = (y + 1.772 * (cb as f32 - 128.0)).round() as u8;

    Rgb([r, g, b])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_watermark_from_str() {
        let words = "Hello, World!";
        let bytes = get_watermark_from_str(words).unwrap();
        assert_eq!(bytes, 5.35);
    }

    #[test]
    fn test_rgb_to_ycbcr() {
        // NOTE: this ycbcr conversion make a little changes to the original rgb value
        for (r, g, b) in vec![
            (255, 255, 255),
            (254, 0, 0),
            (0, 255, 1),
            (0, 0, 254),
            (0, 0, 0),
            (128, 128, 128),
        ] {
            let (y, cb, cr) = rgb_to_ycbcr(&Rgb([r, g, b]));
            let color = ycbcr_to_rgb(y as f32, cb as f32, cr as f32);

            assert_eq!(Rgb([r, g, b]), color, "rgb: {:?} {:?}", (r, g, b), color);
        }
    }

    #[test]
    fn test_watermark() {
        let img = image::open("image.png").unwrap();
        let watermark = "Hello, World!";
        let watermarked_img = embed_watermark_color(&img, watermark);
        assert!(watermarked_img.is_ok(), "Failed to embed watermark");
        assert!(
            watermarked_img.unwrap().save("output.png").is_ok(),
            "Failed to save image"
        )
    }
}

use std::error::Error;

use image::{DynamicImage, GenericImage, GenericImageView, Pixel};
use rustdct::DctPlanner;

type Result<T> = std::result::Result<T, Box<dyn Error>>;

pub fn words_to_bytes(words: &str) -> Result<Vec<u8>> {
    let mut bytes = Vec::new();
    let char_map = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

    for i in 0..words.len() {
        let c = words.chars().nth(i).ok_or("invalid index")?;
        let char_idx = char_map.find(c).ok_or("Invalid character")?;
        bytes.push(char_idx as u8);
    }

    Ok(bytes)
}

pub fn embed_watermark_color(image: &DynamicImage, watermark: &str) -> Result<DynamicImage> {
    // let img = image::open("image.png").unwrap().to_rgb8();

    // // Create the ImageDct object from ImageBuffer
    // let mut image_dct = ImageDct::new(img);

    // // Compute the DCT of the image then compute the inverse DCT on the coefficients
    // image_dct.compute_dct();
    // image_dct.compute_idct();

    // // Reconstruct it back into an RGB ImageBuffer
    // let reconstructed_image = image_dct.reconstructe_image();

    // // Save the reconstructed image into a PNG
    // image::save_buffer(
    //     "./output.png",
    //     &reconstructed_image,
    //     image_dct.width() as u32,
    //     image_dct.height(),
    //     image::ColorType::Rgb8,
    // )
    //     .unwrap();

    // Split image into R, G, B channels
    let watermark_bytes = words_to_bytes(watermark)?;

    let (width, height) = image.dimensions();
    let len = (width * height) as usize;
    let mut r_channel = vec![0.0; len];
    let mut g_channel = vec![0.0; len];
    let mut b_channel = vec![0.0; len];
    let mut a_channel = vec![0.0; len];

    for (x, y, pixel) in image.pixels() {
        let rgba = pixel.to_rgba();
        let idx = (y * width + x) as usize;
        let r = rgba[0];
        let g = rgba[1];
        let b = rgba[2];
        let a = rgba[3];
        r_channel[idx] = r as f64;
        g_channel[idx] = g as f64;
        b_channel[idx] = b as f64;
        a_channel[idx] = a as f64;
    }

    let mut dct_planner = DctPlanner::new();
    let dct = dct_planner.plan_dct2(len);
    dct.process_dct2(&mut g_channel);

    let alpha = 10.0;
    g_channel[0] += alpha * (watermark_bytes[0] as f64);

    dct.process_dct3(&mut g_channel);

    let mut output_image = DynamicImage::new_rgb8(width, height);
    for (x, y, _) in image.pixels() {
        let idx = (y * width + x) as usize;
        let r = r_channel[idx].clamp(0.0, 255.0) as u8;
        let g = g_channel[idx].clamp(0.0, 255.0) as u8;
        let b = b_channel[idx].clamp(0.0, 255.0) as u8;
        let a = a_channel[idx].clamp(0.0, 255.0) as u8;
        output_image.put_pixel(x, y, image::Rgba([r, g, b, a]));
    }

    Ok(output_image)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_watermark() {
        let img = image::open("image.png").unwrap();
        let watermark = "Hello, World!";
        let watermarked_img = embed_watermark_color(&img, watermark).unwrap();
        watermarked_img.save("output.png").unwrap();
    }
}

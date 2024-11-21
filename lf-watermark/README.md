# Low frequency watermark

## Usage this package
- `image` crate will be utilized to open and save image
  - If you don't need to save and load from a file, you don't need to add `image` crate.

``` bash
cargo add lf-watermark image
```

### Example code
- Simple example code to embed `Hello, World!` to an image by DCT.

``` rust
    let img = image::open("image.png").unwrap();
    let watermark = "Hello, World!";
    let watermarked_img = lf_watermark::embed_watermark_color(&img, watermark);
    assert!(watermarked_img.is_ok(), "Failed to embed watermark");
    assert!(
        watermarked_img.unwrap().save("output.png").is_ok(),
        "Failed to save image"
    )
```

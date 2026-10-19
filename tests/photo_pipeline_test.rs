use iris_core::Rect;
use iris_images::{
    crop_image, decode_image, encode_jpeg, encode_png, flip_horizontal, resize_bilinear, rotate_90,
    straighten_image, RgbaImage,
};

#[test]
fn test_image_transforms_pipeline() {
    let img = RgbaImage::create_test_gradient(200, 100);
    assert_eq!(img.width, 200);
    assert_eq!(img.height, 100);

    // 1. Crop
    let crop_rect = Rect::new(20.0, 10.0, 80.0, 50.0);
    let cropped = crop_image(&img, crop_rect).expect("Crop failed");
    assert_eq!(cropped.width, 80);
    assert_eq!(cropped.height, 50);

    // 2. Rotate 90
    let rotated = rotate_90(&cropped, true);
    assert_eq!(rotated.width, 50);
    assert_eq!(rotated.height, 80);

    // 3. Flip Horizontal
    let flipped = flip_horizontal(&rotated);
    assert_eq!(flipped.width, 50);
    assert_eq!(flipped.height, 80);

    // 4. Resize Bilinear
    let resized = resize_bilinear(&flipped, 100, 160);
    assert_eq!(resized.width, 100);
    assert_eq!(resized.height, 160);

    // 5. Straighten
    let straightened = straighten_image(&resized, 5.0);
    assert_eq!(straightened.width, 100);
    assert_eq!(straightened.height, 160);
}

#[test]
fn test_png_and_jpeg_codec_roundtrip() {
    let img = RgbaImage::create_test_gradient(64, 64);

    // PNG Roundtrip
    let png_bytes = encode_png(&img).expect("PNG encode failed");
    assert!(!png_bytes.is_empty());

    let decoded_png = decode_image(&png_bytes).expect("PNG decode failed");
    assert_eq!(decoded_png.width, 64);
    assert_eq!(decoded_png.height, 64);

    // JPEG Roundtrip
    let jpeg_bytes = encode_jpeg(&img, 90).expect("JPEG encode failed");
    assert!(!jpeg_bytes.is_empty());

    let decoded_jpeg = decode_image(&jpeg_bytes).expect("JPEG decode failed");
    assert_eq!(decoded_jpeg.width, 64);
    assert_eq!(decoded_jpeg.height, 64);
}

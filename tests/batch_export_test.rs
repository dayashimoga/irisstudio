use iris_effects::AdjustmentParameters;
use iris_export::{
    BatchPhotoItem, BatchPhotoProcessor, BatchPhotoTask, CollisionStrategy, PhotoExportFormat,
};
use iris_images::{encode_png, RgbaImage};
use std::fs;

#[test]
fn test_batch_photo_export_pipeline() {
    let temp_dir = std::env::temp_dir().join("iris_test_batch_export");
    let dest_dir = temp_dir.join("outputs");
    let _ = fs::remove_dir_all(&temp_dir);
    fs::create_dir_all(&temp_dir).expect("Failed to create temp test dir");
    fs::create_dir_all(&dest_dir).expect("Failed to create dest dir");

    // Create 2 test input PNG images
    let img1 = RgbaImage::create_test_gradient(64, 64);
    let img2 = RgbaImage::create_test_gradient(48, 48);

    let path1 = temp_dir.join("photo_alpha.png");
    let path2 = temp_dir.join("photo_beta.png");

    fs::write(&path1, encode_png(&img1).unwrap()).unwrap();
    fs::write(&path2, encode_png(&img2).unwrap()).unwrap();

    let task = BatchPhotoTask {
        items: vec![
            BatchPhotoItem {
                input_path: path1.to_string_lossy().into_owned(),
                custom_output_name: None,
                adjustments: Some(AdjustmentParameters {
                    exposure: 0.5,
                    contrast: 10.0,
                    ..Default::default()
                }),
                crop: None,
                resize: Some((32, 32)),
                exif_orientation: Some(1),
            },
            BatchPhotoItem {
                input_path: path2.to_string_lossy().into_owned(),
                custom_output_name: None,
                adjustments: None,
                crop: None,
                resize: None,
                exif_orientation: None,
            },
        ],
        destination_dir: dest_dir.to_string_lossy().into_owned(),
        naming_template: "{name}_{preset}_{index}".into(),
        preset_name: "web_optimized".into(),
        format: PhotoExportFormat::Jpeg { quality: 85 },
        collision_strategy: CollisionStrategy::RenameWithIndex,
    };

    let processor = BatchPhotoProcessor::new();
    let mut progress_count = 0;

    let res = processor.process_batch(&task, |_progress| {
        progress_count += 1;
    });

    assert!(res.is_ok());
    assert!(progress_count > 0);

    // Verify output files exist and are not empty
    let expected_file1 = dest_dir.join("photo_alpha_web_optimized_001.jpg");
    let expected_file2 = dest_dir.join("photo_beta_web_optimized_002.jpg");

    assert!(expected_file1.exists());
    assert!(expected_file2.exists());
    assert!(fs::metadata(&expected_file1).unwrap().len() > 0);
    assert!(fs::metadata(&expected_file2).unwrap().len() > 0);

    // Test collision handling: run again with RenameWithIndex
    let res_collision = processor.process_batch(&task, |_| {});
    assert!(res_collision.is_ok());

    // Clean up
    let _ = fs::remove_dir_all(&temp_dir);
}

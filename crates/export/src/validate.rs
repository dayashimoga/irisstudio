use iris_core::{IrisError, IrisResult};
use std::fs::File;
use std::io::Read;
use std::path::Path;

pub fn validate_exported_media(path: impl AsRef<Path>) -> IrisResult<()> {
    let p = path.as_ref();
    if !p.exists() {
        return Err(IrisError::EncodeError(format!(
            "Export validation failed: output file {:?} does not exist",
            p
        )));
    }

    let meta = std::fs::metadata(p)
        .map_err(|e| IrisError::IoError(format!("Failed to read export metadata: {}", e)))?;

    if meta.len() == 0 {
        return Err(IrisError::EncodeError(
            "Export validation failed: output file is empty (0 bytes)".into(),
        ));
    }

    // Read header validation block
    let mut file = File::open(p)
        .map_err(|e| IrisError::IoError(format!("Failed to re-open exported file: {}", e)))?;

    let mut buf = [0u8; 64];
    let n = file
        .read(&mut buf)
        .map_err(|e| IrisError::IoError(e.to_string()))?;

    let header_str = String::from_utf8_lossy(&buf[..n]);
    let is_iris = header_str.contains("IRIS_STUDIO_EXPORT");
    let is_png = buf.starts_with(b"\x89PNG");
    let is_jpeg = buf.starts_with(b"\xFF\xD8\xFF");

    if !is_iris && !is_png && !is_jpeg {
        return Err(IrisError::EncodeError(
            "Export validation failed: container header mismatch or corrupt stream".into(),
        ));
    }

    Ok(())
}

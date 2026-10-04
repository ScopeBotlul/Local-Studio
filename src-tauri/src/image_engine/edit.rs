//! Prepare local edit inputs; no arbitrary destination or original-file writes.
use super::*;
use reference::{ImageMask, ReferencePreview};
fn store(temporary: &Path, data: &[u8], width: u32, height: u32) -> Result<ImageReference> {
    let root = temporary.join("image-edit-inputs");
    fs::create_dir_all(&root).map_err(|_| "image_storage")?;
    let _pins = crate::gallery::directory_guards(&root)?;
    let path = root.join(format!("{}.png", uuid::Uuid::new_v4()));
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&path)
        .map_err(|_| "image_storage")?;
    file.write_all(data)
        .and_then(|_| file.sync_all())
        .map_err(|_| "image_storage")?;
    Ok(ImageReference {
        path: path.to_string_lossy().into(),
        sha256: format!("{:x}", Sha256::digest(data)),
        width,
        height,
        strength: 0.65,
        mask: None,
    })
}
fn prepare(path: &Path, temporary: &Path) -> Result<ReferencePreview> {
    crate::privacy::check(path)?;
    let mut file = crate::gallery::lock_file(path)?;
    if file.metadata().map_err(|_| "image_path")?.len() > 32 * 1024 * 1024 {
        return Err("image_reference_parameters".into());
    }
    let mut data = vec![];
    file.read_to_end(&mut data).map_err(|_| "image_path")?;
    let mut reader = image::ImageReader::new(std::io::Cursor::new(data))
        .with_guessed_format()
        .map_err(|_| "image_reference_parameters")?;
    let mut limits = image::Limits::default();
    limits.max_image_width = Some(8192);
    limits.max_image_height = Some(8192);
    limits.max_alloc = Some(256 * 1024 * 1024);
    reader.limits(limits);
    let image = reader.decode().map_err(|_| "image_reference_parameters")?;
    let scale = (4194304.0 / (image.width() as f64 * image.height() as f64))
        .sqrt()
        .min(1.0);
    let mut width = ((image.width() as f64 * scale / 64.0).round() as u32 * 64).clamp(256, 4096);
    let mut height = ((image.height() as f64 * scale / 64.0).round() as u32 * 64).clamp(256, 4096);
    while width as u64 * height as u64 > 4194304 {
        if width >= height {
            width -= 64;
        } else {
            height -= 64;
        }
    }
    let image = image
        .resize_exact(width, height, image::imageops::FilterType::Lanczos3)
        .to_rgba8();
    let mut encoded = std::io::Cursor::new(Vec::new());
    image::DynamicImage::ImageRgba8(image)
        .write_to(&mut encoded, image::ImageFormat::Png)
        .map_err(|_| "image_storage")?;
    let encoded = encoded.into_inner();
    let reference = store(temporary, &encoded, width, height)?;
    Ok(ReferencePreview {
        reference,
        preview: format!(
            "data:image/png;base64,{}",
            base64::engine::general_purpose::STANDARD.encode(encoded)
        ),
    })
}
fn mask(data: &str, reference: &ImageReference, temporary: &Path) -> Result<ImageMask> {
    reference::validate(reference, reference.width, reference.height)?;
    if !valid_dimensions(reference.width, reference.height) || data.len() > 32 * 1024 * 1024 {
        return Err("image_mask".into());
    }
    reference::bytes(reference, Path::new(&reference.path))?;
    let data = base64::engine::general_purpose::STANDARD
        .decode(data)
        .map_err(|_| "image_mask")?;
    let dimensions = image::ImageReader::new(std::io::Cursor::new(&data))
        .with_guessed_format()
        .map_err(|_| "image_mask")?
        .into_dimensions()
        .map_err(|_| "image_mask")?;
    if dimensions != (reference.width, reference.height) {
        return Err("image_mask".into());
    }
    reference::validate_mask(&data)?;
    let stored = store(temporary, &data, reference.width, reference.height)?;
    Ok(ImageMask {
        path: stored.path,
        sha256: stored.sha256,
        width: stored.width,
        height: stored.height,
    })
}
#[tauri::command]
pub async fn image_edit_source(
    path: String,
    core: tauri::State<'_, Arc<Core>>,
) -> Result<ReferencePreview> {
    let epoch = crate::privacy::epoch();
    let temporary = PathBuf::from(core.storage_paths()?.temporary);
    let result =
        tauri::async_runtime::spawn_blocking(move || prepare(Path::new(&path), &temporary))
            .await
            .map_err(|_| "image_storage")?;
    crate::privacy::finish(epoch, result)
}
#[tauri::command]
pub async fn image_edit_mask(
    data: String,
    reference: ImageReference,
    core: tauri::State<'_, Arc<Core>>,
) -> Result<ImageMask> {
    let epoch = crate::privacy::epoch();
    let temporary = PathBuf::from(core.storage_paths()?.temporary);
    let result = tauri::async_runtime::spawn_blocking(move || mask(&data, &reference, &temporary))
        .await
        .map_err(|_| "image_storage")?;
    crate::privacy::finish(epoch, result)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn edits_use_copies_and_reject_invalid_masks() {
        let t = tempfile::tempdir().unwrap();
        let path = t.path().join("source.jpg");
        image::RgbImage::from_pixel(300, 200, image::Rgb([40, 70, 100]))
            .save(&path)
            .unwrap();
        let original = fs::read(&path).unwrap();
        let source = prepare(&path, t.path()).unwrap();
        assert_eq!(
            (source.reference.width, source.reference.height),
            (320, 256)
        );
        assert_eq!(fs::read(&path).unwrap(), original);
        let mut image = image::RgbaImage::from_pixel(320, 256, image::Rgba([0, 0, 0, 255]));
        image.put_pixel(30, 30, image::Rgba([255, 255, 255, 255]));
        let mut data = std::io::Cursor::new(vec![]);
        image::DynamicImage::ImageRgba8(image)
            .write_to(&mut data, image::ImageFormat::Png)
            .unwrap();
        let data = base64::engine::general_purpose::STANDARD.encode(data.into_inner());
        let result = mask(&data, &source.reference, t.path()).unwrap();
        assert_ne!(result.path, source.reference.path);
        assert!(mask("invalid", &source.reference, t.path()).is_err());
        let wrong = ImageReference {
            width: 512,
            ..source.reference
        };
        assert!(mask(&data, &wrong, t.path()).is_err());
    }
}

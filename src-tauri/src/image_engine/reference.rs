use super::*;
#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ImageReference {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mask: Option<ImageMask>,
    pub path: String,
    pub sha256: String,
    pub width: u32,
    pub height: u32,
    pub strength: f32,
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ImageMask {
    pub path: String,
    pub sha256: String,
    pub width: u32,
    pub height: u32,
}
impl ImageMask {
    pub(crate) fn reference(&self) -> ImageReference {
        ImageReference {
            mask: None,
            path: self.path.clone(),
            sha256: self.sha256.clone(),
            width: self.width,
            height: self.height,
            strength: 1.,
        }
    }
}
impl ImageReference {
    pub(crate) fn inputs(&self) -> Vec<(&'static str, ImageReference)> {
        let mut main = self.clone();
        main.mask = None;
        let mut result = vec![("reference.png", main)];
        if let Some(mask) = &self.mask {
            result.push(("mask.png", mask.reference()));
        }
        result
    }
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReferencePreview {
    pub reference: ImageReference,
    pub preview: String,
}
pub(super) fn validate(reference: &ImageReference, width: u32, height: u32) -> Result<()> {
    if let Some(mask) = &reference.mask {
        validate(&mask.reference(), width, height)?;
    }
    if reference.path.is_empty()
        || reference.path.len() > 32768
        || reference.path.contains('\0')
        || reference.sha256.len() != 64
        || !reference.sha256.bytes().all(|b| b.is_ascii_hexdigit())
        || reference.width != width
        || reference.height != height
        || !reference.strength.is_finite()
        || !(0.05..=1.).contains(&reference.strength)
    {
        return Err("image_reference_parameters".into());
    }
    Ok(())
}
pub(crate) fn bytes(reference: &ImageReference, path: &Path) -> Result<Vec<u8>> {
    let _pins = crate::gallery::directory_guards(path.parent().ok_or("image_path")?)?;
    let data = png_bytes(path, reference.width, reference.height)?;
    let decoder = png::Decoder::new(std::io::Cursor::new(&data));
    let reader = decoder
        .read_info()
        .map_err(|_| "image_reference_parameters")?;
    if reader.info().animation_control.is_some() || reader.info().bit_depth != png::BitDepth::Eight
    {
        return Err("image_reference_parameters".into());
    }
    if format!("{:x}", Sha256::digest(&data)) != reference.sha256 {
        return Err("image_reference_changed".into());
    }
    Ok(data)
}
pub(super) fn prepare(request: &ImageRequest, directory: &Path, resume: bool) -> Result<()> {
    let Some(reference) = &request.reference else {
        return Ok(());
    };
    let _pins = crate::gallery::directory_guards(directory)?;
    let mut inputs = vec![];
    for (name, input) in reference.inputs() {
        let data = bytes(
            &input,
            if resume {
                directory.join(name)
            } else {
                PathBuf::from(&input.path)
            }
            .as_path(),
        )?;
        if name == "mask.png" {
            validate_mask(&data)?;
        }
        inputs.push((name, data));
    }
    if resume {
        return Ok(());
    }
    use std::os::windows::fs::OpenOptionsExt;
    let mut files = vec![];
    let result = (|| {
        for (name, data) in inputs {
            let file = OpenOptions::new()
                .write(true)
                .access_mode(0xc0010000)
                .share_mode(0)
                .create_new(true)
                .open(directory.join(name))
                .map_err(|_| "image_storage")?;
            files.push(file);
            let file = files.last_mut().unwrap();
            file.write_all(&data)
                .and_then(|_| file.sync_all())
                .map_err(|_| "image_storage")?;
        }
        Ok(())
    })();
    if result.is_err() {
        for file in files {
            let _ = crate::gallery::delete_handle(&file);
        }
    }
    result
}
fn validate_mask(data: &[u8]) -> Result<image::RgbaImage> {
    let image = image::load_from_memory_with_format(data, image::ImageFormat::Png)
        .map_err(|_| "image_mask")?
        .to_rgba8();
    if image
        .pixels()
        .any(|p| p[0] != p[1] || p[1] != p[2] || p[3] != 255)
        || !image.pixels().any(|p| p[0] > 0)
    {
        return Err("image_mask".into());
    }
    Ok(image)
}
pub(super) fn run_inputs(
    request: &ImageRequest,
    directory: &Path,
    cmd: &mut Command,
) -> Result<Vec<File>> {
    let mut pins = vec![];
    if let Some(reference) = &request.reference {
        for (name, input) in reference.inputs() {
            let path = directory.join(name);
            pins.push(read_locked(&path)?);
            let data = bytes(&input, &path)?;
            if name == "mask.png" {
                validate_mask(&data)?;
            }
            cmd.arg(if name == "mask.png" {
                "--mask"
            } else {
                "--init-img"
            })
            .arg(path);
        }
        cmd.args(["--strength", &reference.strength.to_string()]);
    }
    Ok(pins)
}
pub(super) fn compose(request: &ImageRequest, directory: &Path, output: &Path) -> Result<()> {
    let Some(reference) = request.reference.as_ref().filter(|r| r.mask.is_some()) else {
        return Ok(());
    };
    let original = bytes(reference, &directory.join("reference.png"))?;
    let mask = reference.mask.as_ref().unwrap();
    let mask = validate_mask(&bytes(&mask.reference(), &directory.join("mask.png"))?)?;
    let rendered = png_bytes(output, request.width, request.height)?;
    use std::{
        io::Seek,
        os::windows::fs::{MetadataExt, OpenOptionsExt},
    };
    let mut target = OpenOptions::new()
        .read(true)
        .write(true)
        .share_mode(0)
        .custom_flags(0x00200000)
        .open(output)
        .map_err(|_| "image_storage")?;
    let meta = target.metadata().map_err(|_| "image_storage")?;
    if !meta.is_file() || meta.file_attributes() & 0x400 != 0 || meta.len() > MAX_IMAGE {
        return Err("image_output".into());
    }
    let mut current = vec![];
    target
        .read_to_end(&mut current)
        .map_err(|_| "image_storage")?;
    if current != rendered {
        return Err("image_output".into());
    }
    let source = image::load_from_memory_with_format(&original, image::ImageFormat::Png)
        .map_err(|_| "image_reference_parameters")?
        .to_rgba8();
    let mut result = image::load_from_memory_with_format(&rendered, image::ImageFormat::Png)
        .map_err(|_| "image_output")?
        .to_rgba8();
    for ((pixel, old), mask) in result.pixels_mut().zip(source.pixels()).zip(mask.pixels()) {
        let weight = mask[0] as u32;
        for channel in 0..4 {
            pixel[channel] =
                ((pixel[channel] as u32 * weight + old[channel] as u32 * (255 - weight) + 127)
                    / 255) as u8;
        }
    }
    let mut encoded = vec![];
    {
        let mut encoder = png::Encoder::new(&mut encoded, request.width, request.height);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        encoder
            .write_header()
            .map_err(|_| "image_output")?
            .write_image_data(result.as_raw())
            .map_err(|_| "image_output")?;
    }
    target.rewind().map_err(|_| "image_storage")?;
    target.set_len(0).map_err(|_| "image_storage")?;
    target
        .write_all(&encoded)
        .and_then(|_| target.sync_all())
        .map_err(|_| "image_storage")?;
    Ok(())
}
#[tauri::command]
pub async fn image_reference(path: String, mask: Option<bool>) -> Result<ReferencePreview> {let privacy_epoch=crate::privacy::epoch();let privacy_result=(async {
    tauri::async_runtime::spawn_blocking(move || {
        let path = PathBuf::from(path);
        let _pins = crate::gallery::directory_guards(path.parent().ok_or("image_path")?)?;
        let file = read_locked(&path)?;
        let _guard = &file;
        if file.metadata().map_err(|_| "image_path")?.len() > MAX_IMAGE {
            return Err("image_reference_parameters".into());
        }
        let mut data = vec![];
        (&file)
            .take(MAX_IMAGE + 1)
            .read_to_end(&mut data)
            .map_err(|_| "image_path")?;
        let decoder = png::Decoder::new(std::io::Cursor::new(&data));
        let reader = decoder
            .read_info()
            .map_err(|_| "image_reference_parameters")?;
        let info = reader.info();
        let (width, height) = (info.width, info.height);
        if info.animation_control.is_some()
            || info.bit_depth != png::BitDepth::Eight
            || ![512, 768, 1024].contains(&width)
            || ![512, 768, 1024].contains(&height)
        {
            return Err("image_reference_parameters".into());
        }
        let path = fs::canonicalize(path).map_err(|_| "image_path")?;
        png_bytes(&path, width, height)?;
        if mask.unwrap_or(false) {
            validate_mask(&data)?;
        }
        Ok(ReferencePreview {
            reference: ImageReference {
                mask: None,
                path: path.to_string_lossy().into(),
                sha256: format!("{:x}", Sha256::digest(&data)),
                width,
                height,
                strength: 0.65,
            },
            preview: format!(
                "data:image/png;base64,{}",
                base64::engine::general_purpose::STANDARD.encode(data)
            ),
        })
    })
    .await
    .map_err(|_| "image_storage")?
}).await;crate::privacy::finish(privacy_epoch,privacy_result)}
#[cfg(test)]
mod tests {
    use super::*;
    fn png_data(pixels: &[u8]) -> Vec<u8> {
        let mut bytes = vec![];
        {
            let mut e = png::Encoder::new(&mut bytes, 512, 512);
            e.set_color(png::ColorType::Rgba);
            e.set_depth(png::BitDepth::Eight);
            e.write_header().unwrap().write_image_data(pixels).unwrap();
        }
        bytes
    }
    #[test]
    fn inpainting_preserves_black_pixels_and_alpha_and_blends_gray_edges() {
        let t = tempfile::tempdir().unwrap();
        let source = png_data(&[20, 40, 60, 128].repeat(512 * 512));
        let mut pixels = vec![];
        for _y in 0..512 {
            for x in 0..512 {
                let value = if x < 128 {
                    255
                } else if x < 256 {
                    128
                } else {
                    0
                };
                pixels.extend([value, value, value, 255]);
            }
        }
        let mask = png_data(&pixels);
        let original = t.path().join("original.png");
        let mask_path = t.path().join("input-mask.png");
        fs::write(&original, &source).unwrap();
        fs::write(&mask_path, &mask).unwrap();
        let reference = ImageReference {
            mask: Some(ImageMask {
                path: mask_path.to_string_lossy().into(),
                sha256: format!("{:x}", Sha256::digest(&mask)),
                width: 512,
                height: 512,
            }),
            path: original.to_string_lossy().into(),
            sha256: format!("{:x}", Sha256::digest(&source)),
            width: 512,
            height: 512,
            strength: 0.65,
        };
        let mut request = super::super::tests::request();
        request.reference = Some(reference.clone());
        let work = t.path().join("job");
        fs::create_dir(&work).unwrap();
        prepare(&request, &work, false).unwrap();
        let output = work.join("image.png");
        fs::write(&output, png_data(&[100, 120, 140, 255].repeat(512 * 512))).unwrap();
        compose(&request, &work, &output).unwrap();
        let result = image::load_from_memory(&fs::read(output).unwrap())
            .unwrap()
            .to_rgba8();
        assert_eq!(result.get_pixel(10, 10).0, [100, 120, 140, 255]);
        assert_eq!(result.get_pixel(200, 10).0, [60, 80, 100, 192]);
        assert_eq!(result.get_pixel(400, 10).0, [20, 40, 60, 128]);
        assert_eq!(fs::read(original).unwrap(), source);
        assert_eq!(fs::read(mask_path).unwrap(), mask);
        assert!(validate_mask(&png_data(&[0, 0, 0, 255].repeat(512 * 512))).is_err());
        assert!(validate_mask(&png_data(&[255, 0, 0, 255].repeat(512 * 512))).is_err());
        assert!(validate_mask(&png_data(&[255, 255, 255, 0].repeat(512 * 512))).is_err());
    }
    #[test]
    fn queued_reference_keeps_exact_bytes_and_rejects_changed_source() {
        let t = tempfile::tempdir().unwrap();
        let source = t.path().join("source.png");
        let mut file = File::create(&source).unwrap();
        {
            let mut e = png::Encoder::new(&mut file, 512, 512);
            e.set_color(png::ColorType::Rgba);
            e.set_depth(png::BitDepth::Eight);
            e.write_header()
                .unwrap()
                .write_image_data(&vec![255; 512 * 512 * 4])
                .unwrap();
        }
        drop(file);
        let original = fs::read(&source).unwrap();
        let reference = ImageReference {
            mask: None,
            path: source.to_string_lossy().into(),
            sha256: format!("{:x}", Sha256::digest(&original)),
            width: 512,
            height: 512,
            strength: 0.5,
        };
        let mut request = super::super::tests::request();
        request.reference = Some(reference.clone());
        let output = t.path().join("job");
        fs::create_dir(&output).unwrap();
        prepare(&request, &output, false).unwrap();
        fs::write(&source, b"changed").unwrap();
        assert_eq!(
            bytes(&reference, &output.join("reference.png")).unwrap(),
            original
        );
        assert!(bytes(&reference, &source).is_err());
        prepare(&request, &output, true).unwrap();
        assert!(validate(&reference, 768, 512).is_err());
    }
}

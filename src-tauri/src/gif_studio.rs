use crate::{comfy::{Comfy, WanModel}, core::Core, gallery, model_library};
use base64::Engine;
use image::{
    codecs::gif::{GifEncoder, Repeat},
    imageops::FilterType,
    Delay, Frame, ImageReader,
};
use serde::Deserialize;
use std::{
    fs::{self, File},
    io::{BufWriter, Cursor},
    path::{Path, PathBuf},
    sync::Arc,
};
use tauri::State;

type Result<T> = std::result::Result<T, String>;

fn source(path: &Path) -> Result<image::DynamicImage> {
    model_library::no_links(path).map_err(|_| "gif_source")?;
    let metadata = fs::metadata(path).map_err(|_| "gif_source")?;
    if !metadata.is_file() || metadata.len() > 32 * 1024 * 1024 {
        return Err("gif_source".into());
    }
    let mut reader = ImageReader::open(path)
        .map_err(|_| "gif_source")?
        .with_guessed_format()
        .map_err(|_| "gif_source")?;
    let mut limits = image::Limits::default();
    limits.max_image_width = Some(4096);
    limits.max_image_height = Some(4096);
    limits.max_alloc = Some(256 * 1024 * 1024);
    reader.limits(limits);
    reader.decode().map_err(|_| "gif_source".into())
}

pub(crate) fn create(
    paths: Vec<String>,
    folder: String,
    name: String,
    delay_ms: u32,
    looped: bool,
    root: PathBuf,
) -> Result<String> {
    if paths.is_empty() || paths.len() > 200 || !(20..=10_000).contains(&delay_ms) {
        return Err("gif_parameters".into());
    }
    let name = if name.to_ascii_lowercase().ends_with(".gif") {
        name
    } else {
        format!("{name}.gif")
    };
    if !gallery::valid_name(&name) {
        return Err("gallery_name".into());
    }
    // `root` has already been canonicalized and link-checked by `gallery::root`.
    // Resolving an empty relative path again is both unnecessary and, on Windows,
    // can turn an already-verbatim path into an invalid double-verbatim path.
    let destination = if folder.is_empty() {
        root.clone()
    } else {
        gallery::resolve(&root, &folder)?
    };
    let _guards = gallery::directory_guards(&destination)?;
    let output = destination.join(name);
    if output.exists() {
        return Err("gallery_exists".into());
    }
    let restricted = paths
        .iter()
        .any(|path| crate::privacy::media(Path::new(path)));
    let first = source(Path::new(&paths[0]))?;
    let (width, height) = (first.width(), first.height());
    if width == 0
        || height == 0
        || width > 2048
        || height > 2048
        || u64::from(width) * u64::from(height) > 4_194_304
    {
        return Err("gif_dimensions".into());
    }
    let temp = destination.join(format!(".local-studio-{}.tmp", uuid::Uuid::new_v4()));
    let result = (|| {
        let file = File::options()
            .write(true)
            .create_new(true)
            .open(&temp)
            .map_err(|_| "gallery_storage")?;
        let mut encoder = GifEncoder::new_with_speed(BufWriter::new(file), 10);
        encoder
            .set_repeat(if looped {
                Repeat::Infinite
            } else {
                Repeat::Finite(0)
            })
            .map_err(|_| "gif_encode")?;
        for (index, path) in paths.iter().enumerate() {
            let image = if index == 0 {
                first.clone()
            } else {
                source(Path::new(path))?
            };
            let rgba = if image.width() == width && image.height() == height {
                image.into_rgba8()
            } else {
                image
                    .resize_exact(width, height, FilterType::Triangle)
                    .into_rgba8()
            };
            encoder
                .encode_frame(Frame::from_parts(
                    rgba,
                    0,
                    0,
                    Delay::from_numer_denom_ms(delay_ms, 1),
                ))
                .map_err(|_| "gif_encode")?;
        }
        drop(encoder);
        gallery::publish(&temp, &output).map_err(|_| "gallery_storage")?;
        if restricted {
            crate::privacy::mark(&output)?;
        }
        Ok(output.to_string_lossy().into())
    })();
    if result.is_err() {
        let _ = fs::remove_file(&temp);
    }
    result
}

#[tauri::command]
pub async fn gif_create(
    paths: Vec<String>,
    folder: String,
    name: String,
    delay_ms: u32,
    looped: bool,
    core: State<'_, Arc<Core>>,
) -> Result<String> {
    let epoch = crate::privacy::epoch();
    let result = (async {
        let root =
            fs::canonicalize(core.storage_paths()?.gallery).map_err(|_| "gallery_missing")?;
        tauri::async_runtime::spawn_blocking(move || {
            create(paths, folder, name, delay_ms, looped, root)
        })
        .await
        .map_err(|_| "gallery_storage")?
    })
    .await;
    crate::privacy::finish(epoch, result)
}

#[tauri::command]
pub async fn gif_model_catalog(comfy: State<'_, Arc<Comfy>>) -> Result<Vec<WanModel>> {
    let comfy = comfy.inner().clone();
    tauri::async_runtime::spawn_blocking(move || comfy.wan_models())
        .await
        .map_err(|_| "comfy_storage")?
}

#[tauri::command]
pub async fn gif_source_preview(path: String) -> Result<String> {
    let epoch = crate::privacy::epoch();
    let result = tauri::async_runtime::spawn_blocking(move || {
        let path = Path::new(&path);
        crate::privacy::check(path)?;
        let preview = source(path)?.thumbnail(1024, 1024);
        let mut bytes = Cursor::new(Vec::new());
        preview.write_to(&mut bytes, image::ImageFormat::Png)
            .map_err(|_| "gif_source")?;
        Ok(format!("data:image/png;base64,{}", base64::engine::general_purpose::STANDARD.encode(bytes.into_inner())))
    }).await.map_err(|_| "gif_source")?;
    crate::privacy::finish(epoch, result)
}

#[derive(Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct GifAiRequest {
    pub source_path: String,
    pub model_path: String,
    pub prompt: String,
    pub negative_prompt: String,
    pub width: u32,
    pub height: u32,
    pub frames: u32,
    pub steps: u32,
    pub guidance: f32,
    pub seed: u32,
    pub folder: String,
    pub name: String,
    pub delay_ms: u32,
    pub looped: bool,
}

#[tauri::command]
pub async fn gif_ai_create(
    request: GifAiRequest,
    core: State<'_, Arc<Core>>,
    comfy: State<'_, Arc<Comfy>>,
) -> Result<String> {
    let epoch = crate::privacy::epoch();
    let result = (async {
        let root = fs::canonicalize(core.storage_paths()?.gallery).map_err(|_| "gallery_missing")?;
        let comfy = comfy.inner().clone();
        let generation = request.clone();
        let frames = tauri::async_runtime::spawn_blocking(move || comfy.wan_image_to_frames(generation))
            .await
            .map_err(|_| "gif_ai")??;
        let temporary = frames.clone();
        let output = tauri::async_runtime::spawn_blocking(move || {
            create(frames, request.folder, request.name, request.delay_ms, request.looped, root)
        })
        .await
        .map_err(|_| "gallery_storage")?;
        for frame in temporary { let _ = fs::remove_file(frame); }
        output
    })
    .await;
    crate::privacy::finish(epoch, result)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rejects_empty_and_unsafe_names() {
        let root = tempfile::tempdir().unwrap();
        assert_eq!(
            create(vec![], "".into(), "x".into(), 100, true, root.path().into()).unwrap_err(),
            "gif_parameters"
        );
        assert_eq!(
            create(
                vec!["missing".into()],
                "".into(),
                "../x".into(),
                100,
                true,
                root.path().into()
            )
            .unwrap_err(),
            "gallery_name"
        );
    }

    #[test]
    fn creates_a_real_two_frame_gif() {
        use image::AnimationDecoder;
        let root = tempfile::tempdir().unwrap();
        let first = root.path().join("one.png");
        let second = root.path().join("two.png");
        let canonical_root = fs::canonicalize(root.path()).unwrap();
        assert!(model_library::no_links(&canonical_root).is_ok());
        assert!(gallery::directory_guards(&canonical_root).is_ok());
        image::DynamicImage::ImageRgba8(image::RgbaImage::from_pixel(
            8,
            8,
            image::Rgba([255, 0, 0, 255]),
        ))
        .save(&first)
        .unwrap();
        image::DynamicImage::ImageRgba8(image::RgbaImage::from_pixel(
            4,
            6,
            image::Rgba([0, 255, 0, 255]),
        ))
        .save(&second)
        .unwrap();
        let output = create(
            vec![
                first.to_string_lossy().into(),
                second.to_string_lossy().into(),
            ],
            String::new(),
            "animation".into(),
            120,
            true,
            canonical_root,
        )
        .unwrap();
        let decoder = image::codecs::gif::GifDecoder::new(std::io::BufReader::new(
            File::open(output).unwrap(),
        ))
        .unwrap();
        assert_eq!(decoder.into_frames().collect_frames().unwrap().len(), 2);
    }
}

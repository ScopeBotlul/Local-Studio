use crate::{
    comfy::{Comfy, WanModel},
    core::Core,
    gallery,
    image_engine::{ImageEngine, NativeVideoRequest},
    model_library,
};
use base64::Engine;
use image::{
    codecs::gif::{GifEncoder, Repeat},
    imageops::FilterType,
    Delay, Frame, ImageReader,
};
use serde::{Deserialize, Serialize};
use std::{
    fs::{self, File},
    io::{BufWriter, Cursor},
    path::{Path, PathBuf},
    sync::Arc,
    time::UNIX_EPOCH,
};
use tauri::State;

type Result<T> = std::result::Result<T, String>;

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GifPending {
    id: String,
    bytes: u64,
    created_at: u64,
}

fn pending_root(temporary: &Path) -> Result<PathBuf> {
    let root = temporary.join("gif-results");
    fs::create_dir_all(&root).map_err(|_| "gif_storage")?;
    model_library::no_links(&root).map_err(|_| "gif_storage")?;
    fs::canonicalize(root).map_err(|_| "gif_storage".into())
}

fn pending_path(root: &Path, id: &str) -> Result<PathBuf> {
    let uuid = uuid::Uuid::parse_str(id).map_err(|_| "gif_missing")?;
    if uuid.to_string() != id { return Err("gif_missing".into()); }
    let path = root.join(format!("{id}.gif"));
    model_library::no_links(&path).map_err(|_| "gif_missing")?;
    Ok(path)
}

fn pending_info(path: &Path, id: String) -> Result<GifPending> {
    let metadata = fs::metadata(path).map_err(|_| "gif_missing")?;
    if !metadata.is_file() { return Err("gif_missing".into()); }
    let created_at = metadata.modified().ok().and_then(|time| time.duration_since(UNIX_EPOCH).ok())
        .map_or(0, |duration| duration.as_millis().min(u128::from(u64::MAX)) as u64);
    Ok(GifPending { id, bytes: metadata.len(), created_at })
}

fn create_pending(paths: Vec<String>, delay_ms: u32, looped: bool, root: PathBuf) -> Result<GifPending> {
    let id = uuid::Uuid::new_v4().to_string();
    let path = create(paths, String::new(), id.clone(), delay_ms, looped, root)?;
    pending_info(Path::new(&path), id)
}

fn list_pending(root: &Path) -> Result<Vec<GifPending>> {
    let mut found = Vec::new();
    for entry in fs::read_dir(root).map_err(|_| "gif_storage")?.flatten().take(1000) {
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("gif") { continue; }
        let Some(id) = path.file_stem().and_then(|stem| stem.to_str()) else { continue };
        if pending_path(root, id).is_ok() {
            if let Ok(info) = pending_info(&path, id.to_owned()) { found.push(info); }
        }
    }
    found.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    found.truncate(100);
    Ok(found)
}

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

fn save_pending(root: &Path, gallery_root: &Path, id: &str, folder: &str) -> Result<String> {
    let source = pending_path(root, id)?;
    let mut input = gallery::lock_file(&source).map_err(|_| "gif_missing")?;
    let destination = if folder.is_empty() { gallery_root.to_path_buf() } else { gallery::resolve(gallery_root, folder)? };
    let _guards = gallery::directory_guards(&destination)?;
    let output = destination.join(format!("Local-Studio-{id}.gif"));
    if output.exists() { return Err("gallery_exists".into()); }
    let staging = destination.join(format!(".local-studio-{}.tmp", uuid::Uuid::new_v4()));
    let result = (|| {
        let mut file = File::options().write(true).create_new(true).open(&staging).map_err(|_| "gallery_storage")?;
        std::io::copy(&mut input, &mut file).map_err(|_| "gallery_storage")?;
        file.sync_all().map_err(|_| "gallery_storage")?;
        drop(file);
        drop(input);
        gallery::publish(&staging, &output).map_err(|_| "gallery_storage")?;
        let _ = fs::remove_file(&source);
        Ok(output.to_string_lossy().into_owned())
    })();
    if result.is_err() { let _ = fs::remove_file(&staging); }
    result
}

#[tauri::command]
pub async fn gif_create(
    paths: Vec<String>,
    delay_ms: u32,
    looped: bool,
    core: State<'_, Arc<Core>>,
) -> Result<GifPending> {
    let epoch = crate::privacy::epoch();
    let result = (async {
        let root = pending_root(Path::new(&core.storage_paths()?.temporary))?;
        tauri::async_runtime::spawn_blocking(move || {
            create_pending(paths, delay_ms, looped, root)
        })
        .await
        .map_err(|_| "gallery_storage")?
    })
    .await;
    crate::privacy::finish(epoch, result)
}

#[tauri::command]
pub async fn gif_pending_list(core: State<'_, Arc<Core>>) -> Result<Vec<GifPending>> {
    let root = pending_root(Path::new(&core.storage_paths()?.temporary))?;
    tauri::async_runtime::spawn_blocking(move || list_pending(&root)).await.map_err(|_| "gif_storage")?
}

#[tauri::command]
pub async fn gif_pending_preview(id: String, core: State<'_, Arc<Core>>) -> Result<String> {
    let root = pending_root(Path::new(&core.storage_paths()?.temporary))?;
    tauri::async_runtime::spawn_blocking(move || {
        let path = pending_path(&root, &id)?;
        let metadata = fs::metadata(&path).map_err(|_| "gif_missing")?;
        if !metadata.is_file() || metadata.len() > 64 * 1024 * 1024 { return Err("gif_preview_large".into()); }
        let bytes = fs::read(path).map_err(|_| "gif_missing")?;
        Ok(format!("data:image/gif;base64,{}", base64::engine::general_purpose::STANDARD.encode(bytes)))
    }).await.map_err(|_| "gif_storage")?
}

#[tauri::command]
pub async fn gif_pending_save(id: String, folder: String, core: State<'_, Arc<Core>>) -> Result<String> {
    let root = pending_root(Path::new(&core.storage_paths()?.temporary))?;
    let gallery_root = gallery::root(&core)?;
    tauri::async_runtime::spawn_blocking(move || save_pending(&root, &gallery_root, &id, &folder))
        .await.map_err(|_| "gif_storage")?
}

#[tauri::command]
pub async fn gif_pending_discard(id: String, core: State<'_, Arc<Core>>) -> Result<()> {
    let root = pending_root(Path::new(&core.storage_paths()?.temporary))?;
    tauri::async_runtime::spawn_blocking(move || {
        let path = pending_path(&root, &id)?;
        fs::remove_file(path).map_err(|_| "gif_missing".into())
    }).await.map_err(|_| "gif_storage")?
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
    pub delay_ms: u32,
    pub looped: bool,
}

#[derive(Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct GifVulkanRequest {
    pub source_path: String,
    pub model_path: String,
    pub encoder_path: String,
    pub vae_path: String,
    pub prompt: String,
    pub negative_prompt: String,
    pub width: u32,
    pub height: u32,
    pub frames: u32,
    pub steps: u32,
    pub guidance: f32,
    pub seed: u32,
    pub delay_ms: u32,
    pub looped: bool,
}

#[tauri::command]
pub async fn gif_ai_create(
    request: GifAiRequest,
    core: State<'_, Arc<Core>>,
    comfy: State<'_, Arc<Comfy>>,
) -> Result<GifPending> {
    let epoch = crate::privacy::epoch();
    let result = (async {
        let root = pending_root(Path::new(&core.storage_paths()?.temporary))?;
        let comfy = comfy.inner().clone();
        let generation = request.clone();
        let frames = tauri::async_runtime::spawn_blocking(move || comfy.wan_image_to_frames(generation))
            .await
            .map_err(|_| "gif_ai")??;
        let temporary = frames.clone();
        let output = tauri::async_runtime::spawn_blocking(move || {
            create_pending(frames, request.delay_ms, request.looped, root)
        }).await.map_err(|_| "gif_storage");
        for frame in temporary { let _ = fs::remove_file(frame); }
        output?
    })
    .await;
    crate::privacy::finish(epoch, result)
}

#[tauri::command]
pub async fn gif_vulkan_create(
    request: GifVulkanRequest,
    core: State<'_, Arc<Core>>,
    images: State<'_, Arc<ImageEngine>>,
) -> Result<GifPending> {
    let epoch = crate::privacy::epoch();
    let result = (async {
        if !(20..=10_000).contains(&request.delay_ms) {
            return Err("gif_parameters".into());
        }
        let temporary = PathBuf::from(core.storage_paths()?.temporary);
        let pending = pending_root(&temporary)?;
        let jobs_root = temporary.join("gif-vulkan-jobs");
        fs::create_dir_all(&jobs_root).map_err(|_| "gif_storage")?;
        model_library::no_links(&jobs_root).map_err(|_| "gif_storage")?;
        let jobs_root = fs::canonicalize(jobs_root).map_err(|_| "gif_storage")?;
        let directory = jobs_root.join(uuid::Uuid::new_v4().to_string());
        fs::create_dir(&directory).map_err(|_| "gif_storage")?;
        let directory = fs::canonicalize(directory).map_err(|_| "gif_storage")?;
        if !directory.starts_with(&jobs_root) || directory == jobs_root {
            return Err("gif_storage".into());
        }
        let engine = images.inner().clone();
        let delay_ms = request.delay_ms;
        let looped = request.looped;
        let generation = NativeVideoRequest {
            source_path: request.source_path,
            model_path: request.model_path,
            encoder_path: request.encoder_path,
            vae_path: request.vae_path,
            prompt: request.prompt,
            negative_prompt: request.negative_prompt,
            width: request.width,
            height: request.height,
            frames: request.frames,
            steps: request.steps,
            guidance: request.guidance,
            seed: request.seed,
            fps: (1_000 / delay_ms.max(20)).clamp(1, 50),
        };
        let work = directory.clone();
        let frames = tauri::async_runtime::spawn_blocking(move || {
            engine.generate_vulkan_video(generation, &work)
        })
        .await
        .map_err(|_| "gif_vulkan_execution".to_string());
        let output = match frames {
            Ok(Ok(frames)) => {
                let root = pending.clone();
                match tauri::async_runtime::spawn_blocking(move || {
                    create_pending(frames, delay_ms, looped, root)
                })
                .await
                .map_err(|_| "gif_storage".to_string())
                {
                    Ok(result) => result,
                    Err(error) => Err(error),
                }
            }
            Ok(Err(error)) | Err(error) => Err(error),
        };
        if directory.starts_with(&jobs_root) && directory != jobs_root {
            let _ = fs::remove_dir_all(&directory);
        }
        output
    })
    .await;
    crate::privacy::finish(epoch, result)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn gif_stays_temporary_until_saved_and_uses_unique_ids() {
        let temporary = tempfile::tempdir().unwrap();
        let gallery_dir = tempfile::tempdir().unwrap();
        let gallery_root = fs::canonicalize(gallery_dir.path()).unwrap();
        let root = pending_root(temporary.path()).unwrap();
        let image = temporary.path().join("source.png");
        image::DynamicImage::ImageRgba8(image::RgbaImage::from_pixel(8, 8, image::Rgba([9, 8, 7, 255]))).save(&image).unwrap();
        let source = image.to_string_lossy().into_owned();
        let first = create_pending(vec![source.clone()], 100, true, root.clone()).unwrap();
        let second = create_pending(vec![source], 100, true, root.clone()).unwrap();
        assert_ne!(first.id, second.id);
        assert_eq!(list_pending(&root).unwrap().len(), 2);
        assert_eq!(fs::read_dir(&gallery_root).unwrap().count(), 0);
        let first_saved = save_pending(&root, &gallery_root, &first.id, "").unwrap();
        let second_saved = save_pending(&root, &gallery_root, &second.id, "").unwrap();
        assert_ne!(first_saved, second_saved);
        assert!(Path::new(&first_saved).is_file());
        assert!(Path::new(&second_saved).is_file());
        assert_eq!(list_pending(&root).unwrap().len(), 0);
    }
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

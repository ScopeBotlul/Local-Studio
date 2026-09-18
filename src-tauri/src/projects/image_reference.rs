use super::*;
impl Projects {
    pub(super) fn ensure_reference_asset(&self) -> Result<()> {
        let Some(mut p) = self.snapshot()? else {
            return Ok(());
        };
        let Some(reference) = p.request.as_ref().and_then(|r| r.reference.as_ref()) else {
            return Ok(());
        };
        for (_, reference) in reference.inputs() {
            if p.assets
                .iter()
                .any(|a| a.sha256 == reference.sha256 && a.kind == "image")
            {
                continue;
            }
            let source = Path::new(&reference.path);
            let _parents = gallery::directory_guards(source.parent().ok_or("project_path")?)?;
            let _file = gallery::lock_file(source)?;
            crate::image_engine::reference::bytes(&reference, source)?;
            let next = self.add_to(&p.id, vec![reference.path.clone()])?;
            if !next
                .assets
                .iter()
                .any(|a| a.sha256 == reference.sha256 && a.kind == "image")
            {
                return Err("project_hash".into());
            }
            p = next;
        }
        Ok(())
    }
}
pub(super) fn restore_reference(p: &mut Project) -> Result<()> {
    let Some(reference) = p.request.as_ref().and_then(|r| r.reference.as_ref()) else {
        return Ok(());
    };
    let reference = reference.clone();
    for (name, input) in reference.inputs() {
        let asset = p
            .assets
            .iter()
            .find(|a| a.sha256 == input.sha256 && a.archive_name == input.path && a.kind == "image")
            .ok_or("project_manifest")?;
        let path = owned(p, asset)?;
        crate::image_engine::reference::bytes(&input, &path)?;
        let reference = p.request.as_mut().unwrap().reference.as_mut().unwrap();
        if name == "mask.png" {
            reference.mask.as_mut().unwrap().path = path.to_string_lossy().into();
        } else {
            reference.path = path.to_string_lossy().into();
        }
    }
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn image_reference_is_embedded_and_relinked_after_source_is_removed() {
        let t = tempfile::tempdir().unwrap();
        let source = t.path().join("reference.png");
        let mut bytes = vec![];
        {
            let mut encoder = png::Encoder::new(&mut bytes, 512, 512);
            encoder.set_color(png::ColorType::Rgb);
            encoder.set_depth(png::BitDepth::Eight);
            encoder
                .write_header()
                .unwrap()
                .write_image_data(&vec![70; 512 * 512 * 3])
                .unwrap();
        }
        fs::write(&source, &bytes).unwrap();
        let p = Projects::new(t.path()).unwrap();
        let mut r = ImageRequest {
            vae_on_cpu: false,
            reference: None,
            model_path: String::new(),
            prompt: "test".into(),
            negative_prompt: String::new(),
            width: 512,
            height: 512,
            steps: 3,
            guidance: 2.,
            seed: 1,
            sampler: "euler".into(),
        };
        r.model_path.clear();
        r.reference = Some(crate::image_engine::ImageReference {
            mask: None,
            path: source.to_string_lossy().into(),
            sha256: format!("{:x}", Sha256::digest(&bytes)),
            width: 512,
            height: 512,
            strength: 0.6,
        });
        p.new_project(t.path(), "Reference".into(), Some(r), false)
            .unwrap();
        let target = t.path().join("reference.localstudio");
        let saved = p.save(&target).unwrap();
        assert_eq!(saved.assets.len(), 1);
        fs::remove_file(source).unwrap();
        let opened = p.open(&target, t.path(), true).unwrap();
        let reference = opened.request.unwrap().reference.unwrap();
        assert_eq!(fs::read(reference.path).unwrap(), bytes);
        let mut file = File::open(&target).unwrap();
        let mut archive = ZipArchive::new(&mut file).unwrap();
        let manifest = read_manifest(&mut archive).unwrap();
        assert_eq!(manifest.version, 4);
        assert!(manifest
            .request
            .unwrap()
            .reference
            .unwrap()
            .path
            .starts_with("media/"));
    }
}

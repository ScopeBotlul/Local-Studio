//! Passive library descriptions. These never grant permission to execute a model.
use super::*;
use std::{collections::HashMap, sync::OnceLock};

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Requirement { pub name: String, pub embedded: bool }
#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelProfile {
    pub purpose: String, pub role: String, pub family: Option<String>,
    #[serde(default)] pub base_family: Option<String>,
    pub packaging: String, pub evidence: String, pub support: String,
    pub requirements: Vec<Requirement>,
}
impl Default for ModelProfile {
    fn default() -> Self { Self { purpose: "unknown".into(), role: "unknown".into(), family: None, base_family: None,
        packaging: "unknown".into(), evidence: "unknown".into(), support: "unknown".into(), requirements: vec![] } }
}
impl ModelProfile {
    fn identified(purpose: &str, role: &str, family: &str, evidence: &str) -> Self {
        Self { purpose: purpose.into(), role: role.into(), family: Some(family.into()), evidence: evidence.into(),
            support: if matches!(role,"extension"|"component") {"dependency"} else {"unsupported"}.into(), ..Self::default() }
    }
}
static CACHE: OnceLock<Mutex<HashMap<String, ModelProfile>>> = OnceLock::new();

fn header(path: &Path, budget: &mut u64) -> Option<Value> {
    let mut file = safe_file(path).ok()?;
    let mut length = [0;8]; file.read_exact(&mut length).ok()?;
    let length = u64::from_le_bytes(length);
    if !(2..=MAX_HEADER).contains(&length) || length > *budget || length+8 > file.metadata().ok()?.len() { return None; }
    *budget -= length;
    let mut bytes = vec![0;length as usize]; file.read_exact(&mut bytes).ok()?;
    serde_json::from_slice(&bytes).ok()
}

fn from_tensors(value: &Value) -> Option<ModelProfile> {
    let object = value.as_object()?;
    let has = |prefix: &str| object.keys().any(|key| key.starts_with(prefix));
    let shape = |key: &str, wanted: &[u64]| value[key]["shape"] == serde_json::json!(wanted);
    // Extension signals precede backbone signals: LoRAs name the layers they adapt.
    if value["__metadata__"]["ss_network_module"].is_string() || object.keys().any(|k| k.contains("lora_") || k.contains(".lora_A.") || k.contains(".lora_B.") || k.contains(".lora_a.") || k.contains(".lora_b.")) {
        let mut profile=ModelProfile::identified("unknown","extension","LoRA","structure");
        let base=["ss_base_model_version","modelspec.architecture"].into_iter()
            .filter_map(|key|value["__metadata__"][key].as_str())
            .find_map(|name|architecture(name.strip_suffix("/lora").unwrap_or(name)));
        if let Some(base)=base{profile.purpose=base.purpose;profile.base_family=base.family;}
        return Some(profile);
    }
    if has("control_model.") || has("controlnet_cond_embedding.") || has("controlnet_blocks.") {
        return Some(ModelProfile::identified("image","extension","ControlNet","structure"));
    }
    if has("ip_adapter.") { return Some(ModelProfile::identified("image","extension","IP-Adapter","structure")); }
    let sdxl = shape("conditioner.embedders.1.model.token_embedding.weight", &[49408,1280])
        || value["model.diffusion_model.label_emb.0.0.weight"]["shape"][1].as_u64() == Some(2816)
        || value["add_embedding.linear_1.weight"]["shape"][1].as_u64() == Some(2816);
    if sdxl {
        let mut profile = ModelProfile::identified("image","model","SDXL","structure");
        profile.requirements = [
            ("UNet",shape("model.diffusion_model.input_blocks.0.0.weight", &[320,4,3,3]) || shape("conv_in.weight", &[320,4,3,3])),
            ("CLIP-L",shape("conditioner.embedders.0.transformer.text_model.embeddings.token_embedding.weight", &[49408,768])),
            ("CLIP-G",shape("conditioner.embedders.1.model.token_embedding.weight", &[49408,1280])),
            ("VAE",shape("first_stage_model.encoder.conv_in.weight", &[128,3,3,3]) && shape("first_stage_model.decoder.conv_out.weight", &[3,128,3,3])),
        ].into_iter().map(|(name,embedded)| Requirement{name:name.into(),embedded}).collect();
        profile.packaging = if profile.requirements.iter().all(|r|r.embedded) {"checkpoint"} else {"backbone"}.into();
        profile.support = if profile.packaging == "checkpoint" {"preflight"} else {"missing"}.into();
        return Some(profile);
    }
    if has("model.diffusion_model.") && has("cond_stage_model.") {
        let mut p=ModelProfile::identified("image","model","Stable Diffusion","structure");
        p.packaging=if has("first_stage_model.encoder.")&&has("first_stage_model.decoder."){ "checkpoint" }else{ "backbone" }.into();return Some(p);
    }
    if has("double_blocks.") && has("single_blocks.") && has("img_in.") {
        let mut p=ModelProfile::identified("image","model","FLUX","structure");p.packaging="backbone".into();return Some(p);
    }
    // An autoencoder's encoder/decoder pair is not an image generation backbone.
    if (has("encoder.") && has("decoder.") && (has("quant_conv.") || has("post_quant_conv.")))
        || (has("first_stage_model.encoder.") && has("first_stage_model.decoder.") && !has("model.diffusion_model.")) {
        return Some(ModelProfile::identified("image","component","VAE","structure"));
    }
    if has("text_model.embeddings.") && has("text_model.encoder.") && !has("vision_model.") {
        return Some(ModelProfile::identified("language","component","CLIP text encoder","structure"));
    }
    if has("encoder.block.") && has("shared.") && !has("decoder.block.") {
        return Some(ModelProfile::identified("language","component","T5 text encoder","structure"));
    }
    if has("model.embed_tokens.") && has("model.layers.") && has("lm_head.") {
        return Some(ModelProfile::identified("language","model","Transformer","structure"));
    }
    if has("model.encoder.conv1.") && has("model.decoder.embed_tokens.") {
        return Some(ModelProfile::identified("audio","model","Whisper","structure"));
    }
    None
}

fn architecture(value: &str) -> Option<ModelProfile> {
    let arch=value.to_ascii_lowercase();
    let (purpose,role,family)=match arch.as_str() {
        "sdxl"|"sdxl_base_v1-0"|"stable-diffusion-xl-v1-base"|"stablediffusionxlpipeline" => ("image","model","SDXL"),
        "sd_v1"|"stable-diffusion-v1"|"stablediffusionpipeline" => ("image","model","Stable Diffusion 1.x"),
        "flux"|"fluxpipeline"|"fluxtransformer2dmodel" => ("image","model","FLUX"),
        "z_image"|"zimagepipeline"|"zimagetransformer2dmodel" => ("image","model","Z-Image"),
        "wan"|"wanpipeline"|"wantransformer3dmodel" => ("video","model","Wan"),
        "cogvideoxpipeline"|"cogvideoxtransformer3dmodel" => ("video","model","CogVideoX"),
        "hunyuanvideopipeline"|"hunyuanvideotransformer3dmodel" => ("video","model","Hunyuan Video"),
        "clip"|"cliptextmodel"|"cliptextmodelwithprojection"|"t5encodermodel"|"umt5encodermodel" => ("language","component","Text encoder"),
        "autoencoderkl"|"autoencodertiny"|"autoencoderklwan" => ("image","component","VAE"),
        "controlnetmodel"|"fluxcontrolnetmodel" => ("image","extension","ControlNet"),
        "whisper"|"whisperforconditionalgeneration" => ("audio","model","Whisper"),
        "musicgen"|"musicgenforconditionalgeneration"|"stableaudiopipeline" => ("audio","model","Audio / music"),
        "llama"|"qwen2"|"qwen3"|"qwen2moe"|"qwen3moe"|"mistral"|"gemma"|"gemma2"|"gemma3"|"phi2"|"phi3"|"phi4"|"gpt2"|"gptneox" => ("language","model",value),
        _=>return None,
    };
    Some(ModelProfile::identified(purpose,role,family,"metadata"))
}

// Bounded GGUF metadata reader. Never assumes that every .gguf is a language model.
// Specification: https://github.com/ggml-org/ggml/blob/master/docs/gguf.md
struct Reader<'a> { bytes: &'a [u8], at: usize }
impl<'a> Reader<'a> {
    fn take(&mut self,n:usize)->Option<&'a [u8]>{let end=self.at.checked_add(n)?;let part=self.bytes.get(self.at..end)?;self.at=end;Some(part)}
    fn u32(&mut self)->Option<u32>{Some(u32::from_le_bytes(self.take(4)?.try_into().ok()?))}
    fn u64(&mut self)->Option<u64>{Some(u64::from_le_bytes(self.take(8)?.try_into().ok()?))}
    fn string(&mut self)->Option<&'a str>{let n=usize::try_from(self.u64()?).ok()?;std::str::from_utf8(self.take(n)?).ok()}
    fn skip(&mut self,kind:u32,depth:u8)->Option<()> {
        match kind {0|1|7=>{self.take(1)?;},2|3=>{self.take(2)?;},4|5|6=>{self.take(4)?;},10|11|12=>{self.take(8)?;},8=>{self.string()?;},9=>{
            if depth>=2{return None;}let element=self.u32()?;let n=self.u64()?;if n>100_000{return None;}
            for _ in 0..n{self.skip(element,depth+1)?;}
        },_=>return None}Some(())
    }
}
fn gguf(path:&Path,budget:&mut u64)->Option<ModelProfile>{
    let file=safe_file(path).ok()?;let cap=file.metadata().ok()?.len().min(1024*1024);
    if cap>*budget{return None;}*budget-=cap;
    let mut bytes=Vec::new();file.take(cap).read_to_end(&mut bytes).ok()?;
    let mut r=Reader{bytes:&bytes,at:0};if r.take(4)?!=b"GGUF"||![2,3].contains(&r.u32()?){return None;}
    r.u64()?;let count=r.u64()?;if count>100_000{return None;}
    for _ in 0..count {let key=r.string()?;let kind=r.u32()?;if key=="general.architecture"&&kind==8{return architecture(r.string()?);}r.skip(kind,0)?;}
    None
}

fn hint(path:&Path)->ModelProfile{
    let name=path.file_stem().unwrap_or_default().to_string_lossy().to_lowercase();
    let parts:Vec<_>=path.components().filter_map(|c|if let Component::Normal(v)=c{Some(v.to_string_lossy().to_lowercase())}else{None}).collect();
    let folder=parts.windows(2).find(|p|p[0]=="models").map(|p|p[1].as_str()).unwrap_or("");
    let (purpose,role,family)=if matches!(folder,"loras"|"lora") {("unknown","extension","LoRA")}
        else if folder=="controlnet" {("image","extension","ControlNet")}
        else if folder=="ipadapter" {("image","extension","IP-Adapter")}
        else if folder=="embeddings" {("unknown","extension","Embedding")}
        else if folder=="vae_approx"||["taesd","taesdxl","taesd3","taef1","taef2","taesana"].iter().any(|p|name==format!("{p}_encoder")||name==format!("{p}_decoder")){("image","component","Tiny autoencoder")}
        else if folder=="vae"||name=="ae"||name.ends_with("_vae")||name.ends_with("-vae"){("image","component","VAE")}
        else if matches!(folder,"text_encoders"|"clip")||matches!(name.as_str(),"clip_l"|"clip_g")||name.starts_with("umt5_")||name.starts_with("t5xxl"){("language","component","Text encoder")}
        else if folder=="clip_vision"||name.starts_with("mmproj-"){("image","component","Vision encoder")}
        else if name.starts_with("wan2.")||name.starts_with("wan_2."){("video","model","Wan")}
        else if name.starts_with("flux-")||name.starts_with("flux1")||name.starts_with("flux2"){("image","model","FLUX")}
        else if name.starts_with("z_image")||name.starts_with("z-image"){("image","model","Z-Image")}
        else if name.contains("sdxl"){("image","model","SDXL")}
        else{return ModelProfile::default();};
    let mut profile=ModelProfile::identified(purpose,role,family,"location");
    if role=="model"{profile.support="unknown".into();}profile
}

pub(super) fn describe(entry:&LocalModel,budget:&mut u64)->ModelProfile{
    let path=Path::new(&entry.path);
    let mut fallback=hint(path);
    if entry.discovery==Discovery::Excluded{return ModelProfile::default();}
    if matches!(entry.status.as_str(),"missing"|"unavailable"|"invalid"|"incomplete") {fallback.support=entry.status.clone();return fallback;}
    let config_path=path.parent().unwrap_or(path).join("config.json");
    let config_stamp=stamp(&config_path);
    let fingerprint=format!("{}:{}:{}:{:?}",entry.path,entry.status,serde_json::to_string(&entry.files).unwrap_or_default(),(config_stamp.size,config_stamp.modified));
    let key=format!("{:x}",Sha256::digest(fingerprint.as_bytes()));
    let cache=CACHE.get_or_init(Default::default);
    if let Some(profile)=cache.lock().ok().and_then(|c|c.get(&key).cloned()){return profile;}
    // Large libraries are described incrementally; never cache an exhausted budget.
    if *budget<MAX_HEADER{return fallback;}
    *budget=budget.saturating_sub(config_stamp.size.unwrap_or(0).min(1024*1024));
    let config=read_json(&config_path,1024*1024).ok();
    let config_profile=config.as_ref().and_then(|v|[v["_class_name"].as_str(),v["model_type"].as_str()].into_iter().flatten().find_map(architecture));
    let tensor_header=if entry.format=="safetensors" {header(path,budget)} else {None};
    let structural=tensor_header.as_ref().and_then(from_tensors).or_else(|| {
        if entry.format!="safetensors-index"{return None;}
        let index=read_json(path,1024*1024).ok()?;
        *budget=budget.saturating_sub(1024*1024);
        from_tensors(index.get("weight_map")?)
    });
    let metadata=tensor_header.as_ref().and_then(|v|v["__metadata__"]["modelspec.architecture"].as_str()).and_then(architecture);
    let mut result=structural.or(metadata).or_else(||if entry.format=="gguf"{gguf(path,budget)}else{None}).or(config_profile.clone()).unwrap_or_else(||fallback.clone());
    if result.family.as_deref()==Some("Transformer") {
        if let Some(config)=config_profile.filter(|p|p.purpose==result.purpose&&p.role==result.role){result.family=config.family;}
    }
    // A generator family in config.json does not turn a text encoder into a generator.
    if fallback.role=="component"&&result.role=="model"&&result.packaging!="checkpoint" {result.role="component".into();result.support="dependency".into();result.requirements.clear();}
    if result.role=="unknown"&&entry.kind=="component" {result.role="component".into();result.support="dependency".into();}
    if result.support=="preflight" {
        if *budget < MAX_HEADER*2 {return fallback;}
        *budget-=MAX_HEADER*2;
        if entry.status!="checked" || !crate::image_engine::model_parts(path).is_ok_and(|(_,missing)|missing.is_empty()) {result.support="recheck".into();}
    }
    if entry.format.ends_with("-index") {result.packaging="sharded".into();}
    if let Ok(mut cache)=cache.lock(){if cache.len()>=10_000{cache.clear();}cache.insert(key,result.clone());}
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    #[test]
    fn sdxl_checkpoint_and_missing_components_are_distinguished() {
        let mut h=json!({
            "model.diffusion_model.input_blocks.0.0.weight":{"shape":[320,4,3,3]},
            "conditioner.embedders.0.transformer.text_model.embeddings.token_embedding.weight":{"shape":[49408,768]},
            "conditioner.embedders.1.model.token_embedding.weight":{"shape":[49408,1280]},
            "first_stage_model.encoder.conv_in.weight":{"shape":[128,3,3,3]},
            "first_stage_model.decoder.conv_out.weight":{"shape":[3,128,3,3]}
        });
        let full=from_tensors(&h).unwrap();assert_eq!(full.packaging,"checkpoint");assert_eq!(full.support,"preflight");
        h.as_object_mut().unwrap().remove("first_stage_model.decoder.conv_out.weight");
        let partial=from_tensors(&h).unwrap();assert_eq!(partial.support,"missing");
        assert_eq!(partial.requirements.iter().filter(|r|!r.embedded).map(|r|r.name.as_str()).collect::<Vec<_>>(),vec!["VAE"]);
    }
    #[test]
    fn extensions_and_encoders_do_not_become_generators() {
        let lora=from_tensors(&json!({"__metadata__":{"ss_base_model_version":"sdxl_base_v1-0"},"lora_unet_a.weight":{"shape":[2,2]}})).unwrap();
        assert_eq!(lora.role,"extension");assert_eq!(lora.base_family.as_deref(),Some("SDXL"));
        let clip=from_tensors(&json!({"text_model.embeddings.token_embedding.weight":{},"text_model.encoder.layers.0.weight":{}})).unwrap();assert_eq!(clip.role,"component");
        let control=from_tensors(&json!({"control_model.input_blocks.0.weight":{}})).unwrap();assert_eq!(control.family.as_deref(),Some("ControlNet"));
        assert!(from_tensors(&json!({"arbitrary.weight":{}})).is_none());
    }
    #[test]
    fn gguf_metadata_is_bounded_and_not_assumed_to_be_language() {
        let dir=tempfile::tempdir().unwrap();let path=dir.path().join("renamed.gguf");
        for (arch,purpose,role) in [("qwen2","language","model"),("flux","image","model"),("clip","language","component")] {
            let mut b=b"GGUF".to_vec();b.extend(3u32.to_le_bytes());b.extend(1u64.to_le_bytes());b.extend(1u64.to_le_bytes());
            b.extend(20u64.to_le_bytes());b.extend(b"general.architecture");b.extend(8u32.to_le_bytes());b.extend((arch.len()as u64).to_le_bytes());b.extend(arch.as_bytes());
            fs::write(&path,&b).unwrap();let p=gguf(&path,&mut (1024*1024)).unwrap();assert_eq!(p.purpose,purpose);assert_eq!(p.role,role);
            fs::write(&path,&b[..b.len()-1]).unwrap();assert!(gguf(&path,&mut (1024*1024)).is_none());
        }
        let mut reader=Reader{bytes:&[0;8],at:0};assert!(reader.take(usize::MAX).is_none());assert!(reader.skip(100,0).is_none());
    }
}

use super::*;
use serde_json::Value;
// Admission uses authoritative local provenance, never a caller-supplied adult flag.
// Workers already admitted continue independently of this IPC boundary.
fn paths(value:&Value,key_name:&str,gallery_root:Option<&Path>)->Result<()> {
    match value {
        Value::Object(o)=>for(k,v)in o{paths(v,k,gallery_root)?;},
        Value::Array(a)=>for v in a{paths(v,key_name,gallery_root)?;},
        Value::String(v) if !v.is_empty()=>{
            if ["path","modelPath","source","sources","folder","destination"].contains(&key_name){
                let p=Path::new(v);if p.is_absolute(){check(p)?;}else if let Some(root)=gallery_root{check(&root.join(v))?;}
            }
        },_=>{}
    }Ok(())
}
pub fn guard(invoke:&tauri::ipc::Invoke)->Result<()> {
    let cmd=invoke.message.command();
    if invoke.message.webview_ref().label()!="main" {return Err("privacy_window".into());}
    if !locked() {return Ok(());}
    if cmd.starts_with("privacy_")||matches!(cmd,"image_probe"|"image_cancel"|"video_cancel"|"transcription_cancel"|"media_cancel"|"assistant_cancel"|"assistant_unload"|"mark_clean_exit"){return Ok(());}
    let app=invoke.message.webview_ref().app_handle();
    let protected=app.state::<Arc<crate::projects::Projects>>().privacy_restricted()?;
    if protected&&(cmd.starts_with("project_")||cmd.starts_with("canvas_")||cmd.starts_with("video_")||cmd.starts_with("media_")||cmd=="transcription_start")&&!matches!(cmd,"project_history"|"project_snapshot"|"project_close"|"project_take_open"|"project_ack_open"|"project_recent"|"project_forget_recent"){return Err("privacy_locked".into());}
    if has_protected()&&matches!(cmd,"project_restore_point"|"preference_save"|"get_logs"|"storage_cleanup_preview"|"storage_cleanup_apply"|"storage_cleanup_auto"|"caption_read"|"caption_write"|"gallery_trash_list"|"gallery_trash_detail"|"gallery_trash_action"){return Err("privacy_locked".into());}
    let ai=app.state::<Arc<crate::ai::AiEngine>>();
    if matches!(cmd,"assistant_send"|"assistant_clear")&&ai.privacy_chat()?{return Err("privacy_locked".into());}
    if let tauri::ipc::InvokeBody::Json(payload)=invoke.message.payload(){
        if matches!(cmd,"assistant_load"|"transcription_start"){if let Some(id)=payload.get("modelId").and_then(Value::as_str){if ai.privacy_model(id)?{return Err("privacy_locked".into());}}}

        let root=if cmd.starts_with("gallery_")||cmd=="editor_preview"||cmd=="editor_export"||cmd=="project_add_gallery"||cmd=="project_add_edit"{Some(crate::gallery::root(&app.state::<Arc<crate::core::Core>>())?)}else{None};
        // Workspace persistence merges hidden model preferences itself.
        if cmd!="image_workspace_save"{paths(payload,"",root.as_deref())?;}
        if cmd.starts_with("image_")||cmd=="project_add_image"{
            if let Some(id)=payload.get("jobId").or_else(||payload.get("id")).and_then(Value::as_str){if app.state::<Arc<crate::image_engine::ImageEngine>>().list()?.iter().any(|j|j.id==id&&(j.restricted||request(&j.request))){return Err("privacy_locked".into());}}
        }
    }Ok(())
}

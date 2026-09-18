use serde::Deserialize;
use tauri::{menu::{Menu,Submenu,MenuItem},Manager,Emitter};
#[derive(Deserialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct MenuState {de:bool,enabled:Vec<String>,shortcuts:std::collections::BTreeMap<String,String>,recent:Vec<String>,theme:String}
pub const IDS:&[&str]=&["projectNew","projectOpen","projectSave","projectSaveAs","projectClose","projectAdd","projectRecover","exit","undo","redo","settings","imageFit","imageActual","imageReset","uiZoomIn","uiZoomOut","uiZoomReset","help","updates","about"];
#[tauri::command]
pub fn app_menu_update(state:MenuState,app:tauri::AppHandle)->Result<(),String>{
 if state.enabled.len()>40||state.recent.len()>12||state.recent.iter().any(|s|s.len()>500)||state.shortcuts.len()>40||state.shortcuts.values().any(|v|v.len()>48){return Err("menu_state".into());}
 let menu=Menu::new(&app).map_err(|e|e.to_string())?;
 let groups=[("&Datei","&File",vec![("projectNew","Neues Projekt …","New project …"),("projectOpen","Projekt öffnen …","Open project …"),("projectSave","Projekt speichern","Save project"),("projectSaveAs","Speichern unter …","Save as …"),("projectAdd","Medien hinzufügen …","Add media …"),("projectRecover","Projekt fortsetzen","Resume project"),("projectClose","Projekt schließen","Close project"),("exit","Beenden","Exit")]),("&Bearbeiten","&Edit",vec![("undo","Rückgängig","Undo"),("redo","Wiederholen","Redo"),("settings","Einstellungen","Settings")]),("&Ansicht","&View",vec![("imageFit","Bild einpassen","Fit image"),("imageActual","Bild in 100 %","Actual image size"),("imageReset","Bildansicht zurücksetzen","Reset image view"),("uiZoomIn","Oberfläche vergrößern","Zoom interface in"),("uiZoomOut","Oberfläche verkleinern","Zoom interface out"),("uiZoomReset","Oberfläche auf 100 %","Reset interface zoom")]),("&Hilfe","&Help",vec![("help","Anleitung","User guide"),("updates","Nach Updates suchen …","Check for updates …"),("about","Über Local Studio","About Local Studio")])];
 for(de,en,items)in groups{let sub=Submenu::new(&app,if state.de{de}else{en},true).map_err(|e|e.to_string())?;
 for(id,de,en)in items{let mut label=if state.de{de}else{en}.to_string();if let Some(key)=state.shortcuts.get(id).filter(|s|!s.is_empty()){label.push('\t');label.push_str(&if state.de{key.replace("Ctrl","Strg").replace("Shift","Umschalt")}else{key.clone()});}
 let item=MenuItem::with_id(&app,id,label,state.enabled.iter().any(|v|v==id),None::<&str>).map_err(|e|e.to_string())?;sub.append(&item).map_err(|e|e.to_string())?;
 if id=="projectOpen"{let recent=Submenu::new(&app,if state.de{"Zuletzt geöffnet"}else{"Recent projects"},!state.recent.is_empty()).map_err(|e|e.to_string())?;for(index,name)in state.recent.iter().enumerate(){let id=format!("recent-{index}");let item=MenuItem::with_id(&app,&id,name.replace('&',"&&"),state.enabled.contains(&id),None::<&str>).map_err(|e|e.to_string())?;recent.append(&item).map_err(|e|e.to_string())?;}sub.append(&recent).map_err(|e|e.to_string())?;}
 }menu.append(&sub).map_err(|e|e.to_string())?;}
 let window=app.get_webview_window("main").ok_or("menu_window")?;window.set_menu(menu).map_err(|e|e.to_string())?;window.set_theme(match state.theme.as_str(){"light"=>Some(tauri::Theme::Light),"dark"=>Some(tauri::Theme::Dark),_=>None}).map_err(|e|e.to_string())?;Ok(())
}
pub fn event(app:&tauri::AppHandle,event:tauri::menu::MenuEvent){let id=event.id().as_ref();if IDS.contains(&id)||id.strip_prefix("recent-").and_then(|s|s.parse::<usize>().ok()).is_some_and(|n|n<12){let _=app.emit_to("main","app-menu",id);}}

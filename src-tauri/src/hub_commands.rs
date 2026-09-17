use crate::{
    hf_auth::{AuthStatus, HfAuth},
    hub::{self, ModelDetail, SearchPage, SearchQuery},
};
use std::sync::Arc;
use tauri::State;

#[tauri::command]
pub fn hf_status(state: State<'_, Arc<HfAuth>>) -> Result<AuthStatus, String> {
    state.status()
}

#[tauri::command]
pub async fn hf_start_login(app: tauri::AppHandle, state: State<'_, Arc<HfAuth>>) -> Result<AuthStatus, String> {
    let auth = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || auth.start_login(app))
        .await
        .map_err(|_| "internal")?
}

#[tauri::command]
pub async fn hf_cancel_login(app: tauri::AppHandle, state: State<'_, Arc<HfAuth>>) -> Result<AuthStatus, String> {
    crate::hf_browser::cancel_login(&app, state.inner())
}

#[tauri::command]
pub async fn hf_connect_token(
    token: String,
    state: State<'_, Arc<HfAuth>>,
) -> Result<AuthStatus, String> {
    let auth = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || auth.connect_token(token))
        .await
        .map_err(|_| "internal")?
}

#[tauri::command]
pub async fn hf_logout(state: State<'_, Arc<HfAuth>>) -> Result<AuthStatus, String> {
    let auth = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || auth.logout())
        .await
        .map_err(|_| "internal")?
}

#[tauri::command]
pub async fn hf_verify(state: State<'_, Arc<HfAuth>>) -> Result<AuthStatus, String> {
    let auth = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || auth.verify())
        .await
        .map_err(|_| "internal")?
}

#[tauri::command]
pub async fn hf_search(
    query: SearchQuery,
    state: State<'_, Arc<HfAuth>>,
) -> Result<SearchPage, String> {
    let auth = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        let token = auth.token()?;
        hub::search(query, token.as_ref().map(|token| token.as_str()))
    })
    .await
    .map_err(|_| "internal")?
}

#[tauri::command]
pub async fn hf_model_detail(
    repo: String,
    revision: String,
    state: State<'_, Arc<HfAuth>>,
) -> Result<ModelDetail, String> {
    let auth = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        let token = auth.token()?;
        hub::detail(&repo, &revision, token.as_ref().map(|token| token.as_str()))
    })
    .await
    .map_err(|_| "internal")?
}

#[tauri::command]
pub async fn hf_model_size(repo: String, revision: String, state: State<'_, Arc<HfAuth>>) -> Result<Option<u64>, String> {
    let auth = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        let token = auth.token()?;
        hub::model_size(&repo, &revision, token.as_ref().map(|token| token.as_str()))
    }).await.map_err(|_| "internal")?
}

#[tauri::command]
pub async fn hf_open_page(page: String, repo: Option<String>) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || hub::open_page(&page, repo.as_deref()))
        .await
        .map_err(|_| "internal")?
}

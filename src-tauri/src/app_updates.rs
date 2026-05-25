use std::sync::Mutex;

use serde::Serialize;
use tauri::{ipc::Channel, AppHandle, State, Url};
use tauri_plugin_updater::{Update, UpdaterExt};

const STABLE_ENDPOINTS: [&str; 3] = [
    "https://gh-proxy.com/https://github.com/Thunder-077/Boom/releases/latest/download/latest.json",
    "https://ghfast.top/https://github.com/Thunder-077/Boom/releases/latest/download/latest.json",
    "https://github.com/Thunder-077/Boom/releases/latest/download/latest.json",
];

const CANARY_ENDPOINTS: [&str; 3] = [
    "https://gh-proxy.com/https://raw.githubusercontent.com/Thunder-077/Boom/update-manifests/canary/latest.json",
    "https://ghfast.top/https://raw.githubusercontent.com/Thunder-077/Boom/update-manifests/canary/latest.json",
    "https://raw.githubusercontent.com/Thunder-077/Boom/update-manifests/canary/latest.json",
];

#[derive(Clone, Copy, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum UpdateChannel {
    Stable,
    Canary,
}

#[derive(Debug)]
pub enum Error {
    Updater(tauri_plugin_updater::Error),
    NoPendingUpdate,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Updater(error) => write!(f, "{error}"),
            Self::NoPendingUpdate => write!(f, "there is no pending update"),
        }
    }
}

impl From<tauri_plugin_updater::Error> for Error {
    fn from(value: tauri_plugin_updater::Error) -> Self {
        Self::Updater(value)
    }
}

impl Serialize for Error {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.to_string().as_str())
    }
}

type Result<T> = std::result::Result<T, Error>;

#[derive(Clone, Serialize)]
#[serde(tag = "event", content = "data")]
pub enum DownloadEvent {
    #[serde(rename_all = "camelCase")]
    Started { content_length: Option<u64> },
    #[serde(rename_all = "camelCase")]
    Progress { chunk_length: usize },
    Finished,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateMetadata {
    version: String,
    current_version: String,
    channel: UpdateChannel,
}

pub struct PendingUpdate(pub Mutex<Option<Update>>);

fn detect_update_channel(version: &str) -> UpdateChannel {
    if version.contains("+canary.") {
        UpdateChannel::Canary
    } else {
        UpdateChannel::Stable
    }
}

fn endpoints_for(channel: UpdateChannel) -> Result<Vec<Url>> {
    let endpoints = match channel {
        UpdateChannel::Stable => &STABLE_ENDPOINTS,
        UpdateChannel::Canary => &CANARY_ENDPOINTS,
    };

    endpoints
        .iter()
        .map(|endpoint| Url::parse(endpoint).map_err(tauri_plugin_updater::Error::from).map_err(Error::from))
        .collect()
}

#[tauri::command]
pub async fn fetch_update(
    app: AppHandle,
    pending_update: State<'_, PendingUpdate>,
) -> Result<Option<UpdateMetadata>> {
    let current_version = app.package_info().version.to_string();
    let channel = detect_update_channel(&current_version);

    // Stable 与 Canary 使用不同的更新清单，避免正式用户误收到预发布构建。
    let update = app
        .updater_builder()
        .endpoints(endpoints_for(channel)?)?
        .build()?
        .check()
        .await?;

    let metadata = update.as_ref().map(|update| UpdateMetadata {
        version: update.version.clone(),
        current_version: update.current_version.clone(),
        channel,
    });

    *pending_update.0.lock().unwrap() = update;

    Ok(metadata)
}

#[tauri::command]
pub async fn install_update(
    app: AppHandle,
    pending_update: State<'_, PendingUpdate>,
    on_event: Channel<DownloadEvent>,
) -> Result<()> {
    let Some(update) = pending_update.0.lock().unwrap().take() else {
        return Err(Error::NoPendingUpdate);
    };

    let mut started = false;

    update
        .download_and_install(
            |chunk_length, content_length| {
                if !started {
                    let _ = on_event.send(DownloadEvent::Started { content_length });
                    started = true;
                }

                let _ = on_event.send(DownloadEvent::Progress { chunk_length });
            },
            || {
                let _ = on_event.send(DownloadEvent::Finished);
            },
        )
        .await?;

    app.restart();
}

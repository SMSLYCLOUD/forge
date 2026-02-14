use forge_lsp::LspClient;
use lsp_types::Location;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::runtime::Runtime;
use url::Url;

pub struct GoToDef;

impl GoToDef {
    pub fn execute(
        rt: &Arc<Runtime>,
        client: &Option<Arc<LspClient>>,
        file_path: &str,
        line: u32,
        character: u32,
        notifications: &mut crate::notifications::NotificationManager,
        event_proxy: std::sync::mpsc::Sender<crate::application::AppEvent>,
    ) {
        if let Some(client) = client {
            if let Ok(path_abs) = std::fs::canonicalize(file_path) {
                if let Ok(uri) = Url::from_file_path(&path_abs) {
                    let client = client.clone();
                    let uri = uri.clone();
                    let proxy = event_proxy.clone();

                    rt.spawn(async move {
                        match client.goto_definition(uri, line, character).await {
                            Ok(Some(location)) => {
                                let _ = proxy.send(crate::application::AppEvent::GoToLocation(location));
                            }
                            Ok(None) => {
                                tracing::info!("No definition found");
                            }
                            Err(e) => {
                                tracing::error!("GoToDef error: {}", e);
                            }
                        }
                    });
                }
            }
        } else {
            notifications.show(
                "LSP client not connected",
                crate::notifications::Level::Warning,
            );
        }
    }
}

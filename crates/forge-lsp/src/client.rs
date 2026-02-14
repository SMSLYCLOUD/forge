use crate::server::LspServer;
use crate::transport::Transport;
use anyhow::{Context, Result};
use lsp_types::{
    ClientCapabilities, CompletionItem, CompletionParams, CompletionResponse,
    DidChangeTextDocumentParams, DidOpenTextDocumentParams, Hover, HoverParams, InitializeParams,
    InitializeResult, InitializedParams, Position, TextDocumentContentChangeEvent,
    TextDocumentIdentifier, TextDocumentItem, TextDocumentPositionParams, Uri,
    VersionedTextDocumentIdentifier, WorkspaceFolder,
};
use serde_json::json;
use std::str::FromStr;
use std::sync::atomic::{AtomicI64, Ordering};
use url::Url;

pub struct LspClient {
    server: LspServer,
    transport: Option<Transport>,
    next_id: AtomicI64,
}

impl LspClient {
    pub fn new(server: LspServer) -> Self {
        Self {
            server,
            transport: None,
            next_id: AtomicI64::new(1),
        }
    }

    pub fn initialize_transport(&mut self) -> Result<()> {
        let stdin = self
            .server
            .stdin
            .take()
            .context("Server stdin already taken")?;
        let stdout = self
            .server
            .stdout
            .take()
            .context("Server stdout already taken")?;
        self.transport = Some(Transport::new(stdin, stdout));
        Ok(())
    }

    pub async fn initialize(&self, root_url: Url) -> Result<InitializeResult> {
        let root_uri =
            Uri::from_str(root_url.as_str()).map_err(|e| anyhow::anyhow!("Invalid URI: {}", e))?;
        let workspace_name = root_url
            .path_segments()
            .and_then(|mut segments| segments.rfind(|s| !s.is_empty()))
            .unwrap_or("workspace")
            .to_string();

        let params = InitializeParams {
            process_id: Some(std::process::id()),
            capabilities: ClientCapabilities::default(),
            workspace_folders: Some(vec![WorkspaceFolder {
                uri: root_uri,
                name: workspace_name,
            }]),
            ..Default::default()
        };

        let response = self.request("initialize", params).await?;
        let result: InitializeResult = serde_json::from_value(response)?;

        // Send initialized notification
        self.notify("initialized", InitializedParams {}).await?;

        Ok(result)
    }

    pub async fn did_open(&self, uri: Url, text: String, language_id: String) -> Result<()> {
        let uri = Uri::from_str(uri.as_str()).map_err(|e| anyhow::anyhow!("Invalid URI: {}", e))?;

        let params = DidOpenTextDocumentParams {
            text_document: TextDocumentItem {
                uri,
                language_id,
                version: 1,
                text,
            },
        };
        self.notify("textDocument/didOpen", params).await
    }

    pub async fn did_change(&self, uri: Url, version: i32, text: String) -> Result<()> {
        let uri = Uri::from_str(uri.as_str()).map_err(|e| anyhow::anyhow!("Invalid URI: {}", e))?;

        let params = DidChangeTextDocumentParams {
            text_document: VersionedTextDocumentIdentifier { uri, version },
            content_changes: vec![TextDocumentContentChangeEvent {
                range: None,
                range_length: None,
                text,
            }],
        };
        self.notify("textDocument/didChange", params).await
    }

    pub async fn completion(
        &self,
        uri: Url,
        line: u32,
        character: u32,
    ) -> Result<Vec<CompletionItem>> {
        let uri = Uri::from_str(uri.as_str()).map_err(|e| anyhow::anyhow!("Invalid URI: {}", e))?;

        let params = CompletionParams {
            text_document_position: TextDocumentPositionParams {
                text_document: TextDocumentIdentifier { uri },
                position: Position { line, character },
            },
            work_done_progress_params: Default::default(),
            partial_result_params: Default::default(),
            context: None,
        };

        let response = self.request("textDocument/completion", params).await?;
        let result: CompletionResponse = serde_json::from_value(response)?;

        match result {
            CompletionResponse::Array(items) => Ok(items),
            CompletionResponse::List(list) => Ok(list.items),
        }
    }

    pub async fn hover(&self, uri: Url, line: u32, character: u32) -> Result<Option<Hover>> {
        let uri = Uri::from_str(uri.as_str()).map_err(|e| anyhow::anyhow!("Invalid URI: {}", e))?;

        let params = HoverParams {
            text_document_position_params: TextDocumentPositionParams {
                text_document: TextDocumentIdentifier { uri },
                position: Position { line, character },
            },
            work_done_progress_params: Default::default(),
        };

        let response = self.request("textDocument/hover", params).await?;
        if response.is_null() {
            Ok(None)
        } else {
            Ok(Some(serde_json::from_value(response)?))
        }
    }

    pub async fn goto_definition(
        &self,
        uri: Url,
        line: u32,
        character: u32,
    ) -> Result<Option<lsp_types::Location>> {
        let uri = Uri::from_str(uri.as_str()).map_err(|e| anyhow::anyhow!("Invalid URI: {}", e))?;

        let params = TextDocumentPositionParams {
            text_document: TextDocumentIdentifier { uri },
            position: Position { line, character },
        };

        let response = self.request("textDocument/definition", params).await?;

        // Response can be Location | Location[] | LocationLink[] | null
        if response.is_null() {
            return Ok(None);
        }

        // Try single Location
        if let Ok(loc) = serde_json::from_value::<lsp_types::Location>(response.clone()) {
            return Ok(Some(loc));
        }

        // Try Vec<Location> - take first
        if let Ok(locs) = serde_json::from_value::<Vec<lsp_types::Location>>(response.clone()) {
            return Ok(locs.into_iter().next());
        }

        // Try LocationLink[] - map to Location (targetUri + targetRange)
        if let Ok(links) = serde_json::from_value::<Vec<lsp_types::LocationLink>>(response) {
            if let Some(link) = links.into_iter().next() {
                return Ok(Some(lsp_types::Location {
                    uri: link.target_uri,
                    range: link.target_range,
                }));
            }
        }

        Ok(None)
    }

    async fn request<T: serde::Serialize>(
        &self,
        method: &str,
        params: T,
    ) -> Result<serde_json::Value> {
        let transport = self
            .transport
            .as_ref()
            .context("Transport not initialized")?;
        let id = self.next_id.fetch_add(1, Ordering::SeqCst);

        let msg = json!({
            "jsonrpc": "2.0",
            "id": id,
            "method": method,
            "params": params,
        });

        transport.send(&msg).await?;

        // Wait for response with matching ID
        // Simplified: assuming next message is response. In reality, need a loop and map.
        loop {
            let response = transport.receive().await?;
            if let Some(resp_id) = response.get("id").and_then(|id| id.as_i64()) {
                if resp_id == id {
                    if let Some(error) = response.get("error") {
                        return Err(anyhow::anyhow!("LSP Error: {:?}", error));
                    }
                    return Ok(response
                        .get("result")
                        .cloned()
                        .unwrap_or(serde_json::Value::Null));
                }
            }
            // Handle notifications or other responses?
        }
    }

    async fn notify<T: serde::Serialize>(&self, method: &str, params: T) -> Result<()> {
        let transport = self
            .transport
            .as_ref()
            .context("Transport not initialized")?;

        let msg = json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": params,
        });

        transport.send(&msg).await
    }
}

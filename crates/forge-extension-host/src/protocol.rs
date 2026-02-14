use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum HostMessage {
    ShowInfo { message: String },
    ShowError { message: String },
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ClientMessage {
    ExecuteCommand { command: String, args: Vec<String> },
}

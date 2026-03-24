use archeo::content_analysis::notebooks::notebook::Notebook;
use std::collections::BTreeSet;
use std::path::PathBuf;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum ChatRole {
    User,
    Assistant,
    System,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ChatMessage {
    pub role: ChatRole,
    pub content: String,
}

pub struct AppState {
    pub notebook_path: Option<PathBuf>,
    pub notebook: Option<Notebook>,

    pub selected_cells: BTreeSet<usize>,
    pub selected_outputs: BTreeSet<String>,

    pub ollama_url: String,
    pub model_name: String,
    pub user_input: String,
    pub context_preview: String,
    pub status: String,

    pub messages: Vec<ChatMessage>,
    pub ai_busy: bool,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            notebook_path: None,
            notebook: None,
            selected_cells: BTreeSet::new(),
            selected_outputs: BTreeSet::new(),
            ollama_url: "http://127.0.0.1:11434/api/generate".to_string(),
            model_name: "llama3.2".to_string(),
            user_input: String::new(),
            context_preview: String::new(),
            status: "Ready".to_string(),
            messages: Vec::new(),
            ai_busy: false,
        }
    }
}

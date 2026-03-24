use anyhow::{Context, Result};
use chrono::{DateTime, Local};
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AppEventLog {
    pub timestamp: DateTime<Local>,
    pub level: String,
    pub message: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AiExchangeLog {
    pub timestamp: DateTime<Local>,
    pub notebook_path: Option<PathBuf>,
    pub selected_cells: Vec<usize>,
    pub selected_outputs: Vec<String>,
    pub model: String,
    pub ollama_url: String,
    pub user_message: String,
    pub context_preview: String,
    pub full_prompt: String,
    pub response: Option<String>,
    pub error: Option<String>,
}

pub struct SessionLogger {
    session_dir: PathBuf,
}

impl SessionLogger {
    pub fn new() -> Result<Self> {
        let now = Local::now().format("%Y-%m-%d_%H-%M-%S").to_string();
        let session_dir = PathBuf::from("logs").join(format!("session_{now}"));
        fs::create_dir_all(&session_dir)
            .with_context(|| format!("Failed to create session dir {}", session_dir.display()))?;
        Ok(Self { session_dir })
    }

    pub fn session_dir(&self) -> &Path {
        &self.session_dir
    }

    pub fn append_event(&self, event: &AppEventLog) -> Result<()> {
        let path = self.session_dir.join("events.jsonl");
        let mut fh = File::options()
            .create(true)
            .append(true)
            .open(&path)
            .with_context(|| format!("Failed to open {}", path.display()))?;

        writeln!(fh, "{}", serde_json::to_string(event)?)?;
        Ok(())
    }

    pub fn write_exchange(&self, idx: usize, exchange: &AiExchangeLog) -> Result<()> {
        let base = format!("exchange_{idx:04}");
        fs::write(
            self.session_dir.join(format!("{base}.json")),
            serde_json::to_string_pretty(exchange)?,
        )?;

        fs::write(
            self.session_dir.join(format!("{base}_prompt.txt")),
            &exchange.full_prompt,
        )?;

        fs::write(
            self.session_dir.join(format!("{base}_context.txt")),
            &exchange.context_preview,
        )?;

        if let Some(response) = &exchange.response {
            fs::write(
                self.session_dir.join(format!("{base}_response.txt")),
                response,
            )?;
        }

        if let Some(error) = &exchange.error {
            fs::write(self.session_dir.join(format!("{base}_error.txt")), error)?;
        }

        Ok(())
    }
}

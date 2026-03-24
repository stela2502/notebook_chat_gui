use crate::ai::run_ollama_prompt;
use crate::logging::{AiExchangeLog, AppEventLog, SessionLogger};
use crate::state::{AppState, ChatMessage, ChatRole};
use anyhow::{Context, Result};
use archeo::content_analysis::notebooks::notebook::{Notebook, NotebookParserConfig};
use chrono::Local;
use crossbeam_channel::{unbounded, Receiver, Sender};
use eframe::egui;
use rfd::FileDialog;
use std::path::{Path, PathBuf};
use std::thread;

#[derive(Debug, Clone)]
struct PendingAiResult {
    exchange: AiExchangeLog,
}

pub struct NotebookChatApp {
    state: AppState,
    logger: SessionLogger,
    events: Vec<AppEventLog>,
    exchanges: Vec<AiExchangeLog>,
    ai_tx: Sender<PendingAiResult>,
    ai_rx: Receiver<PendingAiResult>,
}

impl NotebookChatApp {
    pub fn new(startup_notebook: Option<PathBuf>) -> Self {
        let logger = SessionLogger::new().expect("failed to create logger");
        let (ai_tx, ai_rx) = unbounded();

        let mut app = Self {
            state: AppState::default(),
            logger,
            events: Vec::new(),
            exchanges: Vec::new(),
            ai_tx,
            ai_rx,
        };

        app.log_event("INFO", "Application started");

        if let Some(path) = startup_notebook {
            if let Err(err) = app.load_notebook(&path) {
                app.state.status = format!("Failed to load notebook: {err:#}");
                let msg = app.state.status.clone();
                app.log_event("ERROR", &msg);
            }
        }

        app
    }

    fn log_event(&mut self, level: &str, message: &str) {
        let event = AppEventLog {
            timestamp: Local::now(),
            level: level.to_string(),
            message: message.to_string(),
        };

        let _ = self.logger.append_event(&event);
        self.events.push(event);
    }

    fn load_notebook(&mut self, path: &Path) -> Result<()> {
        let cfg = NotebookParserConfig::default();
        let notebook = Notebook::from_file(path, cfg)
            .with_context(|| format!("Failed to parse notebook {}", path.display()))?;

        self.state.notebook_path = Some(path.to_path_buf());
        self.state.notebook = Some(notebook);
        self.state.selected_cells.clear();
        self.state.selected_outputs.clear();
        self.rebuild_context_preview();

        self.state.status = format!("Loaded notebook {}", path.display());
        let msg = self.state.status.clone();
        self.log_event("INFO", &msg);

        Ok(())
    }

    fn rebuild_context_preview(&mut self) {
        let mut out = String::new();

        if let Some(path) = &self.state.notebook_path {
            out.push_str(&format!("Notebook: {}\n\n", path.display()));
        }

        if let Some(notebook) = &self.state.notebook {
            let area = notebook.get_for_area(0, notebook.len());

            for cell in &area.code_cells {
                if self.state.selected_cells.contains(&cell.id) {
                    out.push_str("[Selected code cell]\n");
                    out.push_str(&format!("{cell}\n\n"));
                }
            }

            for output in &area.outputs {
                if self.state.selected_outputs.contains(&output.id) {
                    out.push_str("[Selected retained output]\n");
                    out.push_str(&format!("{output}\n\n"));
                }
            }
        }

        self.state.context_preview = out;
    }

    fn start_ai_request(&mut self) {
        if self.state.ai_busy {
            return;
        }

        let user_message = self.state.user_input.trim().to_string();
        if user_message.is_empty() {
            self.state.status = "Cannot send an empty message".to_string();
            let msg = self.state.status.clone();
            self.log_event("WARN", &msg);
            return;
        }

        let full_prompt = if self.state.context_preview.trim().is_empty() {
            user_message.clone()
        } else {
            format!(
                "{}\n\nUser question:\n{}",
                self.state.context_preview, user_message
            )
        };

        let mut exchange = AiExchangeLog {
            timestamp: Local::now(),
            notebook_path: self.state.notebook_path.clone(),
            selected_cells: self.state.selected_cells.iter().copied().collect(),
            selected_outputs: self.state.selected_outputs.iter().cloned().collect(),
            model: self.state.model_name.clone(),
            ollama_url: self.state.ollama_url.clone(),
            user_message: user_message.clone(),
            context_preview: self.state.context_preview.clone(),
            full_prompt: full_prompt.clone(),
            response: None,
            error: None,
        };

        let exchange_idx = self.exchanges.len() + 1;
        let _ = self.logger.write_exchange(exchange_idx, &exchange);

        self.state.messages.push(ChatMessage {
            role: ChatRole::User,
            content: user_message,
        });

        self.state.user_input.clear();
        self.state.ai_busy = true;
        self.state.status = format!("Sending request to model {}", self.state.model_name);
        let msg = self.state.status.clone();
        self.log_event("INFO", &msg);

        let tx = self.ai_tx.clone();

        thread::spawn(move || {
            match run_ollama_prompt(&exchange.ollama_url, &exchange.model, &exchange.full_prompt) {
                Ok(response) => {
                    exchange.response = Some(response);
                }
                Err(err) => {
                    exchange.error = Some(format!("{err:#}"));
                }
            }

            let _ = tx.send(PendingAiResult { exchange });
        });

        // Reset selection after sending
        self.state.selected_cells.clear();
        self.state.selected_outputs.clear();

        self.log_event("INFO", "Cleared selection after sending request");
    }

    fn poll_ai_results(&mut self) {
        while let Ok(result) = self.ai_rx.try_recv() {
            self.state.ai_busy = false;

            let idx = self.exchanges.len() + 1;

            if let Some(response) = &result.exchange.response {
                self.state.messages.push(ChatMessage {
                    role: ChatRole::Assistant,
                    content: response.clone(),
                });

                self.state.status = "AI response received".to_string();
                self.log_event("INFO", "AI response received");
            } else if let Some(error) = &result.exchange.error {
                self.state.messages.push(ChatMessage {
                    role: ChatRole::System,
                    content: format!("AI request failed:\n{error}"),
                });

                self.state.status = "AI request failed".to_string();
                self.log_event("ERROR", "AI request failed");
            }

            let _ = self.logger.write_exchange(idx, &result.exchange);
            self.exchanges.push(result.exchange);
        }
    }

    fn ui_top_bar(&mut self, ui: &mut egui::Ui) {
        if ui.button("Load Notebook").clicked() {
            if let Some(path) = FileDialog::new()
                .add_filter("Jupyter notebook", &["ipynb"])
                .pick_file()
            {
                if let Err(err) = self.load_notebook(&path) {
                    self.state.status = format!("Load failed: {err:#}");
                    let msg = self.state.status.clone();
                    self.log_event("ERROR", &msg);
                }
            }
        }

        if ui.button("Reload").clicked() {
            if let Some(path) = self.state.notebook_path.clone() {
                if let Err(err) = self.load_notebook(&path) {
                    self.state.status = format!("Reload failed: {err:#}");
                    let msg = self.state.status.clone();
                    self.log_event("ERROR", &msg);
                }
            }
        }

        if ui.button("Clear Selection").clicked() {
            self.state.selected_cells.clear();
            self.state.selected_outputs.clear();
            self.rebuild_context_preview();
            self.log_event("INFO", "Cleared notebook selection");
        }

        ui.separator();
        ui.label("Ollama URL:");
        ui.text_edit_singleline(&mut self.state.ollama_url);

        ui.label("Model:");
        ui.text_edit_singleline(&mut self.state.model_name);

        if self.state.ai_busy {
            ui.separator();
            ui.spinner();
            ui.label("AI busy");
        }
    }

    fn ui_notebook_panel(&mut self, ui: &mut egui::Ui) {
        ui.heading("Notebook");

        if self.state.notebook.is_none() {
            ui.label("No notebook loaded.");
            return;
        }

        if let Some(path) = &self.state.notebook_path {
            ui.label(format!("File: {}", path.display()));
        }

        let area = {
            let notebook = self.state.notebook.as_ref().expect("checked above");
            ui.label(format!("Cells: {}", notebook.len()));
            ui.label(format!(
                "Retained outputs: {}",
                notebook.retained_outputs().len()
            ));
            notebook.get_for_area(0, notebook.len())
        };

        ui.separator();

        ui.horizontal(|ui| {
            if ui.button("All").clicked() {
                self.state.selected_cells = area.code_cells.iter().map(|c| c.id).collect();
                self.state.selected_outputs = area.outputs.iter().map(|o| o.id.clone()).collect();
                self.rebuild_context_preview();
                self.log_event("INFO", "Selected entire notebook");
            }

            if ui.button("Only Code").clicked() {
                self.state.selected_cells = area.code_cells.iter().map(|c| c.id).collect();
                self.state.selected_outputs.clear();
                self.rebuild_context_preview();
                self.log_event("INFO", "Selected all code cells");
            }

            if ui.button("Only Outputs").clicked() {
                self.state.selected_cells.clear();
                self.state.selected_outputs = area.outputs.iter().map(|o| o.id.clone()).collect();
                self.rebuild_context_preview();
                self.log_event("INFO", "Selected all outputs");
            }

            if ui.button("Clear All").clicked() {
                self.state.selected_cells.clear();
                self.state.selected_outputs.clear();
                self.rebuild_context_preview();
                self.log_event("INFO", "Cleared all selections");
            }
        });

        let mut include_all = self.state.selected_cells.len() == area.code_cells.len()
            && self.state.selected_outputs.len() == area.outputs.len();

        if ui.checkbox(&mut include_all, "Include entire notebook").changed() {
            if include_all {
                self.state.selected_cells = area.code_cells.iter().map(|c| c.id).collect();
                self.state.selected_outputs = area.outputs.iter().map(|o| o.id.clone()).collect();
                self.log_event("INFO", "Enabled entire notebook selection");
            } else {
                self.state.selected_cells.clear();
                self.state.selected_outputs.clear();
                self.log_event("INFO", "Disabled entire notebook selection");
            }

            self.rebuild_context_preview();
        }

        ui.separator();

        egui::ScrollArea::vertical().show(ui, |ui| {
            for cell in &area.code_cells {
                egui::CollapsingHeader::new(format!("Cell {}", cell.id))
                    .default_open(false)
                    .show(ui, |ui| {
                        let mut selected = self.state.selected_cells.contains(&cell.id);
                        if ui.checkbox(&mut selected, "Include cell in AI context").changed() {
                            if selected {
                                self.state.selected_cells.insert(cell.id);
                            } else {
                                self.state.selected_cells.remove(&cell.id);
                            }
                            self.rebuild_context_preview();
                        }

                        ui.label(format!(
                            "Execution count: {}",
                            cell.execution_count
                                .map(|v| v.to_string())
                                .unwrap_or_else(|| "<none>".to_string())
                        ));

                        ui.label("Code:");
                        let mut code = cell.source.clone();
                        ui.add(
                            egui::TextEdit::multiline(&mut code)
                                .desired_rows(8)
                                .interactive(false),
                        );

                        if !cell.output_ids.is_empty() {
                            ui.separator();
                            ui.label("Linked outputs:");

                            for output_id in &cell.output_ids {
                                let mut selected = self.state.selected_outputs.contains(output_id);
                                if ui.checkbox(&mut selected, output_id).changed() {
                                    if selected {
                                        self.state.selected_outputs.insert(output_id.clone());
                                    } else {
                                        self.state.selected_outputs.remove(output_id);
                                    }
                                    self.rebuild_context_preview();
                                }
                            }
                        }
                    });
            }
        });
    }

    fn ui_context_panel(&mut self, ui: &mut egui::Ui) {
        ui.heading("AI Context");
        ui.label("Selected notebook content that will be sent to the AI:");
        ui.separator();

        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.add(
                egui::TextEdit::multiline(&mut self.state.context_preview)
                    .desired_rows(30)
                    .interactive(false),
            );
        });
    }

    fn ui_chat_panel(&mut self, ui: &mut egui::Ui) {
        ui.heading("Chat");
        ui.separator();

        // Bottom-fixed composer area
        let composer_height = 140.0;
        let available_width = ui.available_width();

        ui.allocate_ui_with_layout(
            egui::vec2(available_width, composer_height),
            egui::Layout::bottom_up(egui::Align::LEFT),
            |ui| {
                ui.separator();

                let send_clicked = ui
                    .add_enabled(!self.state.ai_busy, egui::Button::new("Send to AI"))
                    .clicked();

                ui.label("Message:");
                ui.add_enabled(
                    !self.state.ai_busy,
                    egui::TextEdit::multiline(&mut self.state.user_input)
                        .desired_rows(5)
                        .desired_width(f32::INFINITY),
                );

                if send_clicked {
                    self.start_ai_request();
                }
            },
        );

        ui.separator();

        // Remaining space goes to chat history
        egui::ScrollArea::vertical()
            .stick_to_bottom(true)
            .show(ui, |ui| {
                for msg in &self.state.messages {
                    let who = match msg.role {
                        ChatRole::User => "User",
                        ChatRole::Assistant => "Assistant",
                        ChatRole::System => "System",
                    };

                    ui.group(|ui| {
                        ui.label(format!("{who}:"));
                        ui.separator();
                        ui.label(&msg.content);
                    });

                    ui.add_space(6.0);
                }
            });
    }

    fn ui_log_panel(&mut self, ui: &mut egui::Ui) {
        ui.heading("Log");
        ui.label(format!(
            "Session dir: {}",
            self.logger.session_dir().display()
        ));
        ui.separator();

        egui::ScrollArea::vertical().show(ui, |ui| {
            for event in self.events.iter().rev() {
                ui.label(format!(
                    "[{}] {} - {}",
                    event.timestamp.format("%Y-%m-%d %H:%M:%S"),
                    event.level,
                    event.message
                ));
            }
        });
    }
}

impl eframe::App for NotebookChatApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.poll_ai_results();

        egui::TopBottomPanel::top("top_bar").show(ctx, |ui| {
            self.ui_top_bar(ui);
            ui.separator();
            ui.label(format!("Status: {}", self.state.status));
        });

        egui::SidePanel::left("notebook_panel")
            .resizable(true)
            .default_width(320.0)
            .show(ctx, |ui| {
                self.ui_notebook_panel(ui);
            });

        egui::SidePanel::right("chat_panel")
            .resizable(true)
            .default_width(420.0)
            .show(ctx, |ui| {
                self.ui_chat_panel(ui);
            });

        egui::TopBottomPanel::bottom("log_panel")
            .resizable(true)
            .default_height(160.0)
            .show(ctx, |ui| {
                self.ui_log_panel(ui);
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            self.ui_context_panel(ui);
        });

        ctx.request_repaint();
    }
}

![CI](https://github.com/stela2502/notebook_chat_gui/actions/workflows/ci.yml/badge.svg)

# 📓 Notebook Chat GUI

An interactive Rust-based GUI for exploring **Jupyter notebooks** and chatting with a **local AI (Ollama)** using selected notebook content.

This tool bridges the gap between **data analysis notebooks** and **AI-assisted interpretation**, allowing you to selectively include code cells and outputs in a structured prompt — without losing control over what the AI sees.

---

## 🚀 Features

* 📂 Load Jupyter notebooks (`.ipynb`)
* ✅ Select individual code cells and outputs via checkboxes
* 📦 Include entire notebook or partial selections
* 🧠 Send structured context to a local AI model (Ollama)
* 💬 Chat interface with persistent conversation history
* 📜 Full logging of:

  * user messages
  * AI responses
  * exact prompts sent
* 🔍 Context preview before sending (no hidden magic!)
* 🧪 Designed for **scientific workflows** (e.g. single-cell analysis)

---

## 🧠 Why this exists

Notebook analysis is often:

* fragmented
* hard to summarize
* easy for AI to hallucinate on

This tool enforces:

* **explicit context selection**
* **traceable outputs**
* **reproducible AI interaction**

👉 You decide what the AI sees.

---

## ⚙️ Requirements

* Rust (stable, ≥1.75 recommended)
* A running **local Ollama server**

Install Ollama:
👉 https://ollama.com

---

## 🤖 Setup Ollama

Start the local server:

```bash
ollama serve
```

Pull a model (example):

```bash
ollama pull llama3.2
```

The GUI expects:

```text
http://127.0.0.1:11434/api/generate
```

---

## 🛠 Installation

Clone the repository:

```bash
git clone https://github.com/stela2502/notebook_chat_gui.git
cd notebook_chat_gui
```

Build:

```bash
cargo build --release
```

Run:

```bash
cargo run
```

---

## 🖥 Usage

### 1. Load a notebook

* Open a `.ipynb` file from the GUI

### 2. Select context

* Choose:

  * individual cells
  * outputs
  * or entire notebook

### 3. Inspect context preview

* See exactly what will be sent to the AI

### 4. Ask a question

Examples:

* *"Summarize the analysis"*
* *"What files are generated?"*
* *"Extract all parameters used in filtering"*

### 5. Send to AI

* The selected content is sent to your local Ollama model

---

## 📊 Example Use Cases

* 🧬 Single-cell RNA-seq notebook interpretation
* 📈 Statistical analysis documentation
* 📄 Generating method sections for publications
* 🔍 Extracting parameters and results from messy notebooks
* 🧪 Debugging analysis workflows

---

## ⚠️ Important Notes

### ❗ The AI only sees what you select

If you don't include a cell/output → it does not exist for the model.

### ❗ Hallucinations are still possible

Use strict prompts like:

```text
Only use information present in the notebook.
Do not introduce external tools or assumptions.
```

### ❗ Local-first design

* No cloud required
* No data leaves your machine

---

## 🧱 Architecture Overview

```text
Notebook (.ipynb)
   ↓
Parsed into:
  - code cells
  - retained outputs
   ↓
User selection (GUI)
   ↓
Context builder
   ↓
Prompt
   ↓
Ollama (local model)
   ↓
Response + logging
```

---

## 📁 Logging

Each interaction stores:

* timestamp
* notebook path
* selected cells/outputs
* full prompt
* AI response

👉 Fully reproducible AI analysis

---

## 🔮 Roadmap

* [ ] Persistent session memory
* [ ] Notebook fact extraction (auto-grounding)
* [ ] Embedding-based retrieval across notebooks
* [ ] Multi-notebook context support
* [ ] Export to Markdown / publication format

---

## 🤝 Contributing

Pull requests are welcome.

Focus areas:

* UI improvements (egui)
* better prompt control
* grounding / anti-hallucination features

---

## 📜 License

MIT

---

## 👤 Author

Stefan Lang
Bioinformatics / HPC / Rust enthusiast
Lund University

---

## 💡 Final Thought

This is not "chat with your notebook".

This is:

> **controlled AI interaction with structured scientific context**

Which is a very different — and much more powerful — thing.


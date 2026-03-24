use anyhow::Result;
use archeo::ollama::Ollama;

pub fn run_ollama_prompt(base_url: &str, model: &str, prompt: &str) -> Result<String> {
    let ollama = Ollama::new(base_url.to_string());
    ollama.generate(model, prompt)
}

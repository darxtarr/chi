// Model Cradle: Loading and running LLMs with Candle
//
// This module provides the core infrastructure for running local models
// in Chorus/Chi. It handles:
// - Loading GGUF quantized models from disk
// - Managing context windows and conversation history
// - Running inference with controllable sampling
// - Multiple models running simultaneously
//
// We roll our own model loading (no hf-hub dependency).
// User downloads models manually to a known directory.

use anyhow::{Context, Result};
use candle_core::{DType, Device, Tensor};
use candle_transformers::models::quantized_llama as llama;
use std::path::PathBuf;
use tokenizers::Tokenizer;

/// A message in the conversation
#[derive(Debug, Clone)]
pub struct Message {
    pub role: String,  // "system", "user", "assistant"
    pub content: String,
}

/// Sampling parameters for inference
#[derive(Debug, Clone)]
pub struct SamplingParams {
    pub temperature: f64,
    pub top_p: f64,
    pub max_tokens: usize,
    pub stop_sequences: Vec<String>,
}

impl Default for SamplingParams {
    fn default() -> Self {
        Self {
            temperature: 0.7,
            top_p: 0.9,
            max_tokens: 512,
            stop_sequences: vec![],
        }
    }
}

/// The model cradle - loads and runs one LLM
pub struct ModelCradle {
    model: llama::ModelWeights,
    tokenizer: Tokenizer,
    device: Device,
    context: Vec<Message>,
    max_context_tokens: usize,
}

impl ModelCradle {
    /// Load a GGUF model from disk
    ///
    /// # Arguments
    /// * `model_path` - Path to .gguf file (e.g., "models/qwen2.5-3b-instruct-q4_K_M.gguf")
    /// * `tokenizer_path` - Path to tokenizer.json
    /// * `max_context_tokens` - Maximum context window size
    ///
    /// # Example
    /// ```no_run
    /// let cradle = ModelCradle::load(
    ///     "models/qwen2.5-3b-instruct-q4_K_M.gguf",
    ///     "models/qwen2.5-tokenizer.json",
    ///     8192
    /// )?;
    /// ```
    pub fn load(
        model_path: impl Into<PathBuf>,
        tokenizer_path: impl Into<PathBuf>,
        max_context_tokens: usize,
    ) -> Result<Self> {
        let model_path = model_path.into();
        let tokenizer_path = tokenizer_path.into();

        // Determine device (CPU for now, GPU support later)
        let device = Device::Cpu;

        // Load tokenizer
        let tokenizer = Tokenizer::from_file(&tokenizer_path)
            .map_err(|e| anyhow::anyhow!("Failed to load tokenizer: {}", e))
            .context(format!("Loading tokenizer from {:?}", tokenizer_path))?;

        // Load GGUF model
        let mut file = std::fs::File::open(&model_path)
            .context(format!("Opening model file {:?}", model_path))?;

        let model = llama::ModelWeights::from_gguf(&mut file, &device)
            .context("Loading GGUF model")?;

        Ok(Self {
            model,
            tokenizer,
            device,
            context: Vec::new(),
            max_context_tokens,
        })
    }

    /// Add a system prompt (typically called once at initialization)
    pub fn set_system_prompt(&mut self, prompt: impl Into<String>) {
        self.context.insert(
            0,
            Message {
                role: "system".to_string(),
                content: prompt.into(),
            },
        );
    }

    /// Think: send a prompt and get a response
    ///
    /// This adds the user message to context, runs inference, and adds
    /// the assistant's response to context.
    ///
    /// # Example
    /// ```no_run
    /// let response = cradle.think("What should I do about this incident?").await?;
    /// ```
    pub async fn think(&mut self, prompt: impl Into<String>) -> Result<String> {
        self.think_with_params(prompt, SamplingParams::default())
            .await
    }

    /// Think with custom sampling parameters
    ///
    /// Allows control over temperature, top_p, etc.
    pub async fn think_with_params(
        &mut self,
        prompt: impl Into<String>,
        params: SamplingParams,
    ) -> Result<String> {
        // Add user message to context
        let user_message = Message {
            role: "user".to_string(),
            content: prompt.into(),
        };
        self.context.push(user_message);

        // Format context into prompt (model-specific formatting)
        let formatted_prompt = self.format_context()?;

        // Tokenize
        let tokens = self
            .tokenizer
            .encode(formatted_prompt.clone(), false)
            .map_err(|e| anyhow::anyhow!("Tokenization failed: {}", e))?;

        let input_ids = tokens.get_ids();

        // Check if we're approaching context limit
        if input_ids.len() > self.max_context_tokens {
            // Prune old messages (keep system prompt + recent messages)
            self.prune_context(input_ids.len())?;
            // Re-format and re-tokenize
            let formatted_prompt = self.format_context()?;
            let tokens = self
                .tokenizer
                .encode(formatted_prompt, false)
                .map_err(|e| anyhow::anyhow!("Tokenization failed after pruning: {}", e))?;
        }

        // Run inference (synchronous for now, can make async later)
        let response = tokio::task::spawn_blocking({
            let model = self.model.clone();
            let device = self.device.clone();
            let tokenizer = self.tokenizer.clone();
            let input_ids = input_ids.to_vec();
            let params = params.clone();

            move || -> Result<String> {
                Self::generate(
                    &model,
                    &device,
                    &tokenizer,
                    &input_ids,
                    params,
                )
            }
        })
        .await
        .context("Inference task panicked")??;

        // Add assistant's response to context
        self.context.push(Message {
            role: "assistant".to_string(),
            content: response.clone(),
        });

        Ok(response)
    }

    /// Format conversation context into a prompt string
    ///
    /// This uses a simple chat format. Real implementations would use
    /// model-specific chat templates (Qwen, Llama, etc. have different formats).
    fn format_context(&self) -> Result<String> {
        let mut prompt = String::new();

        for msg in &self.context {
            match msg.role.as_str() {
                "system" => {
                    prompt.push_str(&format!("<|im_start|>system\n{}<|im_end|>\n", msg.content));
                }
                "user" => {
                    prompt.push_str(&format!("<|im_start|>user\n{}<|im_end|>\n", msg.content));
                }
                "assistant" => {
                    prompt.push_str(&format!(
                        "<|im_start|>assistant\n{}<|im_end|>\n",
                        msg.content
                    ));
                }
                _ => {
                    anyhow::bail!("Unknown role: {}", msg.role);
                }
            }
        }

        // Add assistant prompt starter
        prompt.push_str("<|im_start|>assistant\n");

        Ok(prompt)
    }

    /// Prune old messages from context to stay under token limit
    fn prune_context(&mut self, current_tokens: usize) -> Result<()> {
        // Keep system prompt (index 0) + last N messages
        // Simple strategy: remove oldest user/assistant pairs until we're under limit

        let target_tokens = (self.max_context_tokens as f64 * 0.8) as usize;

        if current_tokens <= target_tokens {
            return Ok(());
        }

        // Keep system prompt, remove from index 1 onwards until we're under target
        let mut pruned = vec![self.context[0].clone()];
        let messages_to_keep = self.context.len() / 2; // Keep roughly half

        let start_idx = self.context.len().saturating_sub(messages_to_keep);
        pruned.extend_from_slice(&self.context[start_idx..]);

        self.context = pruned;

        Ok(())
    }

    /// Generate tokens from the model
    ///
    /// This is the core inference loop. It's currently a simplified version.
    /// A production implementation would handle:
    /// - Proper sampling (nucleus, top-k, temperature)
    /// - Stop sequences
    /// - Streaming
    fn generate(
        model: &llama::ModelWeights,
        device: &Device,
        tokenizer: &Tokenizer,
        input_ids: &[u32],
        params: SamplingParams,
    ) -> Result<String> {
        // Convert input IDs to tensor
        let input_tensor = Tensor::new(input_ids, device)?;

        // Generate tokens (simplified - production version needs proper sampling)
        let mut generated_ids = input_ids.to_vec();
        let mut logits_processor = llama::Cache::new(true, DType::F32, &model.config, device)?;

        for _step in 0..params.max_tokens {
            // Forward pass
            let logits = model.forward(
                &Tensor::new(&generated_ids[generated_ids.len().saturating_sub(1)..], device)?,
                generated_ids.len() - 1,
                &mut logits_processor,
            )?;

            // Sample next token (simplified - just argmax for now)
            let next_token = Self::sample_token(&logits, params.temperature)?;

            // Check for EOS
            if next_token == tokenizer.token_to_id("<|im_end|>").unwrap_or(2) {
                break;
            }

            generated_ids.push(next_token);
        }

        // Decode only the newly generated tokens
        let generated_tokens = &generated_ids[input_ids.len()..];
        let response = tokenizer
            .decode(generated_tokens, true)
            .map_err(|e| anyhow::anyhow!("Decoding failed: {}", e))?;

        Ok(response.trim().to_string())
    }

    /// Sample a token from logits (simplified version)
    fn sample_token(logits: &Tensor, temperature: f64) -> Result<u32> {
        // Get logits as vec
        let logits = logits.squeeze(0)?.squeeze(0)?;
        let logits_vec = logits.to_vec1::<f32>()?;

        // Apply temperature
        let scaled: Vec<f32> = if temperature > 0.0 {
            logits_vec.iter().map(|&x| x / temperature as f32).collect()
        } else {
            logits_vec
        };

        // Argmax (simplified - production would do proper sampling)
        let max_idx = scaled
            .iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
            .map(|(idx, _)| idx)
            .ok_or_else(|| anyhow::anyhow!("No tokens in logits"))?;

        Ok(max_idx as u32)
    }

    /// Get current context size in messages
    pub fn context_len(&self) -> usize {
        self.context.len()
    }

    /// Clear context (keeping system prompt if present)
    pub fn clear_context(&mut self) {
        if !self.context.is_empty() && self.context[0].role == "system" {
            let system = self.context[0].clone();
            self.context.clear();
            self.context.push(system);
        } else {
            self.context.clear();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore] // Requires actual model files
    async fn test_load_and_think() -> Result<()> {
        let mut cradle = ModelCradle::load(
            "models/qwen2.5-3b-instruct-q4_K_M.gguf",
            "models/qwen2.5-tokenizer.json",
            8192,
        )?;

        cradle.set_system_prompt("You are a helpful assistant.");

        let response = cradle.think("Hello, how are you?").await?;

        assert!(!response.is_empty());
        assert_eq!(cradle.context_len(), 3); // system + user + assistant

        Ok(())
    }
}

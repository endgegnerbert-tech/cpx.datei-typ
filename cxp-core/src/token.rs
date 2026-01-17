//! Token estimation and savings calculation for CXP
//!
//! This module provides utilities to estimate token counts and calculate
//! savings when using CXP format vs. raw file transmission to LLMs.
//!
//! # Token Estimation
//! We use a conservative estimate of ~4 characters per token, which is
//! typical for English text and code. This aligns with OpenAI's tokenizer
//! estimates for GPT models.
//!
//! # Example
//! ```
//! use cxp_core::token::{estimate_tokens, calculate_savings};
//!
//! let original_size = 10_000_000; // 10MB
//! let cxp_size = 1_500_000;       // 1.5MB
//!
//! let savings = calculate_savings(original_size, cxp_size);
//! println!("Token savings: {}%", savings.savings_percent);
//! ```

use serde::{Deserialize, Serialize};

/// Characters per token - conservative estimate
/// This is based on typical tokenization for GPT models
/// where 1 token ≈ 4 characters for English text and code
pub const CHARS_PER_TOKEN: u64 = 4;

/// Estimate tokens from byte size
///
/// Uses a conservative estimate of 4 characters per token.
/// This works well for:
/// - English text
/// - Source code (most programming languages)
/// - JSON/YAML/TOML
/// - Markdown
///
/// # Arguments
/// * `size_bytes` - Size in bytes
///
/// # Returns
/// Estimated number of tokens
pub fn estimate_tokens(size_bytes: u64) -> u64 {
    // Assume 1 byte = 1 char for most text (ASCII/UTF-8 common chars)
    // Then divide by chars per token
    size_bytes / CHARS_PER_TOKEN
}

/// Token savings analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenSavings {
    /// Original tokens (without CXP)
    pub original_tokens: u64,
    /// CXP tokens (with CXP compression)
    pub cxp_tokens: u64,
    /// Savings percentage
    pub savings_percent: f64,
    /// Absolute token savings
    pub savings_tokens: u64,
}

impl TokenSavings {
    /// Calculate cost savings based on pricing model
    ///
    /// # Arguments
    /// * `input_price` - Price per 1M input tokens (e.g., $3.00 for Claude Opus 4.5)
    /// * `output_price` - Price per 1M output tokens (e.g., $15.00 for Claude Opus 4.5)
    /// * `output_ratio` - Estimated output/input token ratio (default: 0.1 = 10% output)
    ///
    /// # Returns
    /// Cost savings per query in dollars
    pub fn calculate_cost_savings(
        &self,
        input_price: f64,
        output_price: f64,
        output_ratio: f64,
    ) -> CostSavings {
        // Calculate cost for original
        let original_input_cost = (self.original_tokens as f64 / 1_000_000.0) * input_price;
        let original_output_cost =
            (self.original_tokens as f64 * output_ratio / 1_000_000.0) * output_price;
        let original_total_cost = original_input_cost + original_output_cost;

        // Calculate cost for CXP
        let cxp_input_cost = (self.cxp_tokens as f64 / 1_000_000.0) * input_price;
        let cxp_output_cost = (self.cxp_tokens as f64 * output_ratio / 1_000_000.0) * output_price;
        let cxp_total_cost = cxp_input_cost + cxp_output_cost;

        // Calculate savings
        let savings_per_query = original_total_cost - cxp_total_cost;
        let savings_percent =
            (savings_per_query / original_total_cost * 100.0).max(0.0).min(100.0);

        CostSavings {
            original_cost: original_total_cost,
            cxp_cost: cxp_total_cost,
            savings_per_query,
            savings_percent,
            input_price,
            output_price,
            output_ratio,
        }
    }
}

/// Cost savings analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostSavings {
    /// Original cost per query (dollars)
    pub original_cost: f64,
    /// CXP cost per query (dollars)
    pub cxp_cost: f64,
    /// Savings per query (dollars)
    pub savings_per_query: f64,
    /// Savings percentage
    pub savings_percent: f64,
    /// Input token price per 1M
    pub input_price: f64,
    /// Output token price per 1M
    pub output_price: f64,
    /// Output/input ratio
    pub output_ratio: f64,
}

/// Calculate token savings
///
/// # Arguments
/// * `original_size` - Original size in bytes (before CXP)
/// * `cxp_size` - CXP size in bytes (after compression/dedup)
///
/// # Returns
/// TokenSavings with detailed analysis
pub fn calculate_savings(original_size: u64, cxp_size: u64) -> TokenSavings {
    let original_tokens = estimate_tokens(original_size);
    let cxp_tokens = estimate_tokens(cxp_size);

    let savings_tokens = original_tokens.saturating_sub(cxp_tokens);
    let savings_percent = if original_tokens > 0 {
        (savings_tokens as f64 / original_tokens as f64 * 100.0)
            .max(0.0)
            .min(100.0)
    } else {
        0.0
    };

    TokenSavings {
        original_tokens,
        cxp_tokens,
        savings_percent,
        savings_tokens,
    }
}

/// Format bytes as human-readable size
pub fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} bytes", bytes)
    }
}

/// Format tokens as human-readable count
pub fn format_tokens(tokens: u64) -> String {
    const K: u64 = 1_000;
    const M: u64 = K * 1_000;

    if tokens >= M {
        format!("{:.2}M", tokens as f64 / M as f64)
    } else if tokens >= K {
        format!("{:.2}K", tokens as f64 / K as f64)
    } else {
        format!("{}", tokens)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_estimate_tokens() {
        // 1000 bytes = ~250 tokens (4 chars per token)
        assert_eq!(estimate_tokens(1000), 250);

        // 4096 bytes (4KB) = 1024 tokens
        assert_eq!(estimate_tokens(4096), 1024);

        // 1MB = 262,144 tokens
        assert_eq!(estimate_tokens(1_048_576), 262_144);
    }

    #[test]
    fn test_calculate_savings() {
        let original = 10_000_000; // 10MB
        let cxp = 1_500_000; // 1.5MB

        let savings = calculate_savings(original, cxp);

        assert_eq!(savings.original_tokens, 2_500_000);
        assert_eq!(savings.cxp_tokens, 375_000);
        assert_eq!(savings.savings_tokens, 2_125_000);
        assert_eq!(savings.savings_percent, 85.0);
    }

    #[test]
    fn test_calculate_savings_edge_cases() {
        // No compression
        let savings = calculate_savings(1000, 1000);
        assert_eq!(savings.savings_percent, 0.0);

        // 100% compression (unlikely but test edge case)
        let savings = calculate_savings(1000, 0);
        assert_eq!(savings.savings_percent, 100.0);

        // Zero original size
        let savings = calculate_savings(0, 0);
        assert_eq!(savings.savings_percent, 0.0);
    }

    #[test]
    fn test_cost_savings() {
        let token_savings = TokenSavings {
            original_tokens: 2_500_000,
            cxp_tokens: 375_000,
            savings_tokens: 2_125_000,
            savings_percent: 85.0,
        };

        // Claude Opus 4.5 pricing: $3 input, $15 output per 1M tokens
        // Assume 10% output ratio
        let cost_savings = token_savings.calculate_cost_savings(3.0, 15.0, 0.1);

        // Original: 2.5M tokens * $3/M = $7.50 input + 0.25M * $15/M = $3.75 output = $11.25
        // CXP: 0.375M tokens * $3/M = $1.125 input + 0.0375M * $15/M = $0.5625 output = $1.6875
        // Savings: $11.25 - $1.6875 = $9.5625

        assert!((cost_savings.original_cost - 11.25).abs() < 0.01);
        assert!((cost_savings.cxp_cost - 1.6875).abs() < 0.01);
        assert!((cost_savings.savings_per_query - 9.5625).abs() < 0.01);
        assert!((cost_savings.savings_percent - 85.0).abs() < 0.1);
    }

    #[test]
    fn test_format_bytes() {
        assert_eq!(format_bytes(500), "500 bytes");
        assert_eq!(format_bytes(1024), "1.00 KB");
        assert_eq!(format_bytes(1_048_576), "1.00 MB");
        assert_eq!(format_bytes(1_073_741_824), "1.00 GB");
        assert_eq!(format_bytes(10_485_760), "10.00 MB");
    }

    #[test]
    fn test_format_tokens() {
        assert_eq!(format_tokens(500), "500");
        assert_eq!(format_tokens(1_000), "1.00K");
        assert_eq!(format_tokens(1_500), "1.50K");
        assert_eq!(format_tokens(1_000_000), "1.00M");
        assert_eq!(format_tokens(2_500_000), "2.50M");
    }
}

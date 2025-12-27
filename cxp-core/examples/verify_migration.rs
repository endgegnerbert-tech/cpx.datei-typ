//! Verification script to check migrated data in CXP file
//!
//! Run with: cargo run --features contextai --example verify_migration

use cxp_core::{CxpReader, contextai::ContextAIExtension};
use std::collections::HashMap;

fn main() -> anyhow::Result<()> {
    let cxp_path = "test_output.cxp";

    println!("Opening CXP file: {}", cxp_path);
    let reader = CxpReader::open(cxp_path)?;

    println!("\nExtensions found: {:?}", reader.list_extensions());

    // Read ContextAI extension data
    let mut extension_data: HashMap<String, Vec<u8>> = HashMap::new();

    let keys = reader.list_extension_keys("contextai");
    println!("\nContextAI extension keys: {:?}", keys);

    for key in keys {
        let data = reader.read_extension("contextai", &key)?;
        extension_data.insert(key, data);
    }

    // Reconstruct ContextAI extension
    let contextai = ContextAIExtension::from_extension_data(extension_data)?;

    // Verify conversations
    let conversations = contextai.list_conversations();
    println!("\n=== Conversations ===");
    println!("Total: {}", conversations.len());
    for conv in conversations {
        println!("\nConversation ID: {}", conv.id);
        println!("  Title: {}", conv.title);
        println!("  Created: {}", conv.created_at);
        println!("  Messages: {}", conv.messages.len());
        for msg in &conv.messages {
            println!("    [{:9}] {}", msg.role, msg.content.chars().take(50).collect::<String>());
        }
    }

    // Verify user habits
    let habits = contextai.list_habits();
    println!("\n=== User Habits ===");
    println!("Total: {}", habits.len());
    for habit in habits {
        println!("  {} = {} (confidence: {})", habit.habit_key, habit.habit_value, habit.confidence);
    }

    // Verify watched folders
    let folders = contextai.get_watched_folders();
    println!("\n=== Watched Folders ===");
    println!("Total: {}", folders.len());
    for folder in folders {
        println!("  {} (enabled: {})", folder.path, folder.enabled);
    }

    // Verify app settings
    let settings = contextai.get_settings();
    println!("\n=== App Settings ===");
    println!("Theme: {}", settings.theme);
    println!("Auto Index: {}", settings.auto_index);
    println!("Max Context Files: {}", settings.max_context_files);

    // Verify dictionary
    let dictionary = contextai.list_dictionary();
    println!("\n=== Custom Dictionary ===");
    println!("Total: {}", dictionary.len());
    for entry in dictionary {
        println!("  - {}: {}", entry.term, entry.definition);
    }

    println!("\n✅ Verification completed successfully!");

    Ok(())
}

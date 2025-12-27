//! Example usage of the ContextAI extension
//!
//! This example demonstrates how to use the ContextAI extension
//! to store app-specific data in a CXP file.
//!
//! Run with:
//! ```bash
//! cargo run --example contextai_example --features contextai
//! ```

#[cfg(feature = "contextai")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    use cxp_core::contextai::{
        ContextAIExtension, Conversation, ChatMessage, UserHabit,
        WatchedFolder, AppSettings, DictionaryEntry,
    };

    println!("=== ContextAI Extension Example ===\n");

    // Create a new ContextAI extension
    let mut ext = ContextAIExtension::new();
    println!("Created new ContextAI extension");

    // ============================================================
    // 1. Manage Conversations
    // ============================================================
    println!("\n--- Managing Conversations ---");

    let conv = Conversation {
        id: "conv-123".to_string(),
        title: "Project Planning Discussion".to_string(),
        created_at: "2025-01-15T10:00:00Z".to_string(),
        updated_at: "2025-01-15T10:00:00Z".to_string(),
        messages: vec![],
    };

    ext.add_conversation(conv);
    println!("Added conversation: {}", ext.get_conversation("conv-123").unwrap().title);

    // Add messages to the conversation
    let message1 = ChatMessage {
        id: "msg-1".to_string(),
        role: "user".to_string(),
        content: "What should we build next?".to_string(),
        timestamp: "2025-01-15T10:01:00Z".to_string(),
        referenced_files: vec![],
    };

    let message2 = ChatMessage {
        id: "msg-2".to_string(),
        role: "assistant".to_string(),
        content: "Based on your codebase, I suggest focusing on...".to_string(),
        timestamp: "2025-01-15T10:01:30Z".to_string(),
        referenced_files: vec![],
    };

    ext.add_message("conv-123", message1)?;
    ext.add_message("conv-123", message2)?;

    let conversation = ext.get_conversation("conv-123").unwrap();
    println!("Conversation has {} messages", conversation.messages.len());

    // ============================================================
    // 2. Set User Habits
    // ============================================================
    println!("\n--- Setting User Habits ---");

    let habit1 = UserHabit {
        id: "habit-1".to_string(),
        habit_key: "preferred_language".to_string(),
        habit_value: "de".to_string(),
        confidence: 0.9,
        updated_at: "2025-01-15T10:00:00Z".to_string(),
        learned_from_message_id: None,
    };

    let habit2 = UserHabit {
        id: "habit-2".to_string(),
        habit_key: "coding_style".to_string(),
        habit_value: "tabs, no-semicolons".to_string(),
        confidence: 0.85,
        updated_at: "2025-01-15T10:00:00Z".to_string(),
        learned_from_message_id: None,
    };

    ext.set_habit(habit1);
    ext.set_habit(habit2);
    println!("Set {} habits", ext.list_habits().len());
    if let Some(lang_habit) = ext.get_habit("preferred_language") {
        println!("User prefers language: {} (confidence: {})", lang_habit.habit_value, lang_habit.confidence);
    }

    // ============================================================
    // 3. Configure Settings
    // ============================================================
    println!("\n--- Configuring Settings ---");

    let settings = AppSettings {
        theme: "dark".to_string(),
        auto_index: true,
        max_context_files: 100,
    };

    ext.set_settings(settings);
    println!("Theme: {}", ext.get_settings().theme);
    println!("Auto-index: {}", ext.get_settings().auto_index);
    println!("Max context files: {}", ext.get_settings().max_context_files);

    // ============================================================
    // 4. Add Watched Folders
    // ============================================================
    println!("\n--- Adding Watched Folders ---");

    let folder1 = WatchedFolder {
        path: "/home/user/projects/app".to_string(),
        enabled: true,
        last_scan: Some("2025-01-15T09:00:00Z".to_string()),
    };

    let folder2 = WatchedFolder {
        path: "/home/user/projects/lib".to_string(),
        enabled: true,
        last_scan: None,
    };

    ext.add_watched_folder(folder1);
    ext.add_watched_folder(folder2);
    println!("Watching {} folders", ext.get_watched_folders().len());

    // ============================================================
    // 5. Manage Dictionary
    // ============================================================
    println!("\n--- Managing Dictionary ---");

    let dict1 = DictionaryEntry {
        id: "dict-1".to_string(),
        term: "TypeScript".to_string(),
        definition: "A typed superset of JavaScript".to_string(),
        category: Some("Programming".to_string()),
        learned_from_message_id: None,
        created_at: "2025-01-15T10:00:00Z".to_string(),
        updated_at: "2025-01-15T10:00:00Z".to_string(),
    };

    let dict2 = DictionaryEntry {
        id: "dict-2".to_string(),
        term: "Rust".to_string(),
        definition: "A systems programming language".to_string(),
        category: Some("Programming".to_string()),
        learned_from_message_id: None,
        created_at: "2025-01-15T10:00:00Z".to_string(),
        updated_at: "2025-01-15T10:00:00Z".to_string(),
    };

    ext.add_dictionary_entry(dict1);
    ext.add_dictionary_entry(dict2);

    println!("Dictionary has {} entries", ext.list_dictionary().len());
    for entry in ext.list_dictionary() {
        println!("  - {}: {}", entry.term, entry.definition);
    }

    // ============================================================
    // 6. Serialize to Extension Data
    // ============================================================
    println!("\n--- Serialization ---");

    // Convert to extension data (as would be stored in CXP file)
    let extension_data = ext.to_extension_data()?;
    println!("Serialized to {} files:", extension_data.len());
    for (name, data) in &extension_data {
        println!("  - {} ({} bytes)", name, data.len());
    }

    // Deserialize back
    let restored = ContextAIExtension::from_extension_data(extension_data)?;
    println!("\nRestored extension:");
    println!("  - {} conversations", restored.list_conversations().len());
    println!("  - {} watched folders", restored.get_watched_folders().len());
    println!("  - {} dictionary entries", restored.list_dictionary().len());
    println!("  - {} habits", restored.list_habits().len());
    println!("  - Theme: {}", restored.get_settings().theme);

    // ============================================================
    // 7. JSON Export (for debugging)
    // ============================================================
    println!("\n--- JSON Export ---");

    let json = ext.to_json()?;
    println!("JSON export (first 200 chars):");
    println!("{}", &json[..200.min(json.len())]);
    println!("...");

    println!("\n=== Example Complete ===");

    Ok(())
}

#[cfg(not(feature = "contextai"))]
fn main() {
    eprintln!("This example requires the 'contextai' feature.");
    eprintln!("Run with: cargo run --example contextai_example --features contextai");
    std::process::exit(1);
}

//! Example of integrating extensions with CXP files
//!
//! This example demonstrates how to:
//! 1. Create a CXP file with source code
//! 2. Add ContextAI extension data to it
//! 3. Read the extension data back from the file
//!
//! Run with:
//! ```bash
//! cargo run --example extension_integration --features contextai
//! ```

#[cfg(feature = "contextai")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    use cxp_core::{CxpBuilder, CxpReader};
    use cxp_core::contextai::{
        ContextAIExtension, Conversation, ChatMessage, UserHabit,
    };
    use std::fs;

    println!("=== CXP Extension Integration Example ===\n");

    // Create a temporary directory with some test files
    let temp_dir = std::env::temp_dir().join("cxp_extension_test");
    fs::create_dir_all(&temp_dir)?;

    // Create some test source files
    let test_file = temp_dir.join("example.rs");
    fs::write(&test_file, r#"
        fn hello_world() {
            println!("Hello from CXP!");
        }
    "#)?;

    println!("1. Created test source files in: {:?}", temp_dir);

    // ============================================================
    // PART 1: Create CXP file with extension data
    // ============================================================
    println!("\n2. Building CXP file with ContextAI extension...");

    let output_path = temp_dir.join("example_with_extension.cxp");

    // Create a ContextAI extension with some data
    let mut contextai_ext = ContextAIExtension::new();

    // Add a conversation
    let conv = Conversation {
        id: "conv-001".to_string(),
        title: "CXP Extension Discussion".to_string(),
        created_at: "2025-01-15T10:00:00Z".to_string(),
        updated_at: "2025-01-15T10:00:00Z".to_string(),
        messages: vec![
            ChatMessage {
                id: "msg-1".to_string(),
                role: "user".to_string(),
                content: "How does the extension system work?".to_string(),
                timestamp: "2025-01-15T10:01:00Z".to_string(),
                referenced_files: vec![],
            },
            ChatMessage {
                id: "msg-2".to_string(),
                role: "assistant".to_string(),
                content: "Extensions are stored in the extensions/ directory in the CXP file.".to_string(),
                timestamp: "2025-01-15T10:01:30Z".to_string(),
                referenced_files: vec![],
            },
        ],
    };
    contextai_ext.add_conversation(conv);

    // Set user habits
    let habit = UserHabit {
        id: "habit-1".to_string(),
        habit_key: "preferred_language".to_string(),
        habit_value: "en".to_string(),
        confidence: 1.0,
        updated_at: "2025-01-15T10:00:00Z".to_string(),
        learned_from_message_id: None,
    };
    contextai_ext.set_habit(habit);

    // Build the CXP file
    let mut builder = CxpBuilder::new(&temp_dir);
    builder
        .scan()?
        .process()?;

    // Add the extension data
    let extension_data = contextai_ext.to_extension_data()?;
    builder.add_extension(&contextai_ext, extension_data)?;

    // Write the CXP file
    builder.build(&output_path)?;

    println!("   ✓ CXP file created: {:?}", output_path);
    let file_size = fs::metadata(&output_path)?.len();
    println!("   ✓ File size: {} bytes", file_size);

    // ============================================================
    // PART 2: Read the CXP file and extract extension data
    // ============================================================
    println!("\n3. Reading CXP file and extracting extension data...");

    let reader = CxpReader::open(&output_path)?;

    // List all extensions in the file
    let extensions = reader.list_extensions();
    println!("   ✓ Found {} extension(s):", extensions.len());
    for ext in &extensions {
        println!("     - {}", ext);

        // Get extension manifest
        if let Some(manifest) = reader.get_extension_manifest(ext) {
            println!("       Version: {}", manifest.version);
            if let Some(desc) = &manifest.description {
                println!("       Description: {}", desc);
            }
        }

        // List all data keys in the extension
        let keys = reader.list_extension_keys(ext);
        println!("       Data files: {}", keys.len());
        for key in &keys {
            println!("         - {}", key);
        }
    }

    // Read the ContextAI extension data
    if extensions.contains(&"contextai".to_string()) {
        println!("\n4. Reconstructing ContextAI extension from CXP file...");

        // Read all extension data
        let mut extension_data = std::collections::HashMap::new();
        for key in reader.list_extension_keys("contextai") {
            let data = reader.read_extension("contextai", &key)?;
            extension_data.insert(key, data);
        }

        // Reconstruct the extension
        let restored_ext = ContextAIExtension::from_extension_data(extension_data)?;

        println!("   ✓ ContextAI extension restored:");
        println!("     - Conversations: {}", restored_ext.list_conversations().len());

        for conv in restored_ext.list_conversations() {
            println!("       • {} (ID: {})", conv.title, conv.id);
            println!("         Messages: {}", conv.messages.len());
        }

        println!("     - Habits:");
        println!("       • Total habits: {}", restored_ext.list_habits().len());
        if let Some(lang_habit) = restored_ext.get_habit("preferred_language") {
            println!("       • Language: {}", lang_habit.habit_value);
        }

        println!("     - Settings:");
        println!("       • Theme: {}", restored_ext.get_settings().theme);
        println!("       • Auto-index: {}", restored_ext.get_settings().auto_index);
    }

    // ============================================================
    // PART 3: Show manifest information
    // ============================================================
    println!("\n5. CXP Manifest Information:");
    println!("   • Total files: {}", reader.manifest().stats.total_files);
    println!("   • Unique chunks: {}", reader.manifest().stats.unique_chunks);
    println!("   • Extensions: {}", reader.manifest().extensions.join(", "));

    // Cleanup
    println!("\n6. Cleaning up temporary files...");
    fs::remove_dir_all(&temp_dir)?;
    println!("   ✓ Cleanup complete");

    println!("\n=== Example Complete ===");
    println!("\nKey takeaways:");
    println!("• Extensions are stored in extensions/{{namespace}}/ directories");
    println!("• Each extension has a manifest.msgpack with metadata");
    println!("• Extension data is serialized with MessagePack for efficiency");
    println!("• CxpReader provides easy access to extension data");

    Ok(())
}

#[cfg(not(feature = "contextai"))]
fn main() {
    eprintln!("This example requires the 'contextai' feature.");
    eprintln!("Run with: cargo run --example extension_integration --features contextai");
    std::process::exit(1);
}

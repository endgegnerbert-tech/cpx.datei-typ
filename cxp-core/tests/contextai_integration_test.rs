//! Integration test for ContextAI extension with CXP files
//!
//! This test demonstrates how to store and retrieve ContextAI data
//! in CXP files using the extensions mechanism.

#![cfg(feature = "contextai")]

use cxp_core::contextai::{
    ContextAIExtension, Conversation, ChatMessage, UserHabits, AppSettings, WatchedFolder,
};

#[test]
fn test_contextai_roundtrip_serialization() {
    // Create a new extension with data
    let mut ext = ContextAIExtension::new();

    // Add a conversation with messages
    let conv = Conversation {
        id: "conv-test-1".to_string(),
        title: "Integration Test Conversation".to_string(),
        created_at: "2025-01-15T10:00:00Z".to_string(),
        updated_at: "2025-01-15T10:30:00Z".to_string(),
        messages: vec![
            ChatMessage {
                id: "msg-1".to_string(),
                role: "user".to_string(),
                content: "Hello, AI!".to_string(),
                timestamp: "2025-01-15T10:00:00Z".to_string(),
            },
            ChatMessage {
                id: "msg-2".to_string(),
                role: "assistant".to_string(),
                content: "Hello! How can I help you?".to_string(),
                timestamp: "2025-01-15T10:00:30Z".to_string(),
            },
        ],
    };
    ext.add_conversation(conv);

    // Set user habits
    ext.set_habits(UserHabits {
        preferred_language: "de".to_string(),
        coding_style: Some("tabs".to_string()),
        custom_instructions: vec!["Use TypeScript".to_string(), "Test everything".to_string()],
    });

    // Set app settings
    ext.set_settings(AppSettings {
        theme: "dark".to_string(),
        auto_index: true,
        max_context_files: 100,
    });

    // Add watched folders
    ext.add_watched_folder(WatchedFolder {
        path: "/home/user/projects".to_string(),
        enabled: true,
        last_scan: Some("2025-01-15T09:00:00Z".to_string()),
    });

    // Add dictionary words
    ext.add_to_dictionary("Rust".to_string());
    ext.add_to_dictionary("CXP".to_string());

    // Serialize to extension data
    let extension_data = ext.to_extension_data().expect("Failed to serialize");

    // Verify we have all expected files
    assert!(extension_data.contains_key("conversations.msgpack"));
    assert!(extension_data.contains_key("habits.msgpack"));
    assert!(extension_data.contains_key("settings.msgpack"));
    assert!(extension_data.contains_key("watched_folders.msgpack"));
    assert!(extension_data.contains_key("dictionary.msgpack"));

    // Deserialize back
    let restored = ContextAIExtension::from_extension_data(extension_data)
        .expect("Failed to deserialize");

    // Verify conversations
    assert_eq!(restored.list_conversations().len(), 1);
    let conv = restored.get_conversation("conv-test-1").expect("Conversation not found");
    assert_eq!(conv.title, "Integration Test Conversation");
    assert_eq!(conv.messages.len(), 2);
    assert_eq!(conv.messages[0].content, "Hello, AI!");
    assert_eq!(conv.messages[1].role, "assistant");

    // Verify habits
    let habits = restored.get_habits();
    assert_eq!(habits.preferred_language, "de");
    assert_eq!(habits.coding_style, Some("tabs".to_string()));
    assert_eq!(habits.custom_instructions.len(), 2);

    // Verify settings
    let settings = restored.get_settings();
    assert_eq!(settings.theme, "dark");
    assert_eq!(settings.auto_index, true);
    assert_eq!(settings.max_context_files, 100);

    // Verify watched folders
    assert_eq!(restored.get_watched_folders().len(), 1);
    assert_eq!(restored.get_watched_folders()[0].path, "/home/user/projects");

    // Verify dictionary
    assert_eq!(restored.get_dictionary().len(), 2);
    assert!(restored.get_dictionary().contains(&"Rust".to_string()));
    assert!(restored.get_dictionary().contains(&"CXP".to_string()));
}

#[test]
fn test_contextai_json_roundtrip() {
    let mut ext = ContextAIExtension::new();

    // Add some data
    ext.add_conversation(Conversation {
        id: "conv-1".to_string(),
        title: "Test".to_string(),
        created_at: "2025-01-15T10:00:00Z".to_string(),
        updated_at: "2025-01-15T10:00:00Z".to_string(),
        messages: vec![],
    });

    ext.add_to_dictionary("test".to_string());

    // Export to JSON
    let json = ext.to_json().expect("Failed to export to JSON");
    assert!(!json.is_empty());
    assert!(json.contains("conv-1"));
    assert!(json.contains("test"));

    // Import from JSON
    let restored = ContextAIExtension::from_json(&json).expect("Failed to import from JSON");
    assert_eq!(restored.list_conversations().len(), 1);
    assert_eq!(restored.get_dictionary().len(), 1);
}

#[test]
fn test_contextai_partial_data() {
    use std::collections::HashMap;

    // Create extension data with only conversations
    let mut ext = ContextAIExtension::new();
    ext.add_conversation(Conversation {
        id: "conv-1".to_string(),
        title: "Partial Test".to_string(),
        created_at: "2025-01-15T10:00:00Z".to_string(),
        updated_at: "2025-01-15T10:00:00Z".to_string(),
        messages: vec![],
    });

    let mut partial_data = HashMap::new();
    let conversations = ext.list_conversations().to_vec();
    let conv_data = rmp_serde::to_vec(&conversations).unwrap();
    partial_data.insert("conversations.msgpack".to_string(), conv_data);

    // Should successfully deserialize with defaults for missing data
    let restored = ContextAIExtension::from_extension_data(partial_data)
        .expect("Failed to deserialize partial data");

    assert_eq!(restored.list_conversations().len(), 1);
    assert_eq!(restored.get_habits().preferred_language, "en"); // default
    assert_eq!(restored.get_settings().theme, "auto"); // default
    assert_eq!(restored.get_watched_folders().len(), 0); // empty
    assert_eq!(restored.get_dictionary().len(), 0); // empty
}

#[test]
fn test_contextai_conversation_operations() {
    let mut ext = ContextAIExtension::new();

    // Add conversation
    let conv = Conversation {
        id: "conv-ops".to_string(),
        title: "Operations Test".to_string(),
        created_at: "2025-01-15T10:00:00Z".to_string(),
        updated_at: "2025-01-15T10:00:00Z".to_string(),
        messages: vec![],
    };
    ext.add_conversation(conv.clone());

    // Update conversation
    let mut updated = conv.clone();
    updated.title = "Updated Title".to_string();
    ext.update_conversation("conv-ops", updated).expect("Failed to update");

    let retrieved = ext.get_conversation("conv-ops").expect("Conversation not found");
    assert_eq!(retrieved.title, "Updated Title");

    // Add messages
    ext.add_message(
        "conv-ops",
        ChatMessage {
            id: "msg-1".to_string(),
            role: "user".to_string(),
            content: "Test message".to_string(),
            timestamp: "2025-01-15T10:01:00Z".to_string(),
        },
    )
    .expect("Failed to add message");

    let conv_with_msg = ext.get_conversation("conv-ops").expect("Conversation not found");
    assert_eq!(conv_with_msg.messages.len(), 1);

    // Delete conversation
    ext.delete_conversation("conv-ops").expect("Failed to delete");
    assert!(ext.get_conversation("conv-ops").is_none());
}

#[test]
fn test_contextai_watched_folders_operations() {
    let mut ext = ContextAIExtension::new();

    // Add multiple folders
    ext.add_watched_folder(WatchedFolder {
        path: "/path/1".to_string(),
        enabled: true,
        last_scan: None,
    });

    ext.add_watched_folder(WatchedFolder {
        path: "/path/2".to_string(),
        enabled: false,
        last_scan: Some("2025-01-15T10:00:00Z".to_string()),
    });

    assert_eq!(ext.get_watched_folders().len(), 2);

    // Remove a folder
    ext.remove_watched_folder("/path/1").expect("Failed to remove folder");
    assert_eq!(ext.get_watched_folders().len(), 1);
    assert_eq!(ext.get_watched_folders()[0].path, "/path/2");

    // Try to remove non-existent folder
    let result = ext.remove_watched_folder("/path/nonexistent");
    assert!(result.is_err());
}

#[test]
fn test_contextai_dictionary_operations() {
    let mut ext = ContextAIExtension::new();

    // Add words
    ext.add_to_dictionary("Word1".to_string());
    ext.add_to_dictionary("Word2".to_string());
    ext.add_to_dictionary("Word3".to_string());

    assert_eq!(ext.get_dictionary().len(), 3);

    // Adding duplicate should not increase count
    ext.add_to_dictionary("Word1".to_string());
    assert_eq!(ext.get_dictionary().len(), 3);

    // Remove a word
    ext.remove_from_dictionary("Word2").expect("Failed to remove word");
    assert_eq!(ext.get_dictionary().len(), 2);
    assert!(!ext.get_dictionary().contains(&"Word2".to_string()));

    // Try to remove non-existent word
    let result = ext.remove_from_dictionary("NonExistent");
    assert!(result.is_err());
}

#[test]
fn test_contextai_empty_extension() {
    // Create empty extension
    let ext = ContextAIExtension::new();

    // Serialize and deserialize
    let data = ext.to_extension_data().expect("Failed to serialize");
    let restored = ContextAIExtension::from_extension_data(data)
        .expect("Failed to deserialize");

    // Verify defaults
    assert_eq!(restored.list_conversations().len(), 0);
    assert_eq!(restored.get_habits().preferred_language, "en");
    assert_eq!(restored.get_settings().auto_index, true);
    assert_eq!(restored.get_watched_folders().len(), 0);
    assert_eq!(restored.get_dictionary().len(), 0);
}

#[test]
fn test_contextai_large_conversation() {
    let mut ext = ContextAIExtension::new();

    // Create a conversation with many messages
    let mut messages = Vec::new();
    for i in 0..100 {
        messages.push(ChatMessage {
            id: format!("msg-{}", i),
            role: if i % 2 == 0 { "user" } else { "assistant" }.to_string(),
            content: format!("Message content {}", i),
            timestamp: format!("2025-01-15T10:{:02}:00Z", i),
        });
    }

    let conv = Conversation {
        id: "large-conv".to_string(),
        title: "Large Conversation".to_string(),
        created_at: "2025-01-15T10:00:00Z".to_string(),
        updated_at: "2025-01-15T10:00:00Z".to_string(),
        messages,
    };

    ext.add_conversation(conv);

    // Serialize and deserialize
    let data = ext.to_extension_data().expect("Failed to serialize");
    let restored = ContextAIExtension::from_extension_data(data)
        .expect("Failed to deserialize");

    // Verify
    let conv = restored.get_conversation("large-conv").expect("Conversation not found");
    assert_eq!(conv.messages.len(), 100);
    assert_eq!(conv.messages[99].content, "Message content 99");
}

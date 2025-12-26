//! ContextAI Extension for CXP
//!
//! This module provides application-specific data structures for the ContextAI app.
//! Instead of using SQLite, all data is stored in the CXP file's extensions/ directory.

#[cfg(feature = "contextai")]
use serde::{Deserialize, Serialize};
#[cfg(feature = "contextai")]
use std::collections::HashMap;
#[cfg(feature = "contextai")]
use crate::{Result, CxpError};

/// A conversation in ContextAI
#[cfg(feature = "contextai")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conversation {
    /// Unique conversation ID
    pub id: String,
    /// Conversation title
    pub title: String,
    /// When the conversation was created
    pub created_at: String,
    /// When the conversation was last updated
    pub updated_at: String,
    /// All messages in this conversation
    pub messages: Vec<ChatMessage>,
}

/// A single chat message
#[cfg(feature = "contextai")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    /// Unique message ID
    pub id: String,
    /// Role: "user" or "assistant"
    pub role: String,
    /// Message content
    pub content: String,
    /// When the message was sent
    pub timestamp: String,
}

/// User habits and preferences
#[cfg(feature = "contextai")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserHabits {
    /// Preferred language (e.g., "en", "de")
    pub preferred_language: String,
    /// Coding style preferences (optional)
    pub coding_style: Option<String>,
    /// Custom instructions from the user
    pub custom_instructions: Vec<String>,
}

/// A watched folder for automatic indexing
#[cfg(feature = "contextai")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WatchedFolder {
    /// Folder path
    pub path: String,
    /// Whether watching is enabled
    pub enabled: bool,
    /// Last time the folder was scanned
    pub last_scan: Option<String>,
}

/// Application settings
#[cfg(feature = "contextai")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    /// UI theme (e.g., "light", "dark", "auto")
    pub theme: String,
    /// Whether to automatically index new files
    pub auto_index: bool,
    /// Maximum number of files to include in context
    pub max_context_files: u32,
}

/// The ContextAI extension manager
#[cfg(feature = "contextai")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextAIExtension {
    /// All conversations
    conversations: Vec<Conversation>,
    /// User habits and preferences
    habits: UserHabits,
    /// Watched folders
    watched_folders: Vec<WatchedFolder>,
    /// App settings
    settings: AppSettings,
    /// User dictionary (for custom words/terms)
    dictionary: Vec<String>,
}

#[cfg(feature = "contextai")]
impl ContextAIExtension {
    /// Create a new ContextAI extension with default values
    pub fn new() -> Self {
        Self {
            conversations: Vec::new(),
            habits: UserHabits {
                preferred_language: "en".to_string(),
                coding_style: None,
                custom_instructions: Vec::new(),
            },
            watched_folders: Vec::new(),
            settings: AppSettings {
                theme: "auto".to_string(),
                auto_index: true,
                max_context_files: 50,
            },
            dictionary: Vec::new(),
        }
    }

    // ============================================================
    // Conversation Management
    // ============================================================

    /// Add a new conversation
    pub fn add_conversation(&mut self, conv: Conversation) {
        self.conversations.push(conv);
    }

    /// Get a conversation by ID
    pub fn get_conversation(&self, id: &str) -> Option<&Conversation> {
        self.conversations.iter().find(|c| c.id == id)
    }

    /// Get a mutable reference to a conversation by ID
    pub fn get_conversation_mut(&mut self, id: &str) -> Option<&mut Conversation> {
        self.conversations.iter_mut().find(|c| c.id == id)
    }

    /// List all conversations
    pub fn list_conversations(&self) -> &[Conversation] {
        &self.conversations
    }

    /// Update a conversation
    pub fn update_conversation(&mut self, id: &str, updated: Conversation) -> Result<()> {
        if let Some(conv) = self.conversations.iter_mut().find(|c| c.id == id) {
            *conv = updated;
            Ok(())
        } else {
            Err(CxpError::FileNotFound(format!("Conversation not found: {}", id)))
        }
    }

    /// Delete a conversation
    pub fn delete_conversation(&mut self, id: &str) -> Result<()> {
        if let Some(pos) = self.conversations.iter().position(|c| c.id == id) {
            self.conversations.remove(pos);
            Ok(())
        } else {
            Err(CxpError::FileNotFound(format!("Conversation not found: {}", id)))
        }
    }

    /// Add a message to a conversation
    pub fn add_message(&mut self, conversation_id: &str, message: ChatMessage) -> Result<()> {
        if let Some(conv) = self.get_conversation_mut(conversation_id) {
            conv.messages.push(message);
            Ok(())
        } else {
            Err(CxpError::FileNotFound(format!("Conversation not found: {}", conversation_id)))
        }
    }

    // ============================================================
    // Habits Management
    // ============================================================

    /// Set user habits
    pub fn set_habits(&mut self, habits: UserHabits) {
        self.habits = habits;
    }

    /// Get user habits
    pub fn get_habits(&self) -> &UserHabits {
        &self.habits
    }

    /// Get mutable user habits
    pub fn get_habits_mut(&mut self) -> &mut UserHabits {
        &mut self.habits
    }

    // ============================================================
    // Settings Management
    // ============================================================

    /// Set app settings
    pub fn set_settings(&mut self, settings: AppSettings) {
        self.settings = settings;
    }

    /// Get app settings
    pub fn get_settings(&self) -> &AppSettings {
        &self.settings
    }

    /// Get mutable app settings
    pub fn get_settings_mut(&mut self) -> &mut AppSettings {
        &mut self.settings
    }

    // ============================================================
    // Watched Folders Management
    // ============================================================

    /// Add a watched folder
    pub fn add_watched_folder(&mut self, folder: WatchedFolder) {
        self.watched_folders.push(folder);
    }

    /// Get all watched folders
    pub fn get_watched_folders(&self) -> &[WatchedFolder] {
        &self.watched_folders
    }

    /// Get a mutable reference to all watched folders
    pub fn get_watched_folders_mut(&mut self) -> &mut Vec<WatchedFolder> {
        &mut self.watched_folders
    }

    /// Remove a watched folder by path
    pub fn remove_watched_folder(&mut self, path: &str) -> Result<()> {
        if let Some(pos) = self.watched_folders.iter().position(|f| f.path == path) {
            self.watched_folders.remove(pos);
            Ok(())
        } else {
            Err(CxpError::FileNotFound(format!("Watched folder not found: {}", path)))
        }
    }

    // ============================================================
    // Dictionary Management
    // ============================================================

    /// Add a word to the dictionary
    pub fn add_to_dictionary(&mut self, word: String) {
        if !self.dictionary.contains(&word) {
            self.dictionary.push(word);
        }
    }

    /// Get the dictionary
    pub fn get_dictionary(&self) -> &[String] {
        &self.dictionary
    }

    /// Remove a word from the dictionary
    pub fn remove_from_dictionary(&mut self, word: &str) -> Result<()> {
        if let Some(pos) = self.dictionary.iter().position(|w| w == word) {
            self.dictionary.remove(pos);
            Ok(())
        } else {
            Err(CxpError::FileNotFound(format!("Word not found in dictionary: {}", word)))
        }
    }

    // ============================================================
    // Serialization for CXP Extensions
    // ============================================================

    /// Convert to extension data (HashMap of file name -> bytes)
    /// This can be stored in the CXP file's extensions/ directory
    pub fn to_extension_data(&self) -> Result<HashMap<String, Vec<u8>>> {
        let mut data = HashMap::new();

        // Serialize conversations
        let conversations_data = rmp_serde::to_vec(&self.conversations)
            .map_err(|e| CxpError::Serialization(e.to_string()))?;
        data.insert("conversations.msgpack".to_string(), conversations_data);

        // Serialize habits
        let habits_data = rmp_serde::to_vec(&self.habits)
            .map_err(|e| CxpError::Serialization(e.to_string()))?;
        data.insert("habits.msgpack".to_string(), habits_data);

        // Serialize watched folders
        let folders_data = rmp_serde::to_vec(&self.watched_folders)
            .map_err(|e| CxpError::Serialization(e.to_string()))?;
        data.insert("watched_folders.msgpack".to_string(), folders_data);

        // Serialize settings
        let settings_data = rmp_serde::to_vec(&self.settings)
            .map_err(|e| CxpError::Serialization(e.to_string()))?;
        data.insert("settings.msgpack".to_string(), settings_data);

        // Serialize dictionary
        let dictionary_data = rmp_serde::to_vec(&self.dictionary)
            .map_err(|e| CxpError::Serialization(e.to_string()))?;
        data.insert("dictionary.msgpack".to_string(), dictionary_data);

        Ok(data)
    }

    /// Create from extension data (HashMap of file name -> bytes)
    /// This reads data from the CXP file's extensions/ directory
    pub fn from_extension_data(data: HashMap<String, Vec<u8>>) -> Result<Self> {
        // Deserialize conversations
        let conversations = if let Some(conv_data) = data.get("conversations.msgpack") {
            rmp_serde::from_slice(conv_data)
                .map_err(|e| CxpError::Serialization(e.to_string()))?
        } else {
            Vec::new()
        };

        // Deserialize habits
        let habits = if let Some(habits_data) = data.get("habits.msgpack") {
            rmp_serde::from_slice(habits_data)
                .map_err(|e| CxpError::Serialization(e.to_string()))?
        } else {
            UserHabits {
                preferred_language: "en".to_string(),
                coding_style: None,
                custom_instructions: Vec::new(),
            }
        };

        // Deserialize watched folders
        let watched_folders = if let Some(folders_data) = data.get("watched_folders.msgpack") {
            rmp_serde::from_slice(folders_data)
                .map_err(|e| CxpError::Serialization(e.to_string()))?
        } else {
            Vec::new()
        };

        // Deserialize settings
        let settings = if let Some(settings_data) = data.get("settings.msgpack") {
            rmp_serde::from_slice(settings_data)
                .map_err(|e| CxpError::Serialization(e.to_string()))?
        } else {
            AppSettings {
                theme: "auto".to_string(),
                auto_index: true,
                max_context_files: 50,
            }
        };

        // Deserialize dictionary
        let dictionary = if let Some(dict_data) = data.get("dictionary.msgpack") {
            rmp_serde::from_slice(dict_data)
                .map_err(|e| CxpError::Serialization(e.to_string()))?
        } else {
            Vec::new()
        };

        Ok(Self {
            conversations,
            habits,
            watched_folders,
            settings,
            dictionary,
        })
    }

    /// Serialize to JSON (for debugging/human reading)
    pub fn to_json(&self) -> Result<String> {
        serde_json::to_string_pretty(self)
            .map_err(|e| CxpError::Serialization(e.to_string()))
    }

    /// Deserialize from JSON
    pub fn from_json(json: &str) -> Result<Self> {
        serde_json::from_str(json)
            .map_err(|e| CxpError::Serialization(e.to_string()))
    }
}

#[cfg(feature = "contextai")]
impl Default for ContextAIExtension {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(all(test, feature = "contextai"))]
mod tests {
    use super::*;

    #[test]
    fn test_contextai_creation() {
        let ext = ContextAIExtension::new();
        assert_eq!(ext.conversations.len(), 0);
        assert_eq!(ext.habits.preferred_language, "en");
        assert_eq!(ext.settings.theme, "auto");
    }

    #[test]
    fn test_conversation_management() {
        let mut ext = ContextAIExtension::new();

        let conv = Conversation {
            id: "conv-1".to_string(),
            title: "Test Conversation".to_string(),
            created_at: "2025-01-01T00:00:00Z".to_string(),
            updated_at: "2025-01-01T00:00:00Z".to_string(),
            messages: vec![],
        };

        ext.add_conversation(conv.clone());
        assert_eq!(ext.list_conversations().len(), 1);

        let retrieved = ext.get_conversation("conv-1").unwrap();
        assert_eq!(retrieved.title, "Test Conversation");

        ext.delete_conversation("conv-1").unwrap();
        assert_eq!(ext.list_conversations().len(), 0);
    }

    #[test]
    fn test_message_management() {
        let mut ext = ContextAIExtension::new();

        let conv = Conversation {
            id: "conv-1".to_string(),
            title: "Test Conversation".to_string(),
            created_at: "2025-01-01T00:00:00Z".to_string(),
            updated_at: "2025-01-01T00:00:00Z".to_string(),
            messages: vec![],
        };

        ext.add_conversation(conv);

        let message = ChatMessage {
            id: "msg-1".to_string(),
            role: "user".to_string(),
            content: "Hello!".to_string(),
            timestamp: "2025-01-01T00:00:01Z".to_string(),
        };

        ext.add_message("conv-1", message).unwrap();

        let conv = ext.get_conversation("conv-1").unwrap();
        assert_eq!(conv.messages.len(), 1);
        assert_eq!(conv.messages[0].content, "Hello!");
    }

    #[test]
    fn test_habits_management() {
        let mut ext = ContextAIExtension::new();

        let habits = UserHabits {
            preferred_language: "de".to_string(),
            coding_style: Some("4-space-indent".to_string()),
            custom_instructions: vec!["Be concise".to_string()],
        };

        ext.set_habits(habits);

        let retrieved = ext.get_habits();
        assert_eq!(retrieved.preferred_language, "de");
        assert_eq!(retrieved.coding_style, Some("4-space-indent".to_string()));
    }

    #[test]
    fn test_settings_management() {
        let mut ext = ContextAIExtension::new();

        let settings = AppSettings {
            theme: "dark".to_string(),
            auto_index: false,
            max_context_files: 100,
        };

        ext.set_settings(settings);

        let retrieved = ext.get_settings();
        assert_eq!(retrieved.theme, "dark");
        assert_eq!(retrieved.auto_index, false);
        assert_eq!(retrieved.max_context_files, 100);
    }

    #[test]
    fn test_watched_folders() {
        let mut ext = ContextAIExtension::new();

        let folder = WatchedFolder {
            path: "/home/user/projects".to_string(),
            enabled: true,
            last_scan: Some("2025-01-01T00:00:00Z".to_string()),
        };

        ext.add_watched_folder(folder);
        assert_eq!(ext.get_watched_folders().len(), 1);

        ext.remove_watched_folder("/home/user/projects").unwrap();
        assert_eq!(ext.get_watched_folders().len(), 0);
    }

    #[test]
    fn test_dictionary() {
        let mut ext = ContextAIExtension::new();

        ext.add_to_dictionary("TypeScript".to_string());
        ext.add_to_dictionary("React".to_string());

        assert_eq!(ext.get_dictionary().len(), 2);
        assert!(ext.get_dictionary().contains(&"TypeScript".to_string()));

        ext.remove_from_dictionary("React").unwrap();
        assert_eq!(ext.get_dictionary().len(), 1);
    }

    #[test]
    fn test_serialization() {
        let mut ext = ContextAIExtension::new();

        let conv = Conversation {
            id: "conv-1".to_string(),
            title: "Test".to_string(),
            created_at: "2025-01-01T00:00:00Z".to_string(),
            updated_at: "2025-01-01T00:00:00Z".to_string(),
            messages: vec![],
        };

        ext.add_conversation(conv);
        ext.add_to_dictionary("test-word".to_string());

        // Test to_extension_data
        let data = ext.to_extension_data().unwrap();
        assert!(data.contains_key("conversations.msgpack"));
        assert!(data.contains_key("habits.msgpack"));
        assert!(data.contains_key("settings.msgpack"));
        assert!(data.contains_key("dictionary.msgpack"));

        // Test from_extension_data
        let restored = ContextAIExtension::from_extension_data(data).unwrap();
        assert_eq!(restored.conversations.len(), 1);
        assert_eq!(restored.conversations[0].id, "conv-1");
        assert_eq!(restored.dictionary.len(), 1);
        assert_eq!(restored.dictionary[0], "test-word");
    }

    #[test]
    fn test_json_serialization() {
        let ext = ContextAIExtension::new();

        let json = ext.to_json().unwrap();
        assert!(!json.is_empty());

        let restored = ContextAIExtension::from_json(&json).unwrap();
        assert_eq!(restored.habits.preferred_language, ext.habits.preferred_language);
        assert_eq!(restored.settings.theme, ext.settings.theme);
    }

    #[test]
    fn test_partial_deserialization() {
        // Test that we can deserialize even if some data is missing
        let mut data = HashMap::new();

        let conversations = vec![Conversation {
            id: "conv-1".to_string(),
            title: "Test".to_string(),
            created_at: "2025-01-01T00:00:00Z".to_string(),
            updated_at: "2025-01-01T00:00:00Z".to_string(),
            messages: vec![],
        }];

        let conv_data = rmp_serde::to_vec(&conversations).unwrap();
        data.insert("conversations.msgpack".to_string(), conv_data);

        // Only conversations are provided, others should use defaults
        let ext = ContextAIExtension::from_extension_data(data).unwrap();
        assert_eq!(ext.conversations.len(), 1);
        assert_eq!(ext.habits.preferred_language, "en"); // default
        assert_eq!(ext.settings.theme, "auto"); // default
    }
}

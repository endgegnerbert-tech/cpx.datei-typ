//! ContextAI Extension for CXP
//!
//! This module provides application-specific data structures for the ContextAI app.
//! Instead of using SQLite, all data is stored in the CXP file's extensions/ directory.

#[cfg(feature = "contextai")]
use serde::{Deserialize, Serialize};
#[cfg(feature = "contextai")]
use std::collections::HashMap;
#[cfg(feature = "contextai")]
use crate::{Result, CxpError, Extension};

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
    /// Referenced files in this message
    pub referenced_files: Vec<String>,
}

/// A single user habit entry
#[cfg(feature = "contextai")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserHabit {
    /// Unique habit ID
    pub id: String,
    /// Habit key (e.g., "preferred_language", "coding_style")
    pub habit_key: String,
    /// Habit value
    pub habit_value: String,
    /// Confidence level (0.0 - 1.0)
    pub confidence: f32,
    /// When the habit was last updated
    pub updated_at: String,
    /// Message ID this habit was learned from
    pub learned_from_message_id: Option<String>,
}

/// History of habit changes
#[cfg(feature = "contextai")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HabitHistory {
    /// Unique history entry ID
    pub id: String,
    /// Habit key
    pub habit_key: String,
    /// Old value (if any)
    pub old_value: Option<String>,
    /// New value
    pub new_value: String,
    /// When the change occurred
    pub updated_at: String,
}

/// A file entry in the context database
#[cfg(feature = "contextai")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileEntry {
    /// Unique file ID
    pub id: String,
    /// File name (without path)
    pub filename: String,
    /// Full file path
    pub filepath: String,
    /// File type (extension or MIME type)
    pub file_type: Option<String>,
    /// File size in bytes
    pub file_size: u64,
    /// AI-generated summary of the file
    pub summary: Option<String>,
    /// When the file was added
    pub created_at: String,
    /// When the file was last accessed
    pub last_accessed: Option<String>,
    /// File hash for change detection
    pub hash: Option<String>,
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

/// Context log entry - tracks which files were used in which messages
#[cfg(feature = "contextai")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextLogEntry {
    /// Unique log entry ID
    pub id: String,
    /// Message ID this relates to
    pub message_id: String,
    /// File ID that was loaded
    pub file_id: String,
    /// Whether the file was auto-loaded or manually selected
    pub auto_loaded: bool,
    /// When this context was created
    pub created_at: String,
}

/// Dictionary entry for custom terms and definitions
#[cfg(feature = "contextai")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DictionaryEntry {
    /// Unique dictionary entry ID
    pub id: String,
    /// The term or word
    pub term: String,
    /// Definition or description
    pub definition: String,
    /// Category or tag for organization
    pub category: Option<String>,
    /// Message ID this was learned from
    pub learned_from_message_id: Option<String>,
    /// When the entry was created
    pub created_at: String,
    /// When the entry was last updated
    pub updated_at: String,
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
    pub conversations: Vec<Conversation>,
    /// All indexed files
    pub files: Vec<FileEntry>,
    /// Watched folders
    pub watched_folders: Vec<WatchedFolder>,
    /// Context log - tracks file-message relationships
    pub context_log: Vec<ContextLogEntry>,
    /// User habits and learned preferences
    pub habits: Vec<UserHabit>,
    /// History of habit changes
    pub habit_history: Vec<HabitHistory>,
    /// User dictionary for custom terms
    pub dictionary: Vec<DictionaryEntry>,
    /// App settings
    pub settings: AppSettings,
    /// Last full CXP update timestamp (RFC3339 format)
    #[serde(default)]
    pub last_full_update: Option<String>,
}

#[cfg(feature = "contextai")]
impl ContextAIExtension {
    /// Create a new ContextAI extension with default values
    pub fn new() -> Self {
        Self {
            conversations: Vec::new(),
            files: Vec::new(),
            watched_folders: Vec::new(),
            context_log: Vec::new(),
            habits: Vec::new(),
            habit_history: Vec::new(),
            dictionary: Vec::new(),
            settings: AppSettings {
                theme: "auto".to_string(),
                auto_index: true,
                max_context_files: 50,
            },
            last_full_update: None,
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
    // File Management
    // ============================================================

    /// Add a file entry
    pub fn add_file(&mut self, file: FileEntry) {
        self.files.push(file);
    }

    /// Get a file by ID
    pub fn get_file(&self, id: &str) -> Option<&FileEntry> {
        self.files.iter().find(|f| f.id == id)
    }

    /// Get a file by path
    pub fn get_file_by_path(&self, path: &str) -> Option<&FileEntry> {
        self.files.iter().find(|f| f.filepath == path)
    }

    /// Get a mutable reference to a file by ID
    pub fn get_file_mut(&mut self, id: &str) -> Option<&mut FileEntry> {
        self.files.iter_mut().find(|f| f.id == id)
    }

    /// List all files
    pub fn list_files(&self) -> &[FileEntry] {
        &self.files
    }

    /// Update a file
    pub fn update_file(&mut self, id: &str, updated: FileEntry) -> Result<()> {
        if let Some(file) = self.files.iter_mut().find(|f| f.id == id) {
            *file = updated;
            Ok(())
        } else {
            Err(CxpError::FileNotFound(format!("File not found: {}", id)))
        }
    }

    /// Delete a file
    pub fn delete_file(&mut self, id: &str) -> Result<()> {
        if let Some(pos) = self.files.iter().position(|f| f.id == id) {
            self.files.remove(pos);
            Ok(())
        } else {
            Err(CxpError::FileNotFound(format!("File not found: {}", id)))
        }
    }

    // ============================================================
    // Context Log Management
    // ============================================================

    /// Add a context log entry
    pub fn add_context_log(&mut self, entry: ContextLogEntry) {
        self.context_log.push(entry);
    }

    /// Get context logs for a message
    pub fn get_context_logs_for_message(&self, message_id: &str) -> Vec<&ContextLogEntry> {
        self.context_log.iter()
            .filter(|e| e.message_id == message_id)
            .collect()
    }

    /// Get context logs for a file
    pub fn get_context_logs_for_file(&self, file_id: &str) -> Vec<&ContextLogEntry> {
        self.context_log.iter()
            .filter(|e| e.file_id == file_id)
            .collect()
    }

    /// List all context logs
    pub fn list_context_logs(&self) -> &[ContextLogEntry] {
        &self.context_log
    }

    /// Delete a context log entry
    pub fn delete_context_log(&mut self, id: &str) -> Result<()> {
        if let Some(pos) = self.context_log.iter().position(|e| e.id == id) {
            self.context_log.remove(pos);
            Ok(())
        } else {
            Err(CxpError::FileNotFound(format!("Context log entry not found: {}", id)))
        }
    }

    // ============================================================
    // Habits Management
    // ============================================================

    /// Add or update a habit
    /// Automatically creates history entries when habits are updated
    pub fn set_habit(&mut self, habit: UserHabit) {
        use std::time::{SystemTime, UNIX_EPOCH};

        // Generate a simple unique ID based on timestamp and counter
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();

        if let Some(existing) = self.habits.iter_mut().find(|h| h.habit_key == habit.habit_key) {
            // Record history
            let history = HabitHistory {
                id: format!("history-{}-{}", habit.habit_key, timestamp),
                habit_key: habit.habit_key.clone(),
                old_value: Some(existing.habit_value.clone()),
                new_value: habit.habit_value.clone(),
                updated_at: habit.updated_at.clone(),
            };
            self.habit_history.push(history);

            // Update habit
            *existing = habit;
        } else {
            // New habit, record in history
            let history = HabitHistory {
                id: format!("history-{}-{}", habit.habit_key, timestamp),
                habit_key: habit.habit_key.clone(),
                old_value: None,
                new_value: habit.habit_value.clone(),
                updated_at: habit.updated_at.clone(),
            };
            self.habit_history.push(history);
            self.habits.push(habit);
        }
    }

    /// Get a habit by key
    pub fn get_habit(&self, key: &str) -> Option<&UserHabit> {
        self.habits.iter().find(|h| h.habit_key == key)
    }

    /// List all habits
    pub fn list_habits(&self) -> &[UserHabit] {
        &self.habits
    }

    /// Delete a habit
    pub fn delete_habit(&mut self, key: &str) -> Result<()> {
        if let Some(pos) = self.habits.iter().position(|h| h.habit_key == key) {
            self.habits.remove(pos);
            Ok(())
        } else {
            Err(CxpError::FileNotFound(format!("Habit not found: {}", key)))
        }
    }

    /// Get habit history for a specific key
    pub fn get_habit_history(&self, key: &str) -> Vec<&HabitHistory> {
        self.habit_history.iter()
            .filter(|h| h.habit_key == key)
            .collect()
    }

    /// List all habit history
    pub fn list_habit_history(&self) -> &[HabitHistory] {
        &self.habit_history
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

    /// Add a dictionary entry
    pub fn add_dictionary_entry(&mut self, entry: DictionaryEntry) {
        // Check if term already exists
        if let Some(existing) = self.dictionary.iter_mut().find(|e| e.term == entry.term) {
            *existing = entry;
        } else {
            self.dictionary.push(entry);
        }
    }

    /// Get a dictionary entry by term
    pub fn get_dictionary_entry(&self, term: &str) -> Option<&DictionaryEntry> {
        self.dictionary.iter().find(|e| e.term == term)
    }

    /// Get a dictionary entry by ID
    pub fn get_dictionary_entry_by_id(&self, id: &str) -> Option<&DictionaryEntry> {
        self.dictionary.iter().find(|e| e.id == id)
    }

    /// List all dictionary entries
    pub fn list_dictionary(&self) -> &[DictionaryEntry] {
        &self.dictionary
    }

    /// Get dictionary entries by category
    pub fn get_dictionary_by_category(&self, category: &str) -> Vec<&DictionaryEntry> {
        self.dictionary.iter()
            .filter(|e| e.category.as_deref() == Some(category))
            .collect()
    }

    /// Update a dictionary entry
    pub fn update_dictionary_entry(&mut self, id: &str, updated: DictionaryEntry) -> Result<()> {
        if let Some(entry) = self.dictionary.iter_mut().find(|e| e.id == id) {
            *entry = updated;
            Ok(())
        } else {
            Err(CxpError::FileNotFound(format!("Dictionary entry not found: {}", id)))
        }
    }

    /// Remove a dictionary entry by ID
    pub fn remove_dictionary_entry(&mut self, id: &str) -> Result<()> {
        if let Some(pos) = self.dictionary.iter().position(|e| e.id == id) {
            self.dictionary.remove(pos);
            Ok(())
        } else {
            Err(CxpError::FileNotFound(format!("Dictionary entry not found: {}", id)))
        }
    }

    /// Remove a dictionary entry by term
    pub fn remove_dictionary_entry_by_term(&mut self, term: &str) -> Result<()> {
        if let Some(pos) = self.dictionary.iter().position(|e| e.term == term) {
            self.dictionary.remove(pos);
            Ok(())
        } else {
            Err(CxpError::FileNotFound(format!("Dictionary term not found: {}", term)))
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

        // Serialize files
        let files_data = rmp_serde::to_vec(&self.files)
            .map_err(|e| CxpError::Serialization(e.to_string()))?;
        data.insert("files.msgpack".to_string(), files_data);

        // Serialize watched folders
        let folders_data = rmp_serde::to_vec(&self.watched_folders)
            .map_err(|e| CxpError::Serialization(e.to_string()))?;
        data.insert("watched_folders.msgpack".to_string(), folders_data);

        // Serialize context log
        let context_log_data = rmp_serde::to_vec(&self.context_log)
            .map_err(|e| CxpError::Serialization(e.to_string()))?;
        data.insert("context_log.msgpack".to_string(), context_log_data);

        // Serialize habits
        let habits_data = rmp_serde::to_vec(&self.habits)
            .map_err(|e| CxpError::Serialization(e.to_string()))?;
        data.insert("habits.msgpack".to_string(), habits_data);

        // Serialize habit history
        let habit_history_data = rmp_serde::to_vec(&self.habit_history)
            .map_err(|e| CxpError::Serialization(e.to_string()))?;
        data.insert("habit_history.msgpack".to_string(), habit_history_data);

        // Serialize dictionary
        let dictionary_data = rmp_serde::to_vec(&self.dictionary)
            .map_err(|e| CxpError::Serialization(e.to_string()))?;
        data.insert("dictionary.msgpack".to_string(), dictionary_data);

        // Serialize settings
        let settings_data = rmp_serde::to_vec(&self.settings)
            .map_err(|e| CxpError::Serialization(e.to_string()))?;
        data.insert("settings.msgpack".to_string(), settings_data);

        // Serialize last_full_update
        let update_data = rmp_serde::to_vec(&self.last_full_update)
            .map_err(|e| CxpError::Serialization(e.to_string()))?;
        data.insert("last_full_update.msgpack".to_string(), update_data);

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

        // Deserialize files
        let files = if let Some(files_data) = data.get("files.msgpack") {
            rmp_serde::from_slice(files_data)
                .map_err(|e| CxpError::Serialization(e.to_string()))?
        } else {
            Vec::new()
        };

        // Deserialize watched folders
        let watched_folders = if let Some(folders_data) = data.get("watched_folders.msgpack") {
            rmp_serde::from_slice(folders_data)
                .map_err(|e| CxpError::Serialization(e.to_string()))?
        } else {
            Vec::new()
        };

        // Deserialize context log
        let context_log = if let Some(log_data) = data.get("context_log.msgpack") {
            rmp_serde::from_slice(log_data)
                .map_err(|e| CxpError::Serialization(e.to_string()))?
        } else {
            Vec::new()
        };

        // Deserialize habits
        let habits = if let Some(habits_data) = data.get("habits.msgpack") {
            rmp_serde::from_slice(habits_data)
                .map_err(|e| CxpError::Serialization(e.to_string()))?
        } else {
            Vec::new()
        };

        // Deserialize habit history
        let habit_history = if let Some(history_data) = data.get("habit_history.msgpack") {
            rmp_serde::from_slice(history_data)
                .map_err(|e| CxpError::Serialization(e.to_string()))?
        } else {
            Vec::new()
        };

        // Deserialize dictionary
        let dictionary = if let Some(dict_data) = data.get("dictionary.msgpack") {
            rmp_serde::from_slice(dict_data)
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

        // Deserialize last_full_update
        let last_full_update = if let Some(update_data) = data.get("last_full_update.msgpack") {
            rmp_serde::from_slice(update_data)
                .map_err(|e| CxpError::Serialization(e.to_string()))?
        } else {
            None
        };

        Ok(Self {
            conversations,
            files,
            watched_folders,
            context_log,
            habits,
            habit_history,
            dictionary,
            settings,
            last_full_update,
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

/// Implement the Extension trait for ContextAI
#[cfg(feature = "contextai")]
impl Extension for ContextAIExtension {
    fn namespace(&self) -> &str {
        "contextai"
    }

    fn version(&self) -> &str {
        "1.0.0"
    }
}

#[cfg(all(test, feature = "contextai"))]
mod tests {
    use super::*;

    #[test]
    fn test_contextai_creation() {
        let ext = ContextAIExtension::new();
        assert_eq!(ext.conversations.len(), 0);
        assert_eq!(ext.files.len(), 0);
        assert_eq!(ext.habits.len(), 0);
        assert_eq!(ext.dictionary.len(), 0);
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
            referenced_files: vec![],
        };

        ext.add_message("conv-1", message).unwrap();

        let conv = ext.get_conversation("conv-1").unwrap();
        assert_eq!(conv.messages.len(), 1);
        assert_eq!(conv.messages[0].content, "Hello!");
    }

    #[test]
    fn test_habits_management() {
        let mut ext = ContextAIExtension::new();

        let habit = UserHabit {
            id: "habit-1".to_string(),
            habit_key: "preferred_language".to_string(),
            habit_value: "de".to_string(),
            confidence: 0.9,
            updated_at: "2025-01-01T00:00:00Z".to_string(),
            learned_from_message_id: None,
        };

        ext.set_habit(habit);

        let retrieved = ext.get_habit("preferred_language").unwrap();
        assert_eq!(retrieved.habit_value, "de");
        assert_eq!(retrieved.confidence, 0.9);

        // Check that history was created
        assert_eq!(ext.habit_history.len(), 1);
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

        let entry1 = DictionaryEntry {
            id: "dict-1".to_string(),
            term: "TypeScript".to_string(),
            definition: "A typed superset of JavaScript".to_string(),
            category: Some("Programming".to_string()),
            learned_from_message_id: None,
            created_at: "2025-01-01T00:00:00Z".to_string(),
            updated_at: "2025-01-01T00:00:00Z".to_string(),
        };

        let entry2 = DictionaryEntry {
            id: "dict-2".to_string(),
            term: "React".to_string(),
            definition: "A JavaScript library for building UIs".to_string(),
            category: Some("Programming".to_string()),
            learned_from_message_id: None,
            created_at: "2025-01-01T00:00:00Z".to_string(),
            updated_at: "2025-01-01T00:00:00Z".to_string(),
        };

        ext.add_dictionary_entry(entry1);
        ext.add_dictionary_entry(entry2);

        assert_eq!(ext.list_dictionary().len(), 2);
        assert!(ext.get_dictionary_entry("TypeScript").is_some());

        ext.remove_dictionary_entry_by_term("React").unwrap();
        assert_eq!(ext.list_dictionary().len(), 1);
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

        let dict_entry = DictionaryEntry {
            id: "dict-1".to_string(),
            term: "test-word".to_string(),
            definition: "A test definition".to_string(),
            category: None,
            learned_from_message_id: None,
            created_at: "2025-01-01T00:00:00Z".to_string(),
            updated_at: "2025-01-01T00:00:00Z".to_string(),
        };

        ext.add_conversation(conv);
        ext.add_dictionary_entry(dict_entry);

        // Test to_extension_data
        let data = ext.to_extension_data().unwrap();
        assert!(data.contains_key("conversations.msgpack"));
        assert!(data.contains_key("files.msgpack"));
        assert!(data.contains_key("habits.msgpack"));
        assert!(data.contains_key("habit_history.msgpack"));
        assert!(data.contains_key("context_log.msgpack"));
        assert!(data.contains_key("settings.msgpack"));
        assert!(data.contains_key("dictionary.msgpack"));

        // Test from_extension_data
        let restored = ContextAIExtension::from_extension_data(data).unwrap();
        assert_eq!(restored.conversations.len(), 1);
        assert_eq!(restored.conversations[0].id, "conv-1");
        assert_eq!(restored.dictionary.len(), 1);
        assert_eq!(restored.dictionary[0].term, "test-word");
    }

    #[test]
    fn test_json_serialization() {
        let ext = ContextAIExtension::new();

        let json = ext.to_json().unwrap();
        assert!(!json.is_empty());

        let restored = ContextAIExtension::from_json(&json).unwrap();
        assert_eq!(restored.habits.len(), ext.habits.len());
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
        assert_eq!(ext.habits.len(), 0); // default - empty
        assert_eq!(ext.files.len(), 0); // default - empty
        assert_eq!(ext.settings.theme, "auto"); // default
    }

    #[test]
    fn test_file_management() {
        let mut ext = ContextAIExtension::new();

        let file = FileEntry {
            id: "file-1".to_string(),
            filename: "test.txt".to_string(),
            filepath: "/path/to/test.txt".to_string(),
            file_type: Some("text/plain".to_string()),
            file_size: 1024,
            summary: Some("A test file".to_string()),
            created_at: "2025-01-01T00:00:00Z".to_string(),
            last_accessed: None,
            hash: Some("abc123".to_string()),
        };

        ext.add_file(file);
        assert_eq!(ext.list_files().len(), 1);

        let retrieved = ext.get_file("file-1").unwrap();
        assert_eq!(retrieved.filename, "test.txt");

        let by_path = ext.get_file_by_path("/path/to/test.txt").unwrap();
        assert_eq!(by_path.id, "file-1");

        ext.delete_file("file-1").unwrap();
        assert_eq!(ext.list_files().len(), 0);
    }

    #[test]
    fn test_context_log() {
        let mut ext = ContextAIExtension::new();

        let log1 = ContextLogEntry {
            id: "log-1".to_string(),
            message_id: "msg-1".to_string(),
            file_id: "file-1".to_string(),
            auto_loaded: true,
            created_at: "2025-01-01T00:00:00Z".to_string(),
        };

        let log2 = ContextLogEntry {
            id: "log-2".to_string(),
            message_id: "msg-1".to_string(),
            file_id: "file-2".to_string(),
            auto_loaded: false,
            created_at: "2025-01-01T00:00:01Z".to_string(),
        };

        ext.add_context_log(log1);
        ext.add_context_log(log2);

        let logs_for_msg = ext.get_context_logs_for_message("msg-1");
        assert_eq!(logs_for_msg.len(), 2);

        let logs_for_file = ext.get_context_logs_for_file("file-1");
        assert_eq!(logs_for_file.len(), 1);
    }

    #[test]
    fn test_habit_history() {
        let mut ext = ContextAIExtension::new();

        let habit1 = UserHabit {
            id: "habit-1".to_string(),
            habit_key: "theme".to_string(),
            habit_value: "dark".to_string(),
            confidence: 0.8,
            updated_at: "2025-01-01T00:00:00Z".to_string(),
            learned_from_message_id: None,
        };

        ext.set_habit(habit1);

        // Update the habit
        let habit2 = UserHabit {
            id: "habit-1".to_string(),
            habit_key: "theme".to_string(),
            habit_value: "light".to_string(),
            confidence: 0.9,
            updated_at: "2025-01-02T00:00:00Z".to_string(),
            learned_from_message_id: Some("msg-1".to_string()),
        };

        ext.set_habit(habit2);

        // Should have 2 history entries (initial + update)
        let history = ext.get_habit_history("theme");
        assert_eq!(history.len(), 2);
        assert_eq!(history[0].new_value, "dark");
        assert_eq!(history[1].old_value, Some("dark".to_string()));
        assert_eq!(history[1].new_value, "light");
    }
}

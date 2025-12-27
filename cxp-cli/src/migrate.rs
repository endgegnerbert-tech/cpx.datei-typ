//! SQLite to CXP Migration
//!
//! Migrates ContextAI SQLite database to CXP format with ContextAI extension.

use anyhow::{Context, Result};
use cxp_core::contextai::{
    ContextAIExtension, Conversation, ChatMessage, WatchedFolder, AppSettings,
    UserHabit, HabitHistory, FileEntry, ContextLogEntry, DictionaryEntry,
};
use cxp_core::CxpBuilder;
use rusqlite::Connection;
use std::path::Path;
use tracing::info;

/// Migration statistics
#[derive(Debug, Default)]
#[allow(dead_code)]
pub struct MigrationStats {
    pub files_migrated: usize,
    pub conversations_migrated: usize,
    pub messages_migrated: usize,
    pub context_log_entries: usize,
    pub habits_migrated: usize,
    pub habit_history_entries: usize,
    pub dictionary_entries: usize,
    pub watched_folders: usize,
}

/// Migrate a ContextAI SQLite database to CXP format with full statistics
///
/// This is the main migration function that migrates ALL SQLite tables to CXP format.
///
/// # Arguments
/// * `sqlite_path` - Path to the SQLite database file
/// * `output_path` - Path for the output CXP file
/// * `include_files_content` - If true, scans and includes actual file content from watched folders
///
/// # Returns
/// Returns `MigrationStats` with counts of all migrated entities
#[allow(dead_code)]
pub fn migrate_full_database(
    sqlite_path: &Path,
    output_path: &Path,
    include_files_content: bool,
) -> Result<MigrationStats> {
    info!("Starting full SQLite to CXP migration...");
    info!("  SQLite DB: {}", sqlite_path.display());
    info!("  Output CXP: {}", output_path.display());
    info!("  Include file content: {}", include_files_content);
    println!();

    let mut stats = MigrationStats::default();

    // Open SQLite database
    let conn = Connection::open(sqlite_path)
        .context("Failed to open SQLite database")?;

    // Create ContextAI extension
    let mut contextai = ContextAIExtension::new();

    // Migrate all tables
    info!("Migrating file entries...");
    stats.files_migrated = migrate_files(&conn, &mut contextai)?;

    info!("Migrating watched folders...");
    stats.watched_folders = migrate_watched_folders(&conn, &mut contextai)?;

    info!("Migrating conversations and messages...");
    let (conv_count, msg_count) = migrate_conversations(&conn, &mut contextai)?;
    stats.conversations_migrated = conv_count;
    stats.messages_migrated = msg_count;

    info!("Migrating context log...");
    stats.context_log_entries = migrate_context_log(&conn, &mut contextai)?;

    info!("Migrating user habits...");
    stats.habits_migrated = migrate_user_habits(&conn, &mut contextai)?;

    info!("Migrating habit history...");
    stats.habit_history_entries = migrate_habit_history(&conn, &mut contextai)?;

    info!("Migrating custom dictionary...");
    stats.dictionary_entries = migrate_custom_dictionary(&conn, &mut contextai)?;

    info!("Migrating app settings...");
    migrate_app_settings(&conn, &mut contextai)?;

    // Create CXP builder
    let mut builder = if include_files_content && !contextai.watched_folders.is_empty() {
        // Try to use the first watched folder as source
        let first_folder = &contextai.watched_folders[0].path;
        info!("Using watched folder as source: {}", first_folder);
        let folder_path = Path::new(first_folder);
        if folder_path.exists() {
            CxpBuilder::new(folder_path)
        } else {
            info!("Watched folder doesn't exist, creating empty CXP");
            CxpBuilder::new(Path::new("."))
        }
    } else {
        CxpBuilder::new(Path::new("."))
    };

    // Scan and process files if requested
    if include_files_content {
        info!("Scanning and processing source files...");
        if let Err(e) = builder.scan() {
            info!("Warning: Failed to scan files: {}", e);
        } else if let Err(e) = builder.process() {
            info!("Warning: Failed to process files: {}", e);
        }
    }

    // Add ContextAI extension to builder
    info!("Adding ContextAI extension to CXP...");
    let extension_data = contextai.to_extension_data()
        .context("Failed to serialize ContextAI extension")?;

    builder.add_extension(&contextai, extension_data)
        .context("Failed to add ContextAI extension to CXP")?;

    // Build CXP file
    info!("Building CXP file...");
    builder.build(output_path)
        .context("Failed to build CXP file")?;

    info!("Migration completed successfully!");
    println!();
    println!("Migration Statistics:");
    println!("  Files:             {}", stats.files_migrated);
    println!("  Conversations:     {}", stats.conversations_migrated);
    println!("  Messages:          {}", stats.messages_migrated);
    println!("  Context Logs:      {}", stats.context_log_entries);
    println!("  Habits:            {}", stats.habits_migrated);
    println!("  Habit History:     {}", stats.habit_history_entries);
    println!("  Dictionary:        {}", stats.dictionary_entries);
    println!("  Watched Folders:   {}", stats.watched_folders);

    Ok(stats)
}

/// Migrate a ContextAI SQLite database to CXP format (legacy interface)
///
/// # Arguments
/// * `sqlite_path` - Path to the SQLite database file
/// * `output_cxp` - Path for the output CXP file
/// * `source_files_dir` - Optional directory containing source files to include in the CXP
///
/// # Example
/// ```no_run
/// use std::path::Path;
/// migrate_sqlite_to_cxp(
///     Path::new("contextai.db"),
///     Path::new("output.cxp"),
///     Some(Path::new("/path/to/source/files"))
/// )?;
/// ```
pub fn migrate_sqlite_to_cxp(
    sqlite_path: &Path,
    output_cxp: &Path,
    source_files_dir: Option<&Path>,
) -> Result<()> {
    info!("Starting SQLite to CXP migration...");
    info!("  SQLite DB: {}", sqlite_path.display());
    info!("  Output CXP: {}", output_cxp.display());
    if let Some(dir) = source_files_dir {
        info!("  Source files: {}", dir.display());
    }

    // Open SQLite database
    let conn = Connection::open(sqlite_path)
        .context("Failed to open SQLite database")?;

    // Create ContextAI extension
    let mut contextai = ContextAIExtension::new();

    // Migrate files
    info!("Migrating file entries...");
    migrate_files(&conn, &mut contextai)?;

    // Migrate watched folders
    info!("Migrating watched folders...");
    migrate_watched_folders(&conn, &mut contextai)?;

    // Migrate conversations and messages
    info!("Migrating conversations and messages...");
    migrate_conversations(&conn, &mut contextai)?;

    // Migrate context log
    info!("Migrating context log...");
    migrate_context_log(&conn, &mut contextai)?;

    // Migrate user habits and habit history
    info!("Migrating user habits...");
    migrate_user_habits(&conn, &mut contextai)?;

    info!("Migrating habit history...");
    migrate_habit_history(&conn, &mut contextai)?;

    // Migrate custom dictionary
    info!("Migrating custom dictionary...");
    migrate_custom_dictionary(&conn, &mut contextai)?;

    // Migrate app settings (inferred from database)
    info!("Migrating app settings...");
    migrate_app_settings(&conn, &mut contextai)?;

    // Create CXP builder
    let source_dir = source_files_dir.unwrap_or_else(|| Path::new("."));
    let mut builder = CxpBuilder::new(source_dir);

    // Scan and process files if source directory is provided
    if source_files_dir.is_some() {
        info!("Scanning source directory for files...");
        builder.scan().context("Failed to scan directory")?;
        info!("Processing files...");
        builder.process().context("Failed to process files")?;
    } else {
        info!("No source directory provided, creating CXP with extension data only");
    }

    // Add ContextAI extension to builder
    info!("Adding ContextAI extension to CXP...");
    let extension_data = contextai.to_extension_data()
        .context("Failed to serialize ContextAI extension")?;

    builder.add_extension(&contextai, extension_data)
        .context("Failed to add ContextAI extension to CXP")?;

    // Build CXP file
    info!("Building CXP file...");
    builder.build(output_cxp)
        .context("Failed to build CXP file")?;

    info!("Migration completed successfully!");
    Ok(())
}

/// Migrate file entries from SQLite
fn migrate_files(conn: &Connection, contextai: &mut ContextAIExtension) -> Result<usize> {
    let mut stmt = conn.prepare(
        "SELECT id, filename, filepath, file_type, file_size, summary, created_at, last_accessed, hash
         FROM files
         ORDER BY created_at"
    )?;

    let files_iter = stmt.query_map([], |row| {
        Ok(FileEntry {
            id: format!("file-{}", row.get::<_, i64>(0)?),
            filename: row.get::<_, String>(1)?,
            filepath: row.get::<_, String>(2)?,
            file_type: row.get::<_, Option<String>>(3)?,
            file_size: row.get::<_, i64>(4)? as u64,
            summary: row.get::<_, Option<String>>(5)?,
            created_at: row.get::<_, String>(6)?,
            last_accessed: row.get::<_, Option<String>>(7)?,
            hash: row.get::<_, Option<String>>(8)?,
        })
    })?;

    let mut file_count = 0;
    for file_result in files_iter {
        let file = file_result?;
        contextai.add_file(file);
        file_count += 1;
    }

    info!("  Migrated {} files", file_count);
    Ok(file_count)
}

/// Migrate conversations and chat messages from SQLite
fn migrate_conversations(conn: &Connection, contextai: &mut ContextAIExtension) -> Result<(usize, usize)> {
    // Query all conversations
    let mut stmt = conn.prepare(
        "SELECT id, title, created_at, updated_at FROM conversations ORDER BY created_at"
    )?;

    let conversations_iter = stmt.query_map([], |row| {
        Ok((
            row.get::<_, i64>(0)?,      // id
            row.get::<_, String>(1)?,   // title
            row.get::<_, String>(2)?,   // created_at
            row.get::<_, String>(3)?,   // updated_at
        ))
    })?;

    let mut conversation_count = 0;
    let mut message_count = 0;

    for conv_result in conversations_iter {
        let (conv_id, title, created_at, updated_at) = conv_result?;

        // Query messages for this conversation
        let mut msg_stmt = conn.prepare(
            "SELECT id, role, content, referenced_files, created_at
             FROM chat_messages
             WHERE conversation_id = ?
             ORDER BY created_at"
        )?;

        let messages_iter = msg_stmt.query_map([conv_id], |row| {
            // Parse referenced_files JSON if present
            let referenced_files: Vec<String> = row.get::<_, Option<String>>(3)?
                .and_then(|json_str| serde_json::from_str(&json_str).ok())
                .unwrap_or_default();

            Ok(ChatMessage {
                id: format!("msg-{}", row.get::<_, i64>(0)?),
                role: row.get::<_, String>(1)?,
                content: row.get::<_, String>(2)?,
                referenced_files,
                timestamp: row.get::<_, String>(4)?,
            })
        })?;

        let messages: Vec<ChatMessage> = messages_iter
            .filter_map(|msg| msg.ok())
            .collect();

        message_count += messages.len();

        // Create conversation
        let conversation = Conversation {
            id: format!("conv-{}", conv_id),
            title,
            created_at,
            updated_at,
            messages,
        };

        contextai.add_conversation(conversation);
        conversation_count += 1;
    }

    info!("  Migrated {} conversations with {} messages", conversation_count, message_count);
    Ok((conversation_count, message_count))
}

/// Migrate user habits from SQLite
fn migrate_user_habits(conn: &Connection, contextai: &mut ContextAIExtension) -> Result<usize> {
    let mut stmt = conn.prepare(
        "SELECT id, habit_key, habit_value, confidence, updated_at, learned_from_message_id
         FROM user_habits
         ORDER BY updated_at DESC"
    )?;

    let habits_iter = stmt.query_map([], |row| {
        // Convert learned_from_message_id to String format if present
        let learned_from_message_id: Option<String> = row.get::<_, Option<i64>>(5)?
            .map(|id| format!("msg-{}", id));

        Ok(UserHabit {
            id: format!("habit-{}", row.get::<_, i64>(0)?),
            habit_key: row.get::<_, String>(1)?,
            habit_value: row.get::<_, String>(2)?,
            confidence: row.get::<_, f64>(3)? as f32,
            updated_at: row.get::<_, String>(4)?,
            learned_from_message_id,
        })
    })?;

    let mut habit_count = 0;
    for habit_result in habits_iter {
        let habit = habit_result?;
        contextai.set_habit(habit);
        habit_count += 1;
    }

    info!("  Migrated {} user habits", habit_count);
    Ok(habit_count)
}

/// Migrate habit history from SQLite
fn migrate_habit_history(conn: &Connection, contextai: &mut ContextAIExtension) -> Result<usize> {
    let mut stmt = conn.prepare(
        "SELECT id, habit_key, old_value, new_value, updated_at
         FROM habit_history
         ORDER BY updated_at"
    )?;

    let history_iter = stmt.query_map([], |row| {
        Ok(HabitHistory {
            id: format!("history-{}", row.get::<_, i64>(0)?),
            habit_key: row.get::<_, String>(1)?,
            old_value: row.get::<_, Option<String>>(2)?,
            new_value: row.get::<_, String>(3)?,
            updated_at: row.get::<_, String>(4)?,
        })
    })?;

    let mut history_count = 0;
    for history_result in history_iter {
        let history = history_result?;
        // Note: We can't use contextai.add_habit_history directly because it's not exposed,
        // but set_habit already creates history entries, so we need to add them manually
        contextai.habit_history.push(history);
        history_count += 1;
    }

    info!("  Migrated {} habit history entries", history_count);
    Ok(history_count)
}

/// Migrate watched folders from SQLite
fn migrate_watched_folders(conn: &Connection, contextai: &mut ContextAIExtension) -> Result<usize> {
    let mut stmt = conn.prepare(
        "SELECT folder_path, enabled, created_at FROM watched_folders"
    )?;

    let folders_iter = stmt.query_map([], |row| {
        Ok(WatchedFolder {
            path: row.get::<_, String>(0)?,
            enabled: row.get::<_, i32>(1)? != 0,
            last_scan: Some(row.get::<_, String>(2)?),
        })
    })?;

    let mut folder_count = 0;
    for folder_result in folders_iter {
        let folder = folder_result?;
        contextai.add_watched_folder(folder);
        folder_count += 1;
    }

    info!("  Migrated {} watched folders", folder_count);
    Ok(folder_count)
}

/// Migrate context log from SQLite
fn migrate_context_log(conn: &Connection, contextai: &mut ContextAIExtension) -> Result<usize> {
    let mut stmt = conn.prepare(
        "SELECT id, message_id, file_id, auto_loaded, created_at
         FROM context_log
         ORDER BY created_at"
    )?;

    let log_iter = stmt.query_map([], |row| {
        Ok(ContextLogEntry {
            id: format!("log-{}", row.get::<_, i64>(0)?),
            message_id: format!("msg-{}", row.get::<_, i64>(1)?),
            file_id: format!("file-{}", row.get::<_, i64>(2)?),
            auto_loaded: row.get::<_, i32>(3)? != 0,
            created_at: row.get::<_, String>(4)?,
        })
    })?;

    let mut log_count = 0;
    for log_result in log_iter {
        let log = log_result?;
        contextai.add_context_log(log);
        log_count += 1;
    }

    info!("  Migrated {} context log entries", log_count);
    Ok(log_count)
}

/// Migrate app settings (inferred from database or use defaults)
fn migrate_app_settings(conn: &Connection, contextai: &mut ContextAIExtension) -> Result<()> {
    // Try to find settings in user_habits table
    let theme = conn.query_row(
        "SELECT habit_value FROM user_habits WHERE habit_key = 'theme'",
        [],
        |row| row.get::<_, String>(0)
    ).unwrap_or_else(|_| "auto".to_string());

    let auto_index = conn.query_row(
        "SELECT habit_value FROM user_habits WHERE habit_key = 'auto_index'",
        [],
        |row| row.get::<_, String>(0)
    ).unwrap_or_else(|_| "true".to_string()) == "true";

    let max_context_files = conn.query_row(
        "SELECT habit_value FROM user_habits WHERE habit_key = 'max_context_files'",
        [],
        |row| row.get::<_, String>(0)
    ).ok()
        .and_then(|s| s.parse::<u32>().ok())
        .unwrap_or(50);

    let settings = AppSettings {
        theme,
        auto_index,
        max_context_files,
    };

    contextai.set_settings(settings);
    info!("  Migrated app settings");
    Ok(())
}

/// Migrate custom dictionary from SQLite
fn migrate_custom_dictionary(conn: &Connection, contextai: &mut ContextAIExtension) -> Result<usize> {
    let mut stmt = conn.prepare(
        "SELECT id, term, definition, category, learned_from_message_id, created_at, updated_at
         FROM custom_dictionary
         ORDER BY created_at"
    )?;

    let dict_iter = stmt.query_map([], |row| {
        // Convert learned_from_message_id to String format if present
        let learned_from_message_id: Option<String> = row.get::<_, Option<i64>>(4)?
            .map(|id| format!("msg-{}", id));

        Ok(DictionaryEntry {
            id: format!("dict-{}", row.get::<_, i64>(0)?),
            term: row.get::<_, String>(1)?,
            definition: row.get::<_, String>(2)?,
            category: row.get::<_, Option<String>>(3)?,
            learned_from_message_id,
            created_at: row.get::<_, String>(5)?,
            updated_at: row.get::<_, String>(6)?,
        })
    })?;

    let mut dict_count = 0;
    for dict_result in dict_iter {
        let entry = dict_result?;
        contextai.add_dictionary_entry(entry);
        dict_count += 1;
    }

    info!("  Migrated {} dictionary entries", dict_count);
    Ok(dict_count)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_migration_with_empty_db() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let output_path = temp_dir.path().join("output.cxp");

        // Create an empty SQLite database with schema
        let conn = Connection::open(&db_path).unwrap();
        conn.execute_batch(
            "
            CREATE TABLE files (
                id INTEGER PRIMARY KEY,
                filename TEXT NOT NULL,
                filepath TEXT UNIQUE NOT NULL,
                file_type TEXT,
                file_size INTEGER NOT NULL,
                summary TEXT,
                created_at TEXT NOT NULL,
                last_accessed TEXT,
                hash TEXT
            );
            CREATE TABLE conversations (
                id INTEGER PRIMARY KEY,
                title TEXT NOT NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );
            CREATE TABLE chat_messages (
                id INTEGER PRIMARY KEY,
                conversation_id INTEGER,
                role TEXT NOT NULL,
                content TEXT NOT NULL,
                referenced_files TEXT,
                created_at TEXT NOT NULL
            );
            CREATE TABLE context_log (
                id INTEGER PRIMARY KEY,
                message_id INTEGER NOT NULL,
                file_id INTEGER NOT NULL,
                auto_loaded INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL
            );
            CREATE TABLE user_habits (
                id INTEGER PRIMARY KEY,
                habit_key TEXT UNIQUE NOT NULL,
                habit_value TEXT NOT NULL,
                confidence REAL NOT NULL,
                updated_at TEXT NOT NULL,
                learned_from_message_id INTEGER
            );
            CREATE TABLE habit_history (
                id INTEGER PRIMARY KEY,
                habit_key TEXT NOT NULL,
                old_value TEXT,
                new_value TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );
            CREATE TABLE watched_folders (
                id INTEGER PRIMARY KEY,
                folder_path TEXT UNIQUE NOT NULL,
                enabled INTEGER NOT NULL DEFAULT 1,
                created_at TEXT NOT NULL
            );
            CREATE TABLE custom_dictionary (
                id INTEGER PRIMARY KEY,
                term TEXT UNIQUE NOT NULL,
                definition TEXT NOT NULL,
                category TEXT,
                learned_from_message_id INTEGER,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );
            "
        ).unwrap();
        drop(conn);

        // Run migration
        let result = migrate_sqlite_to_cxp(&db_path, &output_path, None);
        assert!(result.is_ok(), "Migration should succeed with empty database");
        assert!(output_path.exists(), "Output CXP file should be created");
    }
}

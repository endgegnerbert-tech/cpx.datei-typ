//! SQLite to CXP Migration
//!
//! Migrates ContextAI SQLite database to CXP format with ContextAI extension.

use anyhow::{Context, Result};
use cxp_core::contextai::{ContextAIExtension, Conversation, ChatMessage, UserHabits, WatchedFolder, AppSettings};
use cxp_core::CxpBuilder;
use rusqlite::Connection;
use std::path::Path;
use std::collections::HashMap;
use tracing::info;

/// Migrate a ContextAI SQLite database to CXP format
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

    // Migrate conversations and messages
    info!("Migrating conversations and messages...");
    migrate_conversations(&conn, &mut contextai)?;

    // Migrate user habits
    info!("Migrating user habits...");
    migrate_user_habits(&conn, &mut contextai)?;

    // Migrate watched folders
    info!("Migrating watched folders...");
    migrate_watched_folders(&conn, &mut contextai)?;

    // Migrate app settings (inferred from database)
    info!("Migrating app settings...");
    migrate_app_settings(&conn, &mut contextai)?;

    // Migrate custom dictionary
    info!("Migrating custom dictionary...");
    migrate_custom_dictionary(&conn, &mut contextai)?;

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

/// Migrate conversations and chat messages from SQLite
fn migrate_conversations(conn: &Connection, contextai: &mut ContextAIExtension) -> Result<()> {
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
            "SELECT id, role, content, created_at
             FROM chat_messages
             WHERE conversation_id = ?
             ORDER BY created_at"
        )?;

        let messages_iter = msg_stmt.query_map([conv_id], |row| {
            Ok(ChatMessage {
                id: row.get::<_, i64>(0)?.to_string(),
                role: row.get::<_, String>(1)?,
                content: row.get::<_, String>(2)?,
                timestamp: row.get::<_, String>(3)?,
            })
        })?;

        let messages: Vec<ChatMessage> = messages_iter
            .filter_map(|msg| msg.ok())
            .collect();

        message_count += messages.len();

        // Create conversation
        let conversation = Conversation {
            id: conv_id.to_string(),
            title,
            created_at,
            updated_at,
            messages,
        };

        contextai.add_conversation(conversation);
        conversation_count += 1;
    }

    info!("  Migrated {} conversations with {} messages", conversation_count, message_count);
    Ok(())
}

/// Migrate user habits from SQLite
fn migrate_user_habits(conn: &Connection, contextai: &mut ContextAIExtension) -> Result<()> {
    let mut stmt = conn.prepare(
        "SELECT habit_key, habit_value FROM user_habits ORDER BY updated_at DESC"
    )?;

    let habits_iter = stmt.query_map([], |row| {
        Ok((
            row.get::<_, String>(0)?,   // habit_key
            row.get::<_, String>(1)?,   // habit_value
        ))
    })?;

    // Convert habits to a hashmap
    let mut habits_map: HashMap<String, String> = HashMap::new();
    for habit_result in habits_iter {
        let (key, value) = habit_result?;
        habits_map.insert(key, value);
    }

    // Extract known habits into UserHabits structure
    let preferred_language = habits_map.get("preferred_language")
        .cloned()
        .unwrap_or_else(|| "en".to_string());

    let coding_style = habits_map.get("coding_style").cloned();

    // Custom instructions: collect other habits as custom instructions
    let custom_instructions: Vec<String> = habits_map
        .iter()
        .filter(|(k, _)| k.as_str() != "preferred_language" && k.as_str() != "coding_style")
        .map(|(k, v)| format!("{}: {}", k, v))
        .collect();

    let user_habits = UserHabits {
        preferred_language,
        coding_style,
        custom_instructions,
    };

    contextai.set_habits(user_habits);
    info!("  Migrated {} user habits", habits_map.len());
    Ok(())
}

/// Migrate watched folders from SQLite
fn migrate_watched_folders(conn: &Connection, contextai: &mut ContextAIExtension) -> Result<()> {
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
    Ok(())
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
fn migrate_custom_dictionary(conn: &Connection, contextai: &mut ContextAIExtension) -> Result<()> {
    let mut stmt = conn.prepare(
        "SELECT term, definition FROM custom_dictionary ORDER BY created_at"
    )?;

    let dict_iter = stmt.query_map([], |row| {
        Ok((
            row.get::<_, String>(0)?,   // term
            row.get::<_, String>(1)?,   // definition
        ))
    })?;

    let mut dict_count = 0;
    for dict_result in dict_iter {
        let (term, definition) = dict_result?;
        // Store as "term: definition" in the dictionary
        contextai.add_to_dictionary(format!("{}: {}", term, definition));
        dict_count += 1;
    }

    info!("  Migrated {} dictionary entries", dict_count);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
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
                created_at TEXT NOT NULL
            );
            CREATE TABLE user_habits (
                id INTEGER PRIMARY KEY,
                habit_key TEXT UNIQUE NOT NULL,
                habit_value TEXT NOT NULL,
                confidence REAL NOT NULL,
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

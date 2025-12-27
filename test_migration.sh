#!/bin/bash

# Test script for SQLite to CXP migration

set -e

echo "Creating test SQLite database..."

TEST_DB="test_contextai.db"
TEST_CXP="test_output.cxp"

# Clean up previous test files
rm -f "$TEST_DB" "$TEST_CXP"

# Create test database with schema
sqlite3 "$TEST_DB" <<EOF
-- Create schema
CREATE TABLE conversations (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    title TEXT NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE chat_messages (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    conversation_id INTEGER,
    role TEXT NOT NULL CHECK(role IN ('user', 'assistant')),
    content TEXT NOT NULL,
    created_at TEXT NOT NULL,
    FOREIGN KEY (conversation_id) REFERENCES conversations(id) ON DELETE CASCADE
);

CREATE TABLE user_habits (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    habit_key TEXT UNIQUE NOT NULL,
    habit_value TEXT NOT NULL,
    confidence REAL NOT NULL CHECK(confidence >= 0.0 AND confidence <= 1.0),
    updated_at TEXT NOT NULL
);

CREATE TABLE watched_folders (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    folder_path TEXT UNIQUE NOT NULL,
    enabled INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL
);

CREATE TABLE custom_dictionary (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    term TEXT UNIQUE NOT NULL,
    definition TEXT NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

-- Insert test data
INSERT INTO conversations (title, created_at, updated_at) VALUES
    ('First Conversation', '2025-01-01T10:00:00Z', '2025-01-01T10:30:00Z'),
    ('Second Conversation', '2025-01-02T14:00:00Z', '2025-01-02T14:15:00Z');

INSERT INTO chat_messages (conversation_id, role, content, created_at) VALUES
    (1, 'user', 'Hello, how are you?', '2025-01-01T10:00:00Z'),
    (1, 'assistant', 'I am doing well, thank you!', '2025-01-01T10:01:00Z'),
    (1, 'user', 'Can you help me with Rust?', '2025-01-01T10:02:00Z'),
    (1, 'assistant', 'Of course! What do you need help with?', '2025-01-01T10:03:00Z'),
    (2, 'user', 'What is the weather?', '2025-01-02T14:00:00Z'),
    (2, 'assistant', 'I cannot access real-time weather data.', '2025-01-02T14:01:00Z');

INSERT INTO user_habits (habit_key, habit_value, confidence, updated_at) VALUES
    ('preferred_language', 'en', 1.0, '2025-01-01T00:00:00Z'),
    ('coding_style', '4-space-indent', 0.9, '2025-01-01T00:00:00Z'),
    ('theme', 'dark', 0.8, '2025-01-01T00:00:00Z');

INSERT INTO watched_folders (folder_path, enabled, created_at) VALUES
    ('/home/user/projects', 1, '2025-01-01T00:00:00Z'),
    ('/home/user/documents', 0, '2025-01-01T00:00:00Z');

INSERT INTO custom_dictionary (term, definition, created_at, updated_at) VALUES
    ('CXP', 'Universal AI Context Format', '2025-01-01T00:00:00Z', '2025-01-01T00:00:00Z'),
    ('Rust', 'A systems programming language', '2025-01-01T00:00:00Z', '2025-01-01T00:00:00Z');
EOF

echo "Test database created: $TEST_DB"
echo ""

# Run migration
echo "Running migration..."
cargo run --package cxp-cli -- migrate "$TEST_DB" "$TEST_CXP"

echo ""
echo "Migration completed!"
echo ""

# Show info about created CXP file
echo "CXP file information:"
cargo run --package cxp-cli -- info "$TEST_CXP"

echo ""
echo "Test completed successfully!"
echo ""
echo "Cleanup:"
echo "  rm $TEST_DB $TEST_CXP"

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::models::FileFormat;

/// 履歴エントリ
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HistoryEntry {
    pub id: String,
    pub source_folder: String,
    pub output_format: FileFormat,
    pub cleanup_enabled: bool,
    pub executed_at: DateTime<Utc>,
    pub status: HistoryStatus,
    pub file_count: u32,
}

/// 履歴ステータス
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum HistoryStatus {
    Success,
    Failed,
}

/// 履歴ストア（最大50件）
pub struct HistoryStore {
    pub entries: Vec<HistoryEntry>,
    pub max_entries: usize,
    /// 永続化先のファイルパス
    storage_path: Option<PathBuf>,
}

impl Default for HistoryStore {
    fn default() -> Self {
        Self {
            entries: Vec::new(),
            max_entries: 50,
            storage_path: None,
        }
    }
}

impl HistoryStore {
    pub fn new() -> Self {
        Self::default()
    }

    /// 永続化パスを指定して作成し、既存データを読み込む
    pub fn with_storage_path(path: PathBuf) -> Self {
        let mut store = Self {
            entries: Vec::new(),
            max_entries: 50,
            storage_path: Some(path),
        };
        store.load();
        store
    }

    /// 履歴エントリを追加（最大件数を超えた場合は古いものを削除）
    pub fn add_entry(&mut self, entry: HistoryEntry) {
        self.entries.insert(0, entry);
        if self.entries.len() > self.max_entries {
            self.entries.truncate(self.max_entries);
        }
        self.save();
    }

    /// 全履歴をクリア
    pub fn clear(&mut self) {
        self.entries.clear();
        self.save();
    }

    /// 履歴一覧を取得（実行日時の降順）
    pub fn get_entries(&self) -> &[HistoryEntry] {
        &self.entries
    }

    /// 全履歴を取得（get_entries のエイリアス）
    pub fn get_all(&self) -> &[HistoryEntry] {
        &self.entries
    }

    /// ファイルから履歴を読み込む
    fn load(&mut self) {
        if let Some(ref path) = self.storage_path {
            if path.exists() {
                if let Ok(content) = fs::read_to_string(path) {
                    if let Ok(entries) = serde_json::from_str::<Vec<HistoryEntry>>(&content) {
                        self.entries = entries;
                        // 最大件数を超えている場合はtruncate
                        if self.entries.len() > self.max_entries {
                            self.entries.truncate(self.max_entries);
                        }
                    }
                }
            }
        }
    }

    /// 履歴をファイルに保存する
    fn save(&self) {
        if let Some(ref path) = self.storage_path {
            // 親ディレクトリが存在しない場合は作成
            if let Some(parent) = path.parent() {
                let _ = fs::create_dir_all(parent);
            }
            if let Ok(json) = serde_json::to_string_pretty(&self.entries) {
                let _ = fs::write(path, json);
            }
        }
    }
}

/// アプリケーション状態として管理する履歴ステート
pub struct HistoryState {
    pub store: Mutex<HistoryStore>,
}

impl HistoryState {
    /// app_data_dir のパスを受け取り、history.json を永続化先として初期化する
    pub fn new(app_data_dir: &Path) -> Self {
        let storage_path = app_data_dir.join("history.json");
        let store = HistoryStore::with_storage_path(storage_path);
        Self {
            store: Mutex::new(store),
        }
    }

    /// テスト用: 永続化なしで作成
    #[allow(dead_code)]
    pub fn in_memory() -> Self {
        Self {
            store: Mutex::new(HistoryStore::new()),
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn make_entry(id: &str) -> HistoryEntry {
        HistoryEntry {
            id: id.to_string(),
            source_folder: "/tmp/test".to_string(),
            output_format: crate::models::FileFormat::Png,
            cleanup_enabled: false,
            executed_at: Utc::now(),
            status: HistoryStatus::Success,
            file_count: 5,
        }
    }

    #[test]
    fn test_add_entry_fifo_max_50() {
        let mut store = HistoryStore::new();
        assert_eq!(store.max_entries, 50);

        // Add 55 entries
        for i in 0..55 {
            store.add_entry(make_entry(&format!("entry-{}", i)));
        }

        // Should be capped at 50
        assert_eq!(store.entries.len(), 50);

        // Most recent entry should be first
        assert_eq!(store.entries[0].id, "entry-54");
        // Oldest surviving entry should be entry-5 (entries 0-4 were truncated)
        assert_eq!(store.entries[49].id, "entry-5");
    }

    #[test]
    fn test_clear() {
        let mut store = HistoryStore::new();
        store.add_entry(make_entry("a"));
        store.add_entry(make_entry("b"));
        assert_eq!(store.entries.len(), 2);

        store.clear();
        assert_eq!(store.entries.len(), 0);
    }

    #[test]
    fn test_persistence_save_and_load() {
        let tmp = TempDir::new().unwrap();
        let storage_path = tmp.path().join("history.json");

        // Create store, add entries, it should auto-save
        {
            let mut store = HistoryStore::with_storage_path(storage_path.clone());
            store.add_entry(make_entry("persist-1"));
            store.add_entry(make_entry("persist-2"));
        }

        // Verify the file was written
        assert!(storage_path.exists());

        // Create a new store from the same path - should load entries
        let store = HistoryStore::with_storage_path(storage_path);
        assert_eq!(store.entries.len(), 2);
        assert_eq!(store.entries[0].id, "persist-2");
        assert_eq!(store.entries[1].id, "persist-1");
    }

    #[test]
    fn test_persistence_clear_removes_file_content() {
        let tmp = TempDir::new().unwrap();
        let storage_path = tmp.path().join("history.json");

        {
            let mut store = HistoryStore::with_storage_path(storage_path.clone());
            store.add_entry(make_entry("x"));
            store.add_entry(make_entry("y"));
        }

        // Clear and reload
        {
            let mut store = HistoryStore::with_storage_path(storage_path.clone());
            assert_eq!(store.entries.len(), 2);
            store.clear();
        }

        // Reloading should show empty
        let store = HistoryStore::with_storage_path(storage_path);
        assert_eq!(store.entries.len(), 0);
    }

    #[test]
    fn test_history_state_new() {
        let tmp = TempDir::new().unwrap();
        let state = HistoryState::new(tmp.path());

        let store = state.store.lock().unwrap();
        assert_eq!(store.entries.len(), 0);
    }

    #[test]
    fn test_history_state_add_and_get() {
        let tmp = TempDir::new().unwrap();
        let state = HistoryState::new(tmp.path());

        {
            let mut store = state.store.lock().unwrap();
            store.add_entry(make_entry("state-1"));
        }

        let store = state.store.lock().unwrap();
        assert_eq!(store.get_all().len(), 1);
        assert_eq!(store.get_all()[0].id, "state-1");
    }
}

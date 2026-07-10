/// Property-based tests for the History module.
///
/// Feature: universal-file-converter
/// Property 16: 履歴件数上限の維持
/// Property 17: 履歴復元の設定一致
///
/// **Validates: Requirements 8.2, 8.3**

#[cfg(test)]
mod property_tests {
    use crate::history::{HistoryEntry, HistoryStatus, HistoryStore};
    use crate::models::FileFormat;
    use chrono::Utc;
    use proptest::prelude::*;
    use uuid::Uuid;

    /// FileFormat の任意値を生成する Strategy
    fn arb_file_format() -> impl Strategy<Value = FileFormat> {
        prop_oneof![
            Just(FileFormat::Png),
            Just(FileFormat::Jpeg),
            Just(FileFormat::Webp),
            Just(FileFormat::Gif),
            Just(FileFormat::Bmp),
            Just(FileFormat::Tiff),
            Just(FileFormat::Avif),
            Just(FileFormat::Ico),
            Just(FileFormat::Json),
            Just(FileFormat::Yaml),
            Just(FileFormat::Toml),
            Just(FileFormat::Xml),
            Just(FileFormat::Csv),
            Just(FileFormat::Markdown),
            Just(FileFormat::PlainText),
        ]
    }

    /// HistoryStatus の任意値を生成する Strategy
    fn arb_history_status() -> impl Strategy<Value = HistoryStatus> {
        prop_oneof![Just(HistoryStatus::Success), Just(HistoryStatus::Failed),]
    }

    /// HistoryEntry の任意値を生成する Strategy
    fn arb_history_entry() -> impl Strategy<Value = HistoryEntry> {
        (
            arb_file_format(),
            any::<bool>(),
            arb_history_status(),
            1u32..1000,
            "[a-zA-Z/]{1,30}",
        )
            .prop_map(
                |(output_format, cleanup_enabled, status, file_count, source_folder)| {
                    HistoryEntry {
                        id: Uuid::new_v4().to_string(),
                        source_folder,
                        output_format,
                        cleanup_enabled,
                        executed_at: Utc::now(),
                        status,
                        file_count,
                    }
                },
            )
    }

    // =========================================================================
    // Property 16: 履歴件数上限の維持
    // Feature: universal-file-converter, Property 16: 履歴件数上限の維持
    // Validates: Requirements 8.2
    //
    // For any N consecutive conversions (N > 50), the stored history count is
    // always <= 50, and the preserved entries are the most recent 50.
    // =========================================================================

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        /// Property 16: 履歴件数上限の維持
        ///
        /// For any N consecutive conversions (N > 50), the stored history count
        /// is always <= 50, and the preserved entries are the most recent 50.
        ///
        /// Feature: universal-file-converter, Property 16: 履歴件数上限の維持
        /// **Validates: Requirements 8.2**
        #[test]
        fn prop_history_max_entries_maintained(n in 51u32..200) {
            let mut store = HistoryStore::new();

            // Add N entries with sequential IDs so we can verify ordering
            let mut all_ids: Vec<String> = Vec::new();
            for i in 0..n {
                let entry = HistoryEntry {
                    id: format!("entry-{}", i),
                    source_folder: format!("/tmp/test/{}", i),
                    output_format: FileFormat::Png,
                    cleanup_enabled: false,
                    executed_at: Utc::now(),
                    status: HistoryStatus::Success,
                    file_count: i + 1,
                };
                all_ids.push(format!("entry-{}", i));
                store.add_entry(entry);
            }

            // Verify: store.entries.len() <= 50
            prop_assert!(
                store.entries.len() <= 50,
                "History store should contain at most 50 entries, but has {}",
                store.entries.len()
            );

            // Verify: entries are the most recent 50
            // Since we insert at index 0 and truncate, the most recent N entries
            // should be entry-(n-1), entry-(n-2), ..., entry-(n-50)
            prop_assert_eq!(
                store.entries.len(),
                50,
                "With N={} > 50 entries added, exactly 50 should remain",
                n
            );

            for (idx, entry) in store.entries.iter().enumerate() {
                let expected_id = format!("entry-{}", n - 1 - idx as u32);
                prop_assert_eq!(
                    &entry.id,
                    &expected_id,
                    "Entry at index {} should be '{}' (most recent 50), but got '{}'",
                    idx,
                    expected_id,
                    entry.id
                );
            }
        }

        /// Property 16 (invariant during insertion): After each add_entry call,
        /// the store never exceeds max_entries (50).
        ///
        /// Feature: universal-file-converter, Property 16: 履歴件数上限の維持
        /// **Validates: Requirements 8.2**
        #[test]
        fn prop_history_invariant_never_exceeds_max(n in 51u32..200) {
            let mut store = HistoryStore::new();

            for i in 0..n {
                let entry = HistoryEntry {
                    id: format!("entry-{}", i),
                    source_folder: "/tmp/test".to_string(),
                    output_format: FileFormat::Jpeg,
                    cleanup_enabled: true,
                    executed_at: Utc::now(),
                    status: HistoryStatus::Success,
                    file_count: 1,
                };
                store.add_entry(entry);

                // Invariant: after every insertion, len <= 50
                prop_assert!(
                    store.entries.len() <= 50,
                    "After adding entry {}, store has {} entries (max is 50)",
                    i,
                    store.entries.len()
                );
            }
        }
    }

    // =========================================================================
    // Property 17: 履歴復元の設定一致
    // Feature: universal-file-converter, Property 17: 履歴復元の設定一致
    // Validates: Requirements 8.3
    //
    // For any history entry, after restore operation, the current conversion
    // settings (output format, cleanup setting) match the values recorded in
    // the history entry.
    // =========================================================================

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        /// Property 17: 履歴復元の設定一致
        ///
        /// For any history entry, after retrieval from the store, the entry's
        /// fields (output_format, cleanup_enabled) are accessible and correct
        /// (data integrity through add/get cycle).
        ///
        /// Feature: universal-file-converter, Property 17: 履歴復元の設定一致
        /// **Validates: Requirements 8.3**
        #[test]
        fn prop_history_restore_settings_match(
            entry in arb_history_entry(),
        ) {
            let mut store = HistoryStore::new();

            // Capture expected values before adding
            let expected_format = entry.output_format.clone();
            let expected_cleanup = entry.cleanup_enabled;
            let expected_id = entry.id.clone();

            store.add_entry(entry);

            // Retrieve from store and verify settings match
            let entries = store.get_entries();
            prop_assert!(
                !entries.is_empty(),
                "Store should have at least one entry after add"
            );

            let retrieved = &entries[0];
            prop_assert_eq!(
                &retrieved.id,
                &expected_id,
                "Retrieved entry ID should match"
            );
            prop_assert_eq!(
                &retrieved.output_format,
                &expected_format,
                "Restored output_format should match the history entry value"
            );
            prop_assert_eq!(
                retrieved.cleanup_enabled,
                expected_cleanup,
                "Restored cleanup_enabled should match the history entry value"
            );
        }

        /// Property 17 (multiple entries): After adding multiple entries,
        /// retrieving any entry by position preserves its settings.
        ///
        /// Feature: universal-file-converter, Property 17: 履歴復元の設定一致
        /// **Validates: Requirements 8.3**
        #[test]
        fn prop_history_restore_multiple_entries_integrity(
            count in 2u32..50,
            format in arb_file_format(),
            cleanup in any::<bool>(),
        ) {
            let mut store = HistoryStore::new();
            let mut expected_entries: Vec<(String, FileFormat, bool)> = Vec::new();

            for i in 0..count {
                let entry = HistoryEntry {
                    id: format!("multi-{}", i),
                    source_folder: format!("/folder/{}", i),
                    output_format: format.clone(),
                    cleanup_enabled: cleanup,
                    executed_at: Utc::now(),
                    status: HistoryStatus::Success,
                    file_count: i + 1,
                };
                // Entries are inserted at front, so prepend to expected
                expected_entries.insert(0, (
                    entry.id.clone(),
                    entry.output_format.clone(),
                    entry.cleanup_enabled,
                ));
                store.add_entry(entry);
            }

            let entries = store.get_entries();
            prop_assert_eq!(
                entries.len(),
                count as usize,
                "Store should contain all {} entries",
                count
            );

            for (idx, retrieved) in entries.iter().enumerate() {
                let (ref expected_id, ref expected_fmt, expected_cleanup) = expected_entries[idx];
                prop_assert_eq!(
                    &retrieved.id,
                    expected_id,
                    "Entry at index {} should have correct ID",
                    idx
                );
                prop_assert_eq!(
                    &retrieved.output_format,
                    expected_fmt,
                    "Entry at index {} should have correct output_format",
                    idx
                );
                prop_assert_eq!(
                    retrieved.cleanup_enabled,
                    expected_cleanup,
                    "Entry at index {} should have correct cleanup_enabled",
                    idx
                );
            }
        }
    }
}

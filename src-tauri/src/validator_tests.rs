/// Property-based tests for the Validator module.
///
/// Feature: universal-file-converter, Property 14: ディスク容量事前検証
///
/// **Validates: Requirements 10.4**
///
/// For any input file group total size S and output destination available space A,
/// if S > A then conversion should not start and an error containing the required
/// capacity and available space should be returned.
#[cfg(test)]
mod property_tests {
    use crate::errors::ConversionError;
    use crate::validator::Validator;
    use proptest::prelude::*;
    use tempfile::TempDir;

    // Helper: get the actual available space for a temp directory
    fn get_available_space_for_temp_dir(dir: &std::path::Path) -> u64 {
        // Use the same logic as Validator to get available space.
        // We call check_disk_space with 0 bytes to confirm it returns Ok,
        // then use a very large value to get the error containing available_bytes.
        match Validator::check_disk_space(u64::MAX, dir) {
            Err(ConversionError::DiskSpaceError {
                available_bytes, ..
            }) => available_bytes,
            // If somehow u64::MAX passes (extremely unlikely), fallback to 0
            Ok(()) => u64::MAX,
            _ => 0,
        }
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        /// Property 14: ディスク容量事前検証
        ///
        /// When input_size > available_space, check_disk_space should return
        /// a DiskSpaceError containing both required_bytes and available_bytes.
        ///
        /// Feature: universal-file-converter, Property 14: ディスク容量事前検証
        /// **Validates: Requirements 10.4**
        #[test]
        fn prop_disk_space_insufficient_returns_error(input_size in 1u64..=u64::MAX) {
            let temp_dir = TempDir::new().expect("Failed to create temp dir");
            let available = get_available_space_for_temp_dir(temp_dir.path());

            // Only test cases where input_size > available
            if input_size > available {
                let result = Validator::check_disk_space(input_size, temp_dir.path());

                // Must be an error
                prop_assert!(result.is_err(), "Expected DiskSpaceError when input_size ({}) > available ({})", input_size, available);

                match result.unwrap_err() {
                    ConversionError::DiskSpaceError {
                        required_bytes,
                        available_bytes,
                    } => {
                        // Error must contain correct required_bytes
                        prop_assert_eq!(required_bytes, input_size,
                            "required_bytes should equal input_size");
                        // Error must contain correct available_bytes
                        prop_assert_eq!(available_bytes, available,
                            "available_bytes should match actual available space");
                    }
                    other => {
                        prop_assert!(false, "Expected DiskSpaceError but got: {:?}", other);
                    }
                }
            }
        }

        /// Property 14 (complement): When input_size <= available_space,
        /// check_disk_space should return Ok(()).
        ///
        /// Feature: universal-file-converter, Property 14: ディスク容量事前検証
        /// **Validates: Requirements 10.4**
        #[test]
        fn prop_disk_space_sufficient_returns_ok(input_size in 0u64..=(1024 * 1024 * 1024)) {
            let temp_dir = TempDir::new().expect("Failed to create temp dir");
            let available = get_available_space_for_temp_dir(temp_dir.path());

            // Only test cases where input_size <= available
            if input_size <= available {
                let result = Validator::check_disk_space(input_size, temp_dir.path());
                prop_assert!(result.is_ok(),
                    "Expected Ok(()) when input_size ({}) <= available ({}), but got: {:?}",
                    input_size, available, result);
            }
        }

        /// Property 14 (boundary): Generate sizes relative to actual available space
        /// to ensure we hit the boundary conditions properly.
        ///
        /// Feature: universal-file-converter, Property 14: ディスク容量事前検証
        /// **Validates: Requirements 10.4**
        #[test]
        fn prop_disk_space_boundary_behavior(offset in 0u64..1000) {
            let temp_dir = TempDir::new().expect("Failed to create temp dir");
            let available = get_available_space_for_temp_dir(temp_dir.path());

            // Test: available + 1 + offset should always fail
            let input_size = available.saturating_add(1).saturating_add(offset);
            if input_size > available {
                let result = Validator::check_disk_space(input_size, temp_dir.path());
                prop_assert!(result.is_err(),
                    "Expected DiskSpaceError when input_size ({}) > available ({})",
                    input_size, available);

                match result.unwrap_err() {
                    ConversionError::DiskSpaceError {
                        required_bytes,
                        available_bytes,
                    } => {
                        prop_assert_eq!(required_bytes, input_size);
                        prop_assert_eq!(available_bytes, available);
                    }
                    other => {
                        prop_assert!(false, "Expected DiskSpaceError but got: {:?}", other);
                    }
                }
            }

            // Test: values from 0 to min(available, 100) should pass
            let safe_size = std::cmp::min(available, offset);
            let result = Validator::check_disk_space(safe_size, temp_dir.path());
            prop_assert!(result.is_ok(),
                "Expected Ok(()) when input_size ({}) <= available ({}), but got: {:?}",
                safe_size, available, result);
        }
    }
}

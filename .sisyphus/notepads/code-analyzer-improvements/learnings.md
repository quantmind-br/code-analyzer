
## Single File Analysis Support (2026-01-24)

### Implementation Pattern
- `cli.rs:validate()` - removed `is_dir()` check, kept `exists()` check
- `walker.rs:discover_files()` - added early return for `is_file()` case
- `walker.rs:discover_single_file()` - new helper method for single file handling

### Key Points
- Single file path check comes BEFORE directory walker initialization
- `is_supported_file()` from LanguageManager for extension validation
- File size check uses same `filter_config.max_file_size_bytes` as directory walker
- Tests split: `test_discover_files_single_supported_file` (success) and `test_discover_files_single_unsupported_file` (error)
## 2026-01-24: Adding SortBy::Methods

### Pattern: Adding new CLI sort option
1. Add variant to `SortBy` enum in `src/cli.rs`
2. Add match arm in `Display` impl for SortBy
3. Add match arm in `apply_sorting()` in `src/output/terminal.rs`
4. clap derive API auto-generates CLI options from enum variants

### Pre-existing issues found
- `test_validate_file_instead_of_directory` test fails - validate() was changed to allow files but test still expects error
- `max_nesting_depth` field was added to FileAnalysis but many test fixtures were not updated
- clippy error in walker.rs for field_reassign_with_default


# Learnings

## 2026-01-24 Task 0: Baseline
- Test count: 120 unit + 16 integration = 136 tests
- Clippy: 0 errors, 0 warnings
- 1 expected cfg warning for cli_tests feature (not a real issue)
- make quality passes cleanly

## 2026-01-24 Task 1: LanguageSpec refactoring
- Created `LanguageSpec` struct with 8 fields for node kinds
- 8 static constants: RUST_SPEC, JS_SPEC, TS_SPEC, PYTHON_SPEC, JAVA_SPEC, C_SPEC, CPP_SPEC, GO_SPEC
- TypeScript and Tsx share TS_SPEC (via pattern match in spec())
- C and Cpp have separate specs (different control_flow and nesting nodes)
- NodeKindMapper impl reduced from ~270 lines to ~54 lines (5x reduction)
- Pattern: delegate to `self.spec().field_name` for all getter methods
- Characterization test added: covers all 9 languages × all NodeKindMapper methods
- Test count: 137 (121 unit + 16 integration) - +1 new characterization test

## 2026-01-24 Completion
- All 4 tasks completed successfully.
- Final test count: 137 tests.
- Performance optimization (cq-005) verified with correct thread-local parser reuse.
- Codebase structure improved significantly (LanguageSpec, sanitizer module).

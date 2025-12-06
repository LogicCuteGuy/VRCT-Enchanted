# Design Document

## Overview

This design document outlines the approach for removing the legacy `useStdoutToPython` hook and all related Python communication code from the VRCT frontend. After the successful migration from Python to Rust backend, these legacy communication hooks are now no-ops that do nothing. The cleanup will simplify the codebase by removing dead code and updating components to use Tauri commands directly.

## Architecture

### Current State

The frontend currently has a legacy communication layer that was used to communicate with the Python backend:

```
┌─────────────────────────────────────────────────────────────┐
│                     React Components                         │
└────────────────────────┬────────────────────────────────────┘
                         │
┌────────────────────────▼────────────────────────────────────┐
│              useStdoutToPython Hook (DEAD CODE)              │
│  - asyncStdoutToPython() - now a no-op                      │
│  - All calls do nothing                                      │
└────────────────────────┬────────────────────────────────────┘
                         │ (no actual communication)
                         ▼
                    [Nothing]
```

### Target State

After cleanup, components will communicate directly with the Rust backend via Tauri commands:

```
┌─────────────────────────────────────────────────────────────┐
│                     React Components                         │
└────────────────────────┬────────────────────────────────────┘
                         │ invoke("command_name", args)
                         │
┌────────────────────────▼────────────────────────────────────┐
│                   Tauri IPC Layer                            │
│              (@tauri-apps/api/core)                          │
└────────────────────────┬────────────────────────────────────┘
                         │
┌────────────────────────▼────────────────────────────────────┐
│                   Rust Backend                               │
│              (src-tauri/src/commands.rs)                     │
└─────────────────────────────────────────────────────────────┘
```

## Components and Interfaces

### Files to Delete

1. `src-ui/logics/common/useStdoutToPython.js` - The legacy hook file

### Files to Modify

The following files import and use `useStdoutToPython`:

| File | Current Usage | Replacement Strategy |
|------|---------------|---------------------|
| `src-ui/logics/main/useLanguageSettings.js` | Multiple get/set calls | Use Tauri commands or BackendInitController data |
| `src-ui/logics/common/useUpdateSoftware.js` | `/run/update_software` | Use `invoke("download_update")` |
| `src-ui/logics/main/useIsMainPageCompactMode.js` | get/set compact mode | Use Tauri commands |
| `src-ui/logics/main/useMessageInputBoxRatio.js` | get/set message box ratio | Use Tauri commands |
| `src-ui/logics/common/useLLMConnection.js` | LM Studio/Ollama connection | Use Tauri commands |
| `src-ui/logics/common/useOpenFolder.js` | Open folder commands | Use Tauri commands |
| `src-ui/logics/common/useMessage.js` | Send message, typing indicators | Use Tauri commands |
| `src-ui/logics/common/useSoftwareVersion.js` | Get version | Use Tauri commands |
| `src-ui/logics/common/useVolume.js` | Mic/speaker threshold | Use Tauri commands |
| `src-ui/logics/common/useWindow.js` | Window geometry | Use Tauri commands |
| `src-ui/logics/configs/config_page_setter/useSettingsLogics.js` | Generic settings | Use Tauri commands |
| `src-ui/logics/configs/config_page_setter/hotkeys/useHotkeys.js` | Hotkey settings | Use Tauri commands |
| `src-ui/logics/configs/config_page_setter/plugins/usePlugins.js` | Plugin settings | Use Tauri commands |
| `src-ui/views/app/config_page/setting_section/setting_box/advanced_settings/AdvancedSettings.jsx` | ZLUDA info | Use Tauri commands |
| `vite.config.js` | `@useStdoutToPython` alias | Remove alias |

### Endpoint to Tauri Command Mapping

| Legacy Endpoint | Tauri Command | Notes |
|-----------------|---------------|-------|
| `/run/send_message_box` | `send_message_box` | Already exists |
| `/run/typing_message_box` | `start_typing` | Need to add |
| `/run/stop_typing_message_box` | `stop_typing` | Need to add |
| `/run/update_software` | `download_update` | Already exists |
| `/run/update_cuda_software` | `download_update` | Same command with param |
| `/set/enable/check_mic_threshold` | `enable_mic_threshold_check` | Need to add |
| `/set/disable/check_mic_threshold` | `disable_mic_threshold_check` | Need to add |
| `/set/enable/check_speaker_threshold` | `enable_speaker_threshold_check` | Need to add |
| `/set/disable/check_speaker_threshold` | `disable_speaker_threshold_check` | Need to add |
| `/get/data/version` | `get_version` | Already exists |
| `/run/open_filepath_logs` | `open_logs_folder` | Already exists |
| `/run/open_filepath_config_file` | `open_config_folder` | Need to add |
| `/get/data/selected_tab_no` | Data from BackendInitController | Already loaded at startup |
| `/set/data/selected_tab_no` | `set_config` | Use generic config setter |
| `/get/data/selected_your_languages` | Data from BackendInitController | Already loaded at startup |
| `/set/data/selected_your_languages` | `set_config` | Use generic config setter |
| `/get/data/selected_target_languages` | Data from BackendInitController | Already loaded at startup |
| `/set/data/selected_target_languages` | `set_config` | Use generic config setter |
| `/get/data/selectable_translation_engines` | Data from BackendInitController | Already loaded at startup |
| `/get/data/selected_translation_engines` | Data from BackendInitController | Already loaded at startup |
| `/set/data/selected_translation_engines` | `set_translation_engine` | Already exists |
| `/run/swap_your_language_and_target_language` | `swap_languages` | Need to add |
| `/get/data/selectable_language_list` | Data from BackendInitController | Already loaded at startup |
| `/run/lmstudio_connection` | `check_lmstudio_connection` | Need to add |
| `/run/ollama_connection` | `check_ollama_connection` | Need to add |
| `/get/data/main_window_sidebar_compact_mode` | Data from BackendInitController | Already loaded at startup |
| `/set/enable/main_window_sidebar_compact_mode` | `set_config` | Use generic config setter |
| `/set/disable/main_window_sidebar_compact_mode` | `set_config` | Use generic config setter |
| `/get/data/message_box_ratio` | Data from BackendInitController | Already loaded at startup |
| `/set/data/message_box_ratio` | `set_config` | Use generic config setter |
| `/set/data/main_window_geometry` | `set_config` | Use generic config setter |
| `/get/data/hotkeys` | Data from BackendInitController | Already loaded at startup |
| `/set/data/hotkeys` | `set_config` | Use generic config setter |
| `/get/data/zluda_installation_info` | `get_zluda_info` | Need to add |

## Data Models

No new data models are required. The existing Tauri command response format will be used:

```typescript
interface Response {
  status: number;
  endpoint: string;
  result: any;
}
```

## Error Handling

- Components that previously called `asyncStdoutToPython` should handle errors from Tauri commands using try/catch
- Error responses from Tauri commands follow the existing `Response` structure with `status !== 200`
- UI should display appropriate error messages using the existing notification system

## Testing Strategy

### Manual Testing

Since most acceptance criteria require verifying that functionality is preserved (Requirements 4.1-4.4), manual testing will be the primary verification method:

1. **UI Interaction Testing** (Requirement 4.1): Test all UI interactions that previously used `asyncStdoutToPython` to ensure same behavior
2. **Settings Persistence Testing** (Requirement 4.2): Verify settings changes are persisted correctly to the Rust backend
3. **Transcription/Translation Testing** (Requirement 4.3): Verify transcription and translation features work via Tauri commands
4. **Communication Testing** (Requirement 4.4): Verify OSC and WebSocket features work correctly

### Automated Verification

The following can be verified automatically after cleanup:

| Verification | Requirement | Method |
|--------------|-------------|--------|
| No imports of `useStdoutToPython` remain | 1.1 | grep search |
| No calls to `asyncStdoutToPython` remain | 1.2 | grep search |
| `useStdoutToPython.js` file is deleted | 1.3 | file existence check |
| `@useStdoutToPython` alias removed from `vite.config.js` | 1.4 | content check |
| No Python-related comments remain | 5.1 | grep search |
| Build succeeds without errors | All | npm build |

### Code Review Verification

The following require code review to verify:

| Verification | Requirement |
|--------------|-------------|
| Components use BackendInitController or Tauri commands for GET operations | 2.1 |
| Components use Tauri commands for SET operations | 2.2 |
| Components use Tauri commands for RUN operations | 2.3 |
| Components use Tauri commands for enable/disable operations | 2.4 |
| Hooks are properly refactored or removed | 3.1, 3.2 |
| User-facing functionality is maintained | 3.3 |
| Variable/function names reference Rust backend | 5.2 |
| Documentation references Rust backend | 5.3 |

## Correctness Properties

*A property is a characteristic or behavior that should hold true across all valid executions of a system-essentially, a formal statement about what the system should do. Properties serve as the bridge between human-readable specifications and machine-verifiable correctness guarantees.*

### Prework Analysis Summary

After analyzing all 18 acceptance criteria across 5 requirements, the criteria fall into these categories:

1. **One-time verification checks** (Requirements 1.1-1.4, 5.1): These can be verified via grep searches and file existence checks, but are examples rather than properties since they verify a specific end state, not a rule that holds across inputs.

2. **Refactoring guidelines** (Requirements 2.1-2.4, 3.1-3.3): These describe how code should be restructured but are not testable properties - they require code review and successful compilation.

3. **Behavioral preservation** (Requirements 4.1-4.4): These require manual testing to verify the application continues to function correctly after refactoring.

4. **Documentation/naming requirements** (Requirements 5.2-5.3): These require human judgment and code review.

### Conclusion

No property-based tests are applicable for this cleanup task. This is expected for a refactoring effort where the goal is removing dead code rather than implementing new functionality with invariants.

The primary verification methods are:
1. **Build verification**: The application compiles without errors
2. **Grep verification**: No references to legacy code remain (Requirements 1.1, 1.2, 5.1)
3. **File verification**: Legacy files are deleted (Requirement 1.3)
4. **Manual testing**: Application functions correctly (Requirements 4.1-4.4)

## Migration Strategy

### Phase 1: Add Missing Tauri Commands

Before removing the legacy code, ensure all required Tauri commands exist (supports Requirements 2.1-2.4):

1. Add `start_typing` and `stop_typing` commands
2. Add `enable_mic_threshold_check` and `disable_mic_threshold_check` commands
3. Add `enable_speaker_threshold_check` and `disable_speaker_threshold_check` commands
4. Add `open_config_folder` command
5. Add `swap_languages` command
6. Add `check_lmstudio_connection` and `check_ollama_connection` commands
7. Add `get_zluda_info` command

### Phase 2: Update Frontend Components

Update each file that uses `useStdoutToPython` (supports Requirements 2.1-2.4, 3.1-3.3):

1. Remove the import of `useStdoutToPython`
2. Add import for `invoke` from `@tauri-apps/api/core`
3. Replace `asyncStdoutToPython` calls with appropriate `invoke` calls
4. For data that's already loaded at startup, use the store directly
5. Simplify or remove hooks that only wrapped asyncStdoutToPython calls

### Phase 3: Remove Legacy Code

Remove all legacy Python communication code (supports Requirements 1.1-1.4):

1. Delete `src-ui/logics/common/useStdoutToPython.js` (Requirement 1.3)
2. Remove `@useStdoutToPython` alias from `vite.config.js` (Requirement 1.4)
3. Verify no imports remain (Requirement 1.1)
4. Verify no calls remain (Requirement 1.2)

### Phase 4: Cleanup

Remove Python references from codebase (supports Requirements 5.1-5.3):

1. Remove Python-related comments (Requirement 5.1)
2. Rename variables/functions that reference Python/Stdout (Requirement 5.2)
3. Update documentation to reference Rust backend (Requirement 5.3)

### Phase 5: Verification

Verify application functionality is preserved (supports Requirements 4.1-4.4):

1. Test all UI interactions work correctly (Requirement 4.1)
2. Verify settings persistence (Requirement 4.2)
3. Test transcription and translation (Requirement 4.3)
4. Test OSC and WebSocket communication (Requirement 4.4)

## Requirements Traceability

| Requirement | Design Section | Implementation Approach |
|-------------|----------------|------------------------|
| 1.1 No useStdoutToPython imports | Files to Modify, Phase 3 | Remove imports from all listed files |
| 1.2 No asyncStdoutToPython calls | Files to Modify, Phase 2 | Replace calls with Tauri invoke |
| 1.3 Delete useStdoutToPython.js | Files to Delete, Phase 3 | Delete file after all usages removed |
| 1.4 Remove vite alias | Files to Modify, Phase 3 | Remove @useStdoutToPython alias |
| 2.1 GET endpoints use Tauri/store | Endpoint Mapping, Phase 2 | Use BackendInitController or invoke |
| 2.2 SET endpoints use Tauri | Endpoint Mapping, Phase 2 | Use invoke with set_config |
| 2.3 RUN endpoints use Tauri | Endpoint Mapping, Phase 2 | Use invoke with action commands |
| 2.4 Enable/disable use Tauri | Endpoint Mapping, Phase 2 | Use invoke with toggle commands |
| 3.1 Refactor wrapper hooks | Files to Modify, Phase 2 | Inline Tauri calls or remove hooks |
| 3.2 Update hook consumers | Files to Modify, Phase 2 | Update all component imports |
| 3.3 Maintain functionality | Testing Strategy, Phase 5 | Manual testing of all features |
| 4.1 UI interactions work | Testing Strategy, Phase 5 | Manual UI testing |
| 4.2 Settings persist | Testing Strategy, Phase 5 | Test config save/load |
| 4.3 Transcription/translation work | Testing Strategy, Phase 5 | Test speech features |
| 4.4 OSC/WebSocket work | Testing Strategy, Phase 5 | Test communication features |
| 5.1 No Python comments | Phase 4 | Search and remove comments |
| 5.2 Rename Python references | Phase 4 | Rename variables/functions |
| 5.3 Update documentation | Phase 4 | Update docs to reference Rust |

## Risks and Mitigations

| Risk | Mitigation |
|------|------------|
| Breaking existing functionality | Test each component after refactoring (Req 4.1-4.4) |
| Missing Tauri commands | Add commands before removing legacy code (Phase 1) |
| Build failures | Verify build after each change |
| Runtime errors | Test in development before committing |
| Incomplete cleanup | Use grep verification to find remaining references (Req 1.1, 1.2, 5.1) |


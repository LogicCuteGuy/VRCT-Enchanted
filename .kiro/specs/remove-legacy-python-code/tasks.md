# Implementation Plan

- [x] 1. Add missing Tauri commands to Rust backend





  - [x] 1.1 Add typing indicator commands (`start_typing`, `stop_typing`)


    - Add commands to `src-tauri/src/commands.rs`
    - Register commands in `src-tauri/src/lib.rs`
    - _Requirements: 2.3_
  - [x] 1.2 Add threshold check commands (`enable_mic_threshold_check`, `disable_mic_threshold_check`, `enable_speaker_threshold_check`, `disable_speaker_threshold_check`)

    - Add commands to `src-tauri/src/commands.rs`
    - Register commands in `src-tauri/src/lib.rs`
    - _Requirements: 2.4_

  - [x] 1.3 Add folder and utility commands (`open_config_folder`, `swap_languages`, `get_zluda_info`)
    - Add commands to `src-tauri/src/commands.rs`
    - Register commands in `src-tauri/src/lib.rs`
    - _Requirements: 2.3_
  - [x] 1.4 Add LLM connection check commands (`check_lmstudio_connection`, `check_ollama_connection`)
    - Add commands to `src-tauri/src/commands.rs`
    - Register commands in `src-tauri/src/lib.rs`
    - _Requirements: 2.3_

- [x] 2. Update language settings hook






  - [x] 2.1 Refactor `useLanguageSettings.js` to use Tauri commands

    - Remove `useStdoutToPython` import
    - Replace GET calls with BackendInitController data or Tauri invoke
    - Replace SET calls with Tauri invoke
    - _Requirements: 1.1, 1.2, 2.1, 2.2, 3.1_

- [x] 3. Up date software update hook






  - [x] 3.1 Refactor `useUpdateSoftware.js` to use Tauri commands

    - Remove `useStdoutToPython` import
    - Replace `/run/update_software` with `invoke("download_update")`
    - _Requirements: 1.1, 1.2, 2.3_

- [x] 4. Update UI state hooks






  - [x] 4.1 Refactor `useIsMainPageCompactMode.js` to use Tauri commands

    - Remove `useStdoutToPython` import
    - Use BackendInitController data for GET, Tauri invoke for SET
    - _Requirements: 1.1, 1.2, 2.1, 2.2_
  - [x] 4.2 Refactor `useMessageInputBoxRatio.js` to use Tauri commands


    - Remove `useStdoutToPython` import
    - Use BackendInitController data for GET, Tauri invoke for SET
    - _Requirements: 1.1, 1.2, 2.1, 2.2_

- [x] 5. Update LLM connection hook





  - [x] 5.1 Refactor `useLLMConnection.js` to use Tauri commands


    - Remove `useStdoutToPython` import
    - Replace connection check calls with Tauri invoke
    - _Requirements: 1.1, 1.2, 2.3_

- [x] 6. Update folder and utility hooks







  - [x] 6.1 Refactor `useOpenFolder.js` to use Tauri commands
    - Remove `useStdoutToPython` import
    - Replace folder open calls with Tauri invoke

    - _Requirements: 1.1, 1.2, 2.3_
  - [x] 6.2 Refactor `useSoftwareVersion.js` to use Tauri commands

    - Remove `useStdoutToPython` import
    - Use BackendInitController data or Tauri invoke
    - _Requirements: 1.1, 1.2, 2.1_

- [x] 7. Update message and volume hooks






  - [x] 7.1 Refactor `useMessage.js` to use Tauri commands

    - Remove `useStdoutToPython` import
    - Replace send message and typing calls with Tauri invoke
    - _Requirements: 1.1, 1.2, 2.3_


  - [x] 7.2 Refactor `useVolume.js` to use Tauri commands





    - Remove `useStdoutToPython` import
    - Replace threshold enable/disable calls with Tauri invoke
    - _Requirements: 1.1, 1.2, 2.4_

- [x] 8. Update window hook






  - [x] 8.1 Refactor `useWindow.js` to use Tauri commands

    - Remove `useStdoutToPython` import
    - Replace window geometry calls with Tauri invoke
    - _Requirements: 1.1, 1.2, 2.2_


- [x] 9. Update config page hooks





  - [x] 9.1 Refactor `useSettingsLogics.js` to use Tauri commands


    - Remove `useStdoutToPython` import
    - Replace generic settings calls with Tauri invoke
    - _Requirements: 1.1, 1.2, 2.1, 2.2, 3.1_
  - [x] 9.2 Refactor `useHotkeys.js` to use Tauri commands


    - Remove `useStdoutToPython` import
    - Use BackendInitController data for GET, Tauri invoke for SET
    - _Requirements: 1.1, 1.2, 2.1, 2.2_
  - [x] 9.3 Refactor `usePlugins.js` to use Tauri commands


    - Remove `useStdoutToPython` import
    - Replace plugin settings calls with Tauri invoke
    - _Requirements: 1.1, 1.2, 2.1, 2.2_

- [x] 10. Update JSX components




  - [x] 10.1 Refactor `AdvancedSettings.jsx` to use Tauri commands


    - Remove `useStdoutToPython` import
    - Replace ZLUDA info call with `invoke("get_zluda_info")`
    - _Requirements: 1.1, 1.2, 2.1_

- [x] 11. Checkpoint - Verify all usages removed





  - Ensure all tests pass, ask the user if questions arise.

- [x] 12. Remove legacy code





  - [x] 12.1 Delete `useStdoutToPython.js` file


    - Delete `src-ui/logics/common/useStdoutToPython.js`
    - _Requirements: 1.3_
  - [x] 12.2 Remove vite alias


    - Remove `@useStdoutToPython` alias from `vite.config.js`
    - _Requirements: 1.4_
  - [x] 12.3 Verify no remaining imports or calls


    - Run grep search for `useStdoutToPython` and `asyncStdoutToPython`
    - _Requirements: 1.1, 1.2_

- [x] 13. Clean up Python references






  - [x] 13.1 Remove Python-related comments


    - Search for and remove comments referencing Python backend
    - _Requirements: 5.1_
  - [x] 13.2 Rename Python/Stdout references in variable and function names


    - Update any remaining variable/function names that reference Python or Stdout
    - _Requirements: 5.2_
  - [x] 13.3 Update documentation


    - Update any documentation files to reference Rust backend instead of Python
    - _Requirements: 5.3_

- [x] 14. Final Checkpoint - Build and verify





  - Ensure all tests pass, ask the user if questions arise.

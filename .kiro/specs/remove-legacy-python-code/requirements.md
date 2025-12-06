# Requirements Document

## Introduction

This document specifies the requirements for removing the legacy Python communication code (`useStdoutToPython` and related hooks) from the VRCT frontend. After the successful migration from Python to Rust backend, these legacy communication hooks are now no-ops that do nothing. The goal is to clean up the codebase by removing dead code, updating components to use the new Rust backend via Tauri commands, and simplifying the frontend architecture.

## Glossary

- **VRCT Frontend**: The React-based user interface layer of the VRCT application
- **useStdoutToPython**: A legacy React hook that was used to communicate with the Python backend via stdout/stdin IPC
- **asyncStdoutToPython**: The async function exported by useStdoutToPython that previously sent commands to Python
- **Tauri Commands**: The new IPC mechanism for communicating with the Rust backend via `@tauri-apps/api/core`
- **Legacy Endpoint**: A URL-like path (e.g., `/get/data/version`) that was used to route commands to Python handlers
- **BackendInitController**: The new controller that initializes data from the Rust backend at startup
- **Rust Backend**: The Tauri-based backend implemented in Rust that replaced the Python backend

## Requirements

### Requirement 1

**User Story:** As a developer, I want to remove the useStdoutToPython hook and all its usages, so that the codebase no longer contains dead code that does nothing.

#### Acceptance Criteria

1. WHEN the cleanup is complete THEN the system SHALL have no imports of `useStdoutToPython` in any JavaScript or JSX file
2. WHEN the cleanup is complete THEN the system SHALL have no calls to `asyncStdoutToPython` in any JavaScript or JSX file
3. WHEN the cleanup is complete THEN the system SHALL delete the `src-ui/logics/common/useStdoutToPython.js` file
4. WHEN the cleanup is complete THEN the system SHALL remove the `@useStdoutToPython` alias from `vite.config.js`

### Requirement 2

**User Story:** As a developer, I want to update components that used legacy Python endpoints to use Tauri commands or existing state, so that the application continues to function correctly.

#### Acceptance Criteria

1. WHEN a component previously called `asyncStdoutToPython("/get/data/...")` THEN the component SHALL either use data from BackendInitController or call the appropriate Tauri command
2. WHEN a component previously called `asyncStdoutToPython("/set/data/...")` THEN the component SHALL call the appropriate Tauri command to update the Rust backend
3. WHEN a component previously called `asyncStdoutToPython("/run/...")` THEN the component SHALL call the appropriate Tauri command to execute the action
4. WHEN a component previously called `asyncStdoutToPython("/set/enable/...")` or `asyncStdoutToPython("/set/disable/...")` THEN the component SHALL call the appropriate Tauri command

### Requirement 3

**User Story:** As a developer, I want to simplify hooks that only existed to wrap asyncStdoutToPython calls, so that the code is more maintainable.

#### Acceptance Criteria

1. WHEN a hook's only purpose was to call asyncStdoutToPython THEN the hook SHALL be refactored to call Tauri commands directly or be removed if the functionality is handled elsewhere
2. WHEN a hook is removed THEN all components using that hook SHALL be updated to use the replacement mechanism
3. WHEN refactoring hooks THEN the system SHALL maintain the same user-facing functionality

### Requirement 4

**User Story:** As a user, I want the application to continue working after the cleanup, so that my workflow is not disrupted.

#### Acceptance Criteria

1. WHEN the cleanup is complete THEN the VRCT Frontend SHALL respond to user interactions with the same behavior as before the cleanup
2. WHEN the cleanup is complete THEN the VRCT Frontend SHALL persist settings changes to the Rust Backend configuration store
3. WHEN the cleanup is complete THEN the VRCT Frontend SHALL invoke transcription and translation Tauri Commands that return valid results
4. WHEN the cleanup is complete THEN the VRCT Frontend SHALL invoke OSC and WebSocket Tauri Commands that communicate with external services

### Requirement 5

**User Story:** As a developer, I want to remove any Python-related comments or documentation references, so that the codebase accurately reflects the current architecture.

#### Acceptance Criteria

1. WHEN the cleanup is complete THEN the VRCT Frontend SHALL contain no comments referencing Python-based communication mechanisms
2. WHEN the cleanup is complete THEN the VRCT Frontend SHALL have variable and function names that reference the Rust Backend instead of Python or Stdout
3. WHEN the cleanup is complete THEN the VRCT Frontend documentation SHALL reference the Rust Backend architecture


import { invoke } from "@tauri-apps/api/core";
import { useStore_IsMainPageCompactMode } from "@store";

export const useIsMainPageCompactMode = () => {
    const { currentIsMainPageCompactMode, updateIsMainPageCompactMode, pendingIsMainPageCompactMode } = useStore_IsMainPageCompactMode();

    // GET: Data is already loaded by BackendInitController at startup
    // This function is kept for API compatibility but now just returns current store data
    const getIsMainPageCompactMode = () => {
        // Data is already in store from BackendInitController initialization
        return currentIsMainPageCompactMode.data;
    };

    const toggleIsMainPageCompactMode = async () => {
        pendingIsMainPageCompactMode();
        const newValue = !currentIsMainPageCompactMode.data;
        
        try {
            await invoke("set_config", {
                config: { MAIN_WINDOW_SIDEBAR_COMPACT_MODE: newValue }
            });
            updateIsMainPageCompactMode(newValue);
        } catch (error) {
            console.error("Failed to toggle compact mode:", error);
        }
    };

    return {
        currentIsMainPageCompactMode,
        getIsMainPageCompactMode,
        toggleIsMainPageCompactMode,
        updateIsMainPageCompactMode,
    };
};
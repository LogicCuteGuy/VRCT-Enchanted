import { invoke } from "@tauri-apps/api/core";
import { store } from "@store";
import { useStore_MessageInputBoxRatio } from "@store";
import { clampMinMax } from "@utils";

export const useMessageInputBoxRatio = () => {
    const appWindow = store.appWindow;

    const { currentMessageInputBoxRatio, updateMessageInputBoxRatio, pendingMessageInputBoxRatio } = useStore_MessageInputBoxRatio();

    // GET: Data is already loaded by BackendInitController at startup
    // This function is kept for API compatibility but now just returns current store data
    const getMessageInputBoxRatio = () => {
        // Data is already in store from BackendInitController initialization
        return currentMessageInputBoxRatio.data;
    };

    const asyncSetMessageInputBoxRatio = async (ratio) => {
        const minimized = await appWindow.isMinimized();
        if (minimized === true) return; // don't save while the window is minimized.
        const parsed = parseFloat(ratio.toFixed(2));
        const valid_ratio = clampMinMax(parsed, 1, 99);
        
        try {
            await invoke("set_config", {
                config: { MESSAGE_BOX_RATIO: valid_ratio }
            });
            updateMessageInputBoxRatio(valid_ratio);
        } catch (error) {
            console.error("Failed to set message box ratio:", error);
        }
    };

    return {
        currentMessageInputBoxRatio,
        getMessageInputBoxRatio,
        updateMessageInputBoxRatio,
        asyncSetMessageInputBoxRatio,
    };
};
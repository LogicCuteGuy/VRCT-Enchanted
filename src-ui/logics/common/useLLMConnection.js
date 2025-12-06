import { invoke } from "@tauri-apps/api/core";
import {
    useStore_IsLMStudioConnected,
    useStore_IsOllamaConnected,
    getStoreHook,
} from "@store";

export const useLLMConnection = () => {
    const {
        currentIsLMStudioConnected,
        updateIsLMStudioConnected,
        pendingIsLMStudioConnected,
    } = useStore_IsLMStudioConnected();
    const {
        currentIsOllamaConnected,
        updateIsOllamaConnected,
        pendingIsOllamaConnected,
    } = useStore_IsOllamaConnected();

    // Get LMStudioURL from the store
    const useStore_LMStudioURL = getStoreHook("LMStudioURL");
    const lmStudioURLStore = useStore_LMStudioURL ? useStore_LMStudioURL() : null;

    const checkConnection_LMStudio = async () => {
        pendingIsLMStudioConnected();
        try {
            // Get the LM Studio URL from the store, fallback to default
            const baseUrl = lmStudioURLStore?.currentLMStudioURL?.data || "http://127.0.0.1:1234/v1";
            const response = await invoke("check_lmstudio_connection", { base_url: baseUrl });
            const result = response?.result || response;
            updateIsLMStudioConnected(result?.connected || false);
        } catch (error) {
            console.error("Failed to check LM Studio connection:", error);
            updateIsLMStudioConnected(false);
        }
    };
    const setConnectionStatus_LMStudio = (is_connected) => {
        updateIsLMStudioConnected(is_connected);
    };

    const checkConnection_Ollama = async () => {
        pendingIsOllamaConnected();
        try {
            const response = await invoke("check_ollama_connection");
            const result = response?.result || response;
            updateIsOllamaConnected(result?.connected || false);
        } catch (error) {
            console.error("Failed to check Ollama connection:", error);
            updateIsOllamaConnected(false);
        }
    };
    const setConnectionStatus_Ollama = (is_connected) => {
        updateIsOllamaConnected(is_connected);
    };

    return {
        currentIsLMStudioConnected,
        updateIsLMStudioConnected,
        setConnectionStatus_LMStudio,
        checkConnection_LMStudio,

        currentIsOllamaConnected,
        updateIsOllamaConnected,
        setConnectionStatus_Ollama,
        checkConnection_Ollama,
    };
};
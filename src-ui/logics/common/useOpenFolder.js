import { invoke } from "@tauri-apps/api/core";

export const useOpenFolder = () => {
    const openFolder_MessageLogs = async () => {
        try {
            await invoke("open_logs_folder");
            console.log("Opened Directory, Message Logs");
        } catch (error) {
            console.error("Failed to open logs folder:", error);
        }
    };

    const openFolder_ConfigFile = async () => {
        try {
            await invoke("open_config_folder");
            console.log("Opened Directory, Config File");
        } catch (error) {
            console.error("Failed to open config folder:", error);
        }
    };

    return {
        openFolder_MessageLogs,
        openFolder_ConfigFile,
    };
};
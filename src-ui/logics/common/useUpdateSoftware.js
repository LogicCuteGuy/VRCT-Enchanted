import { invoke } from "@tauri-apps/api/core";

export const useUpdateSoftware = () => {
    const updateSoftware = async () => {
        try {
            // First check for updates to get the download URL
            const updateInfo = await invoke("check_for_updates");
            const result = updateInfo?.result || updateInfo;
            
            if (result?.download_url) {
                // Download and launch the updater
                await invoke("download_update", { download_url: result.download_url });
            } else {
                console.error("No download URL available for update");
            }
        } catch (error) {
            console.error("Failed to update software:", error);
        }
    };

    const updateSoftware_CUDA = async () => {
        // CUDA update uses the same updater - the updater itself handles CPU/CUDA selection
        try {
            // First check for updates to get the download URL
            const updateInfo = await invoke("check_for_updates");
            const result = updateInfo?.result || updateInfo;
            
            if (result?.download_url) {
                // Download and launch the updater
                await invoke("download_update", { download_url: result.download_url });
            } else {
                console.error("No download URL available for CUDA update");
            }
        } catch (error) {
            console.error("Failed to update CUDA software:", error);
        }
    };

    return {
        updateSoftware,
        updateSoftware_CUDA,
    };
};

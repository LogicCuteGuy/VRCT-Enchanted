import { invoke } from "@tauri-apps/api/core";
import {
    useStore_MicVolume,
    useStore_SpeakerVolume,
    useStore_MicThresholdCheckStatus,
    useStore_SpeakerThresholdCheckStatus,
} from "@store";

export const useVolume = () => {
    const { updateMicVolume } = useStore_MicVolume();
    const { updateSpeakerVolume } = useStore_SpeakerVolume();
    const {
        currentMicThresholdCheckStatus,
        updateMicThresholdCheckStatus,
        pendingMicThresholdCheckStatus,
    } = useStore_MicThresholdCheckStatus();
    const {
        currentSpeakerThresholdCheckStatus,
        updateSpeakerThresholdCheckStatus,
        pendingSpeakerThresholdCheckStatus,
    } = useStore_SpeakerThresholdCheckStatus();

    return {
        volumeCheckStart_Mic: async () => {
            pendingMicThresholdCheckStatus();
            try {
                await invoke("enable_mic_threshold_check");
            } catch (error) {
                console.error("Failed to enable mic threshold check:", error);
            }
        },
        volumeCheckStop_Mic: async () => {
            pendingMicThresholdCheckStatus();
            try {
                await invoke("disable_mic_threshold_check");
            } catch (error) {
                console.error("Failed to disable mic threshold check:", error);
            }
        },
        updateVolumeVariable_Mic: (payload) => {
            updateMicVolume(payload);
        },
        currentMicThresholdCheckStatus: currentMicThresholdCheckStatus,
        updateMicThresholdCheckStatus: (payload) => {
            updateMicThresholdCheckStatus(payload);
            if (payload === false) updateMicVolume("0");
        },

        volumeCheckStart_Speaker: async () => {
            updateSpeakerVolume("0");
            pendingSpeakerThresholdCheckStatus();
            try {
                await invoke("enable_speaker_threshold_check");
            } catch (error) {
                console.error("Failed to enable speaker threshold check:", error);
            }
        },
        volumeCheckStop_Speaker: async () => {
            pendingSpeakerThresholdCheckStatus();
            try {
                await invoke("disable_speaker_threshold_check");
            } catch (error) {
                console.error("Failed to disable speaker threshold check:", error);
            }
        },
        updateVolumeVariable_Speaker: (payload) => {
            updateSpeakerVolume(payload);
        },
        currentSpeakerThresholdCheckStatus: currentSpeakerThresholdCheckStatus,
        updateSpeakerThresholdCheckStatus: (payload) => {
            updateSpeakerThresholdCheckStatus(payload);
            if (payload === false) updateSpeakerVolume("0");
        }

    };
};
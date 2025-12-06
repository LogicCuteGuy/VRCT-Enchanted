import { invoke } from "@tauri-apps/api/core";
import { store } from "@store";

import {
    useStore_TranslationStatus,
    useStore_TranscriptionSendStatus,
    useStore_TranscriptionReceiveStatus,
    useStore_ForegroundStatus,
} from "@store";

export const useMainFunction = () => {
    const appWindow = store.appWindow;

    const {
        currentTranslationStatus,
        updateTranslationStatus,
        pendingTranslationStatus,
    } = useStore_TranslationStatus();
    const {
        currentTranscriptionSendStatus,
        updateTranscriptionSendStatus,
        pendingTranscriptionSendStatus,
    } = useStore_TranscriptionSendStatus();
    const {
        currentTranscriptionReceiveStatus,
        updateTranscriptionReceiveStatus,
        pendingTranscriptionReceiveStatus,
    } = useStore_TranscriptionReceiveStatus();
    const {
        currentForegroundStatus,
        updateForegroundStatus,
    } = useStore_ForegroundStatus();

    const setTranslation = (to_enable) => {
        pendingTranslationStatus();
        if (to_enable) {
            invoke("enable_translation");
        } else {
            invoke("disable_translation");
        }
    };
    const toggleTranslation = () => {
        updateTranslationStatus(prev_state => {
            if (prev_state.state === "ok") setTranslation(!prev_state.data);
        }, { set_state: "pending" });
    };

    const setTranscriptionSend = (to_enable) => {
        pendingTranscriptionSendStatus();
        if (to_enable) {
            invoke("enable_transcription_send");
        } else {
            invoke("disable_transcription_send");
        }
    };
    const toggleTranscriptionSend = () => {
        updateTranscriptionSendStatus(prev_state => {
            if (prev_state.state === "ok") setTranscriptionSend(!prev_state.data);
        }, { set_state: "pending" });
    };

    const setTranscriptionReceive = (to_enable) => {
        pendingTranscriptionReceiveStatus();
        if (to_enable) {
            invoke("enable_transcription_receive");
        } else {
            invoke("disable_transcription_receive");
        }
    };
    const toggleTranscriptionReceive = () => {
        updateTranscriptionReceiveStatus(prev_state => {
            if (prev_state.state === "ok") setTranscriptionReceive(!prev_state.data);
        }, { set_state: "pending" });
    };


    const toggleForeground = async () => {
        const is_foreground_enabled = !currentForegroundStatus.data;
        await appWindow.setAlwaysOnTop(is_foreground_enabled);
        updateForegroundStatus(is_foreground_enabled);
    };

    return {
        currentTranslationStatus,
        toggleTranslation,
        updateTranslationStatus,
        setTranslation,
        pendingTranslationStatus, // Exception.(It shouldn't be used in other function, normally.)

        currentTranscriptionSendStatus,
        toggleTranscriptionSend,
        updateTranscriptionSendStatus,
        setTranscriptionSend,
        pendingTranscriptionSendStatus, // Exception.(It shouldn't be used in other function, normally.)

        currentTranscriptionReceiveStatus,
        toggleTranscriptionReceive,
        updateTranscriptionReceiveStatus,
        setTranscriptionReceive,
        pendingTranscriptionReceiveStatus, // Exception.(It shouldn't be used in other function, normally.)

        currentForegroundStatus,
        toggleForeground,
        updateForegroundStatus,

    };
};
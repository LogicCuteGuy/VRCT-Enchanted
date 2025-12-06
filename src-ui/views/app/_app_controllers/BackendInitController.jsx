import { invoke } from "@tauri-apps/api/core";
import { useEffect, useRef } from "react";

import { 
    getStoreHook,
    useStore_SelectableFontFamilyList,
    useStore_SelectedYourLanguages,
    useStore_SelectedTargetLanguages,
    useStore_SelectedTranslationEngines,
    useStore_SelectedPresetTabNumber,
    useStore_TranslationEngines,
} from "@store";
import { arrayToObject } from "@utils";

import {
    useNotificationStatus,
    useIsBackendReady,
} from "@logics_common";

export const BackendInitController = () => {
    const { asyncInitializeBackend } = useInitializeBackend();
    const hasRunRef = useRef(false);
    const { asyncFetchFonts } = useAsyncFetchFonts();
    const { asyncFetchAudioDevices } = useAsyncFetchAudioDevices();

    useEffect(() => {
        if (!hasRunRef.current) {
            asyncInitializeBackend().then(() => {
                asyncFetchFonts();
                asyncFetchAudioDevices();
            }).catch((err) => {
                console.error("Backend initialization error:", err);
            });
        }
        return () => hasRunRef.current = true;
    }, []);

    return null;
};

const useInitializeBackend = () => {
    const { showNotification_Error } = useNotificationStatus();
    const { updateIsBackendReady } = useIsBackendReady();
    const { updateSelectedYourLanguages } = useStore_SelectedYourLanguages();
    const { updateSelectedTargetLanguages } = useStore_SelectedTargetLanguages();
    const { updateSelectedTranslationEngines } = useStore_SelectedTranslationEngines();
    const { updateSelectedPresetTabNumber } = useStore_SelectedPresetTabNumber();
    const { updateTranslationEngines } = useStore_TranslationEngines();

    const asyncInitializeBackend = async () => {
        try {
            // Initialize the Rust controller
            const isInitialized = await invoke("is_controller_initialized");
            
            if (!isInitialized) {
                await invoke("initialize_controller");
            }
            
            // Load initial config to populate the store
            const config = await invoke("get_config");
            console.log("Config loaded:", config);
            
            // Extract the result from the Response structure
            const configData = config?.result || config;
            
            // Initialize store with config data from Rust backend
            // This ensures the UI has the necessary data structure
            
            // Default language structure for all 3 presets
            const defaultYourLanguages = {
                "1": { "1": { language: "English", country: "United States", enable: true } },
                "2": { "1": { language: "English", country: "United States", enable: true } },
                "3": { "1": { language: "English", country: "United States", enable: true } },
            };
            
            const defaultTargetLanguages = {
                "1": {
                    "1": { language: "Japanese", country: "Japan", enable: true },
                    "2": { language: "Korean", country: "Korea", enable: false },
                    "3": { language: "Chinese (Simplified)", country: "China", enable: false },
                },
                "2": {
                    "1": { language: "Japanese", country: "Japan", enable: true },
                    "2": { language: "Korean", country: "Korea", enable: false },
                    "3": { language: "Chinese (Simplified)", country: "China", enable: false },
                },
                "3": {
                    "1": { language: "Japanese", country: "Japan", enable: true },
                    "2": { language: "Korean", country: "Korea", enable: false },
                    "3": { language: "Chinese (Simplified)", country: "China", enable: false },
                },
            };
            
            if (configData) {
                // Use config data if available, otherwise use defaults
                updateSelectedYourLanguages(configData.SELECTED_YOUR_LANGUAGES || defaultYourLanguages);
                updateSelectedTargetLanguages(configData.SELECTED_TARGET_LANGUAGES || defaultTargetLanguages);
                
                if (configData.SELECTED_TRANSLATION_ENGINES) {
                    updateSelectedTranslationEngines(configData.SELECTED_TRANSLATION_ENGINES);
                } else {
                    updateSelectedTranslationEngines({ "1": "CTranslate2", "2": "CTranslate2", "3": "CTranslate2" });
                }
                
                updateSelectedPresetTabNumber(configData.SELECTED_TAB_NO || "1");
            } else {
                // No config data, use all defaults
                updateSelectedYourLanguages(defaultYourLanguages);
                updateSelectedTargetLanguages(defaultTargetLanguages);
                updateSelectedTranslationEngines({ "1": "CTranslate2", "2": "CTranslate2", "3": "CTranslate2" });
                updateSelectedPresetTabNumber("1");
            }
            
            // Initialize translation engines as available
            // In the Rust backend, these are handled natively
            const availableEngines = [
                { id: "CTranslate2", label: "AI\nCTranslate2", is_available: true, is_default: true },
                { id: "DeepL_API", label: "DeepL API", is_available: true },
                { id: "Gemini_API", label: "Gemini API", is_available: true },
                { id: "OpenAI_API", label: "OpenAI API", is_available: true },
                { id: "LMStudio", label: "LMStudio", is_available: true },
                { id: "Ollama", label: "Ollama", is_available: true },
                { id: "Plamo_API", label: "Plamo API", is_available: true },
            ];
            updateTranslationEngines(availableEngines);
            
            // Mark backend as ready
            updateIsBackendReady(true);
            console.log("Rust backend initialized successfully");
        } catch (error) {
            console.error("Failed to initialize backend:", error);
            showNotification_Error(
                `Failed to initialize backend: ${error}`, 
                { hide_duration: null }
            );
            throw error;
        }
    };

    return { asyncInitializeBackend };
};

const useAsyncFetchFonts = () => {
    const { updateSelectableFontFamilyList } = useStore_SelectableFontFamilyList();
    const asyncFetchFonts = async () => {
        try {
            let fonts = await invoke("get_font_list");
            fonts = fonts.sort((a, b) => a.localeCompare(b, undefined, { sensitivity: "base" }));
            updateSelectableFontFamilyList(arrayToObject(fonts));
        } catch (error) {
            console.error("Error fetching fonts:", error);
        }
    };
    return { asyncFetchFonts };
};

const useAsyncFetchAudioDevices = () => {
    // Get hooks for audio device lists from dynamic store registry
    const useMicHostList = getStoreHook("MicHostList");
    const useMicDeviceList = getStoreHook("MicDeviceList");
    const useSpeakerDeviceList = getStoreHook("SpeakerDeviceList");
    const useSelectedMicHost = getStoreHook("SelectedMicHost");
    const useSelectedMicDevice = getStoreHook("SelectedMicDevice");
    const useSelectedSpeakerDevice = getStoreHook("SelectedSpeakerDevice");
    
    const micHostList = useMicHostList?.();
    const micDeviceList = useMicDeviceList?.();
    const speakerDeviceList = useSpeakerDeviceList?.();
    const selectedMicHost = useSelectedMicHost?.();
    const selectedMicDevice = useSelectedMicDevice?.();
    const selectedSpeakerDevice = useSelectedSpeakerDevice?.();

    const asyncFetchAudioDevices = async () => {
        try {
            const response = await invoke("get_audio_devices");
            console.log("Audio devices loaded:", response);
            
            if (response?.data) {
                const { input_devices, output_devices } = response.data;
                
                // Process input devices (microphones)
                if (input_devices && Array.isArray(input_devices)) {
                    // Extract unique hosts
                    const hosts = [...new Set(input_devices.map(d => d.host || "Default"))];
                    micHostList?.updateMicHostList?.(arrayToObject(hosts));
                    
                    // Set device list
                    const deviceNames = input_devices.map(d => d.name || d);
                    micDeviceList?.updateMicDeviceList?.(arrayToObject(deviceNames));
                    
                    // Set default selection if available
                    if (hosts.length > 0 && !selectedMicHost?.currentSelectedMicHost?.data) {
                        selectedMicHost?.updateSelectedMicHost?.(hosts[0]);
                    }
                    if (deviceNames.length > 0 && !selectedMicDevice?.currentSelectedMicDevice?.data) {
                        selectedMicDevice?.updateSelectedMicDevice?.(deviceNames[0]);
                    }
                }
                
                // Process output devices (speakers)
                if (output_devices && Array.isArray(output_devices)) {
                    const deviceNames = output_devices.map(d => d.name || d);
                    speakerDeviceList?.updateSpeakerDeviceList?.(arrayToObject(deviceNames));
                    
                    // Set default selection if available
                    if (deviceNames.length > 0 && !selectedSpeakerDevice?.currentSelectedSpeakerDevice?.data) {
                        selectedSpeakerDevice?.updateSelectedSpeakerDevice?.(deviceNames[0]);
                    }
                }
            }
        } catch (error) {
            console.error("Error fetching audio devices:", error);
        }
    };
    
    return { asyncFetchAudioDevices };
};
import { invoke } from "@tauri-apps/api/core";
import { useStore_SelectedPresetTabNumber, useStore_SelectedYourLanguages, useStore_SelectedTargetLanguages, useStore_TranslationEngines, useStore_SelectedTranslationEngines, useStore_SelectableLanguageList } from "@store";
import { translator_status } from "@ui_configs";

export const useLanguageSettings = () => {
    const {
        currentSelectedYourLanguages,
        updateSelectedYourLanguages,
        pendingSelectedYourLanguages,
    } = useStore_SelectedYourLanguages();
    const {
        currentSelectedTargetLanguages,
        updateSelectedTargetLanguages,
        pendingSelectedTargetLanguages,
    } = useStore_SelectedTargetLanguages();
    const {
        currentSelectedPresetTabNumber,
        updateSelectedPresetTabNumber,
        pendingSelectedPresetTabNumber,
    } = useStore_SelectedPresetTabNumber();
    const {
        currentTranslationEngines,
        updateTranslationEngines,
        pendingTranslationEngines,
    } = useStore_TranslationEngines();
    const {
        currentSelectedTranslationEngines,
        updateSelectedTranslationEngines,
        pendingSelectedTranslationEngines,
    } = useStore_SelectedTranslationEngines();

    const {
        currentSelectableLanguageList,
        updateSelectableLanguageList,
    } = useStore_SelectableLanguageList();


    // GET: Data is already loaded by BackendInitController at startup
    // This function is kept for API compatibility but now just returns current store data
    const getSelectedPresetTabNumber = () => {
        // Data is already in store from BackendInitController initialization
        // No need to fetch - just return current value
        return currentSelectedPresetTabNumber.data;
    };

    const setSelectedPresetTabNumber = async (preset_number) => {
        pendingSelectedPresetTabNumber();
        
        try {
            await invoke("set_config", {
                config: { SELECTED_TAB_NO: preset_number }
            });
            updateSelectedPresetTabNumber(preset_number);
        } catch (error) {
            console.error("Failed to set preset tab number:", error);
        }
    };


    // GET: Data is already loaded by BackendInitController at startup
    const getSelectedYourLanguages = () => {
        // Data is already in store from BackendInitController initialization
        return currentSelectedYourLanguages.data;
    };

    const setSelectedYourLanguages = async (selected_language_data) => {
        pendingSelectedYourLanguages();
        const send_obj = {
            ...currentSelectedYourLanguages.data,
            [currentSelectedPresetTabNumber.data]: {
                1: { // Fixed key 1.
                    language: selected_language_data.language,
                    country: selected_language_data.country,
                    enable: true,
                }
            }
        };
        
        try {
            await invoke("set_config", {
                config: { SELECTED_YOUR_LANGUAGES: send_obj }
            });
            updateSelectedYourLanguages(send_obj);
        } catch (error) {
            console.error("Failed to set your languages:", error);
        }
    };


    // GET: Data is already loaded by BackendInitController at startup
    const getSelectedTargetLanguages = () => {
        // Data is already in store from BackendInitController initialization
        return currentSelectedTargetLanguages.data;
    };

    const setSelectedTargetLanguages = async (selected_language_data) => {
        pendingSelectedTargetLanguages();
        let send_obj = { ...currentSelectedTargetLanguages.data };
        send_obj[currentSelectedPresetTabNumber.data] = {
            ...send_obj[currentSelectedPresetTabNumber.data],
            [selected_language_data.target_key]: {
                ...send_obj[currentSelectedPresetTabNumber.data][selected_language_data.target_key],
                language: selected_language_data.language,
                country: selected_language_data.country,
            }
        };
        
        try {
            await invoke("set_config", {
                config: { SELECTED_TARGET_LANGUAGES: send_obj }
            });
            updateSelectedTargetLanguages(send_obj);
        } catch (error) {
            console.error("Failed to set target languages:", error);
        }
    };

    const addTargetLanguage = async () => {
        pendingSelectedTargetLanguages();
        let send_obj = { ...currentSelectedTargetLanguages.data };
        let target_key = "2";
        if (send_obj[currentSelectedPresetTabNumber.data]["2"].enable === true) {
            target_key = "3";
        }
        send_obj[currentSelectedPresetTabNumber.data] = {
            ...send_obj[currentSelectedPresetTabNumber.data],
            [target_key]: {
                ...send_obj[currentSelectedPresetTabNumber.data][target_key],
                enable: true,
            }
        };
        
        try {
            await invoke("set_config", {
                config: { SELECTED_TARGET_LANGUAGES: send_obj }
            });
            updateSelectedTargetLanguages(send_obj);
        } catch (error) {
            console.error("Failed to add target language:", error);
        }
    };

    const removeTargetLanguage = async () => {
        pendingSelectedTargetLanguages();
        let send_obj = { ...currentSelectedTargetLanguages.data };
        let target_key = "3";
        if (send_obj[currentSelectedPresetTabNumber.data]["3"].enable === false) {
            target_key = "2";
        }
        send_obj[currentSelectedPresetTabNumber.data] = {
            ...send_obj[currentSelectedPresetTabNumber.data],
            [target_key]: {
                ...send_obj[currentSelectedPresetTabNumber.data][target_key],
                enable: false,
            }
        };
        
        try {
            await invoke("set_config", {
                config: { SELECTED_TARGET_LANGUAGES: send_obj }
            });
            updateSelectedTargetLanguages(send_obj);
        } catch (error) {
            console.error("Failed to remove target language:", error);
        }
    };


    // GET: Data is already loaded by BackendInitController at startup
    const getTranslationEngines = () => {
        // Data is already in store from BackendInitController initialization
        return currentTranslationEngines.data;
    };

    const updateTranslatorAvailability = (payload) => {
        const keys = payload;
        const updated_list = translator_status.map(translator => ({
            ...translator,
            is_available: keys.includes(translator.id),
        }));
        updateTranslationEngines(updated_list);
    };


    // GET: Data is already loaded by BackendInitController at startup
    const getSelectedTranslationEngines = () => {
        // Data is already in store from BackendInitController initialization
        return currentSelectedTranslationEngines.data;
    };

    const setSelectedTranslationEngines = async (selected_translator) => {
        pendingSelectedTranslationEngines();
        let send_obj = { ...currentSelectedTranslationEngines.data };
        send_obj[currentSelectedPresetTabNumber.data] = selected_translator;
        
        try {
            // Use set_translation_engine for the active engine
            await invoke("set_translation_engine", {
                engine: selected_translator
            });
            // Also persist to config
            await invoke("set_config", {
                config: { SELECTED_TRANSLATION_ENGINES: send_obj }
            });
            updateSelectedTranslationEngines(send_obj);
        } catch (error) {
            console.error("Failed to set translation engine:", error);
        }
    };

    const swapSelectedLanguages = async () => {
        pendingSelectedYourLanguages();
        pendingSelectedTargetLanguages();
        
        try {
            const response = await invoke("swap_languages");
            // The swap_languages command should return the swapped languages
            // Update the store with the new values
            if (response?.result) {
                const { your_languages, target_languages } = response.result;
                if (your_languages) updateSelectedYourLanguages(your_languages);
                if (target_languages) updateSelectedTargetLanguages(target_languages);
            } else {
                // Fallback: swap locally if backend doesn't return new values
                const currentYour = currentSelectedYourLanguages.data;
                const currentTarget = currentSelectedTargetLanguages.data;
                const presetTab = currentSelectedPresetTabNumber.data;
                
                // Get the first target language to swap with your language
                const yourLang = currentYour[presetTab]?.["1"];
                const targetLang = currentTarget[presetTab]?.["1"];
                
                if (yourLang && targetLang) {
                    const newYour = {
                        ...currentYour,
                        [presetTab]: {
                            "1": {
                                language: targetLang.language,
                                country: targetLang.country,
                                enable: true,
                            }
                        }
                    };
                    const newTarget = {
                        ...currentTarget,
                        [presetTab]: {
                            ...currentTarget[presetTab],
                            "1": {
                                ...currentTarget[presetTab]["1"],
                                language: yourLang.language,
                                country: yourLang.country,
                            }
                        }
                    };
                    updateSelectedYourLanguages(newYour);
                    updateSelectedTargetLanguages(newTarget);
                }
            }
        } catch (error) {
            console.error("Failed to swap languages:", error);
        }
    };

    const updateBothSelectedLanguages = (payload) => {
        updateSelectedYourLanguages(payload.your);
        updateSelectedTargetLanguages(payload.target);
    };


    // GET: Data is already loaded by BackendInitController at startup
    const getSelectableLanguageList = () => {
        // Data is already in store from BackendInitController initialization
        return currentSelectableLanguageList.data;
    };


    return {
        currentSelectedPresetTabNumber,
        getSelectedPresetTabNumber,
        updateSelectedPresetTabNumber,
        setSelectedPresetTabNumber,

        currentSelectedYourLanguages,
        getSelectedYourLanguages,
        updateSelectedYourLanguages,
        setSelectedYourLanguages,

        currentSelectedTargetLanguages,
        getSelectedTargetLanguages,
        updateSelectedTargetLanguages,
        setSelectedTargetLanguages,

        addTargetLanguage,
        removeTargetLanguage,

        currentTranslationEngines,
        getTranslationEngines,
        updateTranslationEngines,
        updateTranslatorAvailability,

        currentSelectedTranslationEngines,
        getSelectedTranslationEngines,
        updateSelectedTranslationEngines,
        setSelectedTranslationEngines,

        swapSelectedLanguages,
        updateBothSelectedLanguages,

        currentSelectableLanguageList,
        getSelectableLanguageList,
        updateSelectableLanguageList,
    };
};

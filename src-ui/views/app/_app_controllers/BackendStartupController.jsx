import { invoke } from "@tauri-apps/api/core";
import { useEffect, useRef } from "react";

import { store, useStore_SelectableFontFamilyList } from "@store";
import { arrayToObject } from "@utils";

export const BackendStartupController = () => {
    const { asyncInitBackend } = useInitBackend();
    const hasRunRef = useRef(false);
    const { asyncFetchFonts } = useAsyncFetchFonts();

    useEffect(() => {
        if (!hasRunRef.current) {
            asyncInitBackend().then(() => {
                asyncFetchFonts();
            }).catch((err) => {
                console.error(err);
            });
        }
        return () => hasRunRef.current = true;
    }, []);

    return null;
};

const useInitBackend = () => {
    const asyncInitBackend = async () => {
        // The Rust backend is now built directly into the Tauri application
        // Initialize the controller before using any other commands
        try {
            // Check if controller is already initialized
            const isInitialized = await invoke("is_controller_initialized");
            
            if (!isInitialized) {
                console.log("Initializing Rust backend controller...");
                await invoke("initialize_controller");
                console.log("Rust backend controller initialized successfully");
            } else {
                console.log("Rust backend controller already initialized");
            }
        } catch (error) {
            console.error("Failed to initialize Rust backend:", error);
            throw error;
        }
    };

    return { asyncInitBackend };
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

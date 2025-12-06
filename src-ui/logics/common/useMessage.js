import { invoke } from "@tauri-apps/api/core";
import {
    useStore_MessageLogs,
    useStore_MessageInputValue,
    store,
} from "@store";

export const useMessage = () => {
    const { currentMessageLogs, addMessageLogs, updateMessageLogs } = useStore_MessageLogs();
    const { currentMessageInputValue, updateMessageInputValue } = useStore_MessageInputValue();

    const sendMessage = async (message) => {
        const uuid = crypto.randomUUID();

        // Add message to logs immediately with pending status
        addMessageLogs({
            id: uuid,
            category: "sent",
            status: "pending",
            created_at: generateTimeData(),
            messages: {
                original: { message: message, transliteration: [] },
                translations: [],
            },
        });

        // Send message via Tauri command
        try {
            await invoke("send_message_box", { message });
        } catch (error) {
            console.error("Failed to send message:", error);
        }
    };

    const addSystemMessageLog = (message) => {
        const uuid = crypto.randomUUID();
        const date = generateTimeData();

        addMessageLogs({
            id: uuid,
            category: "system",
            status: "system",
            created_at: date,
            messages: {
                original: { message: message, transliteration: [] },
                translations: [],
            },
        });
    };

    const addSystemMessageLog_FromBackend = (payload) => {
        addSystemMessageLog(payload.message);
    };

    const updateSentMessageLogById = (payload) => {
        updateMessageLogs(updateItemById(payload.id, payload));
    };

    const addSentMessageLog = (payload) => {
        const message_object = generateMessageObject(payload, "sent");
        addMessageLogs(message_object);
    };

    const addReceivedMessageLog = (payload) => {
        const message_object = generateMessageObject(payload, "received");
        addMessageLogs(message_object);
    };

    const startTyping = async () => {
        const now = Date.now();
        if (now - store.last_executed_time_startTyping >= 2000) {
            store.last_executed_time_startTyping = now;
            try {
                await invoke("start_typing");
            } catch (error) {
                console.error("Failed to start typing indicator:", error);
            }
        }
    };

    const stopTyping = async () => {
        try {
            await invoke("stop_typing");
        } catch (error) {
            console.error("Failed to stop typing indicator:", error);
        }
    };

    return {
        currentMessageLogs,
        sendMessage,
        addSystemMessageLog,
        addSystemMessageLog_FromBackend,
        updateSentMessageLogById,
        addSentMessageLog,
        addReceivedMessageLog,

        currentMessageInputValue,
        updateMessageInputValue,

        startTyping,
        stopTyping,
    };
};

const generateTimeData = () => {
    return new Date().toLocaleTimeString(
        "ja-JP",
        { hour12: false, hour: "2-digit", minute: "2-digit" }
    );
};

const generateMessageObject = (data, category) => {
    return {
        id: crypto.randomUUID(),
        created_at: generateTimeData(),
        category: category,
        status: "ok",
        messages: {
            original: data.original,
            translations: data.translations ?? [],
        },
    };
};

const updateItemById = (id, updated_data) => (current_items) => {
    return current_items.data.map(item => {
        if (item.id === id) {
            item.status = "ok";
            if (updated_data.translations) item.messages.translations = updated_data.translations;
        }
        return item;
    });
};
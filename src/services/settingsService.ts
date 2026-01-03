import {invoke} from "@tauri-apps/api/core";
import {open, save} from "@tauri-apps/plugin-dialog";
import {KeysBatch, ImportSummary} from "../types";
import {ERROR_MESSAGES, parseBackupError} from "@/constants/errorMessages.ts";

export const settingsService = {
    getSecrets: async (): Promise<KeysBatch> => {
        return await invoke<KeysBatch>("get_secrets");
    },

    setSecrets: async (keys: {
        steamId: string | null;
        steamApiKey: string | null;
        rawgApiKey: string | null;
    }): Promise<void> => {
        await invoke("set_secrets", keys);
    },

    importSteamLibrary: async (
        steamId: string,
        apiKey: string
    ): Promise<string> => {
        return await invoke<string>("import_steam_library", {steamId, apiKey});
    },

    enrichLibrary: async (): Promise<ImportSummary> => {
        return await invoke<ImportSummary>("enrich_library");
    },

    exportDatabase: async (): Promise<string> => {
        try {
            const filePath = await save({
                defaultPath: `playlite-backup-${new Date().toISOString().split('T')[0]}.json`,
                filters: [{
                    name: 'JSON',
                    extensions: ['json']
                }]
            });

            if (!filePath) {
                throw new Error(ERROR_MESSAGES.CANCELLED);
            }

            await invoke("export_database", {filePath});
            return "Backup exportado com sucesso!";
        } catch (error: any) {
            if (error.message === ERROR_MESSAGES.CANCELLED) {
                throw new Error(ERROR_MESSAGES.CANCELLED);
            }

            const friendlyError = parseBackupError(error);

            if (friendlyError === String(error)) {
                throw new Error(ERROR_MESSAGES.BACKUP_EXPORT_FAILED);
            }

            throw new Error(friendlyError);
        }
    },

    importDatabase: async (): Promise<string> => {
        try {
            const selected = await open({
                multiple: false,
                filters: [{
                    name: 'JSON',
                    extensions: ['json']
                }]
            });

            if (!selected) {
                throw new Error(ERROR_MESSAGES.CANCELLED);
            }

            const filePath = selected as string;
            return await invoke<string>("import_database", {filePath});
        } catch (error: any) {
            if (error.message === ERROR_MESSAGES.CANCELLED) {
                throw new Error(ERROR_MESSAGES.CANCELLED);
            }

            const friendlyError = parseBackupError(error);

            if (friendlyError === String(error)) {
                throw new Error(ERROR_MESSAGES.BACKUP_IMPORT_FAILED);
            }

            throw new Error(friendlyError);
        }
    },
};
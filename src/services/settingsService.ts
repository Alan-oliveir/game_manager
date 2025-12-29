import { invoke } from "@tauri-apps/api/core";
import { KeysBatch } from "../types";

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
    return await invoke<string>("import_steam_library", { steamId, apiKey });
  },

  enrichLibrary: async (): Promise<string> => {
    return await invoke<string>("enrich_library");
  },
};

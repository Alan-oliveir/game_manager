import {useEffect, useState} from "react";
import {settingsService} from "../services/settingsService";
import {ERROR_MESSAGES} from "../constants/errorMessages";

export function useSettings(onLibraryUpdate: () => void) {
    const [keys, setKeys] = useState({
        steamId: "",
        steamApiKey: "",
        rawgApiKey: "",
    });
    const [loading, setLoading] = useState({
        initial: true,
        saving: false,
        importing: false,
        enriching: false,
        exporting: false,
        importingBackup: false,
    });
    const [status, setStatus] = useState<{
        type: "success" | "error" | null;
        message: string;
    }>({type: null, message: ""});

    useEffect(() => {
        settingsService
            .getSecrets()
            .then((data) => {
                setKeys({
                    steamId: data.steam_id || "",
                    steamApiKey: data.steam_api_key || "",
                    rawgApiKey: data.rawg_api_key || "",
                });
            })
            .catch((e) => console.error("Erro ao carregar settings", e))
            .finally(() => setLoading((prev) => ({...prev, initial: false})));
    }, []);

    const saveKeys = async () => {
        setLoading((prev) => ({...prev, saving: true}));
        setStatus({type: null, message: ""});
        try {
            await settingsService.setSecrets({
                steamId: keys.steamId.trim() || null,
                steamApiKey: keys.steamApiKey.trim() || null,
                rawgApiKey: keys.rawgApiKey.trim() || null,
            });
            setStatus({
                type: "success",
                message: "Configurações salvas com segurança!",
            });
        } catch (error) {
            setStatus({type: "error", message: `Erro ao salvar: ${error}`});
        } finally {
            setLoading((prev) => ({...prev, saving: false}));
        }
    };

    const importLibrary = async () => {
        if (!keys.steamId || !keys.steamApiKey) {
            setStatus({
                type: "error",
                message: "Preencha e salve as chaves da Steam primeiro.",
            });
            return;
        }
        setLoading((prev) => ({...prev, importing: true}));
        setStatus({type: null, message: "Importando..."});
        try {
            const msg = await settingsService.importSteamLibrary(
                keys.steamId,
                keys.steamApiKey
            );
            setStatus({type: "success", message: msg});
            onLibraryUpdate();
        } catch (error) {
            setStatus({type: "error", message: String(error)});
        } finally {
            setLoading((prev) => ({...prev, importing: false}));
        }
    };

    const enrichLibrary = async () => {
        setLoading((prev) => ({...prev, enriching: true}));
        setStatus({type: null, message: "Buscando dados extras..."});
        try {
            const summary = await settingsService.enrichLibrary();

            // Cria mensagem detalhada do resultado
            let detailedMessage = summary.message;
            if (summary.error_count > 0) {
                detailedMessage += ` | ${summary.error_count} falhas detectadas.`;
            }

            setStatus({
                type: summary.error_count === 0 ? "success" : "error",
                message: detailedMessage
            });
            onLibraryUpdate();
        } catch (error) {
            setStatus({type: "error", message: String(error)});
        } finally {
            setLoading((prev) => ({...prev, enriching: false}));
        }
    };

    const exportDatabase = async () => {
        setLoading((prev) => ({...prev, exporting: true}));
        setStatus({type: null, message: "Exportando backup..."});
        try {
            const msg = await settingsService.exportDatabase();
            setStatus({type: "success", message: msg});
        } catch (error: any) {
            if (error.message !== ERROR_MESSAGES.CANCELLED) {
                setStatus({type: "error", message: error.message || "Erro ao exportar"});
            } else {
                setStatus({type: null, message: ""});
            }
        } finally {
            setLoading((prev) => ({...prev, exporting: false}));
        }
    };

    const importDatabase = async () => {
        setLoading((prev) => ({...prev, importingBackup: true}));
        setStatus({type: null, message: "Importando backup..."});
        try {
            const msg = await settingsService.importDatabase();
            setStatus({type: "success", message: msg});
            onLibraryUpdate();
        } catch (error: any) {
            if (error.message !== ERROR_MESSAGES.CANCELLED) {
                setStatus({type: "error", message: error.message || "Erro ao importar"});
            } else {
                setStatus({type: null, message: ""});
            }
        } finally {
            setLoading((prev) => ({...prev, importingBackup: false}));
        }
    };

    return {
        keys,
        setKeys,
        loading,
        status,
        actions: {
            saveKeys,
            importLibrary,
            enrichLibrary,
            exportDatabase,
            importDatabase
        },
    };
}
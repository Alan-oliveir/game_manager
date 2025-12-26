import {useEffect, useState} from "react";
import {invoke} from "@tauri-apps/api/core";
import {Store} from '@tauri-apps/plugin-store';
import {Button} from "@/components/ui/button";
import {Input} from "@/components/ui/input";
import {Label} from "@/components/ui/label";
import {AlertCircle, CheckCircle, CloudDownload, Loader2} from "lucide-react";

interface SettingsProps {
    onLibraryUpdate: () => void;
}

export default function Settings({onLibraryUpdate}: SettingsProps) {
    const [steamId, setSteamId] = useState("");
    const [apiKey, setApiKey] = useState("");
    const [isLoading, setIsLoading] = useState(false);
    const [status, setStatus] = useState<{ type: 'success' | 'error' | null, message: string }>({
        type: null,
        message: ""
    });
    const [store, setStore] = useState<Store | null>(null);

    useEffect(() => {
        // Carregar configurações do store seguro
        const loadSettings = async () => {
            try {
                // Usar Store.load() ao invés de new Store()
                const storeInstance = await Store.load('.settings.dat');
                setStore(storeInstance);

                const savedId = await storeInstance.get<string>('steam_id');
                const savedKey = await storeInstance.get<string>('steam_api_key');

                if (savedId) setSteamId(savedId);
                if (savedKey) setApiKey(savedKey);
            } catch (error) {
                console.error("Erro ao carregar configurações:", error);
            }
        };

        loadSettings();
    }, []);

    const handleImport = async () => {
        if (!steamId || !apiKey) {
            setStatus({type: 'error', message: "Preencha o Steam ID e a API Key."});
            return;
        }

        if (!store) {
            setStatus({type: 'error', message: "Store ainda não foi inicializado."});
            return;
        }

        setIsLoading(true);
        setStatus({type: null, message: ""});

        try {
            // Salvar de forma segura
            await store.set('steam_id', steamId.trim());
            await store.set('steam_api_key', apiKey.trim());
            await store.save(); // Persiste no disco de forma criptografada

            const result = await invoke<string>("import_steam_library", {
                steamId: steamId.trim(),
                apiKey: apiKey.trim(),
            });

            setStatus({type: 'success', message: result});
            onLibraryUpdate();

        } catch (error) {
            console.error(error);
            setStatus({type: 'error', message: String(error)});

        } finally {
            setIsLoading(false);
        }
    };

    return (
        <div className="flex-1 p-8 overflow-y-auto">
            <h2 className="text-3xl font-bold mb-6">Configurações</h2>

            {/* Card de Integração Steam */}
            <div className="max-w-2xl border border-border rounded-xl bg-card p-6 shadow-sm">
                <div className="flex items-center gap-3 mb-6 border-b border-border pb-4">
                    <div className="p-2 bg-blue-900/20 rounded-lg">
                        <CloudDownload size={24}/>
                    </div>
                    <div>
                        <h3 className="text-lg font-semibold">Importação da Steam</h3>
                        <p className="text-sm text-muted-foreground">Sincronize sua biblioteca automaticamente.</p>
                    </div>
                </div>

                <div className="space-y-4">
                    <div className="grid gap-2">
                        <Label htmlFor="steamId">Steam ID (64-bit)</Label>
                        <Input
                            id="steamId"
                            placeholder="Ex: 76561198000000000"
                            value={steamId}
                            onChange={(e) => setSteamId(e.target.value)}
                        />
                        <p className="text-xs text-muted-foreground">
                            Você pode encontrar seu ID em sites como steamidfinder.com
                        </p>
                    </div>

                    <div className="grid gap-2">
                        <Label htmlFor="apiKey">Web API Key</Label>
                        <Input
                            id="apiKey"
                            type="password"
                            placeholder="Cole sua chave aqui..."
                            value={apiKey}
                            onChange={(e) => setApiKey(e.target.value)}
                        />
                        <p className="text-xs text-muted-foreground">
                            Sua chave de desenvolvedor Steam. Não compartilhamos esse dado.
                        </p>
                    </div>

                    {/* Feedback Visual de Status */}
                    {status.type && (
                        <div className={`p-3 rounded-md flex items-center gap-2 text-sm ${
                            status.type === 'success' ? 'bg-green-500/10 text-green-500' : 'bg-red-500/10 text-red-500'
                        }`}>
                            {status.type === 'success' ? <CheckCircle size={16}/> : <AlertCircle size={16}/>}
                            {status.message}
                        </div>
                    )}

                    <div className="pt-2 flex justify-end">
                        <Button
                            onClick={handleImport}
                            disabled={isLoading}
                            className="min-w-35"
                        >
                            {isLoading ? (
                                <>
                                    <Loader2 className="mr-2 h-4 w-4 animate-spin"/>
                                    Importando...
                                </>
                            ) : (
                                "Iniciar Importação"
                            )}
                        </Button>
                    </div>
                </div>
            </div>
        </div>
    );
}

import {useEffect, useState} from "react";
import {invoke} from "@tauri-apps/api/core";
import {Button} from "@/components/ui/button";
import {Input} from "@/components/ui/input";
import {Label} from "@/components/ui/label";
import {AlertCircle, CheckCircle, CloudDownload, Loader2, Save, Sparkles, Shield} from "lucide-react";

interface SettingsProps {
    onLibraryUpdate: () => void;
}

export default function Settings({onLibraryUpdate}: SettingsProps) {
    const [steamId, setSteamId] = useState("");
    const [apiKey, setApiKey] = useState("");
    const [rawgApiKey, setRawgApiKey] = useState("");
    const [isLoading, setIsLoading] = useState(false);
    const [status, setStatus] = useState<{ type: 'success' | 'error' | null, message: string }>({
        type: null,
        message: ""
    });
    const [isEnriching, setIsEnriching] = useState(false);

    useEffect(() => {
        // Carrega as configurações criptografadas ao montar o componente
        const loadSettings = async () => {
            try {
                const savedId = await invoke<string>('get_encrypted_key', {keyName: 'steam_id'});
                const savedKey = await invoke<string>('get_encrypted_key', {keyName: 'steam_api_key'});
                const savedRawg = await invoke<string>('get_encrypted_key', {keyName: 'rawg_api_key'});

                if (savedId) setSteamId(savedId);
                if (savedKey) setApiKey(savedKey);
                if (savedRawg) setRawgApiKey(savedRawg);
            } catch (error) {
                console.error("Erro ao carregar configurações:", error);
            }
        };
        loadSettings();
    }, []);

    // Função para salvar todas as configurações criptografadas
    const handleSaveAll = async () => {
        setIsLoading(true);
        setStatus({type: null, message: ""});
        try {
            if (steamId.trim()) {
                await invoke('save_encrypted_key', {
                    keyName: 'steam_id',
                    keyValue: steamId.trim()
                });
            }

            if (apiKey.trim()) {
                await invoke('save_encrypted_key', {
                    keyName: 'steam_api_key',
                    keyValue: apiKey.trim()
                });
            }

            if (rawgApiKey.trim()) {
                await invoke('save_encrypted_key', {
                    keyName: 'rawg_api_key',
                    keyValue: rawgApiKey.trim()
                });
            }

            setStatus({
                type: 'success',
                message: "Configurações salvas com segurança!"
            });
        } catch (error) {
            console.error(error);
            setStatus({
                type: 'error',
                message: `Erro ao salvar: ${error}`
            });
        } finally {
            setIsLoading(false);
        }
    };

    // Importa a biblioteca da Steam
    const handleImport = async () => {
        if (!steamId || !apiKey) {
            setStatus({type: 'error', message: "Preencha e SALVE as chaves antes de importar."});
            return;
        }
        setIsLoading(true);
        setStatus({type: null, message: "Iniciando importação da Steam..."});
        try {
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

    // Busca gênero dos jogos na Steam
    const handleEnrich = async () => {
        setIsEnriching(true);
        setStatus({type: null, message: "Buscando gêneros na Steam... Isso pode demorar."});
        try {
            const result = await invoke<string>("enrich_library");
            setStatus({type: 'success', message: result});
            onLibraryUpdate();
        } catch (error) {
            setStatus({type: 'error', message: String(error)});
        } finally {
            setIsEnriching(false);
        }
    };

    return (
        <div className="flex-1 p-8 overflow-y-auto pb-20">
            <h2 className="text-3xl font-bold mb-6">Configurações</h2>

            {/* Feedback Global */}
            {status.type && (
                <div className={`mb-6 p-4 rounded-lg flex items-center gap-3 ${
                    status.type === 'success' ? 'bg-green-500/10 text-green-500 border border-green-500/20' : 'bg-red-500/10 text-red-500 border border-red-500/20'
                }`}>
                    {status.type === 'success' ? <CheckCircle size={20}/> : <AlertCircle size={20}/>}
                    <span className="font-medium">{status.message}</span>
                </div>
            )}

            <div className="space-y-6">
                {/* Inputs para inserir KEYs */}
                <div className="grid gap-6 border border-border rounded-xl bg-card p-6">
                    <div className="flex items-center gap-2">
                        <Shield className="text-green-500" size={20}/>
                        <h3 className="text-lg font-semibold">Credenciais de API (Criptografadas)</h3>
                    </div>

                    <div className="grid gap-2">
                        <Label>Steam ID</Label>
                        <Input value={steamId} onChange={e => setSteamId(e.target.value)} placeholder="765..."/>
                    </div>

                    <div className="grid gap-2">
                        <Label>Steam API Key</Label>
                        <Input
                            type="password"
                            value={apiKey}
                            onChange={e => setApiKey(e.target.value)}
                            placeholder="••••••••••••••••"
                        />
                    </div>

                    <div className="grid gap-2">
                        <Label>RAWG API Key</Label>
                        <Input
                            type="password"
                            value={rawgApiKey}
                            onChange={e => setRawgApiKey(e.target.value)}
                            placeholder="••••••••••••••••"
                        />
                    </div>

                    <Button onClick={handleSaveAll} className="w-full mt-2" disabled={isLoading}>
                        {isLoading ? <Loader2 className="animate-spin mr-2"/> : <Save className="mr-2 h-4 w-4"/>}
                        Salvar Todas as Configurações
                    </Button>
                </div>

                {/* Botões de Execução */}
                <div className="grid md:grid-cols-2 gap-6">
                    <div className="border border-border rounded-xl bg-card p-6">
                        <div className="flex items-center gap-2 mb-4 text-blue-500">
                            <CloudDownload/> <h3 className="font-semibold text-foreground">Sincronizar Steam</h3>
                        </div>
                        <p className="text-sm text-muted-foreground mb-4">Importa jogos básicos da sua conta.</p>
                        <Button onClick={handleImport} variant="outline" className="w-full" disabled={isLoading}>
                            Iniciar Importação
                        </Button>
                    </div>

                    <div className="border border-border rounded-xl bg-card p-6">
                        <div className="flex items-center gap-2 mb-4 text-purple-500">
                            <Sparkles/> <h3 className="font-semibold text-foreground">Buscar Dados</h3>
                        </div>
                        <p className="text-sm text-muted-foreground mb-4">Busca gêneros e tags detalhados.</p>
                        <Button onClick={handleEnrich} variant="outline" className="w-full" disabled={isEnriching}>
                            {isEnriching ? "Processando..." : "Buscar Metadados"}
                        </Button>
                    </div>
                </div>
            </div>
        </div>
    );
}
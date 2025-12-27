import {useEffect, useState} from "react";
import {invoke} from "@tauri-apps/api/core";
import {Store} from '@tauri-apps/plugin-store';
import {Button} from "@/components/ui/button";
import {Input} from "@/components/ui/input";
import {Label} from "@/components/ui/label";
import {AlertCircle, CheckCircle, CloudDownload, Loader2, Sparkles, Gamepad2} from "lucide-react";

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
    const [isEnriching, setIsEnriching] = useState(false);
    const [rawgApiKey, setRawgApiKey] = useState("");

    useEffect(() => {
        // Carregar configurações do store seguro
        const loadSettings = async () => {
            try {
                // Usar Store.load() ao invés de new Store()
                const storeInstance = await Store.load('.settings.dat');
                setStore(storeInstance);

                const savedId = await storeInstance.get<string>('steam_id');
                const savedKey = await storeInstance.get<string>('steam_api_key');
                const savedRawg = await storeInstance.get<string>('rawg_api_key');

                if (savedId) setSteamId(savedId);
                if (savedKey) setApiKey(savedKey);
                if (savedRawg) setRawgApiKey(savedRawg);
            } catch (error) {
                console.error("Erro ao carregar configurações:", error);
            }
        };

        loadSettings();
    }, []);

    const handleSaveKeys = async () => {
        // Vamos criar uma função genérica ou adicionar no handleImport/handleEnrich
        // Mas o ideal é salvar assim que digita ou ter um botão "Salvar Chaves" geral.
        // Para simplificar, vamos salvar dentro do handleImport ou criar um botão de salvar específico.

        // Sugestão: Adicione isso no início do handleImport ou crie um useEffect que salva quando muda (com debounce)
        // Ou melhor: Vamos salvar ao clicar nos botões de ação existente por enquanto.
        if (store) {
            await store.set('rawg_api_key', rawgApiKey.trim());
            await store.save();
        }
    };

    // Duvida: não seria melhor salvar as chaves steam e rawg juntas numa função só, assim que o usuário digitar?

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

    const handleEnrich = async () => {
        setIsEnriching(true);
        setStatus({type: null, message: "Buscando gêneros na Steam... Isso pode demorar."});

        try {
            const result = await invoke<string>("enrich_library");
            setStatus({type: 'success', message: result});
            onLibraryUpdate(); // Atualiza a UI para ver os novos gêneros
        } catch (error) {
            setStatus({type: 'error', message: String(error)});
        } finally {
            setIsEnriching(false);
        }
    };

    return (
        <div className="flex-1 p-8 overflow-y-auto">
            <h2 className="text-3xl font-bold mb-6">Configurações</h2>

            {/* Card de importação de games da Steam */}
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

            {/* Card integração RAWG */}
            <div className="max-w-2xl border border-border rounded-xl bg-card p-6 shadow-sm mt-6">
                <div className="flex items-center gap-3 mb-6 border-b border-border pb-4">
                    <div className="p-2 bg-orange-900/20 rounded-lg">
                        <Gamepad2 size={24} className="text-orange-500"/>
                    </div>
                    <div>
                        <h3 className="text-lg font-semibold">Descoberta de Jogos (RAWG)</h3>
                        <p className="text-sm text-muted-foreground">Necessário para a aba "Em Alta".</p>
                    </div>
                </div>

                <div className="space-y-4">
                    <div className="grid gap-2">
                        <Label htmlFor="rawgKey">RAWG API Key</Label>
                        <Input
                            id="rawgKey"
                            type="password"
                            placeholder="Cole sua chave RAWG aqui..."
                            value={rawgApiKey}
                            onChange={(e) => setRawgApiKey(e.target.value)}
                            onBlur={handleSaveKeys} // Salva quando sair do campo
                        />
                        <p className="text-xs text-muted-foreground">
                            Obtenha gratuitamente em rawg.io/apidocs
                        </p>
                    </div>
                </div>
            </div>

            {/* Card de Enriquecimento de Dados */}
            <div className="max-w-2xl border border-border rounded-xl bg-card p-6 shadow-sm mt-6">
                <div className="flex items-center gap-3 mb-4">
                    <div className="p-2 bg-purple-900/20 rounded-lg">
                        <Sparkles size={24} className="text-purple-500"/>
                    </div>
                    <div>
                        <h3 className="text-lg font-semibold">Enriquecer Dados</h3>
                        <p className="text-sm text-muted-foreground">
                            Busca gêneros e tags reais para jogos que estão como "Desconhecido".
                        </p>
                    </div>
                </div>

                <div className="flex justify-end">
                    <Button
                        onClick={handleEnrich}
                        disabled={isEnriching}
                        variant="secondary"
                    >
                        {isEnriching ? (
                            <>
                                <Loader2 className="mr-2 h-4 w-4 animate-spin"/>
                                Processando... (Veja o terminal)
                            </>
                        ) : (
                            "Buscar Gêneros Faltantes"
                        )}
                    </Button>
                </div>
            </div>
        </div>
    );
}

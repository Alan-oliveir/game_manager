import {
  AlertCircle,
  CheckCircle,
  CloudDownload,
  Loader2,
  Save,
  Shield,
  Sparkles,
} from "lucide-react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { useSettings } from "../hooks/useSettings";

interface SettingsProps {
  onLibraryUpdate: () => void;
}

export default function Settings({ onLibraryUpdate }: SettingsProps) {
  const { keys, setKeys, loading, status, actions } =
    useSettings(onLibraryUpdate);

  if (loading.initial) {
    return (
      <div className="flex-1 p-8">
        <h2 className="text-3xl font-bold mb-6">Configurações</h2>
        <div className="flex items-center gap-2 text-muted-foreground">
          <Loader2 className="animate-spin" /> Carregando chaves...
        </div>
      </div>
    );
  }

  return (
    <div className="flex-1 p-8 overflow-y-auto custom-scrollbar pb-20">
      <h2 className="text-3xl font-bold mb-6">Configurações</h2>

      {/* Feedback Visual */}
      {status.type && (
        <div
          className={`mb-6 p-4 rounded-lg flex items-center gap-3 ${
            status.type === "success"
              ? "bg-green-500/10 text-green-500 border border-green-500/20"
              : "bg-red-500/10 text-red-500 border border-red-500/20"
          }`}
        >
          {status.type === "success" ? (
            <CheckCircle size={20} />
          ) : (
            <AlertCircle size={20} />
          )}
          <span className="font-medium">{status.message}</span>
        </div>
      )}

      {/* Formulário de Chaves */}
      <div className="space-y-6">
        <div className="grid gap-6 border border-border rounded-xl bg-card p-6">
          <div className="flex items-center gap-2">
            <Shield className="text-green-500" size={20} />
            <h3 className="text-lg font-semibold">Credenciais de API</h3>
          </div>

          {/* Steam ID */}
          <div className="grid gap-2">
            <Label>Steam ID</Label>
            <Input
              value={keys.steamId}
              onChange={(e) => setKeys({ ...keys, steamId: e.target.value })}
              placeholder="765..."
            />
          </div>

          {/* Steam API Key */}
          <div className="grid gap-2">
            <Label>Steam API Key</Label>
            <Input
              type="password"
              value={keys.steamApiKey}
              onChange={(e) =>
                setKeys({ ...keys, steamApiKey: e.target.value })
              }
              placeholder="••••••••••••••••"
            />
          </div>

          {/* RAWG API Key */}
          <div className="grid gap-2">
            <Label>RAWG API Key</Label>
            <Input
              type="password"
              value={keys.rawgApiKey}
              onChange={(e) => setKeys({ ...keys, rawgApiKey: e.target.value })}
              placeholder="••••••••••••••••"
            />
          </div>

          {/* Botão Salvar */}
          <Button
            onClick={actions.saveKeys}
            className="w-full mt-2"
            disabled={loading.saving}
          >
            {loading.saving ? (
              <Loader2 className="animate-spin mr-2" />
            ) : (
              <Save className="mr-2 h-4 w-4" />
            )}
            Salvar Todas as Configurações
          </Button>
        </div>

        {/* Ações de Biblioteca */}
        <div className="grid md:grid-cols-2 gap-6">
          <div className="border border-border rounded-xl bg-card p-6">
            <div className="flex items-center gap-2 mb-4 text-blue-500">
              <CloudDownload />{" "}
              <h3 className="font-semibold text-foreground">
                Sincronizar Steam
              </h3>
            </div>
            <p className="text-sm text-muted-foreground mb-4">
              Importa jogos básicos da sua conta.
            </p>
            <Button
              onClick={actions.importLibrary}
              variant="outline"
              className="w-full"
              disabled={loading.importing}
            >
              {loading.importing ? (
                <Loader2 className="animate-spin mr-2" />
              ) : (
                "Iniciar Importação"
              )}
            </Button>
          </div>

          <div className="border border-border rounded-xl bg-card p-6">
            <div className="flex items-center gap-2 mb-4 text-purple-500">
              <Sparkles />{" "}
              <h3 className="font-semibold text-foreground">Buscar Dados</h3>
            </div>
            <p className="text-sm text-muted-foreground mb-4">
              Busca gêneros e tags detalhados.
            </p>
            <Button
              onClick={actions.enrichLibrary}
              variant="outline"
              className="w-full"
              disabled={loading.enriching}
            >
              {loading.enriching ? "Processando..." : "Buscar Metadados"}
            </Button>
          </div>
        </div>
      </div>
    </div>
  );
}

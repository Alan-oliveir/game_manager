import { Dialog, DialogContent } from "@/components/ui/dialog";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Separator } from "@/components/ui/separator";
import { Clock, Gamepad2, Globe, Star, Trophy } from "lucide-react";
import { Game } from "../types";
import { useGameDetails } from "../hooks/useGameDetails";

interface GameDetailsModalProps {
  game: Game | null;
  isOpen: boolean;
  onClose: () => void;
  allGames: Game[];
  onSwitchGame: (id: string) => void;
}

export default function GameDetailsModal({
  game,
  isOpen,
  onClose,
  allGames,
  onSwitchGame,
}: GameDetailsModalProps) {
  const { details, loading, siblings } = useGameDetails(game, allGames);

  if (!game) return null;

  return (
    <Dialog open={isOpen} onOpenChange={onClose}>
      <DialogContent className="max-w-4xl max-h-[90vh] overflow-y-auto p-0 gap-0 border-none bg-card">
        {/* Header com Banner (Capa em blur) */}
        <div className="relative h-48 w-full overflow-hidden bg-muted">
          {game.cover_url && (
            <div
              className="absolute inset-0 bg-cover bg-center blur-xl opacity-50 scale-110"
              style={{ backgroundImage: `url(${game.cover_url})` }}
            />
          )}
          <div className="absolute inset-0 bg-linear-to-b from-transparent to-card" />

          <div className="absolute bottom-4 left-6 z-10 flex items-end gap-4">
            {/* Capa Principal */}
            <div className="w-32 h-48 bg-background rounded-lg shadow-2xl overflow-hidden border-4 border-card translate-y-12">
              {game.cover_url ? (
                <img
                  src={game.cover_url}
                  alt={game.name}
                  className="w-full h-full object-cover"
                />
              ) : (
                <div className="w-full h-full flex items-center justify-center bg-muted text-muted-foreground">
                  Sem capa
                </div>
              )}
            </div>

            <div className="mb-2">
              <h2 className="text-3xl font-bold text-foreground drop-shadow-md">
                {game.name}
              </h2>
              <Badge variant="secondary" className="mt-2 text-md">
                {game.platform || "PC"}
              </Badge>
            </div>
          </div>
        </div>

        {/* Corpo do Modal */}
        <div className="pt-16 px-6 pb-8 grid md:grid-cols-3 gap-8">
          {/* Coluna Esquerda: Estatísticas Locais */}
          <div className="space-y-6">
            <div className="bg-muted/30 p-4 rounded-xl space-y-4 border border-border">
              <h3 className="font-semibold flex items-center gap-2">
                <Trophy size={18} className="text-yellow-500" /> Seus Dados
              </h3>

              <div className="grid grid-cols-2 gap-4">
                <div className="space-y-1">
                  <span className="text-xs text-muted-foreground">
                    Tempo Jogado
                  </span>
                  <div className="flex items-center gap-2 font-medium">
                    <Clock size={16} /> {game.playtime}h
                  </div>
                </div>

                <div className="space-y-1">
                  <span className="text-xs text-muted-foreground">
                    Sua Nota
                  </span>
                  <div className="flex items-center gap-1 font-medium text-yellow-500">
                    <Star size={16} fill="currentColor" />
                    {game.rating ? game.rating : "-"}
                  </div>
                </div>
              </div>

              <div className="space-y-1">
                <span className="text-xs text-muted-foreground">Gênero</span>
                <p className="font-medium">{game.genre || "Não definido"}</p>
              </div>
            </div>

            {/* Outras Plataformas (Siblings) */}
            {siblings.length > 0 && (
              <div className="bg-blue-500/10 p-4 rounded-xl border border-blue-500/20">
                <h4 className="text-sm font-semibold mb-2 text-blue-500 flex items-center gap-2">
                  <Gamepad2 size={16} /> Também disponível em:
                </h4>
                <div className="flex flex-wrap gap-2">
                  {siblings.map((sib) => (
                    <Button
                      key={sib.id}
                      variant="outline"
                      size="sm"
                      onClick={() => onSwitchGame(sib.id)}
                      className="bg-background"
                    >
                      {sib.platform}
                    </Button>
                  ))}
                </div>
              </div>
            )}
          </div>

          {/* Coluna Direita: Dados da Nuvem (2/3 da largura) */}
          <div className="md:col-span-2 space-y-6">
            {/* Metacritic & Links */}
            <div className="flex items-center justify-between">
              {details?.metacritic && (
                <div className="flex items-center gap-2">
                  <span
                    className={`text-xl font-bold px-3 py-1 rounded border ${
                      details.metacritic >= 75
                        ? "border-green-500 text-green-500 bg-green-500/10"
                        : details.metacritic >= 50
                        ? "border-yellow-500 text-yellow-500 bg-yellow-500/10"
                        : "border-red-500 text-red-500 bg-red-500/10"
                    }`}
                  >
                    {details.metacritic}
                  </span>
                  <span className="text-sm text-muted-foreground">
                    Metascore
                  </span>
                </div>
              )}

              {details?.website && (
                <Button variant="ghost" size="sm" asChild>
                  <a href={details.website} target="_blank" rel="noreferrer">
                    <Globe size={16} className="mr-2" /> Site Oficial
                  </a>
                </Button>
              )}
            </div>

            <Separator />

            {/* Descrição */}
            <div className="space-y-2">
              <h3 className="font-semibold text-lg">Sobre o Jogo</h3>
              {loading ? (
                <div className="space-y-2 animate-pulse">
                  <div className="h-4 bg-muted rounded w-full" />
                  <div className="h-4 bg-muted rounded w-5/6" />
                  <div className="h-4 bg-muted rounded w-4/6" />
                </div>
              ) : details ? (
                <p className="text-muted-foreground leading-relaxed whitespace-pre-line text-sm">
                  {details.description_raw || "Sem descrição disponível."}
                </p>
              ) : (
                <p className="text-muted-foreground italic">
                  Não foi possível carregar detalhes online.
                </p>
              )}
            </div>

            {/* Tags */}
            {details?.tags && details.tags.length > 0 && (
              <div className="space-y-2">
                <h4 className="text-xs font-semibold text-muted-foreground uppercase tracking-wider">
                  Tags Populares
                </h4>
                <div className="flex flex-wrap gap-2">
                  {details.tags.slice(0, 10).map((tag) => (
                    <Badge
                      key={tag.id}
                      variant="secondary"
                      className="font-normal"
                    >
                      {tag.name}
                    </Badge>
                  ))}
                </div>
              </div>
            )}
          </div>
        </div>
      </DialogContent>
    </Dialog>
  );
}

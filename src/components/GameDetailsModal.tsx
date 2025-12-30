import {
  Dialog,
  DialogContent,
  DialogTitle,
} from "@/components/ui/dialog";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Separator } from "@/components/ui/separator";
import { ScrollArea } from "@/components/ui/scroll-area";
import {
  Clock,
  Gamepad2,
  Globe,
  Star,
  Trophy,
  Building2,
  Tag,
  X
} from "lucide-react";
import { Game } from "../types";
import { useGameDetails } from "../hooks/useGameDetails";
import { cn } from "@/lib/utils";

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
        {/* Modal: Ocupa quase toda a tela em mobile (95vh) para dar espaço ao conteúdo */}
        <DialogContent className="max-w-[95vw] lg:max-w-7xl h-[95vh] md:h-[90vh] p-0 border-none bg-background flex flex-col overflow-hidden shadow-2xl">

          {/* Título para acessibilidade */}
          <DialogTitle className="sr-only">{game.name} - Detalhes</DialogTitle>

          {/* Botão Fechar */}
          <button
              onClick={onClose}
              className="absolute right-3 top-3 md:right-4 md:top-4 z-50 p-2 bg-black/50 hover:bg-black/80 text-white rounded-full backdrop-blur-sm transition-all"
          >
            <X size={20} />
          </button>

          {/* HEADER (Banner) - Altura responsiva (h-48 mobile / h-64 desktop) */}
          <div className="relative shrink-0 h-48 md:h-64 w-full bg-muted overflow-hidden">
            {/* Background Image */}
            {game.cover_url && (
                <div
                    className="absolute inset-0 bg-cover bg-center blur-2xl opacity-50 scale-110"
                    style={{ backgroundImage: `url(${game.cover_url})` }}
                />
            )}
            <div className="absolute inset-0 bg-gradient-to-b from-black/20 via-transparent to-background" />

            {/* Conteúdo do Header */}
            <div className="absolute bottom-0 left-0 w-full p-6 md:p-8 flex items-end gap-6 md:gap-8 z-10">

              {/* Capa (Box Art) - Escondida em mobile muito pequeno, visível em sm+ */}
              <div className="relative shrink-0 w-32 h-48 md:w-40 md:h-60 rounded-lg shadow-2xl overflow-hidden border-4 border-background -mb-12 shadow-black/50 hidden sm:block">
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

              {/* Título e Badges */}
              <div className="mb-2 md:mb-4 flex-1 space-y-2">
                <div className="flex items-center gap-2 md:gap-3">
                  <Badge className="bg-primary/20 text-primary hover:bg-primary/30 border-primary/20 backdrop-blur-md text-[10px] md:text-xs">
                    {game.platform || "PC"}
                  </Badge>
                  {game.rating && (
                      <div className="flex items-center gap-1 text-yellow-400 font-bold text-[10px] md:text-sm bg-black/40 backdrop-blur-md px-2 py-0.5 md:px-3 md:py-1 rounded-full border border-white/10">
                        <Star size={12} className="md:w-3.5 md:h-3.5" fill="currentColor" /> {game.rating}
                      </div>
                  )}
                </div>
                {/* Título responsivo: text-2xl em mobile -> text-5xl em desktop */}
                <h2 className="text-2xl sm:text-4xl lg:text-5xl font-black text-white drop-shadow-lg tracking-tight line-clamp-2 leading-tight">
                  {game.name}
                </h2>
              </div>
            </div>
          </div>

          {/* CORPO (Grid Responsivo)
              Mobile: Flex Column (Sidebar em cima, Descrição embaixo)
              Desktop: Grid 12 colunas (Sidebar esquerda, Descrição direita)
          */}
          <div className="flex-1 overflow-hidden flex flex-col lg:grid lg:grid-cols-12 bg-background min-h-0">

            {/* COLUNA 1: Sidebar de Informações
                Mobile: Altura 45% da área restante, borda inferior.
                Desktop: Altura total, col-span-4, borda direita.
            */}
            <ScrollArea className="h-[45%] lg:h-full lg:col-span-4 border-b lg:border-b-0 lg:border-r border-border bg-muted/5 w-full custom-scrollbar">
              <div className="p-6 md:p-8 pt-6 md:pt-16 space-y-6 md:space-y-8">

                {/* Seção 1: Dados */}
                <div className="space-y-4">
                  <h3 className="text-xs md:text-sm font-bold text-muted-foreground uppercase tracking-wider flex items-center gap-2">
                    <Trophy size={14} className="md:w-4 md:h-4 text-primary" /> Seus Dados
                  </h3>
                  <div className="grid grid-cols-2 gap-3">
                    <div className="bg-card p-3 rounded-lg border shadow-sm">
                      <span className="text-[10px] md:text-xs text-muted-foreground block mb-1">Tempo Jogado</span>
                      <div className="flex items-center gap-2 font-mono font-semibold text-sm md:text-lg">
                        <Clock size={16} className="md:w-[18px] text-muted-foreground" />
                        {game.playtime}h
                      </div>
                    </div>
                    <div className="bg-card p-3 rounded-lg border shadow-sm">
                      <span className="text-[10px] md:text-xs text-muted-foreground block mb-1">Status</span>
                      <div className="flex items-center gap-2 font-medium text-sm md:text-base">
                        {game.playtime === 0 ? "Nunca Jogado" : "Em Progresso"}
                      </div>
                    </div>
                  </div>
                </div>

                <Separator />

                {/* Seção 2: Detalhes */}
                <div className="space-y-3">
                  <h3 className="text-xs md:text-sm font-bold text-muted-foreground uppercase tracking-wider">
                    Detalhes
                  </h3>
                  <div className="space-y-2 text-sm">
                    <div className="flex justify-between py-1 border-b border-border/50">
                      <span className="text-muted-foreground flex items-center gap-2 text-xs md:text-sm"><Gamepad2 size={14}/> Gênero</span>
                      <span className="font-medium text-xs md:text-sm">{game.genre || "N/A"}</span>
                    </div>

                    {details?.metacritic && (
                        <div className="flex justify-between py-1 border-b border-border/50 items-center">
                          <span className="text-muted-foreground text-xs md:text-sm">Metascore</span>
                          <Badge variant="outline" className={cn(
                              "font-bold border-2 text-[10px] md:text-xs",
                              details.metacritic >= 75 ? "border-green-500/50 text-green-500" :
                                  details.metacritic >= 50 ? "border-yellow-500/50 text-yellow-500" :
                                      "border-red-500/50 text-red-500"
                          )}>
                            {details.metacritic}
                          </Badge>
                        </div>
                    )}

                    {details?.developers && details.developers.length > 0 && (
                        <div className="flex justify-between py-1 border-b border-border/50">
                          <span className="text-muted-foreground flex items-center gap-2 text-xs md:text-sm"><Building2 size={14}/> Dev</span>
                          <span className="font-medium truncate max-w-[120px] text-right text-xs md:text-sm">{details.developers[0].name}</span>
                        </div>
                    )}
                  </div>
                </div>

                {/* Seção 3: Tags */}
                {details?.tags && details.tags.length > 0 && (
                    <div className="space-y-3">
                      <h3 className="text-xs md:text-sm font-bold text-muted-foreground uppercase tracking-wider flex items-center gap-2">
                        <Tag size={14} className="md:w-4 md:h-4" /> Tags
                      </h3>
                      <div className="flex flex-wrap gap-1.5 md:gap-2">
                        {details.tags.slice(0, 8).map(tag => (
                            <Badge key={tag.id} variant="secondary" className="font-normal bg-secondary/50 hover:bg-secondary text-[10px] md:text-xs">
                              {tag.name}
                            </Badge>
                        ))}
                      </div>
                    </div>
                )}

                {/* Links */}
                <div className="space-y-4 pt-2">
                  {siblings.length > 0 && (
                      <div className="space-y-2">
                        <span className="text-xs text-muted-foreground">Outras Versões:</span>
                        <div className="flex flex-wrap gap-2">
                          {siblings.map((sib) => (
                              <Button
                                  key={sib.id}
                                  variant="outline"
                                  size="sm"
                                  onClick={() => onSwitchGame(sib.id)}
                                  className="h-6 text-xs"
                              >
                                {sib.platform}
                              </Button>
                          ))}
                        </div>
                      </div>
                  )}

                  {details?.website && (
                      <Button variant="default" className="w-full h-8 md:h-10 text-xs md:text-sm" asChild>
                        <a href={details.website} target="_blank" rel="noreferrer">
                          <Globe size={14} className="mr-2" /> Visitar Site Oficial
                        </a>
                      </Button>
                  )}
                </div>
              </div>
            </ScrollArea>

            {/* COLUNA 2: Descrição
                Mobile: Flex-1 (Ocupa o resto da altura).
                Desktop: Altura total, col-span-8.
            */}
            <ScrollArea className="flex-1 lg:h-full lg:col-span-8 bg-background p-6 md:p-10 w-full custom-scrollbar">
              <div className="max-w-3xl mx-auto space-y-4 md:space-y-6 pb-8">
                <h3 className="text-xl md:text-2xl font-bold border-b pb-4 mb-4 md:mb-6">Sobre o Jogo</h3>

                {loading ? (
                    <div className="space-y-4 animate-pulse opacity-50">
                      <div className="h-4 bg-muted rounded w-full" />
                      <div className="h-4 bg-muted rounded w-full" />
                      <div className="h-4 bg-muted rounded w-3/4" />
                      <div className="space-y-2 pt-8">
                        <div className="h-4 bg-muted rounded w-full" />
                        <div className="h-4 bg-muted rounded w-5/6" />
                      </div>
                    </div>
                ) : details ? (
                    <div className="prose prose-sm md:prose-lg dark:prose-invert max-w-none text-foreground/80 leading-relaxed whitespace-pre-line">
                      {details.description_raw || "Nenhuma descrição fornecida pelo desenvolvedor."}
                    </div>
                ) : (
                    <div className="flex flex-col items-center justify-center py-10 md:py-20 text-muted-foreground border-2 border-dashed border-border rounded-xl">
                      <p className="text-sm md:text-base">Não foi possível carregar a descrição online.</p>
                      <Button variant="link" size="sm" onClick={() => window.location.reload()}>Tentar novamente</Button>
                    </div>
                )}
              </div>
            </ScrollArea>

          </div>
        </DialogContent>
      </Dialog>
  );
}
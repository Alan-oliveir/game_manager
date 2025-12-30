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
        {/* Modal responsivo: 95vw em mobile, 7xl em telas grandes */}
        <DialogContent className="max-w-[95vw] lg:max-w-7xl h-[90vh] p-0 border-none bg-background flex flex-col overflow-hidden shadow-2xl">

          {/* Título para acessibilidade (Invisível) */}
          <DialogTitle className="sr-only">{game.name} - Detalhes</DialogTitle>

          {/* BOTÃO FECHAR CUSTOMIZADO */}
          {/* Z-Index 50 garante que fique acima do banner */}
          <button
              onClick={onClose}
              className="absolute right-4 top-4 z-50 p-2 bg-black/50 hover:bg-black/80 text-white rounded-full backdrop-blur-sm transition-all"
          >
            <X size={20} />
          </button>

          {/* HEADER (Banner) */}
          <div className="relative shrink-0 h-64 w-full bg-muted overflow-hidden">
            {/* Imagem de Fundo (Blur) */}
            {game.cover_url && (
                <div
                    className="absolute inset-0 bg-cover bg-center blur-2xl opacity-50 scale-110"
                    style={{ backgroundImage: `url(${game.cover_url})` }}
                />
            )}
            {/* Gradiente para suavizar a transição para o corpo */}
            <div className="absolute inset-0 bg-linear-to-b from-black/20 via-transparent to-background" />

            {/* Conteúdo do Header */}
            <div className="absolute bottom-0 left-0 w-full p-8 flex items-end gap-8 z-10">
              {/* Capa do Jogo (Box Art) */}
              <div className="relative shrink-0 w-40 h-60 rounded-lg shadow-2xl overflow-hidden border-4 border-background -mb-12 shadow-black/50 hidden sm:block">
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

              {/* Informações Principais (Título) */}
              <div className="mb-4 flex-1 space-y-2">
                <div className="flex items-center gap-3">
                  <Badge className="bg-primary/20 text-primary hover:bg-primary/30 border-primary/20 backdrop-blur-md">
                    {game.platform || "PC"}
                  </Badge>
                  {game.rating && (
                      <div className="flex items-center gap-1 text-yellow-400 font-bold text-sm bg-black/40 backdrop-blur-md px-3 py-1 rounded-full border border-white/10">
                        <Star size={14} fill="currentColor" /> {game.rating}
                      </div>
                  )}
                </div>
                <h2 className="text-5xl font-black text-white drop-shadow-lg tracking-tight line-clamp-2">
                  {game.name}
                </h2>
              </div>
            </div>
          </div>

          {/* CORPO (Grid de Conteúdo) */}
          <div className="flex-1 overflow-hidden grid grid-cols-1 lg:grid-cols-12 bg-background min-h-0">

            {/* COLUNA ESQUERDA: Sidebar de Informações (4 colunas) */}
            {/* ScrollArea customizada para metadados */}
            <ScrollArea className="lg:col-span-4 border-r border-border h-full bg-muted/5 w-full custom-scrollbar">
              <div className="p-8 pt-16 space-y-8">

                {/* Seção 1: Estatísticas do Usuário */}
                <div className="space-y-4">
                  <h3 className="text-sm font-bold text-muted-foreground uppercase tracking-wider flex items-center gap-2">
                    <Trophy size={16} className="text-primary" /> Seus Dados
                  </h3>
                  <div className="grid grid-cols-2 gap-3">
                    <div className="bg-card p-3 rounded-lg border shadow-sm">
                      <span className="text-xs text-muted-foreground block mb-1">Tempo Jogado</span>
                      <div className="flex items-center gap-2 font-mono font-semibold text-lg">
                        <Clock size={18} className="text-muted-foreground" />
                        {game.playtime}h
                      </div>
                    </div>
                    <div className="bg-card p-3 rounded-lg border shadow-sm">
                      <span className="text-xs text-muted-foreground block mb-1">Status</span>
                      <div className="flex items-center gap-2 font-medium">
                        {/* Lógica simples de status baseada no tempo */}
                        {game.playtime === 0 ? "Nunca Jogado" : "Em Progresso"}
                      </div>
                    </div>
                  </div>
                </div>

                <Separator />

                {/* Seção 2: Detalhes do Jogo (Metadados) */}
                <div className="space-y-4">
                  <h3 className="text-sm font-bold text-muted-foreground uppercase tracking-wider flex items-center gap-2">
                    Detalhes
                  </h3>

                  <div className="space-y-3 text-sm">
                    {/* Gênero */}
                    <div className="flex justify-between py-1 border-b border-border/50">
                      <span className="text-muted-foreground flex items-center gap-2"><Gamepad2 size={14}/> Gênero</span>
                      <span className="font-medium">{game.genre || "N/A"}</span>
                    </div>

                    {/* Metacritic (Se houver) */}
                    {details?.metacritic && (
                        <div className="flex justify-between py-1 border-b border-border/50 items-center">
                          <span className="text-muted-foreground">Metascore</span>
                          <Badge variant="outline" className={cn(
                              "font-bold border-2",
                              details.metacritic >= 75 ? "border-green-500/50 text-green-500" :
                                  details.metacritic >= 50 ? "border-yellow-500/50 text-yellow-500" :
                                      "border-red-500/50 text-red-500"
                          )}>
                            {details.metacritic}
                          </Badge>
                        </div>
                    )}

                    {/* Desenvolvedores (Se houver) */}
                    {details?.developers && details.developers.length > 0 && (
                        <div className="flex justify-between py-1 border-b border-border/50">
                          <span className="text-muted-foreground flex items-center gap-2"><Building2 size={14}/> Dev</span>
                          <span className="font-medium truncate max-w-[150px] text-right">{details.developers[0].name}</span>
                        </div>
                    )}
                  </div>
                </div>

                {/* Seção 3: Tags */}
                {details?.tags && details.tags.length > 0 && (
                    <div className="space-y-3">
                      <h3 className="text-sm font-bold text-muted-foreground uppercase tracking-wider flex items-center gap-2">
                        <Tag size={16} /> Tags
                      </h3>
                      <div className="flex flex-wrap gap-2">
                        {details.tags.slice(0, 8).map(tag => (
                            <Badge key={tag.id} variant="secondary" className="font-normal bg-secondary/50 hover:bg-secondary">
                              {tag.name}
                            </Badge>
                        ))}
                      </div>
                    </div>
                )}

                {/* Seção 4: Links e Outras Plataformas */}
                <div className="space-y-4 pt-4">
                  {/* Botões de Plataforma (Siblings) */}
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
                                  className="h-7 text-xs"
                              >
                                {sib.platform}
                              </Button>
                          ))}
                        </div>
                      </div>
                  )}

                  {/* Website */}
                  {details?.website && (
                      <Button variant="default" className="w-full" asChild>
                        <a href={details.website} target="_blank" rel="noreferrer">
                          <Globe size={16} className="mr-2" /> Visitar Site Oficial
                        </a>
                      </Button>
                  )}
                </div>

              </div>
            </ScrollArea>

            {/* COLUNA DIREITA: Descrição (8 colunas) */}
            <ScrollArea className="lg:col-span-8 h-full bg-background p-8 md:p-10 w-full custom-scrollbar">
              <div className="max-w-3xl mx-auto space-y-6">
                <h3 className="text-2xl font-bold border-b pb-4 mb-6">Sobre o Jogo</h3>

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
                    <div className="prose prose-lg dark:prose-invert max-w-none text-foreground/80 leading-relaxed whitespace-pre-line">
                      {/* description_raw vem da RAWG como texto puro mas com quebras de linha.
                        A classe whitespace-pre-line garante que os parágrafos sejam respeitados.
                     */}
                      {details.description_raw || "Nenhuma descrição fornecida pelo desenvolvedor."}
                    </div>
                ) : (
                    <div className="flex flex-col items-center justify-center py-20 text-muted-foreground border-2 border-dashed border-border rounded-xl">
                      <p>Não foi possível carregar a descrição online.</p>
                      <Button variant="link" onClick={() => window.location.reload()}>Tentar novamente</Button>
                    </div>
                )}
              </div>
            </ScrollArea>

          </div>
        </DialogContent>
      </Dialog>
  );
}
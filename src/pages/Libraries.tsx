import StandardGameCard from "../components/StandardGameCard";
import { Game, GameActions } from "../types";
import { useMemo } from "react";
import {
  Check,
  Edit,
  Heart,
  Library,
  ListPlus,
  MoreVertical,
  Trash2,
} from "lucide-react";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import { launchGame } from "../utils/launcher";
import { usePlaylist } from "../hooks/usePlaylist";
import { toast } from "sonner";
import { ActionButton } from "@/components/ActionButton.tsx";

interface LibraryProps extends GameActions {
  games: Game[];
  searchTerm: string;
}

export default function Libraries({
  games,
  searchTerm,
  ...actions
}: LibraryProps) {
  // Instancia o hook de playlist usando a lista de jogos atual
  const { addToPlaylist, isInPlaylist } = usePlaylist(games);

  const displayedGames = useMemo(() => {
    if (!searchTerm) return games;
    const term = searchTerm.toLowerCase();
    return games.filter(
      (game) =>
        game.name.toLowerCase().includes(term) ||
        (game.genre && game.genre.toLowerCase().includes(term)) ||
        (game.platform && game.platform.toLowerCase().includes(term))
    );
  }, [games, searchTerm]);

  if (displayedGames.length === 0) {
    return (
      <div className="flex-1 flex flex-col items-center justify-center text-muted-foreground text-lg">
        <Library className="w-16 h-16 mb-4 opacity-20" />
        {searchTerm
          ? "Nenhum jogo encontrado com os critérios de busca."
          : "Nenhum jogo na biblioteca. Adicione seu primeiro jogo!"}
      </div>
    );
  }

  return (
    <div className="flex-1 overflow-y-auto custom-scrollbar p-8">
      <div className="space-y-6">
        <div className="flex items-center gap-3">
          <div className="p-2 bg-blue-500/10 rounded-lg text-blue-500">
            <Library size={24} />
          </div>
          <div>
            <h1 className="text-2xl font-bold">Minha Biblioteca</h1>
            <p className="text-muted-foreground text-sm">
              {displayedGames.length} jogo
              {displayedGames.length === 1 ? "" : "s"} encontrado
              {displayedGames.length === 1 ? "" : "s"}
            </p>
          </div>
        </div>
        <div className="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-4 xl:grid-cols-5 gap-6">
          {displayedGames.map((game) => {
            const inPlaylist = isInPlaylist(game.id);

            return (
              <div key={game.id} className="relative group">
                <StandardGameCard
                  title={game.name}
                  coverUrl={game.cover_url}
                  subtitle={game.genre || "Sem gênero"}
                  rating={game.rating || undefined}
                  onClick={() => actions.onGameClick(game)}
                  onPlay={() => launchGame(game)}
                  actions={
                    <>
                      {/* Botão de Favorito */}
                      <ActionButton
                        icon={Heart}
                        variant={game.favorite ? "glass-destructive" : "glass"}
                        tooltip={
                          game.favorite
                            ? "Remover dos Favoritos"
                            : "Adicionar aos Favoritos"
                        }
                        onClick={() => actions.onToggleFavorite(game.id)}
                      />
                      {/* Menu de Opções */}
                      <DropdownMenu>
                        <DropdownMenuTrigger asChild>
                          <ActionButton
                            icon={MoreVertical}
                            variant="glass"
                            tooltip="Mais Opções"
                          />
                        </DropdownMenuTrigger>
                        <DropdownMenuContent align="end">
                          {/* OPÇÃO: Adicionar à Playlist */}
                          <DropdownMenuItem
                            disabled={inPlaylist}
                            onClick={(e) => {
                              e.stopPropagation();
                              if (!inPlaylist) {
                                addToPlaylist(game.id);
                                toast.success(
                                  `${game.name} adicionado à playlist!`
                                );
                              }
                            }}
                          >
                            {inPlaylist ? (
                              <>
                                <Check className="mr-2 h-4 w-4 text-green-500" />
                                <span className="text-muted-foreground">
                                  Na Playlist
                                </span>
                              </>
                            ) : (
                              <>
                                <ListPlus className="mr-2 h-4 w-4" />
                                <span>Playlist</span>
                              </>
                            )}
                          </DropdownMenuItem>

                          {/* OPÇÃO: Editar Jogo */}
                          <DropdownMenuItem
                            onClick={(e) => {
                              e.stopPropagation();
                              actions.onEditGame(game);
                            }}
                          >
                            <Edit className="mr-2 h-4 w-4" />
                            <span>Editar</span>
                          </DropdownMenuItem>

                          {/* OPÇÃO: Remover da Biblioteca */}
                          <DropdownMenuItem
                            className="text-red-600 focus:text-red-600 focus:bg-red-100/10"
                            onClick={(e) => {
                              e.stopPropagation();
                              actions.onDeleteGame(game.id);
                            }}
                          >
                            <Trash2 className="mr-2 h-4 w-4" />
                            <span>Excluir</span>
                          </DropdownMenuItem>
                        </DropdownMenuContent>
                      </DropdownMenu>
                    </>
                  }
                />
              </div>
            );
          })}
        </div>
      </div>
    </div>
  );
}

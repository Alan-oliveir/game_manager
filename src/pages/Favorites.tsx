import StandardGameCard from "../components/StandardGameCard";
import { Game, GameActions } from "../types";
import {
  Check,
  Edit,
  Heart,
  ListPlus,
  MoreVertical,
  Trash2,
} from "lucide-react";
import { useMemo } from "react";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import { launchGame } from "../utils/launcher";
// Importar Hooks
import { usePlaylist } from "../hooks/usePlaylist";
import { toast } from "sonner";
import { ActionButton } from "@/components/ActionButton.tsx";

interface FavoritesProps extends GameActions {
  games: Game[];
  searchTerm: string;
}

export default function Favorites({
  games,
  searchTerm,
  ...actions
}: FavoritesProps) {
  // Instancia o hook de playlist
  const { addToPlaylist, isInPlaylist } = usePlaylist(games);

  const displayedGames = useMemo(() => {
    const favorites = games.filter((g) => g.favorite);

    if (!searchTerm) return favorites;

    const term = searchTerm.toLowerCase();
    return favorites.filter(
      (game) =>
        game.name.toLowerCase().includes(term) ||
        (game.genre && game.genre.toLowerCase().includes(term)) ||
        (game.platform && game.platform.toLowerCase().includes(term))
    );
  }, [games, searchTerm]);

  if (displayedGames.length === 0) {
    return (
      <div className="flex-1 flex flex-col items-center justify-center text-muted-foreground text-lg">
        <Heart className="w-16 h-16 mb-4 opacity-20" />
        {searchTerm
          ? "Nenhum favorito encontrado com os critérios de busca."
          : "Adicione alguns jogos à sua lista de favoritos!"}
      </div>
    );
  }

  return (
    <div className="flex-1 overflow-y-auto custom-scrollbar p-8">
      <div className="space-y-6">
        <div className="flex items-center gap-3">
          <div className="p-2 bg-pink-500/10 rounded-lg text-pink-500">
            <Heart size={24} />
          </div>
          <div>
            <h1 className="text-2xl font-bold">Meus Favoritos</h1>
            <p className="text-muted-foreground text-sm">
              {displayedGames.length} jogo
              {displayedGames.length === 1 ? "" : "s"} amado
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
                        tooltip="Remover dos Favoritos"
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
                          {/* PLAYLIST */}
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

                          {/* EDITAR */}
                          <DropdownMenuItem
                            onClick={(e) => {
                              e.stopPropagation();
                              actions.onEditGame(game);
                            }}
                          >
                            <Edit className="mr-2 h-4 w-4" />
                            <span>Editar</span>
                          </DropdownMenuItem>

                          {/* REMOVER */}
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

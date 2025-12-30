import StandardGameCard from "../components/StandardGameCard";
import { Game, GameActions } from "../types";
import {Heart, MoreVertical, Edit, Trash2} from "lucide-react";
import { useMemo } from "react";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import { launchGame } from "../utils/launcher";

interface FavoritesProps extends GameActions {
  games: Game[];
  searchTerm: string;
}

export default function Favorites({
  games,
  searchTerm,
  ...actions
}: FavoritesProps) {
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

  if (
    displayedGames.length === 0 &&
    games.filter((g) => g.favorite).length === 0
  ) {
    return (
      <div className="flex-1 flex flex-col items-center justify-center text-muted-foreground">
        <Heart className="w-16 h-16 mb-4 opacity-20" />
        <p>Você ainda não tem favoritos.</p>
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
              {displayedGames.length} jogo{displayedGames.length === 1 ? "" : "s"}{" "}
              amado{displayedGames.length === 1 ? "" : "s"}
            </p>
          </div>
        </div>
        <div className="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-4 xl:grid-cols-5 gap-6">
          {displayedGames.map((game) => (
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
                    <button
                      onClick={(e) => {
                        e.stopPropagation();
                        actions.onToggleFavorite(game.id);
                      }}
                      className="p-2 bg-black/60 backdrop-blur-sm rounded-full hover:bg-black/80 transition-colors z-10"
                      title="Remover dos Favoritos"
                    >
                      <Heart
                        size={18}
                        className={
                          game.favorite
                            ? "fill-red-500 text-red-500"
                            : "text-white"
                        }
                      />
                    </button>

                    {/* Menu de Opções */}
                    <DropdownMenu>
                      <DropdownMenuTrigger asChild>
                        <button
                          className="p-2 rounded-full bg-black/60 text-white hover:bg-black/80 backdrop-blur-md"
                          onClick={(e) => e.stopPropagation()}
                          title="Mais Opções"
                        >
                          <MoreVertical size={18} />
                        </button>
                      </DropdownMenuTrigger>
                      <DropdownMenuContent align="end">
                        <DropdownMenuItem
                          onClick={(e) => {
                            e.stopPropagation();
                            actions.onEditGame(game);
                          }}
                        >
                          <Edit className="mr-2 h-4 w-4" />
                          <span>Editar</span>
                        </DropdownMenuItem>
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
          ))}
        </div>
      </div>
    </div>
  );
}

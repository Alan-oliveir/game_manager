import { useEffect, useState } from "react";
import {
  Clock,
  ExternalLink,
  Filter,
  Flame,
  Heart,
  Loader2,
  TrendingUp,
  Gamepad2,
} from "lucide-react";
import { Button } from "@/components/ui/button";
import { Game, RawgGame } from "../types";
import { useTrending } from "../hooks/useTrending";
import { useRecommendation } from "../hooks/useRecommendation";
import { openExternalLink } from "../utils/navigation";
import StandardGameCard from "@/components/StandardGameCard.tsx";
import { trendingService } from "../services/trendingService";
import { toast } from "sonner";
import Hero from "@/components/Hero";
import { useNetworkStatus } from "../hooks/useNetworkStatus";
import { ErrorState } from "../components/ErrorState";
import { ActionButton } from "@/components/ActionButton.tsx";

interface TrendingProps {
  userGames: Game[];
  onChangeTab: (tab: string) => void;
  cachedGames: RawgGame[];
  setCachedGames: (games: RawgGame[]) => void;
}

export default function Trending(props: TrendingProps) {
  const isOnline = useNetworkStatus();

  const {
    games,
    allGenres,
    loading,
    error,
    selectedGenre,
    setSelectedGenre,
    addToWishlist,
  } = useTrending(props);

  const { calculateAffinity, profile } = useRecommendation();
  const [upcomingGames, setUpcomingGames] = useState<RawgGame[]>([]);
  const [heroIndex, setHeroIndex] = useState(0);

  useEffect(() => {
    const fetchUpcoming = async () => {
      try {
        const apiKey = await trendingService.getApiKey();
        if (apiKey) {
          const upcoming = await trendingService.getUpcoming(apiKey);
          setUpcomingGames(upcoming);
        }
      } catch (e) {
        console.error("Erro ao buscar lançamentos:", e);
      }
    };
    fetchUpcoming();
  }, []);

  const handleRetry = () => {
    window.location.reload();
  };

  const handleWishlistClick = async (game: RawgGame) => {
    try {
      await addToWishlist(game);
      toast.success(`${game.name} adicionado!`, {
        description: "O jogo já está na sua lista de desejos.",
      });
    } catch {
      toast.error("Erro ao adicionar à lista", {
        description: "Verifique sua conexão e tente novamente.",
      });
    }
  };

  if (!isOnline) {
    return (
      <ErrorState
        type="offline"
        onAction={() => props.onChangeTab("libraries")}
      />
    );
  }

  if (error) {
    const isConfigError = error.includes("401") || error.includes("Key");

    if (isConfigError) {
      return (
        <ErrorState
          type="config"
          onAction={() => props.onChangeTab("settings")}
        />
      );
    }

    return <ErrorState type="api" message={error} onRetry={handleRetry} />;
  }

  // Loading State
  if (loading) {
    return (
      <div className="flex-1 flex flex-col items-center justify-center space-y-4">
        <Loader2 className="w-10 h-10 animate-spin text-primary" />
        <p className="text-muted-foreground">
          Consultando tendências mundiais...
        </p>
      </div>
    );
  }

  // Lógica do Hero e Grid
  const heroGames = games.slice(0, 5);
  const currentHero = heroGames[heroIndex];
  const nextHero = () => setHeroIndex((prev) => (prev + 1) % heroGames.length);
  const prevHero = () =>
    setHeroIndex((prev) => (prev - 1 + heroGames.length) % heroGames.length);

  if (!currentHero) {
    // Empty State
    return (
      <div className="flex-1 flex flex-col items-center justify-center text-muted-foreground">
        <div className="p-4 bg-muted/20 rounded-full mb-4">
          <Gamepad2 className="w-12 h-12 opacity-50" />
        </div>
        <h3 className="text-lg font-medium">Nenhum jogo encontrado</h3>
        <p className="text-sm max-w-xs text-center mt-2">
          Não conseguimos carregar as sugestões no momento. Verifique seus
          filtros ou tente recarregar.
        </p>
        <Button variant="outline" className="mt-6" onClick={handleRetry}>
          Recarregar
        </Button>
      </div>
    );
  }

  let gridGames = games.slice(5, 15);
  if (profile) {
    gridGames = [...gridGames].sort((a, b) => {
      const scoreA = calculateAffinity(a.genres);
      const scoreB = calculateAffinity(b.genres);
      return scoreB - scoreA;
    });
  }

  // Renderização Principal
  return (
    <div className="flex-1 overflow-y-auto custom-scrollbar bg-background pb-10">
      {/* 1. Hero */}
      <Hero
        title={currentHero.name}
        backgroundUrl={currentHero.background_image}
        coverUrl={currentHero.background_image}
        genres={currentHero.genres.map((g) => g.name)} // Normaliza gêneros
        rating={currentHero.rating}
        showNavigation={heroGames.length > 1}
        onNext={nextHero}
        onPrev={prevHero}
        // Composição: Badge específica de Trending
        badges={
          <div className="inline-flex items-center gap-2 px-3 py-1 bg-orange-500/20 text-orange-400 rounded-full text-sm font-medium border border-orange-500/20">
            <Flame size={16} /> EM ALTA
          </div>
        }
        // Composição: Botões específicos de Trending
        actions={
          <>
            <Button
              variant="secondary"
              className="gap-2"
              onClick={() => handleWishlistClick(currentHero)}
            >
              <Heart size={18} className="text-red-500" /> Lista de Desejos
            </Button>

            <Button
              variant="outline"
              className="gap-2 bg-transparent text-white border-white/20 hover:bg-white/10"
              onClick={() =>
                openExternalLink(`https://rawg.io/games/${currentHero.id}`)
              }
            >
              <ExternalLink size={18} /> Ver Detalhes
            </Button>
          </>
        }
      />

      {/* 2. Barra de Filtros */}
      <div className="sticky top-0 z-20 bg-background/80 backdrop-blur-md border-b border-border p-4 shadow-sm">
        <div className="max-w-7xl mx-auto flex flex-wrap items-center gap-4 px-6">
          <div className="flex items-center gap-2 text-muted-foreground">
            <Filter size={18} />
            <span className="text-sm font-medium">Filtrar:</span>
          </div>

          <select
            value={selectedGenre}
            onChange={(e) => setSelectedGenre(e.target.value)}
            className="px-3 py-1.5 bg-secondary text-secondary-foreground rounded-md text-sm border-none focus:ring-1 focus:ring-primary cursor-pointer"
          >
            <option value="all">Todos os Gêneros</option>
            {allGenres.map((g) => (
              <option key={g} value={g}>
                {g}
              </option>
            ))}
          </select>

          <div className="ml-auto text-sm text-muted-foreground hidden sm:block">
            15 sugestões disponíveis
          </div>
        </div>
      </div>

      {/* 3. Sugestões (Trending) */}
      <div className="px-6 py-8 max-w-7xl mx-auto">
        <div className="flex items-center gap-2 mb-6">
          <div className="p-2 bg-green-500/10 rounded-lg text-green-400">
            <TrendingUp size={20} />
          </div>
          <h2 className="text-2xl font-bold">Mais Sugestões</h2>
        </div>

        <div className="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-4 xl:grid-cols-5 gap-6">
          {gridGames.map((game) => {
            // Lógica de badge baseada no perfil
            const affinity = calculateAffinity(game.genres);
            const isRecommended = affinity > 100;

            return (
              <StandardGameCard
                key={game.id}
                title={game.name}
                coverUrl={game.background_image}
                rating={game.rating}
                subtitle={game.genres
                  .map((g) => g.name)
                  .slice(0, 2)
                  .join(", ")}
                badge={isRecommended ? "TOP PICK" : undefined}
                // Ações Personalizadas do Trending (Wishlist + Details)
                actions={
                  <>
                    {/* Botões Wishlist e Detalhes */}
                    <ActionButton
                      icon={Heart}
                      variant="secondary"
                      onClick={() => handleWishlistClick(game)}
                      tooltip="Lista de Desejos"
                    />
                    <ActionButton
                      icon={ExternalLink}
                      variant="secondary"
                      size={16}
                      onClick={() =>
                        openExternalLink(`https://rawg.io/games/${game.id}`)
                      }
                      tooltip="Ver Detalhes"
                    />
                  </>
                }
              />
            );
          })}
        </div>
      </div>

      {/* 4. Lançamentos Aguardados */}
      {upcomingGames.length > 0 && (
        <div className="px-6 max-w-7xl mx-auto">
          <div className="flex items-center gap-2 mb-6">
            <div className="p-2 bg-blue-500/10 rounded-lg text-blue-400">
              <Clock size={20} />
            </div>
            <h2 className="text-2xl font-bold">Lançamentos Aguardados</h2>
          </div>

          <div className="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-5 gap-6">
            {upcomingGames.map((game) => {
              const affinity = calculateAffinity(game.genres);
              const isMatch = affinity > 50;

              return (
                <StandardGameCard
                  key={game.id}
                  title={game.name}
                  coverUrl={game.background_image}
                  subtitle={
                    game.released
                      ? `Lança: ${new Date(game.released).toLocaleDateString()}`
                      : "Em breve"
                  }
                  badge={isMatch ? "PARA VOCÊ" : undefined}
                  actions={
                    <>
                      {/* Botões Wishlist e Detalhes */}
                      <ActionButton
                        icon={Heart}
                        variant="secondary"
                        onClick={() => handleWishlistClick(game)}
                        tooltip="Lista de Desejos"
                      />
                      <ActionButton
                        icon={ExternalLink}
                        variant="secondary"
                        size={16}
                        onClick={() =>
                          openExternalLink(`https://rawg.io/games/${game.id}`)
                        }
                        tooltip="Ver Detalhes"
                      />
                    </>
                  }
                />
              );
            })}
          </div>
        </div>
      )}
    </div>
  );
}

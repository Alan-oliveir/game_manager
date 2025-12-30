import { useState, useEffect } from "react";
import {
  AlertCircle,
  ChevronLeft,
  ChevronRight,
  Filter,
  Flame,
  Heart,
  Loader2,
  Star,
  TrendingUp,
  ExternalLink,
  Clock,
} from "lucide-react";
import { Button } from "@/components/ui/button";
import { RawgGame, Game } from "../types";
import { useTrending } from "../hooks/useTrending";
import { useRecommendation } from "../hooks/useRecommendation";
import { openExternalLink } from "../utils/navigation";
import StandardGameCard from "@/components/StandardGameCard.tsx";
import { trendingService } from "../services/trendingService";

interface TrendingProps {
  userGames: Game[];
  onChangeTab: (tab: string) => void;
  cachedGames: RawgGame[];
  setCachedGames: (games: RawgGame[]) => void;
}

export default function Trending(props: TrendingProps) {
  // Hook Principal (Trending)
  const {
    games,
    allGenres,
    loading,
    error,
    selectedGenre,
    setSelectedGenre,
    retry,
    addToWishlist,
  } = useTrending(props);

  // Hook de Recomendação (Perfil)
  const { calculateAffinity, profile } = useRecommendation();

  // Estado para Upcoming (Lançamentos)
  const [upcomingGames, setUpcomingGames] = useState<RawgGame[]>([]);

  // Carrossel Hero
  const [heroIndex, setHeroIndex] = useState(0);

  // EFEITO: Buscar Lançamentos Aguardados
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
  }, []); // Executa uma vez na montagem

  // Helpers de UI
  const heroGames = games.slice(0, 5);
  const currentHero = heroGames[heroIndex];
  const nextHero = () => setHeroIndex((prev) => (prev + 1) % heroGames.length);
  const prevHero = () =>
    setHeroIndex((prev) => (prev - 1 + heroGames.length) % heroGames.length);

  // Jogos para o grid (Trending), ordenados por afinidade se houver perfil
  let gridGames = games.slice(5, 15);
  if (profile) {
    gridGames = [...gridGames].sort((a, b) => {
      const scoreA = calculateAffinity(a.genres);
      const scoreB = calculateAffinity(b.genres);
      return scoreB - scoreA;
    });
  }

  const handleWishlistClick = async (game: RawgGame) => {
    try {
      await addToWishlist(game);
      alert(`❤️ ${game.name} adicionado à Lista de Desejos!`);
    } catch {
      alert("Erro ao adicionar à lista.");
    }
  };

  // Renderização de Carregamento e Erro
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

  if (error) {
    return (
      <div className="flex-1 flex flex-col items-center justify-center text-center p-8">
        <div className="bg-red-500/10 p-4 rounded-full mb-4">
          <AlertCircle className="w-10 h-10 text-red-500" />
        </div>
        <h2 className="text-xl font-bold mb-2">Ops! Algo deu errado.</h2>
        <p className="text-muted-foreground mb-6 max-w-md">{error}</p>
        <div className="flex gap-3">
          <Button
            onClick={() => props.onChangeTab("settings")}
            variant="outline"
          >
            Configurar API Key
          </Button>
          <Button onClick={retry}>Tentar Novamente</Button>
        </div>
      </div>
    );
  }

  if (!currentHero) {
    return null; // Evita erro se a lista estiver vazia momentaneamente
  }

  // Renderização Principal
  return (
    <div className="flex-1 overflow-y-auto bg-background pb-10">
      {/* 1. HERO SECTION */}
      <div className="relative h-[500px] bg-background group/hero">
        <div
          className="absolute inset-0 bg-cover bg-center transition-all duration-700"
          style={{
            backgroundImage: `url(${currentHero.background_image})`,
            filter: "blur(20px) brightness(0.25)",
          }}
        />
        <div className="absolute inset-0 bg-gradient-to-t from-background via-transparent to-transparent" />

        <div className="relative h-full flex items-center px-8 max-w-7xl mx-auto z-10">
          {heroGames.length > 1 && (
            <>
              <button
                onClick={prevHero}
                className="absolute left-4 p-2 bg-black/40 hover:bg-black/60 rounded-full text-white backdrop-blur-sm transition z-20"
              >
                <ChevronLeft size={24} />
              </button>
              <button
                onClick={nextHero}
                className="absolute right-4 p-2 bg-black/40 hover:bg-black/60 rounded-full text-white backdrop-blur-sm transition z-20"
              >
                <ChevronRight size={24} />
              </button>
            </>
          )}

          <div
            className="flex flex-col md:flex-row items-center gap-8 w-full animate-in fade-in duration-500"
            key={currentHero.id}
          >
            <img
              src={currentHero.background_image || ""}
              alt={currentHero.name}
              className="w-64 md:w-80 aspect-3/4 object-cover rounded-lg shadow-2xl border border-white/10"
            />

            <div className="flex-1 space-y-4 text-center md:text-left">
              <div className="inline-flex items-center gap-2 px-3 py-1 bg-orange-500/20 text-orange-400 rounded-full text-sm font-medium border border-orange-500/20">
                <Flame size={16} /> EM ALTA
              </div>

              <h1 className="text-4xl md:text-5xl font-bold text-white leading-tight">
                {currentHero.name}
              </h1>

              <div className="flex flex-wrap gap-2 justify-center md:justify-start">
                {currentHero.genres.map((g) => (
                  <span
                    key={g.name}
                    className="px-3 py-1 bg-white/10 rounded-full text-xs text-white"
                  >
                    {g.name}
                  </span>
                ))}
              </div>

              <div className="flex items-center gap-6 justify-center md:justify-start pt-2">
                <div className="flex items-center gap-2">
                  <Star className="text-yellow-400 fill-yellow-400" size={24} />
                  <div>
                    <span className="text-2xl font-bold text-white">
                      {currentHero.rating}
                    </span>
                    <span className="text-white/50 text-sm ml-1">/ 5.0</span>
                  </div>
                </div>
              </div>

              <div className="flex gap-3 justify-center md:justify-start mt-6">
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
              </div>
            </div>
          </div>
        </div>
      </div>

      {/* 2. BARRA DE FILTROS */}
      <div className="sticky top-0 z-20 bg-background/80 backdrop-blur-md border-b border-border p-4 shadow-sm">
        <div className="max-w-7xl mx-auto flex flex-wrap items-center gap-4">
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

          <div className="ml-auto text-xs text-muted-foreground hidden sm:block">
            {games.length} sugestões disponíveis
          </div>
        </div>
      </div>

      {/* 3. GRID DE SUGESTÕES (TRENDING) */}
      <div className="p-8 max-w-7xl mx-auto">
        <div className="flex items-center gap-2 mb-6">
          <div className="p-2 bg-green-500/10 rounded-lg text-green-400">
            < TrendingUp size={20} />
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
                    <Button
                      size="sm"
                      variant="secondary"
                      className="rounded-full h-10 w-10 shadow-lg"
                      onClick={() => handleWishlistClick(game)}
                      title={"Lista de Desejos"}
                    >
                      <Heart size={16} />
                    </Button>
                    <Button
                      size="sm"
                      variant="secondary"
                      className="rounded-full h-10 w-10 shadow-lg"
                      onClick={(e) => {
                        e.stopPropagation();
                        openExternalLink(`https://rawg.io/games/${game.id}`);
                      }}
                        title={"Ver Detalhes"}
                    >
                      <ExternalLink size={16} />
                    </Button>
                  </>
                }
                onClick={() =>
                  openExternalLink(`https://rawg.io/games/${game.id}`)
                }
              />
            );
          })}
        </div>
      </div>

      {/* 4. LANÇAMENTOS AGUARDADOS */}
      {upcomingGames.length > 0 && (
        <div className="p-8 max-w-7xl mx-auto pt-0">
          <div className="flex items-center gap-2 mb-6">
            <div className="p-2 bg-blue-500/10 rounded-lg text-blue-400">
              <Clock size={20} />
            </div>
            <h2 className="text-2xl font-bold">Lançamentos Aguardados</h2>
          </div>

          <div className="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-5 gap-6">
            {upcomingGames.map((game) => {
              // Lógica de filtro por afinidade
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
                    <Button
                      size="sm"
                      variant="secondary"
                      className="w-full gap-2"
                      onClick={() => handleWishlistClick(game)}
                    >
                      <Heart size={14} /> Adicionar à Lista
                    </Button>
                  }
                  onClick={() =>
                    openExternalLink(`https://rawg.io/games/${game.id}`)
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

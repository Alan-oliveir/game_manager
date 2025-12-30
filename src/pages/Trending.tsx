import { useState } from "react";
import {
  AlertCircle,
  ChevronLeft,
  ChevronRight,
  Filter,
  Flame,
  Heart,
  Loader2,
  Sparkles,
  Star,
  TrendingUp,
  ExternalLink,
} from "lucide-react";
import { Button } from "@/components/ui/button";
import { RawgGame, Game } from "../types";
import { useTrending } from "../hooks/useTrending";
import { useRecommendation } from "../hooks/useRecommendation";
import { openExternalLink } from "../utils/navigation";

interface TrendingProps {
  userGames: Game[];
  onChangeTab: (tab: string) => void;
  cachedGames: RawgGame[];
  setCachedGames: (games: RawgGame[]) => void;
}

export default function Trending(props: TrendingProps) {
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

  // Hook de Recomendação
  const { calculateAffinity, profile } = useRecommendation();

  // Estado estritamente visual (Carrossel)
  const [heroIndex, setHeroIndex] = useState(0);

  // Helpers de UI
  const heroGames = games.slice(0, 5);
  const currentHero = heroGames[heroIndex];
  const nextHero = () => setHeroIndex((prev) => (prev + 1) % heroGames.length);
  const prevHero = () =>
    setHeroIndex((prev) => (prev - 1 + heroGames.length) % heroGames.length);

  // Jogos para o grid, ordenados por afinidade se houver perfil
  let gridGames = games.slice(5);
  if (profile) {
    gridGames = [...gridGames].sort((a, b) => {
      const scoreA = calculateAffinity(a.genres);
      const scoreB = calculateAffinity(b.genres);
      // Ordem decrescente (maior score primeiro)
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

  // Renderização de Estados de Carregamento/Erro

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

  if (heroGames.length === 0) {
    return (
      <div className="flex-1 flex flex-col items-center justify-center space-y-4">
        <p className="text-muted-foreground">
          {games.length === 0 && props.cachedGames.length === 0
            ? "Nenhum jogo encontrado."
            : "Você já tem todos os jogos em alta! 🎉"}
        </p>
        <Button onClick={retry} variant="outline">
          Atualizar
        </Button>
      </div>
    );
  }

  // Renderização Principal

  return (
    <div className="flex-1 overflow-y-auto">
      {/* Hero Section */}
      <div className="relative h-125 bg-background group/hero">
        <div
          className="absolute inset-0 bg-cover bg-center transition-all duration-700"
          style={{
            backgroundImage: `url(${currentHero.background_image})`,
            filter: "blur(20px) brightness(0.25)",
          }}
        />
        <div className="absolute inset-0 bg-linear-to-t from-background via-transparent to-transparent" />

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

      {/* Barra de filtros */}
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

      {/* Grid de sugestões */}
      <div className="p-8 max-w-7xl mx-auto">
        <div className="flex items-center gap-2 mb-6">
          <TrendingUp className="text-primary" />
          <h2 className="text-2xl font-bold">Mais Sugestões {profile && "(Recomendadas)"}</h2>
        </div>

        <div className="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-4 xl:grid-cols-5 gap-6">
          {gridGames.map((game) => {
            // Calculando afinidade para renderizar badge (Opcional)
            const affinity = calculateAffinity(game.genres);
            // Exemplo: Se o score for > 0 e estiver no top 20% do perfil (lógica simplificada aqui: > 100 pts)
            const isRecommended = affinity > 100;

            return (
                <div
                    key={game.id}
                    className="group relative bg-card rounded-xl overflow-hidden border border-border hover:shadow-xl transition-all hover:-translate-y-1"
                >
                  {/* Badge de Recomendação */}
                  {isRecommended && (
                      <div className="absolute top-2 left-2 z-10 bg-purple-600/90 backdrop-blur-md text-white text-[10px] font-bold px-2 py-1 rounded-full flex items-center gap-1 shadow-sm border border-purple-400/30">
                        <Sparkles size={10} /> TOP PICK
                      </div>
                  )}

                  <div className="aspect-video overflow-hidden relative">
                    {game.background_image ? (
                        <img
                            src={game.background_image}
                            alt={game.name}
                            className="w-full h-full object-cover transition-transform duration-500 group-hover:scale-110"
                        />
                    ) : (
                        <div className="w-full h-full bg-muted flex items-center justify-center text-muted-foreground">
                          Sem Imagem
                        </div>
                    )}

                    <div className="absolute inset-0 bg-black/60 opacity-0 group-hover:opacity-100 transition-opacity flex items-center justify-center gap-2">
                      {/* Botões Wishlist e Detalhes */}
                      <Button
                          size="sm"
                          variant="secondary"
                          className="h-8 text-xs"
                          onClick={() => handleWishlistClick(game)}
                      >
                        <Heart size={14} className="mr-1" /> Desejos
                      </Button>
                      <Button
                          size="sm"
                          variant="secondary"
                          className="h-8 text-xs"
                          onClick={(e) => {
                            e.stopPropagation();
                            openExternalLink(`https://rawg.io/games/${game.id}`);
                          }}
                      >
                        <ExternalLink size={14} /> Detalhes
                      </Button>
                    </div>
                  </div>

                  <div className="p-3">
                    {/* Info do card */}
                    <div className="flex justify-between items-start gap-2 mb-1">
                      <h3 className="font-semibold text-sm line-clamp-1" title={game.name}>
                        {game.name}
                      </h3>
                      <div className="flex items-center gap-1 shrink-0 bg-yellow-500/10 px-1.5 py-0.5 rounded text-[10px] text-yellow-500 font-bold">
                        <Star size={10} className="fill-yellow-500" /> {game.rating}
                      </div>
                    </div>
                    <div className="text-xs text-muted-foreground line-clamp-1">
                      {game.genres.map((g) => g.name).join(", ")}
                    </div>
                  </div>
                </div>
            );
          })}
        </div>
      </div>
    </div>
  );
}

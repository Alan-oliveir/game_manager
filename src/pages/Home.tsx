import { useState } from "react";
import {
  Clock,
  Gamepad2,
  Heart,
  Library,
  Sparkles,
  TrendingUp,
  Play,
  Trophy,
  Dna,
  ExternalLink,
  ChartBar,
} from "lucide-react";
import { useHome } from "../hooks/useHome";
import { Button } from "@/components/ui/button";
import { openExternalLink } from "../utils/navigation";
import { launchGame } from "../utils/launcher";
import { Game, RawgGame, UserProfile } from "../types";
import StandardGameCard from "@/components/StandardGameCard";
import { Separator } from "@/components/ui/separator.tsx";
import Hero from "@/components/Hero";

interface HomeProps {
  onChangeTab: (tab: string) => void;
  games: Game[];
  trendingCache: RawgGame[];
  setTrendingCache: (games: RawgGame[]) => void;
  profileCache: UserProfile | null;
  setProfileCache: (profile: UserProfile) => void;
}

// Utilitário para formatar tempo
const formatTime = (minutes: number) => {
  const h = Math.floor(minutes / 60);
  if (h < 1) return `${minutes}m`;
  return `${h}h`;
};

export default function Home({
  onChangeTab,
  games,
  trendingCache,
  setTrendingCache,
  profileCache,
  setProfileCache,
}: HomeProps) {
  const {
    stats,
    continuePlaying,
    backlogRecommendations,
    mostPlayed,
    topGenres,
    loading,
    trending,
  } = useHome({
    games,
    trendingCache,
    setTrendingCache,
    profileCache,
    setProfileCache,
  });

  const [heroIndex, setHeroIndex] = useState(0);

  if (loading) {
    return (
      <div className="flex-1 flex items-center justify-center text-muted-foreground">
        Carregando sua central...
      </div>
    );
  }

  // Lógica do Hero
  const heroSlides = [
    backlogRecommendations[0],
    ...(trending || []).slice(0, 2),
    mostPlayed[0],
  ].filter(Boolean);

  const currentHero = heroSlides[heroIndex] || mostPlayed[0];
  const nextHero = () => setHeroIndex((prev) => (prev + 1) % heroSlides.length);
  const prevHero = () =>
    setHeroIndex((prev) => (prev - 1 + heroSlides.length) % heroSlides.length);

  // Helper para normalizar gêneros (Home mistura tipos Game e RawgGame)
  const getGenresList = (game: any): string[] => {
    if (game.genres && Array.isArray(game.genres)) {
      return game.genres.map((g: any) => g.name); // RAWG
    }
    if (game.genre && typeof game.genre === "string") {
      return game.genre.split(",").map((g: string) => g.trim()); // Local
    }
    return [];
  };

  // Helper para imagens e nomes
  const getHeroImage = (game: any) =>
    game.cover_url || game.background_image || "";

  // Verifica se é um jogo local (possui playtime)
  const isLocalGame = (game: any) => "playtime" in game;

  return (
    <div className="flex-1 overflow-y-auto custom-scrollbar bg-background pb-10">
      {/* Hero Section */}
      {currentHero && (
        <Hero
          title={currentHero.name}
          backgroundUrl={getHeroImage(currentHero)}
          coverUrl={getHeroImage(currentHero)}
          genres={getGenresList(currentHero)}
          rating={currentHero.rating}
          showNavigation={heroSlides.length > 1}
          onNext={nextHero}
          onPrev={prevHero}
          // Badge Dinâmica da Home
          badges={
            <div className="inline-flex items-center gap-2 px-3 py-1 bg-primary/20 text-primary-foreground rounded-full text-sm font-medium border border-primary/30">
              {backlogRecommendations.some((g) => g.id === currentHero.id) && (
                <>
                  <Sparkles size={14} /> SUGESTÃO
                </>
              )}
              {trending?.some((g) => g.id === currentHero.id) && (
                <>
                  <TrendingUp size={14} /> TENDÊNCIA GLOBAL
                </>
              )}
              {mostPlayed.some((g) => g.id === currentHero.id) && (
                <>
                  <Trophy size={14} /> SEU CAMPEÃO
                </>
              )}
            </div>
          }
          // Ação Dinâmica (Local vs Remoto)
          actions={
            isLocalGame(currentHero) ? (
              <Button
                className="px-8 h-12 text-md"
                onClick={() => launchGame(currentHero)}
              >
                <Play size={20} className="mr-2" /> Jogar Agora
              </Button>
            ) : (
              <Button
                className="px-8 h-12 text-md"
                variant="secondary"
                onClick={() =>
                  openExternalLink(`https://rawg.io/games/${currentHero.id}`)
                }
              >
                <ExternalLink size={20} className="mr-2" /> Ver Detalhes
              </Button>
            )
          }
        />
      )}

      <Separator className={"mb-3"} />

      {/* Conteúdo Principal da Home */}
      <div className="p-8 max-w-7xl mx-auto space-y-10 relative z-20">
        {/* Stats Cards */}
        <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4">
          <StatCard
            icon={<Library size={24} />}
            label="Total de Jogos"
            value={stats.totalGames}
            color="text-blue-500"
            bg="bg-blue-500/10"
          />
          <StatCard
            icon={<Clock size={24} />}
            label="Tempo Jogado"
            value={formatTime(stats.totalPlaytime)}
            color="text-purple-500"
            bg="bg-purple-500/10"
          />
          <StatCard
            icon={<Heart size={24} />}
            label="Favoritos"
            value={stats.totalFavorites}
            color="text-pink-500"
            bg="bg-pink-500/10"
          />
          <StatCard
            icon={<Trophy size={24} />}
            label="Gênero Favorito"
            value={topGenres[0]?.[0] || "-"}
            color="text-yellow-500"
            bg="bg-yellow-500/10"
          />
        </div>

        {/* Continue Jogando */}
        {continuePlaying.length > 0 && (
          <section>
            <div className="flex items-center gap-2 mb-6">
              <div className="flex items-center gap-2">
                <div className="p-2 bg-blue-500/10 rounded-lg text-blue-400">
                  <Clock size={24} />
                </div>
                <h2 className="text-2xl font-bold">Continue Jogando</h2>
              </div>
            </div>
            <div className="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-5 gap-4">
              {continuePlaying.map((game) => (
                <StandardGameCard
                  key={game.id}
                  title={game.name}
                  coverUrl={game.cover_url}
                  subtitle={`${formatTime(game.playtime)} jogadas`}
                  onClick={() => launchGame(game)}
                  // Ação de Play no Hover
                  actions={
                    <Button
                      size="icon"
                      className="bg-transparent border border-transparent hover:border-white hover:bg-muted-foreground text-white p-3 rounded-full shadow-xl transform scale-90 group-hover:scale-100 transition-all"
                      onClick={() => launchGame(game)}
                      title="Jogar Agora"
                    >
                      <Play className="text-white fill-white" />
                    </Button>
                  }
                />
              ))}
            </div>
          </section>
        )}

        {/* Recomendações */}
        <section>
          <div className="flex items-center justify-between mb-6">
            <div className="flex items-center gap-2">
              <div className="flex items-center gap-2">
                <div className="p-2 bg-purple-500/10 rounded-lg text-purple-400">
                  <Dna size={24} />
                </div>
                <h2 className="text-2xl font-bold">Recomendados</h2>
              </div>
            </div>
            <Button
              variant="ghost"
              size="sm"
              onClick={() => onChangeTab("libraries")}
            >
              Ver Tudo
            </Button>
          </div>

          {backlogRecommendations.length > 0 ? (
            <div className="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-5 gap-4">
              {backlogRecommendations.slice(0, 5).map((game) => (
                <StandardGameCard
                  key={game.id}
                  title={game.name}
                  coverUrl={game.cover_url}
                  subtitle={game.genre?.split(",")[0]}
                  badge="Recomendado"
                  onClick={() => launchGame(game)}
                  actions={
                    <Button
                      size="icon"
                      className="bg-transparent border border-transparent hover:border-white hover:bg-muted-foreground text-white p-3 rounded-full shadow-xl transform scale-90 group-hover:scale-100 transition-all"
                      onClick={() => launchGame(game)}
                      title="Jogar Agora"
                    >
                      <Play className="text-white fill-white" />
                    </Button>
                  }
                />
              ))}
            </div>
          ) : (
            <div className="p-8 border border-dashed border-border rounded-xl text-center text-muted-foreground">
              <p>
                Tudo em dia! Nenhum jogo parado encontrado para o seu perfil.
              </p>
            </div>
          )}
        </section>

        {/* Grid Inferior (Mais Jogados + Gêneros) - Estatísticas */}
        <div className="flex items-center gap-2 mb-6">
          <div className="p-2 bg-yellow-500/10 rounded-lg text-yellow-400">
            <ChartBar size={24} />
          </div>
          <h2 className="text-2xl font-bold">Estatísticas</h2>
        </div>

        {/* Cards de Estatísticas */}
        <div className="grid grid-cols-1 lg:grid-cols-3 gap-8">
          <div className="lg:col-span-2 bg-card border border-border rounded-xl p-6">
            <div className="flex items-center gap-2 mb-3">
              <TrendingUp size={20} className="text-primary" />
              <h2 className="text-lg font-semibold">Mais Jogados</h2>
            </div>
            <Separator className={"mb-3"} />
            <div className="grid grid-cols-1 gap-4">
              {mostPlayed.map((game, index) => (
                <div
                  key={game.id}
                  className="flex items-center gap-3 p-2 rounded-lg hover:bg-muted/50 transition-colors cursor-pointer group"
                  onClick={() => launchGame(game)}
                >
                  <div className="font-bold text-muted-foreground w-6 text-center">
                    {index + 1}
                  </div>
                  {game.cover_url ? (
                    <img
                      src={game.cover_url}
                      alt=""
                      className="w-12 h-16 object-cover rounded bg-muted"
                    />
                  ) : (
                    <div className="w-12 h-16 bg-muted rounded flex items-center justify-center text-[10px]">
                      Sem Capa
                    </div>
                  )}
                  <div className="flex-1 min-w-0">
                    <h4 className="text-sm font-medium truncate group-hover:text-primary transition-colors">
                      {game.name}
                    </h4>
                    <div className="flex justify-between items-center mt-1">
                      <span className="text-xs text-muted-foreground truncate max-w-25">
                        {game.genre?.split(",")[0]}
                      </span>
                      <span className="text-xs font-mono bg-secondary px-1.5 py-0.5 rounded">
                        {formatTime(game.playtime)}
                      </span>
                    </div>
                  </div>
                </div>
              ))}
            </div>
          </div>

          <div className="lg:col-span-1 bg-card border border-border rounded-xl p-6 h-full">
            <div className="flex items-center gap-2 mb-3">
              <Gamepad2 size={20} className="text-primary" />
              <h2 className="text-lg font-semibold">Gêneros + Jogados</h2>
            </div>
            <Separator className={"mb-3"} />
            <div className="space-y-4">
              {topGenres.map(([genre, count]) => {
                const percent = Math.round((count / stats.totalGames) * 100);
                return (
                  <div key={genre}>
                    <div className="flex justify-between text-xs mb-1">
                      <span className="font-medium">{genre}</span>
                      <span className="text-muted-foreground">
                        {count} jogos
                      </span>
                    </div>
                    <div className="h-2 bg-secondary rounded-full overflow-hidden">
                      <div
                        className="h-full bg-primary/80 transition-all duration-1000 ease-out"
                        style={{ width: `${percent}%` }}
                      />
                    </div>
                  </div>
                );
              })}
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}

// Subcomponente StatCard
function StatCard({ icon, label, value, color, bg }: any) {
  return (
    <div className="bg-card border border-border rounded-xl p-5 flex items-center gap-4 hover:border-primary/50 transition-colors">
      <div className={`p-3 rounded-lg ${bg} ${color}`}>{icon}</div>
      <div>
        <p className="text-sm text-muted-foreground">{label}</p>
        <p className="text-2xl font-bold">{value}</p>
      </div>
    </div>
  );
}

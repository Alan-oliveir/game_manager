import {
  Clock,
  Gamepad2,
  Heart,
  Library,
  Sparkles,
  TrendingUp,
} from "lucide-react";
import { Game } from "../types";

interface HomeProps {
  games: Game[];
}

export default function Home({ games }: HomeProps) {
  // Cálculos de estatísticas
  const totalGames = games.length;
  const totalPlaytime = games.reduce(
    (acc, game) => acc + (game.playtime || 0),
    0
  );
  const totalFavorites = games.filter((g) => g.favorite).length;
  const gamesThisMonth = 5; // Mock - seria calculado pela data de adição

  // Jogos mais jogados (top 5)
  const mostPlayed = [...games]
    .filter((g) => g.playtime && g.playtime > 0)
    .sort((a, b) => (b.playtime || 0) - (a.playtime || 0))
    .slice(0, 5);

  // Adicionados recentemente (últimos 6)
  const recentlyAdded = [...games].slice(0, 6);

  // Gêneros favoritos
  const genreStats = games.reduce((acc, game) => {
    const genre = game.genre || "Desconhecido";
    acc[genre] = (acc[genre] || 0) + 1;
    return acc;
  }, {} as Record<string, number>);

  const topGenres = Object.entries(genreStats)
    .sort(([, a], [, b]) => b - a)
    .slice(0, 5);

  // Sugestão do dia (jogo aleatório com pouco tempo jogado) - CORRIGIDO
  const lowPlaytimeGames = games.filter((g) => !g.playtime || g.playtime < 5);
  const suggestion =
    lowPlaytimeGames.length > 0
      ? lowPlaytimeGames[Math.floor(Math.random() * lowPlaytimeGames.length)]
      : null;

  return (
    <div className="flex-1 overflow-y-auto p-6">
      {/* Header */}
      <div className="mb-8">
        <h1 className="text-3xl font-bold text-foreground mb-2">Início</h1>
        <p className="text-muted-foreground">
          Bem-vindo de volta! Veja um resumo da sua biblioteca.
        </p>
      </div>

      {/* Cards de Estatísticas */}
      <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4 mb-8">
        <StatCard
          icon={<Library size={24} />}
          label="Total de Jogos"
          value={totalGames}
          color="bg-blue-500/10 text-blue-500"
        />
        <StatCard
          icon={<Clock size={24} />}
          label="Horas Jogadas"
          value={`${totalPlaytime}h`}
          color="bg-purple-500/10 text-purple-500"
        />
        <StatCard
          icon={<Heart size={24} />}
          label="Favoritos"
          value={totalFavorites}
          color="bg-pink-500/10 text-pink-500"
        />
        <StatCard
          icon={<TrendingUp size={24} />}
          label="Este Mês"
          value={`+${gamesThisMonth}`}
          color="bg-green-500/10 text-green-500"
        />
      </div>

      {/* Grid Principal */}
      <div className="grid grid-cols-1 lg:grid-cols-3 gap-6 mb-8">
        {/* Jogos Mais Jogados - 2 colunas */}
        <div className="lg:col-span-2 bg-card border border-border rounded-xl p-6">
          <h2 className="text-xl font-semibold mb-4 flex items-center gap-2">
            <Gamepad2 size={20} />
            Seus Jogos Mais Jogados
          </h2>

          {mostPlayed.length > 0 ? (
            <div className="space-y-3">
              {mostPlayed.map((game, index) => (
                <div
                  key={game.id}
                  className="flex items-center gap-4 p-3 rounded-lg bg-muted/30 hover:bg-muted/50 transition-colors cursor-pointer"
                >
                  <div className="flex items-center justify-center w-8 h-8 rounded-full bg-primary/10 text-primary font-bold text-sm">
                    {index + 1}
                  </div>
                  <div className="flex-1 min-w-0">
                    <h3 className="font-medium truncate">{game.name}</h3>
                    <p className="text-sm text-muted-foreground">
                      {game.genre}
                    </p>
                  </div>
                  <div className="text-right">
                    <div className="font-semibold">{game.playtime}h</div>
                    <div className="text-xs text-muted-foreground">jogadas</div>
                  </div>
                </div>
              ))}
            </div>
          ) : (
            <div className="text-center py-8 text-muted-foreground">
              <p>Comece a jogar para ver estatísticas aqui!</p>
            </div>
          )}
        </div>

        {/* Gêneros Favoritos - 1 coluna */}
        <div className="bg-card border border-border rounded-xl p-6">
          <h2 className="text-xl font-semibold mb-4">Gêneros Favoritos</h2>

          {topGenres.length > 0 ? (
            <div className="space-y-4">
              {topGenres.map(([genre, count]) => {
                const percentage = (count / totalGames) * 100;
                return (
                  <div key={genre}>
                    <div className="flex justify-between text-sm mb-1">
                      <span className="font-medium">{genre}</span>
                      <span className="text-muted-foreground">
                        {count} jogos
                      </span>
                    </div>
                    <div className="h-2 bg-muted rounded-full overflow-hidden">
                      <div
                        className="h-full bg-primary transition-all duration-300"
                        style={{ width: `${percentage}%` }}
                      />
                    </div>
                  </div>
                );
              })}
            </div>
          ) : (
            <div className="text-center py-8 text-muted-foreground text-sm">
              <p>Adicione jogos para ver seus gêneros favoritos</p>
            </div>
          )}
        </div>
      </div>

      {/* Sugestão do Dia */}
      {suggestion && (
        <div className="bg-gradient-to-r from-purple-500/10 via-pink-500/10 to-blue-500/10 border border-purple-500/20 rounded-xl p-6 mb-8">
          <div className="flex items-center gap-2 mb-3">
            <Sparkles size={20} className="text-purple-400" />
            <h2 className="text-xl font-semibold">Sugestão do Dia</h2>
          </div>
          <div className="flex items-center gap-4">
            <div className="flex-1">
              <h3 className="text-lg font-medium mb-1">
                Que tal jogar {suggestion.name}?
              </h3>
              <p className="text-sm text-muted-foreground">
                {suggestion.playtime
                  ? `Você já jogou ${suggestion.playtime}h`
                  : "Você ainda não experimentou este jogo"}{" "}
                - hora de dar uma chance!
              </p>
            </div>
            <button className="px-4 py-2 bg-primary text-primary-foreground rounded-lg hover:opacity-90 transition-opacity whitespace-nowrap">
              Jogar Agora
            </button>
          </div>
        </div>
      )}

      {/* Adicionados Recentemente */}
      <div className="bg-card border border-border rounded-xl p-6">
        <h2 className="text-xl font-semibold mb-4">Adicionados Recentemente</h2>

        {recentlyAdded.length > 0 ? (
          <div className="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-6 gap-4">
            {recentlyAdded.map((game) => (
              <div key={game.id} className="group cursor-pointer">
                <div className="aspect-[3/4] bg-muted rounded-lg mb-2 overflow-hidden relative">
                  {game.cover_url ? (
                    <img
                      src={game.cover_url}
                      alt={game.name}
                      className="w-full h-full object-cover group-hover:scale-105 transition-transform duration-300"
                    />
                  ) : (
                    <div className="w-full h-full flex items-center justify-center">
                      <Library
                        size={32}
                        className="text-muted-foreground opacity-50"
                      />
                    </div>
                  )}
                  <div className="absolute inset-0 bg-black/0 group-hover:bg-black/40 transition-colors duration-300" />
                </div>
                <h3 className="text-sm font-medium truncate">{game.name}</h3>
                <p className="text-xs text-muted-foreground truncate">
                  {game.platform}
                </p>
              </div>
            ))}
          </div>
        ) : (
          <div className="text-center py-12 text-muted-foreground">
            <Library size={48} className="mx-auto mb-3 opacity-20" />
            <p>Adicione jogos à sua biblioteca para começar!</p>
          </div>
        )}
      </div>
    </div>
  );
}

// Componente auxiliar para os cards de estatísticas
function StatCard({
  icon,
  label,
  value,
  color,
}: {
  icon: React.ReactNode;
  label: string;
  value: string | number;
  color: string;
}) {
  return (
    <div className="bg-card border border-border rounded-xl p-5 hover:shadow-lg transition-shadow">
      <div className="flex items-center gap-3">
        <div className={`p-3 rounded-lg ${color}`}>{icon}</div>
        <div className="flex-1">
          <p className="text-sm text-muted-foreground mb-1">{label}</p>
          <p className="text-2xl font-bold">{value}</p>
        </div>
      </div>
    </div>
  );
}

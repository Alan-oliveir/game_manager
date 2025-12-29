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
    ChevronLeft,
    ChevronRight,
    ExternalLink
} from "lucide-react";
import { useHome } from "../hooks/useHome";
import { Button } from "@/components/ui/button";
import { openExternalLink } from "../utils/navigation";

// Utilitário para formatar tempo
const formatTime = (minutes: number) => {
    const h = Math.floor(minutes / 60);
    if (h < 1) return `${minutes}m`;
    return `${h}h`;
};

export default function Home({ onChangeTab }: { onChangeTab: (tab: string) => void }) {
    const {
        stats,
        continuePlaying,
        backlogRecommendations,
        mostPlayed,
        topGenres,
        loading,
        trending // Recebendo os jogos em alta da RAWG
    } = useHome();

    // Estado do Carrossel Hero
    const [heroIndex, setHeroIndex] = useState(0);

    if (loading) {
        return <div className="flex-1 flex items-center justify-center text-muted-foreground">Carregando sua central...</div>;
    }

    // --- Lógica do Hero Rotativo ---
    // Criamos um "Mix" interessante para o Banner:
    // 1. A melhor recomendação do Backlog (se houver)
    // 2. O Top 1 Trending Global
    // 3. O Jogo Mais Jogado (Favorito pessoal)
    // 4. Outros Trendings...
    const heroSlides = [
        backlogRecommendations[0],
        ...(trending || []).slice(0, 2),
        mostPlayed[0]
    ].filter(Boolean); // Remove undefined se alguma lista estiver vazia

    const currentHero = heroSlides[heroIndex] || mostPlayed[0];

    const nextHero = () => setHeroIndex((prev) => (prev + 1) % heroSlides.length);
    const prevHero = () => setHeroIndex((prev) => (prev - 1 + heroSlides.length) % heroSlides.length);

    // Helpers para renderizar dados do Hero dependendo se é local (Game) ou nuvem (RawgGame)
    const getHeroImage = (game: any) => game.cover_url || game.background_image || "";
    const getHeroName = (game: any) => game.name;
    const isLocalGame = (game: any) => "playtime" in game; // Verifica se é da biblioteca

    // Função para iniciar o jogo
    const handlePlay = (game: any) => {
        if (game.platform === "Steam") {
            // Abre o protocolo da Steam (o Windows/Linux reconhece e abre o app)
            // _self garante que não abra uma aba branca no navegador
            window.open(`steam://rungameid/${game.id}`, '_self');
        } else {
            // Futuramente implementaremos o lançamento de executáveis locais (.exe)
            alert(`Lançamento de jogos ${game.platform || "Manuais"} será implementado em breve!`);
        }
    };

    return (
        <div className="flex-1 overflow-y-auto bg-background pb-10">

            {/* Hero Section */}
            {currentHero && (
                <div className="relative h-112.5 bg-background group/hero overflow-hidden">
                    {/* Background Blur */}
                    <div
                        className="absolute inset-0 bg-cover bg-center transition-all duration-700"
                        style={{
                            backgroundImage: `url(${getHeroImage(currentHero)})`,
                            filter: "blur(20px) brightness(0.3)",
                        }}
                    />
                    <div className="absolute inset-0 bg-gradient-to-t from-background via-background/60 to-transparent" />

                    {/* Botões de Navegação */}
                    {heroSlides.length > 1 && (
                        <>
                            <button onClick={prevHero} className="absolute left-4 top-1/2 -translate-y-1/2 p-2 bg-black/40 hover:bg-black/60 rounded-full text-white backdrop-blur-sm transition z-20">
                                <ChevronLeft size={24} />
                            </button>
                            <button onClick={nextHero} className="absolute right-4 top-1/2 -translate-y-1/2 p-2 bg-black/40 hover:bg-black/60 rounded-full text-white backdrop-blur-sm transition z-20">
                                <ChevronRight size={24} />
                            </button>
                        </>
                    )}

                    <div className="relative h-full flex items-center px-8 max-w-7xl mx-auto z-10">
                        <div key={getHeroName(currentHero)} className="flex flex-col md:flex-row items-center gap-8 w-full animate-in fade-in slide-in-from-bottom-4 duration-700">
                            <img
                                src={getHeroImage(currentHero)}
                                alt={getHeroName(currentHero)}
                                className="w-48 md:w-64 aspect-3/4 object-cover rounded-lg shadow-2xl border border-white/10"
                            />

                            <div className="flex-1 space-y-4 text-center md:text-left">
                                {/* Badge Contextual */}
                                <div className="inline-flex items-center gap-2 px-3 py-1 bg-primary/20 text-primary-foreground rounded-full text-sm font-medium border border-primary/30">
                                    {backlogRecommendations.some(g => g.id === currentHero.id) && <><Sparkles size={14}/> SUGESTÃO</>}
                                    {trending?.some(g => g.id === currentHero.id) && <><TrendingUp size={14}/> TENDÊNCIA GLOBAL</>}
                                    {mostPlayed.some(g => g.id === currentHero.id) && <><Trophy size={14}/> SEU CAMPEÃO</>}
                                </div>

                                <h1 className="text-4xl md:text-5xl font-bold text-white leading-tight">
                                    {getHeroName(currentHero)}
                                </h1>

                                {/* Gêneros (Compatível com ambos os tipos de objeto) */}
                                <div className="flex flex-wrap justify-center md:justify-start gap-2 text-white/80">
                                    {/* Lógica para extrair gêneros string vs array de objetos */}
                                    {(() => {
                                        // Type guard: Game has 'genre' string, RawgGame has 'genres' array
                                        if ('genre' in currentHero && typeof currentHero.genre === 'string') {
                                            return currentHero.genre.split(',').slice(0,3).map((g:string) => <span key={g} className="px-2 py-0.5 bg-white/10 rounded text-xs">{g}</span>);
                                        }
                                        if ('genres' in currentHero && Array.isArray(currentHero.genres)) {
                                            return currentHero.genres.slice(0,3).map((g:any) => <span key={g.name} className="px-2 py-0.5 bg-white/10 rounded text-xs">{g.name}</span>);
                                        }
                                        return null;
                                    })()}
                                </div>

                                <div className="flex justify-center md:justify-start gap-4 pt-4">
                                    {isLocalGame(currentHero) ? (
                                        <Button className="px-8 h-12 text-md" onClick={() => handlePlay(currentHero)}>
                                            <Play size={20} className="mr-2" /> Jogar Agora
                                        </Button>
                                    ) : (
                                        <Button className="px-8 h-12 text-md" variant="secondary" onClick={() => openExternalLink(`https://rawg.io/games/${currentHero.id}`)}>
                                            <ExternalLink size={20} className="mr-2" /> Ver Detalhes
                                        </Button>
                                    )}
                                </div>
                            </div>
                        </div>
                    </div>
                </div>
            )}

            <div className="p-8 max-w-7xl mx-auto space-y-10 relative z-20">
                {/* Stats Cards */}
                <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4">
                    <StatCard icon={<Library size={24} />} label="Total de Jogos" value={stats.totalGames} color="text-blue-500" bg="bg-blue-500/10" />
                    <StatCard icon={<Clock size={24} />} label="Tempo Jogado" value={formatTime(stats.totalPlaytime)} color="text-purple-500" bg="bg-purple-500/10" />
                    <StatCard icon={<Heart size={24} />} label="Favoritos" value={stats.totalFavorites} color="text-pink-500" bg="bg-pink-500/10" />
                    <StatCard icon={<Trophy size={24} />} label="Gênero Favorito" value={topGenres[0]?.[0] || "-"} color="text-yellow-500" bg="bg-yellow-500/10" />
                </div>

                {/* Continue Jogando */}
                {continuePlaying.length > 0 && (
                    <section>
                        <div className="flex items-center gap-2 mb-4">
                            <Clock className="text-primary" size={24} />
                            <h2 className="text-2xl font-bold">Continue Jogando</h2>
                        </div>
                        <div className="grid grid-cols-1 md:grid-cols-5 gap-4">
                            {continuePlaying.map((game) => <GameCardSimple key={game.id} game={game} />)}
                        </div>
                    </section>
                )}

                {/* Recomendações */}
                <section>
                    <div className="flex items-center justify-between mb-4">
                        <div className="flex items-center gap-2">
                            <Dna className="text-purple-400" size={24} />
                            <h2 className="text-2xl font-bold">Recomendados</h2>
                        </div>
                        <Button variant="ghost" size="sm" onClick={() => onChangeTab("library")}>Ver Tudo</Button>
                    </div>

                    {backlogRecommendations.length > 0 ? (
                        // Grid de 5 colunas para ficar em uma linha
                        <div className="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-5 gap-4">
                            {backlogRecommendations.slice(0, 5).map((game) => (
                                <GameCardSimple key={game.id} game={game} showBadge="Recomendado" />
                            ))}
                        </div>
                    ) : (
                        <div className="p-8 border border-dashed border-border rounded-xl text-center text-muted-foreground">
                            <p>Tudo em dia! Nenhum jogo parado encontrado para o seu perfil.</p>
                        </div>
                    )}
                </section>

                {/* GRID INFERIOR (2 Colunas Mais Jogados + 1 Coluna Gêneros) */}
                <div className="grid grid-cols-1 lg:grid-cols-3 gap-8">

                    {/* Coluna: Mais Jogados */}
                    <div className="lg:col-span-2 bg-card border border-border rounded-xl p-6">
                        <div className="flex items-center gap-2 mb-4">
                            <TrendingUp size={20} className="text-primary" />
                            <h2 className="text-lg font-semibold">Mais Jogados</h2>
                        </div>
                        <div className="grid grid-cols-1 gap-4">
                            {mostPlayed.map((game, index) => (
                                <div key={game.id} className="flex items-center gap-3 p-2 rounded-lg hover:bg-muted/50 transition-colors cursor-pointer group">
                                    <div className="font-bold text-muted-foreground w-6 text-center">{index + 1}</div>
                                    {game.cover_url ? (
                                        <img src={game.cover_url} alt="" className="w-12 h-16 object-cover rounded bg-muted" />
                                    ) : (
                                        <div className="w-12 h-16 bg-muted rounded flex items-center justify-center text-[10px]">Sem Capa</div>
                                    )}
                                    <div className="flex-1 min-w-0">
                                        <h4 className="text-sm font-medium truncate group-hover:text-primary transition-colors">{game.name}</h4>
                                        <div className="flex justify-between items-center mt-1">
                                            <span className="text-xs text-muted-foreground truncate max-w-[100px]">{game.genre?.split(',')[0]}</span>
                                            <span className="text-xs font-mono bg-secondary px-1.5 py-0.5 rounded">{formatTime(game.playtime)}</span>
                                        </div>
                                    </div>
                                </div>
                            ))}
                        </div>
                    </div>

                    {/* Coluna: Top Gêneros */}
                    <div className="lg:col-span-1 bg-card border border-border rounded-xl p-6 h-full">
                        <div className="flex items-center gap-2 mb-4">
                            <Gamepad2 size={20} className="text-primary" />
                            <h2 className="text-lg font-semibold">Gêneros + Jogados</h2>
                        </div>
                        <div className="space-y-4">
                            {topGenres.map(([genre, count]) => {
                                const percent = Math.round((count / stats.totalGames) * 100);
                                return (
                                    <div key={genre}>
                                        <div className="flex justify-between text-xs mb-1">
                                            <span className="font-medium">{genre}</span>
                                            <span className="text-muted-foreground">{count} jogos</span>
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

// Subcomponentes StatCard e GameCardSimple
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

function GameCardSimple({ game, showBadge }: any) {
    return (
        <div className="group relative bg-card rounded-xl overflow-hidden border border-border hover:shadow-lg transition-all hover:-translate-y-1 cursor-pointer">
            <div className="aspect-3/4 bg-muted relative">
                {game.cover_url ? (
                    <img src={game.cover_url} alt={game.name} className="w-full h-full object-cover" />
                ) : (
                    <div className="w-full h-full flex items-center justify-center text-muted-foreground text-xs">Sem Capa</div>
                )}

                {/* Overlay Play */}
                <div className="absolute inset-0 bg-black/40 opacity-0 group-hover:opacity-100 transition-opacity flex items-center justify-center">
                    <Play className="text-white fill-white" size={32} />
                </div>

                {showBadge && (
                    <div className="absolute top-2 left-2 bg-purple-600 text-white text-[10px] font-bold px-2 py-0.5 rounded-full shadow-lg z-10">
                        {showBadge}
                    </div>
                )}
            </div>
            <div className="p-3">
                <h3 className="font-semibold text-sm truncate" title={game.name}>{game.name}</h3>
                <div className="flex justify-between items-center mt-1">
                    <span className="text-xs text-muted-foreground truncate max-w-[80px]">
                        {game.genre?.split(',')[0] || "Geral"}
                    </span>
                    {game.playtime > 0 && (
                        <span className="text-xs font-mono bg-secondary px-1 rounded">
                            {Math.floor(game.playtime / 60)}h
                        </span>
                    )}
                </div>
            </div>
        </div>
    );
}
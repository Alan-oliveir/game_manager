import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Store } from '@tauri-apps/plugin-store';
import { Flame, TrendingUp, Star, Heart, Filter, ChevronLeft, ChevronRight, Loader2, AlertCircle } from "lucide-react";
import { Game } from "../types";
import { Button } from "@/components/ui/button";

interface TrendingProps {
    userGames: Game[]; // Jogos que o usuário já tem (para filtrar)
    onChangeTab: (tab: string) => void; // Para mandar o usuário para Settings se precisar
}

// Interface para os dados que vêm do Rust (RAWG)
interface RawgGame {
    id: number;
    name: string;
    background_image: string | null;
    rating: number;
    released: string | null;
    genres: { name: string }[];
}

export default function Trending({ userGames, onChangeTab }: TrendingProps) {
    const [games, setGames] = useState<RawgGame[]>([]);
    const [loading, setLoading] = useState(true);
    const [error, setError] = useState<string | null>(null);

    const [selectedGenre, setSelectedGenre] = useState<string>("all");
    const [heroIndex, setHeroIndex] = useState(0);

    // Buscar dados ao carregar a página
    useEffect(() => {
        const fetchTrending = async () => {
            try {
                const store = await Store.load('.settings.dat');
                const apiKey = await store.get<string>('rawg_api_key');

                if (!apiKey) {
                    setError("API Key da RAWG não configurada.");
                    setLoading(false);
                    return;
                }

                // Chama o Rust
                const result = await invoke<RawgGame[]>("get_trending_games", { apiKey });
                setGames(result);
            } catch (err) {
                console.error(err);
                setError("Erro ao buscar jogos em alta.");
            } finally {
                setLoading(false);
            }
        };

        fetchTrending();
    }, []);

    // Lógica de filtragem

    // Remove jogos que o usuário já tem (comparando nomes normalizados)
    const userGameNames = userGames.map(g => g.name.toLowerCase().replace(/[^a-z0-9]/g, ""));

    const availableGames = games.filter(rawgGame => {
        const rawgName = rawgGame.name.toLowerCase().replace(/[^a-z0-9]/g, "");
        // Se o usuário NÃO tem o jogo, mantemos na lista
        return !userGameNames.includes(rawgName);
    });

    // Filtra por Gênero selecionado na UI
    const filteredGames = availableGames.filter(game => {
        if (selectedGenre === "all") return true;
        return game.genres.some(g => g.name === selectedGenre);
    });

    // Seção de destaques

    // Pega os 5 primeiros para o Hero (Carrossel)
    const heroGames = filteredGames.slice(0, 5);
    const currentHero = heroGames[heroIndex];

    // Restante da lista para o Grid
    const gridGames = filteredGames.slice(5);

    // Lista de gêneros dinâmicos baseados nos resultados
    const allGenres = Array.from(new Set(games.flatMap(g => g.genres.map(genre => genre.name)))).sort();

    // Handlers do Carrossel
    const nextHero = () => setHeroIndex((prev) => (prev + 1) % heroGames.length);
    const prevHero = () => setHeroIndex((prev) => (prev - 1 + heroGames.length) % heroGames.length);

    // Renderização condicional
    if (loading) {
        return (
            <div className="flex-1 flex flex-col items-center justify-center space-y-4">
                <Loader2 className="w-10 h-10 animate-spin text-primary" />
                <p className="text-muted-foreground">Consultando tendências mundiais...</p>
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
                <Button onClick={() => onChangeTab("settings")}>
                    Configurar API Key
                </Button>
            </div>
        );
    }

    if (heroGames.length === 0) {
        return (
            <div className="flex-1 flex flex-col items-center justify-center">
                <p className="text-muted-foreground">Nenhum jogo novo encontrado.</p>
            </div>
        )
    }

    return (
        <div className="flex-1 overflow-y-auto">
            {/* HERO SECTION */}
            <div className="relative h-[500px] bg-background">
                {/* Imagem de Fundo Borrada */}
                <div
                    className="absolute inset-0 bg-cover bg-center transition-all duration-700"
                    style={{
                        backgroundImage: `url(${currentHero.background_image})`,
                        filter: "blur(20px) brightness(0.25)",
                    }}
                />

                {/* Degradê inferior para mesclar com o corpo */}
                <div className="absolute inset-0 bg-gradient-to-t from-background via-transparent to-transparent" />

                <div className="relative h-full flex items-center px-8 max-w-7xl mx-auto z-10">
                    {/* Botões de Navegação */}
                    {heroGames.length > 1 && (
                        <>
                            <button onClick={prevHero} className="absolute left-4 p-2 bg-black/40 hover:bg-black/60 rounded-full text-white backdrop-blur-sm transition">
                                <ChevronLeft size={24} />
                            </button>
                            <button onClick={nextHero} className="absolute right-4 p-2 bg-black/40 hover:bg-black/60 rounded-full text-white backdrop-blur-sm transition">
                                <ChevronRight size={24} />
                            </button>
                        </>
                    )}

                    {/* Conteúdo do Hero */}
                    <div className="flex flex-col md:flex-row items-center gap-8 w-full animate-in fade-in duration-500">
                        <img
                            src={currentHero.background_image || ""}
                            alt={currentHero.name}
                            className="w-64 md:w-80 aspect-[3/4] object-cover rounded-lg shadow-2xl border border-white/10"
                        />

                        <div className="flex-1 space-y-4 text-center md:text-left">
                            <div className="inline-flex items-center gap-2 px-3 py-1 bg-orange-500/20 text-orange-400 rounded-full text-sm font-medium border border-orange-500/20">
                                <Flame size={16} />
                                EM ALTA
                            </div>

                            <h1 className="text-4xl md:text-5xl font-bold text-white leading-tight">
                                {currentHero.name}
                            </h1>

                            <div className="flex flex-wrap gap-2 justify-center md:justify-start">
                                {currentHero.genres.map(g => (
                                    <span key={g.name} className="px-3 py-1 bg-white/10 hover:bg-white/20 rounded-full text-xs text-white transition cursor-default">
                                        {g.name}
                                    </span>
                                ))}
                            </div>

                            <div className="flex items-center gap-6 justify-center md:justify-start pt-2">
                                <div className="flex items-center gap-2">
                                    <Star className="text-yellow-400 fill-yellow-400" size={24} />
                                    <div>
                                        <span className="text-2xl font-bold text-white">{currentHero.rating}</span>
                                        <span className="text-white/50 text-sm ml-1">/ 5.0</span>
                                    </div>
                                </div>

                                {currentHero.released && (
                                    <div className="text-white/80 text-sm">
                                        Lançamento:<br/>
                                        <span className="font-semibold text-white">{new Date(currentHero.released).toLocaleDateString()}</span>
                                    </div>
                                )}
                            </div>
                        </div>
                    </div>
                </div>
            </div>

            {/* BARRA DE FILTROS */}
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
                        {allGenres.map(g => (
                            <option key={g} value={g}>{g}</option>
                        ))}
                    </select>

                    <div className="ml-auto text-xs text-muted-foreground hidden sm:block">
                        Mostrando {filteredGames.length} sugestões baseadas no que você não tem.
                    </div>
                </div>
            </div>

            {/* GRID DE SUGESTÕES */}
            <div className="p-8 max-w-7xl mx-auto">
                <div className="flex items-center gap-2 mb-6">
                    <TrendingUp className="text-primary" />
                    <h2 className="text-2xl font-bold">Mais Sugestões</h2>
                </div>

                <div className="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-4 xl:grid-cols-5 gap-6">
                    {gridGames.map((game) => (
                        <div key={game.id} className="group relative bg-card rounded-xl overflow-hidden border border-border hover:shadow-xl transition-all hover:-translate-y-1">
                            <div className="aspect-[16/9] overflow-hidden">
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

                                {/* Overlay Hover */}
                                <div className="absolute inset-0 bg-black/60 opacity-0 group-hover:opacity-100 transition-opacity flex items-center justify-center gap-2">
                                    {/* Botão Fake de Wishlist por enquanto */}
                                    <Button size="sm" variant="secondary" className="h-8 text-xs">
                                        <Heart size={14} className="mr-1"/> Desejos
                                    </Button>
                                </div>
                            </div>

                            <div className="p-3">
                                <div className="flex justify-between items-start gap-2 mb-1">
                                    <h3 className="font-semibold text-sm line-clamp-1" title={game.name}>{game.name}</h3>
                                    <div className="flex items-center gap-1 shrink-0 bg-yellow-500/10 px-1.5 py-0.5 rounded text-[10px] text-yellow-500 font-bold">
                                        <Star size={10} className="fill-yellow-500" />
                                        {game.rating}
                                    </div>
                                </div>
                                <div className="text-xs text-muted-foreground line-clamp-1">
                                    {game.genres.map(g => g.name).join(", ")}
                                </div>
                            </div>
                        </div>
                    ))}
                </div>
            </div>
        </div>
    );
}
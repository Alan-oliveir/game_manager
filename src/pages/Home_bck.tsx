import { Game } from "../types";
import { Clock, Trophy, Gamepad2, Dices, PlayCircle } from "lucide-react";
import { Button } from "@/components/ui/button";
import { useMemo, useState } from "react";

interface HomeProps {
    games: Game[];
    onGameClick: (game: Game) => void;
    onChangeTab: (tab: string) => void;
}

export default function Home({ games, onGameClick, onChangeTab }: HomeProps) {
    // Estado para forçar recalculo do jogo aleatório
    const [randomSeed, setRandomSeed] = useState(0);

    // Estatísticas Gerais
    const totalGames = games.length;
    const totalFavorites = games.filter(g => g.favorite).length;

    // Soma total de horas
    const totalHours = games.reduce((acc, game) => acc + game.playtime, 0);

    // Conversão amigável (ex: 120h -> 5 dias)
    const timeDisplay = totalHours > 24
        ? `${(totalHours / 24).toFixed(1)} dias`
        : `${totalHours}h`;

    // Top 5 Mais Jogados (Ordenação)
    const mostPlayed = useMemo(() =>
            [...games]
                .sort((a, b) => b.playtime - a.playtime)
                .slice(0, 5),
        [games]
    );

    // Jogo Aleatório (Recomendação Rápida) - usa useMemo para performance
    const randomGame = useMemo(() => {
        if (games.length === 0) return null;
        return games[Math.floor(Math.random() * games.length)];
    }, [games, randomSeed]);

    if (games.length === 0) {
        return (
            <div className="flex-1 flex flex-col items-center justify-center p-8 text-center">
                <h2 className="text-2xl font-bold mb-2">Bem-vindo ao Game Manager!</h2>
                <p className="text-muted-foreground mb-6">Sua biblioteca está vazia. Comece importando seus jogos.</p>
                <Button onClick={() => onChangeTab("settings")}>
                    Ir para Configurações
                </Button>
            </div>
        );
    }

    return (
        <div className="flex-1 p-8 overflow-y-auto space-y-8">
            <div>
                <h2 className="text-3xl font-bold">Visão Geral</h2>
                <p className="text-muted-foreground">Resumo da sua coleção pessoal.</p>
            </div>

            {/* --- SEÇÃO 1: CARDS DE ESTATÍSTICAS --- */}
            <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
                {/* Card 1: Total */}
                <div className="bg-card border border-border p-6 rounded-xl flex items-center gap-4 shadow-sm">
                    <div className="p-3 bg-blue-500/10 rounded-full text-blue-500">
                        <Gamepad2 size={24} />
                    </div>
                    <div>
                        <p className="text-sm text-muted-foreground font-medium">Total de Jogos</p>
                        <h3 className="text-2xl font-bold">{totalGames}</h3>
                    </div>
                </div>

                {/* Card 2: Tempo Jogado */}
                <div className="bg-card border border-border p-6 rounded-xl flex items-center gap-4 shadow-sm">
                    <div className="p-3 bg-amber-500/10 rounded-full text-amber-500">
                        <Clock size={24} />
                    </div>
                    <div>
                        <p className="text-sm text-muted-foreground font-medium">Tempo de Vida</p>
                        <h3 className="text-2xl font-bold">{timeDisplay}</h3>
                        <p className="text-xs text-muted-foreground">jogando videogame</p>
                    </div>
                </div>

                {/* Card 3: Favoritos */}
                <div className="bg-card border border-border p-6 rounded-xl flex items-center gap-4 shadow-sm">
                    <div className="p-3 bg-red-500/10 rounded-full text-red-500">
                        <Trophy size={24} />
                    </div>
                    <div>
                        <p className="text-sm text-muted-foreground font-medium">Favoritos</p>
                        <h3 className="text-2xl font-bold">{totalFavorites}</h3>
                    </div>
                </div>
            </div>

            <div className="grid grid-cols-1 lg:grid-cols-3 gap-8">
                {/* --- SEÇÃO 2: RECOMENDAÇÃO ALEATÓRIA --- */}
                <div className="lg:col-span-1 space-y-4">
                    <h3 className="text-xl font-semibold flex items-center gap-2">
                        <Dices className="text-purple-500" />
                        O que jogar hoje?
                    </h3>

                    {randomGame && (
                        <div
                            className="group relative aspect-[3/4] rounded-xl overflow-hidden cursor-pointer shadow-lg border border-border"
                            onClick={() => onGameClick(randomGame)}
                        >
                            {randomGame.cover_url ? (
                                <img
                                    src={randomGame.cover_url}
                                    className="w-full h-full object-cover transition-transform duration-700 group-hover:scale-105"
                                    alt={randomGame.name}
                                />
                            ) : (
                                <div className="w-full h-full bg-muted flex items-center justify-center">
                                    <span className="text-xl font-bold">{randomGame.name}</span>
                                </div>
                            )}

                            <div className="absolute inset-0 bg-gradient-to-t from-black/90 via-transparent to-transparent flex flex-col justify-end p-6">
                        <span className="text-purple-300 text-xs font-bold uppercase tracking-wider mb-1">
                            Sugestão Aleatória
                        </span>
                                <h4 className="text-white font-bold text-2xl leading-tight mb-2">
                                    {randomGame.name}
                                </h4>
                                <div className="flex items-center gap-2 text-white/80 text-sm">
                                    <PlayCircle size={16} />
                                    Clique para ver detalhes
                                </div>
                            </div>
                        </div>
                    )}
                    <Button
                        variant="outline"
                        className="w-full"
                        onClick={() => setRandomSeed(prev => prev + 1)}
                    >
                        <Dices size={16} className="mr-2"/>
                        Rodar Dados Novamente
                    </Button>
                </div>

                {/* --- SEÇÃO 3: TOP 5 --- */}
                <div className="lg:col-span-2 space-y-4">
                    <h3 className="text-xl font-semibold flex items-center gap-2">
                        <Trophy className="text-yellow-500" />
                        Mais Jogados
                    </h3>

                    <div className="space-y-3">
                        {mostPlayed.map((game, index) => (
                            <div
                                key={game.id}
                                className="flex items-center gap-4 p-3 rounded-lg bg-card/50 hover:bg-accent/50 border border-transparent hover:border-border transition-all cursor-pointer group"
                                onClick={() => onGameClick(game)}
                            >
                                <div className="w-8 h-8 flex items-center justify-center font-bold text-muted-foreground bg-muted rounded-full text-sm">
                                    #{index + 1}
                                </div>

                                <div className="h-12 w-8 bg-muted rounded overflow-hidden shrink-0">
                                    {game.cover_url && <img src={game.cover_url} alt={game.name} className="w-full h-full object-cover" />}
                                </div>

                                <div className="flex-1 min-w-0">
                                    <h4 className="font-medium truncate group-hover:text-primary transition-colors">
                                        {game.name}
                                    </h4>
                                    <p className="text-xs text-muted-foreground">
                                        {game.platform || "Manual"}
                                    </p>
                                </div>

                                <div className="text-right px-2">
                            <span className="font-bold text-foreground block">
                                {game.playtime}h
                            </span>
                                </div>
                            </div>
                        ))}
                    </div>
                </div>
            </div>
        </div>
    );
}
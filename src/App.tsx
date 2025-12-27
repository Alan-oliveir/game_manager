import {useEffect, useState} from "react";
import {invoke} from "@tauri-apps/api/core";
import {Game, RawgGame} from "./types";
import Sidebar from "./components/Sidebar";
import Header from "./components/Header";
import AddGameModal from "./components/AddGameModal";
import Library from "./pages/Library";
import Favorites from "./pages/Favorites";
import Settings from "./pages/Settings";
import Home from "./pages/Home";
import Trending from "./pages/Trending.tsx";

function App() {
    // Estados principais
    const [games, setGames] = useState<Game[]>([]);
    const [activeSection, setActiveSection] = useState("home");
    const [isModalOpen, setIsModalOpen] = useState(false);
    const [gameToEdit, setGameToEdit] = useState<Game | null>(null);
    const [searchTerm, setSearchTerm] = useState("");
    const [trendingCache, setTrendingCache] = useState<RawgGame[]>([]);

    // Consulta os jogos na base de dados
    const refreshGames = async () => {
        try {
            const result = await invoke<Game[]>("get_games");
            setGames(result);
        } catch (error) {
            console.error("Erro ao buscar jogos:", error);
        }
    };

    useEffect(() => {
        invoke("init_db")
            .finally(() => refreshGames())
            .catch(console.error);
    }, []);

    // Ações de manipulação de jogos
    const handleSaveGame = async (gameData: Partial<Game>) => {
        try {
            if (gameToEdit) {
                await invoke("update_game", {
                    id: gameToEdit.id,
                    name: gameData.name,
                    genre: gameData.genre,
                    platform: gameData.platform,
                    coverUrl: gameData.cover_url || null,
                    playtime: gameData.playtime || 0,
                    rating: gameData.rating || null,
                });
            } else {
                await invoke("add_game", {
                    id: crypto.randomUUID(),
                    name: gameData.name,
                    genre: gameData.genre || "Desconhecido",
                    platform: gameData.platform || "Manual",
                    coverUrl: gameData.cover_url || null,
                    playtime: gameData.playtime || 0,
                    rating: gameData.rating || null,
                });
            }
            await refreshGames();
            setIsModalOpen(false);
            setGameToEdit(null);
        } catch (e) {
            console.error(e);
            alert("Erro ao salvar: " + e);
        }
    };

    const handleDeleteGame = async (id: string) => {
        if (!confirm("Tem certeza que deseja excluir este jogo?")) return;
        try {
            await invoke("delete_game", {id});
            await refreshGames();
        } catch (e) {
            console.error(e);
            alert("Erro ao excluir: " + e);
        }
    };

    const handleToggleFavorite = async (id: string) => {
        try {
            await invoke("toggle_favorite", {id});
            await refreshGames();
        } catch (e) {
            console.error(e);
        }
    };

    const handleGameClick = (game: Game) => {
        console.log("Clicou no jogo:", game.name);
    };

    const handleEditClick = (game: Game) => {
        setGameToEdit(game);
        setIsModalOpen(true);
    };

    const handleAddClick = () => {
        setGameToEdit(null);
        setIsModalOpen(true);
    };

    // Agrupa as ações para passar como props
    const gameActions = {
        onToggleFavorite: handleToggleFavorite,
        onGameClick: handleGameClick,
        onDeleteGame: handleDeleteGame,
        onEditGame: handleEditClick,
    };

    // Renderiza o conteúdo baseado na seção ativa
    const renderContent = () => {
        switch (activeSection) {
            case "home":
                return (
                    <Home
                        games={games}
                    />
                );
            case "library":
                return (
                    <Library
                        games={games}
                        searchTerm={searchTerm}
                        {...gameActions}
                    />
                );
            case "favorites":
                return (
                    <Favorites
                        games={games}
                        searchTerm={searchTerm}
                        {...gameActions}
                    />
                );
            case "trending":
                return (
                    <Trending
                        userGames={games}
                        onChangeTab={setActiveSection}
                        cachedGames={trendingCache}
                        setCachedGames={setTrendingCache}
                    />
                );
            case "settings":
                return <Settings onLibraryUpdate={refreshGames}/>;
            default:
                return (
                    <div className="flex-1 flex flex-col items-center justify-center text-muted-foreground opacity-50">
                        <div className="text-6xl mb-4">🚧</div>
                        <p className="text-xl font-medium">
                            Seção "{activeSection}" em desenvolvimento
                        </p>
                    </div>
                );
        }
    };

    return (
        <div className="flex h-screen bg-background text-foreground overflow-hidden">
            <Sidebar
                activeSection={activeSection}
                onSectionChange={setActiveSection}
                games={games}
            />

            <main className="flex-1 flex flex-col min-w-0">
                <Header
                    onAddGame={handleAddClick}
                    searchTerm={searchTerm}
                    onSearchChange={setSearchTerm}
                />

                {renderContent()}
            </main>

            <AddGameModal
                isOpen={isModalOpen}
                onClose={() => setIsModalOpen(false)}
                onSave={handleSaveGame}
                gameToEdit={gameToEdit}
            />
        </div>
    );
}

export default App;
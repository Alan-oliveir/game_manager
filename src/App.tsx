import { useMemo, useState } from "react";
import { Game, RawgGame, UserProfile } from "./types";
import { useLibraries } from "./hooks/useLibraries.ts";
import { toast, Toaster } from "sonner";
import { useDebounce } from "./hooks/useDebounce";

// Componentes
import Sidebar from "./components/Sidebar";
import Header from "./components/Header";
import AddGameModal from "./components/AddGameModal";
import GameDetailsModal from "./components/GameDetailsModal.tsx";

// Páginas
import Libraries from "./pages/Libraries.tsx";
import Favorites from "./pages/Favorites";
import Settings from "./pages/Settings";
import Home from "./pages/Home";
import Trending from "./pages/Trending";
import Wishlist from "./pages/Wishlist";
import Playlist from "./pages/Playlist";

function App() {
  // Hook Principal de Dados
  const { games, refreshGames, saveGame, removeGame, toggleFavorite } =
    useLibraries();

  // Estado de Busca com Debounce
  const [searchTerm, setSearchTerm] = useState("");
  const debouncedSearchTerm = useDebounce(searchTerm, 300);

  // Estado de UI (Navegação e Modais)

  // Navegação Simples
  const [activeSection, setActiveSection] = useState("home");

  // Modal Add Game State
  const [isModalOpen, setIsModalOpen] = useState(false);
  const [gameToEdit, setGameToEdit] = useState<Game | null>(null);

  // Estado para detalhes do jogo selecionado
  const [selectedGameId, setSelectedGameId] = useState<string | null>(null);

  // Jogo selecionado para o modal de detalhes
  const selectedGame = useMemo(
    () => games.find((g) => g.id === selectedGameId) || null,
    [games, selectedGameId]
  );

  // Cache do Trending
  const [trendingCache, setTrendingCache] = useState<RawgGame[]>([]);
  const [trendingKey, setTrendingKey] = useState(0);

  // Cache do Perfil do Usuário para recomendações
  const [profileCache, setProfileCache] = useState<UserProfile | null>(null);

  // Handlers de UI e Ações

  const handleSettingsUpdate = () => {
    refreshGames();
    setTrendingCache([]);
    setTrendingKey((k) => k + 1);
  };

  const openAddModal = () => {
    setGameToEdit(null);
    setIsModalOpen(true);
  };

  const openEditModal = (game: Game) => {
    setGameToEdit(game);
    setIsModalOpen(true);
  };

  const handleSaveGameWrapper = async (data: Partial<Game>) => {
    try {
      await saveGame(data, gameToEdit?.id);
      setIsModalOpen(false);
      setGameToEdit(null);
    } catch (e) {
      toast.error("Erro ao salvar: " + e);
    }
  };

  const handleDeleteWrapper = async (id: string) => {
    if (confirm("Tem certeza que deseja excluir?")) {
      await removeGame(id);
    }
  };

  const handleGameClick = (game: Game) => {
    setSelectedGameId(game.id);
  };

  const handleSwitchGame = (id: string) => {
    setSelectedGameId(id);
  };

  const closeDetails = () => {
    setSelectedGameId(null);
  };

  // Props comuns passadas para as listas de jogos
  const commonGameActions = {
    onToggleFavorite: toggleFavorite,
    onGameClick: handleGameClick,
    onDeleteGame: handleDeleteWrapper,
    onEditGame: openEditModal,
  };

  // Roteamento Simples

  const renderContent = () => {
    switch (activeSection) {
      case "home":
        return (
          <Home
            onChangeTab={setActiveSection}
            games={games}
            trendingCache={trendingCache}
            setTrendingCache={setTrendingCache}
            profileCache={profileCache}
            setProfileCache={setProfileCache}
          />
        );
      case "libraries":
        return (
          <Libraries
            games={games}
            searchTerm={debouncedSearchTerm}
            {...commonGameActions}
          />
        );
      case "favorites":
        return (
          <Favorites
            games={games}
            searchTerm={debouncedSearchTerm}
            {...commonGameActions}
          />
        );
      case "playlist":
        return (
          <Playlist
            allGames={games}
            onGameClick={handleGameClick}
            profileCache={profileCache}
          />
        );
      case "trending":
        return (
          <Trending
            key={trendingKey}
            userGames={games}
            onChangeTab={setActiveSection}
            cachedGames={trendingCache}
            setCachedGames={setTrendingCache}
          />
        );
      case "wishlist":
        return <Wishlist />;
      case "settings":
        return <Settings onLibraryUpdate={handleSettingsUpdate} />;
      default:
        return <div className="p-10 text-center">Página não encontrada</div>;
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
          onAddGame={openAddModal}
          searchTerm={searchTerm}
          onSearchChange={setSearchTerm}
        />
        {renderContent()}
      </main>

      <AddGameModal
        isOpen={isModalOpen}
        onClose={() => setIsModalOpen(false)}
        onSave={handleSaveGameWrapper}
        gameToEdit={gameToEdit}
      />

      <GameDetailsModal
        isOpen={!!selectedGameId}
        onClose={closeDetails}
        game={selectedGame}
        allGames={games}
        onSwitchGame={handleSwitchGame}
      />

      <Toaster />
    </div>
  );
}

export default App;

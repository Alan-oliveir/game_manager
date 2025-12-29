import { useState } from "react";
import { RawgGame, Game } from "./types";
import { useLibrary } from "./hooks/useLibrary";

// Componentes
import Sidebar from "./components/Sidebar";
import Header from "./components/Header";
import AddGameModal from "./components/AddGameModal";
import GameDetailsModal from "./components/GameDetailsModal.tsx";

// Páginas
import Library from "./pages/Library";
import Favorites from "./pages/Favorites";
import Settings from "./pages/Settings";
import Home from "./pages/Home";
import Trending from "./pages/Trending";
import Wishlist from "./pages/Wishlist";

function App() {
  // Hook Principal de Dados
  const { games, refreshGames, saveGame, removeGame, toggleFavorite } =
    useLibrary();

  // Estado de UI (Navegação e Modais)
  const [activeSection, setActiveSection] = useState("home");
  const [searchTerm, setSearchTerm] = useState("");

  // Modal State
  const [isModalOpen, setIsModalOpen] = useState(false);
  const [gameToEdit, setGameToEdit] = useState<Game | null>(null);

  // Estado para detalhes do jogo selecionado
  const [selectedGameId, setSelectedGameId] = useState<string | null>(null);

  // Jogo selecionado para o modal de detalhes
  const selectedGame = games.find(g => g.id === selectedGameId) || null;

  // Cache do Trending
  const [trendingCache, setTrendingCache] = useState<RawgGame[]>([]);
  const [trendingKey, setTrendingKey] = useState(0);

  // Handlers de UI

  const handleSettingsUpdate = () => {
    refreshGames(); // Atualiza a lista após importação
    setTrendingCache([]); // Limpa cache para o usuário ver novidades
    setTrendingKey((k) => k + 1); // Força refresh da tela Trending
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
      await saveGame(data, gameToEdit?.id); // Chama o hook
      setIsModalOpen(false);
      setGameToEdit(null);
    } catch (e) {
      alert("Erro ao salvar: " + e);
    }
  };

  const handleDeleteWrapper = async (id: string) => {
    if (confirm("Tem certeza que deseja excluir?")) {
      await removeGame(id);
    }
  };

  const handleGameClick = (game: Game) => {
    setSelectedGameId(game.id); // Abre o modal de detalhes
  };

  const handleSwitchGame = (id: string) => {
    setSelectedGameId(id); // Troca o jogo visualizado dentro do modal
  };

  const closeDetails = () => setSelectedGameId(null);

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
        return <Home onChangeTab={setActiveSection} />;
      case "library":
        return (
          <Library
            games={games}
            searchTerm={searchTerm}
            {...commonGameActions}
          />
        );
      case "favorites":
        return (
          <Favorites
            games={games}
            searchTerm={searchTerm}
            {...commonGameActions}
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
    </div>
  );
}

export default App;

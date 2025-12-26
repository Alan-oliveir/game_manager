import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Game } from "./types";

import Sidebar from "./components/Sidebar";
import Header from "./components/Header";
import GameGrid from "./components/GameGrid";
import AddGameModal from "./components/AddGameModal";

function App() {
  // Estado dos dados
  const [games, setGames] = useState<Game[]>([]);
  const [activeSection, setActiveSection] = useState("library"); // Controla qual aba está ativa
  const [isModalOpen, setIsModalOpen] = useState(false); // Estado do Modal de Adicionar Jogo
  const [gameToEdit, setGameToEdit] = useState<Game | null>(null); // Jogo sendo editado
  const [searchTerm, setSearchTerm] = useState(""); // Termo de busca

  // Recarrega a lista do banco
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
      .then(() => refreshGames())
      .catch(console.error);
  }, []);

  // Função para obter a lista de jogos a serem exibidos com base na seção ativa e no termo de busca
  const getDisplayedGames = () => {
    let result = games;

    // Filtro de Seção: Se estiver na aba Favoritos, filtra apenas os favoritados
    if (activeSection === "favorites") {
      result = result.filter((game) => game.favorite);
    }

    // Filtro de Busca: Aplica o texto da barra de pesquisa
    if (searchTerm) {
      const term = searchTerm.toLowerCase();
      result = result.filter(
        (game) =>
          game.name.toLowerCase().includes(term) ||
          (game.genre && game.genre.toLowerCase().includes(term)) ||
          (game.platform && game.platform.toLowerCase().includes(term))
      );
    }

    return result;
  };

  // Chama a função a cada render para ter a lista atualizada
  const displayedGames = getDisplayedGames();

  // Função para editar/adicionar um novo jogo
  const handleSaveGame = async (gameData: Partial<Game>) => {
    try {
      if (gameToEdit) {
        // Editar jogo existente
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
        // Adicionar novo jogo
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

  // Função para favoritar/desfavoritar um jogo
  const handleToggleFavorite = async (id: string) => {
    try {
      await invoke("toggle_favorite", { id });
      await refreshGames();
    } catch (e) {
      console.error(e);
    }
  };

  // Função para exibir detalhes do jogo
  const handleGameClick = (game: Game) => {
    console.log("Clicou no jogo:", game.name);
    // Futuro: Abrir detalhes do jogo
  };

  // Função para excluir um jogo
  const handleDeleteGame = async (id: string) => {
    if (!confirm("Tem certeza que deseja excluir este jogo?")) return;

    try {
      await invoke("delete_game", { id });
      await refreshGames();
    } catch (e) {
      console.error(e);
      alert("Erro ao excluir: " + e);
    }
  };

  // Função chamada ao clicar em "Editar" no card
  const handleEditClick = (game: Game) => {
    setGameToEdit(game);
    setIsModalOpen(true);
  };

  // Função para abrir modal limpo (botão Adicionar)
  const handleAddClick = () => {
    setGameToEdit(null);
    setIsModalOpen(true);
  };

  return (
    <div className="flex h-screen bg-background text-foreground overflow-hidden">
      <Sidebar
        activeSection={activeSection}
        onSectionChange={(section) => {
          setActiveSection(section);
        }}
      />

      <main className="flex-1 flex flex-col min-w-0">
        <Header
          onAddGame={handleAddClick}
          searchTerm={searchTerm}
          onSearchChange={setSearchTerm}
        />

        {/* Caso 1: Seções que usam o Grid (Biblioteca e Favoritos) */}
        {activeSection === "library" || activeSection === "favorites" ? (
          <GameGrid
            games={displayedGames}
            onToggleFavorite={handleToggleFavorite}
            onGameClick={handleGameClick}
            onDeleteGame={handleDeleteGame}
            onEditGame={handleEditClick}
          />
        ) : activeSection === "settings" ? (
          /* Caso 2: Página de Configurações (Placeholder simples por enquanto) */
          <div className="flex-1 p-8">
            <h2 className="text-2xl font-bold mb-4">Configurações</h2>
            <div className="p-4 border rounded-lg bg-card max-w-md">
              <h3 className="font-semibold mb-2">Dados</h3>
              <p className="text-sm text-muted-foreground mb-4">
                Gerencie seus dados locais.
              </p>
              <button
                onClick={() => alert("Funcionalidade de exportar em breve!")}
                className="px-4 py-2 bg-secondary text-secondary-foreground rounded-md text-sm font-medium"
              >
                Exportar Backup (JSON)
              </button>
            </div>
          </div>
        ) : (
          /* Caso 3: Outras páginas (Home, Trending) */
          <div className="flex-1 flex flex-col items-center justify-center text-muted-foreground opacity-50">
            <div className="text-6xl mb-4">🚧</div>
            <p className="text-xl font-medium">
              Seção "{activeSection}" em desenvolvimento
            </p>
          </div>
        )}
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

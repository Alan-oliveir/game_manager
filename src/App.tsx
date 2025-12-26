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

  // Função para recarregar a lista do banco
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

  // --- Ações do Usuário ---

  // Função para editar/adicionar um novo jogo
  const handleSaveGame = async (gameData: Partial<Game>) => {
    try {
      if (gameToEdit) {
        // Edição
        await invoke("update_game", {
          id: gameToEdit.id,
          name: gameData.name,
          genre: gameData.genre,
          platform: gameData.platform,
          coverUrl: gameData.cover_url || null, // Lembra do camelCase!
          playtime: gameData.playtime || 0,
          rating: gameData.rating || null,
        });
      } else {
        // Criação
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
      // Chama o Rust para mudar no banco
      await invoke("toggle_favorite", { id });
      // Recarrega a lista para atualizar o ícone de coração
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
    // Confirmação nativa simples
    if (!confirm("Tem certeza que deseja excluir este jogo?")) return;

    try {
      await invoke("delete_game", { id });
      await refreshGames(); // Atualiza a lista
    } catch (e) {
      console.error(e);
      alert("Erro ao excluir: " + e);
    }
  };

  // Função chamada ao clicar em "Editar" no card
  const handleEditClick = (game: Game) => {
    setGameToEdit(game); // Define quem vamos editar
    setIsModalOpen(true); // Abre o modal
  };

  // Função para abrir modal limpo (botão Adicionar)
  const handleAddClick = () => {
    setGameToEdit(null); // Garante que não tem lixo de edição
    setIsModalOpen(true);
  };

  return (
    <div className="flex h-screen bg-background text-foreground overflow-hidden">
      {/* Sidebar */}
      <Sidebar
        activeSection={activeSection}
        onSectionChange={setActiveSection}
      />

      {/* Área Principal */}
      <main className="flex-1 flex flex-col min-w-0">
        {/* Header no topo */}
        <Header onAddGame={handleAddClick} />

        {/* Conteúdo Variável */}
        {activeSection === "library" ? (
          <GameGrid
            games={games}
            onToggleFavorite={handleToggleFavorite}
            onGameClick={handleGameClick}
            onDeleteGame={handleDeleteGame}
            onEditGame={handleEditClick}
          />
        ) : (
          // Placeholder para outras abas (Home, Configurações, etc)
          <div className="flex-1 flex items-center justify-center text-muted-foreground">
            <p>Seção {activeSection} em construção...</p>
          </div>
        )}
      </main>

      {/* Modal de Adicionar Jogo */}
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

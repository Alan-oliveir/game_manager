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

  // Função para recarregar a lista do banco
  const refreshGames = async () => {
    try {
      const result = await invoke<Game[]>("get_games");
      setGames(result);
    } catch (error) {
      console.error("Erro ao buscar jogos:", error);
    }
  };

  // Inicialização
  useEffect(() => {
    invoke("init_db")
      .then(() => refreshGames())
      .catch(console.error);
  }, []);

  // --- Ações do Usuário ---

  const handleAddGame = async (name: string, coverUrl: string) => {
    try {
      await invoke("add_game", {
        id: crypto.randomUUID(),
        name: name,
        genre: "Desconhecido",
        platform: "Manual",
        coverUrl: coverUrl || null,
      });
      await refreshGames();
    } catch (e) {
      console.error(e);
      alert("Erro ao salvar: " + e);
    }
  };

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

  const handleGameClick = (game: Game) => {
    console.log("Clicou no jogo:", game.name);
    // Futuro: Abrir detalhes do jogo
  };

  return (
    <div className="flex h-screen bg-background text-foreground overflow-hidden">
      {/* 1. Sidebar Fixa */}
      <Sidebar
        activeSection={activeSection}
        onSectionChange={setActiveSection}
      />

      {/* 2. Área Principal */}
      <main className="flex-1 flex flex-col min-w-0">
        {/* Header no topo */}
        <Header onAddGame={() => setIsModalOpen(true)} />

        {/* Conteúdo Variável */}
        {activeSection === "library" ? (
          <GameGrid
            games={games}
            onToggleFavorite={handleToggleFavorite}
            onGameClick={handleGameClick}
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
        onSave={handleAddGame}
      />
    </div>
  );
}

export default App;

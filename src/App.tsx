import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Game } from "./types";

import Sidebar from "./components/Sidebar";
import Header from "./components/Header";
import GameGrid from "./components/GameGrid";

function App() {
  // Estado dos dados
  const [games, setGames] = useState<Game[]>([]);
  const [activeSection, setActiveSection] = useState("library"); // Controla qual aba está ativa

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

  const handleAddGame = async () => {
    try {
      // Aqui futuramente abriremos um Modal
      // Por enquanto, cria um jogo random para testar
      await invoke("add_game", {
        id: crypto.randomUUID(),
        name: `Novo Jogo ${Math.floor(Math.random() * 100)}`,
        genre: "Aventura",
        platform: "Steam",
      });
      await refreshGames();
    } catch (e) {
      console.error(e);
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
        <Header onAddGame={handleAddGame} />

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
    </div>
  );
}

export default App;

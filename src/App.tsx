import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";

function App() {
    const [status, setStatus] = useState("Aguardando...");

    // Ao abrir o app, iniciamos o banco
    useEffect(() => {
        invoke("init_db")
            .then((msg) => setStatus(msg as string))
            .catch((err) => setStatus("Erro: " + err));
    }, []);

    const handleSaveGame = async () => {
        try {
            // Chama a função 'add_game' definida no Rust
            await invoke("add_game", {
                id: crypto.randomUUID(), // Gera ID único no JS
                name: "God of War",
            });
            alert("Jogo salvo no SQLite via Rust!");
        } catch (e) {
            console.error(e);
        }
    };

    return (
        <div className="min-h-screen bg-gradient-to-br from-blue-500 to-purple-600 flex flex-col items-center justify-center p-8">
            <h1 className="text-5xl font-bold text-white mb-4">Game Manager</h1>
            <p className="text-white text-xl mb-6">Status do Banco: {status}</p>
            <button
                onClick={handleSaveGame}
                className="bg-white text-purple-600 px-6 py-3 rounded-lg font-semibold hover:bg-gray-100 transition-colors"
            >
                Criar Jogo de Teste
            </button>
        </div>
    );
}

export default App;
import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/tauri";

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
        <div className="container">
            <h1>Game Manager</h1>
            <p>Status do Banco: {status}</p>
            <button onClick={handleSaveGame}>Criar Jogo de Teste</button>
        </div>
    );
}

export default App;
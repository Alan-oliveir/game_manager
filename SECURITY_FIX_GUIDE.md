# 🔒 Guia de Correção de Segurança - API Key Storage

## ⚠️ PROBLEMA CRÍTICO IDENTIFICADO

O código atual armazena a Steam API Key no `localStorage`, que é **inseguro** pois:
- Pode ser acessado por qualquer script JavaScript
- Extensões do navegador podem ler o valor
- Vulnerável a ataques XSS
- API Keys são credenciais sensíveis que não devem ser expostas

## ✅ SOLUÇÃO RECOMENDADA: Tauri Store Plugin

### Passo 1: Instalar o Plugin

```bash
# No terminal do projeto:
npm install @tauri-apps/plugin-store
```

```bash
# Adicionar ao Cargo.toml do Rust:
cd src-tauri
cargo add tauri-plugin-store
```

### Passo 2: Configurar o Plugin no Rust

**Arquivo:** `src-tauri/src/lib.rs`

```rust
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let conn = Connection::open("library.db").expect("Erro ao abrir banco");

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_store::Builder::default().build()) // <- ADICIONAR ESTA LINHA
        .manage(AppState {
            db: Mutex::new(conn),
        })
        .invoke_handler(tauri::generate_handler![
            init_db,
            add_game,
            get_games,
            toggle_favorite,
            delete_game,
            update_game,
            import_steam_library
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

### Passo 3: Atualizar capabilities (Permissões)

**Arquivo:** `src-tauri/capabilities/default.json`

Adicionar a permissão do store:

```json
{
  "$schema": "../gen/schemas/desktop-schema.json",
  "identifier": "default",
  "description": "Capability for the main window",
  "windows": ["main"],
  "permissions": [
    "core:default",
    "opener:default",
    "store:default",
    "store:allow-get",
    "store:allow-set",
    "store:allow-save"
  ]
}
```

### Passo 4: Substituir o código em Settings.tsx

**Arquivo:** `src/pages/Settings.tsx`

**ANTES (INSEGURO):**
```typescript
import {useEffect, useState} from "react";
import {invoke} from "@tauri-apps/api/core";

export default function Settings({onLibraryUpdate}: SettingsProps) {
    const [steamId, setSteamId] = useState("");
    const [apiKey, setApiKey] = useState("");

    useEffect(() => {
        const savedId = localStorage.getItem("steam_id");
        const savedKey = localStorage.getItem("steam_api_key");
        if (savedId) setSteamId(savedId);
        if (savedKey) setApiKey(savedKey);
    }, []);

    const handleImport = async () => {
        // ...
        localStorage.setItem("steam_id", steamId);
        localStorage.setItem("steam_api_key", apiKey);
        // ...
    };
}
```

**DEPOIS (SEGURO):**
```typescript
import {useEffect, useState} from "react";
import {invoke} from "@tauri-apps/api/core";
import {Store} from '@tauri-apps/plugin-store';

// Criar instância do store (arquivo será criptografado)
const store = new Store('.settings.dat');

export default function Settings({onLibraryUpdate}: SettingsProps) {
    const [steamId, setSteamId] = useState("");
    const [apiKey, setApiKey] = useState("");
    const [isLoading, setIsLoading] = useState(false);
    const [status, setStatus] = useState<{ type: 'success' | 'error' | null, message: string }>({
        type: null,
        message: ""
    });

    useEffect(() => {
        // Carregar configurações do store seguro
        const loadSettings = async () => {
            try {
                const savedId = await store.get<string>('steam_id');
                const savedKey = await store.get<string>('steam_api_key');
                
                if (savedId) setSteamId(savedId);
                if (savedKey) setApiKey(savedKey);
            } catch (error) {
                console.error("Erro ao carregar configurações:", error);
            }
        };

        loadSettings();
    }, []);

    const handleImport = async () => {
        if (!steamId || !apiKey) {
            setStatus({type: 'error', message: "Preencha o Steam ID e a API Key."});
            return;
        }

        setIsLoading(true);
        setStatus({type: null, message: ""});

        try {
            // Salvar de forma segura
            await store.set('steam_id', steamId.trim());
            await store.set('steam_api_key', apiKey.trim());
            await store.save(); // Persiste no disco de forma criptografada

            const result = await invoke<string>("import_steam_library", {
                steamId: steamId.trim(),
                apiKey: apiKey.trim(),
            });

            setStatus({type: 'success', message: result});
            onLibraryUpdate();

        } catch (error) {
            console.error(error);
            setStatus({type: 'error', message: String(error)});

        } finally {
            setIsLoading(false);
        }
    };

    // ... resto do componente permanece igual
}
```

### Passo 5: Limpar dados antigos (opcional)

Adicionar um botão ou executar uma vez para migrar dados:

```typescript
const migrateFromLocalStorage = async () => {
    const oldId = localStorage.getItem("steam_id");
    const oldKey = localStorage.getItem("steam_api_key");
    
    if (oldId) {
        await store.set('steam_id', oldId);
        localStorage.removeItem("steam_id");
    }
    
    if (oldKey) {
        await store.set('steam_api_key', oldKey);
        localStorage.removeItem("steam_api_key");
    }
    
    await store.save();
    console.log("Migração concluída!");
};
```

## 🔐 Benefícios da Solução

✅ **Criptografia**: Os dados são armazenados de forma criptografada  
✅ **Isolamento**: Não pode ser acessado via JavaScript do browser  
✅ **Persistência**: Dados salvos de forma permanente e segura  
✅ **Cross-platform**: Funciona em Windows, macOS e Linux  
✅ **Controle de Acesso**: Apenas o backend Rust pode acessar  

## 📍 Localização do Arquivo

O arquivo `.settings.dat` será criado automaticamente na pasta de dados do app:
- **Windows**: `%APPDATA%/com.game_manager.app/`
- **macOS**: `~/Library/Application Support/com.game_manager.app/`
- **Linux**: `~/.config/com.game_manager.app/`

## ⚠️ IMPORTANTE

Após implementar esta correção:
1. ✅ Teste a importação da Steam para garantir que funciona
2. ✅ Verifique se os dados persistem após fechar e reabrir o app
3. ✅ Remova manualmente os dados do localStorage do navegador (F12 > Application > Local Storage)
4. ✅ Adicione `.settings.dat` ao `.gitignore` se ainda não estiver

## 📚 Documentação Oficial

- [Tauri Store Plugin](https://v2.tauri.app/plugin/store/)
- [API Reference](https://v2.tauri.app/reference/javascript/store/)

---

**Status:** 🔴 NÃO IMPLEMENTADO (URGENTE)  
**Prioridade:** CRÍTICA  
**Tempo Estimado:** 15-30 minutos  


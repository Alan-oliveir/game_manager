# Correção do Problema de Loop de Reinício

## Problema Identificado

A aplicação estava abrindo e reiniciando continuamente devido ao seguinte erro:
```
Info File src-tauri\library.db-shm changed. Rebuilding application...
```

## Causa Raiz

O banco de dados SQLite (`library.db`) estava sendo criado no diretório `src-tauri/`, que é monitorado pelo Tauri para detectar mudanças no código-fonte. Quando o banco era modificado, o Tauri detectava a mudança e recompilava a aplicação, criando um loop infinito.

## Correções Realizadas

### 1. Movido o banco de dados para o diretório de dados da aplicação

**Arquivo modificado:** `src-tauri/src/lib.rs`

#### Mudanças:

1. **Adicionado import do trait Manager:**
```rust
use tauri::{State, Manager};
```

2. **Modificada a função `run()` para usar o diretório de dados apropriado:**
```rust
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            // Obtém o diretório de dados da aplicação
            let app_data_dir = app.path().app_data_dir()
                .expect("Falha ao obter diretório de dados da aplicação");
            
            // Cria o diretório se não existir
            std::fs::create_dir_all(&app_data_dir)
                .expect("Falha ao criar diretório de dados");
            
            // Caminho completo do banco de dados
            let db_path = app_data_dir.join("library.db");
            
            // Inicializa conexão com arquivo no diretório de dados
            let conn = Connection::open(&db_path)
                .expect(&format!("Erro ao abrir banco em {:?}", db_path));
            
            // Registra o estado da aplicação
            app.manage(AppState {
                db: Mutex::new(conn),
            });
            
            Ok(())
        })
        // ... resto do código
}
```

### 2. Removidos arquivos antigos do banco

Os arquivos `library.db`, `library.db-shm` e `library.db-wal` foram removidos do diretório `src-tauri/`.

## Localização do Novo Banco de Dados

O banco de dados agora será criado no diretório de dados da aplicação:
- **Windows:** `%APPDATA%\com.game-manager.app\library.db`
- **Linux:** `~/.local/share/com.game-manager.app/library.db`
- **macOS:** `~/Library/Application Support/com.game-manager.app/library.db`

## Benefícios

1. **Sem loop de reinício:** O banco não está mais no diretório monitorado pelo Tauri
2. **Melhor organização:** Os dados da aplicação ficam separados do código-fonte
3. **Compatível com builds de produção:** O mesmo caminho funcionará tanto em desenvolvimento quanto em produção
4. **Seguir boas práticas:** Cada aplicação deve armazenar seus dados no diretório apropriado do sistema

## Como Testar

Execute a aplicação normalmente:
```bash
npm run tauri dev
```

A aplicação deve:
1. Abrir normalmente
2. Permanecer aberta sem reiniciar
3. Criar o banco de dados no diretório de dados da aplicação
4. Funcionar normalmente com todas as operações (adicionar, editar, deletar jogos)

## Nota sobre Migração de Dados

Se você tinha jogos cadastrados no banco antigo (`src-tauri/library.db`), eles não serão migrados automaticamente. Você pode:
1. Copiar manualmente o arquivo para o novo local, OU
2. Re-adicionar os jogos na aplicação, OU
3. Importar novamente da Steam (se aplicável)


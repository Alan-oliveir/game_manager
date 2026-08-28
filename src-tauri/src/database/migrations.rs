//! Sistema de migração de schema do banco `games.db`, baseado em `rusqlite_migration`.
//!
//! Regras:
//! - Cada arquivo em `migrations/` é uma etapa numerada e IMUTÁVEL — nunca edite
//!   uma migration já lançada, sempre crie uma nova (m0002, m0003...).
//! - `PRAGMA user_version` passa a significar "quantas migrations foram aplicadas",
//!   não mais a MAJOR version do app (esquema antigo).
//!
//! ## Transição do esquema antigo
//!
//! Bancos criados antes desta mudança têm `user_version` == MAJOR version do app
//! (ex: 1), não a contagem de migrations. Como `m0001_baseline.sql` usa
//! `CREATE TABLE IF NOT EXISTS` para tudo, é seguro reaplicá-lo num banco já populado
//! — nenhuma tabela existente é tocada, só o que faltar é criado.
//!
//! Fazemos essa transição (zerar `user_version` antes de rodar as migrations) uma
//! única vez, controlada por uma flag em `app_config` (config.db). Depois da primeira vez,
//! o boot normal do `rusqlite_migration` assume o controle e passa a aplicar só o que for novo.

use crate::database::configs::{get_config, set_config};
use crate::errors::AppError;
use rusqlite::Connection;
use rusqlite_migration::{Migrations, M};

const MIGRATION_BOOTSTRAP_FLAG: &str = "schema_migration_system";
const MIGRATION_BOOTSTRAP_VALUE: &str = "v2";

fn migrations() -> Migrations<'static> {
    Migrations::new(vec![
        M::up(include_str!("../../migrations/m0001_baseline.sql")),
        // Próxima mudança de schema, exemplo:
        // M::up(include_str!("../../migrations/m0002_xxx.sql")),
    ])
}

/// Executa as migrations pendentes em `games.db`.
///
/// Deve ser chamada logo após abrir a conexão do `games.db` e ANTES de qualquer query nas tabelas
/// do domínio. Recebe `config_conn` porque a flag de transição vive em `config.db`, não em `games.db`.
pub fn run_migrations(
    config_conn: &Connection,
    games_conn: &mut Connection,
) -> Result<(), AppError> {
    let already_bootstrapped = get_config(config_conn, MIGRATION_BOOTSTRAP_FLAG)
        .map_err(|e| AppError::DatabaseError(e.to_string()))?
        .as_deref()
        == Some(MIGRATION_BOOTSTRAP_VALUE);

    if !already_bootstrapped {
        // Transição única — zera a versão do esquema antigo (major do app) antes do
        // rusqlite_migration assumir a contagem real de migrations.
        // Seguro mesmo em banco recém-criado (idempotente).
        games_conn
            .pragma_update(None, "user_version", 0u32)
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;
    }

    migrations()
        .to_latest(games_conn)
        .map_err(|e| AppError::DatabaseError(format!("Erro ao aplicar migrations: {e}")))?;

    if !already_bootstrapped {
        set_config(config_conn, MIGRATION_BOOTSTRAP_FLAG, MIGRATION_BOOTSTRAP_VALUE)
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;
    }

    Ok(())
}

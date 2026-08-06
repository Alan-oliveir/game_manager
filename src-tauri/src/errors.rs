use serde::{Serialize, Serializer};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Erro de banco de dados: {0}")]
    DatabaseError(String),

    #[error("Erro de validação: {0}")]
    ValidationError(String),

    #[error("Erro de rede: {0}")]
    NetworkError(String),

    #[error("Não encontrado: {0}")]
    NotFound(String),

    #[error("Erro interno: falha ao acessar recurso compartilhado")]
    MutexError,

    #[error("Erro de I/O: {0}")]
    IoError(String),

    #[error("Erro de serialização: {0}")]
    SerializationError(String),

    #[error("Erro ao interpretar resposta da API: {0}")]
    ParseError(String),

    #[error("Recurso já existe: {0}")]
    AlreadyExists(String),

    #[error("Campo ausente no manifest Epic: {0}")]
    EpicMissingField(String),

    #[error("Erro ao ler steamapps: {0}")]
    SteamReadError(String),

    #[error("Erro ao ler libraryfolders.vdf: {0}")]
    SteamLibraryFoldersError(String),

    #[error("Erro ao ler manifest: {0}")]
    SteamManifestError(String),

    #[error("Pasta inválida para scan")]
    ScanInvalidFolder,

    #[error("Erro ao ler pasta raiz: {0}")]
    ScanReadRootError(String),

    #[error("Erro ao ler pasta '{0}': {1}")]
    ScanReadDirError(String, String),

    #[error("Erro ao obter metadata de '{0}': {1}")]
    ScanMetadataError(String, String),

    #[error("Erro ao obter permissões: {0}")]
    ScanPermissionsError(String),

    #[error("Limite de arquivos atingido ({0}). Scan interrompido.")]
    ScanFileLimitReached(usize),

    #[error("Erro ao criar tabela api_cache: {0}")]
    CacheTableCreationError(String),

    #[error("Erro ao criar índice: {0}")]
    CacheIndexCreationError(String),

    #[error("Erro ao salvar cache: {0}")]
    CacheSaveError(String),

    #[error("Erro ao limpar cache: {0}")]
    CacheCleanupError(String),

    #[error("Falha ao obter app_data_dir: {0}")]
    AppDataDirError(String),

    #[error("Falha ao criar diretório: {0}")]
    DirectoryCreationError(String),

    #[error("Erro ao abrir {0}: {1}")]
    DatabaseOpenError(String, String),

    #[error("Erro ao configurar WAL em {0}: {1}")]
    DatabaseWalConfigError(String, String),

    #[error("Erro de configuração OAuth: {0}")]
    OAuthConfigError(String),

    #[error("Erro ao trocar código por token OAuth: {0}")]
    OAuthTokenExchangeError(String),

    #[error("Erro ao renovar token OAuth: {0}")]
    OAuthRefreshError(String),

    #[error("Token OAuth não encontrado para o provedor: {0}")]
    OAuthTokenNotFound(String),

    #[error("Erro ao lançar jogo: {0}")]
    LaunchError(String),

    #[error("Ferramenta '{0}' não encontrada")]
    ToolNotFound(String),

    #[error("Erro ao executar ferramenta '{0}': {1}")]
    ToolExecutionError(String, String),

    #[error("Erro ao verificar versão da ferramenta '{0}': {1}")]
    ToolVersionCheckError(String, String),

    #[error("Erro ao consultar API externa: {0}")]
    ExternalApiError(String),
}

impl From<rusqlite::Error> for AppError {
    fn from(err: rusqlite::Error) -> Self {
        AppError::DatabaseError(err.to_string())
    }
}

// Para erros de Mutex/PoisonError
impl<T> From<std::sync::PoisonError<T>> for AppError {
    fn from(_: std::sync::PoisonError<T>) -> Self {
        AppError::MutexError
    }
}

// Para erros de I/O
impl From<std::io::Error> for AppError {
    fn from(err: std::io::Error) -> Self {
        AppError::IoError(err.to_string())
    }
}

// Para erros de serialização/deserialização JSON
impl From<serde_json::Error> for AppError {
    fn from(err: serde_json::Error) -> Self {
        AppError::SerializationError(err.to_string())
    }
}

// Para erros de rede vindos do reqwest
impl From<reqwest::Error> for AppError {
    fn from(err: reqwest::Error) -> Self {
        AppError::NetworkError(err.to_string())
    }
}

// Para converter String em AppError (útil para migração gradual)
impl From<String> for AppError {
    fn from(err: String) -> Self {
        AppError::DatabaseError(err)
    }
}

// Para converter AppError em String (compatibilidade com código antigo)
impl From<AppError> for String {
    fn from(err: AppError) -> Self {
        err.to_string()
    }
}

// Serializa como string simples usando o Display gerado pelo thiserror, em vez do enum.
// O frontend recebe uma string no catch do invoke(), sem precisar inspecionar o shape do erro.
impl Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

//! Heurísticas compartilhadas para localizar o executável principal de um jogo quando a
//! plataforma não fornece um manifest confiável (EA, IndieGala).

use std::fs;
use std::path::{Path, PathBuf};

/// Substrings (case-insensitive) que indicam um executável auxiliar, não o jogo em si.
/// Usa `contains` em vez de igualdade exata porque esses nomes variam por engine/versão
/// (ex: `UnityCrashHandler32.exe` vs `UnityCrashHandler64.exe`, `UE4PrereqSetup_x64.exe`
/// vs `UE5PrereqSetup_x64.exe`) — comparar por substring cobre as variações sem precisar
/// enumerar cada versão específica.
const IGNORED_EXECUTABLE_SUBSTRINGS: &[&str] = &[
    "unins",         // unins000.exe, uninstall.exe, Uninstaller.exe
    "crashhandler",  // UnityCrashHandler32.exe, UnityCrashHandler64.exe
    "crashreporter", // CrashReporter.exe
    "crashpad",      // chrome_crashpad_handler.exe (launchers baseados em Electron)
    "vc_redist",     // vc_redist.x64.exe, vc_redist.x86.exe
    "vcredist",
    "directx_setup",
    "dxsetup",
    "prereqsetup", // UE4PrereqSetup_x64.exe, UE5PrereqSetup_x64.exe
    "redist",      // instaladores de redistribuível em geral
];

/// Retorna `true` se o nome do arquivo (case-insensitive) contém alguma substring
/// conhecida de executável auxiliar (uninstaller, crash handler, redistribuível, etc).
fn is_ignored_executable(path: &Path) -> bool {
    let Some(file_name) = path.file_name().and_then(|n| n.to_str()) else {
        return true; // nome ilegível — descarta por segurança
    };

    let lower = file_name.to_lowercase();
    IGNORED_EXECUTABLE_SUBSTRINGS
        .iter()
        .any(|ignored| lower.contains(ignored))
}

/// Tenta localizar o executável principal de um jogo sem manifest confiável: pega o
/// maior `.exe` na *raiz* da pasta de instalação, ignorando utilitários auxiliares
/// conhecidos (uninstaller, crash handler, redistribuíveis).
///
/// **Limitação deliberada — não escaneia subpastas.** Jogos com o executável aninhado
/// (ex: relançamentos antigos rodando via DOSBox/emulador) retornam `None`.
pub fn guess_main_executable(install_path: &Path) -> Option<PathBuf> {
    fs::read_dir(install_path)
        .ok()?
        .flatten()
        .map(|e| e.path())
        .filter(|p| {
            p.is_file()
                && p.extension()
                    .and_then(|e| e.to_str())
                    .map(|e| e.eq_ignore_ascii_case("exe"))
                    .unwrap_or(false)
                && !is_ignored_executable(p)
        })
        .max_by_key(|p| fs::metadata(p).map(|m| m.len()).unwrap_or(0))
}

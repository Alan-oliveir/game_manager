import { Game, RawgGame } from "../types";

/**
 * Tenta iniciar o jogo baseado na plataforma.
 * Aceita tanto Game (local) quanto RawgGame (nuvem - apenas para log/erro).
 */
export const launchGame = (game: Game | RawgGame | any) => {
  // Se for um jogo da biblioteca local da plataforma Steam e tivermos ID
  if (!game.platform || game.platform === "Steam") {
    window.open(`steam://rungameid/${game.id}`, "_self");
  } else {
    alert(
      `O lançamento para ${
        game.platform || "esta plataforma"
      } será implementado em breve!`
    );
  }
};

import { Game, RawgGame } from "../types";
import { toast } from "sonner";

/**
 * Tenta iniciar o jogo baseado na plataforma.
 * Aceita tanto Game (local) quanto RawgGame (nuvem - apenas para log/erro).
 */
export const launchGame = (game: Game | RawgGame | any) => {
  // Se for um jogo da biblioteca local da plataforma Steam e tivermos ID
  if (!game.platform || game.platform === "Steam") {
    window.open(`steam://rungameid/${game.id}`, "_self");
  } else {
    toast.info(
      `O lançamento para ${
        game.platform || "esta plataforma"
      } será implementado em breve!`
    );
  }
};

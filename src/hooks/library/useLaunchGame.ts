import { useCallback, useState } from 'react';
import { useTranslation } from 'react-i18next';

import { storesService } from '@/services/storesService.ts';
import { Game } from '@/types';
import { toast } from '@/utils/toast';

const launcherPathOverrideKey = (platform: string) =>
  `launcher_path_override:${platform}`;

/**
 * Gerencia o lançamento de jogos, delegando ao backend a escolha da melhor
 * estratégia disponível: protocolo do launcher (Steam, Battle.net), executável
 * direto (Epic, GOG), ou abertura do launcher/loja quando o jogo não está instalado.
 *
 * Rastreia o estado de "iniciando" por `gameId` (não globalmente), já que a
 * grade da biblioteca pode ter várias ações de lançamento disponíveis ao mesmo tempo
 * — evita desabilitar/mostrar loading em cards que não foram acionados.
 */
export function useLaunchGame() {
  const { t } = useTranslation('library');
  const [launchingId, setLaunchingId] = useState<string | null>(null);

  const launchGame = useCallback(
    async (game: Game) => {
      setLaunchingId(game.id);

      try {
        const override =
          localStorage.getItem(launcherPathOverrideKey(game.platform)) ||
          undefined;

        const outcome = await storesService.launchGame(game.id, override);

        switch (outcome.kind) {
          case 'launched':
            toast.success(t('launch_starting', { name: game.name }));
            break;
          case 'openedLauncher':
            toast.info(
              outcome.installed
                ? t('launch_opening_launcher', {
                    name: game.name,
                    platform: game.platform,
                  })
                : t('launch_opening_launcher_not_installed', {
                    name: game.name,
                    platform: game.platform,
                  })
            );
            break;
          case 'openedStore':
            toast.info(t('launch_opening_store', { name: game.name }));
            break;
          case 'unavailable':
            toast.error(t('launch_unavailable', { name: game.name }));
            break;
        }
      } catch (error) {
        console.error(error);
        toast.error(t('launch_error'));
      } finally {
        setLaunchingId(null);
      }
    },
    [t]
  );

  const isLaunching = useCallback(
    (gameId: string) => launchingId === gameId,
    [launchingId]
  );

  return { launchGame, isLaunching };
}

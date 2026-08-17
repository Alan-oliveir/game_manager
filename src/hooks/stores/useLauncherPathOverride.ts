import { useTranslation } from 'react-i18next';

import { useLocalStorageLibraryPath, useNativePathPicker } from '@/hooks';

/**
 * Gerencia o override de caminho do launcher de uma plataforma, salvo em
 * localStorage sob a chave `launcher_path_override:{library}` — a mesma
 * que `useLaunchGame`/`resolve_launcher_path` (Rust) já leem no backend.
 * Reaproveitado por qualquer plataforma com launcher instalável em local
 * variável (Battle.net, Epic, GOG, Indiegala, Itch.io, Legacy, Steam, Ubisoft).
 */
export function useLauncherPathOverride(library: string) {
  const { t } = useTranslation('platforms');

  const [launcherPath, setLauncherPath] = useLocalStorageLibraryPath(
    `launcher_path_override:${library}`
  );

  const { pick } = useNativePathPicker({
    directory: false,
    title: t('launcher_path_picker_title', { library }),
    filters: [{ name: 'Executável', extensions: ['exe'] }],
    successMessage: t('launcher_path_picker_success'),
  });

  const chooseLauncherPath = async () => {
    const selected = await pick();

    if (selected) setLauncherPath(selected);
  };

  const clearLauncherPath = () => setLauncherPath('');

  return {
    launcherPath,
    setLauncherPath,
    chooseLauncherPath,
    clearLauncherPath,
  };
}

import { FolderOpen, X } from 'lucide-react';
import { useTranslation } from 'react-i18next';

import { SettingsRow } from '@/components/common';
import { useLauncherPathOverride } from '@/hooks/platforms';
import { Button } from '@/ui/button';

import { PathPickerField } from './PathPickerField';

interface LauncherPathSectionProps {
  platform: string;
  title: string;
  description: string;
}

/**
 * Seção de configuração do caminho customizado do launcher de uma plataforma —
 * usado quando o local de instalação do launcher pode variar (usuário escolheu
 * outro drive, instalação portátil, etc). Não é igual ao caminho de installation
 * dos JOGOS (outro override existente por plataforma).
 */
export function LauncherPathSection({
  platform,
  title,
  description,
}: Readonly<LauncherPathSectionProps>) {
  const { t } = useTranslation('platforms');
  const {
    launcherPath,
    setLauncherPath,
    chooseLauncherPath,
    clearLauncherPath,
  } = useLauncherPathOverride(platform);

  return (
    <SettingsRow icon={FolderOpen} title={title} description={description}>
      <div className="flex items-center gap-2">
        <div className="flex-1">
          <PathPickerField
            value={launcherPath}
            onChange={setLauncherPath}
            onBrowse={chooseLauncherPath}
            placeholder={t('launcher_path_not_set')}
            browseLabel={t('launcher_path_select_button')}
            ariaLabel={title}
            showPreview={false}
          />
        </div>
        {launcherPath && (
          <Button
            size="icon-sm"
            variant="outline"
            onClick={clearLauncherPath}
            title={t('launcher_path_clear_title')}
          >
            <X size={14} />
          </Button>
        )}
      </div>
    </SettingsRow>
  );
}

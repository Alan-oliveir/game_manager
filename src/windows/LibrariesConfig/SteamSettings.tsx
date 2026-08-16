import { FolderOpen, Globe, RefreshCw } from 'lucide-react';
import { useTranslation } from 'react-i18next';

import { SettingsRow, StatusBadge } from '@/components/common';
import { useSteamConfig } from '@/hooks/stores';
import { Input } from '@/ui/input';

import {
  LauncherPathSection,
  LibraryActionButton,
  LibraryActionsFooter,
  LibraryHeader,
  LibraryHelpBox,
  PathPickerField,
} from './components';
import { EXTERNAL_LINKS } from './constants';

interface SteamSettingsProps {
  onLibraryUpdate?: () => void;
}

export function SteamSettings({
  onLibraryUpdate,
}: Readonly<SteamSettingsProps>) {
  const { t } = useTranslation('platforms');
  const {
    steamConfig,
    setSteamConfig,
    loading,
    status,
    actions,
    isLoadingSecrets,
  } = useSteamConfig(onLibraryUpdate);

  return (
    <div className="animate-in fade-in slide-in-from-bottom-2 space-y-6 duration-300">
      <LibraryHeader
        title={t('steam_title')}
        description={t('steam_description')}
        rightSlot={
          status.type && (
            <StatusBadge type={status.type} message={status.message} />
          )
        }
      />

      <div className="space-y-4">
        {/* Credenciais da API */}
        <SettingsRow
          icon={Globe}
          title={t('steam_api_credentials_title')}
          description={t('steam_api_credentials_description')}
        >
          <div className="flex flex-col gap-3">
            <div className="flex flex-col gap-2">
              <Input
                type="text"
                value={steamConfig.steamId}
                onChange={e =>
                  setSteamConfig(prev => ({ ...prev, steamId: e.target.value }))
                }
                placeholder={t('steam_id_placeholder')}
                aria-label={t('steam_id_aria_label')}
                disabled={isLoadingSecrets}
                className="bg-background/50"
              />
              <Input
                type="password"
                value={steamConfig.steamApiKey}
                onChange={e =>
                  setSteamConfig(prev => ({
                    ...prev,
                    steamApiKey: e.target.value,
                  }))
                }
                placeholder={t('steam_api_key_placeholder')}
                aria-label={t('steam_api_key_aria_label')}
                disabled={isLoadingSecrets}
                className="bg-background/50"
              />
            </div>
          </div>
        </SettingsRow>

        {/* Diretório de Instalação */}
        <SettingsRow
          icon={FolderOpen}
          title={t('steam_install_dir_title')}
          description={t('steam_install_dir_description')}
        >
          <PathPickerField
            value={steamConfig.steamRoot}
            onChange={value =>
              setSteamConfig(prev => ({ ...prev, steamRoot: value }))
            }
            onBrowse={actions.chooseSteamDirectory}
            placeholder={t('steam_no_directory_selected')}
            browseLabel={t('steam_change')}
            ariaLabel={t('steam_install_dir_title')}
            showPreview={false}
          />
        </SettingsRow>

        <LauncherPathSection
          platform="Steam"
          title={t('steam_launcher_path_title')}
          description={t('launcher_path_description')}
        />
      </div>

      {/* Ajuda */}
      <LibraryHelpBox
        badge={t('steam_help_badge')}
        title={t('steam_help_title')}
        description={t('steam_help_description')}
        links={[
          {
            questionLabel: t('steam_no_api_key_question'),
            linkLabel: t('steam_get_api_key_button'),
            href: EXTERNAL_LINKS.steam.apiKey,
          },
          {
            questionLabel: t('steam_no_steamid_question'),
            linkLabel: t('steam_get_steamid_button'),
            href: EXTERNAL_LINKS.steam.steamId,
          },
        ]}
      />

      <LibraryActionsFooter>
        <LibraryActionButton
          variant="outline"
          onClick={actions.saveSteamKeys}
          isLoading={loading.saving}
          disabled={
            loading.saving || loading.importingSteam || isLoadingSecrets
          }
          label={t('steam_save_credentials')}
        />
        <LibraryActionButton
          onClick={actions.saveAndImport}
          isLoading={loading.importingSteam}
          disabled={
            loading.saving || loading.importingSteam || isLoadingSecrets
          }
          label={t('steam_save_and_import')}
          icon={RefreshCw}
        />
      </LibraryActionsFooter>
    </div>
  );
}

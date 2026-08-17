import { FolderOpen, LogIn, LogOut, RefreshCw } from 'lucide-react';
import { useTranslation } from 'react-i18next';

import { SettingsRow, StatusBadge } from '@/components/common';
import { useGogConfig, useNativePathPicker } from '@/hooks/stores';
import { Tooltip, TooltipContent, TooltipTrigger } from '@/ui/tooltip';

import {
  ImportedItemsBox,
  LauncherPathSection,
  LibraryActionButton,
  LibraryActionsFooter,
  LibraryHeader,
  PathPickerField,
} from './components';

interface GogSettingsProps {
  onLibraryUpdate?: () => void;
}

export function GogSettings({ onLibraryUpdate }: Readonly<GogSettingsProps>) {
  const { t } = useTranslation('platforms');
  const {
    isAuthenticated,
    gogGamesDir,
    setGogGamesDir,
    loading,
    status,
    actions,
  } = useGogConfig(onLibraryUpdate);
  const { pick } = useNativePathPicker({
    directory: true,
    title: t('gog_select_games_dir_title'),
  });

  const handleChooseDir = async () => {
    const selected = await pick();

    if (selected) setGogGamesDir(selected);
  };

  return (
    <div className="animate-in fade-in slide-in-from-bottom-2 space-y-6 duration-300">
      <LibraryHeader
        title={t('gog_title')}
        description={t('gog_description')}
        rightSlot={
          status.type && (
            <StatusBadge type={status.type} message={status.message} />
          )
        }
      />

      <div className="space-y-4">
        <SettingsRow
          icon={isAuthenticated ? LogOut : LogIn}
          title={
            isAuthenticated ? t('gog_connected_title') : t('gog_login_title')
          }
          description={
            isAuthenticated
              ? t('gog_connected_description')
              : t('gog_login_description')
          }
        >
          {!loading.checkingAuth && (
            <LibraryActionButton
              variant="outline"
              onClick={isAuthenticated ? actions.logout : actions.login}
              isLoading={loading.loggingIn}
              disabled={loading.loggingIn}
              label={
                isAuthenticated
                  ? t('gog_disconnect_button')
                  : t('gog_login_button')
              }
              loadingLabel={t('gog_logging_in')}
              icon={isAuthenticated ? LogOut : LogIn}
              className="ml-auto"
            />
          )}
        </SettingsRow>

        {/* Pasta de jogos instalados */}
        <SettingsRow
          icon={FolderOpen}
          title={t('gog_select_games_dir_title')}
          description={t('gog_games_dir_description')}
        >
          <PathPickerField
            value={gogGamesDir}
            onChange={setGogGamesDir}
            onBrowse={handleChooseDir}
            placeholder={t('gog_games_dir_placeholder')}
            browseLabel={t('gog_browse')}
            ariaLabel={t('gog_games_dir_title')}
            showPreview={false}
          />
        </SettingsRow>

        <LauncherPathSection
          library="GOG"
          title={t('gog_launcher_path_title')}
          description={t('launcher_path_description')}
        />

        <ImportedItemsBox
          title={t('gog_imported_title')}
          items={[t('gog_import_item_owned'), t('gog_import_item_installed')]}
          note={t('gog_import_note_details')}
        />
      </div>

      <LibraryActionsFooter>
        <Tooltip>
          <TooltipTrigger asChild>
            <span
              className={!isAuthenticated ? 'cursor-not-allowed' : undefined}
            >
              <LibraryActionButton
                onClick={actions.importGogGames}
                isLoading={loading.importingGog}
                disabled={!isAuthenticated || loading.importingGog}
                label={t('gog_import_button')}
                loadingLabel={t('gog_importing_short')}
                icon={RefreshCw}
              />
            </span>
          </TooltipTrigger>
          {!isAuthenticated && (
            <TooltipContent>{t('gog_connect_first_tooltip')}</TooltipContent>
          )}
        </Tooltip>
      </LibraryActionsFooter>
    </div>
  );
}

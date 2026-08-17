import { Info, LogIn, LogOut, RefreshCw } from 'lucide-react';
import { useTranslation } from 'react-i18next';

import { SettingsRow, StatusBadge } from '@/components/common';
import { useEpicConfig } from '@/hooks/stores';

import {
  DetectedPathsBox,
  ImportedItemsBox,
  InfoNoteBox,
  LauncherPathSection,
  LibraryActionButton,
  LibraryActionsFooter,
  LibraryHeader,
} from './components';
import { DETECTED_PATHS } from './constants';

interface EpicGamesSettingsProps {
  onLibraryUpdate?: () => void;
}

export function EpicGamesSettings({
  onLibraryUpdate,
}: Readonly<EpicGamesSettingsProps>) {
  const { t } = useTranslation('platforms');
  const { loading, status, actions, isAuthenticated } =
    useEpicConfig(onLibraryUpdate);

  return (
    <div className="animate-in fade-in slide-in-from-bottom-2 space-y-6 duration-300">
      <LibraryHeader
        title={t('epic_title')}
        description={t('epic_description')}
        rightSlot={
          status.type && (
            <StatusBadge type={status.type} message={status.message} />
          )
        }
      />

      <div className="space-y-4">
        {/* Conta Epic (OAuth) */}
        <SettingsRow
          icon={isAuthenticated ? LogOut : LogIn}
          title={
            isAuthenticated ? t('epic_connected_title') : t('epic_login_title')
          }
          description={
            isAuthenticated
              ? t('epic_connected_description')
              : t('epic_login_description')
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
                  ? t('epic_disconnect_button')
                  : t('epic_login_button')
              }
              loadingLabel={t('epic_logging_in')}
              icon={isAuthenticated ? LogOut : LogIn}
              className="ml-auto"
            />
          )}
        </SettingsRow>

        {/* Info sobre Detecção Automática (instalados) */}
        <SettingsRow
          icon={Info}
          title={t('epic_auto_detection_title')}
          description={t('epic_auto_detection_description')}
        >
          <DetectedPathsBox
            intro={t('epic_checked_paths')}
            paths={[
              {
                label: t('epic_windows_label'),
                path: DETECTED_PATHS.epic.windows,
              },
              {
                label: t('epic_linux_wine_label'),
                path: DETECTED_PATHS.epic.linuxWine,
              },
            ]}
            note={t('epic_scanner_item_files')}
          />
        </SettingsRow>

        {/* Nota Wine */}
        <SettingsRow
          icon={Info}
          title={t('epic_wine_title')}
          description={t('epic_wine_description')}
        >
          <InfoNoteBox>
            <p className="text-muted-foreground text-xs leading-relaxed">
              {t('epic_wine_note_before')}{' '}
              <strong>{t('epic_wine_prefix_label')}</strong>
              {t('epic_wine_note_after')}
            </p>
          </InfoNoteBox>
        </SettingsRow>

        <LauncherPathSection
          library="Epic"
          title={t('epic_launcher_path_title')}
          description={t('launcher_path_description')}
        />

        {/* Info sobre o que será importado */}
        <ImportedItemsBox
          title={t('epic_imported_title')}
          items={[
            t('epic_import_item_name'),
            t('epic_import_item_install_dir'),
            t('epic_import_item_executable'),
            t('epic_import_item_status'),
            t('epic_import_item_owned'),
          ]}
        />
      </div>

      <LibraryActionsFooter>
        <LibraryActionButton
          onClick={actions.importEpicGames}
          isLoading={loading.importingEpic}
          disabled={loading.importingEpic}
          label={t('epic_import_button')}
          loadingLabel={t('epic_importing_short')}
          icon={RefreshCw}
        />
      </LibraryActionsFooter>
    </div>
  );
}

import { Info, LogIn, LogOut, RefreshCw } from 'lucide-react';
import { useTranslation } from 'react-i18next';

import { SettingsRow, StatusBadge } from '@/components/common';
import { useAmazonConfig } from '@/hooks/platforms';

import {
  DetectedPathsBox,
  ImportedItemsBox,
  PlatformActionButton,
  PlatformActionsFooter,
  PlatformHeader,
} from './components';
import { DETECTED_PATHS } from './constants';

interface AmazonGamesSettingsProps {
  onLibraryUpdate?: () => void;
}

export function AmazonGamesSettings({
  onLibraryUpdate,
}: Readonly<AmazonGamesSettingsProps>) {
  const { t } = useTranslation('platforms');
  const { loading, status, actions, isAuthenticated } =
    useAmazonConfig(onLibraryUpdate);

  return (
    <div className="animate-in fade-in slide-in-from-bottom-2 space-y-6 duration-300">
      <PlatformHeader
        title={t('amazon_title')}
        description={t('amazon_description')}
        rightSlot={
          status.type && (
            <StatusBadge type={status.type} message={status.message} />
          )
        }
      />

      <div className="space-y-4">
        {/* Conta Amazon (OAuth) */}
        <SettingsRow
          icon={isAuthenticated ? LogOut : LogIn}
          title={
            isAuthenticated
              ? t('amazon_connected_title')
              : t('amazon_login_title')
          }
          description={
            isAuthenticated
              ? t('amazon_connected_description')
              : t('amazon_login_description')
          }
        >
          {!loading.checkingAuth && (
            <PlatformActionButton
              variant="outline"
              onClick={isAuthenticated ? actions.logout : actions.login}
              isLoading={loading.loggingIn}
              disabled={loading.loggingIn}
              label={
                isAuthenticated
                  ? t('amazon_disconnect_button')
                  : t('amazon_login_button')
              }
              loadingLabel={t('amazon_logging_in')}
              icon={isAuthenticated ? LogOut : LogIn}
              className="ml-auto"
            />
          )}
        </SettingsRow>

        {/* Info sobre Detecção Automática (instalados) */}
        <SettingsRow
          icon={Info}
          title={t('amazon_auto_detection_title')}
          description={t('amazon_auto_detection_description')}
        >
          <DetectedPathsBox
            intro={t('amazon_checked_paths')}
            paths={[
              {
                label: t('amazon_windows_label'),
                path: DETECTED_PATHS.amazon.windows,
              },
            ]}
            note={t('amazon_windows_only_note')}
          />
        </SettingsRow>

        {/* Info sobre o que será importado */}
        <ImportedItemsBox
          title={t('amazon_imported_title')}
          items={[
            t('amazon_import_item_name'),
            t('amazon_import_item_install_dir'),
            t('amazon_import_item_status'),
            t('amazon_import_item_owned'),
          ]}
        />
      </div>

      <PlatformActionsFooter>
        <PlatformActionButton
          onClick={actions.importAmazonGames}
          isLoading={loading.importingAmazon}
          disabled={loading.importingAmazon}
          label={t('amazon_import_button')}
          loadingLabel={t('amazon_importing_short')}
          icon={RefreshCw}
        />
      </PlatformActionsFooter>
    </div>
  );
}

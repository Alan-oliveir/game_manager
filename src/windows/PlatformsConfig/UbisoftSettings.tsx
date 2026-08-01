import { Info, RefreshCw } from 'lucide-react';
import { useTranslation } from 'react-i18next';

import { SettingsRow, StatusBadge } from '@/components/common';
import { useUbisoftConfig } from '@/hooks/platforms';

import {
  DetectedPathsBox,
  ImportedItemsBox,
  InfoNoteBox,
  LauncherPathSection,
  PlatformActionButton,
  PlatformActionsFooter,
  PlatformHeader,
} from './components';
import { DETECTED_PATHS } from './constants';

interface UbisoftSettingsProps {
  onLibraryUpdate?: () => void;
}

export function UbisoftSettings({
  onLibraryUpdate,
}: Readonly<UbisoftSettingsProps>) {
  const { t } = useTranslation('platforms');
  const { loading, status, actions } = useUbisoftConfig(onLibraryUpdate);

  return (
    <div className="animate-in fade-in slide-in-from-bottom-2 space-y-6 duration-300">
      <PlatformHeader
        title={t('ubisoft_title')}
        description={t('ubisoft_description')}
        rightSlot={
          status.type && (
            <StatusBadge type={status.type} message={status.message} />
          )
        }
      />

      <div className="space-y-4">
        {/* Info sobre Detecção Automática (instalados) */}
        <SettingsRow
          icon={Info}
          title={t('ubisoft_auto_detection_title')}
          description={t('ubisoft_auto_detection_description')}
        >
          <DetectedPathsBox
            intro={t('ubisoft_checked_paths')}
            paths={[
              {
                label: t('ubisoft_windows_label'),
                path: DETECTED_PATHS.ubisoft.windows,
              },
              {
                label: t('ubisoft_linux_label'),
                path: DETECTED_PATHS.ubisoft.linux,
              },
            ]}
          />
        </SettingsRow>

        {/* Nota Wine */}
        <SettingsRow
          icon={Info}
          title={t('ubisoft_wine_title')}
          description={t('ubisoft_linux_note')}
        >
          <InfoNoteBox>
            <p className="text-muted-foreground text-xs leading-relaxed">
              {t('ubisoft_wine_description')}
            </p>
          </InfoNoteBox>
        </SettingsRow>

        <LauncherPathSection
          platform="Ubisoft"
          title={t('ubisoft_launcher_path_title')}
          description={t('launcher_path_description')}
        />

        {/* Info sobre o que será importado */}
        <ImportedItemsBox
          title={t('ubisoft_imported_title')}
          items={[
            t('ubisoft_import_item_library'),
            t('ubisoft_import_item_installed'),
            t('ubisoft_import_item_name_id'),
            t('ubisoft_import_item_install_exec'),
          ]}
          note={t('ubisoft_import_note')}
        />
      </div>

      <PlatformActionsFooter>
        <PlatformActionButton
          onClick={actions.importUbisoftGames}
          isLoading={loading.importingUbisoft}
          disabled={loading.importingUbisoft}
          label={t('ubisoft_import_button')}
          loadingLabel={t('ubisoft_importing_short')}
          icon={RefreshCw}
        />
      </PlatformActionsFooter>
    </div>
  );
}

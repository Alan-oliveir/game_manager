import { Info, RefreshCw } from 'lucide-react';
import { useTranslation } from 'react-i18next';

import { SettingsRow, StatusBadge } from '@/components/common';
import { ImportProgressPayload, useBattleNetConfig } from '@/hooks/plataforms';
import { DETECTED_PATHS } from '@/windows/PlataformsConfig/constants';

import {
  DetectedPathsBox,
  ImportedItemsBox,
  ImportProgressIndicator,
  LauncherPathSection,
  PlatformActionButton,
  PlatformActionsFooter,
  PlatformHeader,
  WarningBox,
} from './components';

interface BattleNetSettingsProps {
  onLibraryUpdate?: () => void;
  progress: ImportProgressPayload | null;
}

export function BattleNetSettings({
  onLibraryUpdate,
  progress,
}: Readonly<BattleNetSettingsProps>) {
  const { t } = useTranslation('platforms');
  const { loading, status, actions } = useBattleNetConfig(onLibraryUpdate);

  return (
    <div className="animate-in fade-in slide-in-from-bottom-2 space-y-6 duration-300">
      <PlatformHeader
        title={t('battlenet_title')}
        description={t('battlenet_description')}
        rightSlot={
          status.type && (
            <StatusBadge type={status.type} message={status.message} />
          )
        }
      />

      <div className="space-y-4">
        <SettingsRow
          icon={Info}
          title={t('battlenet_auto_detection_title')}
          description={t('battlenet_auto_detection_description')}
        >
          <DetectedPathsBox
            intro={t('battlenet_auto_detection_note')}
            paths={[
              {
                label: t('battlenet_windows_label'),
                path: DETECTED_PATHS.battleNet.windows,
              },
            ]}
            note={t('battlenet_linux_note')}
          />
        </SettingsRow>

        <LauncherPathSection
          platform="BattleNet"
          title={t('battlenet_launcher_path_title')}
          description={t('launcher_path_description')}
        />

        <ImportedItemsBox
          title={t('battlenet_imported_title')}
          items={[
            t('battlenet_import_item_installed'),
            t('battlenet_import_item_name'),
            t('battlenet_import_item_executable'),
            t('battlenet_import_item_last_played'),
          ]}
        />

        <WarningBox icon={Info} title={t('battlenet_warning_library_title')}>
          <p className="text-muted-foreground text-xs leading-relaxed">
            {t('battlenet_import_note')}
          </p>
        </WarningBox>
      </div>

      {loading.importingBattleNet && progress && (
        <ImportProgressIndicator
          label={t('battlenet_importing')}
          progress={progress}
        />
      )}

      <PlatformActionsFooter>
        <PlatformActionButton
          onClick={actions.importBattleNetGames}
          isLoading={loading.importingBattleNet}
          disabled={loading.importingBattleNet}
          label={t('battlenet_import_button')}
          loadingLabel={t('battlenet_importing_short')}
          icon={RefreshCw}
        />
      </PlatformActionsFooter>
    </div>
  );
}

import { HardDrive, Info, Library, RefreshCw } from 'lucide-react';
import { useTranslation } from 'react-i18next';

import { SettingsRow, StatusBadge } from '@/components/common';
import { ImportProgressPayload, useItchConfig } from '@/hooks/plataforms';
import { Switch } from '@/ui/toggle-switch';
import { DETECTED_PATHS } from '@/windows/PlataformsConfig/constants';

import {
  DetectedPathsBox,
  ImportedItemsBox,
  ImportProgressIndicator,
  PlatformActionButton,
  PlatformActionsFooter,
  PlatformHeader,
  WarningBox,
} from './components';

interface ItchioSettingsProps {
  onLibraryUpdate?: () => void;
  progress: ImportProgressPayload | null;
}

export function ItchSettings({
  onLibraryUpdate,
  progress,
}: Readonly<ItchioSettingsProps>) {
  const { t } = useTranslation('platforms');
  const { mode, setMode, loading, status, actions } =
    useItchConfig(onLibraryUpdate);

  const isFull = mode === 'full';

  return (
    <div className="animate-in fade-in slide-in-from-bottom-2 space-y-6 duration-300">
      <PlatformHeader
        title={t('itch_title')}
        description={t('itch_description')}
        rightSlot={
          status.type && (
            <StatusBadge type={status.type} message={status.message} />
          )
        }
      />

      <div className="space-y-4">
        <SettingsRow
          icon={Info}
          title={t('itch_auto_detection_title')}
          description={t('itch_auto_detection_description')}
        >
          <DetectedPathsBox
            intro={t('itch_checked_paths')}
            paths={[
              {
                label: t('itch_windows_db_label'),
                path: DETECTED_PATHS.itch.windows,
              },
              {
                label: t('itch_linux_db_label'),
                path: DETECTED_PATHS.itch.linux,
              },
            ]}
          />
        </SettingsRow>

        <SettingsRow
          icon={isFull ? Library : HardDrive}
          title={t('itch_mode_title')}
          description={t('itch_mode_description')}
        >
          <Switch
            checked={isFull}
            onChange={checked => setMode(checked ? 'full' : 'installed')}
            labelOff={t('itch_mode_installed_option')}
            labelOn={t('itch_mode_full_option')}
            className={
              loading.importingItch ? 'pointer-events-none opacity-50' : ''
            }
          />
        </SettingsRow>

        <ImportedItemsBox
          title={t('itch_imported_title')}
          items={
            isFull
              ? [
                  t('itch_import_item_name'),
                  t('itch_import_item_cover'),
                  t('itch_import_item_status'),
                  t('itch_import_item_description'),
                ]
              : [
                  t('itch_import_item_name'),
                  t('itch_import_item_cover'),
                  t('itch_import_item_install_dir'),
                  t('itch_import_item_executable'),
                  t('itch_import_item_playtime'),
                  t('itch_import_item_description'),
                ]
          }
        />

        {isFull && (
          <WarningBox icon={Info} title={t('itch_warning_full_title')}>
            <p className="text-muted-foreground text-xs leading-relaxed">
              {t('itch_full_note')}
            </p>
          </WarningBox>
        )}
      </div>

      {loading.importingItch && progress && (
        <ImportProgressIndicator
          label={isFull ? t('itch_importing_full') : t('itch_importing')}
          progress={progress}
        />
      )}

      <PlatformActionsFooter>
        <PlatformActionButton
          onClick={actions.importItchGames}
          isLoading={loading.importingItch}
          disabled={loading.importingItch}
          label={
            isFull
              ? t('itch_import_button_full')
              : t('itch_import_button_installed')
          }
          loadingLabel={t('itch_importing_short')}
          icon={RefreshCw}
        />
      </PlatformActionsFooter>
    </div>
  );
}

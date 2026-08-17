import { HardDrive, Info, Library, RefreshCw } from 'lucide-react';
import { useTranslation } from 'react-i18next';

import { SettingsRow, StatusBadge } from '@/components/common';
import { useItchConfig } from '@/hooks/stores';
import { Switch } from '@/ui/toggle-switch';
import { DETECTED_PATHS } from '@/windows/LibrariesConfig/constants';

import {
  DetectedPathsBox,
  ImportedItemsBox,
  LauncherPathSection,
  LibraryActionButton,
  LibraryActionsFooter,
  LibraryHeader,
  WarningBox,
} from './components';

interface ItchioSettingsProps {
  onLibraryUpdate?: () => void;
}

export function ItchSettings({
  onLibraryUpdate,
}: Readonly<ItchioSettingsProps>) {
  const { t } = useTranslation('platforms');
  const { mode, setMode, loading, status, actions } =
    useItchConfig(onLibraryUpdate);

  const isFull = mode === 'full';

  return (
    <div className="animate-in fade-in slide-in-from-bottom-2 space-y-6 duration-300">
      <LibraryHeader
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

        <LauncherPathSection
          library="Itch"
          title={t('itch_launcher_path_title')}
          description={t('launcher_path_description')}
        />

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

      <LibraryActionsFooter>
        <LibraryActionButton
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
      </LibraryActionsFooter>
    </div>
  );
}

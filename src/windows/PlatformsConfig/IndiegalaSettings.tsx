import { HardDrive, Info, Library, RefreshCw } from 'lucide-react';
import { useTranslation } from 'react-i18next';

import { SettingsRow, StatusBadge } from '@/components/common';
import { useIndiegalaConfig } from '@/hooks/platforms';
import { Switch } from '@/ui/toggle-switch';
import { DETECTED_PATHS } from '@/windows/PlatformsConfig/constants';

import {
  DetectedPathsBox,
  ImportedItemsBox,
  LauncherPathSection,
  PlatformActionButton,
  PlatformActionsFooter,
  PlatformHeader,
  WarningBox,
} from './components';

interface IndiegalaSettingsProps {
  onLibraryUpdate?: () => void;
}

export function IndiegalaSettings({
  onLibraryUpdate,
}: Readonly<IndiegalaSettingsProps>) {
  const { t } = useTranslation('platforms');
  const { mode, setMode, loading, status, actions } =
    useIndiegalaConfig(onLibraryUpdate);

  const isFull = mode === 'full';

  return (
    <div className="animate-in fade-in slide-in-from-bottom-2 space-y-6 duration-300">
      <PlatformHeader
        title={t('indiegala_title')}
        description={t('indiegala_description')}
        rightSlot={
          status.type && (
            <StatusBadge type={status.type} message={status.message} />
          )
        }
      />

      <div className="space-y-4">
        <SettingsRow
          icon={Info}
          title={t('indiegala_auto_detection_title')}
          description={t('indiegala_auto_detection_description')}
        >
          <DetectedPathsBox
            intro={t('indiegala_checked_paths')}
            paths={[
              {
                label: t('indiegala_windows_installed_label'),
                path: DETECTED_PATHS.indiegala.windowsInstalled,
              },
              {
                label: t('indiegala_windows_config_label'),
                path: DETECTED_PATHS.indiegala.windowsConfig,
              },
            ]}
            note={t('indiegala_windows_only_note')}
          />
        </SettingsRow>

        <LauncherPathSection
          platform="Indiegala"
          title={t('indiegala_launcher_path_title')}
          description={t('launcher_path_description')}
        />

        {/* Modo de importação */}
        <SettingsRow
          icon={isFull ? Library : HardDrive}
          title={t('indiegala_mode_title')}
          description={t('indiegala_mode_description')}
        >
          <Switch
            checked={isFull}
            onChange={checked => setMode(checked ? 'full' : 'installed')}
            labelOff={t('indiegala_mode_installed_option')}
            labelOn={t('indiegala_mode_full_option')}
            className={
              loading.importingIndiegala ? 'pointer-events-none opacity-50' : ''
            }
          />
        </SettingsRow>

        <ImportedItemsBox
          title={t('indiegala_imported_title')}
          items={
            isFull
              ? [
                  t('indiegala_import_item_name'),
                  t('indiegala_import_item_status'),
                  t('indiegala_import_item_description'),
                  t('indiegala_import_item_tags'),
                ]
              : [
                  t('indiegala_import_item_name'),
                  t('indiegala_import_item_install_dir'),
                  t('indiegala_import_item_executable'),
                  t('indiegala_import_item_playtime'),
                  t('indiegala_import_item_description'),
                  t('indiegala_import_item_tags'),
                ]
          }
        />

        {isFull && (
          <WarningBox icon={Info} title={t('indiegala_warning_full_title')}>
            <p className="text-muted-foreground text-xs leading-relaxed">
              {t('indiegala_full_note')}
            </p>
          </WarningBox>
        )}
      </div>

      <PlatformActionsFooter>
        <PlatformActionButton
          onClick={actions.importIndiegalaGames}
          isLoading={loading.importingIndiegala}
          disabled={loading.importingIndiegala}
          label={
            isFull
              ? t('indiegala_import_button_full')
              : t('indiegala_import_button_installed')
          }
          loadingLabel={t('indiegala_importing_short')}
          icon={RefreshCw}
        />
      </PlatformActionsFooter>
    </div>
  );
}

import { FolderOpen, Info, RefreshCw } from 'lucide-react';
import { useTranslation } from 'react-i18next';

import { SettingsRow, StatusBadge } from '@/components/common';
import { useLegacyConfig, useNativePathPicker } from '@/hooks/stores';

import {
  ImportedItemsBox,
  InfoNoteBox,
  LauncherPathSection,
  LibraryActionButton,
  LibraryActionsFooter,
  LibraryHeader,
  PathPickerField,
} from './components';
import { DETECTED_PATHS } from './constants';

interface LegacySettingsProps {
  onLibraryUpdate?: () => void;
}

export function LegacySettings({
  onLibraryUpdate,
}: Readonly<LegacySettingsProps>) {
  const { t } = useTranslation('platforms');
  const { appStatePath, setAppStatePath, loading, status, actions } =
    useLegacyConfig(onLibraryUpdate);
  const { pick } = useNativePathPicker({
    directory: false,
    title: t('legacy_select_app_state_title'),
    filters: [{ name: t('legacy_json_filter_name'), extensions: ['json'] }],
  });

  const handleChooseFile = async () => {
    const selected = await pick();

    if (selected) setAppStatePath(selected);
  };

  return (
    <div className="animate-in fade-in slide-in-from-bottom-2 space-y-6 duration-300">
      <LibraryHeader
        title={t('legacy_title')}
        description={t('legacy_description')}
        rightSlot={
          status.type && (
            <StatusBadge type={status.type} message={status.message} />
          )
        }
      />

      <div className="space-y-4">
        {/* Detecção automática */}
        <SettingsRow
          icon={Info}
          title={t('legacy_auto_detection_title')}
          description={t('legacy_auto_detection_description')}
        >
          <InfoNoteBox>
            <p className="text-muted-foreground text-xs leading-relaxed">
              {t('legacy_auto_detection_note_before')}{' '}
              <code className="text-primary/80">app-state.json</code>{' '}
              {t('legacy_auto_detection_note_after')}
            </p>
            <ul className="text-muted-foreground space-y-1 text-xs">
              <li>
                <strong className="text-foreground/70">
                  {t('legacy_windows_label')}
                </strong>{' '}
                <code>{DETECTED_PATHS.legacy.windows}</code>
              </li>
              <li>
                <strong className="text-foreground/70">
                  {t('legacy_linux_wine_label')}
                </strong>{' '}
                <code>{DETECTED_PATHS.legacy.linuxWine}</code>
              </li>
            </ul>
          </InfoNoteBox>
        </SettingsRow>

        {/* Nota Wine Linux */}
        <SettingsRow
          icon={Info}
          title={t('legacy_wine_title')}
          description={t('legacy_wine_description')}
        >
          <InfoNoteBox>
            <p className="text-muted-foreground text-xs leading-relaxed">
              {t('legacy_wine_note_before')}{' '}
              <strong className="text-foreground/70">
                {t('legacy_wine_linux_label')}
              </strong>{' '}
              {t('legacy_wine_note_after')}
            </p>
          </InfoNoteBox>
        </SettingsRow>

        {/* Caminho manual (opcional) */}
        <SettingsRow
          icon={FolderOpen}
          title={t('legacy_manual_path_title')}
          description={t('legacy_manual_path_description')}
        >
          <PathPickerField
            value={appStatePath}
            onChange={setAppStatePath}
            onBrowse={handleChooseFile}
            placeholder={t('legacy_manual_path_placeholder')}
            browseLabel={t('legacy_browse')}
            ariaLabel={t('legacy_manual_path_title')}
          />
        </SettingsRow>

        <LauncherPathSection
          library="Legacy"
          title={t('legacy_launcher_path_title')}
          description={t('launcher_path_description')}
        />

        {/* O que será importado */}
        <ImportedItemsBox
          title={t('legacy_imported_title')}
          items={[
            t('legacy_import_item_acquired'),
            t('legacy_import_item_metadata'),
            t('legacy_import_item_install'),
            t('legacy_import_item_status'),
          ]}
          note={t('legacy_import_note')}
        />
      </div>

      <LibraryActionsFooter>
        <LibraryActionButton
          onClick={() => actions.importLegacyGames(appStatePath || undefined)}
          isLoading={loading.importingLegacy}
          disabled={loading.importingLegacy}
          label={t('legacy_import_button')}
          loadingLabel={t('legacy_importing_short')}
          icon={RefreshCw}
        />
      </LibraryActionsFooter>
    </div>
  );
}

import { FolderOpen, Info, RefreshCw } from 'lucide-react';
import { useTranslation } from 'react-i18next';

import { SettingsRow, StatusBadge } from '@/components/common';
import { useEaConfig, useNativePathPicker } from '@/hooks/platforms';

import {
  ImportedItemsBox,
  PathPickerField,
  PlatformActionButton,
  PlatformActionsFooter,
  PlatformHeader,
  WarningBox,
} from './components';

interface EaSettingsProps {
  onLibraryUpdate?: () => void;
}

export function EaSettings({ onLibraryUpdate }: Readonly<EaSettingsProps>) {
  const { t } = useTranslation('platforms');
  const { installDir, setInstallDir, loading, status, actions } =
    useEaConfig(onLibraryUpdate);
  const { pick } = useNativePathPicker({
    directory: true,
    title: t('ea_select_install_dir_title'),
  });

  const handleChooseDir = async () => {
    const selected = await pick();

    if (selected) setInstallDir(selected);
  };

  return (
    <div className="animate-in fade-in slide-in-from-bottom-2 space-y-6 duration-300">
      <PlatformHeader
        title={t('ea_title')}
        description={t('ea_description')}
        rightSlot={
          status.type && (
            <StatusBadge type={status.type} message={status.message} />
          )
        }
      />

      <div className="space-y-4">
        {/* Pasta de instalação dos jogos EA (obrigatória — sem OAuth/manifesto próprio) */}
        <SettingsRow
          icon={FolderOpen}
          title={t('ea_select_install_dir_title')}
          description={t('ea_install_dir_description')}
        >
          <PathPickerField
            value={installDir}
            onChange={setInstallDir}
            onBrowse={handleChooseDir}
            placeholder={t('ea_install_dir_placeholder')}
            browseLabel={t('ea_browse')}
            ariaLabel={t('ea_select_install_dir_title')}
            showPreview={false}
          />
        </SettingsRow>

        <ImportedItemsBox
          title={t('ea_imported_title')}
          items={[t('ea_import_item_installed'), t('ea_import_item_owned')]}
          note={t('ea_import_note_details')}
        />

        <WarningBox icon={Info} title={t('ea_warning_library_title')}>
          <p className="text-muted-foreground text-xs leading-relaxed">
            {t('ea_info_note')}
          </p>
        </WarningBox>
      </div>

      <PlatformActionsFooter>
        <PlatformActionButton
          onClick={actions.importEaGames}
          isLoading={loading.importingEa}
          disabled={loading.importingEa}
          label={t('ea_import_button')}
          loadingLabel={t('ea_importing_short')}
          icon={RefreshCw}
        />
      </PlatformActionsFooter>
    </div>
  );
}

import { Info, RefreshCw } from 'lucide-react';
import { useTranslation } from 'react-i18next';

import { SettingsRow, StatusBadge } from '@/components/common';
import { useXboxConfig } from '@/hooks/stores';

import {
  ImportedItemsBox,
  InfoNoteBox,
  LibraryActionButton,
  LibraryActionsFooter,
  LibraryHeader,
  WarningBox,
} from './components';

interface XboxSettingsProps {
  onLibraryUpdate?: () => void;
}

export function XboxSettings({ onLibraryUpdate }: Readonly<XboxSettingsProps>) {
  const { t } = useTranslation('platforms');
  const { loading, status, actions } = useXboxConfig(onLibraryUpdate);

  return (
    <div className="animate-in fade-in slide-in-from-bottom-2 space-y-6 duration-300">
      <LibraryHeader
        title={t('xbox_title')}
        description={t('xbox_description')}
        rightSlot={
          status.type && (
            <StatusBadge type={status.type} message={status.message} />
          )
        }
      />

      <div className="space-y-4">
        {/* Info sobre detecção automática (sem pasta/login a configurar) */}
        <SettingsRow
          icon={Info}
          title={t('xbox_auto_detection_title')}
          description={t('xbox_auto_detection_description')}
        >
          <InfoNoteBox>
            <p className="text-muted-foreground text-xs leading-relaxed">
              {t('xbox_scanner_note')}
            </p>
          </InfoNoteBox>
        </SettingsRow>

        <ImportedItemsBox
          title={t('xbox_imported_title')}
          items={[
            t('xbox_import_item_name'),
            t('xbox_import_item_install_dir'),
            t('xbox_import_item_executable'),
            t('xbox_import_item_store_id'),
          ]}
        />

        <WarningBox icon={Info} title={t('xbox_warning_library_title')}>
          <p className="text-muted-foreground text-xs leading-relaxed">
            {t('xbox_import_note')}
          </p>
        </WarningBox>

        <WarningBox title={t('xbox_warning_gamepass_title')}>
          {t('xbox_gamepass_note')}
        </WarningBox>
      </div>

      <LibraryActionsFooter>
        <LibraryActionButton
          onClick={actions.importXboxGames}
          isLoading={loading.importingXbox}
          disabled={loading.importingXbox}
          label={t('xbox_import_button')}
          loadingLabel={t('xbox_importing_short')}
          icon={RefreshCw}
        />
      </LibraryActionsFooter>
    </div>
  );
}

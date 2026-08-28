import { Globe, Info, RefreshCw } from 'lucide-react';
import { useTranslation } from 'react-i18next';

import { SettingsRow, StatusBadge } from '@/components/common';
import { useXboxConfig } from '@/hooks/stores';
import { Input } from '@/ui/input';

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
  const {
    xboxConfig,
    setXboxConfig,
    loading,
    status,
    actions,
    isLoadingSecrets,
  } = useXboxConfig(onLibraryUpdate);

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
        {/* Credenciais da API Xbox Live */}
        <SettingsRow
          icon={Globe}
          title={t('xbox_live_title')}
          description={t('xbox_live_description')}
        >
          <div className="flex flex-col gap-3">
            <div className="flex flex-col gap-2">
              <Input
                type="password"
                value={xboxConfig.xboxLiveClientId}
                onChange={e =>
                  setXboxConfig(prev => ({
                    ...prev,
                    xboxLiveClientId: e.target.value,
                  }))
                }
                placeholder={t('xbox_live_client_id_placeholder')}
                aria-label={t('xbox_live_client_id_placeholder')}
                disabled={isLoadingSecrets}
                className="bg-background/50"
              />
              <span className="text-muted-foreground text-xs">
                {t('xbox_live_client_id_label')}
              </span>
              <Input
                type="password"
                value={xboxConfig.xboxLiveClientSecret}
                onChange={e =>
                  setXboxConfig(prev => ({
                    ...prev,
                    xboxLiveClientSecret: e.target.value,
                  }))
                }
                placeholder={t('xbox_live_api_key_placeholder')}
                aria-label={t('xbox_live_api_key_placeholder')}
                disabled={isLoadingSecrets}
                className="bg-background/50"
              />
              <span className="text-muted-foreground text-xs">
                {t('xbox_live_client_secret_label')}
              </span>
            </div>
          </div>
        </SettingsRow>

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
          variant="outline"
          onClick={actions.saveXboxKeys}
          isLoading={loading.saving}
          disabled={loading.saving || loading.importingXbox || isLoadingSecrets}
          label={t('xbox_live_save_credentials')}
        />
        <LibraryActionButton
          onClick={actions.importXboxGames}
          isLoading={loading.importingXbox}
          disabled={loading.importingXbox || loading.saving || isLoadingSecrets}
          label={t('xbox_import_button')}
          loadingLabel={t('xbox_importing_short')}
          icon={RefreshCw}
        />
      </LibraryActionsFooter>
    </div>
  );
}

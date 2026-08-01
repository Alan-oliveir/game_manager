import { FolderOpen, Scan } from 'lucide-react';
import { useTranslation } from 'react-i18next';

import { SettingsRow } from '@/components/common';
import { useScanner } from '@/hooks';
import { Button } from '@/ui/button';

import { PlatformActionButton } from './components/PlatformActionButton';
import { PlatformHeader } from './components/PlatformHeader';
import { ScanResultBanner } from './components/ScanResultBanner';
import { DiscoveriesList } from './DiscoveriesList';

export function LocalScannerSettings() {
  const { t } = useTranslation('platforms');
  const {
    scanning,
    result,
    selectedFolder,
    handleSelectFolder,
    handleScan,
    handleAddAll,
  } = useScanner();

  return (
    <div className="animate-in fade-in slide-in-from-bottom-2 space-y-6 duration-300">
      <PlatformHeader
        title={t('scanner_title')}
        description={t('scanner_description')}
      />

      <div className="space-y-6">
        {/* Área de Controles */}
        <SettingsRow
          icon={FolderOpen}
          title={t('scanner_directory_title')}
          description={t('scanner_directory_description')}
        >
          <div className="flex flex-col gap-3">
            {selectedFolder && (
              <div className="bg-muted/30 flex items-center gap-2 overflow-hidden rounded-md border p-2">
                <span className="text-muted-foreground shrink-0 text-[10px] font-bold uppercase">
                  {t('scanner_path_label')}:
                </span>
                <code className="text-primary/80 truncate text-xs">
                  {selectedFolder}
                </code>
              </div>
            )}
            <div className="flex gap-2">
              <Button
                variant="outline"
                onClick={handleSelectFolder}
                className="hover:border-primary/50 flex-1 border-dashed"
              >
                <FolderOpen className="mr-2 h-4 w-4" />
                {selectedFolder
                  ? t('scanner_change_folder')
                  : t('scanner_select_folder')}
              </Button>
              <PlatformActionButton
                onClick={handleScan}
                isLoading={scanning}
                disabled={!selectedFolder || scanning}
                label={t('scanner_start_scan')}
                loadingLabel={t('scanner_scanning')}
                icon={Scan}
                className="flex-1"
              />
            </div>
          </div>
        </SettingsRow>

        {/* Área de Resultados */}
        {result && (
          <div className="animate-in fade-in slide-in-from-bottom-4 duration-500">
            <ScanResultBanner
              success={result.success}
              message={result.message}
              onAddAll={
                result.success && result.discoveries.length > 0
                  ? handleAddAll
                  : undefined
              }
              addAllDisabled={scanning}
              addAllLabel={t('scanner_add_all')}
            />

            {result.success && result.discoveries.length > 0 && (
              <div className="space-y-4">
                <div className="flex items-center justify-between">
                  <h3 className="text-lg font-semibold">
                    {t('scanner_choose_executables')}
                  </h3>
                </div>
                <DiscoveriesList discoveries={result.discoveries} />
              </div>
            )}
          </div>
        )}

        {!result && (
          <div className="border-border/40 flex flex-col items-center justify-center rounded-lg border border-dashed p-12 text-center">
            <Scan className="text-muted-foreground/30 mb-4 h-12 w-12" />
            <p className="text-muted-foreground text-sm">
              {t('scanner_empty_state')}
            </p>
          </div>
        )}
      </div>
    </div>
  );
}

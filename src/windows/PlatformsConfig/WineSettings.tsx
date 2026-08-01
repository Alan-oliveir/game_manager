import {
  CheckCircle2,
  FolderOpen,
  Info,
  Save,
  Trash2,
  Wine,
} from 'lucide-react';
import { useTranslation } from 'react-i18next';

import { SettingsRow } from '@/components/common';
import { useWineConfig } from '@/hooks/platforms';
import { Button } from '@/ui/button';

import {
  InfoNoteBox,
  PathPickerField,
  PlatformActionsFooter,
  PlatformHeader,
  WarningBox,
} from './components';

export function WineSettings() {
  const { t } = useTranslation('platforms');
  const { winePrefix, setWinePrefix, saved, actions } = useWineConfig();

  return (
    <div className="animate-in fade-in slide-in-from-bottom-2 space-y-6 duration-300">
      <PlatformHeader
        title={t('wine_title')}
        description={t('wine_description')}
        rightSlot={
          saved && (
            <div className="flex items-center gap-1.5 rounded-full bg-emerald-500/10 px-3 py-1.5 text-xs font-medium text-emerald-500">
              <CheckCircle2 size={13} />
              {t('wine_saved')}
            </div>
          )
        }
      />

      {/* Aviso Linux */}
      <WarningBox icon={Info} title={t('wine_linux_only_title')}>
        {t('wine_linux_only_prefix')}{' '}
        <strong className="text-foreground/80">{t('wine_linux_label')}</strong>
        {t('wine_linux_only_suffix')}
      </WarningBox>

      <div className="space-y-4">
        {/* Wine Prefix */}
        <SettingsRow
          icon={FolderOpen}
          title={t('wine_prefix_title')}
          description={t('wine_prefix_description')}
        >
          <PathPickerField
            value={winePrefix}
            onChange={setWinePrefix}
            onBrowse={actions.chooseWinePrefix}
            placeholder={t('wine_prefix_placeholder')}
            browseLabel={t('wine_browse')}
            ariaLabel={t('wine_prefix_title')}
          />
        </SettingsRow>

        {/* Como funciona */}
        <SettingsRow
          icon={Wine}
          title={t('wine_how_used_title')}
          description={t('wine_how_used_description')}
        >
          <InfoNoteBox>
            <p className="text-muted-foreground text-xs">
              {t('wine_command_intro')}
            </p>
            <code className="bg-background/60 block rounded border px-2 py-1.5 text-[10px] leading-relaxed text-emerald-400">
              WINEPREFIX=
              <span className="text-blue-400">
                {winePrefix || '/home/user/.wine'}
              </span>{' '}
              wine &lt;{t('wine_executable_placeholder')}&gt;
            </code>
            <p className="text-muted-foreground mt-1 text-[10px]">
              {t('wine_default_prefix_note')} <code>~/.wine</code>.
            </p>
          </InfoNoteBox>
        </SettingsRow>
      </div>

      {/* Ações */}
      <PlatformActionsFooter>
        {winePrefix && (
          <Button
            variant="outline"
            size="sm"
            onClick={actions.clearWinePrefix}
            className="gap-2 text-xs text-red-400 hover:bg-red-500/10 hover:text-red-500"
          >
            <Trash2 size={13} />
            {t('wine_clear')}
          </Button>
        )}
        <Button
          onClick={actions.saveWinePrefix}
          className="gap-2 bg-blue-600 text-white hover:bg-blue-700"
        >
          <Save size={14} />
          {t('wine_save_configuration')}
        </Button>
      </PlatformActionsFooter>
    </div>
  );
}

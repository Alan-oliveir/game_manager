import {
  BrainCircuit,
  Database,
  Download,
  ExternalLink,
  FileJson,
  Gamepad2,
  GamepadDirectional,
  HardDrive,
  History,
  ImageIcon,
  Images,
  Languages,
  Loader2,
  RefreshCcw,
  Save,
  Search,
  ShieldAlert,
  ShoppingBag,
  Sparkles,
  Trash2,
  Upload,
  WandSparkles,
} from 'lucide-react';
import { useTranslation } from 'react-i18next';

import { AboutPlaylite, SettingsRow, StatusBadge } from '@/components/common';
import { SubscriptionsConfig } from '@/components/subscriptions/SubscriptionsConfig.tsx';
import {
  useRecommendation,
  useRecommendationSliders,
  useSettings,
} from '@/hooks';
import { Button } from '@/ui/button';
import { Input } from '@/ui/input';
import { Separator } from '@/ui/separator';
import { Slider } from '@/ui/slider';
import { Switch } from '@/ui/toggle-switch';
import { toast } from '@/utils/toast';

interface SettingsProps {
  onLibraryUpdate: () => void;
}

export default function Settings({ onLibraryUpdate }: Readonly<SettingsProps>) {
  const { t } = useTranslation('settings');
  const {
    keys,
    setKeys,
    loading,
    status,
    progress,
    actions,
    saveLocally,
    toggleSaveLocally,
    handleClearCache,
  } = useSettings(onLibraryUpdate);

  // Hook de Recomendação para gerenciar configs e preferências
  const {
    config,
    updateConfig,
    resetFeedback,
    ignoredIds,
    toggleAdultFilter,
    setSeriesLimit,
  } = useRecommendation();

  // Hook para gerenciar sliders de recomendação
  const {
    weights,
    decay,
    handleWeightChange,
    handleDecayChange,
    handleSeriesToggle,
    weightsDescription,
    decayDescription,
  } = useRecommendationSliders(config, updateConfig);

  if (loading.initial) {
    return (
      <div className="flex h-full items-center justify-center">
        <Loader2 className="animate-spin text-blue-500" size={32} />
      </div>
    );
  }

  return (
    <div className="custom-scrollbar flex-1 space-y-8 overflow-y-auto p-8 pb-24">
      {/* HEADER E STATUS */}
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-3xl font-bold tracking-tight">
            {t('header_title')}
          </h2>
          <p className="text-muted-foreground mt-2">
            {t('header_description')}
          </p>
        </div>

        <StatusBadge type={status.type} message={status.message} />
      </div>

      <Separator />

      {/* SEÇÃO 1: ASSINATURAS */}
      <section className="space-y-4">
        <h3 className="text-lg font-semibold">{t('subscriptions_section')}</h3>
        <SubscriptionsConfig />
      </section>

      {/* SEÇÃO 2: METADADOS */}
      <section className="space-y-4">
        <h3 className="text-lg font-semibold">{t('metadata_section')}</h3>

        {/* Configurações de API para tradução de descrições - GEMINI */}
        <SettingsRow
          icon={Languages}
          title={t('gemini_title')}
          description={t('gemini_description')}
        >
          <div className="grid gap-2">
            <Input
              type="password"
              placeholder={t('gemini_api_key_placeholder')}
              value={keys.geminiApiKey}
              onChange={e => setKeys({ ...keys, geminiApiKey: e.target.value })}
              className="bg-background/50"
            />
            <div className="text-muted-foreground flex items-center gap-1 text-xs">
              <span>{t('no_key_question')}</span>
              <a
                href="https://aistudio.google.com/app/apikey"
                target="_blank"
                rel="noreferrer"
                className="flex items-center gap-0.5 text-blue-400 hover:underline"
              >
                {t('get_api_key_button')} <ExternalLink size={10} />
              </a>
            </div>
          </div>
        </SettingsRow>

        {/* Configurações de API para descoberta e busca semântica - GAMEBRAIN */}
        <SettingsRow
          icon={BrainCircuit}
          title="GameBrain"
          description={t('gamebrain_description')}
        >
          <div className="grid gap-2">
            <Input
              type="password"
              placeholder={t('gamebain_api_key_placeholder')}
              value={keys.gamebrainApiKey}
              onChange={e =>
                setKeys({
                  ...keys,
                  gamebrainApiKey: e.target.value,
                })
              }
              className="bg-background/50"
            />

            <div className="text-muted-foreground flex items-center gap-1 text-xs">
              <span>{t('no_key_question')}</span>
              <a
                href="https://gamebrain.co/api"
                target="_blank"
                rel="noreferrer"
                className="flex items-center gap-0.5 text-blue-400 hover:underline"
              >
                {t('get_api_key_button_gamebrain')} <ExternalLink size={10} />
              </a>
            </div>
          </div>
        </SettingsRow>

        {/* Configurações de API para imagens - SteamGridDB */}
        <SettingsRow
          icon={Images}
          title={t('steamgriddb_title')}
          description={t('steamgriddb_description')}
        >
          <div className="grid gap-2">
            <Input
              type="password"
              placeholder={t('steamgriddb_api_key_placeholder')}
              value={keys.steamgriddbApiKey}
              onChange={e =>
                setKeys({ ...keys, steamgriddbApiKey: e.target.value })
              }
              className="bg-background/50"
            />

            <div className="text-muted-foreground flex items-center gap-1 text-xs">
              <span>{t('no_key_question')}</span>

              <a
                href="https://steamgriddb.com/developer"
                target="_blank"
                rel="noreferrer"
                className="flex items-center gap-0.5 text-blue-400 hover:underline"
              >
                {t('get_api_key_button_steamgriddb')} <ExternalLink size={10} />
              </a>
            </div>
          </div>
        </SettingsRow>

        {/* Configurações de API para preços - ITAD */}
        <SettingsRow
          icon={ShoppingBag}
          title={t('itad_title')}
          description={t('itad_description')}
        >
          <div className="grid gap-2">
            <Input
              type="password"
              placeholder={t('itad_api_key_placeholder')}
              value={keys.itadApiKey}
              onChange={e => setKeys({ ...keys, itadApiKey: e.target.value })}
              className="bg-background/50"
            />

            <div className="text-muted-foreground flex items-center gap-1 text-xs">
              <span>{t('no_key_question')}</span>

              <a
                href="https://isthereanydeal.com/apps/"
                target="_blank"
                rel="noreferrer"
                className="flex items-center gap-0.5 text-blue-400 hover:underline"
              >
                {t('get_api_key_button_itad')} <ExternalLink size={10} />
              </a>
            </div>
          </div>
        </SettingsRow>

        {/* Configurações de API para mods - Nexus Mods */}
        <SettingsRow
          icon={GamepadDirectional}
          title={t('nexus_title')}
          description={t('nexus_description')}
        >
          <div className="grid gap-2">
            <Input
              type="password"
              placeholder={t('nexus_api_key_placeholder')}
              value={keys.nexusApiKey}
              onChange={e => setKeys({ ...keys, nexusApiKey: e.target.value })}
              className="bg-background/50"
            />

            <div className="text-muted-foreground flex items-center gap-1 text-xs">
              <span>{t('no_key_question')}</span>

              <a
                href="https://www.nexusmods.com/settings/api-keys"
                target="_blank"
                rel="noreferrer"
                className="flex items-center gap-0.5 text-blue-400 hover:underline"
              >
                {t('get_api_key_button_nexus')} <ExternalLink size={10} />
              </a>
            </div>
          </div>
        </SettingsRow>

        {/* Configurações de API para metadados - IGBD */}
        <SettingsRow
          icon={Gamepad2}
          title={t('igdb_title')}
          description={t('igdb_description')}
        >
          <div className="flex flex-col gap-3">
            <div className="flex flex-col gap-2">
              <Input
                type="password"
                value={keys.igdbClientId}
                onChange={e =>
                  setKeys({ ...keys, igdbClientId: e.target.value })
                }
                placeholder={t('igdb_client_id_placeholder')}
                className="bg-background/50"
              />
              <span className="text-muted-foreground text-xs">
                {t('igdb_client_id_label')}
              </span>
              <Input
                type="password"
                placeholder={t('igdb_api_key_placeholder')}
                value={keys.igdbClientSecret}
                onChange={e =>
                  setKeys({ ...keys, igdbClientSecret: e.target.value })
                }
                className="bg-background/50"
              />
              <span className="text-muted-foreground text-xs">
                {t('igdb_client_secret_label')}
              </span>
            </div>
          </div>
        </SettingsRow>

        {/* Ações para enriquecer metadados e buscar capas */}
        <SettingsRow
          icon={Search}
          title={t('search_metadata_title')}
          description={t('search_metadata_description')}
        >
          <div className="w-full space-y-2">
            {/* Container flex para manter os botões lado a lado */}
            <div className="flex w-full gap-2">
              <Button
                onClick={actions.fetchMissingCovers}
                variant="outline"
                className="flex-1" // Alterado de w-full para flex-1
                disabled={loading.fetchingCovers || loading.fillingMissing}
                title={t('fetch_covers_tooltip')}
              >
                {loading.fetchingCovers ? (
                  <>
                    <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                    {t('searching')}
                  </>
                ) : (
                  <>
                    <ImageIcon className="mr-2 h-4 w-4" />
                    {t('fetch_covers_button')}
                  </>
                )}
              </Button>

              <Button
                onClick={actions.fillMissingMetadata}
                variant="outline"
                className="flex-1" // Alterado de w-full para flex-1
                disabled={loading.fillingMissing || loading.fetchingCovers}
                title={t('fill_missing_tooltip')}
              >
                {loading.fillingMissing ? (
                  <>
                    <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                    {t('filling_fields')}
                  </>
                ) : (
                  <>
                    <WandSparkles className="mr-2 h-4 w-4" />
                    {t('fill_missing_fields_button')}
                  </>
                )}
              </Button>
            </div>

            {(loading.fetchingCovers || loading.fillingMissing) && progress && (
              <div className="text-muted-foreground animate-pulse text-center text-xs">
                {t('processing')} {progress.game} ({progress.current}/
                {progress.total})
              </div>
            )}
          </div>
        </SettingsRow>
      </section>

      {/* SEÇÃO 3: ALGORITMO DE RECOMENDAÇÃO */}
      <section className="space-y-4">
        <h3 className="flex items-center gap-2 text-lg font-semibold">
          {t('recommendation_algorithm_section')}
        </h3>

        {/* Slider: Perfil vs Comunidade */}
        <SettingsRow
          icon={BrainCircuit}
          title={t('recommendation_focus_title')}
          description={t('recommendation_focus_description')}
        >
          <Slider
            min={0}
            max={100}
            step={5}
            value={weights}
            onChange={handleWeightChange}
            leftLabel={value => `${t('my_profile_label')} (${100 - value}%)`}
            rightLabel={value => `${t('community_label')} (${value}%)`}
            description={weightsDescription}
          />
        </SettingsRow>

        {/* Slider: Age Decay */}
        <SettingsRow
          icon={History}
          title={t('nostalgia_factor_title')}
          description={t('nostalgia_factor_description')}
        >
          <Slider
            min={90}
            max={100}
            step={1}
            value={decay}
            onChange={handleDecayChange}
            leftLabel={() => t('new_games_label')}
            rightLabel={() => t('classics_label')}
            description={value =>
              `${t('current_value')}: ${value}%. ${decayDescription(value)}`
            }
          />
        </SettingsRow>

        <SettingsRow
          icon={Sparkles}
          title={t('series_settings_title')}
          description={t('series_settings_description')}
        >
          <div className="flex flex-col gap-4 pt-2">
            {/* Toggle Priorizar Séries */}
            <div className="flex items-center justify-between">
              <span className="text-muted-foreground text-sm">
                {t('prioritize_label')}
              </span>
              <Switch
                checked={config.favor_series}
                onChange={handleSeriesToggle}
                labelOff={t('disabled')}
                labelOn={t('enabled')}
              />
            </div>
            {/* Select Diversidade de Séries */}
            <div className="flex items-center justify-between border-t border-white/5 pt-4">
              <span className="text-muted-foreground text-sm">
                {t('limit_label')}
              </span>
              <select
                value={config.series_limit}
                onChange={async e => {
                  const value = e.target.value as
                    | 'none'
                    | 'moderate'
                    | 'aggressive';
                  await setSeriesLimit(value);
                  toast.success(t('series_limit_updated_toast'));
                }}
                className="bg-secondary text-secondary-foreground focus:ring-primary rounded-md border-none px-3 py-2 text-sm font-medium outline-none focus:ring-1"
              >
                <option value="none">{t('no_limit')}</option>
                <option value="moderate">{t('moderate_limit')}</option>
                <option value="aggressive">{t('aggressive_limit')}</option>
              </select>
            </div>
          </div>
        </SettingsRow>

        <SettingsRow
          icon={ShieldAlert}
          title={t('filter_adult_title')}
          description={t('filter_adult_description')}
        >
          <div className="flex justify-end">
            <Switch
              checked={config.filter_adult_content}
              onChange={async () => {
                await toggleAdultFilter();
                toast.success(
                  config.filter_adult_content
                    ? t('filter_adult_disabled_toast')
                    : t('filter_adult_enabled_toast')
                );
              }}
              labelOff={t('disabled')}
              labelOn={t('enabled')}
            />
          </div>
        </SettingsRow>

        <SettingsRow
          icon={Trash2}
          title={t('clear_feedback_title')}
          description={t('clear_feedback_description', {
            count: ignoredIds.length,
          })}
        >
          <Button
            variant="outline"
            onClick={() => {
              resetFeedback();
              toast.success(t('feedback_cleared_toast'));
            }}
            disabled={ignoredIds.length === 0}
            className="w-full text-red-500 hover:bg-red-500/10 hover:text-red-600"
          >
            <>
              <Trash2 className="mr-2 h-4 w-4" />{' '}
              {t('reset_preferences_button')}
            </>
          </Button>
        </SettingsRow>
      </section>

      {/* SEÇÃO 4: ZONA DE DADOS */}
      <section className="space-y-4">
        <h3 className="text-lg font-semibold text-red-500/80">
          {t('data_zone_section')}
        </h3>

        <SettingsRow
          icon={Save}
          title={t('save_credentials_title')}
          description={t('save_credentials_description')}
        >
          <Button
            onClick={actions.saveKeys}
            disabled={loading.saving}
            className="w-full bg-blue-600 text-white hover:bg-blue-700"
          >
            {loading.saving ? (
              <Loader2 className="mr-2 h-4 w-4 animate-spin" />
            ) : (
              <Save className="mr-2 h-4 w-4" />
            )}
            {t('save_keys_button')}
          </Button>
        </SettingsRow>

        <SettingsRow
          icon={FileJson}
          title={t('manage_backup_title')}
          description={t('manage_backup_description')}
        >
          <div className="space-y-2">
            <div className="flex gap-2">
              <Button
                onClick={actions.importDatabase}
                variant="outline"
                className="flex-1"
                disabled={loading.importingBackup}
              >
                {loading.importingBackup ? (
                  <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                ) : (
                  <>
                    <Download className="mr-2 h-4 w-4" /> {t('import_button')}
                  </>
                )}
              </Button>
              <Button
                onClick={actions.exportDatabase}
                variant="outline"
                className="flex-1"
                disabled={loading.exporting}
              >
                {loading.exporting ? (
                  <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                ) : (
                  <>
                    <Upload className="mr-2 h-4 w-4" /> {t('export_button')}
                  </>
                )}
              </Button>
            </div>
            <div className="text-muted-foreground text-xs">
              {t('backup_location_info')}
            </div>
          </div>
        </SettingsRow>

        <SettingsRow
          icon={Database}
          title={t('clear_cache_title')}
          description={t('clear_cache_description')}
        >
          <div className="flex gap-2">
            <Button
              onClick={actions.cleanupCache}
              variant="outline"
              className="flex-1"
            >
              {loading.cleaningCache ? (
                <Loader2 className="mr-2 h-4 w-4 animate-spin" />
              ) : (
                <RefreshCcw className="mr-2 h-4 w-4" />
              )}
              {t('expired_button')}
            </Button>
            <Button
              onClick={actions.clearAllCache}
              variant="outline"
              className="flex-1 text-red-500 hover:bg-red-500/10 hover:text-red-600"
            >
              {loading.clearingAllCache ? (
                <Loader2 className="mr-2 h-4 w-4 animate-spin" />
              ) : (
                <Trash2 className="mr-2 h-4 w-4" />
              )}
              {t('all_button')}
            </Button>
          </div>
        </SettingsRow>

        <SettingsRow
          icon={HardDrive}
          title={t('image_storage_title')}
          description={t('image_storage_description')}
        >
          <div className="flex flex-col gap-3 pt-2">
            {/* Toggle */}
            <div className="flex items-center justify-end">
              <Switch
                checked={saveLocally}
                onChange={toggleSaveLocally}
                labelOff={t('disabled')}
                labelOn={t('enabled')}
              />
            </div>

            {/* Botão de Limpeza */}
            <div className="flex items-center justify-between border-t border-white/5 pt-3">
              <Button
                variant="outline"
                size="sm"
                onClick={handleClearCache}
                className="w-full text-xs text-red-500 hover:bg-red-500/10 hover:text-red-600"
              >
                <Trash2 size={12} className="mr-1" />{' '}
                {t('delete_saved_images_button')}
              </Button>
            </div>
          </div>
        </SettingsRow>
      </section>

      {/* SEÇÃO 5: SOBRE O PLAYLITE */}
      <section className="space-y-4">
        <h3 className="text-lg font-semibold">{t('about_section')}</h3>
        <AboutPlaylite />
      </section>
    </div>
  );
}

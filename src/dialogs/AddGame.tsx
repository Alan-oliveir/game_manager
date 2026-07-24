import { open } from '@tauri-apps/plugin-dialog';
import { FileSearch, FolderOpen, Gamepad2, Star } from 'lucide-react';
import { useTranslation } from 'react-i18next';

import { useGameForm } from '@/hooks';
import { Game } from '@/types';
import { Button } from '@/ui/button';
import {
  Dialog,
  DialogContent,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/ui/dialog';
import { Input } from '@/ui/input';
import { Label } from '@/ui/label';
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/ui/select';
import { Separator } from '@/ui/separator';

interface AddGameModalProps {
  isOpen: boolean;
  onClose: () => void;
  onSave: (gameData: Partial<Game>) => void;
  gameToEdit?: Game | null;
}

export default function AddGame({
  isOpen,
  onClose,
  onSave,
  gameToEdit,
}: Readonly<AddGameModalProps>) {
  const { formData, handleChange, isValid, buildPayload } = useGameForm(
    isOpen,
    gameToEdit
  );
  const { t } = useTranslation('dialog');

  const handleSelectFolder = async () => {
    try {
      const selected = await open({
        directory: true,
        multiple: false,
        title: t('add_game_select_install_folder_title'),
      });

      if (selected) {
        handleChange('installPath', selected);
      }
    } catch (error) {
      console.error('Erro ao selecionar pasta:', error);
    }
  };

  const handleSelectExecutable = async () => {
    try {
      const selected = await open({
        multiple: false,
        title: t('add_game_select_executable_title'),
        filters: [
          {
            name: t('add_game_executable_filter_name'),
            extensions: ['exe'],
          },
        ],
      });

      if (selected) {
        handleChange('executablePath', selected);
      }
    } catch (error) {
      console.error('Erro ao selecionar executável:', error);
    }
  };

  const handleSave = () => {
    if (!isValid()) return;

    const payload = buildPayload();
    onSave(payload as Partial<Game>);
    onClose();
  };

  return (
    <Dialog open={isOpen} onOpenChange={onClose}>
      <DialogContent className="custom-scrollbar max-h-[90vh] overflow-y-auto sm:max-w-150">
        <DialogHeader>
          <DialogTitle>
            {gameToEdit
              ? t('add_game_edit_title')
              : t('add_game_add_manual_title')}
          </DialogTitle>
        </DialogHeader>

        <div className="grid gap-4 py-4">
          {/* === DADOS BÁSICOS === */}
          <div className="grid gap-2">
            <Label htmlFor="name">{t('add_game_name_label')}</Label>
            <Input
              id="name"
              value={formData.name}
              onChange={e => handleChange('name', e.target.value)}
              placeholder={t('add_game_name_placeholder')}
            />
          </div>

          <div className="grid gap-2">
            <Label htmlFor="cover">{t('add_game_cover_label')}</Label>
            <Input
              id="cover"
              value={formData.coverUrl}
              onChange={e => handleChange('coverUrl', e.target.value)}
              placeholder={t('add_game_cover_placeholder')}
            />
          </div>

          <div className="grid grid-cols-3 gap-2">
            <div className="grid gap-2">
              <Label>{t('add_game_platform_label')}</Label>
              <Select
                value={formData.platform}
                onValueChange={val => handleChange('platform', val)}
              >
                <SelectTrigger className="w-full">
                  <SelectValue placeholder={t('add_game_select_placeholder')} />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="Steam">
                    {t('add_game_platform_steam')}
                  </SelectItem>
                  <SelectItem value="Epic">
                    {t('add_game_platform_epic_games')}
                  </SelectItem>
                  <SelectItem value="GOG">
                    {t('add_game_platform_gog_galaxy')}
                  </SelectItem>
                  <SelectItem value="Amazon">
                    {t('add_game_platform_amazon_games')}
                  </SelectItem>
                  <SelectItem value="Ubisoft">
                    {t('add_game_platform_ubisoft')}
                  </SelectItem>
                  <SelectItem value="EA">
                    {t('add_game_platform_ea')}
                  </SelectItem>
                  <SelectItem value="Battle.net">
                    {t('add_game_platform_battlenet')}
                  </SelectItem>
                  <SelectItem value={'Xbox'}>
                    {t('add_game_platform_xbox')}
                  </SelectItem>
                  <SelectItem value="Indiegala">
                    {t('add_game_platform_indiegala')}
                  </SelectItem>
                  <SelectItem value={'Itch.io'}>
                    {t('add_game_platform_itch')}
                  </SelectItem>
                  <SelectItem value="Indie">
                    {t('add_game_platform_indie')}
                  </SelectItem>
                  <SelectItem value="Outra">
                    {t('add_game_platform_other')}
                  </SelectItem>
                </SelectContent>
              </Select>
            </div>
            <div className="grid gap-2">
              <Label>{t('add_game_installed_label')}</Label>
              <div className="border-input bg-background flex w-full items-center gap-2 rounded-md border px-3 py-2">
                <input
                  type="checkbox"
                  id="installed"
                  checked={formData.installed}
                  onChange={e => handleChange('installed', e.target.checked)}
                  className="h-4 w-4 cursor-pointer"
                />
                <label htmlFor="installed" className="cursor-pointer text-sm">
                  {formData.installed
                    ? t('add_game_installed_yes')
                    : t('add_game_installed_no')}
                </label>
              </div>
            </div>
            <div className="grid gap-2">
              <Label>{t('add_game_import_confidence_label')}</Label>
              <Select
                value={formData.importConfidence || 'none'}
                onValueChange={val =>
                  handleChange('importConfidence', val === 'none' ? '' : val)
                }
              >
                <SelectTrigger className="w-full">
                  <SelectValue
                    placeholder={t('add_game_import_confidence_none')}
                  />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="none">
                    {t('add_game_import_confidence_none')}
                  </SelectItem>
                  <SelectItem value="High">
                    {t('add_game_import_confidence_high')}
                  </SelectItem>
                  <SelectItem value="Medium">
                    {t('add_game_import_confidence_medium')}
                  </SelectItem>
                  <SelectItem value="Low">
                    {t('add_game_import_confidence_low')}
                  </SelectItem>
                </SelectContent>
              </Select>
            </div>
          </div>

          <Separator className="my-2" />

          {/* === EXECUÇÃO === */}
          <h3 className="text-muted-foreground flex items-center gap-2 text-sm font-semibold">
            <Gamepad2 size={16} /> {t('add_game_launch_configuration_section')}
          </h3>

          <div className="grid gap-2">
            <Label htmlFor="installPath">
              {t('add_game_install_path_label')}
            </Label>
            <div className="flex gap-2">
              <Input
                id="installPath"
                value={formData.installPath}
                onChange={e => handleChange('installPath', e.target.value)}
                placeholder={t('add_game_install_path_placeholder')}
              />
              <Button
                type="button"
                variant="outline"
                size="icon"
                onClick={handleSelectFolder}
                title={t('add_game_select_folder_button_title')}
              >
                <FolderOpen size={18} />
              </Button>
            </div>
          </div>

          <div className="grid gap-2">
            <Label htmlFor="execPath">
              {t('add_game_executable_path_label')}
            </Label>
            <div className="flex gap-2">
              <Input
                id="execPath"
                value={formData.executablePath}
                onChange={e => handleChange('executablePath', e.target.value)}
                placeholder={t('add_game_executable_path_placeholder')}
              />
              <Button
                type="button"
                variant="outline"
                size="icon"
                onClick={handleSelectExecutable}
                title={t('add_game_select_executable_button_title')}
              >
                <FileSearch size={18} />
              </Button>
            </div>
          </div>

          <div className="grid gap-2">
            <Label htmlFor="launchArgs">
              {t('add_game_launch_args_label')}
            </Label>
            <Input
              id="launchArgs"
              value={formData.launchArgs}
              onChange={e => handleChange('launchArgs', e.target.value)}
              placeholder={t('add_game_launch_args_placeholder')}
            />
          </div>

          <Separator className="my-2" />

          {/* === AVALIAÇÃO === */}
          <div className="grid grid-cols-3 items-end gap-4">
            <div className="grid gap-2">
              <Label htmlFor="playtime">{t('add_game_playtime_label')}</Label>
              <Input
                id="playtime"
                type="number"
                min="0"
                step="0.1"
                value={
                  formData.playtime
                    ? String(
                        Math.round((Number(formData.playtime) / 60) * 100) / 100
                      )
                    : ''
                }
                onChange={e => {
                  const hours = Number.parseFloat(e.target.value);
                  const minutes = Number.isNaN(hours)
                    ? 0
                    : Math.floor(hours * 60);
                  handleChange('playtime', String(minutes));
                }}
              />
            </div>

            <div className="grid gap-1">
              <Label>{t('add_game_status_label')}</Label>
              <Select
                value={formData.status}
                onValueChange={val => handleChange('status', val)}
              >
                <SelectTrigger className="w-full">
                  <SelectValue placeholder={t('add_game_select_placeholder')} />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="playing">
                    {t('add_game_status_playing')}
                  </SelectItem>
                  <SelectItem value="backlog">
                    {t('add_game_status_backlog')}
                  </SelectItem>
                  <SelectItem value="completed">
                    {t('add_game_status_completed')}
                  </SelectItem>
                  <SelectItem value="abandoned">
                    {t('add_game_status_abandoned')}
                  </SelectItem>
                  <SelectItem value="plan_to_play">
                    {t('add_game_status_plan_to_play')}
                  </SelectItem>
                </SelectContent>
              </Select>
            </div>

            <div className="grid gap-1">
              <Label>{t('add_game_rating_label')}</Label>
              <div className="flex h-10 items-center gap-1">
                {[1, 2, 3, 4, 5].map(star => (
                  <button
                    key={star}
                    type="button"
                    onClick={() => handleChange('rating', star)}
                    className={`transition-all hover:scale-110 focus:outline-none ${
                      star <= formData.rating
                        ? 'text-yellow-400'
                        : 'text-muted-foreground/30'
                    }`}
                  >
                    <Star
                      size={24}
                      fill={star <= formData.rating ? 'currentColor' : 'none'}
                    />
                  </button>
                ))}
              </div>
            </div>
          </div>
        </div>

        <DialogFooter>
          <Button variant="outline" onClick={onClose}>
            {t('add_game_cancel_button')}
          </Button>
          <Button onClick={handleSave}>{t('add_game_save_button')}</Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}

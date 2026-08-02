import { open } from '@tauri-apps/plugin-shell';

import i18n from '@/i18n';
import { toast } from '@/utils/toast';

export const openExternalLink = async (url: string): Promise<void> => {
  try {
    await open(url);
  } catch (error) {
    console.error('Erro ao abrir link:', error);
    toast.error(i18n.t('errors:error_msg_open_link_failed'));
  }
};

import { useState } from 'react';
import { useTranslation } from 'react-i18next';

import { addGameFromScan } from '@/services/scannerService';
import { ExecutableCandidate, GameDiscovery } from '@/types/scanner';
import { toast } from '@/utils/toast';

/**
 * Hook para gerenciar a seleção e o salvamento de um executável descoberto
 * pelo scanner local (usado pelo modal ExecutableSelection).
 *
 * @param rootFolderPath - Caminho da pasta raiz onde os jogos foram encontrados
 * @param discovery - Descoberta de jogo cujo executável será selecionado
 * @param onSuccess - Callback chamado após adicionar o jogo com sucesso (ex: fechar o modal)
 * @returns Estado de salvamento e função de seleção
 */
export function useExecutableSelection(
  rootFolderPath: string,
  discovery: GameDiscovery,
  onSuccess: () => void
) {
  const { t } = useTranslation('platforms');
  const [isSaving, setIsSaving] = useState(false);

  const handleSelect = async (executable: ExecutableCandidate) => {
    setIsSaving(true);

    try {
      await addGameFromScan(rootFolderPath, discovery, executable);
      toast.success(
        t('executable_added_to_library', { name: discovery.suggestedName })
      );
      onSuccess();
    } catch (error) {
      toast.error(
        typeof error === 'string' ? error : t('executable_save_failed')
      );
      console.error('Erro ao adicionar jogo:', error);
    } finally {
      setIsSaving(false);
    }
  };

  return { isSaving, handleSelect };
}

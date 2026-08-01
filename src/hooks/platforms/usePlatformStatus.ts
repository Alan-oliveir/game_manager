import { useEffect, useState } from 'react';

import type { ImportStatus } from '@/types';

const AUTO_DISMISS_MS = 5000;

/**
 * Gerencia o estado de status (sucesso/erro) exibido nas telas de
 * configuração de plataformas, com dismiss automático após alguns segundos.
 *
 * Extraído do antigo `useStoresConfig`, onde essa lógica era duplicada
 * implicitamente para todas as plataformas através de um único estado
 * compartilhado.
 */
export function usePlatformStatus() {
  const [status, setStatus] = useState<ImportStatus>({
    type: null,
    message: '',
  });

  useEffect(() => {
    if (!status.type || !status.message) return;

    const timer = setTimeout(() => {
      setStatus({ type: null, message: '' });
    }, AUTO_DISMISS_MS);

    return () => clearTimeout(timer);
  }, [status]);

  return { status, setStatus };
}

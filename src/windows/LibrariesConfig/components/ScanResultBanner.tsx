import { cn } from '@/lib/utils';
import { Button } from '@/ui/button';

interface ScanResultBannerProps {
  success: boolean;
  message: string;
  onAddAll?: () => void;
  addAllDisabled?: boolean;
  addAllLabel?: string;
}

/**
 * Banner de resultado do scan local (verde em sucesso / vermelho em erro),
 * com botão opcional "adicionar todos" quando há descobertas pendentes.
 * `onAddAll` fica de fora quando não há descobertas a adicionar em lote.
 */
export function ScanResultBanner({
  success,
  message,
  onAddAll,
  addAllDisabled,
  addAllLabel,
}: Readonly<ScanResultBannerProps>) {
  return (
    <div
      className={cn(
        'mb-6 flex items-center justify-between rounded-lg border p-4 text-sm',
        success
          ? 'border-green-500/20 bg-green-500/5 text-green-400'
          : 'border-red-500/20 bg-red-500/5 text-red-400'
      )}
    >
      <div className="flex items-center gap-3">
        <div
          className={cn(
            'h-2 w-2 rounded-full',
            success ? 'bg-green-500' : 'bg-red-500'
          )}
        />
        {message}
      </div>
      {onAddAll && addAllLabel && (
        <Button onClick={onAddAll} disabled={addAllDisabled} size="sm">
          {addAllLabel}
        </Button>
      )}
    </div>
  );
}

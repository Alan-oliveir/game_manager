import { FolderOpen } from 'lucide-react';

import { Button } from '@/ui/button';
import { Input } from '@/ui/input';

interface PathPickerFieldProps {
  value: string;
  onChange: (value: string) => void;
  onBrowse: () => void;
  placeholder: string;
  browseLabel: string;
  ariaLabel?: string;
  showPreview?: boolean;
}

/**
 * Campo de caminho editável com botão de "procurar" (abre diálogo nativo)
 * e preview opcional do valor selecionado abaixo. Usado por Steam, Heroic,
 * Legacy Games e Wine — qualquer plataforma que aceite um caminho manual.
 */
export function PathPickerField({
  value,
  onChange,
  onBrowse,
  placeholder,
  browseLabel,
  ariaLabel,
  showPreview = true,
}: Readonly<PathPickerFieldProps>) {
  return (
    <div className="flex flex-col gap-2">
      <div className="flex items-center gap-2">
        <Input
          type="text"
          value={value}
          onChange={e => onChange(e.target.value)}
          placeholder={placeholder}
          aria-label={ariaLabel}
          className="bg-background/50 font-mono text-xs"
        />
        <Button
          variant="outline"
          size="sm"
          onClick={onBrowse}
          className="shrink-0 text-xs"
        >
          <FolderOpen className="mr-1 h-3 w-3" />
          {browseLabel}
        </Button>
      </div>
      {showPreview && value && (
        <code className="text-muted-foreground bg-secondary/40 truncate rounded-md border px-2 py-1.5 text-[10px]">
          {value}
        </code>
      )}
    </div>
  );
}

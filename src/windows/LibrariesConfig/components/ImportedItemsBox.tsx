import type { ReactNode } from 'react';

interface ImportedItemsBoxProps {
  title: ReactNode;
  items: ReactNode[];
  note?: ReactNode;
}

/**
 * Caixa "o que será importado" / "plataformas suportadas", usada por Epic,
 * Ubisoft, Legacy Games e Heroic.
 */
export function ImportedItemsBox({
  title,
  items,
  note,
}: Readonly<ImportedItemsBoxProps>) {
  return (
    <div className="border-white-500/20 bg-muted/20 rounded-lg border p-4">
      <h4 className="mb-2 text-sm font-semibold">{title}</h4>
      <ul className="text-muted-foreground space-y-1 text-xs">
        {items.map((item, index) => (
          <li key={index}> ✓ {item}</li>
        ))}
      </ul>
      {note && <p className="text-muted-foreground mt-4 text-xs">{note}</p>}
    </div>
  );
}

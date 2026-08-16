import type { ReactNode } from 'react';

interface PathEntry {
  label: ReactNode;
  path: string;
}

interface DetectedPathsBoxProps {
  intro?: ReactNode;
  paths: PathEntry[];
  note?: ReactNode;
}

/**
 * Caixa "caminhos verificados automaticamente", usada por Epic e Heroic
 * para listar onde o app procura pela instalação/config de cada launcher
 * (Windows, Linux nativo, Linux via Wine, Flatpak, etc).
 */
export function DetectedPathsBox({
  intro,
  paths,
  note,
}: Readonly<DetectedPathsBoxProps>) {
  return (
    <div className="bg-muted/30 space-y-2 rounded-md border p-3">
      {intro && (
        <p className="text-muted-foreground text-xs font-medium">{intro}</p>
      )}
      <div className="space-y-1">
        {paths.map((entry, index) => (
          <div key={index}>
            <p
              className={
                index > 0
                  ? 'text-muted-foreground pt-1 text-xs'
                  : 'text-muted-foreground text-xs'
              }
            >
              <span className="text-primary/60 font-semibold">
                {entry.label}
              </span>
            </p>
            <code className="text-primary/80 block pl-2 text-xs">
              {entry.path}
            </code>
          </div>
        ))}
      </div>
      {note && <p className="text-muted-foreground mt-2 text-xs">{note}</p>}
    </div>
  );
}

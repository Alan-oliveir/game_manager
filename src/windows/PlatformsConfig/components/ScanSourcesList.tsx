import {
  Check,
  Database,
  FolderOpen,
  Pencil,
  RefreshCw,
  Trash2,
  X,
} from 'lucide-react';
import { useState } from 'react';
import { useTranslation } from 'react-i18next';

import { useScanSources } from '@/hooks/configuration';
import { cn } from '@/lib/utils';
import { ScanSourceInfo } from '@/types/scanner';
import { Badge } from '@/ui/badge';
import { Button } from '@/ui/button';
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/ui/dialog';
import { Input } from '@/ui/input';
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/ui/table';

export function ScanSourcesList() {
  const { t } = useTranslation('platforms');
  const { sources, loading, refresh, rename, remove } = useScanSources();

  const [editingId, setEditingId] = useState<string | null>(null);
  const [editValue, setEditValue] = useState('');
  const [deleteTarget, setDeleteTarget] = useState<ScanSourceInfo | null>(null);
  const [removeGamesToo, setRemoveGamesToo] = useState(false);

  const startEditing = (source: ScanSourceInfo) => {
    setEditingId(source.id);
    setEditValue(source.label);
  };

  const cancelEditing = () => {
    setEditingId(null);
    setEditValue('');
  };

  const confirmEditing = async (id: string) => {
    await rename(id, editValue);
    cancelEditing();
  };

  const openDeleteDialog = (source: ScanSourceInfo) => {
    setDeleteTarget(source);
    setRemoveGamesToo(false);
  };

  const confirmDelete = async () => {
    if (!deleteTarget) return;

    await remove(deleteTarget.id, removeGamesToo);
    setDeleteTarget(null);
  };

  const RefreshButton = (
    <Button
      size="icon"
      variant="secondary"
      className="h-8 w-8 shrink-0"
      disabled={loading}
      onClick={() => refresh()}
      aria-label={t('scanner_sources_refresh')}
    >
      <RefreshCw size={14} className={cn(loading && 'animate-spin')} />
    </Button>
  );

  // Componente de Cabeçalho (Título + Botão na mesma linha)
  const HeaderSection = () => (
    <div className="mb-4 flex items-start justify-between border-b pb-2">
      <div>
        <h2 className="text-2xl font-bold">{t('scanner_sources_title')}</h2>
        <p className="text-muted-foreground mt-1 text-sm">
          {t('scanner_sources_description')}
        </p>
      </div>
      <div className="pt-1">{RefreshButton}</div>
    </div>
  );

  if (loading && sources.length === 0) {
    return (
      <>
        <HeaderSection />
        <div className="flex items-center justify-center">
          <p className="text-muted-foreground py-8 text-center text-sm">
            {t('scanner_sources_loading')}
          </p>
        </div>
      </>
    );
  }

  if (sources.length === 0) {
    return (
      <div className="space-y-3">
        <HeaderSection />
        <div className="border-border/40 flex flex-col items-center justify-center rounded-lg border border-dashed p-8 text-center">
          <Database className="text-muted-foreground/30 mb-3 h-10 w-10" />
          <p className="text-muted-foreground text-sm">
            {t('scanner_sources_empty')}
          </p>
        </div>
      </div>
    );
  }

  return (
    <>
      {/* O título integrado nesta seção */}
      <HeaderSection />

      <Table>
        <TableHeader>
          <TableRow className="border-none hover:bg-transparent">
            <TableHead className="text-muted-foreground/80 text-sm font-bold tracking-widest uppercase">
              {t('scanner_sources_label')}
            </TableHead>
            <TableHead className="text-muted-foreground/80 text-sm font-bold tracking-widest uppercase">
              {t('scanner_sources_path')}
            </TableHead>
            <TableHead className="text-muted-foreground/80 text-sm font-bold tracking-widest uppercase">
              {t('scanner_sources_games')}
            </TableHead>
            <TableHead className="text-muted-foreground/80 text-right text-sm font-bold tracking-widest uppercase">
              {t('scanner_sources_actions')}
            </TableHead>
          </TableRow>
        </TableHeader>
        <TableBody>
          {sources.map(source => (
            <TableRow key={source.id}>
              <TableCell className="font-medium">
                {editingId === source.id ? (
                  <div className="flex items-center gap-2">
                    <Input
                      value={editValue}
                      onChange={e => setEditValue(e.target.value)}
                      className="h-8"
                      autoFocus
                      onKeyDown={e => {
                        if (e.key === 'Enter') confirmEditing(source.id);

                        if (e.key === 'Escape') cancelEditing();
                      }}
                    />
                    <Button
                      size="icon"
                      variant="ghost"
                      className="h-8 w-8 shrink-0"
                      onClick={() => confirmEditing(source.id)}
                    >
                      <Check size={14} />
                    </Button>
                    <Button
                      size="icon"
                      variant="ghost"
                      className="h-8 w-8 shrink-0"
                      onClick={cancelEditing}
                    >
                      <X size={14} />
                    </Button>
                  </div>
                ) : (
                  source.label
                )}
              </TableCell>
              <TableCell>
                <div className="text-muted-foreground flex items-center gap-2 font-mono text-xs">
                  <FolderOpen size={12} className="shrink-0 opacity-60" />
                  <span className="max-w-xs truncate">{source.folderPath}</span>
                </div>
              </TableCell>
              <TableCell>
                <Badge variant="outline" className="font-mono text-xs">
                  {source.gameCount}
                </Badge>
              </TableCell>
              <TableCell className="text-right">
                <div className="flex justify-end gap-1">
                  <Button
                    size="icon"
                    variant="ghost"
                    className="h-8 w-8"
                    disabled={editingId === source.id}
                    onClick={() => startEditing(source)}
                  >
                    <Pencil size={14} />
                  </Button>
                  <Button
                    size="icon"
                    variant="ghost"
                    className="text-destructive hover:text-destructive h-8 w-8"
                    onClick={() => openDeleteDialog(source)}
                  >
                    <Trash2 size={14} />
                  </Button>
                </div>
              </TableCell>
            </TableRow>
          ))}
        </TableBody>
      </Table>

      <Dialog
        open={!!deleteTarget}
        onOpenChange={open => !open && setDeleteTarget(null)}
      >
        <DialogContent className="max-w-md">
          <DialogHeader>
            <DialogTitle>{t('scanner_source_delete_title')}</DialogTitle>
            <DialogDescription>
              {t('scanner_source_delete_description', {
                label: deleteTarget?.label,
              })}
            </DialogDescription>
          </DialogHeader>

          <label className="flex items-center gap-2 text-sm">
            <input
              type="checkbox"
              checked={removeGamesToo}
              onChange={e => setRemoveGamesToo(e.target.checked)}
              className="accent-destructive h-4 w-4"
            />
            {t('scanner_source_delete_remove_games', {
              count: deleteTarget?.gameCount ?? 0,
            })}
          </label>

          <DialogFooter>
            <Button variant="outline" onClick={() => setDeleteTarget(null)}>
              {t('cancel', { ns: 'common' })}
            </Button>
            <Button variant="destructive" onClick={confirmDelete}>
              {t('scanner_source_delete_confirm')}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </>
  );
}

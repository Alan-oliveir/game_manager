import { Cpu, FileText, HardDrive, MousePointer2, Star } from 'lucide-react';
import { useMemo } from 'react';
import { useTranslation } from 'react-i18next';

import { useExecutableSelection } from '@/hooks/configuration';
import { formatFileSize } from '@/services/scannerService';
import { GameDiscovery } from '@/types/scanner';
import { Badge } from '@/ui/badge';
import { Button } from '@/ui/button';
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
} from '@/ui/dialog';
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/ui/table';

interface ExecutableSelectionProps {
  open: boolean;
  onClose: () => void;
  discovery: GameDiscovery;
}

export function ExecutableSelection({
  open,
  onClose,
  discovery,
}: Readonly<ExecutableSelectionProps>) {
  const { t } = useTranslation('platforms');
  const { isSaving, handleSelect } = useExecutableSelection(discovery, onClose);

  const sortedExecutables = useMemo(
    () => [...discovery.executables].sort((a, b) => b.rankScore - a.rankScore),
    [discovery.executables]
  );

  return (
    <Dialog open={open} onOpenChange={onClose}>
      <DialogContent className="border-border/40 flex h-[85vh] w-[95vw] max-w-6xl flex-col gap-0 overflow-hidden p-0 shadow-2xl sm:max-w-6xl">
        <DialogHeader className="bg-muted/10 border-b p-6">
          <div className="flex items-center gap-4">
            <div className="bg-primary/10 rounded-lg p-3">
              <FileText className="text-primary h-6 w-6" />
            </div>
            <div className="flex flex-col gap-1">
              <DialogTitle className="text-2xl leading-none font-bold tracking-tight">
                {discovery.suggestedName}
              </DialogTitle>
              <DialogDescription className="text-sm">
                {t('executable_dialog_description')}
              </DialogDescription>
            </div>
          </div>
        </DialogHeader>

        <div className="custom-scrollbar min-h-0 flex-1 overflow-y-auto">
          <Table>
            <TableHeader className="bg-muted/30 sticky top-0 z-10 backdrop-blur-md">
              <TableRow className="border-none hover:bg-transparent">
                <TableHead className="text-muted-foreground/80 h-12 pl-8 text-sm font-bold tracking-widest uppercase">
                  {t('executable_table_executable_path')}
                </TableHead>
                <TableHead className="text-muted-foreground/80 h-12 text-sm font-bold tracking-widest uppercase">
                  {t('executable_table_size')}
                </TableHead>
                <TableHead className="text-muted-foreground/80 h-12 text-sm font-bold tracking-widest uppercase">
                  {t('executable_table_architecture')}
                </TableHead>
                <TableHead className="text-muted-foreground/80 h-12 pr-8 text-right text-sm font-bold tracking-widest uppercase">
                  {t('executable_table_action')}
                </TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {sortedExecutables.map((exe, idx) => (
                <TableRow key={exe.path}>
                  <TableCell className="py-4 pl-8">
                    <div className="flex items-start gap-3">
                      {idx === 0 ? (
                        <Star className="mt-1 h-4 w-4 shrink-0 fill-yellow-500 text-yellow-500" />
                      ) : (
                        <FileText className="text-muted-foreground mt-1 h-4 w-4 shrink-0" />
                      )}
                      <div className="flex min-w-0 flex-col gap-0.5">
                        <span className="text-xl leading-tight font-bold">
                          {exe.filename}
                          {idx === 0 && (
                            <Badge
                              variant="outline"
                              className="border-primary/30 text-primary ml-2 h-4 text-xs font-black uppercase"
                            >
                              {t('executable_suggestion')}
                            </Badge>
                          )}
                        </span>
                        <span className="text-muted-foreground max-w-2xl truncate font-mono text-xs opacity-60">
                          {exe.path}
                        </span>
                      </div>
                    </div>
                  </TableCell>
                  <TableCell>
                    <div className="flex items-center gap-2 text-sm font-medium">
                      <HardDrive
                        size={14}
                        className="text-muted-foreground opacity-70"
                      />
                      {formatFileSize(exe.sizeMb)}
                    </div>
                  </TableCell>
                  <TableCell>
                    <Badge
                      variant="outline"
                      className="bg-background/50 border-border/50 font-mono text-xs font-semibold uppercase"
                    >
                      <Cpu size={12} className="mr-1.5 opacity-60" />
                      {exe.executableType}
                    </Badge>
                  </TableCell>
                  <TableCell className="pr-8 text-right">
                    <Button
                      size="sm"
                      disabled={isSaving}
                      variant={idx === 0 ? 'default' : 'outline'}
                      onClick={() => handleSelect(exe)}
                      className="h-8 font-bold"
                    >
                      <MousePointer2 size={14} className="mr-2" />
                      {isSaving
                        ? t('executable_saving')
                        : t('executable_select')}
                    </Button>
                  </TableCell>
                </TableRow>
              ))}
            </TableBody>
          </Table>
        </div>

        <div className="bg-muted/5 text-muted-foreground border-t p-3 px-8 text-xs italic opacity-70">
          {t('executable_tip')}
        </div>
      </DialogContent>
    </Dialog>
  );
}

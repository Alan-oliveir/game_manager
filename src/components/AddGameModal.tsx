import { useState } from "react";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogFooter,
} from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";

interface AddGameModalProps {
  isOpen: boolean;
  onClose: () => void;
  onSave: (name: string, coverUrl: string) => void;
}

export default function AddGameModal({
  isOpen,
  onClose,
  onSave,
}: AddGameModalProps) {
  const [name, setName] = useState("");
  const [coverUrl, setCoverUrl] = useState("");

  const handleSave = () => {
    if (!name.trim()) return;
    onSave(name, coverUrl);

    // Limpa o form e fecha
    setName("");
    setCoverUrl("");
    onClose();
  };

  return (
    <Dialog open={isOpen} onOpenChange={onClose}>
      <DialogContent className="sm:max-w-[425px]">
        <DialogHeader>
          <DialogTitle>Adicionar Novo Jogo</DialogTitle>
        </DialogHeader>

        <div className="grid gap-4 py-4">
          {/* Nome do Jogo */}
          <div className="grid gap-2">
            <Label htmlFor="name">Nome do Jogo</Label>
            <Input
              id="name"
              value={name}
              onChange={(e) => setName(e.target.value)}
              placeholder="Ex: The Witcher 3"
              className="col-span-3"
            />
          </div>

          {/* URL da Capa */}
          <div className="grid gap-2">
            <Label htmlFor="cover">URL da Capa (Opcional)</Label>
            <Input
              id="cover"
              value={coverUrl}
              onChange={(e) => setCoverUrl(e.target.value)}
              placeholder="https://..."
              className="col-span-3"
            />
            <p className="text-[0.8rem] text-muted-foreground">
              Dica: Copie o endereço da imagem do Google Imagens ou SteamDB.
            </p>
          </div>
        </div>

        <DialogFooter>
          <Button variant="outline" onClick={onClose}>
            Cancelar
          </Button>
          <Button onClick={handleSave} disabled={!name.trim()}>
            Salvar Jogo
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}

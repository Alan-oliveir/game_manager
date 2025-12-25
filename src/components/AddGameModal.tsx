import { useState, useEffect } from "react";
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
import { Game } from "../types";

interface AddGameModalProps {
  isOpen: boolean;
  onClose: () => void;
  onSave: (name: string, coverUrl: string) => void;
  gameToEdit?: Game | null;
}

export default function AddGameModal({
  isOpen,
  onClose,
  onSave,
  gameToEdit,
}: AddGameModalProps) {
  const [name, setName] = useState("");
  const [coverUrl, setCoverUrl] = useState("");

  // Reseta os campos quando o modal é aberto
  useEffect(() => {
    if (isOpen) {
      if (gameToEdit) {
        // Modo Edição: Preenche com os dados do jogo
        setName(gameToEdit.name);
        setCoverUrl(gameToEdit.cover_url || "");
      } else {
        // Modo Criação: Limpa tudo
        setName("");
        setCoverUrl("");
      }
    }
  }, [isOpen, gameToEdit]);

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
            <DialogTitle>
              {gameToEdit ? "Editar Jogo" : "Adicionar Novo Jogo"}
            </DialogTitle>
          </DialogHeader>

          <div className="grid gap-4 py-4">
            <div className="grid gap-2">
              <Label htmlFor="name">Nome do Jogo</Label>
              <Input
                  id="name"
                  value={name}
                  onChange={(e) => setName(e.target.value)}
                  placeholder="Ex: The Witcher 3"
              />
            </div>
            <div className="grid gap-2">
              <Label htmlFor="cover">URL da Capa</Label>
              <Input
                  id="cover"
                  value={coverUrl}
                  onChange={(e) => setCoverUrl(e.target.value)}
                  placeholder="https://..."
              />
            </div>
          </div>

          <DialogFooter>
            <Button variant="outline" onClick={onClose}>Cancelar</Button>
            <Button onClick={handleSave} disabled={!name.trim()}>
              {gameToEdit ? "Salvar Alterações" : "Adicionar Jogo"}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
  );
}
import { useEffect, useState } from "react";
import {
  Dialog,
  DialogContent,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { Star } from "lucide-react";
import { Game } from "../types";

interface AddGameModalProps {
  isOpen: boolean;
  onClose: () => void;
  onSave: (gameData: Partial<Game>) => void;
  gameToEdit?: Game | null;
}

export default function AddGameModal({
  isOpen,
  onClose,
  onSave,
  gameToEdit,
}: AddGameModalProps) {
  // Estados do Formulário
  const [name, setName] = useState("");
  const [coverUrl, setCoverUrl] = useState("");
  const [genre, setGenre] = useState("");
  const [platform, setPlatform] = useState("Manual");
  const [playtime, setPlaytime] = useState("0");
  const [rating, setRating] = useState(0);

  // Carrega dados ao abrir (Edição vs Criação)
  useEffect(() => {
    if (isOpen) {
      if (gameToEdit) {
        setName(gameToEdit.name);
        setCoverUrl(gameToEdit.cover_url || "");
        setGenre(gameToEdit.genre || "");
        setPlatform(gameToEdit.platform || "Manual");
        setPlaytime(gameToEdit.playtime?.toString() || "0");
        setRating(gameToEdit.rating || 0);
      } else {
        setName("");
        setCoverUrl("");
        setGenre("");
        setPlatform("Manual");
        setPlaytime("0");
        setRating(0);
      }
    }
  }, [isOpen, gameToEdit]);

  const handleSave = () => {
    if (!name.trim()) return;

    onSave({
      name,
      cover_url: coverUrl,
      genre,
      platform,
      playtime: parseInt(playtime) || 0,
      rating: rating > 0 ? rating : undefined,
    });

    onClose();
  };

  return (
    <Dialog open={isOpen} onOpenChange={onClose}>
      <DialogContent className="sm:max-w-[500px]">
        <DialogHeader>
          <DialogTitle>
            {gameToEdit ? "Editar Jogo" : "Adicionar Jogo"}
          </DialogTitle>
        </DialogHeader>

        <div className="grid gap-4 py-4">
          {/* Linha 1: Nome */}
          <div className="grid gap-2">
            <Label htmlFor="name">Nome do Jogo *</Label>
            <Input
              id="name"
              value={name}
              onChange={(e) => setName(e.target.value)}
              placeholder="Ex: Elden Ring"
            />
          </div>

          {/* Linha 2: Gênero e Plataforma */}
          <div className="grid grid-cols-2 gap-4">
            <div className="grid gap-2">
              <Label htmlFor="genre">Gênero</Label>
              <Input
                id="genre"
                value={genre}
                onChange={(e) => setGenre(e.target.value)}
                placeholder="RPG, Ação..."
              />
            </div>
            <div className="grid gap-2">
              <Label>Plataforma</Label>
              <Select value={platform} onValueChange={setPlatform}>
                <SelectTrigger>
                  <SelectValue placeholder="Selecione" />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="Manual">Outro / Manual</SelectItem>
                  <SelectItem value="Steam">Steam</SelectItem>
                  <SelectItem value="Epic">Epic Games</SelectItem>
                  <SelectItem value="GOG">GOG Galaxy</SelectItem>
                  <SelectItem value="Xbox">Xbox App</SelectItem>
                </SelectContent>
              </Select>
            </div>
          </div>

          {/* Linha 3: URL da Capa */}
          <div className="grid gap-2">
            <Label htmlFor="cover">Capa (URL)</Label>
            <Input
              id="cover"
              value={coverUrl}
              onChange={(e) => setCoverUrl(e.target.value)}
              placeholder="https://..."
            />
          </div>

          {/* Linha 4: Tempo e Avaliação */}
          <div className="grid grid-cols-2 gap-4 items-end">
            <div className="grid gap-2">
              <Label htmlFor="playtime">Tempo Jogado (horas)</Label>
              <Input
                id="playtime"
                type="number"
                min="0"
                value={playtime}
                onChange={(e) => setPlaytime(e.target.value)}
              />
            </div>

            <div className="grid gap-2">
              <Label>Sua Avaliação</Label>
              <div className="flex gap-1 h-10 items-center">
                {[1, 2, 3, 4, 5].map((star) => (
                  <button
                    key={star}
                    onClick={() => setRating(star)}
                    className={`transition-all hover:scale-110 focus:outline-none ${
                      star <= rating
                        ? "text-yellow-400"
                        : "text-muted-foreground/30"
                    }`}
                    type="button"
                  >
                    <Star
                      size={24}
                      fill={star <= rating ? "currentColor" : "none"}
                    />
                  </button>
                ))}
              </div>
            </div>
          </div>
        </div>

        <DialogFooter>
          <Button variant="outline" onClick={onClose}>
            Cancelar
          </Button>
          <Button onClick={handleSave}>Salvar</Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}

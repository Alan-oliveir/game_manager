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

const INITIAL_STATE = {
  name: "",
  cover_url: "",
  genre: "",
  platform: "Manual",
  playtime: "0",
  rating: 0,
};

export default function AddGameModal({
  isOpen,
  onClose,
  onSave,
  gameToEdit,
}: AddGameModalProps) {
  const [formData, setFormData] = useState(INITIAL_STATE);

  useEffect(() => {
    if (isOpen) {
      if (gameToEdit) {
        setFormData({
          name: gameToEdit.name,
          cover_url: gameToEdit.cover_url || "",
          genre: gameToEdit.genre || "",
          platform: gameToEdit.platform || "Manual",
          playtime: gameToEdit.playtime?.toString() || "0",
          rating: gameToEdit.rating || 0,
        });
      } else {
        setFormData(INITIAL_STATE);
      }
    }
  }, [isOpen, gameToEdit]);

  const handleChange = (field: string, value: any) => {
    setFormData((prev) => ({ ...prev, [field]: value }));
  };

  const handleSave = () => {
    if (!formData.name.trim()) return;

    onSave({
      ...formData,
      playtime: parseInt(formData.playtime) || 0,
      rating: formData.rating > 0 ? formData.rating : undefined,
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
          <div className="grid gap-2">
            <Label htmlFor="name">Nome do Jogo *</Label>
            <Input
              id="name"
              value={formData.name}
              onChange={(e) => handleChange("name", e.target.value)}
              placeholder="Ex: Elden Ring"
            />
          </div>

          <div className="grid grid-cols-2 gap-4">
            <div className="grid gap-2">
              <Label htmlFor="genre">Gênero</Label>
              <Input
                id="genre"
                value={formData.genre}
                onChange={(e) => handleChange("genre", e.target.value)}
                placeholder="RPG, Ação..."
              />
            </div>
            <div className="grid gap-2">
              <Label>Plataforma</Label>
              <Select
                value={formData.platform}
                onValueChange={(val) => handleChange("platform", val)}
              >
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

          <div className="grid gap-2">
            <Label htmlFor="cover">Capa (URL)</Label>
            <Input
              id="cover"
              value={formData.cover_url}
              onChange={(e) => handleChange("cover_url", e.target.value)}
              placeholder="https://..."
            />
          </div>

          <div className="grid grid-cols-2 gap-4 items-end">
            <div className="grid gap-2">
              <Label htmlFor="playtime">Tempo Jogado (horas)</Label>
              <Input
                id="playtime"
                type="number"
                min="0"
                value={formData.playtime}
                onChange={(e) => handleChange("playtime", e.target.value)}
              />
            </div>

            <div className="grid gap-2">
              <Label>Sua Avaliação</Label>
              <div className="flex gap-1 h-10 items-center">
                {[1, 2, 3, 4, 5].map((star) => (
                  <button
                    key={star}
                    type="button"
                    onClick={() => handleChange("rating", star)}
                    className={`transition-all hover:scale-110 focus:outline-none ${
                      star <= formData.rating
                        ? "text-yellow-400"
                        : "text-muted-foreground/30"
                    }`}
                  >
                    <Star
                      size={24}
                      fill={star <= formData.rating ? "currentColor" : "none"}
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

import { invoke } from '@tauri-apps/api/core';
import { Languages, Loader2, Sparkles } from 'lucide-react';
import { useEffect, useState } from 'react';
import { useTranslation } from 'react-i18next';

import { cn } from '@/lib/utils';
import { GameDetails } from '@/types/game';
import { Button } from '@/ui/button';
import { Skeleton } from '@/ui/skeleton';
import { toast } from '@/utils/toast';

interface GameDescriptionProps {
  gameId: string;
  details: GameDetails | null;
  loading: boolean;
  onDescriptionUpdate?: (newPtBr: string) => void;
}

export function GameDescription({
  gameId,
  details,
  loading,
  onDescriptionUpdate,
}: GameDescriptionProps) {
  const { t } = useTranslation('game_detail');
  // Estado local para controlar qual idioma está sendo exibido
  const [activeLang, setActiveLang] = useState<'en' | 'pt'>('en');
  // Estado local para a tradução (inicia com o que veio do banco, mas pode ser atualizado)
  const [localPtBr, setLocalPtBr] = useState<string | undefined>(undefined);
  // Estado de carregamento da tradução
  const [isTranslating, setIsTranslating] = useState(false);

  // Sincroniza estado local quando os detalhes mudam (ao trocar de jogo)
  useEffect(() => {
    if (details?.description) {
      setLocalPtBr(details.description.descriptionPtbr);
      setActiveLang(details.description.descriptionPtbr ? 'pt' : 'en');
    }
  }, [details]);

  // Texto original resolvido usando optional chaining rigoroso (prioridade: summary > description > shortDescription > storyline)
  const originalText =
    details?.description?.summary ??
    details?.description?.description ??
    details?.description?.shortDescription ??
    details?.description?.storyline;

  if (loading) {
    return (
      <div className="space-y-4 p-1">
        <div className="mb-6 flex items-center justify-between">
          <Skeleton className="h-8 w-48" />
          <Skeleton className="h-8 w-24" />
        </div>
        <Skeleton className="h-4 w-full" />
        <Skeleton className="h-4 w-[90%]" />
        <Skeleton className="h-4 w-[95%]" />
        <Skeleton className="h-4 w-[80%]" />
      </div>
    );
  }

  if (!details || !details.description) {
    return (
      <div className="text-muted-foreground flex h-40 items-center justify-center">
        {t('description_select_game')}
      </div>
    );
  }

  const handleLanguageSwitch = async (targetLang: 'en' | 'pt') => {
    if (targetLang === 'en') {
      setActiveLang('en');

      return;
    }

    // Se usuário quer PT
    if (targetLang === 'pt') {
      // Cenário A: Já temos a tradução em memória
      if (localPtBr) {
        setActiveLang('pt');

        return;
      }

      // Cenário B: Precisamos traduzir (Chamar Rust)
      if (!originalText) {
        toast.error(t('description_no_original_text'));

        return;
      }

      setIsTranslating(true);

      try {
        const translatedText = await invoke<string>('translate_description', {
          gameId: gameId,
          text: originalText,
        });

        setLocalPtBr(translatedText);
        setActiveLang('pt');
        toast.success(t('description_translated_success'));

        if (onDescriptionUpdate) onDescriptionUpdate(translatedText);
      } catch (error) {
        console.error('Erro na tradução:', error);
        toast.error(t('description_translate_failed'));
      } finally {
        setIsTranslating(false);
      }
    }
  };

  // Decide qual texto mostrar
  const textToShow =
    activeLang === 'pt' && localPtBr
      ? localPtBr
      : originalText || t('description_no_description');

  return (
    <div className="pr-4">
      {/* CABEÇALHO: Título + Toggle */}
      <div className="mb-4 flex items-center justify-between">
        <h2 className="text-2xl font-bold tracking-tight">
          {t('description_header_title')}
        </h2>

        {/* Toggle de Idioma */}
        {originalText && (
          <div className="bg-muted border-border flex items-center rounded-lg border p-1">
            <Button
              variant="ghost"
              size="sm"
              onClick={() => handleLanguageSwitch('en')}
              className={cn(
                'h-7 rounded-md px-3 text-xs transition-all',
                activeLang === 'en'
                  ? 'bg-background text-foreground font-bold shadow-sm'
                  : 'text-muted-foreground hover:text-foreground'
              )}
            >
              {t('description_lang_en')}
            </Button>
            <Button
              variant="ghost"
              size="sm"
              onClick={() => handleLanguageSwitch('pt')}
              disabled={isTranslating}
              className={cn(
                'flex h-7 items-center gap-1.5 rounded-md px-3 text-xs transition-all',
                activeLang === 'pt'
                  ? 'border border-blue-500/20 bg-blue-500/10 font-bold text-blue-500'
                  : 'text-muted-foreground hover:text-blue-400'
              )}
            >
              {isTranslating ? (
                <Loader2 size={10} className="animate-spin" />
              ) : !localPtBr ? (
                <Sparkles size={10} />
              ) : (
                <Languages size={10} />
              )}
              {isTranslating
                ? t('description_generating')
                : t('description_lang_pt')}
            </Button>
          </div>
        )}
      </div>

      {/* CONTEÚDO */}
      <div className="text-foreground/90 pb-8 text-sm leading-relaxed transition-opacity duration-300 lg:text-base">
        <p className="text-secondary-foreground font-light whitespace-pre-line">
          {textToShow}
        </p>
      </div>
    </div>
  );
}

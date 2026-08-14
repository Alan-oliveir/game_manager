import { invoke } from '@tauri-apps/api/core';
import { ChevronDown, Languages, Loader2, Sparkles } from 'lucide-react';
import { useEffect, useState } from 'react';
import { useTranslation } from 'react-i18next';

import { cn } from '@/lib/utils';
import { GameDescriptionData, GameDetails } from '@/types/game';
import { Button } from '@/ui/button';
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuRadioGroup,
  DropdownMenuRadioItem,
  DropdownMenuTrigger,
} from '@/ui/dropdown-menu';
import { Skeleton } from '@/ui/skeleton';
import { toast } from '@/utils/toast';

interface DescriptionSection {
  key: 'summary' | 'storyline' | 'shortDescription' | 'description';
  label: string;
  original: string;
  translated?: string;
}

function resolveSections(
  desc: GameDescriptionData,
  t: (key: string) => string
): DescriptionSection[] {
  const sections: DescriptionSection[] = [];

  if (desc.summary) {
    sections.push({
      key: 'summary',
      label: t('description_section_summary'),
      original: desc.summary,
      translated: desc.summaryTranslated,
    });
  }

  if (desc.storyline) {
    sections.push({
      key: 'storyline',
      label: t('description_section_storyline'),
      original: desc.storyline,
      translated: desc.storylineTranslated,
    });
  }

  if (sections.length > 0) return sections;

  if (desc.shortDescription) {
    return [
      {
        key: 'shortDescription',
        label: '',
        original: desc.shortDescription,
        translated: desc.shortDescriptionTranslated,
      },
    ];
  }

  if (desc.description) {
    return [
      {
        key: 'description',
        label: '',
        original: desc.description,
        translated: desc.descriptionTranslated,
      },
    ];
  }

  return [];
}

interface GameDescriptionProps {
  gameId: string;
  details: GameDetails | null;
  loading: boolean;
  onDescriptionUpdate?: (updated: GameDescriptionData) => void;
}

export function GameDescription({
  gameId,
  details,
  loading,
  onDescriptionUpdate,
}: GameDescriptionProps) {
  const { t, i18n } = useTranslation('game_detail');
  const targetLang = i18n.language;

  const [view, setView] = useState<'original' | 'translated'>('original');
  const [localDescription, setLocalDescription] = useState<
    GameDescriptionData | undefined
  >(details?.description);
  const [isTranslating, setIsTranslating] = useState(false);

  useEffect(() => {
    setLocalDescription(details?.description);

    if (details?.description) {
      const sections = resolveSections(details.description, t);
      const hasFullTranslation =
        details.description.translatedLang === targetLang &&
        sections.every(s => s.translated);
      setView(hasFullTranslation ? 'translated' : 'original');

      return;
    }

    setView('original');
  }, [details?.description, targetLang, t]);

  const sections = localDescription ? resolveSections(localDescription, t) : [];
  const hasContent = sections.length > 0;
  const hasFullTranslation =
    localDescription?.translatedLang === targetLang &&
    sections.every(s => s.translated);

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

  const handleLanguageSwitch = async (target: 'original' | 'translated') => {
    if (target === 'original') {
      setView('original');

      return;
    }

    if (hasFullTranslation) {
      setView('translated');

      return;
    }

    if (!hasContent) {
      toast.error(t('description_no_original_text'));

      return;
    }

    setIsTranslating(true);

    try {
      const updated = await invoke<GameDescriptionData>(
        'translate_description',
        { gameId, targetLang }
      );

      setLocalDescription(updated);
      setView('translated');
      toast.success(t('description_translated_success'));

      if (onDescriptionUpdate) onDescriptionUpdate(updated);
    } catch (error) {
      console.error('Erro na tradução:', error);
      toast.error(t('description_translate_failed'));
    } finally {
      setIsTranslating(false);
    }
  };

  return (
    <div className="pr-4">
      <div className="mb-4 flex items-center justify-between">
        <h2 className="text-2xl font-bold tracking-tight">
          {t('description_header_title')}
        </h2>

        {hasContent && (
          <DropdownMenu>
            <DropdownMenuTrigger asChild>
              <Button
                variant="outline"
                size="sm"
                disabled={isTranslating}
                className={cn(
                  'h-8 gap-1.5 text-xs',
                  view === 'translated' &&
                    'border-blue-500/20 bg-blue-500/10 text-blue-500'
                )}
              >
                {isTranslating ? (
                  <Loader2 size={12} className="animate-spin" />
                ) : view === 'translated' ? (
                  <Languages size={12} />
                ) : (
                  <Sparkles size={12} />
                )}
                {isTranslating
                  ? t('description_generating')
                  : view === 'translated'
                    ? t('description_lang_translated')
                    : t('description_lang_original')}
                <ChevronDown size={12} className="text-muted-foreground" />
              </Button>
            </DropdownMenuTrigger>

            <DropdownMenuContent align="end">
              <DropdownMenuRadioGroup
                value={view}
                onValueChange={value =>
                  handleLanguageSwitch(value as 'original' | 'translated')
                }
              >
                <DropdownMenuRadioItem value="original">
                  {t('description_lang_original')}
                </DropdownMenuRadioItem>
                <DropdownMenuRadioItem value="translated">
                  <span className="flex items-center gap-1.5">
                    {!hasFullTranslation && <Sparkles size={12} />}
                    {t('description_lang_translated')}
                  </span>
                </DropdownMenuRadioItem>
              </DropdownMenuRadioGroup>
            </DropdownMenuContent>
          </DropdownMenu>
        )}
      </div>

      <div className="text-foreground/90 pb-8 text-sm leading-relaxed transition-opacity duration-300 lg:text-base">
        {sections.map(section => {
          const text =
            view === 'translated' && section.translated
              ? section.translated
              : section.original;

          return (
            <div key={section.key} className="mb-6 last:mb-0">
              {section.label && (
                <h3 className="text-muted-foreground mb-2 text-xs font-semibold tracking-wide uppercase">
                  {section.label}
                </h3>
              )}
              <p className="text-secondary-foreground font-light whitespace-pre-line">
                {text}
              </p>
            </div>
          );
        })}
      </div>
    </div>
  );
}

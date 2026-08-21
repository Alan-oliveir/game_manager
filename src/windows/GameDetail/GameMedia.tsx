import { invoke } from '@tauri-apps/api/core';
import { ChevronLeft, ChevronRight, Meh, WifiOff } from 'lucide-react';
import React, { useCallback, useEffect, useRef, useState } from 'react';
import { useTranslation } from 'react-i18next';

import { ContentError, ContentLoading } from '@/components';
import {
  useMediaKeyboard,
  useMediaThumbnailScroll,
  useNetworkStatus,
} from '@/hooks';
import { MediaItem } from '@/types';
import { Game } from '@/types/game';
import { Lightbox, MediaThumbnail, MediaViewer } from '@/windows';

// === TIPOS ===

interface GameMediaData {
  screenshots: string[];
  trailers: string[];
  youtube_embeds: string[];
  micro_trailer: string | null;
}

interface GameMediaProps {
  game: Game;
}

// === HELPERS ===

function buildMediaQueue(data: GameMediaData): MediaItem[] {
  const queue: MediaItem[] = [];

  // Intercala: primeiro trailer, depois screenshots, depois mais trailers, depois YouTube
  // Objetivo: o primeiro item ser sempre vídeo se disponível
  const [firstTrailer, ...restTrailers] = data.trailers;

  if (firstTrailer) queue.push({ kind: 'trailer', url: firstTrailer });

  data.screenshots.forEach(url => queue.push({ kind: 'screenshot', url }));
  restTrailers.forEach(url => queue.push({ kind: 'trailer', url }));
  data.youtube_embeds.forEach(url => queue.push({ kind: 'youtube', url }));

  return queue;
}

function ScrollThumb({
  containerRef,
}: {
  containerRef: React.RefObject<HTMLDivElement | null>;
}) {
  const [thumb, setThumb] = useState({ left: 0, width: 0 });
  const [hovered, setHovered] = useState(false);

  useEffect(() => {
    const container = containerRef.current;

    if (!container) return;

    const update = () => {
      const { scrollLeft, scrollWidth, clientWidth } = container;
      const trackWidth = 100;
      const thumbWidth = Math.max((clientWidth / scrollWidth) * trackWidth, 10);
      const thumbLeft =
        (scrollLeft / (scrollWidth - clientWidth)) * (trackWidth - thumbWidth);
      setThumb({ left: thumbLeft, width: thumbWidth });
    };

    update();
    container.addEventListener('scroll', update);
    window.addEventListener('resize', update);

    return () => {
      container.removeEventListener('scroll', update);
      window.removeEventListener('resize', update);
    };
  }, [containerRef]);

  return (
    <div
      className="absolute top-1/2 h-6 -translate-y-1/2 rounded transition-colors duration-150"
      style={{
        left: `${thumb.left}%`,
        width: `${thumb.width}%`,
        backgroundColor: hovered
          ? 'var(--muted-foreground)'
          : 'color-mix(in oklch, var(--muted-foreground), transparent 45%)',
      }}
      onMouseEnter={() => setHovered(true)}
      onMouseLeave={() => setHovered(false)}
    />
  );
}

// === ESTADOS DE UI ===

function MediaEmpty({ gameName }: { gameName: string }) {
  const { t } = useTranslation('game_detail');

  return (
    <div className="flex h-64 flex-col items-center justify-center gap-3 text-center">
      <Meh className="text-muted-foreground/50 h-8 w-8" />
      <p className="text-foreground text-sm font-medium">
        {t('media_empty_title')}
      </p>
      <p className="text-muted-foreground max-w-xs text-xs">
        {t('media_empty_description')}{' '}
        <span className="text-foreground font-medium">{gameName}</span>.
      </p>
    </div>
  );
}

function MediaOffline() {
  const { t } = useTranslation('game_detail');

  return (
    <div className="flex h-64 flex-col items-center justify-center gap-4 text-center">
      <div className="bg-muted/40 rounded-full p-4">
        <WifiOff className="text-muted-foreground h-8 w-8" />
      </div>

      <div className="space-y-1">
        <p className="text-foreground text-sm font-medium">
          {t('media_offline_title')}
        </p>

        <p className="text-muted-foreground max-w-sm text-xs leading-relaxed">
          {t('media_offline_description1')}
          <br />
          {t('media_offline_description2')}
        </p>
      </div>
    </div>
  );
}

// === COMPONENTE PRINCIPAL ===

export function GameMedia({ game }: GameMediaProps) {
  const { t } = useTranslation('game_detail');
  const [status, setStatus] = useState<
    'idle' | 'loading' | 'success' | 'error'
  >('idle');
  const [errorMsg, setErrorMsg] = useState('');
  const [queue, setQueue] = useState<MediaItem[]>([]);
  const [activeIndex, setActiveIndex] = useState(0);
  const [lightboxUrl, setLightboxUrl] = useState<string | null>(null);
  const thumbnailsRef = useRef<HTMLDivElement>(null);
  const isOnline = useNetworkStatus();

  const load = useCallback(async () => {
    setStatus('loading');
    setErrorMsg('');

    try {
      const data = await invoke<GameMediaData>('get_game_media', {
        gameId: game.id,
        gameName: game.name,
      });

      const built = buildMediaQueue(data);
      setQueue(built);
      setActiveIndex(0);
      setStatus('success');
    } catch (err) {
      setErrorMsg(typeof err === 'string' ? err : t('media_unknown_error'));
      setStatus('error');
    }
  }, [game.id, game.name, t]);

  // Reset ao trocar de jogo
  useEffect(() => {
    setQueue([]);
    setActiveIndex(0);
    setStatus('idle');
  }, [game.id]);

  useEffect(() => {
    if (status === 'idle' && isOnline) {
      load();
    }
  }, [status, load, isOnline]);

  // Scroll automático da thumbnail ativa para o centro
  useMediaThumbnailScroll(thumbnailsRef, activeIndex);

  // Navegação por teclado (← →) quando a aba está visível
  useMediaKeyboard({
    enabled: status === 'success',
    activeIndex,
    totalItems: queue.length,
    onNavigate: setActiveIndex,
  });

  if (!isOnline) return <MediaOffline />;

  if (status === 'loading')
    return <ContentLoading message={t('media_loading_message')} />;

  if (status === 'error')
    return (
      <ContentError message={errorMsg} onRetry={() => setStatus('idle')} />
    );

  if (status === 'success' && queue.length === 0)
    return <MediaEmpty gameName={game.name} />;

  if (status !== 'success') return null;

  const activeItem = queue[activeIndex];

  return (
    <>
      {lightboxUrl && (
        <Lightbox url={lightboxUrl} onClose={() => setLightboxUrl(null)} />
      )}
      <div className="flex flex-col gap-4">
        {/* Viewer principal */}
        <MediaViewer
          items={queue}
          activeIndex={activeIndex}
          onNavigate={setActiveIndex}
          onFullscreen={() => {
            if (activeItem?.kind === 'screenshot')
              setLightboxUrl(activeItem.url);
          }}
        />

        {/* Fila de thumbnails */}
        <div
          ref={thumbnailsRef}
          className="flex gap-2 overflow-x-auto"
          style={{ scrollbarWidth: 'none' }}
        >
          {queue.map((item, i) => (
            <MediaThumbnail
              key={`${item.kind}-${i}`}
              item={item}
              active={i === activeIndex}
              onClick={() => setActiveIndex(i)}
            />
          ))}
        </div>

        {/* Barra de navegação inferior — estilo Steam */}
        <div className="flex items-center gap-2">
          {/* Botão esquerdo */}
          <button
            onClick={() =>
              thumbnailsRef.current?.scrollBy({
                left: -240,
                behavior: 'smooth',
              })
            }
            className="bg-muted hover:bg-muted/80 border-border text-foreground shrink-0 rounded border p-1 transition-colors"
          >
            <ChevronLeft className="h-4 w-4" />
          </button>

          {/* Scrollbar customizada */}
          <div
            className="relative h-6 flex-1 cursor-pointer rounded transition-colors"
            style={{
              backgroundColor: 'var(--muted)',
              border: '1px solid var(--border)',
            }}
            onClick={e => {
              const bar = e.currentTarget;
              const container = thumbnailsRef.current;

              if (!container) return;

              const ratio = e.nativeEvent.offsetX / bar.offsetWidth;
              const maxScroll = container.scrollWidth - container.clientWidth;
              container.scrollTo({
                left: ratio * maxScroll,
                behavior: 'smooth',
              });
            }}
          >
            <ScrollThumb containerRef={thumbnailsRef} />
          </div>

          {/* Botão direito */}
          <button
            onClick={() =>
              thumbnailsRef.current?.scrollBy({ left: 240, behavior: 'smooth' })
            }
            className="bg-muted hover:bg-muted/80 border-border text-foreground shrink-0 rounded border p-1 transition-colors"
          >
            <ChevronRight className="h-4 w-4" />
          </button>
        </div>
      </div>
    </>
  );
}

import { ExternalLink, Info } from 'lucide-react';

import { Badge } from '@/ui/badge';

interface HelpLink {
  questionLabel: string;
  linkLabel: string;
  href: string;
}

interface PlatformHelpBoxProps {
  badge: string;
  title: string;
  description: string;
  links: HelpLink[];
}

/**
 * Caixa de ajuda com badge, título, descrição e links externos.
 */
export function PlatformHelpBox({
  badge,
  title,
  description,
  links,
}: Readonly<PlatformHelpBoxProps>) {
  return (
    <div className="space-y-2 rounded-lg border border-blue-500/30 bg-blue-500/10 p-3">
      <div className="flex items-center gap-2">
        <Info className="h-4 w-4 shrink-0 text-blue-400" />
        <Badge variant="secondary" className="bg-blue-500/10 text-blue-300">
          {badge}
        </Badge>
        <p className="text-sm font-semibold text-blue-300">{title}</p>
      </div>

      <p className="text-muted-foreground text-xs leading-relaxed">
        {description}
      </p>

      {links.map(link => (
        <div
          key={link.href}
          className="text-muted-foreground flex flex-wrap items-center gap-1 text-xs"
        >
          <span>{link.questionLabel}</span>
          <a
            href={link.href}
            target="_blank"
            rel="noreferrer"
            className="flex items-center gap-0.5 text-blue-400 hover:underline"
          >
            {link.linkLabel}
            <ExternalLink size={10} />
          </a>
        </div>
      ))}
    </div>
  );
}

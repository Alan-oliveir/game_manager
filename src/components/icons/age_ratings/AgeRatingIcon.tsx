import { getAgeRatingAsset } from '@/components/icons/age_ratings/getAgeRatingAsset.ts';
import { AgeRating } from '@/types';

interface AgeRatingIconProps {
  ageRating: AgeRating;
  size?: number;
  className?: string;
}

export function AgeRatingIcon({
  ageRating,
  size = 56,
  className = '',
}: AgeRatingIconProps) {
  const { organization, rating } = ageRating;
  const src = getAgeRatingAsset(organization, rating);

  if (!src) {
    return null;
  }

  return (
    <div
      className={`bg-card border-border/50 flex items-center justify-center rounded-md border p-1 ${className}`}
      style={{ width: size, height: size }}
    >
      <img
        src={src}
        alt={`${organization} ${rating}`}
        className="h-full w-full object-contain"
      />
    </div>
  );
}

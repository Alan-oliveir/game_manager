import { AgeRatingIcon } from '@/components/icons/age_ratings/AgeRatingIcon.tsx';
import type {
  AgeRatingOrganization,
  AgeRatings as AgeRatingsMap,
  AgeRatingValue,
} from '@/types';

interface AgeRatingsProps {
  ratings: AgeRatingsMap;
  size?: number;
  className?: string;
}

export function AgeRatings({
  ratings,
  size = 56,
  className = '',
}: AgeRatingsProps) {
  if (!ratings || Object.keys(ratings).length === 0) return null;

  return (
    <div className={className}>
      {Object.entries(ratings).map(([organization, rating]) => (
        <AgeRatingIcon
          key={organization}
          ageRating={{
            organization: organization as AgeRatingOrganization,
            rating: rating as AgeRatingValue,
          }}
          size={size}
        />
      ))}
    </div>
  );
}

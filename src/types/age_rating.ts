export type AgeRatingOrganization =
  | 'ESRB'
  | 'PEGI'
  | 'CERO'
  | 'USK'
  | 'GRAC'
  | 'CLASS_IND'
  | 'ACB';

export type AgeRatingValue =
  // PEGI
  | '3'
  | '7'
  | '12'
  | '16'
  | '18'

  // ESRB
  | 'RP'
  | 'EC'
  | 'E'
  | 'E10+'
  | 'T'
  | 'M'
  | 'AO'

  // CERO
  | 'A'
  | 'B'
  | 'C'
  | 'D'
  | 'Z'

  // USK
  | '0'
  | '6'

  // GRAC
  | '12+'
  | '15+'
  | '18+'
  | 'ALL'
  | 'TESTING'

  // CLASS_IND
  | 'L'
  | '10'
  | '14'

  // ACB
  | 'G'
  | 'PG'
  | 'MA15+'
  | 'R18+'
  | 'RC';

export interface AgeRating {
  organization: AgeRatingOrganization;
  rating: AgeRatingValue;
  coverUrl?: string;
}

export type AgeRatings = Partial<Record<AgeRatingOrganization, string>>;

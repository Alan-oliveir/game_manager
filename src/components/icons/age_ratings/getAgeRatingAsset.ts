import acbG from '@/assets/age_ratings/acb/g.png';
import acbM from '@/assets/age_ratings/acb/m.png';
import acbMa15 from '@/assets/age_ratings/acb/ma15.png';
import acbPg from '@/assets/age_ratings/acb/pg.png';
import acbR18 from '@/assets/age_ratings/acb/r18.png';
import acbRc from '@/assets/age_ratings/acb/rc.png';
import ceroA from '@/assets/age_ratings/cero/a.png';
import ceroB from '@/assets/age_ratings/cero/b.png';
import ceroC from '@/assets/age_ratings/cero/c.png';
import ceroD from '@/assets/age_ratings/cero/d.png';
import ceroZ from '@/assets/age_ratings/cero/z.png';
import classInd10 from '@/assets/age_ratings/class_ind/10.png';
import classInd12 from '@/assets/age_ratings/class_ind/12.png';
import classInd14 from '@/assets/age_ratings/class_ind/14.png';
import classInd16 from '@/assets/age_ratings/class_ind/16.png';
import classInd18 from '@/assets/age_ratings/class_ind/18.png';
import classIndL from '@/assets/age_ratings/class_ind/l.png';
import esrbAo from '@/assets/age_ratings/esrb/ao.png';
import esrbE from '@/assets/age_ratings/esrb/e.png';
import esrbE10 from '@/assets/age_ratings/esrb/e10.png';
import esrbEc from '@/assets/age_ratings/esrb/ec.png';
import esrbM from '@/assets/age_ratings/esrb/m.png';
import esrbRp from '@/assets/age_ratings/esrb/rp.png';
import esrbT from '@/assets/age_ratings/esrb/t.png';
import grac12 from '@/assets/age_ratings/grac/12.png';
import grac15 from '@/assets/age_ratings/grac/15.png';
import grac18 from '@/assets/age_ratings/grac/18.png';
import gracAll from '@/assets/age_ratings/grac/all.png';
import gracTesting from '@/assets/age_ratings/grac/testing.png';
import pegi3 from '@/assets/age_ratings/pegi/3.png';
import pegi7 from '@/assets/age_ratings/pegi/7.png';
import pegi12 from '@/assets/age_ratings/pegi/12.png';
import pegi16 from '@/assets/age_ratings/pegi/16.png';
import pegi18 from '@/assets/age_ratings/pegi/18.png';
import usk0 from '@/assets/age_ratings/usk/0.png';
import usk6 from '@/assets/age_ratings/usk/6.png';
import usk12 from '@/assets/age_ratings/usk/12.png';
import usk16 from '@/assets/age_ratings/usk/16.png';
import usk18 from '@/assets/age_ratings/usk/18.png';
import type { AgeRatingOrganization, AgeRatingValue } from '@/types';

type AgeRatingAssetMap = Partial<Record<AgeRatingValue, string>>;

const AGE_RATING_ASSETS: Record<AgeRatingOrganization, AgeRatingAssetMap> = {
  ESRB: {
    RP: esrbRp,
    EC: esrbEc,
    E: esrbE,
    'E10+': esrbE10,
    T: esrbT,
    M: esrbM,
    AO: esrbAo,
  },

  PEGI: {
    '3': pegi3,
    '7': pegi7,
    '12': pegi12,
    '16': pegi16,
    '18': pegi18,
  },

  CERO: {
    A: ceroA,
    B: ceroB,
    C: ceroC,
    D: ceroD,
    Z: ceroZ,
  },

  USK: {
    '0': usk0,
    '6': usk6,
    '12': usk12,
    '16': usk16,
    '18': usk18,
  },

  GRAC: {
    ALL: gracAll,
    '12+': grac12,
    '15+': grac15,
    '18+': grac18,
    TESTING: gracTesting,
  },

  CLASS_IND: {
    L: classIndL,
    '10': classInd10,
    '12': classInd12,
    '14': classInd14,
    '16': classInd16,
    '18': classInd18,
  },

  ACB: {
    G: acbG,
    PG: acbPg,
    M: acbM,
    'MA15+': acbMa15,
    'R18+': acbR18,
    RC: acbRc,
  },
};

export function getAgeRatingAsset(
  organization: AgeRatingOrganization,
  rating: AgeRatingValue
): string | undefined {
  return AGE_RATING_ASSETS[organization]?.[rating];
}

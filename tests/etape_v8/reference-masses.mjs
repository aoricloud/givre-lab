// Le rastériseur de référence de l'étape V8, en JavaScript : l'image nette de la neige à partir des seules masses (`givre_neige_masse`), écrit D'APRÈS LA SEULE DOC de `src/vitrine.rs` (« La masse de chaque case » : l'ordre canonique, la géométrie, l'unité,
// le dégradé, le pixel). C'est le même rendu que `rasteriser` de `tests/etape_v8_commun/mod.rs` (en Rust, `i64`) ; il ne lit ni `rendu_hex` ni le module : seulement un tableau de masses.
//
// **Les entiers qui dépassent 2^53** : `Qn = -ny B + nx A`, `Rn = -2 nx A`, `Sn = ny B + nx A`, `M = 2 N B` et `2 Z + M` (donc `rq M`, `rr M`, `rs M` et `dq`, `dr`, `ds`) sont calculés en `BigInt`. Pour la vue de la vitrine (V = 69) et `cote × sur <= 4096` ils restent
// sous 2^52 (la doc le dit : un `f64` y suffirait) ; `BigInt` rend le calcul exact pour **toute** vue et toute taille permises par le module (`B` et `A` valent près de 2^31 : un produit par une coordonnée de 2^20 dépasse déjà 2^50). Le reste (les indices, les couleurs) tient dans un `Number`.
//
// Aucune dépendance : JavaScript seul.

const A = 1239850262n;
const B = 2147483647n;
const ARRETS = [
  [255, 255, 255],
  [198, 219, 239],
  [107, 174, 214],
  [33, 113, 181],
  [8, 48, 107],
];

/** Le plancher d'une division de deux `BigInt` (le quotient est arrondi vers moins l'infini). */
function plancher(a, b) {
  let q = a / b;
  if (a % b !== 0n && a < 0n !== b < 0n) {
    q -= 1n;
  }
  return q;
}

/** L'indice canonique de la case `(q, r)` du domaine de rayon `rayon` : `avant(r) + (q − max(−R, −R − r))` (la formule fermée de la doc). */
export function indiceDe(rayon, q, r) {
  const avant =
    r < 0
      ? (rayon + r) * (2 * rayon + 1) - ((rayon - r + 1) * (rayon + r)) / 2
      : (rayon * (3 * rayon + 1)) / 2 + r * (2 * rayon + 1) - (r * (r - 1)) / 2;
  return avant + q - Math.max(-rayon, -rayon - r);
}

/** `t` de la doc pour une masse : `min(masse, 3072) / 3`, division entière. */
export function tDeLaMasse(masse) {
  return Math.floor(Math.min(masse, 3072) / 3);
}

/** La couleur du dégradé pour `t` de 0 à 1024 : l'interpolation entière entre les cinq arrêts. */
export function couleurDeT(t) {
  if (t >= 1024) {
    return ARRETS[4].slice();
  }
  const s = Math.floor(t / 256);
  const f = t % 256;
  return [0, 1, 2].map((c) => (ARRETS[s][c] * (256 - f) + ARRETS[s + 1][c] * f + 128) >> 8);
}

/**
 * L'image nette : un `Uint8Array` de `3 × cote²` octets (R, V, B, ligne par ligne de haut en bas).
 * `masses` : les `3 R (R + 1) + 1` masses dans l'ordre canonique (des entiers de 0 à 2^32 − 2) ; `rayonVue` : le rayon de la vue (69 pour la vitrine) ; `sur × sur` échantillons par pixel.
 */
export function rasteriser(rayon, masses, cote, rayonVue, sur) {
  if (masses.length !== 3 * rayon * (rayon + 1) + 1) {
    throw new Error(`il faut ${3 * rayon * (rayon + 1) + 1} masses, pas ${masses.length}`);
  }
  const N = BigInt(cote * sur);
  const large = BigInt(2 * rayonVue + 1);
  const M = 2n * N * B;
  const arrondi = (z) => plancher(2n * z + M, 2n * M);
  const carre = sur * sur;
  const image = new Uint8Array(3 * cote * cote);
  for (let py = 0; py < cote; py++) {
    for (let px = 0; px < cote; px++) {
      const somme = [0, 0, 0];
      for (let j = 0; j < sur; j++) {
        const ny = BigInt(2 * (py * sur + j) + 1 - cote * sur) * large;
        for (let i = 0; i < sur; i++) {
          const nx = BigInt(2 * (px * sur + i) + 1 - cote * sur) * large;
          const qn = -ny * B + nx * A;
          const rn = -2n * nx * A;
          const sn = ny * B + nx * A;
          let rq = arrondi(qn);
          let rr = arrondi(rn);
          const rs = arrondi(sn);
          const dq = (rq * M - qn) < 0n ? qn - rq * M : rq * M - qn;
          const dr = (rr * M - rn) < 0n ? rn - rr * M : rr * M - rn;
          const ds = (rs * M - sn) < 0n ? sn - rs * M : rs * M - sn;
          if (dq > dr && dq > ds) {
            rq = -rr - rs;
          } else if (dr > ds) {
            rr = -rq - rs;
          }
          const q = Number(rq);
          const r = Number(rr);
          const couleur =
            Math.max(Math.abs(q), Math.abs(r), Math.abs(q + r)) > rayon
              ? [255, 255, 255]
              : couleurDeT(tDeLaMasse(masses[indiceDe(rayon, q, r)]));
          somme[0] += couleur[0];
          somme[1] += couleur[1];
          somme[2] += couleur[2];
        }
      }
      const d = 3 * (py * cote + px);
      for (let c = 0; c < 3; c++) {
        image[d + c] = Math.floor((somme[c] + Math.floor(carre / 2)) / carre);
      }
    }
  }
  return image;
}

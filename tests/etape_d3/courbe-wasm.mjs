// Fait calculer la courbe D(η) par le module WebAssembly de la vitrine, un amas par appel, et la fait lire par `docs/lecture.js` (`lireCourbe`), comme le fait la page.
//
// Usage : node courbe-wasm.mjs <fichier.wasm> <lecture.js> <taille> <graines>
//
// <taille> et <graines> sont des entiers de 32 bits non signés (chiffres décimaux seulement). Le script charge le module (sans aucune importation) et `lecture.js` (par son adresse
// `file:`) ; **avant tout démarrage** il vérifie que `givre_courbe_fait`, `givre_courbe_total` et `givre_courbe_code` rendent 0, que `givre_courbe_moyenne(0)` à `(3)` et
// `givre_courbe_erreur_type(0)` à `(3)` rendent NaN et que `givre_courbe_pas()` rend 5 (aucun travail) ; puis il écrit sur la sortie standard, une par ligne, `clé=valeur`, dans cet ordre :
//
//   avant     : 1 si tout ce qui précède est vrai, sinon 0 ;
//   refus     : les codes que rend `givre_courbe_demarrer` pour (0, 4), (101, 4), (4294967295, 4), (40, 1), (40, 0), (40, 9), (40, 4294967295), (0, 0), séparés par des virgules ;
//   sans      : `<fait>,<total>,<code>` lus juste après le dernier refus, puis `;` et ce que rend `givre_courbe_pas()` ;
//   demarre   : ce que rend `givre_courbe_demarrer(taille, graines)` ;
//   total     : `givre_courbe_total()` juste après (lu non signé) ;
//   suivants  : avant chaque `givre_courbe_pas()`, le `suivant` de `lireCourbe(m)` écrit `<eta>:<graine>` (ou `fin`), séparés par des virgules ; au plus 40 pas sont faits ;
//   codes     : ce que rend chaque `givre_courbe_pas()` ;
//   faits     : le `fait` lu après chaque `givre_courbe_pas()` ;
//   fin       : `<fait>,<total>,<code>` à la fin (lus par `lireCourbe(m)`) ;
//   points    : pour `η` de 0 à 3, `<moyenne>:<erreur-type>:<repère>` lus par `lireCourbe(m).points` (NaN, ou 8 octets du f64 petit-boutiste en hexadécimal), séparés par des virgules ;
//   fidele    : 1 si `lireCourbe(m).points` est, au bit près, ce que rendent directement `givre_courbe_moyenne(eta)`, `givre_courbe_erreur_type(eta)` et `givre_courbe_repere(eta)`, ou si
//               NaN des deux côtés ; sinon 0 ;
//   hors      : `givre_courbe_moyenne`, `givre_courbe_erreur_type` et `givre_courbe_repere` pour `eta` = 4 et 4294967295, séparés par des virgules (NaN ou 8 octets) ;
//   encore    : `<c>,<fait>` : ce que rend un `givre_courbe_pas()` de plus et le `fait` lu ensuite ;
//   efface    : `<d>,<fait>,<total>,<moyenne(0)>` après un nouveau `givre_courbe_demarrer(taille, graines)` : le code, `fait`, `total` et la moyenne de η = 0 (NaN).
//
// Si le fichier, `lecture.js` ou les arguments sont faux, ou si une fonction manque, écrit un message sur la sortie d'erreur, sort avec le code 1 et n'écrit **rien** sur la sortie
// standard. Aucune dépendance : les modules intégrés de Node seulement.

import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { pathToFileURL } from "node:url";

const MAX_U32 = 4294967295;
const PAS_MAX = 40;
const EXPORTATIONS = [
  "givre_courbe_demarrer",
  "givre_courbe_pas",
  "givre_courbe_fait",
  "givre_courbe_total",
  "givre_courbe_code",
  "givre_courbe_moyenne",
  "givre_courbe_erreur_type",
  "givre_courbe_repere",
];

/** Lit un entier de 32 bits non signé écrit en chiffres décimaux ; échoue sinon. */
function entier32(texte, nom) {
  if (!/^[0-9]+$/.test(texte) || Number(texte) > MAX_U32) {
    throw new Error(`${nom} doit être un entier de 0 à ${MAX_U32} : « ${texte} »`);
  }
  return Number(texte);
}

/** `NaN` si le nombre est NaN, sinon ses 8 octets petit-boutistes en hexadécimal. */
function hexa(x) {
  if (Number.isNaN(x)) {
    return "NaN";
  }
  const vue = new DataView(new ArrayBuffer(8));
  vue.setFloat64(0, x, true);
  let texte = "";
  for (let i = 0; i < 8; i++) {
    texte += vue.getUint8(i).toString(16).padStart(2, "0");
  }
  return texte;
}

async function principal() {
  const args = process.argv.slice(2);
  if (args.length !== 4) {
    throw new Error("usage : node courbe-wasm.mjs <fichier.wasm> <lecture.js> <taille> <graines>");
  }
  const taille = entier32(args[2], "la taille");
  const graines = entier32(args[3], "le nombre de graines");

  const { instance } = await WebAssembly.instantiate(readFileSync(args[0]), {});
  const m = instance.exports;
  for (const nom of EXPORTATIONS) {
    if (typeof m[nom] !== "function") {
      throw new Error(`le module n'exporte pas la fonction ${nom}`);
    }
  }
  const lecture = await import(pathToFileURL(resolve(args[1])).href);
  if (typeof lecture.lireCourbe !== "function") {
    throw new Error("lecture.js n'exporte pas la fonction lireCourbe");
  }

  let avant =
    (m.givre_courbe_fait() >>> 0) === 0 && (m.givre_courbe_total() >>> 0) === 0 && (m.givre_courbe_code() >>> 0) === 0;
  for (let eta = 0; eta < 4; eta++) {
    avant = avant && Number.isNaN(m.givre_courbe_moyenne(eta)) && Number.isNaN(m.givre_courbe_erreur_type(eta));
  }
  avant = avant && (m.givre_courbe_pas() >>> 0) === 5;

  const refus = [
    [0, 4],
    [101, 4],
    [MAX_U32, 4],
    [40, 1],
    [40, 0],
    [40, 9],
    [40, MAX_U32],
    [0, 0],
  ].map(([t, g]) => m.givre_courbe_demarrer(t, g) >>> 0);
  const sans = `${m.givre_courbe_fait() >>> 0},${m.givre_courbe_total() >>> 0},${m.givre_courbe_code() >>> 0};${m.givre_courbe_pas() >>> 0}`;

  const demarre = m.givre_courbe_demarrer(taille, graines) >>> 0;
  const total = m.givre_courbe_total() >>> 0;
  const suivants = [];
  const codes = [];
  const faits = [];
  for (let k = 0; k < PAS_MAX; k++) {
    const courbe = lecture.lireCourbe(m);
    if (courbe.suivant === null) {
      suivants.push("fin");
      break;
    }
    suivants.push(`${courbe.suivant.eta}:${courbe.suivant.graine}`);
    const code = m.givre_courbe_pas() >>> 0;
    codes.push(code);
    faits.push(m.givre_courbe_fait() >>> 0);
    if (code !== 0) {
      break;
    }
  }
  const courbe = lecture.lireCourbe(m);
  let fidele = courbe.points.length === 4;
  for (let eta = 0; fidele && eta < 4; eta++) {
    const point = courbe.points[eta];
    fidele =
      point.eta === eta &&
      hexa(point.moyenne) === hexa(m.givre_courbe_moyenne(eta)) &&
      hexa(point.erreurType) === hexa(m.givre_courbe_erreur_type(eta)) &&
      hexa(point.repere) === hexa(m.givre_courbe_repere(eta));
  }
  const points = courbe.points.map((p) => `${hexa(p.moyenne)}:${hexa(p.erreurType)}:${hexa(p.repere)}`);
  const hors = [4, MAX_U32].flatMap((eta) => [
    hexa(m.givre_courbe_moyenne(eta)),
    hexa(m.givre_courbe_erreur_type(eta)),
    hexa(m.givre_courbe_repere(eta)),
  ]);
  const encore = `${m.givre_courbe_pas() >>> 0},${m.givre_courbe_fait() >>> 0}`;
  const efface = `${m.givre_courbe_demarrer(taille, graines) >>> 0},${m.givre_courbe_fait() >>> 0},${m.givre_courbe_total() >>> 0},${hexa(m.givre_courbe_moyenne(0))}`;

  const lignes = [
    `avant=${avant ? 1 : 0}`,
    `refus=${refus.join(",")}`,
    `sans=${sans}`,
    `demarre=${demarre}`,
    `total=${total}`,
    `suivants=${suivants.join(",")}`,
    `codes=${codes.join(",")}`,
    `faits=${faits.join(",")}`,
    `fin=${courbe.fait},${courbe.total},${courbe.code}`,
    `points=${points.join(",")}`,
    `fidele=${fidele ? 1 : 0}`,
    `hors=${hors.join(",")}`,
    `encore=${encore}`,
    `efface=${efface}`,
  ];
  process.stdout.write(lignes.join("\n") + "\n");
}

principal().catch((erreur) => {
  process.stderr.write(`courbe-wasm : ${erreur.message}\n`);
  process.exitCode = 1;
});

// Fait lire la mesure de corrélation du module WebAssembly de la vitrine par `docs/lecture.js` (`lireCorrelation`), comme le fait la page.
//
// Usage : node correlation-wasm.mjs <fichier.wasm> <lecture.js> <modele> <graine> <pas>
//
// <modele> est un entier de 0 à 3, <graine> et <pas> des entiers de 32 bits non signés (chiffres décimaux seulement). Le script charge le
// module (sans aucune importation) et `lecture.js` (par son adresse `file:`, ce qui marche sous Linux comme sous Windows) ; **avant tout
// démarrage** il vérifie que `givre_mesurer` rend 0 et que `givre_correlation_rayon(0)` rend 0, `givre_correlation_valeur(0)` et
// `givre_dimension` NaN ; puis il démarre le modèle avec la graine, l'avance de <pas> unités d'un seul appel de `givre_avancer`, appelle
// `givre_mesurer` une fois, et écrit sur la sortie standard, une par ligne, `clé=valeur`, dans cet ordre :
//
//   avant     : 1 si, sur le module neuf, `givre_mesurer()` rend 0, `givre_correlation_rayon(0)` rend 0, et `givre_correlation_valeur(0)` et
//               `givre_dimension()` rendent NaN ; sinon 0 ;
//   joue      : ce que rend `givre_avancer(pas)` ;
//   points    : ce que rend `givre_mesurer()` (lu non signé) ;
//   sites     : `givre_sites()` juste après la mesure ;
//   rayons    : les rayons de `lireCorrelation(m).rayons`, en décimal, séparés par des virgules (vide s'il n'y en a pas) ;
//   valeurs   : les `C(r)` de `lireCorrelation(m).valeurs`, chacun en NaN ou en 8 octets du f64 petit-boutiste hexadécimaux, séparés par des
//               virgules (vide s'il n'y en a pas) ;
//   dimension : `lireCorrelation(m).dimension` : NaN, ou les 8 octets du f64 petit-boutiste en hexadécimal ;
//   fidele    : 1 si `lireCorrelation(m)` a exactement `points` rayons et `points` valeurs et si chacun est ce que rendent directement
//               `givre_correlation_rayon(i) >>> 0` et `givre_correlation_valeur(i)` (au bit près) pour `i` de 0 à points - 1, et si sa
//               dimension est (au bit près, ou NaN des deux côtés) celle de `givre_dimension()` ; sinon 0 ;
//   hors      : `givre_correlation_rayon(points)`, `givre_correlation_rayon(points + 1)` et `givre_correlation_rayon(4294967295)`, lus non
//               signés, séparés par des virgules, puis `;`, puis `givre_correlation_valeur` des mêmes trois indices (NaN ou 8 octets) ;
//   apres     : `<a>,<r>,<d>` : <a> ce que rend un `givre_avancer(1)` supplémentaire, <r> `givre_correlation_rayon(0)` lu ensuite, <d>
//               `givre_dimension()` lu ensuite (NaN ou 8 octets).
//
// Si le fichier, `lecture.js` ou les arguments sont faux, si une fonction manque, ou si `givre_demarrer` ne rend pas 1, écrit un message sur
// la sortie d'erreur, sort avec le code 1 et n'écrit **rien** sur la sortie standard. Aucune dépendance : les modules intégrés de Node
// seulement.

import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { pathToFileURL } from "node:url";

const MAX_U32 = 4294967295;
const EXPORTATIONS = [
  "givre_demarrer",
  "givre_avancer",
  "givre_sites",
  "givre_mesurer",
  "givre_correlation_rayon",
  "givre_correlation_valeur",
  "givre_dimension",
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
  if (args.length !== 5) {
    throw new Error("usage : node correlation-wasm.mjs <fichier.wasm> <lecture.js> <modele> <graine> <pas>");
  }
  const modele = entier32(args[2], "le modèle");
  if (modele > 3) {
    throw new Error(`le modèle doit être 0, 1, 2 ou 3 : « ${args[2]} »`);
  }
  const graine = entier32(args[3], "la graine");
  const pas = entier32(args[4], "le nombre de pas");

  const { instance } = await WebAssembly.instantiate(readFileSync(args[0]), {});
  const m = instance.exports;
  for (const nom of EXPORTATIONS) {
    if (typeof m[nom] !== "function") {
      throw new Error(`le module n'exporte pas la fonction ${nom}`);
    }
  }
  const lecture = await import(pathToFileURL(resolve(args[1])).href);
  if (typeof lecture.lireCorrelation !== "function") {
    throw new Error("lecture.js n'exporte pas la fonction lireCorrelation");
  }

  const avant =
    (m.givre_mesurer() >>> 0) === 0 &&
    (m.givre_correlation_rayon(0) >>> 0) === 0 &&
    Number.isNaN(m.givre_correlation_valeur(0)) &&
    Number.isNaN(m.givre_dimension());

  if ((m.givre_demarrer(modele, graine) >>> 0) !== 1) {
    throw new Error(`givre_demarrer(${modele}, ${graine}) ne rend pas 1`);
  }
  const joue = m.givre_avancer(pas) >>> 0;
  const points = m.givre_mesurer() >>> 0;
  const sites = m.givre_sites() >>> 0;
  const courbe = lecture.lireCorrelation(m);

  let fidele = courbe.rayons.length === points && courbe.valeurs.length === points;
  for (let i = 0; fidele && i < points; i++) {
    fidele =
      courbe.rayons[i] === m.givre_correlation_rayon(i) >>> 0 &&
      hexa(courbe.valeurs[i]) === hexa(m.givre_correlation_valeur(i));
  }
  fidele = fidele && hexa(courbe.dimension) === hexa(m.givre_dimension());

  const hors = [points, points + 1, MAX_U32];
  const horsRayons = hors.map((i) => m.givre_correlation_rayon(i) >>> 0).join(",");
  const horsValeurs = hors.map((i) => hexa(m.givre_correlation_valeur(i))).join(",");

  const a = m.givre_avancer(1) >>> 0;
  const apres = `${a},${m.givre_correlation_rayon(0) >>> 0},${hexa(m.givre_dimension())}`;

  const lignes = [
    `avant=${avant ? 1 : 0}`,
    `joue=${joue}`,
    `points=${points}`,
    `sites=${sites}`,
    `rayons=${courbe.rayons.join(",")}`,
    `valeurs=${courbe.valeurs.map(hexa).join(",")}`,
    `dimension=${hexa(courbe.dimension)}`,
    `fidele=${fidele ? 1 : 0}`,
    `hors=${horsRayons};${horsValeurs}`,
    `apres=${apres}`,
  ];
  process.stdout.write(lignes.join("\n") + "\n");
}

principal().catch((erreur) => {
  process.stderr.write(`correlation-wasm : ${erreur.message}\n`);
  process.exitCode = 1;
});

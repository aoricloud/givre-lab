// Fait lire l'image nette du module WebAssembly de la vitrine par `docs/lecture.js` (`lireImageNette`), comme le fait la page, et écrit tout ce que le module en dit.
//
// Usage : node nette-wasm.mjs <fichier.wasm> <lecture.js> <modele> <graine> <pas> <cote> <sur>
//
// <modele> est un entier de 0 à 7 (voir `src/vitrine.rs` : 0 la DLA, 1 à 3 les flocons, 4 à 7 le claquage) ; <graine>, <pas>, <cote> et <sur> des entiers de 32 bits non signés
// (chiffres décimaux seulement : le module, lui, refuse un `cote` ou un `sur` hors borne par un code). Le script charge le module (sans aucune importation) et `lecture.js` (par son adresse
// `file:`, ce qui marche sous Linux comme sous Windows) ; **avant tout démarrage** il prépare une image (le module neuf doit rendre le code 2) ; puis il démarre le modèle avec la graine,
// l'avance de <pas> unités d'un seul appel de `givre_avancer`, et écrit sur la sortie standard, une par ligne, `clé=valeur`, dans cet ordre :
//
//   avant        : `<code>,<largeur>,<hauteur>,<pixel>` : `givre_nette_preparer(cote, sur)`, `givre_nette_largeur()`, `givre_nette_hauteur()` et `givre_nette_pixel(0, 0)` (non signés, en décimal) lus
//                  sur le module NEUF : `2,0,0,4294967295` ;
//   demarre      : ce que rend `givre_demarrer(modele, graine)` ;
//   joue         : ce que rend `givre_avancer(pas)` ;
//   etat_avant   : `<temps>,<duree>,<sites>,<rayon>,<empreinte>,<largeur>x<hauteur>,<rgba>` : l'état lu AVANT toute préparation : `givre_temps`, `givre_duree`, `givre_sites`, `givre_rayon` (non signés),
//                  `lireEmpreinte(m)`, les dimensions et le SHA-256 des octets de `lireImage(m).rgba` (l'image d'un pixel par case) ;
//   vide         : `<largeur>,<hauteur>,<pixel>` : le tampon de l'image nette AVANT toute préparation, après le démarrage et les pas : `0,0,4294967295` ;
//   code         : le code de `lireImageNette(m, cote, sur)` (le module prépare, puis `lecture.js` lit) ;
//   dimensions   : `<largeur>x<hauteur>` de ce que `lireImageNette` rend ;
//   rgba         : le SHA-256 des octets `rgba` que `lireImageNette` rend (un code non nul : le SHA-256 de zéro octet) ;
//   pixels       : le SHA-256 des octets R, V, B lus **directement** par `givre_nette_pixel(x, y)` (sans `lecture.js`), ligne par ligne de haut en bas, pour `x < largeur` et `y < hauteur` (le tampon tel que
//                  le module le garde : sans tampon, le SHA-256 de zéro octet) ;
//   hors         : `givre_nette_pixel` (non signé) pour `(largeur, 0)`, `(0, hauteur)`, `(largeur, hauteur)`, `(4294967295, 0)` et `(0, 4294967295)`, séparés par des virgules (cinq `4294967295`) ;
//   etat_apres   : comme `etat_avant`, relu APRÈS la préparation : la préparation ne change rien à l'état ;
//   refus        : les codes de `givre_nette_preparer` pour `(0, 2)`, `(1, 2)`, `(3, 2)`, `(2049, 2)`, `(2050, 2)`, `(4294967295, 2)`, `(cote, 0)`, `(cote, 5)`, `(cote, 4294967295)`, `(2048, 3)` et
//                  `(1366, 3)` (dans cet ordre, séparés par des virgules) ;
//   apres_refus  : `<largeur>,<hauteur>,<pixel>` du tampon juste après le dernier refus : `0,0,4294967295` (un refus efface le tampon) ;
//   efface       : après `lireImageNette(m, cote, sur)` puis un `givre_avancer(1)` : `<ce que rend avancer>,<largeur>,<hauteur>,<pixel>` du tampon (effacé si une unité a été jouée, gardé sinon) ;
//   redemarre    : après `lireImageNette(m, cote, sur)` puis `givre_demarrer(modele, graine)` : `<ce que rend demarrer>,<largeur>,<hauteur>,<pixel>` du tampon (effacé).
//
// **L'ordre des lectures compte** : l'état (`etat_avant`, `etat_apres`) est lu avant tout appel qui le change (`givre_avancer(1)` de `efface`, `givre_demarrer` de `redemarre`, qui viennent en dernier).
//
// Si le fichier, `lecture.js` ou les arguments sont faux, si une fonction manque, ou si `givre_demarrer` ne rend pas 1, écrit un message sur la sortie d'erreur, sort avec le code 1 et
// n'écrit **rien** sur la sortie standard. Aucune dépendance : les modules intégrés de Node seulement.

import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { pathToFileURL } from "node:url";

const MAX_U32 = 4294967295;
const EXPORTATIONS = [
  "givre_demarrer",
  "givre_avancer",
  "givre_temps",
  "givre_duree",
  "givre_sites",
  "givre_rayon",
  "givre_nette_preparer",
  "givre_nette_largeur",
  "givre_nette_hauteur",
  "givre_nette_pixel",
];

/** Lit un entier de 32 bits non signé écrit en chiffres décimaux ; échoue sinon. */
function entier32(texte, nom) {
  if (!/^[0-9]+$/.test(texte) || Number(texte) > MAX_U32) {
    throw new Error(`${nom} doit être un entier de 0 à ${MAX_U32} : « ${texte} »`);
  }
  return Number(texte);
}

/** Le SHA-256, en hexadécimal, de ces octets. */
function sha256(octets) {
  return createHash("sha256").update(octets).digest("hex");
}

async function principal() {
  const args = process.argv.slice(2);
  if (args.length !== 7) {
    throw new Error("usage : node nette-wasm.mjs <fichier.wasm> <lecture.js> <modele> <graine> <pas> <cote> <sur>");
  }
  const modele = entier32(args[2], "le modèle");
  if (modele > 7) {
    throw new Error(`le modèle doit être un entier de 0 à 7 : « ${args[2]} »`);
  }
  const graine = entier32(args[3], "la graine");
  const pas = entier32(args[4], "le nombre de pas");
  const cote = entier32(args[5], "le côté");
  const sur = entier32(args[6], "le suréchantillonnage");

  const { instance } = await WebAssembly.instantiate(readFileSync(args[0]), {});
  const m = instance.exports;
  for (const nom of EXPORTATIONS) {
    if (typeof m[nom] !== "function") {
      throw new Error(`le module n'exporte pas la fonction ${nom}`);
    }
  }
  const lecture = await import(pathToFileURL(resolve(args[1])).href);
  for (const nom of ["lireEmpreinte", "lireImage", "lireImageNette"]) {
    if (typeof lecture[nom] !== "function") {
      throw new Error(`lecture.js n'exporte pas la fonction ${nom}`);
    }
  }

  /** Le tampon de l'image nette tel que le module le garde : largeur, hauteur et pixel (0, 0), non signés. */
  const tampon = () => [m.givre_nette_largeur() >>> 0, m.givre_nette_hauteur() >>> 0, m.givre_nette_pixel(0, 0) >>> 0];

  // Le module NEUF : rien n'est démarré, la préparation rend le code 2 et le tampon est vide.
  const avant = [m.givre_nette_preparer(cote, sur) >>> 0, ...tampon()];

  const demarre = m.givre_demarrer(modele, graine) >>> 0;
  if (demarre !== 1) {
    throw new Error(`givre_demarrer(${modele}, ${graine}) ne rend pas 1`);
  }
  const joue = m.givre_avancer(pas) >>> 0;

  /** L'état lu par la page : le temps, la durée, les sites, le rayon, l'empreinte et l'image d'un pixel par case. */
  const etat = () => {
    const temps = m.givre_temps() >>> 0;
    const duree = m.givre_duree() >>> 0;
    const sites = m.givre_sites() >>> 0;
    const rayon = m.givre_rayon() >>> 0;
    const empreinte = lecture.lireEmpreinte(m);
    const image = lecture.lireImage(m);
    const rgba = sha256(Buffer.from(image.rgba.buffer, image.rgba.byteOffset, image.rgba.byteLength));
    return `${temps},${duree},${sites},${rayon},${empreinte},${image.largeur}x${image.hauteur},${rgba}`;
  };

  // TOUT ce qui décrit l'état est lu ICI, avant les appels qui le changent (`efface`, `redemarre`, plus bas).
  const etatAvant = etat();
  const vide = tampon();

  const nette = lecture.lireImageNette(m, cote, sur);
  const rgba = sha256(Buffer.from(nette.rgba.buffer, nette.rgba.byteOffset, nette.rgba.byteLength));

  // Le tampon tel que le module le garde, lu pixel par pixel sans `lecture.js`.
  const largeur = m.givre_nette_largeur() >>> 0;
  const hauteur = m.givre_nette_hauteur() >>> 0;
  const rvb = Buffer.alloc(3 * largeur * hauteur);
  for (let y = 0; y < hauteur; y++) {
    for (let x = 0; x < largeur; x++) {
      const couleur = m.givre_nette_pixel(x, y) >>> 0;
      const k = 3 * (y * largeur + x);
      rvb[k] = (couleur >>> 16) & 255;
      rvb[k + 1] = (couleur >>> 8) & 255;
      rvb[k + 2] = couleur & 255;
    }
  }
  const pixels = sha256(rvb);
  const hors = [
    m.givre_nette_pixel(largeur, 0) >>> 0,
    m.givre_nette_pixel(0, hauteur) >>> 0,
    m.givre_nette_pixel(largeur, hauteur) >>> 0,
    m.givre_nette_pixel(MAX_U32, 0) >>> 0,
    m.givre_nette_pixel(0, MAX_U32) >>> 0,
  ];
  const etatApres = etat();

  const refus = [
    [0, 2],
    [1, 2],
    [3, 2],
    [2049, 2],
    [2050, 2],
    [MAX_U32, 2],
    [cote, 0],
    [cote, 5],
    [cote, MAX_U32],
    [2048, 3],
    [1366, 3],
  ].map(([c, s]) => m.givre_nette_preparer(c, s) >>> 0);
  const apresRefus = tampon();

  // Les deux appels qui changent l'état viennent en dernier.
  lecture.lireImageNette(m, cote, sur);
  const jouee = m.givre_avancer(1) >>> 0;
  const efface = [jouee, ...tampon()];
  lecture.lireImageNette(m, cote, sur);
  const redemarre = [m.givre_demarrer(modele, graine) >>> 0, ...tampon()];

  const lignes = [
    `avant=${avant.join(",")}`,
    `demarre=${demarre}`,
    `joue=${joue}`,
    `etat_avant=${etatAvant}`,
    `vide=${vide.join(",")}`,
    `code=${nette.code}`,
    `dimensions=${nette.largeur}x${nette.hauteur}`,
    `rgba=${rgba}`,
    `pixels=${pixels}`,
    `hors=${hors.join(",")}`,
    `etat_apres=${etatApres}`,
    `refus=${refus.join(",")}`,
    `apres_refus=${apresRefus.join(",")}`,
    `efface=${efface.join(",")}`,
    `redemarre=${redemarre.join(",")}`,
  ];
  process.stdout.write(lignes.join("\n") + "\n");
}

principal().catch((erreur) => {
  process.stderr.write(`nette-wasm : ${erreur.message}\n`);
  process.exitCode = 1;
});

// Fait jouer le métal (étape P4) par le module WebAssembly de la vitrine et le fait lire par `docs/lecture.js`, comme le fait la page, et écrit tout ce que le module en dit.
//
// Usage : node metal-wasm.mjs <fichier.wasm> <lecture.js> <epsilon_ppm> <cote> <morceaux> <taille>
//
// <epsilon_ppm> et <cote> sont passés à `givre_metal_demarrer` ; <morceaux> est une liste de nombres de pas séparés par des virgules (chacun un appel de `givre_avancer`) ; <taille> est le côté, en pixels, de l'image que
// `imageDuMetal` dessine à la fin. Tous sont des entiers de 32 bits non signés (chiffres décimaux seulement). Le script charge le module (sans aucune importation) et `lecture.js` (par son adresse `file:`). Un `f64` s'écrit par ses
// bits en hexadécimal (16 chiffres, gros-boutiste), ou `NaN`. Il écrit sur la sortie standard, une par ligne, `clé=valeur`, dans cet ordre :
//
//   avant      : sur le module NEUF : `<givre_metal_reglage(0)>,<givre_metal_mesure(1)>,<givre_metal_case(0, 0)>,<givre_temps>,<givre_duree>,<lireReglagesMetal>,<lireMesuresMetal>,<lireChampsMetal>` (les trois lectures de
//                `lecture.js` écrites `null` ou `objet`) : `NaN,NaN,NaN,0,0,null,null,null` ;
//   refus      : les codes de `givre_metal_demarrer` pour `(60001, cote)`, `(epsilon, 127)`, `(epsilon, 257)`, `(4294967295, 4294967295)`, `(epsilon, 0)`, `(60001, 0)` sur le module neuf : `1,2,2,1,2,1` ;
//   apres_refus: comme `avant`, sans les lectures de `lecture.js` : rien n'est démarré ;
//   demarre    : ce que rend `givre_metal_demarrer(epsilon, cote)` ;
//   reglages   : les quinze champs de `lireReglagesMetal`, dans l'ordre du module (epsilon, delta, d0, diffusivite, w0, lambda, tau0, dx, dt, rayon, cote, limite, duree, releve, profondeur), séparés par des virgules ;
//   morceau    : une ligne par morceau : `<joué>;<temps>;<durée>;<empreinte>;<mesures>;<champs>;<sites>,<rayon>;<largeur>,<hauteur>,<pixel(0,0)>,<compacité>,<code de givre_nette_preparer(64, 2)>,<givre_neige_reglage(7)>,<givre_neige_mesure(0)>` :
//                ce que rend `givre_avancer`, `givre_temps`, `givre_duree`, `lireEmpreinte(m)`, les onze mesures de `lireMesuresMetal` (position, temps, auMur 0/1, diverge 0/1, releve.temps, releve.position, releve.lignes,
//                releve.sommet, releve.rayon, vitesse, selection), le SHA-256 des bits de `φ` puis de `u` lus par `lireChampsMetal` (8 octets petit-boutistes par valeur), `givre_sites`, `givre_rayon`, et ce que rendent les
//                fonctions d'avant pour le métal (non signés ; la compacité en bits) ;
//   cases      : `givre_metal_case` pour `(0, cote² − 1)`, `(1, cote² − 1)`, `(0, cote²)`, `(1, 4294967295)`, `(2, 0)`, `(4294967295, 0)`, et `givre_metal_mesure(11)`, `givre_metal_reglage(15)` : deux valeurs, puis six `NaN` ;
//   image      : `<largeur>x<hauteur>,<sorte>,<SHA-256 des octets rgba>` de `imageDuMetal(lireChampsMetal(m), taille)` ;
//   autre      : après `givre_demarrer(1, 0)` : `<ce que rend demarrer>,<givre_metal_reglage(0)>,<givre_metal_mesure(1)>,<givre_metal_case(0, 0)>,<lireReglagesMetal>,<lireMesuresMetal>,<lireChampsMetal>` : `1,NaN,NaN,NaN,null,null,null`.
//
// Si le fichier, `lecture.js` ou les arguments sont faux, si une fonction manque, ou si `givre_metal_demarrer` ne rend pas 0, écrit un message sur la sortie d'erreur, sort avec le code 1 et n'écrit **rien** sur la sortie standard.
// Aucune dépendance : les modules intégrés de Node seulement.

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
  "givre_largeur",
  "givre_hauteur",
  "givre_pixel",
  "givre_sites",
  "givre_rayon",
  "givre_compacite",
  "givre_empreinte_mot",
  "givre_nette_preparer",
  "givre_neige_reglage",
  "givre_neige_mesure",
  "givre_metal_demarrer",
  "givre_metal_reglage",
  "givre_metal_mesure",
  "givre_metal_case",
];

/** Lit un entier de 32 bits non signé écrit en chiffres décimaux ; échoue sinon. */
function entier32(texte, nom) {
  if (!/^[0-9]+$/.test(texte) || Number(texte) > MAX_U32) {
    throw new Error(`${nom} doit être un entier de 0 à ${MAX_U32} : « ${texte} »`);
  }
  return Number(texte);
}

/** Les bits d'un f64 en hexadécimal (16 chiffres, gros-boutiste), ou `NaN`. */
function hexa(x) {
  if (Number.isNaN(x)) {
    return "NaN";
  }
  const vue = new DataView(new ArrayBuffer(8));
  vue.setFloat64(0, x);
  return vue.getBigUint64(0).toString(16).padStart(16, "0");
}

/** Le SHA-256, en hexadécimal, de ces octets. */
function sha256(octets) {
  return createHash("sha256").update(octets).digest("hex");
}

/** Le SHA-256 des bits de `phi` puis de `u`, 8 octets petit-boutistes par valeur. */
function hacheDesChamps(champs) {
  const octets = Buffer.alloc(16 * champs.phi.length);
  let k = 0;
  for (const tableau of [champs.phi, champs.u]) {
    for (const v of tableau) {
      octets.writeDoubleLE(v, k);
      k += 8;
    }
  }
  return sha256(octets);
}

async function principal() {
  const args = process.argv.slice(2);
  if (args.length !== 6) {
    throw new Error("usage : node metal-wasm.mjs <fichier.wasm> <lecture.js> <epsilon_ppm> <cote> <morceaux> <taille>");
  }
  const epsilon = entier32(args[2], "epsilon");
  const cote = entier32(args[3], "le côté");
  const morceaux = args[4].split(",").map((texte) => entier32(texte, "un morceau"));
  const taille = entier32(args[5], "la taille");

  const { instance } = await WebAssembly.instantiate(readFileSync(args[0]), {});
  const m = instance.exports;
  for (const nom of EXPORTATIONS) {
    if (typeof m[nom] !== "function") {
      throw new Error(`le module n'exporte pas la fonction ${nom}`);
    }
  }
  const lecture = await import(pathToFileURL(resolve(args[1])).href);
  for (const nom of ["lireEmpreinte", "lireReglagesMetal", "lireMesuresMetal", "lireChampsMetal", "imageDuMetal"]) {
    if (typeof lecture[nom] !== "function") {
      throw new Error(`lecture.js n'exporte pas la fonction ${nom}`);
    }
  }
  const sorte = (valeur) => (valeur === null ? "null" : "objet");
  const bruts = () => [hexa(m.givre_metal_reglage(0)), hexa(m.givre_metal_mesure(1)), hexa(m.givre_metal_case(0, 0)), m.givre_temps() >>> 0, m.givre_duree() >>> 0];
  const lectures = () => [sorte(lecture.lireReglagesMetal(m)), sorte(lecture.lireMesuresMetal(m)), sorte(lecture.lireChampsMetal(m))];

  const sortie = [];
  sortie.push(`avant=${[...bruts(), ...lectures()].join(",")}`);
  const refus = [
    [60001, cote],
    [epsilon, 127],
    [epsilon, 257],
    [MAX_U32, MAX_U32],
    [epsilon, 0],
    [60001, 0],
  ].map(([e, c]) => m.givre_metal_demarrer(e, c) >>> 0);
  sortie.push(`refus=${refus.join(",")}`);
  sortie.push(`apres_refus=${bruts().join(",")}`);

  const demarre = m.givre_metal_demarrer(epsilon, cote) >>> 0;
  if (demarre !== 0) {
    throw new Error(`givre_metal_demarrer(${epsilon}, ${cote}) ne rend pas 0 : ${demarre}`);
  }
  sortie.push(`demarre=${demarre}`);
  const r = lecture.lireReglagesMetal(m);
  const cles = ["epsilon", "delta", "d0", "diffusivite", "w0", "lambda", "tau0", "dx", "dt", "rayon", "cote", "limite", "duree", "releve", "profondeur"];
  sortie.push(`reglages=${cles.map((cle) => hexa(r[cle])).join(",")}`);

  for (const morceau of morceaux) {
    const joue = m.givre_avancer(morceau) >>> 0;
    const temps = m.givre_temps() >>> 0;
    const duree = m.givre_duree() >>> 0;
    const empreinte = lecture.lireEmpreinte(m);
    const me = lecture.lireMesuresMetal(m);
    const mesures = [me.position, me.temps, me.auMur ? 1 : 0, me.diverge ? 1 : 0, me.releve.temps, me.releve.position, me.releve.lignes, me.releve.sommet, me.releve.rayon, me.vitesse, me.selection]
      .map(hexa)
      .join(",");
    const champs = hacheDesChamps(lecture.lireChampsMetal(m));
    const sites = `${m.givre_sites() >>> 0},${m.givre_rayon() >>> 0}`;
    const avant = [m.givre_largeur() >>> 0, m.givre_hauteur() >>> 0, m.givre_pixel(0, 0) >>> 0, hexa(m.givre_compacite()), m.givre_nette_preparer(64, 2) >>> 0, m.givre_neige_reglage(7) >>> 0, m.givre_neige_mesure(0) >>> 0].join(",");
    sortie.push(`morceau=${joue};${temps};${duree};${empreinte};${mesures};${champs};${sites};${avant}`);
  }
  const n = cote * cote;
  const cases = [m.givre_metal_case(0, n - 1), m.givre_metal_case(1, n - 1), m.givre_metal_case(0, n), m.givre_metal_case(1, MAX_U32), m.givre_metal_case(2, 0), m.givre_metal_case(MAX_U32, 0), m.givre_metal_mesure(11), m.givre_metal_reglage(15)];
  sortie.push(`cases=${cases.map(hexa).join(",")}`);
  const image = lecture.imageDuMetal(lecture.lireChampsMetal(m), taille);
  const rgba = sha256(Buffer.from(image.rgba.buffer, image.rgba.byteOffset, image.rgba.byteLength));
  sortie.push(`image=${image.largeur}x${image.hauteur},${image.rgba.constructor.name},${rgba}`);
  const autre = m.givre_demarrer(1, 0) >>> 0;
  sortie.push(`autre=${[autre, hexa(m.givre_metal_reglage(0)), hexa(m.givre_metal_mesure(1)), hexa(m.givre_metal_case(0, 0)), ...lectures()].join(",")}`);
  process.stdout.write(`${sortie.join("\n")}\n`);
}

principal().catch((erreur) => {
  process.stderr.write(`metal-wasm.mjs : ${erreur.message}\n`);
  process.exit(1);
});

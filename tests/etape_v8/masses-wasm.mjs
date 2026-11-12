// Fait lire les masses de la neige par le module WebAssembly de la vitrine (`givre_neige_masses_preparer`, `givre_neige_masse`), comme le fera la page, et écrit tout ce que le module en dit. L'image nette est redessinée **à partir des seules masses** par le
// rastériseur de référence de `reference-masses.mjs` (écrit d'après la doc, en `BigInt`) et comparée à `givre_nette_pixel`.
//
// Usage : node masses-wasm.mjs <fichier.wasm> <jeu> <rho> <beta> <kappa> <mu> <pas> <cote> <sur>
//
// <jeu> est le numéro d'un jeu nommé (0 à 18) ; <rho>, <beta>, <kappa> et <mu> les quatre réglages en millionièmes ; <pas> le nombre de pas demandés à un seul appel de `givre_avancer` ; <cote> et <sur> les réglages de l'image nette. Tous sont des entiers de 32 bits
// non signés (chiffres décimaux seulement). Le script charge le module (sans aucune importation) et écrit sur la sortie standard, une par ligne, `clé=valeur`, dans cet ordre (tous les nombres sont lus non signés, `>>> 0`) :
//
//   avant         : sur le module NEUF : `<givre_neige_masses_preparer()>,<masse(0)>,<masse(15150)>,<masse(30301)>,<masse(u32::MAX)>` : `0,4294967295,4294967295,4294967295,4294967295` (pas de neige : 0 case, rien à lire) ;
//   demarre       : ce que rend `givre_neige_demarrer(jeu, rho, beta, kappa, mu)` ;
//   non_preparee  : `<masse(15150)>,<masse(0)>` juste après le démarrage, **sans préparation** : `4294967295,4294967295` ;
//   joue          : ce que rend `givre_avancer(pas)` ;
//   nombre        : ce que rend `givre_neige_masses_preparer()` : 30301 ;
//   masses        : `<sha256>,<somme>,<non_nulles>,<max>` des 30301 masses lues une à une par `givre_neige_masse(i)` : le SHA-256 de leurs octets (quatre octets chacune, petit-boutiste), leur somme, le nombre de masses non nulles, la plus grande ;
//   hors          : `givre_neige_masse` pour `i` = 30301, 30302, 65535, 2^31, 2^32 − 2 et 2^32 − 1 : six `4294967295` ;
//   joue_de_plus  : ce que rend `givre_avancer(37)` (37, ou moins si la durée est atteinte ou si la neige est au mur) ;
//   gele          : comme `masses`, relu APRÈS ce `givre_avancer(37)` sans préparer : **l'instantané ne bouge pas** (la même ligne que `masses`) ;
//   nombre_2      : ce que rend une seconde `givre_neige_masses_preparer()` : 30301 ;
//   masses_2      : comme `masses`, relu après cette seconde préparation (l'état au temps `pas` + ce qui s'est joué de plus) ;
//   refus_garde   : `<ce que rend givre_neige_demarrer(19, rho, beta, kappa, mu)>,<1 si masse(15150) est inchangée, 0 sinon>` : `1,1` (un démarrage refusé ne change rien) ;
//   temps         : `<givre_temps()>,<givre_duree()>` : le temps joué (`joue` + `joue_de_plus`) : la préparation ne fait pas avancer l'état ;
//   nette         : le code de `givre_nette_preparer(cote, sur)` ;
//   pixels        : le SHA-256 des octets R, V, B lus **directement** par `givre_nette_pixel(x, y)`, ligne par ligne de haut en bas ;
//   reference     : le SHA-256 des octets R, V, B que le rastériseur de référence redessine **à partir de `masses_2`** (les masses lues par `givre_neige_masse`, `3 R (R + 1) + 1` avec `R = 100`, la vue de rayon 69) ;
//   ecarts        : le nombre d'octets où `pixels` et `reference` diffèrent : 0 ;
//   redemarre     : `<ce que rend givre_neige_demarrer (les mêmes réglages)>,<masse(15150)>,<masse(0)>` : `0,4294967295,4294967295` (démarrer efface l'instantané) ;
//   apres_redemarre : `<givre_neige_masses_preparer()>` : 30301 (la préparation refait l'instantané de la neige démarrée) ;
//   autre         : `<givre_demarrer(1, 0)>,<masse(15150)>,<masse(0)>,<givre_neige_masses_preparer()>,<masse(15150)>` : `1,4294967295,4294967295,0,4294967295` : les deux premières masses sont lues **avant toute préparation** (l'instantané de la neige existait : `givre_demarrer` d'un autre modèle l'efface
//                   à lui seul), puis la préparation rend 0 (le modèle en cours n'est plus une neige) ;
//   huit          : ce que rend `givre_demarrer(8, 0)` : 0 (le modèle 8 n'existe pas).
//
// **L'ordre des lectures compte** : l'instantané est lu avant que `givre_avancer(37)` ne joue (`masses`), après (`gele`, sans préparer), puis après la seconde préparation (`masses_2`), l'état est lu avant les démarrages de `redemarre` et d'`autre`, qui viennent en dernier.
// Sur la sortie d'erreur (pas sur la sortie standard : une information, jamais une assertion) : le temps de la préparation et des 30301 lectures sous ce Node.
//
// Si le fichier ou les arguments sont faux, si une fonction manque, ou si `givre_neige_demarrer` ne rend pas 0 pour ce cas, écrit un message sur la sortie d'erreur, sort avec le code 1 et n'écrit **rien** sur la sortie standard. Aucune dépendance : les modules intégrés de Node seulement.

import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import { rasteriser } from "./reference-masses.mjs";

const MAX_U32 = 4294967295;
const CASES = 30301;
const EXPORTATIONS = [
  "givre_demarrer",
  "givre_avancer",
  "givre_temps",
  "givre_duree",
  "givre_nette_preparer",
  "givre_nette_largeur",
  "givre_nette_hauteur",
  "givre_nette_pixel",
  "givre_neige_demarrer",
  "givre_neige_masses_preparer",
  "givre_neige_masse",
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
  if (args.length !== 9) {
    throw new Error("usage : node masses-wasm.mjs <fichier.wasm> <jeu> <rho> <beta> <kappa> <mu> <pas> <cote> <sur>");
  }
  const jeu = entier32(args[1], "le jeu");
  const rho = entier32(args[2], "rho");
  const beta = entier32(args[3], "beta");
  const kappa = entier32(args[4], "kappa");
  const mu = entier32(args[5], "mu");
  const pas = entier32(args[6], "le nombre de pas");
  const cote = entier32(args[7], "le côté");
  const sur = entier32(args[8], "le suréchantillonnage");

  const { instance } = await WebAssembly.instantiate(readFileSync(args[0]), {});
  const m = instance.exports;
  for (const nom of EXPORTATIONS) {
    if (typeof m[nom] !== "function") {
      throw new Error(`le module n'exporte pas la fonction ${nom}`);
    }
  }
  const masse = (i) => m.givre_neige_masse(i) >>> 0;
  const preparer = () => m.givre_neige_masses_preparer() >>> 0;

  /** Les 30301 masses lues une à une par `givre_neige_masse`, et ce qu'on en écrit : `sha256,somme,non nulles,max`. */
  const lire = () => {
    const masses = new Array(CASES);
    for (let i = 0; i < CASES; i++) {
      masses[i] = masse(i);
    }
    const octets = Buffer.alloc(4 * CASES);
    let somme = 0;
    let nonNulles = 0;
    let max = 0;
    masses.forEach((v, i) => {
      octets.writeUInt32LE(v, 4 * i);
      somme += v;
      nonNulles += v > 0 ? 1 : 0;
      max = Math.max(max, v);
    });
    return { masses, ligne: `${sha256(octets)},${somme},${nonNulles},${max}` };
  };

  const sortie = [];
  // Le module NEUF : rien n'est démarré.
  sortie.push(`avant=${[preparer(), masse(0), masse(15150), masse(30301), masse(MAX_U32)].join(",")}`);

  const code = m.givre_neige_demarrer(jeu, rho, beta, kappa, mu) >>> 0;
  if (code !== 0) {
    throw new Error(`givre_neige_demarrer(${jeu}, ${rho}, ${beta}, ${kappa}, ${mu}) rend ${code} : ce cas doit démarrer`);
  }
  sortie.push(`demarre=${code}`);
  sortie.push(`non_preparee=${[masse(15150), masse(0)].join(",")}`);
  sortie.push(`joue=${m.givre_avancer(pas) >>> 0}`);
  const t0 = performance.now();
  sortie.push(`nombre=${preparer()}`);
  const t1 = performance.now();
  const premiere = lire();
  const t2 = performance.now();
  sortie.push(`masses=${premiere.ligne}`);
  sortie.push(`hors=${[30301, 30302, 65535, 2 ** 31, 2 ** 32 - 2, MAX_U32].map(masse).join(",")}`);
  sortie.push(`joue_de_plus=${m.givre_avancer(37) >>> 0}`);
  sortie.push(`gele=${lire().ligne}`);
  sortie.push(`nombre_2=${preparer()}`);
  const seconde = lire();
  sortie.push(`masses_2=${seconde.ligne}`);
  const apresRefus = m.givre_neige_demarrer(19, rho, beta, kappa, mu) >>> 0;
  sortie.push(`refus_garde=${apresRefus},${masse(15150) === seconde.masses[15150] ? 1 : 0}`);
  sortie.push(`temps=${[m.givre_temps() >>> 0, m.givre_duree() >>> 0].join(",")}`);

  sortie.push(`nette=${m.givre_nette_preparer(cote, sur) >>> 0}`);
  const pixels = Buffer.alloc(3 * cote * cote);
  for (let y = 0; y < cote; y++) {
    for (let x = 0; x < cote; x++) {
      const c = m.givre_nette_pixel(x, y) >>> 0;
      const d = 3 * (y * cote + x);
      pixels[d] = (c >>> 16) & 255;
      pixels[d + 1] = (c >>> 8) & 255;
      pixels[d + 2] = c & 255;
    }
  }
  const reference = Buffer.from(rasteriser(100, seconde.masses, cote, 69, sur));
  let ecarts = 0;
  for (let i = 0; i < pixels.length; i++) {
    ecarts += pixels[i] !== reference[i] ? 1 : 0;
  }
  sortie.push(`pixels=${sha256(pixels)}`);
  sortie.push(`reference=${sha256(reference)}`);
  sortie.push(`ecarts=${ecarts}`);

  const redemarre = m.givre_neige_demarrer(jeu, rho, beta, kappa, mu) >>> 0;
  sortie.push(`redemarre=${[redemarre, masse(15150), masse(0)].join(",")}`);
  sortie.push(`apres_redemarre=${preparer()}`);
  const autre = m.givre_demarrer(1, 0) >>> 0;
  const sansPreparer = [masse(15150), masse(0)];
  sortie.push(`autre=${[autre, ...sansPreparer, preparer(), masse(15150)].join(",")}`);
  sortie.push(`huit=${m.givre_demarrer(8, 0) >>> 0}`);

  console.error(`information : préparation ${(t1 - t0).toFixed(2)} ms ; ${CASES} lectures de givre_neige_masse ${(t2 - t1).toFixed(2)} ms`);
  process.stdout.write(sortie.join("\n") + "\n");
}

principal().catch((erreur) => {
  console.error(String(erreur && erreur.message ? erreur.message : erreur));
  process.exit(1);
});

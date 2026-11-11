// Fait jouer la neige par le module WebAssembly de la vitrine et la fait lire par `docs/lecture.js`, comme le fait la page, et écrit tout ce que le module en dit.
//
// Usage : node neige-wasm.mjs <fichier.wasm> <lecture.js> <jeu> <rho> <beta> <kappa> <mu> <pas> <cote> <sur>
//
// <jeu> est le numéro d'un jeu nommé (0 à 18 : voir `src/vitrine.rs`, « La neige ») ; <rho>, <beta>, <kappa> et <mu> les quatre réglages **en millionièmes** (dans cet ordre) ; <pas> le nombre de pas demandés à un seul appel de
// `givre_avancer` ; <cote> et <sur> les réglages de l'image nette. Tous sont des entiers de 32 bits non signés (chiffres décimaux seulement : le module, lui, refuse un réglage hors plage par un code). Le script charge le module (sans
// aucune importation) et `lecture.js` (par son adresse `file:`, ce qui marche sous Linux comme sous Windows). Il écrit sur la sortie standard, une par ligne, `clé=valeur`, dans cet ordre :
//
//   avant        : sur le module NEUF, `<givre_neige_jeux>,<givre_neige_jeu(0, 0)>,<givre_neige_reglage(7)>,<givre_neige_mesure(0)>,<givre_nette_preparer(cote, sur)>` (non signés) : `19,9,4294967295,4294967295,2` ;
//   jeux         : les dix-neuf jeux lus par `lireJeuxNeige` de `lecture.js`, chacun `figure.rang.cristaux.balaye:rho,beta,alpha,theta,kappa,mu,gamma:rayon:pas:deduit` (en millionièmes ; `deduit` 0 ou 1), séparés par `|` ;
//   refus        : les codes de `givre_neige_demarrer` pour huit démarrages refusés (le jeu 19 ; `u32::MAX` partout ; `ρ`, `β` (deux fois), `κ`, `μ` hors plage ; les quatre hors plage), les autres réglages étant ceux du jeu 0, séparés par des virgules ;
//   apres_refus  : sur le module neuf encore, après ces refus : `<givre_neige_reglage(7)>,<givre_neige_mesure(0)>,<givre_nette_preparer(cote, sur)>` : `4294967295,4294967295,2` (rien n'est démarré) ;
//   demarre      : ce que rend `givre_neige_demarrer(jeu, rho, beta, kappa, mu)` ;
//   reglage      : les dix champs de `lireReglagesNeige` : `rho,beta,alpha,theta,kappa,mu,gamma,jeu,rayon,pas` ;
//   joue         : ce que rend `givre_avancer(pas)` ;
//   etat         : `<temps>,<duree>,<sites>,<rayon>,<empreinte>,<largeur>x<hauteur>,<rgba>` : `givre_temps`, `givre_duree`, `givre_sites`, `givre_rayon` (non signés), `lireEmpreinte(m)`, les dimensions et le SHA-256 des octets de
//                  `lireImage(m).rgba` (l'image d'un pixel par case) ;
//   mesure       : les huit champs de `lireMesuresNeige` : `rattachees,rayon,compacite,pointes,aiguilles,hexagonalite,forme,auMur` (`auMur` écrit 0 ou 1) ;
//   vide         : `<largeur>,<hauteur>,<pixel>` : le tampon de l'image nette AVANT toute préparation, après le démarrage et les pas : `0,0,4294967295` ;
//   code         : le code de `lireImageNette(m, cote, sur)` ;
//   dimensions   : `<largeur>x<hauteur>` de ce que `lireImageNette` rend ;
//   rgba         : le SHA-256 des octets `rgba` que `lireImageNette` rend ;
//   pixels       : le SHA-256 des octets R, V, B lus **directement** par `givre_nette_pixel(x, y)` (sans `lecture.js`), ligne par ligne de haut en bas ;
//   hors         : `givre_nette_pixel` (non signé) pour `(largeur, 0)`, `(0, hauteur)`, `(largeur, hauteur)`, `(4294967295, 0)` et `(0, 4294967295)` : cinq `4294967295` ;
//   etat_apres   : comme `etat`, relu APRÈS la préparation : la préparation ne change rien à l'état ;
//   refus_nette  : les codes de `givre_nette_preparer` pour `(1, 2)`, `(cote, 0)`, `(cote, 5)` et `(2048, 3)` (côté impair, `sur` nul, `sur` trop grand, trop d'échantillons) : `3,4,4,5` ;
//   apres_refus_nette : `<largeur>,<hauteur>,<pixel>` du tampon juste après le dernier refus : `0,0,4294967295` (un refus efface le tampon) ;
//   efface       : après `lireImageNette(m, cote, sur)` puis un `givre_avancer(1)` : `<ce que rend avancer>,<largeur>,<hauteur>,<pixel>` du tampon (effacé si un pas a été joué, gardé sinon : la durée atteinte ou le mur) ;
//   redemarre    : après `lireImageNette(m, cote, sur)` puis un nouveau `givre_neige_demarrer` (les mêmes réglages) : `<code>,<largeur>,<hauteur>,<pixel>` du tampon (effacé : `0,0,0,4294967295`) ;
//   autre        : après `givre_demarrer(1, 0)` : `<ce que rend demarrer>,<givre_neige_reglage(7)>,<givre_neige_mesure(0)>,<givre_neige_jeu(0, 0)>` : `1,4294967295,4294967295,9` (le modèle en cours n'est plus une neige ; les jeux ne dépendent d'aucun état) ;
//   huit         : ce que rend `givre_demarrer(8, 0)` : 0 (la neige n'est pas un numéro de `givre_demarrer`).
//
// **L'ordre des lectures compte** : l'état (`etat`, `etat_apres`) et les mesures sont lus avant tout appel qui les change (`givre_avancer(1)` de `efface`, les démarrages de `redemarre` et d'`autre`, qui viennent en dernier).
//
// Si le fichier, `lecture.js` ou les arguments sont faux, si une fonction manque, ou si `givre_neige_demarrer` ne rend pas 0, écrit un message sur la sortie d'erreur, sort avec le code 1 et n'écrit **rien** sur la sortie
// standard. Aucune dépendance : les modules intégrés de Node seulement.

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
  "givre_neige_demarrer",
  "givre_neige_jeux",
  "givre_neige_jeu",
  "givre_neige_reglage",
  "givre_neige_mesure",
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
  if (args.length !== 10) {
    throw new Error("usage : node neige-wasm.mjs <fichier.wasm> <lecture.js> <jeu> <rho> <beta> <kappa> <mu> <pas> <cote> <sur>");
  }
  const jeu = entier32(args[2], "le jeu");
  const rho = entier32(args[3], "rho");
  const beta = entier32(args[4], "beta");
  const kappa = entier32(args[5], "kappa");
  const mu = entier32(args[6], "mu");
  const pas = entier32(args[7], "le nombre de pas");
  const cote = entier32(args[8], "le côté");
  const sur = entier32(args[9], "le suréchantillonnage");

  const { instance } = await WebAssembly.instantiate(readFileSync(args[0]), {});
  const m = instance.exports;
  for (const nom of EXPORTATIONS) {
    if (typeof m[nom] !== "function") {
      throw new Error(`le module n'exporte pas la fonction ${nom}`);
    }
  }
  const lecture = await import(pathToFileURL(resolve(args[1])).href);
  for (const nom of ["lireEmpreinte", "lireImage", "lireImageNette", "lireJeuxNeige", "lireReglagesNeige", "lireMesuresNeige"]) {
    if (typeof lecture[nom] !== "function") {
      throw new Error(`lecture.js n'exporte pas la fonction ${nom}`);
    }
  }

  /** Le tampon de l'image nette tel que le module le garde : largeur, hauteur et pixel (0, 0), non signés. */
  const tampon = () => [m.givre_nette_largeur() >>> 0, m.givre_nette_hauteur() >>> 0, m.givre_nette_pixel(0, 0) >>> 0];

  // Le module NEUF : rien n'est démarré.
  const avant = [m.givre_neige_jeux() >>> 0, m.givre_neige_jeu(0, 0) >>> 0, m.givre_neige_reglage(7) >>> 0, m.givre_neige_mesure(0) >>> 0, m.givre_nette_preparer(cote, sur) >>> 0];

  const jeux = lecture
    .lireJeuxNeige(m)
    .map((j) => {
      const r = j.reglages;
      return `${j.figure}.${j.rang}.${j.cristaux}.${j.balaye}:${[r.rho, r.beta, r.alpha, r.theta, r.kappa, r.mu, r.gamma].join(",")}:${j.rayon}:${j.pas}:${j.deduit ? 1 : 0}`;
    })
    .join("|");

  // Huit démarrages refusés, sur le module neuf : les réglages valides sont ceux du jeu 0 (400000, 1300000, 3000, 70000).
  const refus = [
    [19, 400000, 1300000, 3000, 70000],
    [MAX_U32, MAX_U32, MAX_U32, MAX_U32, MAX_U32],
    [0, 1000001, 1300000, 3000, 70000],
    [0, 400000, 999999, 3000, 70000],
    [0, 400000, 8000001, 3000, 70000],
    [0, 400000, 1300000, 1000001, 70000],
    [0, 400000, 1300000, 3000, 1000001],
    [0, MAX_U32, MAX_U32, MAX_U32, MAX_U32],
  ].map((a) => m.givre_neige_demarrer(...a) >>> 0);
  const apresRefus = [m.givre_neige_reglage(7) >>> 0, m.givre_neige_mesure(0) >>> 0, m.givre_nette_preparer(cote, sur) >>> 0];

  const demarre = m.givre_neige_demarrer(jeu, rho, beta, kappa, mu) >>> 0;
  if (demarre !== 0) {
    throw new Error(`givre_neige_demarrer(${jeu}, ${rho}, ${beta}, ${kappa}, ${mu}) ne rend pas 0 : ${demarre}`);
  }
  const reglages = lecture.lireReglagesNeige(m);
  const r = reglages.reglages;
  const reglage = [r.rho, r.beta, r.alpha, r.theta, r.kappa, r.mu, r.gamma, reglages.jeu, reglages.rayon, reglages.pas];
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

  // TOUT ce qui décrit l'état est lu ICI, avant les appels qui le changent (`efface`, `redemarre`, `autre`, plus bas).
  const etatAvant = etat();
  const mesures = lecture.lireMesuresNeige(m);
  const mesure = [mesures.rattachees, mesures.rayon, mesures.compacite, mesures.pointes, mesures.aiguilles, mesures.hexagonalite, mesures.forme, mesures.auMur ? 1 : 0];
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

  // Des préparations refusées (côté impair, `sur` nul ou trop grand, trop d'échantillons) : un refus efface le tampon.
  const refusNette = [
    [1, 2],
    [cote, 0],
    [cote, 5],
    [2048, 3],
  ].map(([c, s]) => m.givre_nette_preparer(c, s) >>> 0);
  const apresRefusNette = tampon();

  // Les appels qui changent l'état viennent en dernier.
  lecture.lireImageNette(m, cote, sur);
  const jouee = m.givre_avancer(1) >>> 0;
  const efface = [jouee, ...tampon()];
  lecture.lireImageNette(m, cote, sur);
  const redemarre = [m.givre_neige_demarrer(jeu, rho, beta, kappa, mu) >>> 0, ...tampon()];
  const autre = [m.givre_demarrer(1, 0) >>> 0, m.givre_neige_reglage(7) >>> 0, m.givre_neige_mesure(0) >>> 0, m.givre_neige_jeu(0, 0) >>> 0];
  const huit = m.givre_demarrer(8, 0) >>> 0;

  const lignes = [
    `avant=${avant.join(",")}`,
    `jeux=${jeux}`,
    `refus=${refus.join(",")}`,
    `apres_refus=${apresRefus.join(",")}`,
    `demarre=${demarre}`,
    `reglage=${reglage.join(",")}`,
    `joue=${joue}`,
    `etat=${etatAvant}`,
    `mesure=${mesure.join(",")}`,
    `vide=${vide.join(",")}`,
    `code=${nette.code}`,
    `dimensions=${nette.largeur}x${nette.hauteur}`,
    `rgba=${rgba}`,
    `pixels=${pixels}`,
    `hors=${hors.join(",")}`,
    `etat_apres=${etatApres}`,
    `refus_nette=${refusNette.join(",")}`,
    `apres_refus_nette=${apresRefusNette.join(",")}`,
    `efface=${efface.join(",")}`,
    `redemarre=${redemarre.join(",")}`,
    `autre=${autre.join(",")}`,
    `huit=${huit}`,
  ];
  process.stdout.write(lignes.join("\n") + "\n");
}

principal().catch((erreur) => {
  process.stderr.write(`neige-wasm : ${erreur.message}\n`);
  process.exitCode = 1;
});

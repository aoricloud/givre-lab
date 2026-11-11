// Lit l'état d'une simulation par les accesseurs du module WebAssembly de la vitrine, comme le fait la page.
//
// Usage : node etat-wasm.mjs <fichier.wasm> <modele> <graine> <pas>
//
// Complète `empreinte-wasm.mjs`, qui ne lit que l'empreinte de l'état interne : la page, elle, ne lit que les accesseurs
// (temps, image, compteurs). <modele> est un entier de 0 à 3, <graine> et <pas> des entiers de 32 bits non signés (chiffres
// décimaux seulement). Le script charge le module (sans aucune importation) ; **avant tout démarrage** il vérifie le contrat
// « avant tout démarrage » de `src/vitrine.rs` ; puis il démarre le modèle avec la graine, l'avance de <pas> unités d'un seul
// appel, et écrit sur la sortie standard, une par ligne, `clé=valeur`, dans cet ordre :
//
//   avant        : 1 si, sur le module neuf, `givre_avancer(5)`, `givre_temps`, `givre_duree`, `givre_largeur`, `givre_hauteur`,
//                  `givre_sites`, `givre_rayon` et `givre_empreinte_mot(0)` rendent 0, si `givre_pixel(0, 0)` rend 4294967295 et
//                  si `givre_compacite` rend NaN ; sinon 0 ;
//   demarre      : ce que rend `givre_demarrer(modele, graine)` ;
//   depart       : `<largeur>x<hauteur>`, les dimensions de l'image lues juste après le démarrage, AVANT le premier pas ;
//   hache_depart : le SHA-256 de cette image du temps 0, lue pixel par pixel comme `hache_image` (une image gardée d'avant un
//                  `givre_avancer` ne se voit que si elle a été lue avant : c'est le but de cette lecture) ;
//   joue         : ce que rend `givre_avancer(pas)` ;
//   temps, duree, largeur, hauteur, sites, rayon : les entiers, en décimal ;
//   compacite    : `NaN` si l'accesseur rend NaN, sinon les 8 octets du f64, petit-boutiste, en hexadécimal ;
//   hache_image  : le SHA-256 (64 caractères hexadécimaux minuscules) des octets R, V, B de chaque pixel lu par `givre_pixel`,
//                  ligne par ligne (y de 0 à hauteur - 1 et, pour chaque y, x de 0 à largeur - 1), trois octets par pixel,
//                  chacun lu dans `0x00RRGGBB` (R : bits 16 à 23, V : bits 8 à 15, B : bits 0 à 7) ;
//   couleurs     : 1 si tous les pixels lus (ceux du temps 0 et ceux de la fin) sont à `0x00RRGGBB` (au plus 0xFFFFFF), sinon 0 ;
//   bords        : 1 si `givre_pixel` rend 4294967295 pour (largeur, 0), (0, hauteur), (largeur, hauteur), (4294967295, 0) et
//                  (0, 4294967295), si `givre_empreinte_mot` rend 0 pour 8 et pour 4294967295, et si `givre_demarrer` rend 0
//                  pour les modèles 8 et 4294967295 sans changer `givre_temps` ; sinon 0.
//
// Si le fichier ou les arguments sont faux, ou si une exportation manque, écrit un message sur la sortie d'erreur, sort avec
// le code 1 et n'écrit **rien** sur la sortie standard. Aucune dépendance : les modules intégrés de Node seulement.

import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";

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
];
const MAX_U32 = 4294967295;

/** Lit un entier de 32 bits non signé écrit en chiffres décimaux ; échoue sinon. */
function entier32(texte, nom) {
  if (!/^[0-9]+$/.test(texte) || Number(texte) > MAX_U32) {
    throw new Error(`${nom} doit être un entier de 0 à ${MAX_U32} : « ${texte} »`);
  }
  return Number(texte);
}

/** Le SHA-256 en hexadécimal d'un tampon. */
function sha256(tampon) {
  return createHash("sha256").update(tampon).digest("hex");
}

/** Lit l'image courante pixel par pixel : les octets R, V, B, ligne par ligne, et si tous les pixels sont à 0x00RRGGBB. */
function lireImage(m, largeur, hauteur) {
  const rvb = Buffer.alloc(3 * largeur * hauteur);
  let couleurs = true;
  for (let y = 0; y < hauteur; y++) {
    for (let x = 0; x < largeur; x++) {
      const pixel = m.givre_pixel(x, y) >>> 0;
      if (pixel > 0xffffff) {
        couleurs = false;
      }
      const debut = 3 * (y * largeur + x);
      rvb[debut] = (pixel >>> 16) & 255;
      rvb[debut + 1] = (pixel >>> 8) & 255;
      rvb[debut + 2] = pixel & 255;
    }
  }
  return { rvb, couleurs };
}

async function principal() {
  const args = process.argv.slice(2);
  if (args.length !== 4) {
    throw new Error("usage : node etat-wasm.mjs <fichier.wasm> <modele> <graine> <pas>");
  }
  const modele = entier32(args[1], "le modèle");
  if (modele > 3) {
    throw new Error(`le modèle doit être 0, 1, 2 ou 3 : « ${args[1]} »`);
  }
  const graine = entier32(args[2], "la graine");
  const pas = entier32(args[3], "le nombre de pas");

  const octets = readFileSync(args[0]);
  const { instance } = await WebAssembly.instantiate(octets, {});
  const m = instance.exports;
  for (const nom of EXPORTATIONS) {
    if (typeof m[nom] !== "function") {
      throw new Error(`le module n'exporte pas la fonction ${nom}`);
    }
  }

  // Avant tout démarrage. Les fonctions rendent des i32 : `>>> 0` les relit comme des entiers non signés.
  const avant =
    m.givre_avancer(5) >>> 0 === 0 &&
    m.givre_temps() >>> 0 === 0 &&
    m.givre_duree() >>> 0 === 0 &&
    m.givre_largeur() >>> 0 === 0 &&
    m.givre_hauteur() >>> 0 === 0 &&
    m.givre_sites() >>> 0 === 0 &&
    m.givre_rayon() >>> 0 === 0 &&
    m.givre_empreinte_mot(0) >>> 0 === 0 &&
    m.givre_pixel(0, 0) >>> 0 === MAX_U32 &&
    Number.isNaN(m.givre_compacite());

  const demarre = m.givre_demarrer(modele, graine) >>> 0;
  // L'image du temps 0, lue avant le premier pas : une image gardée d'avant `givre_avancer` se voit à la lecture de la fin.
  const largeurDepart = m.givre_largeur() >>> 0;
  const hauteurDepart = m.givre_hauteur() >>> 0;
  const imageDepart = lireImage(m, largeurDepart, hauteurDepart);
  const joue = m.givre_avancer(pas) >>> 0;

  const temps = m.givre_temps() >>> 0;
  const duree = m.givre_duree() >>> 0;
  const largeur = m.givre_largeur() >>> 0;
  const hauteur = m.givre_hauteur() >>> 0;
  const sites = m.givre_sites() >>> 0;
  const rayon = m.givre_rayon() >>> 0;

  const compacite = m.givre_compacite();
  let texteCompacite = "NaN";
  if (!Number.isNaN(compacite)) {
    const tampon = Buffer.alloc(8);
    tampon.writeDoubleLE(compacite);
    texteCompacite = tampon.toString("hex");
  }

  const imageFin = lireImage(m, largeur, hauteur);
  const couleurs = imageDepart.couleurs && imageFin.couleurs;

  const bords =
    m.givre_pixel(largeur, 0) >>> 0 === MAX_U32 &&
    m.givre_pixel(0, hauteur) >>> 0 === MAX_U32 &&
    m.givre_pixel(largeur, hauteur) >>> 0 === MAX_U32 &&
    m.givre_pixel(MAX_U32, 0) >>> 0 === MAX_U32 &&
    m.givre_pixel(0, MAX_U32) >>> 0 === MAX_U32 &&
    m.givre_empreinte_mot(8) >>> 0 === 0 &&
    m.givre_empreinte_mot(MAX_U32) >>> 0 === 0 &&
    m.givre_demarrer(8, 0) >>> 0 === 0 &&
    m.givre_demarrer(MAX_U32, 0) >>> 0 === 0 &&
    m.givre_temps() >>> 0 === temps;

  const lignes = [
    `avant=${avant ? 1 : 0}`,
    `demarre=${demarre}`,
    `depart=${largeurDepart}x${hauteurDepart}`,
    `hache_depart=${sha256(imageDepart.rvb)}`,
    `joue=${joue}`,
    `temps=${temps}`,
    `duree=${duree}`,
    `largeur=${largeur}`,
    `hauteur=${hauteur}`,
    `sites=${sites}`,
    `rayon=${rayon}`,
    `compacite=${texteCompacite}`,
    `hache_image=${sha256(imageFin.rvb)}`,
    `couleurs=${couleurs ? 1 : 0}`,
    `bords=${bords ? 1 : 0}`,
  ];
  process.stdout.write(lignes.join("\n") + "\n");
}

principal().catch((erreur) => {
  process.stderr.write(`etat-wasm : ${erreur.message}\n`);
  process.exitCode = 1;
});

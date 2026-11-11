// Fait appeler par Node les fonctions de `docs/lecture.js` de l'image nette sur un FAUX module, écrit ici, qui change d'état à chaque préparation : `lireImageNette` et `imagesCachees`.
//
// Usage : node lecture-pure.mjs <lecture.js>
//
// Ce script ne charge aucun module WebAssembly : il tourne dès que `lecture.js` est écrit. Le faux module est une image **non carrée** (`cote` de large, `cote + 2` de haut : une largeur
// et une hauteur échangées, ou un pixel `(x, y)` lu en `(y, x)`, changent l'image) dont la couleur du pixel `(x, y)` dépend du dernier `givre_nette_preparer(cote, sur)` (le module change d'état à
// chaque appel : lire avant de préparer, ou dans l'autre ordre des arguments, se voit) :
//
//   rouge = (3 x + 5 y + sur) & 255, vert = (7 x + 11 y) & 255, bleu = (13 x + 17 y + cote) & 255, rendus empaquetés en `(rouge << 16) | (vert << 8) | bleu` ;
//
// et un pixel hors de l'image vaut -1 (le `u32::MAX` lu signé par JavaScript, comme le fait un vrai module). Un `cote` impair est refusé (code 3, tampon vide). Il écrit sur la sortie standard,
// une par ligne, `clé=valeur`, dans cet ordre :
//
//   premier   : `lireImageNette(faux, 6, 2)` : `<code>,<largeur>x<hauteur>,<SHA-256 des octets rgba>,<sorte du tableau>` ;
//   second    : `lireImageNette(faux, 4, 3)` sur le même faux module (qui a changé d'état) : mêmes champs ;
//   refus     : `lireImageNette(faux, 5, 1)` (côté impair) : `<code>,<largeur>x<hauteur>,<longueur de rgba>,<nombre d'appels de givre_nette_pixel faits pendant ce refus>` ;
//   appels    : les appels du faux module hors `givre_nette_pixel`, dans l'ordre, pour les trois lectures, séparés par des virgules ;
//   cachees   : `imagesCachees(code)` pour 0, 1, 2, 3, 6 et 4294967295 : `<code>:<carree>,<nette>` (`true` : le canvas est caché), séparés par des barres verticales.
//
// Si le fichier ou les arguments sont faux, ou si une fonction manque, écrit un message sur la sortie d'erreur, sort avec le code 1 et n'écrit **rien** sur la sortie standard.
// Aucune dépendance : les modules intégrés de Node seulement.

import { createHash } from "node:crypto";
import { resolve } from "node:path";
import { pathToFileURL } from "node:url";

const FONCTIONS = ["lireImageNette", "imagesCachees"];

/** Le faux module : une image de `cote` de large et `cote + 2` de haut, qui dépend du dernier `givre_nette_preparer`. */
function fauxModule() {
  const appels = [];
  const etat = { pret: false, cote: 0, sur: 0, pixelsLus: 0 };
  const m = {
    appels,
    etat,
    givre_nette_preparer(cote, sur) {
      appels.push(`preparer(${cote},${sur})`);
      if (cote % 2 === 1) {
        etat.pret = false;
        return 3;
      }
      etat.pret = true;
      etat.cote = cote;
      etat.sur = sur;
      return 0;
    },
    givre_nette_largeur() {
      appels.push("largeur");
      return etat.pret ? etat.cote : 0;
    },
    givre_nette_hauteur() {
      appels.push("hauteur");
      return etat.pret ? etat.cote + 2 : 0;
    },
    givre_nette_pixel(x, y) {
      etat.pixelsLus++;
      if (!etat.pret || x >= etat.cote || y >= etat.cote + 2) {
        return -1;
      }
      const rouge = (3 * x + 5 * y + etat.sur) & 255;
      const vert = (7 * x + 11 * y) & 255;
      const bleu = (13 * x + 17 * y + etat.cote) & 255;
      return (rouge << 16) | (vert << 8) | bleu;
    },
  };
  return m;
}

/** Le SHA-256, en hexadécimal, des octets d'un tableau typé. */
function sha256(tableau) {
  return createHash("sha256").update(Buffer.from(tableau.buffer, tableau.byteOffset, tableau.byteLength)).digest("hex");
}

async function principal() {
  const args = process.argv.slice(2);
  if (args.length !== 1) {
    throw new Error("usage : node lecture-pure.mjs <lecture.js>");
  }
  const lecture = await import(pathToFileURL(resolve(args[0])).href);
  for (const nom of FONCTIONS) {
    if (typeof lecture[nom] !== "function") {
      throw new Error(`lecture.js n'exporte pas la fonction ${nom}`);
    }
  }
  const faux = fauxModule();
  const decrire = (image) => `${image.code},${image.largeur}x${image.hauteur},${sha256(image.rgba)},${image.rgba.constructor.name}`;
  const premier = decrire(lecture.lireImageNette(faux, 6, 2));
  const second = decrire(lecture.lireImageNette(faux, 4, 3));
  const avantRefus = faux.etat.pixelsLus;
  const refuse = lecture.lireImageNette(faux, 5, 1);
  const refus = `${refuse.code},${refuse.largeur}x${refuse.hauteur},${refuse.rgba.length},${faux.etat.pixelsLus - avantRefus}`;
  const cachees = [0, 1, 2, 3, 6, 4294967295]
    .map((code) => {
      const c = lecture.imagesCachees(code);
      return `${code}:${c.carree},${c.nette}`;
    })
    .join("|");
  const lignes = [`premier=${premier}`, `second=${second}`, `refus=${refus}`, `appels=${faux.appels.join(",")}`, `cachees=${cachees}`];
  process.stdout.write(lignes.join("\n") + "\n");
}

principal().catch((erreur) => {
  process.stderr.write(`lecture-pure : ${erreur.message}\n`);
  process.exitCode = 1;
});

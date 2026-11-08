// Fait lire le module WebAssembly de la vitrine par `docs/lecture.js`, la partie de la page qui lit l'empreinte, l'image et
// la graine : c'est elle que la page affiche et dessine.
//
// Usage : node lecture-wasm.mjs <fichier.wasm> <lecture.js> <modele> <graine> <pas>
//
// <modele> est un entier de 0 à 3, <graine> et <pas> des entiers de 32 bits non signés (chiffres décimaux seulement). Le
// script charge le module (sans aucune importation) et `lecture.js` (par son adresse `file:`, ce qui marche sous Linux comme
// sous Windows), démarre le modèle avec la graine par `givre_demarrer`, l'avance de <pas> unités d'un seul appel de
// `givre_avancer`, puis écrit sur la sortie standard, une par ligne, `clé=valeur`, dans cet ordre :
//
//   empreinte : ce que rend `lireEmpreinte(m)` ;
//   largeur, hauteur : les champs `largeur` et `hauteur` de ce que rend `lireImage(m)` ;
//   rgba : le SHA-256 (64 caractères hexadécimaux minuscules) des octets du champ `rgba`, dans l'ordre ;
//   sorte : `Uint8ClampedArray` si `rgba` en est un, sinon le nom de son constructeur ;
//   graines : ce que rend `lireGraine(texte)` pour chacun des textes de `TEXTES` ci-dessous, dans l'ordre, séparés par `;`
//             (`String(valeur)` : un nombre en décimal, ou `null`).
//
// Si le fichier, `lecture.js` ou les arguments sont faux, si une fonction manque, ou si `givre_demarrer` ne rend pas 1, écrit
// un message sur la sortie d'erreur, sort avec le code 1 et n'écrit **rien** sur la sortie standard. Aucune dépendance : les
// modules intégrés de Node seulement.

import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { pathToFileURL } from "node:url";

const MAX_U32 = 4294967295;

// Les textes donnés à `lireGraine` ; ce que la doc de la page en attend est dans `tests/etape_5_page.rs`.
const TEXTES = ["0", "1", " 42 ", "007", "4294967295", "4294967296", "-1", "", " ", "abc", "1.5", "1e3", "0x10", "+7"];

/** Lit un entier de 32 bits non signé écrit en chiffres décimaux ; échoue sinon. */
function entier32(texte, nom) {
  if (!/^[0-9]+$/.test(texte) || Number(texte) > MAX_U32) {
    throw new Error(`${nom} doit être un entier de 0 à ${MAX_U32} : « ${texte} »`);
  }
  return Number(texte);
}

async function principal() {
  const args = process.argv.slice(2);
  if (args.length !== 5) {
    throw new Error("usage : node lecture-wasm.mjs <fichier.wasm> <lecture.js> <modele> <graine> <pas>");
  }
  const modele = entier32(args[2], "le modèle");
  if (modele > 3) {
    throw new Error(`le modèle doit être 0, 1, 2 ou 3 : « ${args[2]} »`);
  }
  const graine = entier32(args[3], "la graine");
  const pas = entier32(args[4], "le nombre de pas");

  const { instance } = await WebAssembly.instantiate(readFileSync(args[0]), {});
  const m = instance.exports;
  const lecture = await import(pathToFileURL(resolve(args[1])).href);
  for (const nom of ["lireEmpreinte", "lireImage", "lireGraine"]) {
    if (typeof lecture[nom] !== "function") {
      throw new Error(`lecture.js n'exporte pas la fonction ${nom}`);
    }
  }

  if (m.givre_demarrer(modele, graine) >>> 0 !== 1) {
    throw new Error(`givre_demarrer(${modele}, ${graine}) ne rend pas 1`);
  }
  m.givre_avancer(pas);

  const empreinte = lecture.lireEmpreinte(m);
  const image = lecture.lireImage(m);
  const rgba = image.rgba;
  const octets = Buffer.from(rgba.buffer, rgba.byteOffset, rgba.byteLength);
  const sorte = rgba instanceof Uint8ClampedArray ? "Uint8ClampedArray" : String(rgba?.constructor?.name);
  const graines = TEXTES.map((texte) => String(lecture.lireGraine(texte)));

  const lignes = [
    `empreinte=${empreinte}`,
    `largeur=${image.largeur}`,
    `hauteur=${image.hauteur}`,
    `rgba=${createHash("sha256").update(octets).digest("hex")}`,
    `sorte=${sorte}`,
    `graines=${graines.join(";")}`,
  ];
  process.stdout.write(lignes.join("\n") + "\n");
}

principal().catch((erreur) => {
  process.stderr.write(`lecture-wasm : ${erreur.message}\n`);
  process.exitCode = 1;
});

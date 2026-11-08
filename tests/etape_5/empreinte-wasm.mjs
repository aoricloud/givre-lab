// Lit l'empreinte d'une simulation par le module WebAssembly de la vitrine, comme le fait la page.
//
// Usage : node empreinte-wasm.mjs <fichier.wasm> <modele> <graine> <pas>
//
// <modele> est un entier de 0 à 3 (0 la DLA, 1 la plaque, 2 l'étoile, 3 la dendrite : voir `src/vitrine.rs`), <graine> un
// entier de 32 bits non signé, <pas> une liste d'entiers de 32 bits non signés séparés par des virgules (`80`, ou
// `1,0,6,13,0,60`). Chiffres décimaux seulement dans les trois cas.
//
// Charge le module (sans aucune importation), démarre le modèle avec la graine, puis, pour chaque élément de <pas> dans
// l'ordre, appelle `givre_avancer` avec cet entier et lit les huit mots de 32 bits de l'empreinte. Écrit sur la sortie
// standard une ligne par élément de <pas> : les 64 caractères hexadécimaux minuscules de l'empreinte lue après ce morceau,
// rien d'autre. (L'empreinte est donc relue après chaque morceau : un module qui garderait une empreinte d'avant se voit.)
// Si le fichier ou les arguments sont faux, si une exportation manque ou si `givre_demarrer` ne rend pas 1, écrit un message
// sur la sortie d'erreur, sort avec le code 1 et n'écrit **rien** sur la sortie standard.
//
// Ce script n'a aucune dépendance : les modules intégrés de Node seulement. Il est écrit à part des tests Rust et du module :
// c'est lui qui prouve que le fichier `docs/givre.wasm` donne le même résultat que le code natif.

import { readFileSync } from "node:fs";

const EXPORTATIONS = ["givre_demarrer", "givre_avancer", "givre_empreinte_mot"];
const MAX_U32 = 4294967295;

/** Lit un entier de 32 bits non signé écrit en chiffres décimaux ; échoue sinon. */
function entier32(texte, nom) {
  if (!/^[0-9]+$/.test(texte) || Number(texte) > MAX_U32) {
    throw new Error(`${nom} doit être un entier de 0 à ${MAX_U32} : « ${texte} »`);
  }
  return Number(texte);
}

async function principal() {
  const args = process.argv.slice(2);
  if (args.length !== 4) {
    throw new Error("usage : node empreinte-wasm.mjs <fichier.wasm> <modele> <graine> <pas>");
  }
  const modele = entier32(args[1], "le modèle");
  if (modele > 3) {
    throw new Error(`le modèle doit être 0, 1, 2 ou 3 : « ${args[1]} »`);
  }
  const graine = entier32(args[2], "la graine");
  const morceaux = args[3].split(",").map((texte) => entier32(texte, "chaque nombre de pas"));

  const octets = readFileSync(args[0]);
  const { instance } = await WebAssembly.instantiate(octets, {});
  const exportations = instance.exports;
  for (const nom of EXPORTATIONS) {
    if (typeof exportations[nom] !== "function") {
      throw new Error(`le module n'exporte pas la fonction ${nom}`);
    }
  }

  if (exportations.givre_demarrer(modele, graine) !== 1) {
    throw new Error(`givre_demarrer(${modele}, ${graine}) ne rend pas 1`);
  }
  const lignes = [];
  for (const pas of morceaux) {
    exportations.givre_avancer(pas);
    let empreinte = "";
    for (let i = 0; i < 8; i++) {
      // Les fonctions rendent un i32 : `>>> 0` le relit comme un entier non signé.
      empreinte += (exportations.givre_empreinte_mot(i) >>> 0).toString(16).padStart(8, "0");
    }
    lignes.push(empreinte);
  }
  process.stdout.write(lignes.join("\n") + "\n");
}

principal().catch((erreur) => {
  process.stderr.write(`empreinte-wasm : ${erreur.message}\n`);
  process.exitCode = 1;
});

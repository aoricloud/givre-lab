// Éprouve les fonctions du métal de `docs/lecture.js` sur un FAUX module qui change d'état (sans WebAssembly) : ce qu'elles lisent, l'image qu'elles dessinent et les textes qu'elles écrivent.
//
// Usage : node metal-pur.mjs <lecture.js>
//
// Le faux module : `givre_metal_demarrer(e, c)` rend 1 si `e > 60000`, 2 si `c` n'est pas de 1 à 4, sinon démarre (le temps à 0) et rend 0 ; `givre_avancer(n)` ajoute `n` au temps et le rend ; `oublier()` efface le métal (comme un autre
// modèle démarré). Sans métal, les trois lecteurs rendent NaN. Avec un métal `(e, c)` au temps `t` :
//   givre_metal_reglage(k) : 0 → e / 1000000 ; 1 → 0,65 ; 10 → c ; 11 → 2,5 ; sinon (k de 2 à 14) → 10 k + 0,5 ; NaN pour k > 14 ;
//   givre_metal_mesure(k)  : 2 → 1 si t >= 50, sinon 0 ; 3 → 0 ; 9 → NaN si t < 3, sinon 0,125 ; 10 → NaN si t < 3, sinon 0,5 ; sinon (k de 0 à 10) → 100 k + t + 0,25 ; NaN pour k > 10 ;
//   givre_metal_case(f, i) : NaN si f > 1 ou i >= c² ; φ (f = 0) = ((37 i + t) mod 17) / 8 − 1 ; u (f = 1) = ((11 i + t) mod 13) / 20 − 0,65.
// Le script écrit, une par ligne, `clé=valeur` (les nombres par `String(x)` de JavaScript ; les octets d'une image en hexadécimal, deux chiffres par octet) :
//   avant    : `lireReglagesMetal`, `lireMesuresMetal`, `lireChampsMetal` sur le faux module neuf : `null,null,null` ;
//   refus    : les codes de `givre_metal_demarrer(60001, 2)` et `(0, 9)` puis encore les trois lectures : `1,2,null,null,null` ;
//   reglages : après `givre_metal_demarrer(50000, 3)` : les quinze champs de `lireReglagesMetal` dans l'ordre du module ;
//   mesures  : les onze champs de `lireMesuresMetal` (position, temps, auMur, diverge, releve.temps, releve.position, releve.lignes, releve.sommet, releve.rayon, vitesse, selection) ;
//   champs   : `cote`, `delta`, puis les neuf `φ`, puis les neuf `u` de `lireChampsMetal` (des `Float64Array`) : `cote;delta;phi…;u…;sortes` ;
//   image    : les octets de `imageDuMetal(champs, 5)` (5 × 5 pixels) puis `<largeur>x<hauteur>,<sorte>` ;
//   apres    : après `givre_avancer(3)` : `mesures` et `champs` de nouveau, puis les octets de `imageDuMetal(champs, 4)` ;
//   uniformes: `imageDuMetal` d'un champ d'une seule case (cote 1, delta 0,5) en 2 × 2 pixels, pour (φ, u) = (1, 0), (−1, −0,5), (−1, 0), (0, −0,25), (0,5, −0,125) : le premier pixel de chacun, `R.V.B.A` ;
//   textes   : `texteReglagesMetal(reglages)`, `texteMesuresMetal(mesures)` (après les 3 pas) ;
//   fins     : `texteFinMetal` d'un métal au mur, à la durée, divergé, séparés par `|` ;
//   au_mur   : après `givre_avancer(47)` : `lireMesuresMetal(m).auMur` ;
//   refus_texte : `texteRefusMetal(c)` pour les codes 1, 2, 3 et 9, séparés par `|` ;
//   blocs    : `blocsMetal(true)` et `blocsMetal(false)` : `commandes,image,mesures` ;
//   formats  : `formaterMetal` de (NaN, 2), (0.08128, 4), (112, 1), (−0.5, 3), (5.6789, 0) ;
//   oublie   : après `oublier()` : les trois lectures : `null,null,null`.
// Si `lecture.js` manque ou n'exporte pas une fonction : un message sur la sortie d'erreur, le code 1, rien sur la sortie standard.

import { resolve } from "node:path";
import { pathToFileURL } from "node:url";

const FONCTIONS = [
  "lireReglagesMetal",
  "lireMesuresMetal",
  "lireChampsMetal",
  "imageDuMetal",
  "formaterMetal",
  "texteReglagesMetal",
  "texteMesuresMetal",
  "texteFinMetal",
  "texteRefusMetal",
  "blocsMetal",
];

function fauxModule() {
  let metal = null;
  return {
    givre_metal_demarrer(e, c) {
      if (e > 60000) return 1;
      if (c < 1 || c > 4) return 2;
      metal = { e, c, t: 0 };
      return 0;
    },
    givre_avancer(n) {
      if (metal === null) return 0;
      metal.t += n;
      return n;
    },
    oublier() {
      metal = null;
    },
    givre_metal_reglage(k) {
      if (metal === null || k > 14) return NaN;
      if (k === 0) return metal.e / 1000000;
      if (k === 1) return 0.65;
      if (k === 10) return metal.c;
      if (k === 11) return 2.5;
      return 10 * k + 0.5;
    },
    givre_metal_mesure(k) {
      if (metal === null || k > 10) return NaN;
      const t = metal.t;
      if (k === 2) return t >= 50 ? 1 : 0;
      if (k === 3) return 0;
      if (k === 9) return t < 3 ? NaN : 0.125;
      if (k === 10) return t < 3 ? NaN : 0.5;
      return 100 * k + t + 0.25;
    },
    givre_metal_case(f, i) {
      if (metal === null || f > 1 || i >= metal.c * metal.c) return NaN;
      return f === 0 ? ((37 * i + metal.t) % 17) / 8 - 1 : ((11 * i + metal.t) % 13) / 20 - 0.65;
    },
  };
}

const octets = (image) => Array.from(image.rgba, (o) => o.toString(16).padStart(2, "0")).join("");

async function principal() {
  const args = process.argv.slice(2);
  if (args.length !== 1) {
    throw new Error("usage : node metal-pur.mjs <lecture.js>");
  }
  const lecture = await import(pathToFileURL(resolve(args[0])).href);
  for (const nom of FONCTIONS) {
    if (typeof lecture[nom] !== "function") {
      throw new Error(`lecture.js n'exporte pas la fonction ${nom}`);
    }
  }
  const m = fauxModule();
  const trois = () => [lecture.lireReglagesMetal(m), lecture.lireMesuresMetal(m), lecture.lireChampsMetal(m)].map((x) => (x === null ? "null" : "objet")).join(",");
  const mesures = () => {
    const me = lecture.lireMesuresMetal(m);
    return [me.position, me.temps, me.auMur, me.diverge, me.releve.temps, me.releve.position, me.releve.lignes, me.releve.sommet, me.releve.rayon, me.vitesse, me.selection].map(String).join(",");
  };
  const champs = () => {
    const c = lecture.lireChampsMetal(m);
    return [c.cote, c.delta, Array.from(c.phi).join(","), Array.from(c.u).join(","), `${c.phi.constructor.name}.${c.u.constructor.name}`].join(";");
  };
  const sortie = [];
  sortie.push(`avant=${trois()}`);
  sortie.push(`refus=${m.givre_metal_demarrer(60001, 2)},${m.givre_metal_demarrer(0, 9)},${trois()}`);
  m.givre_metal_demarrer(50000, 3);
  const reglages = lecture.lireReglagesMetal(m);
  const cles = ["epsilon", "delta", "d0", "diffusivite", "w0", "lambda", "tau0", "dx", "dt", "rayon", "cote", "limite", "duree", "releve", "profondeur"];
  sortie.push(`reglages=${cles.map((k) => String(reglages[k])).join(",")}`);
  sortie.push(`mesures=${mesures()}`);
  sortie.push(`champs=${champs()}`);
  const image = lecture.imageDuMetal(lecture.lireChampsMetal(m), 5);
  sortie.push(`image=${octets(image)},${image.largeur}x${image.hauteur},${image.rgba.constructor.name}`);
  m.givre_avancer(3);
  sortie.push(`apres=${mesures()}|${champs()}|${octets(lecture.imageDuMetal(lecture.lireChampsMetal(m), 4))}`);
  const uniformes = [
    [1, 0],
    [-1, -0.5],
    [-1, 0],
    [0, -0.25],
    [0.5, -0.125],
  ].map(([phi, u]) => {
    const image = lecture.imageDuMetal({ cote: 1, delta: 0.5, phi: Float64Array.of(phi), u: Float64Array.of(u) }, 2);
    return Array.from(image.rgba.slice(0, 4)).join(".");
  });
  sortie.push(`uniformes=${uniformes.join(",")}`);
  sortie.push(`textes=${lecture.texteReglagesMetal(reglages)}|${lecture.texteMesuresMetal(lecture.lireMesuresMetal(m))}`);
  const base = lecture.lireMesuresMetal(m);
  sortie.push(
    `fins=${[
      { ...base, auMur: true, diverge: false },
      { ...base, auMur: false, diverge: false },
      { ...base, auMur: false, diverge: true },
    ]
      .map((x) => lecture.texteFinMetal(x))
      .join("|")}`,
  );
  m.givre_avancer(47);
  sortie.push(`au_mur=${lecture.lireMesuresMetal(m).auMur}`);
  sortie.push(`refus_texte=${[1, 2, 3, 9].map((c) => lecture.texteRefusMetal(c)).join("|")}`);
  const blocs = (b) => [b.commandes, b.image, b.mesures].join(",");
  sortie.push(`blocs=${blocs(lecture.blocsMetal(true))}|${blocs(lecture.blocsMetal(false))}`);
  sortie.push(
    `formats=${[
      [NaN, 2],
      [0.08128, 4],
      [112, 1],
      [-0.5, 3],
      [5.6789, 0],
    ]
      .map(([x, d]) => lecture.formaterMetal(x, d))
      .join(";")}`,
  );
  m.oublier();
  sortie.push(`oublie=${trois()}`);
  process.stdout.write(`${sortie.join("\n")}\n`);
}

principal().catch((erreur) => {
  process.stderr.write(`metal-pur.mjs : ${erreur.message}\n`);
  process.exit(1);
});

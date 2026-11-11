// Fait appeler par Node les fonctions de `docs/lecture.js` qui ne lisent pas le module : celles qui disent un numéro de modèle et celles qui écrivent les textes de la page.
//
// Usage : node lecture-pure.mjs <lecture.js>
//
// Ce script ne charge aucun module WebAssembly : il tourne dès que `lecture.js` est écrit. Il charge `lecture.js` (par son adresse `file:`) et écrit sur la sortie standard, une par
// ligne, `clé=valeur`, dans cet ordre (les cas sont écrits ici, les valeurs attendues dans `tests/etape_d3_page.rs`) :
//
//   numeros      : `numeroDuModele(choix, eta)` pour ("0", 3), ("1", 0), ("2", 2), ("3", 1), ("claquage", 0), ("claquage", 1), ("claquage", 2), ("claquage", 3), séparés par des virgules ;
//   formater     : `formater(x)` pour 1.2344, 1.2346, 0, 0.0004, 2, 1.9996, séparés par des points-virgules ;
//   avancement   : `texteAvancement(courbe)` pour huit courbes synthétiques (voir `COURBES` ci-dessous), séparés par des barres verticales ;
//   reperes      : `texteRepereEta(eta, repere, dimension)` pour six cas (voir `REPERES`), chacun écrit « repère / écart » (l'écart `null` est écrit « null »), séparés par des barres verticales ;
//   blocs        : `blocsCaches(choix)` pour les choix « 0 », « 1 », « 2 », « 3 » et « claquage », chacun écrit `<choix>:<zinc>,<repereEta>,<commandeEta>` (`true` : le bloc est caché), séparés par des barres verticales.
//
// Si le fichier ou les arguments sont faux, ou si une fonction manque, écrit un message sur la sortie d'erreur, sort avec le code 1 et n'écrit **rien** sur la sortie standard.
// Aucune dépendance : les modules intégrés de Node seulement.

import { resolve } from "node:path";
import { pathToFileURL } from "node:url";

const FONCTIONS = ["numeroDuModele", "formater", "texteAvancement", "texteRepereEta", "blocsCaches"];

// Des courbes comme `lireCourbe` en rend : { code, fait, total, graines, suivant } (les points n'y servent pas).
const COURBES = [
  { code: 0, fait: 0, total: 0, graines: 0, suivant: null },
  { code: 0, fait: 0, total: 24, graines: 6, suivant: { eta: 0, graine: 1 } },
  { code: 0, fait: 14, total: 24, graines: 6, suivant: { eta: 2, graine: 3 } },
  { code: 0, fait: 23, total: 24, graines: 6, suivant: { eta: 3, graine: 6 } },
  { code: 0, fait: 24, total: 24, graines: 6, suivant: null },
  { code: 4, fait: 0, total: 24, graines: 6, suivant: { eta: 0, graine: 1 } },
  { code: 3, fait: 7, total: 24, graines: 6, suivant: { eta: 1, graine: 2 } },
  { code: 9, fait: 1, total: 8, graines: 2, suivant: { eta: 0, graine: 2 } },
];

// Des repères comme `givre_courbe_repere` les rend (NaN : aucun) avec la dimension mesurée : [eta, repere, dimension].
const REPERES = [
  [0, NaN, 1.8],
  [1, 1.5, 1.6],
  [2, 1.25, 1.125],
  [3, 1.5, 1.5],
  [1, 2, 1.4996],
  [2, 1.5, 1.5004],
];

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
  const numeros = [
    ["0", 3],
    ["1", 0],
    ["2", 2],
    ["3", 1],
    ["claquage", 0],
    ["claquage", 1],
    ["claquage", 2],
    ["claquage", 3],
  ].map(([choix, eta]) => String(lecture.numeroDuModele(choix, eta)));
  const formats = [1.2344, 1.2346, 0, 0.0004, 2, 1.9996].map((x) => lecture.formater(x));
  const avancements = COURBES.map((courbe) => lecture.texteAvancement(courbe));
  const reperes = REPERES.map(([eta, repere, dimension]) => {
    const textes = lecture.texteRepereEta(eta, repere, dimension);
    return `${textes.repere} / ${textes.ecart === null ? "null" : textes.ecart}`;
  });
  const blocs = ["0", "1", "2", "3", "claquage"].map((choix) => {
    const caches = lecture.blocsCaches(choix);
    return `${choix}:${caches.zinc},${caches.repereEta},${caches.commandeEta}`;
  });
  const lignes = [
    `numeros=${numeros.join(",")}`,
    `formater=${formats.join(";")}`,
    `avancement=${avancements.join("|")}`,
    `reperes=${reperes.join("|")}`,
    `blocs=${blocs.join("|")}`,
  ];
  process.stdout.write(lignes.join("\n") + "\n");
}

principal().catch((erreur) => {
  process.stderr.write(`lecture-pure : ${erreur.message}\n`);
  process.exitCode = 1;
});

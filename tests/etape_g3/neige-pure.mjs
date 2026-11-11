// Fait appeler par Node les fonctions de `docs/lecture.js` de la neige, sur un **faux module qui change d'état** (il se souvient du dernier démarrage et du temps joué) et sans module WebAssembly : ce qui lit le module (`lireJeuxNeige`,
// `lireReglagesNeige`, `lireMesuresNeige`) et ce qui écrit les textes de la page (`formaterReglage`, `texteDuJeuNeige`, `texteSourceNeige`, `texteDeLaFormeNeige`, `texteMesuresNeige`, `texteFinNeige`, `texteRefusNeige`, `blocsNeige`).
//
// Usage : node neige-pure.mjs <lecture.js>
//
// Ce script ne charge aucun module WebAssembly : il tourne dès que `lecture.js` est écrit. Le faux module (`faux`) a trois jeux nommés dont les champs sont des formules (le jeu `j` : figure 9, 10 ou 13 ; rang 2, 6 ou 1 ; cristaux 6, 6 ou 1 ;
// réglage balayé 0, 1 ou 7 ; `ρ β α θ κ μ γ` = `100 + j`, `200 + j`, … `700 + j` ; rayon 100 ; pas `1000 + j` ; déduit `j % 2`), et, **une fois démarré** par `faux.demarrer(jeu, rho, beta, kappa, mu)`, des réglages (les quatre passés,
// `α = 11 (j + 1)`, `θ = 22 (j + 1)`, `γ = 33 (j + 1)`, le jeu, le rayon 100, les pas `1000 + j`) et des mesures qui **changent avec le temps joué** (`faux.avancer(n)`) : `rattachees = 10 t + 1`, `rayon = t + 2`, `compacite = 500 + t`, `pointes = 7`,
// `aiguilles = t mod 5`, `hexagonalite = 1000 − t`, étiquette `t mod 5`, au mur dès `t >= 50`. Avant tout démarrage les réglages et les mesures valent `4294967295`. Il écrit sur la sortie standard, une par ligne, `clé=valeur`, dans cet ordre
// (les cas sont écrits ici, les valeurs attendues dans `tests/etape_g3_page.rs`) :
//
//   jeux       : `lireJeuxNeige(faux)` : chaque jeu `indice:figure.rang.cristaux.balaye:rho,beta,alpha,theta,kappa,mu,gamma:rayon:pas:deduit` (déduit 0 ou 1), séparés par `|` ;
//   reglages   : `lireReglagesNeige(faux)` quatre fois : avant tout démarrage (`null`), après `demarrer(1, 11, 12, 13, 14)`, après `demarrer(2, 21, 22, 23, 24)` (les réglages du second, pas ceux du premier), puis après un `arreter()` (`null`) :
//                chaque fois `null` ou `jeu:rayon:pas:rho,beta,alpha,theta,kappa,mu,gamma`, séparés par `|` ;
//   mesures    : `lireMesuresNeige(faux)` : avant tout démarrage (`null`), puis démarré au temps 0, au temps 3, au temps 53 (au mur), puis après un nouveau démarrage (le temps revient à 0) : chaque fois `null` ou
//                `rattachees,rayon,compacite,pointes,aiguilles,hexagonalite,forme,auMur` (`auMur` écrit 0 ou 1), séparés par `|` ;
//   formater   : `formaterReglage(x)` pour 0, 1, 5, 50, 250000, 400000, 1090000, 1500000, 8000000, 123456, 999999, 1000000, 74500, 100, 1000, 2000000, séparés par des points-virgules ;
//   jeux_texte : `texteDuJeuNeige(jeu)` pour quatre jeux (figures 9, 10, 12 et 13), séparés par `|` ;
//   sources    : `texteSourceNeige(jeu, modifie)` pour (lu, faux), (déduit, faux), (lu, vrai) et (déduit, vrai), séparés par `|` ;
//   formes     : `texteDeLaFormeNeige(forme)` pour 0, 1, 2, 3, 4 et 9, séparés par `|` ;
//   textes     : `texteMesuresNeige(mesures)` pour trois mesures, séparés par `|` ;
//   fins       : `texteFinNeige(mesures)` pour `auMur` vrai puis faux, séparés par `|` ;
//   refus      : `texteRefusNeige(code)` pour 1, 2, 3, 4, 5, 6 et 9, séparés par `|` ;
//   blocs      : `blocsNeige(neige)` pour vrai puis faux : `commandes,mesures` (`true` : le bloc est caché), séparés par `|`.
//
// Si le fichier ou les arguments sont faux, ou si une fonction manque, écrit un message sur la sortie d'erreur, sort avec le code 1 et n'écrit **rien** sur la sortie standard. Aucune dépendance : les modules intégrés de Node seulement.

import { resolve } from "node:path";
import { pathToFileURL } from "node:url";

const FONCTIONS = [
  "lireJeuxNeige",
  "lireReglagesNeige",
  "lireMesuresNeige",
  "formaterReglage",
  "texteDuJeuNeige",
  "texteSourceNeige",
  "texteDeLaFormeNeige",
  "texteMesuresNeige",
  "texteFinNeige",
  "texteRefusNeige",
  "blocsNeige",
];

const MAX_U32 = 4294967295;

/** Un faux module de la vitrine pour la neige, qui change d'état. */
function faireLeFauxModule() {
  const etat = { demarre: null, temps: 0 };
  return {
    demarrer(jeu, rho, beta, kappa, mu) {
      etat.demarre = { jeu, rho, beta, kappa, mu };
      etat.temps = 0;
    },
    avancer(n) {
      etat.temps += n;
    },
    arreter() {
      etat.demarre = null;
    },
    givre_neige_jeux() {
      return 3;
    },
    givre_neige_jeu(j, c) {
      if (j >= 3 || c >= 14) {
        return -1;
      }
      const valeurs = [[9, 2, 6, 0], [10, 6, 6, 1], [13, 1, 1, 7]][j];
      if (c < 4) {
        return valeurs[c];
      }
      if (c < 11) {
        return 100 * (c - 3) + j;
      }
      return [100, 1000 + j, j % 2][c - 11];
    },
    givre_neige_reglage(c) {
      const d = etat.demarre;
      if (d === null || c >= 10) {
        return -1;
      }
      return [d.rho, d.beta, 11 * (d.jeu + 1), 22 * (d.jeu + 1), d.kappa, d.mu, 33 * (d.jeu + 1), d.jeu, 100, 1000 + d.jeu][c];
    },
    givre_neige_mesure(c) {
      const t = etat.temps;
      if (etat.demarre === null || c >= 8) {
        return -1;
      }
      return [10 * t + 1, t + 2, 500 + t, 7, t % 5, 1000 - t, t % 5, t >= 50 ? 1 : 0][c];
    },
  };
}

async function principal() {
  const args = process.argv.slice(2);
  if (args.length !== 1) {
    throw new Error("usage : node neige-pure.mjs <lecture.js>");
  }
  const lecture = await import(pathToFileURL(resolve(args[0])).href);
  for (const nom of FONCTIONS) {
    if (typeof lecture[nom] !== "function") {
      throw new Error(`lecture.js n'exporte pas la fonction ${nom}`);
    }
  }
  const faux = faireLeFauxModule();

  const jeux = lecture
    .lireJeuxNeige(faux)
    .map((j) => {
      const r = j.reglages;
      return `${j.indice}:${j.figure}.${j.rang}.${j.cristaux}.${j.balaye}:${[r.rho, r.beta, r.alpha, r.theta, r.kappa, r.mu, r.gamma].join(",")}:${j.rayon}:${j.pas}:${j.deduit ? 1 : 0}`;
    })
    .join("|");

  const reglages = (m) => {
    const lus = lecture.lireReglagesNeige(m);
    if (lus === null) {
      return "null";
    }
    const r = lus.reglages;
    return `${lus.jeu}:${lus.rayon}:${lus.pas}:${[r.rho, r.beta, r.alpha, r.theta, r.kappa, r.mu, r.gamma].join(",")}`;
  };
  const sortieReglages = [reglages(faux)];
  faux.demarrer(1, 11, 12, 13, 14);
  sortieReglages.push(reglages(faux));
  faux.demarrer(2, 21, 22, 23, 24);
  sortieReglages.push(reglages(faux));
  faux.arreter();
  sortieReglages.push(reglages(faux));

  const mesures = (m) => {
    const lues = lecture.lireMesuresNeige(m);
    if (lues === null) {
      return "null";
    }
    return [lues.rattachees, lues.rayon, lues.compacite, lues.pointes, lues.aiguilles, lues.hexagonalite, lues.forme, lues.auMur ? 1 : 0].join(",");
  };
  const sortieMesures = [mesures(faux)];
  faux.demarrer(0, 1, 2, 3, 4);
  sortieMesures.push(mesures(faux));
  faux.avancer(3);
  sortieMesures.push(mesures(faux));
  faux.avancer(50);
  sortieMesures.push(mesures(faux));
  faux.demarrer(0, 1, 2, 3, 4);
  sortieMesures.push(mesures(faux));

  const jeuxTexte = [
    { figure: 9, rang: 2, cristaux: 6, balaye: 0 },
    { figure: 10, rang: 6, cristaux: 6, balaye: 1 },
    { figure: 12, rang: 3, cristaux: 6, balaye: 5 },
    { figure: 13, rang: 1, cristaux: 1, balaye: 7 },
  ].map((j) => lecture.texteDuJeuNeige(j));
  const sources = [
    [{ deduit: false }, false],
    [{ deduit: true }, false],
    [{ deduit: false }, true],
    [{ deduit: true }, true],
  ].map(([j, modifie]) => lecture.texteSourceNeige(j, modifie));
  const formes = [0, 1, 2, 3, 4, 9].map((f) => lecture.texteDeLaFormeNeige(f));
  const textes = [
    { rattachees: 1687, rayon: 26, compacite: 800, pointes: 18, aiguilles: 0, hexagonalite: 500 },
    { rattachees: 1, rayon: 0, compacite: 1000, pointes: 1, aiguilles: 1, hexagonalite: 1000 },
    { rattachees: 4639, rayon: 59, compacite: 5, pointes: 120, aiguilles: 12, hexagonalite: 45 },
  ].map((mesure) => lecture.texteMesuresNeige(mesure));
  const fins = [true, false].map((auMur) => lecture.texteFinNeige({ auMur }));
  const refus = [1, 2, 3, 4, 5, 6, 9].map((code) => lecture.texteRefusNeige(code));
  const blocs = [true, false].map((neige) => {
    const caches = lecture.blocsNeige(neige);
    return `${caches.commandes},${caches.mesures}`;
  });

  const lignes = [
    `jeux=${jeux}`,
    `reglages=${sortieReglages.join("|")}`,
    `mesures=${sortieMesures.join("|")}`,
    `formater=${[0, 1, 5, 50, 250000, 400000, 1090000, 1500000, 8000000, 123456, 999999, 1000000, 74500, 100, 1000, 2000000].map((x) => lecture.formaterReglage(x)).join(";")}`,
    `jeux_texte=${jeuxTexte.join("|")}`,
    `sources=${sources.join("|")}`,
    `formes=${formes.join("|")}`,
    `textes=${textes.join("|")}`,
    `fins=${fins.join("|")}`,
    `refus=${refus.join("|")}`,
    `blocs=${blocs.join("|")}`,
  ];
  process.stdout.write(lignes.join("\n") + "\n");
}

principal().catch((erreur) => {
  process.stderr.write(`neige-pure : ${erreur.message}\n`);
  process.exitCode = 1;
});

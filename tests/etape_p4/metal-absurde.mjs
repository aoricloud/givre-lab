// Fait des centaines d'appels tirés, aux arguments absurdes, des fonctions du métal et de celles d'avant, sur le module WebAssembly de la vitrine : une panique du module serait une instruction `unreachable` (une exception
// `RuntimeError` que ce script rattrape et compte).
//
// Usage : node metal-absurde.mjs <fichier.wasm>
//
// Le tirage : un xorshift de 32 bits (`x ^= x << 13 ; x ^= x >>> 17 ; x ^= x << 5`, non signé, graine 2463534242), `tirer(n) = suivant() % n`. Un argument est une des quatorze valeurs de `VALEURS` (`tirer(17) < 14`) ou un `u32`
// tiré (`suivant()`). Il y a `APPELS` = 400 tours ; à chaque tour, `tirer(13)` choisit l'opération (voir `principal`, chaque cas dans l'ordre où il appelle le module). Chaque nombre rendu est écrit (un `u32` en décimal non signé, un
// `f64` par ses bits en hexadécimal, `NaN` pour un NaN) et mis bout à bout avec des virgules. Le script écrit trois lignes : `appels=<le nombre d'appels au module>`, `exceptions=<le nombre d'exceptions rattrapées>`,
// `hache=<le SHA-256 de la suite des nombres rendus>`. `etape_p4_commun::attendu_de_l_absurde` rejoue la même suite sur le code natif.
//
// Si le fichier est faux ou s'il manque une fonction : un message sur la sortie d'erreur, le code 1, rien sur la sortie standard. Aucune dépendance : les modules intégrés de Node seulement.

import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";

const APPELS = 400;
const VALEURS = [0, 1, 2, 5, 127, 128, 129, 256, 257, 500, 50000, 60000, 60001, 4294967295];

let etat = 2463534242;
function suivant() {
  etat ^= etat << 13;
  etat >>>= 0;
  etat ^= etat >>> 17;
  etat ^= etat << 5;
  etat >>>= 0;
  return etat;
}
const tirer = (n) => suivant() % n;
function argument() {
  const r = tirer(17);
  return r < VALEURS.length ? VALEURS[r] : suivant();
}

function hexa(x) {
  if (Number.isNaN(x)) {
    return "NaN";
  }
  const vue = new DataView(new ArrayBuffer(8));
  vue.setFloat64(0, x);
  return vue.getBigUint64(0).toString(16).padStart(16, "0");
}

async function principal() {
  const args = process.argv.slice(2);
  if (args.length !== 1) {
    throw new Error("usage : node metal-absurde.mjs <fichier.wasm>");
  }
  const { instance } = await WebAssembly.instantiate(readFileSync(args[0]), {});
  const m = instance.exports;
  for (const nom of ["givre_metal_demarrer", "givre_metal_reglage", "givre_metal_mesure", "givre_metal_case", "givre_neige_masses_preparer"]) {
    if (typeof m[nom] !== "function") {
      throw new Error(`le module n'exporte pas la fonction ${nom}`);
    }
  }
  const rendus = [];
  let appels = 0;
  let exceptions = 0;
  /** Un appel qui rend un u32. */
  const u = (f, ...a) => {
    appels += 1;
    try {
      rendus.push(String(f(...a) >>> 0));
    } catch {
      exceptions += 1;
      rendus.push("X");
    }
  };
  /** Un appel qui rend un f64. */
  const d = (f, ...a) => {
    appels += 1;
    try {
      rendus.push(hexa(f(...a)));
    } catch {
      exceptions += 1;
      rendus.push("X");
    }
  };
  for (let tour = 0; tour < APPELS; tour++) {
    switch (tirer(13)) {
      case 0: {
        const e = argument();
        const c = [128, 129, argument()][tirer(3)];
        u(m.givre_metal_demarrer, e, c);
        break;
      }
      case 1:
        u(m.givre_avancer, tirer(6));
        break;
      case 2:
        d(m.givre_metal_reglage, argument());
        break;
      case 3:
        d(m.givre_metal_mesure, argument());
        break;
      case 4: {
        const champ = argument();
        const i = argument();
        d(m.givre_metal_case, champ, i);
        break;
      }
      case 5: {
        const modele = [0, 1, 2, 3, 8, 9, 4294967295][tirer(7)];
        u(m.givre_demarrer, modele, argument());
        break;
      }
      case 6:
        if (tirer(2) === 0) {
          u(m.givre_neige_demarrer, 0, 400000, 1300000, 3000, 70000);
        } else {
          const a = [argument(), argument(), argument(), argument(), argument()];
          u(m.givre_neige_demarrer, ...a);
        }
        break;
      case 7:
        u(m.givre_temps);
        u(m.givre_duree);
        u(m.givre_sites);
        u(m.givre_rayon);
        break;
      case 8: {
        d(m.givre_compacite);
        const x = argument();
        const y = argument();
        u(m.givre_pixel, x, y);
        u(m.givre_largeur);
        u(m.givre_hauteur);
        break;
      }
      case 9:
        u(m.givre_empreinte_mot, tirer(10));
        break;
      case 10: {
        const cote = [2, 64, 63, argument()][tirer(4)];
        const sur = [1, 2, 0, argument()][tirer(4)];
        u(m.givre_nette_preparer, cote, sur);
        break;
      }
      case 11:
        u(m.givre_mesurer);
        u(m.givre_correlation_rayon, argument());
        d(m.givre_dimension);
        break;
      default:
        u(m.givre_neige_masses_preparer);
        u(m.givre_neige_masse, argument());
        break;
    }
  }
  const hache = createHash("sha256").update(rendus.join(",")).digest("hex");
  process.stdout.write(`appels=${appels}\nexceptions=${exceptions}\nhache=${hache}\n`);
}

principal().catch((erreur) => {
  process.stderr.write(`metal-absurde.mjs : ${erreur.message}\n`);
  process.exit(1);
});

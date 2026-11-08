// La vitrine de Givre Lab (étape 5) : le module givre.wasm piloté par des nombres, dessiné sur un canvas.

import { lireEmpreinte, lireImage, lireGraine } from "./lecture.js";

// Huit appels de givre_empreinte_mot recalculent huit fois tout l'état : on lit l'empreinte au plus toutes les 500 ms.
const INTERVALLE_EMPREINTE_MS = 500;

const canvas = document.getElementById("vitrine");
const choixModele = document.getElementById("modele");
const champGraine = document.getElementById("graine");
const boutonRelancer = document.getElementById("relancer");
const curseurVitesse = document.getElementById("vitesse");
const valeurVitesse = document.getElementById("valeur-vitesse");
const affichageTemps = document.getElementById("temps");
const affichageEmpreinte = document.getElementById("empreinte");
const affichageMesures = document.getElementById("mesures");
const affichageEtat = document.getElementById("etat");
const contexte = canvas.getContext("2d");

let m = null;
let enCours = false;
let derniereEmpreinte = -Infinity;

/** Dessine l'image de l'état courant, aux dimensions qu'elle a maintenant. */
function dessiner() {
  const { largeur, hauteur, rgba } = lireImage(m);
  if (largeur === 0 || hauteur === 0) {
    return;
  }
  if (canvas.width !== largeur || canvas.height !== hauteur) {
    canvas.width = largeur;
    canvas.height = hauteur;
  }
  contexte.putImageData(new ImageData(rgba, largeur, hauteur), 0, 0);
}

function afficherTemps() {
  affichageTemps.textContent = `${m.givre_temps() >>> 0} / ${m.givre_duree() >>> 0}`;
}

function afficherEmpreinte() {
  affichageEmpreinte.textContent = lireEmpreinte(m);
}

function afficherMesures() {
  const sites = m.givre_sites() >>> 0;
  const rayon = m.givre_rayon() >>> 0;
  const compacite = m.givre_compacite();
  const complement = Number.isNaN(compacite) ? "" : ` · compacité ${compacite.toFixed(4)}`;
  affichageMesures.textContent = `${sites} sites · rayon ${rayon}${complement}`;
}

function afficherVitesse() {
  valeurVitesse.textContent = curseurVitesse.value;
}

/** Une image de l'animation : joue les unités du curseur, dessine, puis se rappelle. */
function image(maintenant) {
  if (!enCours) {
    return;
  }
  const joue = m.givre_avancer(Number(curseurVitesse.value)) >>> 0;
  dessiner();
  afficherTemps();
  if (joue === 0 || maintenant - derniereEmpreinte >= INTERVALLE_EMPREINTE_MS) {
    afficherEmpreinte();
    afficherMesures();
    derniereEmpreinte = maintenant;
  }
  if (joue === 0) {
    enCours = false;
    affichageEtat.textContent = "Scénario terminé : recommencez ou choisissez un autre modèle.";
    return;
  }
  requestAnimationFrame(image);
}

/** (Re)démarre le modèle choisi avec la graine lue dans le champ. */
function relancer() {
  if (m === null) {
    return;
  }
  const modele = Number(choixModele.value);
  const graine = lireGraine(champGraine.value);
  if (graine === null) {
    affichageEtat.textContent = "Graine invalide : un entier de 0 à 4294967295.";
    return;
  }
  if ((m.givre_demarrer(modele, graine) >>> 0) !== 1) {
    affichageEtat.textContent = "Modèle inconnu.";
    return;
  }
  dessiner();
  afficherTemps();
  afficherEmpreinte();
  afficherMesures();
  derniereEmpreinte = performance.now();
  affichageEtat.textContent =
    modele === 0 ? `DLA, graine ${graine}.` : "Flocon de Reiter (la graine ne sert qu'à la DLA).";
  if (!enCours) {
    enCours = true;
    requestAnimationFrame(image);
  }
}

async function charger() {
  try {
    const reponse = await fetch("givre.wasm");
    if (!reponse.ok) {
      throw new Error(`givre.wasm : ${reponse.status}`);
    }
    const { instance } = await WebAssembly.instantiate(await reponse.arrayBuffer(), {});
    m = instance.exports;
  } catch (erreur) {
    affichageEtat.textContent = `Le module ne se charge pas : ${erreur.message}`;
    return;
  }
  boutonRelancer.addEventListener("click", relancer);
  choixModele.addEventListener("change", relancer);
  champGraine.addEventListener("keydown", (evenement) => {
    if (evenement.key === "Enter") {
      relancer();
    }
  });
  curseurVitesse.addEventListener("input", afficherVitesse);
  afficherVitesse();
  relancer();
}

charger();

// La vitrine de Givre Lab (étape 5) : le module givre.wasm piloté par des nombres, dessiné sur un canvas.

import { lireCorrelation, lireEmpreinte, lireImage, lireGraine } from "./lecture.js";

// Huit appels de givre_empreinte_mot recalculent huit fois tout l'état : on lit l'empreinte au plus toutes les 500 ms.
const INTERVALLE_EMPREINTE_MS = 500;

// Le repère publié du zinc (Matsushita et coll. 1984) : une phrase, une constante, jamais une mesure du banc.
const ZINC_DIMENSION = 1.66;
const ZINC_DEMI_LARGEUR = 0.03;
const ZINC_PLAGE = 0.25;

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
const boutonMesurer = document.getElementById("mesurer");
const canvasCourbe = document.getElementById("courbe");
const affichageResultat = document.getElementById("resultat");
const marqueD = document.getElementById("marque-d");
const affichageCompatible = document.getElementById("compatible");
const affichageHorsBarre = document.getElementById("hors-barre");
const contexte = canvas.getContext("2d");
const contexteCourbe = canvasCourbe.getContext("2d");

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

/** Dessine C(r) en échelle log-log et la droite ajustée, de pente D - 2 (la même en log10 qu'en ln). */
function dessinerCourbe({ rayons, valeurs, dimension }) {
  const largeur = canvasCourbe.width;
  const hauteur = canvasCourbe.height;
  const ctx = contexteCourbe;
  ctx.setTransform(1, 0, 0, 1, 0, 0);
  ctx.clearRect(0, 0, largeur, hauteur);
  ctx.fillStyle = "#0b1118";
  ctx.fillRect(0, 0, largeur, hauteur);
  const marge = { gauche: 58, droite: 16, haut: 18, bas: 38 };
  if (rayons.length < 2) {
    ctx.fillStyle = "#8da2b8";
    ctx.font = "15px system-ui, sans-serif";
    ctx.textAlign = "center";
    ctx.textBaseline = "middle";
    ctx.fillText("Aucune courbe : mesurez la DLA.", largeur / 2, hauteur / 2);
    return;
  }
  const xs = rayons.map((rayon) => Math.log10(rayon));
  const ys = valeurs.map((valeur) => Math.log10(valeur));
  let xMin = Math.min(...xs);
  let xMax = Math.max(...xs);
  let yMin = Math.min(...ys);
  let yMax = Math.max(...ys);
  if (xMin === xMax) {
    xMin -= 0.5;
    xMax += 0.5;
  }
  if (yMin === yMax) {
    yMin -= 0.5;
    yMax += 0.5;
  }
  const reperesX = (xMax - xMin) * 0.06;
  const reperesY = (yMax - yMin) * 0.08;
  xMin -= reperesX;
  xMax += reperesX;
  yMin -= reperesY;
  yMax += reperesY;
  const px = (x) => marge.gauche + ((x - xMin) / (xMax - xMin)) * (largeur - marge.gauche - marge.droite);
  const py = (y) => hauteur - marge.bas - ((y - yMin) / (yMax - yMin)) * (hauteur - marge.haut - marge.bas);

  ctx.font = "12px ui-monospace, monospace";
  ctx.lineWidth = 1;
  for (let puissance = Math.ceil(xMin); puissance <= Math.floor(xMax); puissance++) {
    const x = px(puissance);
    ctx.strokeStyle = "#1e2a38";
    ctx.beginPath();
    ctx.moveTo(x, marge.haut);
    ctx.lineTo(x, hauteur - marge.bas);
    ctx.stroke();
    ctx.fillStyle = "#8da2b8";
    ctx.textAlign = "center";
    ctx.textBaseline = "top";
    ctx.fillText(`10^${puissance}`, x, hauteur - marge.bas + 6);
  }
  for (let puissance = Math.ceil(yMin); puissance <= Math.floor(yMax); puissance++) {
    const y = py(puissance);
    ctx.strokeStyle = "#1e2a38";
    ctx.beginPath();
    ctx.moveTo(marge.gauche, y);
    ctx.lineTo(largeur - marge.droite, y);
    ctx.stroke();
    ctx.fillStyle = "#8da2b8";
    ctx.textAlign = "right";
    ctx.textBaseline = "middle";
    ctx.fillText(`10^${puissance}`, marge.gauche - 6, y);
  }

  if (Number.isFinite(dimension)) {
    const pente = dimension - 2;
    let sommeX = 0;
    let sommeY = 0;
    for (let i = 0; i < xs.length; i++) {
      sommeX += xs[i];
      sommeY += ys[i];
    }
    const ordonnee = sommeY / ys.length - pente * (sommeX / xs.length);
    ctx.strokeStyle = "#6cc4ff";
    ctx.lineWidth = 2;
    ctx.beginPath();
    ctx.moveTo(px(xMin), py(pente * xMin + ordonnee));
    ctx.lineTo(px(xMax), py(pente * xMax + ordonnee));
    ctx.stroke();
  }

  ctx.fillStyle = "#e6edf3";
  for (let i = 0; i < xs.length; i++) {
    ctx.beginPath();
    ctx.arc(px(xs[i]), py(ys[i]), 3, 0, 2 * Math.PI);
    ctx.fill();
  }

  ctx.strokeStyle = "#4a5b6c";
  ctx.lineWidth = 1;
  ctx.beginPath();
  ctx.moveTo(marge.gauche, marge.haut);
  ctx.lineTo(marge.gauche, hauteur - marge.bas);
  ctx.lineTo(largeur - marge.droite, hauteur - marge.bas);
  ctx.stroke();
  ctx.fillStyle = "#8da2b8";
  ctx.textAlign = "left";
  ctx.textBaseline = "top";
  ctx.fillText("C(r)", 8, marge.haut);
  ctx.textAlign = "right";
  ctx.fillText("r", largeur - marge.droite, hauteur - 16);
}

/** Efface l'affichage de la mesure : les deux phrases du repère, la marque et la courbe. */
function effacerMesure() {
  affichageCompatible.hidden = true;
  affichageHorsBarre.hidden = true;
  marqueD.hidden = true;
  affichageResultat.textContent = "Aucune mesure pour l'instant.";
  dessinerCourbe({ rayons: [], valeurs: [], dimension: NaN });
}

/** Mesure la corrélation de l'amas du moment, puis écrit D, la courbe et le repère du zinc. */
function mesurer() {
  if (m === null) {
    return;
  }
  const points = m.givre_mesurer() >>> 0;
  const sites = m.givre_sites() >>> 0;
  const courbe = lireCorrelation(m);
  dessinerCourbe(courbe);
  if (points === 0 || courbe.rayons.length === 0) {
    affichageResultat.textContent =
      "Pas de mesure : le modèle DLA seulement, et un amas d'au moins quelques dizaines de sites.";
    affichageCompatible.hidden = true;
    affichageHorsBarre.hidden = true;
    marqueD.hidden = true;
    return;
  }
  const dimension = courbe.dimension;
  affichageResultat.textContent = `mesuré par le banc : D = ${dimension.toFixed(3).replace(".", ",")} à ${sites} sites`;
  const dansLaBarre = Math.abs(dimension - ZINC_DIMENSION) <= ZINC_DEMI_LARGEUR;
  affichageCompatible.hidden = !dansLaBarre;
  affichageHorsBarre.hidden = dansLaBarre;
  const position = ((dimension - (ZINC_DIMENSION - ZINC_PLAGE)) / (2 * ZINC_PLAGE)) * 100;
  marqueD.style.left = `${Math.max(0, Math.min(100, position))}%`;
  marqueD.hidden = false;
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
  effacerMesure();
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
  boutonMesurer.addEventListener("click", mesurer);
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

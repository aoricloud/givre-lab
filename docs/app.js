// La vitrine de Givre Lab (étape 5) : le module givre.wasm piloté par des nombres, dessiné sur un canvas. Z1 y ajoute la mesure de D
// par la corrélation ; D3 le claquage diélectrique (le curseur η) et la courbe D(η) calculée un amas à la fois ; V7 l'image nette des
// flocons de Reiter (chaque case un hexagone lisse, calculé par le module) sur un second canvas, sans `image-rendering: pixelated`.

import {
  formater,
  lireCorrelation,
  lireCourbe,
  lireEmpreinte,
  lireGraine,
  lireImage,
  lireImageNette,
  blocsCaches,
  imagesCachees,
  numeroDuModele,
  texteAvancement,
  texteRepereEta,
} from "./lecture.js";

// Huit appels de givre_empreinte_mot recalculent huit fois tout l'état : on lit l'empreinte au plus toutes les 500 ms.
const INTERVALLE_EMPREINTE_MS = 500;

// Le repère publié du zinc (Matsushita et coll. 1984) : une phrase, une constante, jamais une mesure du banc.
const ZINC_DIMENSION = 1.66;
const ZINC_DEMI_LARGEUR = 0.03;
const ZINC_PLAGE = 0.25;

// Le claquage : une unité de temps y coûte la relaxation du champ ; la page n'en joue qu'une par appel et rend la main quand BUDGET_IMAGE_MS sont écoulées.
const BUDGET_IMAGE_MS = 30;

// La courbe D(η) : un amas par appel de givre_courbe_pas, avec une pause entre deux appels pour que la page se redessine. La taille et le
// nombre de graines sont ceux que le module accepte (jusqu'à 100 sites et de 2 à 8 graines).
const COURBE_TAILLE = 100;
const COURBE_GRAINES = 6;
const PAUSE_COURBE_MS = 20;

// L'image nette (modèles sur la grille hexagonale) : le module la calcule à la demande, `cote` pixels de côté, `sur × sur` échantillons par pixel (bornes du module :
// cote pair jusqu'à 2048, sur jusqu'à 4, cote × sur jusqu'à 4096). Pendant l'animation une image légère (768² échantillons : 26 ms pour le module sous Node, 28 ms avec la lecture) ; quand le
// scénario s'arrête, une image plus fine (3072² échantillons : 0,4 s sous Node) : le navigateur met ensuite le canvas à la taille de l'écran en le lissant, jamais en gros carrés.
const NETTE_COTE_ANIMATION = 384;
const NETTE_SUR_ANIMATION = 2;
const NETTE_COTE_FIN = 1024;
const NETTE_SUR_FIN = 3;

const canvas = document.getElementById("vitrine");
const canvasNette = document.getElementById("vitrine-nette");
const legendeNette = document.getElementById("legende-nette");
const choixModele = document.getElementById("modele");
const commandeEta = document.getElementById("commande-eta");
const curseurEta = document.getElementById("eta");
const valeurEta = document.getElementById("valeur-eta");
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
const blocZinc = document.getElementById("bloc-zinc");
const marqueD = document.getElementById("marque-d");
const affichageCompatible = document.getElementById("compatible");
const affichageHorsBarre = document.getElementById("hors-barre");
const blocRepereEta = document.getElementById("bloc-repere-eta");
const affichageRepereEta = document.getElementById("repere-eta");
const affichageEcartEta = document.getElementById("ecart-eta");
const affichagePasUnAccord = document.getElementById("pas-un-accord");
const boutonCalculerCourbe = document.getElementById("calculer-courbe");
const affichageAvancement = document.getElementById("avancement-courbe");
const canvasCourbeEta = document.getElementById("courbe-eta");
const contexte = canvas.getContext("2d");
const contexteNette = canvasNette.getContext("2d");
const contexteCourbe = canvasCourbe.getContext("2d");
const contexteCourbeEta = canvasCourbeEta.getContext("2d");

let m = null;
let enCours = false;
let derniereEmpreinte = -Infinity;
let minuterieCourbe = null;
// Le modèle en cours est celui du dernier démarrage réussi, pas celui des commandes : une graine invalide laisse l'ancien modèle tourner.
let claquageEnCours = false;
let etaDuModele = 0;

/** Vrai si le modèle choisi est le claquage (le curseur η donne alors le modèle 4 + η). */
function claquageChoisi() {
  return choixModele.value === "claquage";
}

/** L'exposant η du curseur, un entier de 0 à 3. */
function etaChoisi() {
  return Number(curseurEta.value);
}

/** Pose une image lue dans le module sur un canvas, aux dimensions qu'elle a maintenant. */
function poser(cible, contexteCible, { largeur, hauteur, rgba }) {
  if (largeur === 0 || hauteur === 0) {
    return;
  }
  if (cible.width !== largeur || cible.height !== hauteur) {
    cible.width = largeur;
    cible.height = hauteur;
  }
  contexteCible.putImageData(new ImageData(rgba, largeur, hauteur), 0, 0);
}

/**
 * Dessine l'image de l'état courant : l'image nette (des hexagones lisses, sur le canvas sans `pixelated`) pour les modèles sur la grille hexagonale, l'image d'un pixel par case
 * (sur le canvas qui agrandit sans lisser) pour les autres : la DLA et le claquage, dont c'est la vraie géométrie. `fin` vaut vrai quand l'animation s'arrête : l'image nette est alors plus fine.
 */
function dessiner(fin) {
  const nette = lireImageNette(m, fin ? NETTE_COTE_FIN : NETTE_COTE_ANIMATION, fin ? NETTE_SUR_FIN : NETTE_SUR_ANIMATION);
  const caches = imagesCachees(nette.code);
  canvas.hidden = caches.carree;
  canvasNette.hidden = caches.nette;
  legendeNette.hidden = caches.nette;
  if (nette.code === 0) {
    poser(canvasNette, contexteNette, nette);
  } else {
    poser(canvas, contexte, lireImage(m));
  }
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

function afficherEta() {
  valeurEta.textContent = curseurEta.value;
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
    ctx.fillText("Aucune courbe : mesurez la DLA ou le claquage.", largeur / 2, hauteur / 2);
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

/** Efface l'affichage de la mesure : les phrases du repère (zinc et η), la marque et la courbe. */
function effacerMesure() {
  affichageCompatible.hidden = true;
  affichageHorsBarre.hidden = true;
  marqueD.hidden = true;
  affichageRepereEta.textContent = "";
  affichageEcartEta.textContent = "";
  affichageEcartEta.hidden = true;
  affichagePasUnAccord.hidden = true;
  affichageResultat.textContent = "Aucune mesure pour l'instant.";
  dessinerCourbe({ rayons: [], valeurs: [], dimension: NaN });
}

/** Montre ou cache les blocs du repère selon le modèle : le zinc pour la DLA et les flocons, le repère de η pour le claquage. */
function montrerLesReperes() {
  const caches = blocsCaches(choixModele.value);
  blocZinc.hidden = caches.zinc;
  blocRepereEta.hidden = caches.repereEta;
  commandeEta.hidden = caches.commandeEta;
}

/** Écrit le repère publié de l'η du curseur, lu dans le module, l'écart du banc et la phrase « ce n'est pas un accord ». */
function ecrireLeRepereEta(dimension) {
  const eta = etaDuModele;
  const textes = texteRepereEta(eta, m.givre_courbe_repere(eta), dimension);
  affichageRepereEta.textContent = textes.repere;
  affichageEcartEta.textContent = textes.ecart ?? "";
  affichageEcartEta.hidden = textes.ecart === null;
  affichagePasUnAccord.hidden = textes.ecart === null;
}

/** Mesure la corrélation de l'amas du moment, puis écrit D, la courbe et le repère (celui du zinc pour la DLA, celui de η pour le claquage). */
function mesurer() {
  if (m === null) {
    return;
  }
  const points = m.givre_mesurer() >>> 0;
  const sites = m.givre_sites() >>> 0;
  const courbe = lireCorrelation(m);
  dessinerCourbe(courbe);
  if (points === 0 || courbe.rayons.length === 0) {
    affichageResultat.textContent = "Pas de mesure : la DLA ou le claquage seulement, et un amas assez grand.";
    affichageCompatible.hidden = true;
    affichageHorsBarre.hidden = true;
    marqueD.hidden = true;
    affichageRepereEta.textContent = "";
    affichageEcartEta.hidden = true;
    affichagePasUnAccord.hidden = true;
    return;
  }
  const dimension = courbe.dimension;
  affichageResultat.textContent = `mesuré par le banc : D = ${formater(dimension)} à ${sites} sites`;
  if (claquageEnCours) {
    ecrireLeRepereEta(dimension);
    return;
  }
  const dansLaBarre = Math.abs(dimension - ZINC_DIMENSION) <= ZINC_DEMI_LARGEUR;
  affichageCompatible.hidden = !dansLaBarre;
  affichageHorsBarre.hidden = dansLaBarre;
  const position = ((dimension - (ZINC_DIMENSION - ZINC_PLAGE)) / (2 * ZINC_PLAGE)) * 100;
  marqueD.style.left = `${Math.max(0, Math.min(100, position))}%`;
  marqueD.hidden = false;
}

/**
 * Joue les unités d'une image et rend le nombre jouées (0 : le scénario est fini). Le claquage ne joue qu'une unité par appel de
 * givre_avancer et s'arrête dès que BUDGET_IMAGE_MS sont écoulées : une unité y coûte la relaxation du champ.
 */
function avancer() {
  const vitesse = Number(curseurVitesse.value);
  if (!claquageEnCours) {
    return m.givre_avancer(vitesse) >>> 0;
  }
  const debut = performance.now();
  let joue = 0;
  while (joue < vitesse) {
    const unite = m.givre_avancer(1) >>> 0;
    if (unite === 0) {
      break;
    }
    joue += unite;
    if (performance.now() - debut >= BUDGET_IMAGE_MS) {
      break;
    }
  }
  return joue;
}

/** Une image de l'animation : joue les unités du curseur, dessine, puis se rappelle. */
function image(maintenant) {
  if (!enCours) {
    return;
  }
  const joue = avancer();
  dessiner(joue === 0);
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

/** Le texte d'état d'un démarrage : le modèle, l'η et la graine quand ils comptent. */
function texteDEtat(modele, graine) {
  if (claquageEnCours) {
    return `Claquage diélectrique, η = ${etaDuModele}, graine ${graine}.`;
  }
  return modele === 0 ? `DLA, graine ${graine}.` : "Flocon de Reiter (la graine ne sert qu'à la DLA).";
}

/** (Re)démarre le modèle choisi avec la graine lue dans le champ (et l'η du curseur pour le claquage). */
function relancer() {
  if (m === null) {
    return;
  }
  const modele = numeroDuModele(choixModele.value, etaChoisi());
  const graine = lireGraine(champGraine.value);
  if (graine === null) {
    affichageEtat.textContent = "Graine invalide : un entier de 0 à 4294967295.";
    return;
  }
  if ((m.givre_demarrer(modele, graine) >>> 0) !== 1) {
    affichageEtat.textContent = "Modèle inconnu.";
    return;
  }
  claquageEnCours = claquageChoisi();
  etaDuModele = etaChoisi();
  montrerLesReperes();
  dessiner(false);
  afficherTemps();
  afficherEmpreinte();
  afficherMesures();
  effacerMesure();
  derniereEmpreinte = performance.now();
  affichageEtat.textContent = texteDEtat(modele, graine);
  if (!enCours) {
    enCours = true;
    requestAnimationFrame(image);
  }
}

/** Dessine les quatre points de la courbe D(η) : la moyenne du banc et sa barre d'erreur, le repère publié à côté, une grille et des axes. */
function dessinerCourbeEta(courbe) {
  const largeur = canvasCourbeEta.width;
  const hauteur = canvasCourbeEta.height;
  const ctx = contexteCourbeEta;
  const marge = { gauche: 58, droite: 24, haut: 20, bas: 44 };
  const dMin = 0.8;
  const dMax = 2.2;
  const px = (eta) => marge.gauche + ((eta + 0.5) / 4) * (largeur - marge.gauche - marge.droite);
  const py = (d) => marge.haut + ((dMax - d) / (dMax - dMin)) * (hauteur - marge.haut - marge.bas);
  ctx.setTransform(1, 0, 0, 1, 0, 0);
  ctx.clearRect(0, 0, largeur, hauteur);
  ctx.fillStyle = "#0b1118";
  ctx.fillRect(0, 0, largeur, hauteur);

  ctx.font = "12px ui-monospace, monospace";
  ctx.lineWidth = 1;
  for (let d = 1; d <= 2.0001; d += 0.25) {
    ctx.strokeStyle = "#1e2a38";
    ctx.beginPath();
    ctx.moveTo(marge.gauche, py(d));
    ctx.lineTo(largeur - marge.droite, py(d));
    ctx.stroke();
    ctx.fillStyle = "#8da2b8";
    ctx.textAlign = "right";
    ctx.textBaseline = "middle";
    ctx.fillText(d.toFixed(2).replace(".", ","), marge.gauche - 6, py(d));
  }
  for (let eta = 0; eta < 4; eta++) {
    ctx.fillStyle = "#8da2b8";
    ctx.textAlign = "center";
    ctx.textBaseline = "top";
    ctx.fillText(`η = ${eta}`, px(eta), hauteur - marge.bas + 8);
  }
  ctx.strokeStyle = "#4a5b6c";
  ctx.beginPath();
  ctx.moveTo(marge.gauche, marge.haut);
  ctx.lineTo(marge.gauche, hauteur - marge.bas);
  ctx.lineTo(largeur - marge.droite, hauteur - marge.bas);
  ctx.stroke();
  ctx.fillStyle = "#8da2b8";
  ctx.textAlign = "left";
  ctx.textBaseline = "top";
  ctx.fillText("D", 8, marge.haut);

  for (const point of courbe.points) {
    const x = px(point.eta);
    if (Number.isFinite(point.moyenne)) {
      ctx.strokeStyle = "#6cc4ff";
      ctx.fillStyle = "#6cc4ff";
      ctx.lineWidth = 2;
      const haut = py(point.moyenne + point.erreurType);
      const bas = py(point.moyenne - point.erreurType);
      ctx.beginPath();
      ctx.moveTo(x, haut);
      ctx.lineTo(x, bas);
      ctx.moveTo(x - 5, haut);
      ctx.lineTo(x + 5, haut);
      ctx.moveTo(x - 5, bas);
      ctx.lineTo(x + 5, bas);
      ctx.stroke();
      ctx.beginPath();
      ctx.arc(x, py(point.moyenne), 4, 0, 2 * Math.PI);
      ctx.fill();
      ctx.lineWidth = 1;
    }
    if (Number.isFinite(point.repere)) {
      ctx.fillStyle = "#ff6b6b";
      ctx.fillRect(x + 12 - 4, py(point.repere) - 4, 8, 8);
    }
  }
}

/** Un pas du calcul de la courbe : un amas, puis une pause (setTimeout) avant le suivant pour que la page se redessine. */
function pasDeLaCourbe() {
  m.givre_courbe_pas();
  const courbe = lireCourbe(m);
  dessinerCourbeEta(courbe);
  affichageAvancement.textContent = texteAvancement(courbe);
  if (courbe.code === 0 && courbe.suivant !== null) {
    minuterieCourbe = setTimeout(pasDeLaCourbe, PAUSE_COURBE_MS);
  } else {
    boutonCalculerCourbe.disabled = false;
  }
}

/** Démarre le calcul de la courbe D(η) : un nouveau travail du module, puis le premier pas après une pause. */
function calculerLaCourbe() {
  if (m === null) {
    return;
  }
  clearTimeout(minuterieCourbe);
  m.givre_courbe_demarrer(COURBE_TAILLE, COURBE_GRAINES);
  const courbe = lireCourbe(m);
  dessinerCourbeEta(courbe);
  affichageAvancement.textContent = texteAvancement(courbe);
  if (courbe.code !== 0) {
    return;
  }
  boutonCalculerCourbe.disabled = true;
  minuterieCourbe = setTimeout(pasDeLaCourbe, PAUSE_COURBE_MS);
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
  boutonCalculerCourbe.addEventListener("click", calculerLaCourbe);
  choixModele.addEventListener("change", relancer);
  curseurEta.addEventListener("input", afficherEta);
  curseurEta.addEventListener("change", relancer);
  champGraine.addEventListener("keydown", (evenement) => {
    if (evenement.key === "Enter") {
      relancer();
    }
  });
  curseurVitesse.addEventListener("input", afficherVitesse);
  afficherVitesse();
  afficherEta();
  dessinerCourbeEta(lireCourbe(m));
  relancer();
}

charger();

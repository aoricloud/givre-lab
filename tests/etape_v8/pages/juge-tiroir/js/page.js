// Le fil de la page de « juge-tiroir » : il ne calcule rien, il pose ce que le worker envoie. Page d'essai des tests de V8 : pas la page de Givre.
import { blocsCaches, blocsNeige, formater, formaterReglage, imagesCachees, lireGraine, lireJeuxNeige, numeroDuModele, texteAvancement, texteDeLaFormeNeige, texteDuJeuNeige, texteFinNeige, texteMesuresNeige, texteRefusNeige, texteRepereEta, texteSourceNeige } from "../lecture.js";

const element = (id) => document.getElementById(id);
const scene = element("vitrine-nette");
const pinceau = scene.getContext("bitmaprenderer");
const tiroir = element("tiroir");
const SECOURS = "data:text/javascript;charset=utf-8," + encodeURIComponent("postMessage({ type: 'secours' });");
const DELAI_DU_WORKER_MS = 1500;

let ouvrier = new Worker(new URL("../ouvriers/cristal.js", import.meta.url), { type: "module" });
let numero = 0;
let attendu = 0;
let image = null;
let reponse = false;

function demander(type, corps) {
  numero += 1;
  attendu = numero;
  ouvrier.postMessage({ id: numero, type, ...corps });
}

function surReponse(evenement) {
  const message = evenement.data;
  reponse = true;
  if (message.id !== attendu && message.type !== "jeux") {
    return;
  }
  if (message.type === "refus") {
    element("source-neige").textContent = texteRefusNeige(message.code);
    return;
  }
  if (message.type === "jeux") {
    for (const jeu of lireJeuxNeige(message.module)) {
      element("jeu-neige").append(new Option(texteDuJeuNeige(jeu), String(jeu.indice)));
    }
    return;
  }
  image = message.image;
  const neige = message.neige;
  const caches = blocsNeige(neige);
  element("commandes-neige").hidden = caches.commandes;
  element("vitrine").hidden = imagesCachees(message.nette).nette === false;
  element("temps").textContent = "pas " + formater(message.temps) + " / " + formater(message.duree);
  if (neige) {
    // Ligne 1 de la légende du guide : la figure, le rang et les réglages sont LUS dans le module (jamais écrits ici).
    element("legende-nette").textContent = "Figure " + neige.figure + " de Gravner et Griffeath (Physica D, 2008), cristal " + neige.rang + " sur " + neige.cristaux
      + " : vapeur ρ = " + formaterReglage(neige.reglages.rho) + ", seuil β = " + formaterReglage(neige.reglages.beta) + ".";
    element("source-neige").textContent = texteSourceNeige(neige, message.modifie);
    element("mesures-neige").textContent = texteMesuresNeige(message.mesures) + " " + texteFinNeige(message.mesures);
    element("forme-neige").textContent = texteDeLaFormeNeige(message.mesures.forme);
  }
  element("avancement-courbe").textContent = texteAvancement(message.courbe);
  element("repere-eta").textContent = texteRepereEta(Number(element("eta").value), message.repere, message.dimension);
}

function brancher(travailleur) {
  travailleur.addEventListener("message", surReponse);
  travailleur.addEventListener("error", (erreur) => {
    element("source-neige").textContent = "Le calcul a échoué : " + String(erreur.message).replace(/"/g, "'");
  });
}

function demarrer() {
  const choix = element("modele").value;
  const caches = blocsCaches(choix);
  element("commande-eta").hidden = caches.eta;
  demander("demarrer", { modele: numeroDuModele(choix, Number(element("eta").value)), graine: lireGraine(element("graine").value), jeu: Number(element("jeu-neige").value) });
}

function images() {
  if (image) {
    pinceau.transferFromImageBitmap(image);
    image = null;
  }
  demander("avancer", { pas: Number(element("vitesse").value), largeur: scene.width / 2, hauteur: scene.height / 2 });
  requestAnimationFrame(images);
}

brancher(ouvrier);
setTimeout(() => {
  if (!reponse) {
    // Le worker ne répond pas : un worker à l'adresse data:, puis le fil de la page.
    ouvrier = new Worker(SECOURS);
    brancher(ouvrier);
  }
}, DELAI_DU_WORKER_MS);
element("ouvrir").addEventListener("click", () => tiroir.showModal());
element("relancer").addEventListener("click", demarrer);
element("modele").addEventListener("change", demarrer);
element("mesurer").addEventListener("click", () => demander("mesurer", {}));
element("calculer-courbe").addEventListener("click", () => demander("courbe", { taille: 60, graines: 4 }));
element("compatible").textContent = "compatible, ce qui ne prouve pas que le zinc suit la DLA";
demander("jeux", {});
images();

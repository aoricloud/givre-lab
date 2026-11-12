// La page d'essai des tests de l'étape V8 (la « page rhabillée » à la façon du guide des vitrines) : le fil de la page ne calcule rien, il pose ce qu'un worker lui envoie.
// Ce fichier n'est PAS la page de Givre : c'est un exemple que les tests assouplis doivent laisser passer. Il emploie lecture.js (le même que celui de la page) et n'écrit aucun réglage d'un jeu nommé.
import {
  blocsNeige,
  formater,
  lireGraine,
  lireJeuxNeige,
  numeroDuModele,
  texteAvancement,
  texteDeLaFormeNeige,
  texteDuJeuNeige,
  texteRefusNeige,
  texteRepereEta,
} from "./lecture.js";

const tiroir = document.getElementById("reglages");
const ouvrir = document.getElementById("ouvrir-reglages");
const scene = document.getElementById("vitrine-nette");
const dessin = scene.getContext("2d");
const choix = document.getElementById("modele");
const curseurEta = document.getElementById("eta");
const graine = document.getElementById("graine");
const menu = document.getElementById("jeu-neige");
const compatible = document.getElementById("compatible");
const pasUnAccordCourbe = document.getElementById("pas-un-accord-courbe");
const formeNeige = document.getElementById("forme-neige");
const etat = document.getElementById("etat");

// Les phrases d'honnêteté que cette page écrit par un script plutôt que dans le HTML (les tests lisent aussi les chaînes des scripts).
const PHRASE_COMPATIBLE = "compatible, ce qui ne prouve pas que le zinc suit la DLA";
const PHRASE_PAS_UN_ACCORD = "Ce n'est pas un accord : le banc et le repère publié n'ont pas la même méthode.";

// Le calcul est hors du fil de la page : un worker (module) qui avance le module, lit les masses, dessine et envoie l'image.
const calcul = new Worker(new URL("./calcul.js", import.meta.url), { type: "module" });
let derniereImage = null;
let mesures = null;

calcul.onmessage = (message) => {
  const donnee = message.data;
  if (donnee.refus !== undefined) {
    // Un refus du module est affiché, jamais avalé.
    etat.textContent = texteRefusNeige(donnee.refus);
    return;
  }
  derniereImage = donnee.image;
  mesures = donnee.mesures;
  formeNeige.textContent = texteDeLaFormeNeige(donnee.forme);
};

function images() {
  if (derniereImage) {
    dessin.putImageData(derniereImage, 0, 0);
  }
  requestAnimationFrame(images);
}

function demarrer() {
  const modele = numeroDuModele(choix.value, Number(curseurEta.value));
  calcul.postMessage({ demarrer: modele, graine: lireGraine(graine.value), neige: choix.value === "neige", jeu: Number(menu.value) });
  const cache = blocsNeige(choix.value === "neige" ? { jeu: Number(menu.value) } : null);
  document.getElementById("commandes-neige").hidden = cache.commandes;
  etat.textContent = texteAvancement(null) + " " + formater(0);
}

ouvrir.addEventListener("click", () => {
  tiroir.hidden = !tiroir.hidden;
});
document.getElementById("relancer").addEventListener("click", demarrer);
choix.addEventListener("change", demarrer);

calcul.postMessage({ jeux: true });
calcul.addEventListener("message", (message) => {
  if (message.data.jeux) {
    for (const jeu of lireJeuxNeige(message.data.jeux)) {
      const option = document.createElement("option");
      option.textContent = texteDuJeuNeige(jeu);
      menu.appendChild(option);
    }
  }
});

compatible.textContent = PHRASE_COMPATIBLE;
pasUnAccordCourbe.textContent = PHRASE_PAS_UN_ACCORD;
document.getElementById("valeur-eta").textContent = texteRepereEta(Number(curseurEta.value), Number.NaN, Number.NaN);
images();

// La page d'essai des tests de l'étape V8 (la « page rhabillée » à la façon du guide des vitrines) : le fil de la page ne calcule rien, il pose ce qu'un worker lui envoie.
// Ce fichier n'est PAS la page de Givre : c'est un exemple que les tests assouplis doivent laisser passer. Il emploie lecture.js (le même que celui de la page) et n'écrit aucun réglage d'un jeu nommé.
import {
  blocsMetal,
  blocsNeige,
  formater,
  lireGraine,
  lireJeuxNeige,
  numeroDuModele,
  texteAvancement,
  texteDeLaFormeNeige,
  texteDuJeuNeige,
  texteRefusMetal,
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
const toileMetal = document.getElementById("vitrine-metal");
const dessinMetal = toileMetal.getContext("2d");

// La légende du métal, écrite par un script (les tests lisent les chaînes des scripts quand l'élément est vide).
const LEGENDE_METAL =
  "Notre modèle, en 2D, pas une reproduction : en régime établi notre calcul donne 0,0408 ; la vitesse de pointe publiée par Kim et coll. (0,0469, soit 0,0938 en vitesse affichée) n'est pas atteinte. Le mur est à nous : la pointe s'arrête avant le bord. Le régime n'est pas établi. Le rayon est celui de notre mesure, qui dépend de la profondeur.";

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
  if (donnee.refusMetal !== undefined) {
    etat.textContent = texteRefusMetal(donnee.refusMetal);
    return;
  }
  if (donnee.metal !== undefined) {
    dessinMetal.putImageData(new ImageData(donnee.metal.rgba, donnee.metal.largeur, donnee.metal.hauteur), 0, 0);
    document.getElementById("reglages-metal").textContent = donnee.texteReglages;
    document.getElementById("mesures-metal").textContent = donnee.texteMesures;
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
  const metal = choix.value === "metal";
  calcul.postMessage({
    demarrer: modele,
    graine: lireGraine(graine.value),
    neige: choix.value === "neige",
    metal,
    epsilon: Number(document.getElementById("metal-epsilon").value),
    cote: Number(document.getElementById("metal-cote").value),
    jeu: Number(menu.value),
  });
  const cacheMetal = blocsMetal(metal);
  document.getElementById("commandes-metal").hidden = cacheMetal.commandes;
  toileMetal.hidden = cacheMetal.image;
  document.getElementById("mesures-metal-bloc").hidden = cacheMetal.mesures;
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
document.getElementById("legende-metal").textContent = LEGENDE_METAL;
document.getElementById("metal-epsilon").addEventListener("input", (evenement) => {
  document.getElementById("valeur-metal-epsilon").textContent = String(Number(evenement.target.value) / 1000000);
});
pasUnAccordCourbe.textContent = PHRASE_PAS_UN_ACCORD;
document.getElementById("valeur-eta").textContent = texteRepereEta(Number(curseurEta.value), Number.NaN, Number.NaN);
images();

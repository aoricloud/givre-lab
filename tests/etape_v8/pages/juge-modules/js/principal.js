// L'entrée de la page « juge-modules » : elle relie les modules et écrit les phrases que le HTML laisse vides.
import { blocsCaches, formater, lireGraine, numeroDuModele } from "../lecture.js";
import { etat, poser, repli } from "./dessin.js";
import { courbe, mesurer } from "./mesure.js";
import { leModule } from "./module.js";
import * as neige from "./neige.js";

const m = await leModule();
const par = (id) => document.getElementById(id);
const PHRASES = {
  "legende-neige": 'Le cristal s\'arrête près du mur : le mur est à nous ; au-delà ce ne serait plus la neige de l\'article. Ce sont des stades précoces.',
  "repere-zinc": 'zinc, Matsushita et coll. 1984 : 1,66 \u00b1 0,03',
  "taille-zinc": 'taille d\'amas du zinc non relevée',
  "pas-un-accord": 'Ce n\'est pas un accord.',
  "pas-un-accord-courbe": 'Ce n\'est pas un accord : deux méthodes.',
  compatible: "compatible, ce qui ne prouve pas que le zinc suit la DLA",
};

function demarrer() {
  const choix = par("modele").value;
  par("commande-eta").hidden = blocsCaches(choix).eta;
  if (choix !== "neige") {
    return m.givre_demarrer(numeroDuModele(choix, Number(par("eta").value)), lireGraine(par("graine").value));
  }
  const quatre = { rho: Number(par("neige-rho").value), beta: Number(par("neige-beta").value), kappa: Number(par("neige-kappa").value), mu: Number(par("neige-mu").value) };
  const lu = neige.demarrer(m, Number(par("jeu-neige").value), quatre);
  par("source-neige").textContent = lu.refus === undefined ? lu.source : lu.refus;
  return 1;
}

function image() {
  m.givre_avancer(Number(par("vitesse").value));
  const vu = etat(m);
  par("temps").textContent = formater(vu.nombres[0]);
  const textes = neige.textes(m);
  if (textes !== null) {
    par("forme-neige").textContent = textes.forme + " (" + neige.masses(m).length + ")";
  } else {
    poser(par("vitrine-nette"), repli(m).image);
  }
  requestAnimationFrame(image);
}

for (const [id, phrase] of Object.entries(PHRASES)) {
  par(id).textContent = phrase;
}
for (const texte of neige.jeux(m)) {
  par("jeu-neige").append(new Option(texte));
}
par("relancer").addEventListener("click", demarrer);
par("mesurer").addEventListener("click", () => {
  par("repere-eta").textContent = mesurer(m, Number(par("eta").value)).repere;
});
par("calculer-courbe").addEventListener("click", () => {
  par("avancement-courbe").textContent = courbe(m, 60, 4).texte;
});
demarrer();
image();

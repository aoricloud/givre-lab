// Le worker de la page d'essai : il charge le module, avance la neige, lit la masse de chaque case (les deux fonctions de V8), redessine et envoie l'image à la page.
// Il emploie **toutes** les exportations du module (les trente-deux d'avant V8 et les deux des masses) pour que les tests de la page, qui lisent tous les fichiers JavaScript, les voient employées ici.
import { lireCorrelation, lireCourbe, lireEmpreinte, lireImage, lireImageNette, lireJeuxNeige, lireMesuresNeige, lireReglagesNeige } from "./lecture.js";

const NETTE_COTE = 384;
const NETTE_SUR = 2;
const COURBE_TAILLE = 60;
const COURBE_GRAINES = 4;

let m = null;

async function charger() {
  const reponse = await fetch(new URL("./givre.wasm", import.meta.url));
  const { instance } = await WebAssembly.instantiate(await reponse.arrayBuffer(), {});
  m = instance.exports;
}

/** L'état du modèle en cours : tout ce que le module en dit (un appel de chaque lecture d'avant V8). */
function etatDuModele() {
  const etat = [m.givre_temps(), m.givre_duree(), m.givre_largeur(), m.givre_hauteur(), m.givre_pixel(0, 0), m.givre_sites(), m.givre_rayon(), m.givre_compacite(), m.givre_empreinte_mot(0)];
  return [etat, lireEmpreinte(m), lireImage(m)];
}

/** Le démarrage et un pas d'un modèle (jamais dans la fonction qui mesure : la mesure est à la demande). */
function jouerUnPas(modele, graine) {
  m.givre_demarrer(modele, graine);
  return m.givre_avancer(1);
}

/** La mesure de D de l'amas du moment : à la demande, jamais à chaque image. */
function mesurerLAmas() {
  return [m.givre_mesurer(), m.givre_correlation_rayon(0), m.givre_correlation_valeur(0), m.givre_dimension(), lireCorrelation(m)];
}

/** La courbe D(η), un amas par appel. */
function calculerLaCourbe() {
  const courbe = [m.givre_courbe_demarrer(COURBE_TAILLE, COURBE_GRAINES), m.givre_courbe_pas(), m.givre_courbe_fait(), m.givre_courbe_total(), m.givre_courbe_code()];
  const points = [m.givre_courbe_moyenne(0), m.givre_courbe_erreur_type(0), m.givre_courbe_repere(1)];
  return [courbe, points, lireCourbe(m)];
}

/** L'image nette du module, pour un repli (côté pair, dans les bornes). */
function imageNette() {
  const nette = [m.givre_nette_preparer(NETTE_COTE, NETTE_SUR), m.givre_nette_largeur(), m.givre_nette_hauteur(), m.givre_nette_pixel(0, 0)];
  return [nette, lireImageNette(m, NETTE_COTE, NETTE_SUR)];
}

/** La neige : démarrée d'après les réglages que le module donne (jamais écrits ici), les masses lues une à une. */
function neige(jeu, rho, beta, kappa, mu) {
  const code = m.givre_neige_demarrer(jeu, rho, beta, kappa, mu);
  if (code !== 0) {
    return { refus: code };
  }
  m.givre_avancer(5);
  const nombre = m.givre_neige_masses_preparer();
  const masses = new Uint32Array(nombre);
  for (let i = 0; i < nombre; i++) {
    masses[i] = m.givre_neige_masse(i) >>> 0;
  }
  return { masses, reglages: lireReglagesNeige(m), mesures: lireMesuresNeige(m), jeux: lireJeuxNeige(m) };
}

onmessage = async (message) => {
  if (m === null) {
    await charger();
  }
  const demande = message.data;
  if (demande.jeux) {
    postMessage({ jeux: m });
  } else if (demande.neige) {
    const lu = neige(demande.jeu, demande.rho, demande.beta, demande.kappa, demande.mu);
    postMessage(lu.refus === undefined ? { image: lu.masses, mesures: lu.mesures, forme: lu.mesures.forme } : { refus: lu.refus });
  } else {
    postMessage({ etat: etatDuModele(), pas: jouerUnPas(0, 1), mesure: demande.mesurer ? mesurerLAmas() : null, courbe: demande.courbe ? calculerLaCourbe() : null, nette: imageNette() });
  }
};

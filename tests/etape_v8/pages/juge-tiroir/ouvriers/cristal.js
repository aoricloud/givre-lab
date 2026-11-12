// Le worker de « juge-tiroir » : il charge le module, avance, lit les masses, dessine et envoie l'image. Un seul gestionnaire de messages, qui avance ET mesure (selon le message).
import * as lecture from "../lecture.js";

let m = null;
let dernier = { id: 0 };

async function charger() {
  const octets = await (await fetch(new URL("../givre.wasm", import.meta.url))).arrayBuffer();
  m = (await WebAssembly.instantiate(octets, {})).instance.exports;
}

function lireLEtat() {
  return {
    temps: m.givre_temps(), duree: m.givre_duree(), largeur: m.givre_largeur(), hauteur: m.givre_hauteur(), coin: m.givre_pixel(0, 0),
    sites: m.givre_sites(), rayon: m.givre_rayon(), compacite: m.givre_compacite(), mot: m.givre_empreinte_mot(0),
    empreinte: lecture.lireEmpreinte(m), image: lecture.lireImage(m),
  };
}

function lireLaMesure() {
  return { code: m.givre_mesurer(), rayon: m.givre_correlation_rayon(0), valeur: m.givre_correlation_valeur(0), dimension: m.givre_dimension(), courbe: lecture.lireCorrelation(m) };
}

function avancerLaCourbe(taille, graines) {
  const code = taille === undefined ? m.givre_courbe_code() : m.givre_courbe_demarrer(taille, graines);
  return {
    code, pas: m.givre_courbe_pas(), fait: m.givre_courbe_fait(), total: m.givre_courbe_total(),
    point: [m.givre_courbe_moyenne(0), m.givre_courbe_erreur_type(0), m.givre_courbe_repere(0)], lue: lecture.lireCourbe(m),
  };
}

function lireLImageNette(cote, sur) {
  const pair = cote - (cote % 2);
  const code = m.givre_nette_preparer(pair, sur);
  return { code, largeur: m.givre_nette_largeur(), hauteur: m.givre_nette_hauteur(), coin: m.givre_nette_pixel(0, 0), image: lecture.lireImageNette(m, pair, sur) };
}

function lireLesMasses() {
  const nombre = m.givre_neige_masses_preparer() >>> 0;
  const masses = new Uint32Array(nombre);
  for (let i = 0; i < nombre; i += 1) {
    masses[i] = m.givre_neige_masse(i) >>> 0;
  }
  return masses;
}

function dessiner(masses, largeur, hauteur) {
  const toile = new OffscreenCanvas(largeur, hauteur);
  const trait = toile.getContext("2d");
  trait.imageSmoothingEnabled = true;
  const plein = 3072;
  for (let i = 0; i < masses.length; i += 1) {
    const t = Math.floor(Math.min(masses[i], plein) / 3);
    trait.fillStyle = "rgb(" + (255 - (t >> 3)) + " " + (255 - (t >> 4)) + " 255)";
    trait.fillRect(i % largeur, (i / largeur) | 0, 1, 1);
  }
  return toile.transferToImageBitmap();
}

function repondre(id, corps, transferts) {
  postMessage({ id, ...corps }, transferts || []);
}

async function surMessage(evenement) {
  const message = evenement.data;
  if (m === null) {
    await charger();
  }
  if (message.id === dernier.id) {
    return;
  }
  dernier.id = message.id;
  switch (message.type) {
    case "jeux":
      repondre(message.id, { type: "jeux", module: m });
      break;
    case "demarrer": {
      if (message.modele < 8) {
        repondre(message.id, { type: "etat", demarre: m.givre_demarrer(message.modele, message.graine) });
        break;
      }
      // La neige : les quatre réglages viennent des curseurs de la page, eux-mêmes posés d'après le module.
      const code = m.givre_neige_demarrer(message.jeu, message.vapeur, message.seuil, message.gel, message.fonte);
      repondre(message.id, code === 0 ? { type: "etat", neige: lecture.lireReglagesNeige(m) } : { type: "refus", code });
      break;
    }
    case "avancer": {
      const joues = m.givre_avancer(message.pas);
      const masses = lireLesMasses();
      const image = dessiner(masses, message.largeur, message.hauteur);
      repondre(message.id, { type: "etat", joues, image, etat: lireLEtat(), neige: lecture.lireReglagesNeige(m), mesures: lecture.lireMesuresNeige(m), nette: masses.length > 0 ? 0 : lireLImageNette(message.largeur, 2).code }, [image]);
      break;
    }
    case "mesurer":
      repondre(message.id, { type: "etat", code: m.givre_mesurer(), mesure: lireLaMesure() });
      break;
    case "courbe":
      repondre(message.id, { type: "etat", courbe: avancerLaCourbe(message.taille, message.graines) });
      break;
    default:
      repondre(message.id, { type: "refus", code: 1 });
  }
}

addEventListener("message", surMessage);

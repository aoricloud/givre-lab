// Ce que la page lit du module (l'empreinte, l'image, l'image nette, la graine, la mesure de corrélation, le travail de la courbe D(η)) et les textes qu'elle en tire. Aucun DOM ici : Node l'éprouve contre le code natif.

const GRAINE_MAX = 4294967295;

/** Les 64 chiffres hexadécimaux de l'empreinte : les huit mots, non signés, sur 8 chiffres chacun. */
export function lireEmpreinte(m) {
  let texte = "";
  for (let i = 0; i < 8; i++) {
    texte += (m.givre_empreinte_mot(i) >>> 0).toString(16).padStart(8, "0");
  }
  return texte;
}

/** Les octets R, V, B, 255 de `largeur × hauteur` pixels, ligne par ligne de haut en bas ; `pixel(x, y)` rend la couleur `0x00RRGGBB` du module. */
function rgbaDepuis(largeur, hauteur, pixel) {
  const rgba = new Uint8ClampedArray(4 * largeur * hauteur);
  for (let y = 0; y < hauteur; y++) {
    for (let x = 0; x < largeur; x++) {
      const couleur = pixel(x, y) >>> 0;
      const k = 4 * (y * largeur + x);
      rgba[k] = (couleur >>> 16) & 255;
      rgba[k + 1] = (couleur >>> 8) & 255;
      rgba[k + 2] = couleur & 255;
      rgba[k + 3] = 255;
    }
  }
  return rgba;
}

/** L'image courante, prête pour `new ImageData(rgba, largeur, hauteur)`. */
export function lireImage(m) {
  const largeur = m.givre_largeur() >>> 0;
  const hauteur = m.givre_hauteur() >>> 0;
  return { largeur, hauteur, rgba: rgbaDepuis(largeur, hauteur, (x, y) => m.givre_pixel(x, y)) };
}

/**
 * L'image nette du modèle en cours (chaque case un hexagone lisse), prête pour `new ImageData(rgba, largeur, hauteur)` : **demande au module de la préparer**
 * (`givre_nette_preparer(cote, sur)` : `cote` pixels de côté, `sur × sur` échantillons par pixel), puis la lit. Rend { code, largeur, hauteur, rgba } : `code` est celui du
 * module (0 : l'image est prête ; 1 : le modèle est sur une grille carrée, il n'a pas d'image nette ; les autres sont des refus de la préparation). Avec un code non nul,
 * `largeur` et `hauteur` valent 0 et `rgba` est vide : rien n'est lu.
 */
export function lireImageNette(m, cote, sur) {
  const code = m.givre_nette_preparer(cote, sur) >>> 0;
  if (code !== 0) {
    return { code, largeur: 0, hauteur: 0, rgba: new Uint8ClampedArray(0) };
  }
  const largeur = m.givre_nette_largeur() >>> 0;
  const hauteur = m.givre_nette_hauteur() >>> 0;
  return { code, largeur, hauteur, rgba: rgbaDepuis(largeur, hauteur, (x, y) => m.givre_nette_pixel(x, y)) };
}

/**
 * Quel canvas la page cache, d'après le code de `lireImageNette` : l'image nette (code 0) remplace l'image d'un pixel par case ; avec tout autre code (le modèle est sur une grille
 * carrée, ou la préparation a été refusée) la page garde l'image d'un pixel par case. Rend { carree, nette } : vrai quand le canvas est caché.
 */
export function imagesCachees(code) {
  return { carree: code === 0, nette: code !== 0 };
}

/** La graine écrite dans un champ : des chiffres décimaux seulement (espaces autour permis), au plus 4294967295 ; sinon null. */
export function lireGraine(texte) {
  const nu = String(texte).trim();
  if (!/^[0-9]+$/.test(nu)) {
    return null;
  }
  const valeur = Number(nu);
  return valeur <= GRAINE_MAX ? valeur : null;
}

/** La mesure de corrélation gardée par le module : ses rayons, ses C(r) et sa dimension. Ne mesure rien. */
export function lireCorrelation(m) {
  const rayons = [];
  const valeurs = [];
  for (let i = 0; ; i++) {
    const rayon = m.givre_correlation_rayon(i) >>> 0;
    if (rayon === 0) {
      break;
    }
    rayons.push(rayon);
    valeurs.push(m.givre_correlation_valeur(i));
  }
  return { rayons, valeurs, dimension: m.givre_dimension() };
}

/**
 * Le travail de la courbe D(η) gardé par le module : le dernier code, les amas faits, le total, les graines par point (total / 4), les quatre
 * points (η de 0 à 3 : moyenne et erreur-type, NaN tant que les amas de cet η ne sont pas tous faits ; repère publié, NaN s'il n'y en a pas) et le
 * prochain amas à faire ({ eta, graine } avec la graine de 1 à `graines`, ou null quand tout est fait ou qu'il n'y a pas de travail). Ne calcule rien.
 */
export function lireCourbe(m) {
  const code = m.givre_courbe_code() >>> 0;
  const fait = m.givre_courbe_fait() >>> 0;
  const total = m.givre_courbe_total() >>> 0;
  const graines = total / 4;
  const points = [];
  for (let eta = 0; eta < 4; eta++) {
    points.push({
      eta,
      moyenne: m.givre_courbe_moyenne(eta),
      erreurType: m.givre_courbe_erreur_type(eta),
      repere: m.givre_courbe_repere(eta),
    });
  }
  const suivant = fait < total ? { eta: Math.floor(fait / graines), graine: (fait % graines) + 1 } : null;
  return { code, fait, total, graines, points, suivant };
}

// D'où vient le repère de chaque η : un nom, jamais une valeur (la valeur vient du module, givre_courbe_repere).
const SOURCES_REPERES = { 1: "rappel de la DLA", 2: "Hastings 2001", 3: "Hastings 2001" };

// Ce que veut dire chaque code d'erreur du calcul de la courbe (la table de la doc de src/vitrine.rs).
const MESSAGES_CODES = {
  1: "taille hors borne",
  2: "nombre de graines hors borne",
  3: "un amas a touché la frontière avant sa taille",
  4: "un amas n'a pas de dimension (fenêtre vide)",
  5: "aucun calcul démarré",
  6: "erreur de la bibliothèque",
};

// Le premier modèle du claquage dans le module : le modèle 4 + η.
const MODELE_CLAQUAGE = 4;

/** Écrit un nombre avec trois décimales et une virgule : 1,234. */
export function formater(valeur) {
  return valeur.toFixed(3).replace(".", ",");
}

/** Le numéro de modèle du module pour le choix de la liste (« 0 » à « 3 », ou « claquage ») et l'η du curseur : 0 à 3 tels quels, 4 + η pour le claquage. */
export function numeroDuModele(choix, eta) {
  return choix === "claquage" ? MODELE_CLAQUAGE + eta : Number(choix);
}

/**
 * Ce que la page cache pour le choix du modèle de la liste : les phrases du zinc ne sont que pour la DLA et les flocons ; le curseur η et le repère de η ne
 * sont que pour le claquage. Rend { zinc, repereEta, commandeEta } : vrai quand le bloc est caché.
 */
export function blocsCaches(choix) {
  const claquage = choix === "claquage";
  return { zinc: claquage, repereEta: !claquage, commandeEta: !claquage };
}

/** Le texte d'avancement du calcul de la courbe : l'amas qu'on va faire, ou la fin, ou l'erreur qui l'a arrêté. `courbe` est ce que rend `lireCourbe`. */
export function texteAvancement(courbe) {
  if (courbe.code !== 0) {
    return `Calcul arrêté : ${MESSAGES_CODES[courbe.code] ?? "erreur"} (code ${courbe.code}).`;
  }
  if (courbe.total === 0) {
    return "Aucun calcul pour l'instant.";
  }
  if (courbe.suivant !== null) {
    return `η = ${courbe.suivant.eta}, graine ${courbe.suivant.graine} sur ${courbe.graines}`;
  }
  return `Calcul terminé : ${courbe.fait} amas.`;
}

/**
 * Les phrases du repère publié de l'η du curseur : `repere` est ce que rend `givre_courbe_repere(eta)` (NaN s'il n'y en a pas) et `dimension` la
 * dimension mesurée du banc. Rend { repere, ecart } : le texte du repère avec sa source et le texte de l'écart du banc (signé), ou `ecart` à null
 * quand il n'y a pas de repère.
 */
export function texteRepereEta(eta, repere, dimension) {
  if (Number.isNaN(repere)) {
    return { repere: `pas de repère publié pour η = ${eta}`, ecart: null };
  }
  const ecart = dimension - repere;
  return {
    repere: `repère publié (${SOURCES_REPERES[eta]}) : ${formater(repere)}`,
    ecart: `écart du banc au repère : ${ecart < 0 ? "−" : "+"}${formater(Math.abs(ecart))}`,
  };
}

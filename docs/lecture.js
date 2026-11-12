// Ce que la page lit du module (l'empreinte, l'image, l'image nette, la graine, la mesure de corrélation, le travail de la courbe D(η), la neige : ses jeux nommés, ses réglages, ses mesures) et les textes qu'elle en tire. Aucun DOM ici : Node l'éprouve contre le code natif.

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

// ---------------------------------------------------------------------------------------------------------------
// La neige de Gravner et Griffeath (étape G3) : ce que la page lit du module et les textes qu'elle en tire. Les réglages passent en millionièmes (des entiers) ; aucun réglage n'est écrit ici : les jeux nommés viennent du module.
// ---------------------------------------------------------------------------------------------------------------

// « Pas de valeur » du module pour la neige (rien en cours, ou un champ hors de portée) : `u32::MAX`.
const NEIGE_ABSENTE = 4294967295;

// Les sept réglages dans l'ordre du module (numéros 0 à 6) : leur lettre et leur clé.
const REGLAGES_NEIGE = [
  { lettre: "ρ", cle: "rho" },
  { lettre: "β", cle: "beta" },
  { lettre: "α", cle: "alpha" },
  { lettre: "θ", cle: "theta" },
  { lettre: "κ", cle: "kappa" },
  { lettre: "μ", cle: "mu" },
  { lettre: "γ", cle: "gamma" },
];

// Les étiquettes de forme dans l'ordre du module (0 à 4) : notre classement, pas celui de l'article.
const FORMES_NEIGE = ["trop petit pour être rangé", "plaque", "plaque à secteurs", "dendrite", "fougère"];

/** Les sept réglages en millionièmes, lus par `lire(numero)` (le numéro de 0 à 6), sous leur clé : { rho, beta, alpha, theta, kappa, mu, gamma }. */
function reglagesNeige(lire) {
  const reglages = {};
  REGLAGES_NEIGE.forEach(({ cle }, numero) => {
    reglages[cle] = lire(numero) >>> 0;
  });
  return reglages;
}

/**
 * Les jeux de réglages nommés de la neige, lus dans le module (`givre_neige_jeux`, `givre_neige_jeu`) : un tableau, dans l'ordre du module, de { indice, figure, rang, cristaux, balaye, reglages, rayon, pas, deduit }.
 * `balaye` est le numéro du réglage qui varie dans la figure (0 à 6 ; 7 : aucun), `reglages` les sept réglages en millionièmes, `deduit` vrai si les réglages sont déduits de l'annexe de l'article. Ne démarre rien.
 */
export function lireJeuxNeige(m) {
  const nombre = m.givre_neige_jeux() >>> 0;
  const jeux = [];
  for (let indice = 0; indice < nombre; indice++) {
    const champ = (c) => m.givre_neige_jeu(indice, c) >>> 0;
    jeux.push({
      indice,
      figure: champ(0),
      rang: champ(1),
      cristaux: champ(2),
      balaye: champ(3),
      reglages: reglagesNeige((numero) => champ(4 + numero)),
      rayon: champ(11),
      pas: champ(12),
      deduit: champ(13) === 1,
    });
  }
  return jeux;
}

/**
 * Les réglages de la neige **en cours dans le module** (pas ceux des curseurs) : { jeu, rayon, pas, reglages } ou null quand le module n'a pas de neige en cours (rien n'est démarré, ou le modèle en cours est un autre).
 */
export function lireReglagesNeige(m) {
  const jeu = m.givre_neige_reglage(7) >>> 0;
  if (jeu === NEIGE_ABSENTE) {
    return null;
  }
  return {
    jeu,
    rayon: m.givre_neige_reglage(8) >>> 0,
    pas: m.givre_neige_reglage(9) >>> 0,
    reglages: reglagesNeige((numero) => m.givre_neige_reglage(numero)),
  };
}

/**
 * Les mesures de la neige en cours, à l'instant (`givre_neige_mesure` recalcule à chaque appel : la page ne les lit pas à chaque image) : { rattachees, rayon, compacite, pointes, aiguilles, hexagonalite, forme, auMur }
 * ou null sans neige en cours. `forme` est le numéro de l'étiquette (0 à 4), `auMur` vrai quand le cristal a atteint la limite du mur.
 */
export function lireMesuresNeige(m) {
  const rattachees = m.givre_neige_mesure(0) >>> 0;
  if (rattachees === NEIGE_ABSENTE) {
    return null;
  }
  return {
    rattachees,
    rayon: m.givre_neige_mesure(1) >>> 0,
    compacite: m.givre_neige_mesure(2) >>> 0,
    pointes: m.givre_neige_mesure(3) >>> 0,
    aiguilles: m.givre_neige_mesure(4) >>> 0,
    hexagonalite: m.givre_neige_mesure(5) >>> 0,
    forme: m.givre_neige_mesure(6) >>> 0,
    auMur: (m.givre_neige_mesure(7) >>> 0) === 1,
  };
}

/** Un réglage en millionièmes écrit en décimal avec une virgule, sans zéro inutile mais avec au moins deux décimales : 250000 donne « 0,25 », 1500000 « 1,50 », 5 « 0,000005 ». Que des entiers : aucun arrondi. */
export function formaterReglage(millionniemes) {
  const entier = Math.floor(millionniemes / 1000000);
  let fraction = String(millionniemes % 1000000).padStart(6, "0");
  while (fraction.length > 2 && fraction.endsWith("0")) {
    fraction = fraction.slice(0, -1);
  }
  return `${entier},${fraction}`;
}

/** Un nombre de millièmes écrit avec trois décimales et une virgule : 512 donne « 0,512 », 1000 « 1,000 ». */
function formaterMillieme(millieme) {
  return `${Math.floor(millieme / 1000)},${String(millieme % 1000).padStart(3, "0")}`;
}

/** Le texte d'un jeu nommé dans le menu : la figure de l'article, le rang du cristal et le réglage qui varie. Aucun réglage n'y est écrit (il est lu dans le module). */
export function texteDuJeuNeige(jeu) {
  if (jeu.cristaux === 1) {
    return `Figure ${jeu.figure} : le cristal du milieu`;
  }
  const varie = REGLAGES_NEIGE[jeu.balaye];
  return `Figure ${jeu.figure} : cristal ${jeu.rang} sur ${jeu.cristaux}${varie ? ` (${varie.lettre} varie)` : ""}`;
}

/**
 * D'où viennent les réglages d'un jeu : lus dans l'article (la figure 9 : les légendes de la figure donnent ρ, la ligne de l'annexe les six autres, et une autre ligne de l'annexe les redonne tous les sept ; la figure 13 : le texte et l'annexe),
 * ou déduits de l'annexe (la colonne du réglage qui varie y manque et rien d'autre ne la recoupe) ; `modifie` vrai quand les curseurs ne sont plus ceux du jeu :
 * seuls α, θ, γ, le domaine et les pas sont alors encore ceux du jeu.
 */
export function texteSourceNeige(jeu, modifie) {
  if (modifie) {
    return "Curseurs déplacés : ρ, β, κ et μ ne sont plus ceux de la figure ; α, θ, γ, la taille du domaine et le nombre de pas restent ceux du jeu.";
  }
  return jeu.deduit
    ? "Réglages déduits de l'annexe de l'article : la colonne du réglage qui varie y manque."
    : "Réglages lus dans l'article (ses figures, son texte et son annexe se recoupent).";
}

/** Le texte d'une étiquette de forme (le numéro du module) : « notre classement, pas celui de l'article ». */
export function texteDeLaFormeNeige(forme) {
  const nom = FORMES_NEIGE[forme];
  return `Forme (notre classement, pas celui de l'article) : ${nom === undefined ? "inconnue" : nom}.`;
}

/** Le texte des mesures de la neige : cases du cristal, rayon, compacité et hexagonalité (en millièmes), pointes et aiguilles. */
export function texteMesuresNeige(mesures) {
  return `${mesures.rattachees} cases · rayon ${mesures.rayon} · compacité ${formaterMillieme(mesures.compacite)} · pointes ${mesures.pointes} · aiguilles ${mesures.aiguilles} · hexagonalité ${formaterMillieme(mesures.hexagonalite)}`;
}

/** Le texte de la fin d'une neige : le mur du domaine (le mur est à nous : au-delà ce ne serait plus la neige de l'article) ou la durée du jeu. */
export function texteFinNeige(mesures) {
  return mesures.auMur
    ? "Arrêté près du mur du domaine : au-delà, ce ne serait plus la neige de l'article (le mur est à nous, et pour certains réglages le cristal le ressent déjà). Recommencez ou choisissez un autre modèle."
    : "Scénario terminé (tous les pas du jeu sont joués) : recommencez ou choisissez un autre modèle.";
}

/** Le texte d'un refus de `givre_neige_demarrer` (le code du module) : le réglage fautif. */
export function texteRefusNeige(code) {
  const refus = { 1: "jeu inconnu", 2: "ρ hors plage", 3: "β hors plage", 4: "κ hors plage", 5: "μ hors plage" };
  return `La neige ne démarre pas : ${refus[code] ?? "refus du module"} (code ${code}).`;
}

/** Ce que la page montre d'après le modèle en cours (vrai : la neige tourne) : le panneau des réglages et celui des mesures de la neige ne sont montrés que pour elle. Rend { commandes, mesures } : vrai quand le bloc est caché. */
export function blocsNeige(neige) {
  return { commandes: !neige, mesures: !neige };
}

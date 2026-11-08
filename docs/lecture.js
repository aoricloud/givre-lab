// Ce que la page lit du module : l'empreinte, l'image et la graine. Aucun DOM ici : Node l'éprouve contre le code natif.

const GRAINE_MAX = 4294967295;

/** Les 64 chiffres hexadécimaux de l'empreinte : les huit mots, non signés, sur 8 chiffres chacun. */
export function lireEmpreinte(m) {
  let texte = "";
  for (let i = 0; i < 8; i++) {
    texte += (m.givre_empreinte_mot(i) >>> 0).toString(16).padStart(8, "0");
  }
  return texte;
}

/** L'image courante, prête pour `new ImageData(rgba, largeur, hauteur)`. */
export function lireImage(m) {
  const largeur = m.givre_largeur() >>> 0;
  const hauteur = m.givre_hauteur() >>> 0;
  const rgba = new Uint8ClampedArray(4 * largeur * hauteur);
  for (let y = 0; y < hauteur; y++) {
    for (let x = 0; x < largeur; x++) {
      const couleur = m.givre_pixel(x, y) >>> 0;
      const k = 4 * (y * largeur + x);
      rgba[k] = (couleur >>> 16) & 255;
      rgba[k + 1] = (couleur >>> 8) & 255;
      rgba[k + 2] = couleur & 255;
      rgba[k + 3] = 255;
    }
  }
  return { largeur, hauteur, rgba };
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

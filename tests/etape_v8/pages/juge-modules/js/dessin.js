// Le dessin : l'état d'un pixel par case, l'image nette du module en repli.
import { imagesCachees, lireEmpreinte, lireImage, lireImageNette } from "../lecture.js";

const COTE_DU_REPLI = 384;

export function etat(m) {
  const nombres = [m.givre_temps(), m.givre_duree(), m.givre_largeur(), m.givre_hauteur(), m.givre_pixel(0, 0), m.givre_sites(), m.givre_rayon(), m.givre_compacite(), m.givre_empreinte_mot(7)];
  return { nombres, empreinte: lireEmpreinte(m), image: lireImage(m) };
}

export function repli(m) {
  const code = m.givre_nette_preparer(COTE_DU_REPLI, 2);
  const taille = [m.givre_nette_largeur(), m.givre_nette_hauteur(), m.givre_nette_pixel(1, 1)];
  return { code, taille, image: lireImageNette(m, COTE_DU_REPLI, 2), caches: imagesCachees(code) };
}

export function poser(toile, image) {
  const trait = toile.getContext("2d");
  trait.putImageData(image, 0, 0);
}

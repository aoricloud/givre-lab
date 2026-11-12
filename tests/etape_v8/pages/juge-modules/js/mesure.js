// La mesure de D et la courbe D(η).
import { lireCorrelation, lireCourbe, texteAvancement, texteRepereEta } from "../lecture.js";

export function mesurer(m, eta) {
  const code = m.givre_mesurer();
  const points = [m.givre_correlation_rayon(0), m.givre_correlation_valeur(0)];
  return { code, points, lue: lireCorrelation(m), repere: texteRepereEta(eta, m.givre_courbe_repere(eta), m.givre_dimension()) };
}

export function courbe(m, taille, graines) {
  const code = m.givre_courbe_demarrer(taille, graines);
  const fait = [m.givre_courbe_pas(), m.givre_courbe_fait(), m.givre_courbe_total(), m.givre_courbe_code()];
  return { code, fait, point: [m.givre_courbe_moyenne(0), m.givre_courbe_erreur_type(0)], texte: texteAvancement(lireCourbe(m)) };
}

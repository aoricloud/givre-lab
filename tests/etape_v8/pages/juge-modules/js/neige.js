// La neige : le démarrage d'après le module, les masses case par case.
const lecture = await import("../lecture.js");

export function jeux(m) {
  return lecture.lireJeuxNeige(m).map((jeu) => lecture.texteDuJeuNeige(jeu));
}

export function demarrer(m, jeu, quatre) {
  const code = m.givre_neige_demarrer(jeu, quatre.rho, quatre.beta, quatre.kappa, quatre.mu);
  return code === 0 ? { source: lecture.texteSourceNeige(lecture.lireReglagesNeige(m), false) } : { refus: lecture.texteRefusNeige(code) };
}

export function masses(m) {
  const nombre = m.givre_neige_masses_preparer() >>> 0;
  return Uint32Array.from({ length: nombre }, (_, i) => m.givre_neige_masse(i) >>> 0);
}

export function textes(m) {
  const mesures = lecture.lireMesuresNeige(m);
  return mesures === null ? null : { forme: lecture.texteDeLaFormeNeige(mesures.forme), mesures: lecture.texteMesuresNeige(mesures), fin: lecture.texteFinNeige(mesures), cache: lecture.blocsNeige(null), rho: lecture.formaterReglage(0) };
}

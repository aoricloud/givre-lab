//! Les aperçus : des images pour l'œil, jamais une preuve (étape 0, puis chaque étape). Une image est une suite d'octets
//! RVB, ligne par ligne de haut en bas, trois octets par pixel (R, V, B) : c'est ce que prend `image::ecrire_png`.

/// L'aperçu de « la graine seule » : une image RVB de `largeur × hauteur` pixels, donc `3 * largeur * hauteur` octets. On
/// prend `Alea::new(graine)` (flux 54) ; pour chaque pixel, **ligne par ligne** (de haut en bas, et dans chaque ligne de
/// gauche à droite), on tire un seul `suivant()` (32 bits) et `v = (tirage >> 24) as u8` (ses 8 bits de poids fort) ; le
/// pixel vaut `(v, v, v)`. Pour la graine 42, les trois premiers pixels sont donc `(0xA1, 0xA1, 0xA1)`, `(0x7B, 0x7B,
/// 0x7B)`, `(0xBA, 0xBA, 0xBA)` (les premiers tirages officiels sont `0xa15c02b7`, `0x7b47f409`, `0xba1d3330`).
///
/// Une image vide (`largeur` ou `hauteur` nulle) est la suite vide.
#[allow(unused_variables)]
pub fn apercu_graine(largeur: u32, hauteur: u32, graine: u64) -> Vec<u8> {
    todo!()
}

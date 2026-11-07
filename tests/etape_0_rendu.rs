//! Étape 0 : `rendu::apercu_graine`, l'image de « la graine seule ». Valeurs attendues :
//! `references/etape-0-valeurs.md`, identifiants `V-REND-xx`.
//!
//! Sources : le vecteur officiel de PCG32 pour la graine 42 et le flux 54 (V-ALEA-01) ; et un modèle Python de `pcg_basic.c`
//! (V-ALEA-04) pour les autres graines : chaque pixel est `v = (tirage >> 24)` répété trois fois.

use givre_lab::rendu::apercu_graine;

/// Les 40 premiers `v = tirage >> 24` de la graine 42, flux 54 (modèle Python ; les six premiers tirages sont ceux du
/// vecteur officiel V-ALEA-01 : `0xA1`, `0x7B`, `0xBA`, `0x83`, `0xBF`, `0xCB`).
const V_42: [u8; 40] = [
    161, 123, 186, 131, 191, 203, 191, 129, 230, 249, 50, 29, 237, 56, 43, 28, 162, 132, 206, 178, 145, 85, 71, 72,
    177, 194, 26, 121, 95, 30, 133, 41, 109, 195, 160, 228, 226, 64, 142, 227,
];

fn triples(valeurs: &[u8]) -> Vec<u8> {
    valeurs.iter().flat_map(|&v| [v, v, v]).collect()
}

/// juste : les premiers pixels de la graine 42 sont `(0xA1,0xA1,0xA1)`, `(0x7B,0x7B,0x7B)`, `(0xBA,0xBA,0xBA)` : les 8 bits de
/// poids fort des tirages officiels `0xa15c02b7`, `0x7b47f409`, `0xba1d3330` (V-REND-01, V-ALEA-01) ; ce n'est pas l'octet de
/// poids faible (`0xB7`).
///
/// Ne couvre pas : les images de plus de trois pixels (voir `apercu_8_sur_5`).
#[test]
#[cfg_attr(not(feature = "etape-0"), ignore)]
fn premiers_pixels_officiels() {
    let image = apercu_graine(3, 1, 42);
    assert_eq!(image, [0xA1, 0xA1, 0xA1, 0x7B, 0x7B, 0x7B, 0xBA, 0xBA, 0xBA]);
}

/// juste : une image de 8 × 5 pixels de graine 42 est les 40 premiers tirages (V-REND-02), ligne par ligne de gauche à droite
/// et de haut en bas, chacun répété en `(v, v, v)` ; une image de 5 × 8 donne les mêmes octets (la même suite de tirages
/// rangée en lignes) ; un rangement par colonnes les distinguerait.
///
/// Ne couvre pas : les images plus grandes.
#[test]
#[cfg_attr(not(feature = "etape-0"), ignore)]
fn apercu_8_sur_5() {
    assert_eq!(apercu_graine(8, 5, 42), triples(&V_42));
    assert_eq!(apercu_graine(5, 8, 42), triples(&V_42));
}

/// juste : une image de 3 × 2 de graine 7 (`164, 108, 25, 183, 185, 125`) et la longueur `3 · largeur · hauteur` pour
/// plusieurs tailles ; une image de largeur ou de hauteur nulle est vide ; deux graines donnent deux images (V-REND-03).
///
/// Ne couvre pas : le contenu des images de plus de 6 pixels pour d'autres graines que 42.
#[test]
#[cfg_attr(not(feature = "etape-0"), ignore)]
fn apercu_3_sur_2_longueurs_et_graines() {
    assert_eq!(apercu_graine(3, 2, 7), triples(&[164, 108, 25, 183, 185, 125]));
    for (l, h) in [(1_u32, 1_u32), (7, 3), (64, 48), (100, 1)] {
        assert_eq!(apercu_graine(l, h, 1).len(), (3 * l * h) as usize);
    }
    assert!(apercu_graine(0, 5, 1).is_empty());
    assert!(apercu_graine(5, 0, 1).is_empty());
    assert_ne!(apercu_graine(8, 5, 1), apercu_graine(8, 5, 2));
    assert_eq!(apercu_graine(3, 2, 1)[..6], [155, 155, 155, 12, 12, 12]);
}

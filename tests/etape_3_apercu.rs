//! Étape 3 : l'aperçu (`Flocon::apercu`), l'image RVB carrée de côté `2R + 1`, une case par pixel, la case `(q, r)` au pixel
//! `(q + R + floor(r / 2), r + R)`. Valeurs attendues : `references/etape-3-valeurs.md`, identifiants `V-APER-xx`.

mod etape_3_commun;

use etape_3_commun::valeurs::{APERCU_R2, P1_AVANT};
use etape_3_commun::{cases, masses_formule, parametres};
use givre_lab::flocon::Flocon;
use givre_lab::math::Fixe;

/// La couleur attendue d'une masse, d'après la doc : gelée `(235, 245, 255)`, sinon `(0, niveau / 2, niveau)` avec
/// `niveau = (brut * 255) >> 32`.
fn couleur(masse: Fixe) -> [u8; 3] {
    if masse.0 >= Fixe::UN.0 {
        [235, 245, 255]
    } else {
        let niveau = ((i128::from(masse.0) * 255) >> 32) as u8;
        [0, niveau / 2, niveau]
    }
}

/// juste : l'aperçu de `P1_AVANT` (`R = 2`, une masse par case, des cases gelées et une à `1 - 2^-32`) est exactement les 75 octets calculés par Python
/// d'après la doc (`APERCU_R2`, V-APER-01) : 25 pixels RVB, ligne par ligne de haut en bas ; les 6 pixels qui ne sont l'image d'aucune case
/// (les coins du carré) sont noirs. Une correspondance case-pixel décalée, un `floor` qui arrondit vers zéro au lieu de vers le bas, un niveau de
/// couleur mal arrondi échouent ici.
///
/// Ne couvre pas : les autres rayons (voir `correspondance_case_pixel`).
#[test]
#[cfg_attr(not(feature = "etape-3"), ignore)]
fn apercu_python_r2() {
    let masses: Vec<Fixe> = etape_3_commun::valeurs::P1_AVANT.iter().map(|&b| Fixe(b)).collect();
    let f = Flocon::depuis_masses(2, parametres("0.7", "0.3", "0.05"), &masses);
    assert_eq!(f.apercu(), APERCU_R2.to_vec());
    assert_eq!(P1_AVANT.len(), 19);
}

/// juste : la correspondance case-pixel de la doc, recalculée par le test pour `R` de 1 à 6 (V-APER-02) : l'image a `3 (2R + 1)^2` octets ; la case
/// `(q, r)` est le pixel `(x, y) = (q + R + floor(r / 2), r + R)` (`floor` vers le bas : `r = -1` donne `-1`) et porte la couleur de sa masse ;
/// aucun pixel n'est l'image de deux cases ; tous les autres pixels sont `(0, 0, 0)`. Les masses sont variées : 0, `UN - 1`, `UN`, `5/2`, `1/4`,
/// `1/2` et des valeurs de formule.
///
/// Ne couvre pas : le contenu exact des couleurs de niveau (voir `couleurs_de_niveau`).
#[test]
#[cfg_attr(not(feature = "etape-3"), ignore)]
fn correspondance_case_pixel() {
    let p = parametres("1", "0.3", "0.01");
    for rayon in 1_i32..=6 {
        let toutes = cases(rayon);
        let mut masses = masses_formule(rayon, 37, 11, 61, 61, &[]);
        let speciales = [Fixe::ZERO, Fixe(Fixe::UN.0 - 1), Fixe::UN, Fixe::rapport(5, 2), Fixe(1 << 30), Fixe(1 << 31)];
        for (k, v) in speciales.iter().enumerate() {
            masses[(3 * k + 1) % toutes.len()] = *v;
        }
        let f = Flocon::depuis_masses(rayon as u32, p, &masses);
        let image = f.apercu();
        let cote = (2 * rayon + 1) as usize;
        assert_eq!(image.len(), 3 * cote * cote, "R={rayon}");
        let mut attendu = vec![0_u8; image.len()];
        let mut vus = std::collections::BTreeSet::new();
        for (i, &(q, r)) in toutes.iter().enumerate() {
            let x = (q + rayon + r.div_euclid(2)) as usize;
            let y = (r + rayon) as usize;
            assert!(vus.insert((x, y)), "R={rayon} : deux cases sur le pixel ({x}, {y})");
            attendu[3 * (y * cote + x)..3 * (y * cote + x) + 3].copy_from_slice(&couleur(masses[i]));
        }
        assert_eq!(image, attendu, "R={rayon}");
    }
}

/// juste : les couleurs de niveau, calculées à la main (V-APER-03), sur `R = 1` avec le centre et les six voisins de masses `0`, `1/4` (`niveau = 63`
/// → `(0, 31, 63)`), `1/2` (`127` → `(0, 63, 127)`), `3/4` (`191` → `(0, 95, 191)`), `UN - 1` (`254` → `(0, 127, 254)`), `UN` (gelée) et `5/2` (gelée).
/// La masse 1 est gelée (`>=`) ; `UN - 1` ne l'est pas.
///
/// Ne couvre pas : la position des pixels.
#[test]
#[cfg_attr(not(feature = "etape-3"), ignore)]
fn couleurs_de_niveau() {
    let p = parametres("1", "0.3", "0.01");
    let valeurs = [
        (Fixe::ZERO, [0, 0, 0]),
        (Fixe(1 << 30), [0, 31, 63]),
        (Fixe(1 << 31), [0, 63, 127]),
        (Fixe(3 << 30), [0, 95, 191]),
        (Fixe(Fixe::UN.0 - 1), [0, 127, 254]),
        (Fixe::UN, [235, 245, 255]),
        (Fixe::rapport(5, 2), [235, 245, 255]),
    ];
    let toutes = cases(1);
    for (masse, attendu) in valeurs {
        let masses = vec![masse; toutes.len()];
        let image = Flocon::depuis_masses(1, p, &masses).apercu();
        // le pixel (1, 1) est la case (0, 0) : (0 + 1 + 0, 0 + 1)
        let o = 3 * (3 + 1);
        assert_eq!(&image[o..o + 3], &attendu, "masse {masse:?}");
        assert_eq!(couleur(masse), attendu);
    }
}

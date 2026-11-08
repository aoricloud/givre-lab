//! Étape 3c : le BORD du flocon (`Bord::Hexagonal` : la couronne ; `Bord::Circulaire` : `4 · distance2 >= 3 R²`) et `distance2`. Valeurs
//! attendues : `references/etape-3-valeurs.md`, identifiants `V-BORD-xx` ; les nombres et les listes de cases viennent de l'outil indépendant
//! `modele_independant_3c.py` (Python, entiers), jamais de la bibliothèque (`tests/etape_3c_commun/valeurs.rs`).
//!
//! Les fautes que ces tests voient : un bord toujours égal à la couronne ; un bord circulaire de rayon `R` au lieu de `R sin 60°` (`4 d² >= 4 R²`
//! ou `d² >= R²`) ; `>` au lieu de `>=` (l'égalité exacte `4 · distance2 = 3 R²` arrive pour `R` pair au milieu des côtés, et pour `R = 14`, 26,
//! 28, 38 et d'autres jusqu'à 98 aussi à l'intérieur) ; `q² - q r + r²` à la place de `q² + q r + r²` ; un arrondi de racine mal fait (une racine
//! en `f64` bien faite donne le même bord sur les rayons testés : la doc et `AGENTS.md` l'interdisent, ces tests ne le voient pas) ; un bord qui
//! dépend du sens (une symétrie qui ne le garde pas).

mod etape_3_commun;
mod etape_3c_commun;

use etape_3_commun::{cases, distance, parametres, symetries};
use etape_3c_commun::valeurs::{
    BORD_CIRCULAIRE_R3, BORD_CIRCULAIRE_R5, BORD_CIRCULAIRE_R8, BORD_CIRCULAIRE_R10, BORD_CIRCULAIRE_R14,
    EGALITES_INTERIEURES_R14, NOMBRES_DU_BORD, R100_CASES_DU_BORD, R100_PLUS_GRAND_DISTANCE2_HORS_DU_BORD,
    R100_PLUS_PETIT_DISTANCE2_DU_BORD,
};
use givre_lab::flocon::{Bord, Flocon, distance2};

fn flocon(rayon: u32, bord: Bord) -> Flocon {
    Flocon::avec_bord(rayon, parametres("1", "0.4", "0.01"), bord)
}

/// Les cases du bord de `f`, dans l'ordre canonique, d'après `est_du_bord` (sur les cases de l'hexagone seulement).
fn cases_du_bord(f: &Flocon) -> Vec<(i32, i32)> {
    cases(f.rayon() as i32).into_iter().filter(|&(q, r)| f.est_du_bord(q, r)).collect()
}

/// juste : `distance2(q, r) = q² + q r + r²` (V-BORD-08), le carré de la distance euclidienne au centre quand deux cases voisines sont à 1
/// (`x = q + r/2`, `y = r sin 60°`, `x² + y² = q² + q r + r²`). Des valeurs calculées à la main : le centre 0 ; les six voisins du centre 1
/// (`(1, 0)`, `(0, 1)`, `(-1, 1)`, `(-1, 0)`, `(0, -1)`, `(1, -1)`) ; `(1, 1)` 3 ; `(2, -1)` 3 ; `(2, 0)` 4 ; `(3, 4)` 37 (9 + 12 + 16) ;
/// `(3, -4)` 13 (9 - 12 + 16) ; `(5, 3)` 49 (25 + 15 + 9) ; `(-5, -3)` 49 ; `(49, 49)` 7203 (`3 · 49²`) ; `(7, 0)` 49 contre `(4, 4)` 48 ; `(100, 0)`
/// 10000. Puis, sur tous les `(q, r)` de -30 à 30, une AUTRE écriture du même entier : `4 · distance2 = (2q + r)² + 3 r²` (les carrés des
/// coordonnées `2x` et `r√3`). Et les cas extrêmes : aucun débordement pour tout `i32` (`distance2(i32::MIN, i32::MIN) = 3 · 2^62`,
/// `distance2(i32::MAX, i32::MAX) = 3 · (2^31 - 1)²`, `distance2(i32::MIN, i32::MAX)` calculé en entiers 128 bits). Un `q² - q r + r²`, un
/// calcul sur 32 bits ou un flottant échouent ici.
///
/// Ne couvre pas : l'invariance par les symétries (voir `distance2_invariante_par_les_symetries`).
#[test]
#[cfg_attr(not(feature = "etape-3-bord"), ignore)]
fn distance2_valeurs_exactes() {
    assert_eq!(distance2(0, 0), 0);
    for (q, r) in [(1, 0), (0, 1), (-1, 1), (-1, 0), (0, -1), (1, -1)] {
        assert_eq!(distance2(q, r), 1, "voisin ({q}, {r})");
    }
    let a_la_main = [
        ((1, 1), 3),
        ((2, -1), 3),
        ((2, 0), 4),
        ((3, 4), 37),
        ((3, -4), 13),
        ((5, 3), 49),
        ((-5, -3), 49),
        ((49, 49), 7203),
        ((7, 0), 49),
        ((4, 4), 48),
        ((100, 0), 10_000),
        ((-7, 7), 49),
        ((0, -7), 49),
    ];
    for ((q, r), attendu) in a_la_main {
        assert_eq!(distance2(q, r), attendu, "({q}, {r})");
    }
    for q in -30..=30_i32 {
        for r in -30..=30_i32 {
            let quatre = i64::from(2 * q + r).pow(2) + 3 * i64::from(r).pow(2);
            assert_eq!(4 * distance2(q, r) as i64, quatre, "({q}, {r})");
        }
    }
    assert_eq!(distance2(i32::MIN, i32::MIN), 3 * (1_u64 << 62));
    let max = u64::from(i32::MAX.unsigned_abs());
    assert_eq!(distance2(i32::MAX, i32::MAX), 3 * max * max);
    let (q, r) = (i128::from(i32::MIN), i128::from(i32::MAX));
    assert_eq!(i128::from(distance2(i32::MIN, i32::MAX)), q * q + q * r + r * r);
    assert_eq!(distance2(i32::MIN, 0), 1_u64 << 62);
}

/// juste : `distance2` est invariante par les douze symétries de l'hexagone (V-BORD-09), celles de `etape_3_commun::symetries` (écrites en
/// toutes lettres) : pour `q` et `r` de -25 à 25, `distance2(g(q, r)) = distance2(q, r)`. Un `q² - q r + r²` (qui n'est invariant que par
/// quelques-unes) ou un `q² + r²` y échouent.
///
/// Ne couvre pas : les valeurs exactes (voir `distance2_valeurs_exactes` ; ici seulement l'invariance, `distance2(2q, 2r) = 4 distance2(q, r)` et que
/// `distance2` n'est nulle qu'au centre).
#[test]
#[cfg_attr(not(feature = "etape-3-bord"), ignore)]
fn distance2_invariante_par_les_symetries() {
    for (nom, g) in symetries() {
        for q in -25..=25 {
            for r in -25..=25 {
                let (a, b) = g(q, r);
                assert_eq!(distance2(a, b), distance2(q, r), "{nom} ({q}, {r})");
                assert!((q, r) == (0, 0) || distance2(q, r) > 0, "{nom} ({q}, {r}) : seul le centre est à 0");
                assert_eq!(
                    distance2(2 * q, 2 * r),
                    4 * distance2(q, r),
                    "({q}, {r}) : distance2 double avec la distance au carré ×4"
                );
            }
        }
    }
}

/// juste : le NOMBRE de cases du bord (V-BORD-01), pour chaque `R` de 1 à 100, avec chacun des deux bords, d'après `est_du_bord` sur
/// les cases de l'hexagone : la couronne a `6 R` cases ; le bord circulaire a le nombre compté par Python en entiers (`4 (q² + q r + r²) >=
/// 3 R²`) : 6 R pour `R` de 1 à 7 (le bord circulaire est la couronne), puis 54, 60, 66, 84, 90, 96, 114, ... pour `R` = 8, 9, 10, 11, 12,
/// 13, 14... Les rayons 8 à 14 voient les premières cases de plus ; 14, 26, 28, 38 (et onze autres jusqu'à 98) ont des égalités exactes à l'intérieur. Un bord
/// circulaire qui est toujours la couronne, un cercle de rayon `R` (`d² >= R²`, bien moins de cases), un `>` pour `>=` (les 6 milieux de côté
/// de `R` pair en moins, et pour `R = 14` 12 de plus), un `<=` ou un décalage d'un cran échouent ici.
///
/// Ne couvre pas : QUELLES cases (voir `liste_complete_du_bord_circulaire`).
#[test]
#[cfg_attr(not(feature = "etape-3-bord"), ignore)]
fn nombre_de_cases_du_bord() {
    for &(rayon, couronne, circulaire) in NOMBRES_DU_BORD {
        assert_eq!(couronne, 6 * rayon as usize, "la couronne a 6 R cases (R = {rayon})");
        let hexagonal = flocon(rayon, Bord::Hexagonal);
        let rond = flocon(rayon, Bord::Circulaire);
        assert_eq!(cases_du_bord(&hexagonal).len(), couronne, "R = {rayon}, bord hexagonal");
        assert_eq!(cases_du_bord(&rond).len(), circulaire, "R = {rayon}, bord circulaire");
    }
}

/// juste : la LISTE complète des cases du bord circulaire (V-BORD-02), pour `R` = 3, 5, 8, 10 et 14, dans l'ordre canonique, comparée à celle
/// de l'outil indépendant (Python, entiers). `R` = 3 et 5 : c'est la couronne (le bord circulaire n'ajoute rien avant `R = 8`) ; `R` = 8 : la
/// couronne (48 cases) et les 6 coins `(7, 0)`, `(0, 7)`, `(-7, 7)`, `(-7, 0)`, `(0, -7)`, `(7, -7)` de la rangée 7 (`4 · 49 = 196 >= 3 · 64 =
/// 192`, alors que `(7, -1)` : `4 · 43 = 172` est dehors) ; `R` = 10 : la couronne et les 6 coins `(9, 0)`... ; `R` = 14 : 114 cases dont 12 à
/// l'égalité exacte `4 · distance2 = 3 R² = 588` à l'intérieur (voir `egalite_exacte_dans_le_bord`). Une case en trop ou en moins, par un
/// seuil décalé, un autre cercle ou une autre formule, échoue ici.
///
/// Ne couvre pas : les rayons autres que ces cinq (voir `nombre_de_cases_du_bord` pour leur nombre).
#[test]
#[cfg_attr(not(feature = "etape-3-bord"), ignore)]
fn liste_complete_du_bord_circulaire() {
    let listes: [(u32, &[(i32, i32)]); 5] = [
        (3, BORD_CIRCULAIRE_R3),
        (5, BORD_CIRCULAIRE_R5),
        (8, BORD_CIRCULAIRE_R8),
        (10, BORD_CIRCULAIRE_R10),
        (14, BORD_CIRCULAIRE_R14),
    ];
    for (rayon, attendu) in listes {
        let f = flocon(rayon, Bord::Circulaire);
        assert_eq!(cases_du_bord(&f), attendu, "R = {rayon}");
    }
    // R = 8 : les six coins de la rangée 7
    assert_eq!(BORD_CIRCULAIRE_R8.len(), 54);
    let coins = [(7, 0), (0, 7), (-7, 7), (-7, 0), (0, -7), (7, -7)];
    let f = flocon(8, Bord::Circulaire);
    for (q, r) in coins {
        assert!(f.est_du_bord(q, r), "coin ({q}, {r})");
    }
    assert!(!f.est_du_bord(7, -1), "(7, -1) : 4 · 43 = 172 < 192");
    assert!(!f.est_du_bord(6, 0));
    assert!(!flocon(8, Bord::Hexagonal).est_du_bord(7, 0), "bord hexagonal : (7, 0) est à la distance 7, pas 8");
}

/// juste : la couronne est dans le bord circulaire (V-BORD-10), pour `R` de 1 à 40 : toute case à distance hexagonale exactement `R` est du bord
/// circulaire, et le bord hexagonal est EXACTEMENT la couronne (aucune case à distance `< R` n'en est). Pour `R <= 7` les deux bords sont le
/// même ensemble ; pour `R` de 8 à 40 le bord circulaire a strictement plus de cases, et chaque case en plus a `4 · distance2 >= 3 R²` (recalculé
/// en `i64` par le test). Un bord circulaire qui ne contient pas toute la couronne (un cercle de rayon `R sin 60°` trop petit pris au sens
/// strict, `>` à la place de `>=`) ou un bord hexagonal qui prendrait aussi des cases intérieures échouent ici.
///
/// Ne couvre pas : les listes précises (voir `liste_complete_du_bord_circulaire`).
#[test]
#[cfg_attr(not(feature = "etape-3-bord"), ignore)]
fn la_couronne_est_dans_le_bord_circulaire() {
    for rayon in 1..=40_u32 {
        let hexagonal = flocon(rayon, Bord::Hexagonal);
        let rond = flocon(rayon, Bord::Circulaire);
        let mut en_plus = 0;
        for (q, r) in cases(rayon as i32) {
            let sur_la_couronne = distance(q, r) == rayon as i32;
            assert_eq!(
                hexagonal.est_du_bord(q, r),
                sur_la_couronne,
                "R = {rayon}, ({q}, {r}) : bord hexagonal = couronne"
            );
            if sur_la_couronne {
                assert!(rond.est_du_bord(q, r), "R = {rayon}, ({q}, {r}) : la couronne est du bord circulaire");
            } else if rond.est_du_bord(q, r) {
                en_plus += 1;
                let (q, r) = (i64::from(q), i64::from(r));
                let carre = q * q + q * r + r * r;
                assert!(4 * carre >= 3 * i64::from(rayon).pow(2), "R = {rayon}, ({q}, {r}) : seulement hors du cercle");
            }
        }
        if rayon <= 7 {
            assert_eq!(en_plus, 0, "R = {rayon} : le bord circulaire est la couronne");
        } else {
            assert!(en_plus > 0, "R = {rayon} : le bord circulaire a des cases de plus");
        }
    }
}

/// juste : la comparaison est LARGE (`>=`) : pour `R` pair, le milieu `(R/2, R/2)` d'un côté de l'hexagone a `4 · distance2 = 3 R²` exactement
/// et il est du bord circulaire (V-BORD-03 et V-BORD-10). C'est vrai pour `R` = 2, 4, ..., 40. Et pour `R = 14`, 12 cases à distance
/// hexagonale `< 14` ont aussi `4 · distance2 = 3 · 14² = 588` exactement (`147 = 3 · 7²`, l'outil indépendant les liste) : elles sont du bord
/// circulaire ; une comparaison stricte les en exclurait (elle changerait aussi les 6 milieux de côté, et ferait diffuser des cases de la
/// couronne, dont les voisins sont hors de l'hexagone). Et pour toutes les cases de `R = 14` dont `distance2` est de 140 à 150, la règle est
/// `distance2 >= 147` (`4 · distance2 >= 588`) : une case juste en dessous reste hors du bord.
///
/// Ne couvre pas : les rayons impairs (l'égalité y est impossible : `3 R²` est impair, `4 · distance2` est pair).
#[test]
#[cfg_attr(not(feature = "etape-3-bord"), ignore)]
fn egalite_exacte_dans_le_bord() {
    for rayon in (2..=40_u32).step_by(2) {
        let moitie = (rayon / 2) as i32;
        assert_eq!(4 * distance2(moitie, moitie), 3 * u64::from(rayon).pow(2), "milieu de côté, R = {rayon}");
        assert!(flocon(rayon, Bord::Circulaire).est_du_bord(moitie, moitie), "R = {rayon}");
        assert!(flocon(rayon, Bord::Hexagonal).est_du_bord(moitie, moitie), "R = {rayon} (couronne)");
    }
    let f = flocon(14, Bord::Circulaire);
    for &(q, r) in EGALITES_INTERIEURES_R14 {
        assert_eq!(4 * distance2(q, r), 3 * 14 * 14, "({q}, {r})");
        assert!(distance(q, r) < 14, "({q}, {r}) est à l'intérieur de la couronne");
        assert!(f.est_du_bord(q, r), "({q}, {r}) : égalité exacte, bord circulaire");
        assert!(!flocon(14, Bord::Hexagonal).est_du_bord(q, r), "({q}, {r}) : pas du bord hexagonal");
    }
    for (q, r) in cases(14) {
        let d2 = distance2(q, r);
        if (140..=150).contains(&d2) {
            assert_eq!(f.est_du_bord(q, r), d2 >= 147, "({q}, {r}) : distance2 = {d2}");
        }
    }
}

/// juste : le bord est stable par les douze symétries de l'hexagone (V-BORD-11), pour chacun des deux bords et `R` de 1 à 30 : une case est du
/// bord si et seulement si son image par chaque symétrie l'est (les rotations de 60° et les réflexions, écrites dans `etape_3_commun::symetries`) ;
/// et le bord n'est ni vide ni plein (au moins les `6 R` cases de la couronne, jamais le centre : un `est_du_bord` toujours faux ou toujours vrai serait
/// « stable » ; la couronne seule a exactement `6 R` cases). Un bord défini par
/// `q² - q r + r²` ou par `q² + r²` n'est pas stable et échoue ici.
///
/// Ne couvre pas : l'effet du bord sur l'évolution (voir `etape_3c_symetrie`).
#[test]
#[cfg_attr(not(feature = "etape-3-bord"), ignore)]
fn le_bord_est_stable_par_les_symetries() {
    for rayon in 1..=30_u32 {
        for bord in [Bord::Hexagonal, Bord::Circulaire] {
            let f = flocon(rayon, bord);
            for (nom, g) in symetries() {
                for (q, r) in cases(rayon as i32) {
                    let (a, b) = g(q, r);
                    assert_eq!(f.est_du_bord(q, r), f.est_du_bord(a, b), "R = {rayon} {bord:?} {nom} ({q}, {r})");
                }
            }
            // le bord n'est ni vide ni plein : il contient la couronne (6 R cases), jamais le centre, et pas toutes les cases pour R >= 2
            let compte = cases_du_bord(&f).len();
            assert!(compte >= 6 * rayon as usize && !f.est_du_bord(0, 0), "R = {rayon} {bord:?}");
            assert!(compte < cases(rayon as i32).len(), "R = {rayon} {bord:?} : le bord n'est pas tout l'hexagone");
            if bord == Bord::Hexagonal {
                assert_eq!(compte, 6 * rayon as usize, "R = {rayon} : la couronne a 6 R cases");
            }
        }
    }
}

/// juste : `est_du_bord` hors de l'hexagone est faux, sans panique (V-BORD-12), pour les deux bords : les cases à distance hexagonale `R + 1`
/// et `R + 2` (les plus proches de l'hexagone), des cases lointaines, et les valeurs extrêmes d'`i32` (où un calcul sur 32 bits
/// déborderait). Le centre n'est jamais du bord. `bord()` rend le bord choisi, et ne change pas après des pas. Une case hors de l'hexagone dont
/// `distance2` est grand (donc `>= 3 R² / 4`) est quand même fausse : un `est_du_bord` qui ne regarde que `distance2` échoue ici.
///
/// Ne couvre pas : les cases dans l'hexagone (voir les autres tests de ce fichier).
#[test]
#[cfg_attr(not(feature = "etape-3-bord"), ignore)]
fn hors_de_l_hexagone_et_centre() {
    for rayon in [1_u32, 2, 3, 8, 9, 14, 30] {
        for bord in [Bord::Hexagonal, Bord::Circulaire] {
            let mut f = flocon(rayon, bord);
            assert_eq!(f.bord(), bord);
            assert!(!f.est_du_bord(0, 0), "le centre, R = {rayon} {bord:?}");
            let r = rayon as i32;
            let dehors = [
                (r + 1, 0),
                (0, r + 1),
                (-r - 1, 0),
                (0, -r - 1),
                (r + 1, -r - 1),
                (-r - 1, r + 1),
                (r + 2, 0),
                (2 * r, 0),
                (r, r),
                (1000, 1000),
                (-1000, 5),
                (i32::MAX, 0),
                (0, i32::MIN),
                (i32::MIN, i32::MIN),
                (i32::MAX, i32::MAX),
                (i32::MIN, i32::MAX),
            ];
            for (q, rr) in dehors {
                assert!(distance_i64(q, rr) > i64::from(r), "le test place bien ({q}, {rr}) dehors");
                assert!(!f.est_du_bord(q, rr), "R = {rayon} {bord:?} : ({q}, {rr}) est hors de l'hexagone");
            }
            f.avancer(3);
            assert_eq!(f.bord(), bord, "le bord ne change pas");
        }
    }
}

/// juste : `R = 100`, le cas de la table de Reiter (V-BORD-07) : le bord circulaire est exactement `distance2 >= 7500` (`4 · distance2 >= 3 ·
/// 100²`), le `ocir` du script `sfn.ijs` de l'auteur (`d² >= (100 sin 60°)² = 7500`). Le nombre de cases du bord (3126 d'après Python), le plus
/// petit `distance2` d'une case du bord (7500) et le plus grand d'une case hors du bord (7492) sont ceux de l'outil indépendant, et le test
/// recompte à part chaque case par `distance2 >= 7500`. La couronne (600 cases) en est une partie.
///
/// Ne couvre pas : la dynamique (voir `etape_3c_pas`, `etape_3c_table`).
#[test]
#[cfg_attr(not(feature = "etape-3-bord"), ignore)]
fn bord_circulaire_de_rayon_100() {
    let f = flocon(100, Bord::Circulaire);
    let (mut nombre, mut plus_petit, mut plus_grand_dehors) = (0, u64::MAX, 0);
    for (q, r) in cases(100) {
        let d2 = distance2(q, r);
        assert_eq!(f.est_du_bord(q, r), d2 >= 7500, "({q}, {r}) : distance2 = {d2}");
        if f.est_du_bord(q, r) {
            nombre += 1;
            plus_petit = plus_petit.min(d2);
        } else {
            plus_grand_dehors = plus_grand_dehors.max(d2);
        }
    }
    assert_eq!(nombre, R100_CASES_DU_BORD);
    assert_eq!(plus_petit, R100_PLUS_PETIT_DISTANCE2_DU_BORD);
    assert_eq!(plus_grand_dehors, R100_PLUS_GRAND_DISTANCE2_HORS_DU_BORD);
    assert_eq!((plus_petit, plus_grand_dehors), (7500, 7492));
    assert_eq!(couronne_du_bord(&f), 600);
}

/// La distance hexagonale en `i64` (aucun débordement, même pour les valeurs extrêmes d'`i32`).
fn distance_i64(q: i32, r: i32) -> i64 {
    let (q, r) = (i64::from(q), i64::from(r));
    q.abs().max(r.abs()).max((q + r).abs())
}

/// Le nombre de cases à distance hexagonale `R` qui sont du bord (la couronne entière).
fn couronne_du_bord(f: &Flocon) -> usize {
    let rayon = f.rayon() as i32;
    cases(rayon).into_iter().filter(|&(q, r)| distance(q, r) == rayon && f.est_du_bord(q, r)).count()
}

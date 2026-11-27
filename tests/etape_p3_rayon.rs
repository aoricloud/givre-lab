//! Étape P3 : le rayon de la pointe. La parabole des moindres carrés (`givre_lab::pointe::parabole`) et la règle des lignes
//! (`givre_lab::pointe::mesurer_pointe`), sur des points et des champs faits à la main dont la réponse est exacte ou calculée hors de
//! Rust. Valeurs attendues : `references/etape-p3-valeurs.md`, identifiants `V-P3-PARAB-xx` et `V-P3-RAYON-xx`.

mod etape_p3_commun;

use etape_p3_commun::*;
use givre_lab::dendrite::Domaine;
use givre_lab::pointe::{ErreurPointe as E, mesurer_pointe, parabole};

/// juste : (V-P3-PARAB-01) la parabole de la doc à la main. (a) Les huit points `(y, 10 - y² / 8)` pour `y = 0,25 ; 0,75 ; … ; 3,75`
/// (dyadiques : chaque opération de la doc est exacte) : `(sommet, rayon) = (10, 4)` EXACTEMENT ; (b) les quatre points
/// `(0, -2), (1, -3), (2, -6), (3, -11)` (`x = -2 - y²`) : `(-2 ; 0,5)` exactement (un sommet négatif n'est pas une erreur) ; (c) l'ordre
/// des points ne change rien sur (a) (les mêmes points à l'envers : `(10, 4)`, exact) ; (d) trois points pris n'importe où sur la
/// parabole de (a) (`y = 0,25 ; 2,25 ; 3,75`) : `(10, 4)` à `1e-12` près. (e) L'ordre des opérations : sur 300 jeux tirés (3 à 12
/// points, `y` dans `[0 ; 6]`, `x = 20 - y² / (2 ρ) + bruit`, `ρ` de 1 à 40, bruit de ± 0,05), le résultat est, bit pour bit, celui de
/// `parabole_naive` (la doc écrite dans les tests), et la formule des sommes brutes (`(m Σ xY - ΣY Σx) / (m ΣY² - (ΣY)²)`, la même en
/// réels) donne d'autres bits pour au moins 30 jeux.
///
/// Ne couvre pas : des points tirés d'un contour (voir `rayon_sur_une_parabole_exacte`) ; la sensibilité au bruit.
#[test]
#[cfg_attr(not(feature = "etape-p3"), ignore)]
fn parabole_a_la_main() {
    let a: Vec<(f64, f64)> = (0..8)
        .map(|i| {
            let y = (i as f64 + 0.5) * 0.5;
            (y, 10.0 - (y * y) / 8.0)
        })
        .collect();
    assert_eq!(parabole(&a), Ok((10.0, 4.0)));
    assert_eq!(parabole(&[(0.0, -2.0), (1.0, -3.0), (2.0, -6.0), (3.0, -11.0)]), Ok((-2.0, 0.5)));
    let envers: Vec<(f64, f64)> = a.iter().rev().copied().collect();
    assert_eq!(parabole(&envers), Ok((10.0, 4.0)));
    let (s, r) = parabole(&[a[0], a[4], a[7]]).unwrap();
    assert!((s - 10.0).abs() < 1e-12 && (r - 4.0).abs() < 1e-12, "trois points : ({s}, {r})");
    // l'ordre des opérations
    let mut g = Lcg(2_718);
    let (mut jeux, mut brutes_different) = (0, 0);
    for _ in 0..300 {
        let m = 3 + (g.suivant() * 10.0) as usize;
        let rho = g.entre(1.0, 40.0);
        let pts: Vec<(f64, f64)> = (0..m)
            .map(|_| {
                let y = g.entre(0.0, 6.0);
                (y, 20.0 - (y * y) / (2.0 * rho) + g.entre(-0.05, 0.05))
            })
            .collect();
        let lib = parabole(&pts);
        let naif = parabole_naive(&pts, None);
        assert_eq!(lib.map(|(s, r)| (s.to_bits(), r.to_bits())), naif.clone().map(|(s, r)| (s.to_bits(), r.to_bits())));
        if let (Ok(x), Ok(y)) = (naif, parabole_naive(&pts, Some(Faute::SommesBrutes))) {
            jeux += 1;
            brutes_different += usize::from(x.0.to_bits() != y.0.to_bits() || x.1.to_bits() != y.1.to_bits());
        }
    }
    assert!(jeux >= 200 && brutes_different >= 30, "{jeux} jeux, {brutes_different} où les sommes brutes diffèrent");
}

/// juste : (V-P3-PARAB-02) les refus de `parabole`, dans l'ordre de la doc : (1) une coordonnée non finie (NaN, `+inf`, `-inf`, en `y`
/// comme en `x`) : `ParametreInvalide("points")`, même quand il y a moins de trois points ; (2) moins de trois points :
/// `TropPeuDeLignes(m)` pour `m` = 0, 1, 2 ; (3) `PasDeCourbure` : une droite (`x` constant : `b = 0`), une parabole ouverte vers l'avant
/// (`x = y²` : `b = 1`, le rayon serait `-0,5`), trois `y` égaux (`syy = 0` : `b` est NaN), des `y` dont le carré déborde (`1e200` : les
/// sommes sont infinies, puis NaN). Aucun ne panique.
///
/// Ne couvre pas : les points d'un contour (voir `mesure_refus_et_ordre`).
#[test]
#[cfg_attr(not(feature = "etape-p3"), ignore)]
fn parabole_refus_et_ordre() {
    for mauvais in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        assert_eq!(parabole(&[(mauvais, 1.0)]), Err(E::ParametreInvalide("points")));
        assert_eq!(parabole(&[(0.0, 1.0), (1.0, mauvais)]), Err(E::ParametreInvalide("points")));
        assert_eq!(
            parabole(&[(0.0, 1.0), (1.0, 0.0), (2.0, -3.0), (mauvais, 0.0)]),
            Err(E::ParametreInvalide("points"))
        );
    }
    assert_eq!(parabole(&[]), Err(E::TropPeuDeLignes(0)));
    assert_eq!(parabole(&[(0.0, 1.0)]), Err(E::TropPeuDeLignes(1)));
    assert_eq!(parabole(&[(0.0, 1.0), (1.0, 0.5)]), Err(E::TropPeuDeLignes(2)));
    assert_eq!(parabole(&[(0.5, 3.0), (1.5, 3.0), (2.5, 3.0), (3.5, 3.0)]), Err(E::PasDeCourbure));
    assert_eq!(parabole(&[(0.0, 0.0), (1.0, 1.0), (2.0, 4.0)]), Err(E::PasDeCourbure));
    assert_eq!(parabole(&[(1.0, 0.0), (1.0, 1.0), (1.0, 2.0)]), Err(E::PasDeCourbure));
    assert_eq!(parabole(&[(1e200, 0.0), (2e200, 1.0), (3e200, 2.0)]), Err(E::PasDeCourbure));
}

/// juste : (V-P3-RAYON-01) la règle des lignes sur la parabole exacte `x = 10 - y² / 8` (le champ de `contour_a_la_main`, quart de
/// 24 × 24 au pas 0,5) : la profondeur de la ligne `i` est `x0 - x_i = (i² + i) / 32` exactement (0 ; 0,0625 ; 0,1875 ; 0,375 ; 0,625 ;
/// 0,9375 ; 1,3125 ; 1,75 ; 2,25 …). (a) `profondeur = 1,75` : la ligne 7 est à la profondeur EXACTE (la règle est large) : 8 lignes,
/// `(sommet, rayon) = (10, 4)` exactement, `position = 9,9921875` (la ligne 0, celle de `position_pointe`), `temps = 0` ; (b) 1,7 : 7
/// lignes ; 2,25 : 9 lignes ; 100 : les 18 lignes qui croisent (la ligne 18 n'a pas de croisement : la liste s'arrête), et `(10, 4)` à
/// `1e-12` près dans les trois cas ; (c) le même champ au pas 0,25 (`x = 10 - y² / 8` aux centres `(i + 0,5) · 0,25`, 48 × 48) :
/// `(10, 4)` à `1e-12` près, et 11 lignes à `profondeur = 1` (la profondeur est une longueur, pas un nombre de lignes : au pas 0,5 la
/// même profondeur retient 6 lignes). Le `y` d'une ligne est le CENTRE de ses cases : avec `y = i dx` (le bord), le rayon n'est plus 4 (la
/// faute `YAuBord`, voir `etape_p3_fautes.rs`).
///
/// Ne couvre pas : une forme qui n'est pas une parabole (voir `rayon_sur_un_cercle`).
#[test]
#[cfg_attr(not(feature = "etape-p3"), ignore)]
fn rayon_sur_une_parabole_exacte() {
    let n = 24;
    let d = dendrite_de_champ(&champ_par_lignes(n, 0.5, &xs_parabole_exacte(n), 2.0), n, 0.5, Domaine::Quart);
    let p = mesurer_pointe(&d, 1.75).unwrap();
    assert_eq!((p.lignes, p.sommet, p.rayon, p.position, p.temps), (8, 10.0, 4.0, 9.9921875, 0.0));
    for (h, lignes) in [(1.7, 7), (2.25, 9), (100.0, 18)] {
        let p = mesurer_pointe(&d, h).unwrap();
        assert_eq!(p.lignes, lignes, "profondeur {h}");
        assert!((p.sommet - 10.0).abs() < 1e-12 && (p.rayon - 4.0).abs() < 1e-12, "profondeur {h} : {p:?}");
    }
    assert_eq!(mesurer_pointe(&d, 1.0).unwrap().lignes, 6);
    // au pas 0,25
    let n = 48;
    let xs: Vec<f64> = (0..n)
        .map(|i| {
            let y = (i as f64 + 0.5) * 0.25;
            let x = 10.0 - (y * y) / 8.0;
            if x > 0.125 { x } else { f64::NAN }
        })
        .collect();
    let d = dendrite_de_champ(&champ_par_lignes(n, 0.25, &xs, 2.0), n, 0.25, Domaine::Quart);
    let p = mesurer_pointe(&d, 1.0).unwrap();
    assert_eq!(p.lignes, 11);
    assert!((p.sommet - 10.0).abs() < 1e-12 && (p.rayon - 4.0).abs() < 1e-12, "{p:?}");
}

/// juste : (V-P3-RAYON-02) la parabole ajustée à un CERCLE de rayon 8 (centré à l'origine : `x_i = sqrt(64 - y_i²)` aux centres
/// `y_i = (i + 0,5) · 0,5` des lignes, quart de 24 × 24 au pas 0,5, rampes de demi-largeur 2) : le rayon ajusté n'est pas celui du
/// cercle (le cercle tombe plus vite que sa parabole osculatrice), il dépend de la profondeur, et il est celui que donne `cercle.py`
/// (les moindres carrés de la doc et la règle des lignes, en décimal à 60 chiffres, hors de Rust) : profondeur 1 : 8 lignes, sommet
/// 8,005 963 261 271 136, rayon 7,555 419 403 737 679 ; profondeur 2 : 11 lignes, 8,025 548 311 556 944, 7,100 258 720 076 413 ;
/// profondeur 4 : 14 lignes, 8,090 097 127 426 222, 6,370 238 610 025 476 ; à `1e-10` près en relatif (la racine et l'interpolation des
/// rampes arrondissent ; observé : moins de `1e-14`). La position est `sqrt(64 - 0,0625)` = 7,996 092 795 859 738.
///
/// Ne couvre pas : un champ né d'un calcul (voir `mesure_egale_au_naif_bit_pour_bit`).
#[test]
#[cfg_attr(not(feature = "etape-p3"), ignore)]
fn rayon_sur_un_cercle() {
    let n = 24;
    let xs: Vec<f64> = (0..n)
        .map(|i| {
            let y = (i as f64 + 0.5) * 0.5;
            if y < 8.0 { (64.0 - y * y).sqrt() } else { f64::NAN }
        })
        .collect();
    let d = dendrite_de_champ(&champ_par_lignes(n, 0.5, &xs, 2.0), n, 0.5, Domaine::Quart);
    let proche = |a: f64, b: f64| ((a - b) / b).abs() < 1e-10;
    for (h, lignes, sommet, rayon) in [
        (1.0, 8, 8.005_963_261_271_136, 7.555_419_403_737_679),
        (2.0, 11, 8.025_548_311_556_944, 7.100_258_720_076_413),
        (4.0, 14, 8.090_097_127_426_222, 6.370_238_610_025_476),
    ] {
        let p = mesurer_pointe(&d, h).unwrap();
        assert_eq!(p.lignes, lignes, "profondeur {h}");
        assert!(proche(p.sommet, sommet) && proche(p.rayon, rayon), "profondeur {h} : {p:?}");
        assert!(proche(p.position, 7.996_092_795_859_738), "{p:?}");
    }
}

/// juste : (V-P3-RAYON-03) la règle des lignes : CONSÉCUTIVES depuis l'axe, la première ligne qui manque arrête la liste (le champ
/// de la parabole exacte, 24 × 24 au pas 0,5, profondeur 1,75, où 8 lignes sont retenues) : (a) la ligne 2 sans croisement (`φ = -1`
/// partout) : 2 lignes retenues, donc `TropPeuDeLignes(2)` (et non 7 : les lignes 3 à 7, qui satisfont la règle, ne sont pas
/// regardées) ; (b) la ligne 3 enfoncée de 5 (`x_3 - 5`, plus profonde que 1,75) : 3 lignes, `(10, 4)` à `1e-12` près ; (c) la ligne 5
/// AU-DELÀ de la pointe (`x_5 = 11` : sa profondeur `-1,007 812 5` est négative, donc `<= 1,75`) : retenue, 8 lignes, et le résultat est
/// celui de `parabole` sur les huit points `(y_i, x_i)` du champ (le calcul de la doc, bit pour bit) ; (d) la profondeur est comptée
/// depuis la ligne 0 (la position), pas depuis le sommet ajusté : sur (a) sans la ligne 2 enlevée, `profondeur = 0,0625` retient les
/// lignes 0 et 1 (`TropPeuDeLignes(2)`) et `0,1875` les lignes 0 à 2 (`(10, 4)` à `1e-12`).
///
/// Ne couvre pas : une dendrite réelle (voir `mesure_egale_au_naif_bit_pour_bit`).
#[test]
#[cfg_attr(not(feature = "etape-p3"), ignore)]
fn la_regle_des_lignes() {
    let n = 24;
    let base = xs_parabole_exacte(n);
    let mesure = |xs: &[f64], h: f64| {
        mesurer_pointe(&dendrite_de_champ(&champ_par_lignes(n, 0.5, xs, 2.0), n, 0.5, Domaine::Quart), h)
    };
    let mut a = base.clone();
    a[2] = f64::NAN;
    assert_eq!(mesure(&a, 1.75), Err(E::TropPeuDeLignes(2)));
    let mut b = base.clone();
    b[3] -= 5.0;
    let p = mesure(&b, 1.75).unwrap();
    assert_eq!(p.lignes, 3);
    assert!((p.sommet - 10.0).abs() < 1e-12 && (p.rayon - 4.0).abs() < 1e-12, "{p:?}");
    let mut c = base.clone();
    c[5] = 11.0;
    let p = mesure(&c, 1.75).unwrap();
    assert_eq!(p.lignes, 8);
    let points: Vec<(f64, f64)> = (0..8).map(|i| ((i as f64 + 0.5) * 0.5, c[i])).collect();
    let (s, r) = parabole(&points).unwrap();
    assert_eq!((p.sommet.to_bits(), p.rayon.to_bits()), (s.to_bits(), r.to_bits()));
    assert_eq!(mesure(&base, 0.0625), Err(E::TropPeuDeLignes(2)));
    let p = mesure(&base, 0.1875).unwrap();
    assert_eq!(p.lignes, 3);
    assert!((p.sommet - 10.0).abs() < 1e-12 && (p.rayon - 4.0).abs() < 1e-12, "{p:?}");
}

/// juste : (V-P3-RAYON-04) les refus de `mesurer_pointe`, dans l'ordre de la doc : (1) `profondeur` nulle, négative, NaN ou infinie :
/// `ParametreInvalide("profondeur")`, même sur un domaine entier et sur un champ sans pointe ; (2) un domaine entier : `DomaineEntier`,
/// même quand la ligne 0 n'a pas de croisement ; (3) la ligne 0 sans croisement (les autres lignes croisent) : `PasDePointe` ;
/// (4) moins de trois lignes : `TropPeuDeLignes(1)` (`profondeur = 0,01` sur la parabole exacte : la ligne 1 est à 0,0625) et
/// `TropPeuDeLignes(2)` ; (5) `PasDeCourbure` : un front droit (`x = 7,25` partout : toutes les lignes à la profondeur 0, `b = 0`) et
/// une parabole ouverte vers l'avant (`x = 5 + y² / 8` : les profondeurs sont négatives, `b > 0`). Aucun ne panique.
///
/// Ne couvre pas : les refus de `parabole` qu'une mesure ne peut pas atteindre (`ParametreInvalide("points")` : un contour est fini).
#[test]
#[cfg_attr(not(feature = "etape-p3"), ignore)]
fn mesure_refus_et_ordre() {
    let n = 24;
    let parabole_exacte =
        dendrite_de_champ(&champ_par_lignes(n, 0.5, &xs_parabole_exacte(n), 2.0), n, 0.5, Domaine::Quart);
    let entier = dendrite_de_champ(&champ_par_lignes(n, 0.5, &xs_parabole_exacte(n), 2.0), n, 0.5, Domaine::Entier);
    let mut sans_pointe = xs_parabole_exacte(n);
    sans_pointe[0] = f64::NAN;
    let sans_pointe = dendrite_de_champ(&champ_par_lignes(n, 0.5, &sans_pointe, 2.0), n, 0.5, Domaine::Quart);
    let entier_sans_pointe = dendrite_de_champ(&vec![-1.0; n * n], n, 0.5, Domaine::Entier);
    for h in [0.0, -1.0, f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        for d in [&parabole_exacte, &entier, &sans_pointe, &entier_sans_pointe] {
            assert_eq!(mesurer_pointe(d, h), Err(E::ParametreInvalide("profondeur")), "profondeur {h}");
        }
    }
    assert_eq!(mesurer_pointe(&entier, 2.0), Err(E::DomaineEntier));
    assert_eq!(mesurer_pointe(&entier_sans_pointe, 2.0), Err(E::DomaineEntier));
    assert_eq!(mesurer_pointe(&sans_pointe, 2.0), Err(E::PasDePointe));
    assert_eq!(mesurer_pointe(&parabole_exacte, 0.01), Err(E::TropPeuDeLignes(1)));
    assert_eq!(mesurer_pointe(&parabole_exacte, 0.1), Err(E::TropPeuDeLignes(2)));
    let droit = dendrite_de_champ(&champ_par_lignes(16, 0.5, &[7.25; 16], 2.0), 16, 0.5, Domaine::Quart);
    assert_eq!(mesurer_pointe(&droit, 1.0), Err(E::PasDeCourbure));
    let avant: Vec<f64> = (0..n)
        .map(|i| {
            let y = (i as f64 + 0.5) * 0.5;
            let x = 5.0 + (y * y) / 8.0;
            if x < 11.5 { x } else { f64::NAN }
        })
        .collect();
    let d = dendrite_de_champ(&champ_par_lignes(n, 0.5, &avant, 2.0), n, 0.5, Domaine::Quart);
    assert_eq!(mesurer_pointe(&d, 1.0), Err(E::PasDeCourbure));
}

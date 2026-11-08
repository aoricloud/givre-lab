//! Étape 3c : `Flocon::distance2_cristal` (la mesure d'arrêt de l'auteur : le plus grand `distance2` d'une case gelée). Valeurs attendues :
//! `references/etape-3-valeurs.md`, identifiants `V-BORD-xx` ; les `distance2` à la main sont recomptés par l'outil indépendant
//! (`modele_independant_3c.py`).
//!
//! Les fautes que ces tests voient : le rayon hexagonal à la place de `distance2` ; le plus petit à la place du plus grand ; le centre ou une case
//! non gelée compté ; `>` à la place de `>=` pour le gel (une masse exactement égale à 1 est gelée, `UN - 1` ne l'est pas) ; un `distance2` qui
//! dépend du bord ; une case du bord gelée oubliée.

mod etape_3_commun;
mod etape_3c_commun;

use etape_3_commun::{cases, masses_formule, parametres, symetries};
use givre_lab::flocon::{Bord, Flocon, distance2};
use givre_lab::math::Fixe;

/// Un flocon de rayon `rayon` posé à la main : toutes les cases valent `fond`, sauf `posees` (case, masse), au temps 0.
fn pose(rayon: u32, bord: Bord, fond: Fixe, posees: &[((i32, i32), Fixe)]) -> Flocon {
    let toutes = cases(rayon as i32);
    let mut masses = vec![fond; toutes.len()];
    for &(cellule, masse) in posees {
        masses[toutes.iter().position(|&c| c == cellule).expect("la case est dans l'hexagone")] = masse;
    }
    Flocon::avec_bord_et_masses(rayon, parametres("1", "0.4", "0.01"), bord, &masses)
}

/// juste : `distance2_cristal` sur des configurations posées à la main (V-BORD-22), `R = 10`, les deux bords. (a) aucune case gelée (toutes les masses
/// à 0,3) : 0. (b) seule la case centrale gelée : 0 (c'est aussi l'état initial de `new`). (c) la seule case gelée `(3, 4)` : 37 (`9 + 12 + 16`).
/// (d) `(7, 0)` et `(4, 4)` gelées : `distance2` 49 et 48, donc 49, alors que `rayon_cristal()` vaut 8 (celui de `(4, 4)`) : le plus grand `distance2` n'est
/// pas la plus grande distance hexagonale. (e) `(5, 0)` à `UN - 1` (non gelée) et `(2, 0)` à `UN` (gelée, la masse égale à 1 est gelée) : 4. (f) `(5, 0)` à
/// `UN + 1` : 25. (g) `(-3, -1)` et `(2, -5)` : `distance2` 13 et 19, donc 19 (des coordonnées négatives ; la plus grande n'est pas la dernière de l'ordre canonique). Un `distance2`
/// qui prend le centre, la plus petite valeur, la dernière case gelée de l'énumération (au lieu de la plus grande) ou la distance hexagonale échoue ici.
///
/// Ne couvre pas : une case du bord gelée (voir `case_du_bord_gelee`), les grands rayons (voir `seuil_de_l_auteur_a_rayon_100`).
#[test]
#[cfg_attr(not(feature = "etape-3-bord"), ignore)]
fn distance2_cristal_a_la_main() {
    let (un, demi) = (Fixe::UN, Fixe::rapport(3, 10));
    let un_moins = Fixe(Fixe::UN.0 - 1);
    let un_plus = Fixe(Fixe::UN.0 + 1);
    for bord in [Bord::Circulaire, Bord::Hexagonal] {
        let f = pose(10, bord, demi, &[]);
        assert_eq!((f.gelees(), f.distance2_cristal()), (0, 0), "(a) {bord:?}");
        let f = pose(10, bord, demi, &[((0, 0), un)]);
        assert_eq!((f.gelees(), f.distance2_cristal()), (1, 0), "(b) {bord:?}");
        let f = pose(10, bord, demi, &[((3, 4), un)]);
        assert_eq!(f.distance2_cristal(), 37, "(c) {bord:?}");
        let f = pose(10, bord, demi, &[((7, 0), un), ((4, 4), un)]);
        assert_eq!((f.rayon_cristal(), f.distance2_cristal()), (8, 49), "(d) {bord:?}");
        let f = pose(10, bord, demi, &[((5, 0), un_moins), ((2, 0), un)]);
        assert_eq!((f.gelees(), f.distance2_cristal()), (1, 4), "(e) {bord:?}");
        let f = pose(10, bord, demi, &[((5, 0), un_plus), ((2, 0), un)]);
        assert_eq!((f.gelees(), f.distance2_cristal()), (2, 25), "(f) {bord:?}");
        let f = pose(10, bord, demi, &[((-3, -1), un), ((2, -5), un)]);
        assert_eq!(f.distance2_cristal(), 19, "(g) {bord:?}");
        assert_eq!(distance2(-3, -1), 13);
        assert_eq!(distance2(2, -5), 19);
    }
}

/// juste : une case du BORD qui est gelée compte (V-BORD-23) : `R = 9`, bord circulaire, la case `(-8, 0)` (une des 6 cases en plus de la couronne : `4 · 64
/// = 256 >= 243`, `distance2` 64) gelée : `distance2_cristal() = 64` ; une case de la couronne `(0, -9)` gelée : 81. Avec le bord hexagonal, les
/// mêmes configurations donnent les mêmes valeurs (`distance2_cristal` ne dépend pas du bord). Après UN pas, les cases du bord sont remises à `β < 1` : plus
/// aucune n'est gelée (la case `(-8, 0)` n'est plus gelée avec le bord circulaire, car elle est du bord). Une mesure qui ignore le bord, ou qui en dépend, échoue ici.
///
/// Ne couvre pas : les cases à l'intérieur (voir `distance2_cristal_a_la_main`).
#[test]
#[cfg_attr(not(feature = "etape-3-bord"), ignore)]
fn case_du_bord_gelee() {
    let (un, fond) = (Fixe::UN, Fixe::rapport(3, 10));
    for bord in [Bord::Circulaire, Bord::Hexagonal] {
        let f = pose(9, bord, fond, &[((-8, 0), un)]);
        assert_eq!((f.gelees(), f.distance2_cristal()), (1, 64), "{bord:?} : (-8, 0)");
        let f = pose(9, bord, fond, &[((0, -9), un)]);
        assert_eq!((f.gelees(), f.distance2_cristal()), (1, 81), "{bord:?} : (0, -9)");
    }
    let mut f = pose(9, Bord::Circulaire, fond, &[((-8, 0), un), ((0, -9), un)]);
    assert_eq!((f.gelees(), f.distance2_cristal()), (2, 81));
    f.pas();
    assert_eq!((f.gelees(), f.distance2_cristal()), (0, 0), "les cases du bord sont remises à β < 1");
    let mut f = pose(9, Bord::Hexagonal, fond, &[((-8, 0), un), ((0, -9), un)]);
    f.pas();
    assert_eq!(
        f.distance2_cristal(),
        64,
        "(-8, 0) n'est pas du bord hexagonal : elle reste gelée (elle reçoit γ) et la couronne est remise à β"
    );
}

/// juste : `distance2_cristal` ne dépend d'aucune symétrie (V-BORD-24) : une case gelée et ses onze images par les douze symétries de l'hexagone
/// (écrites dans `etape_3_commun::symetries`) donnent toutes le même `distance2_cristal`, pour trois cases `(3, 4)`, `(-5, 2)` et `(7, 0)` sur `R = 10`,
/// et ce nombre est le `distance2` recalculé par le test. Un calcul qui regarde `q` seul, `r` seul ou `q + r` seul échoue ici.
///
/// Ne couvre pas : les valeurs précises (voir `distance2_cristal_a_la_main`).
#[test]
#[cfg_attr(not(feature = "etape-3-bord"), ignore)]
fn distance2_cristal_invariant_par_les_symetries() {
    let (un, fond) = (Fixe::UN, Fixe::rapport(3, 10));
    for (q, r) in [(3, 4), (-5, 2), (7, 0)] {
        let attendu = i64::from(q) * i64::from(q) + i64::from(q) * i64::from(r) + i64::from(r) * i64::from(r);
        for (nom, g) in symetries() {
            let image = g(q, r);
            let f = pose(10, Bord::Circulaire, fond, &[(image, un)]);
            assert_eq!(f.distance2_cristal(), attendu as u64, "({q}, {r}) {nom} -> {image:?}");
        }
    }
}

/// juste : pendant une croissance, `distance2_cristal()` est à chaque pas le plus grand `distance2` des cases dont la masse lue par `masses()` est au
/// moins 1 (V-BORD-25) : recalculé par le test sur toutes les cases, 120 pas, `R = 30`, bord circulaire et hexagonal, deux jeux (une plaque `α = 1, β = 0,
/// γ = 0.2` et une dendrite `α = 1, β = 0.4, γ = 0.001`). Il ne décroît jamais (une case gelée garde sa masse et reçoit `γ` et la diffusion, qui ne la
/// diminuent pas) et il grandit au moins une fois. Une mesure en retard d'un pas, ou qui lit un tampon, échoue ici.
///
/// Ne couvre pas : les valeurs de Python (voir `trajectoires_python_a_bord_circulaire`).
#[test]
#[cfg_attr(not(feature = "etape-3-bord"), ignore)]
fn distance2_cristal_suit_la_croissance() {
    for (a, b, g) in [("1", "0", "0.2"), ("1", "0.4", "0.001")] {
        for bord in [Bord::Circulaire, Bord::Hexagonal] {
            let mut f = Flocon::avec_bord(30, parametres(a, b, g), bord);
            let (mut precedent, mut grandi) = (0, false);
            for t in 1..=120 {
                f.pas();
                let masses = f.masses();
                // le `distance2` recalculé ici, sans la bibliothèque : q² + q r + r²
                let attendu = cases(30)
                    .iter()
                    .zip(&masses)
                    .filter(|(_, m)| m.0 >= Fixe::UN.0)
                    .map(|(&(q, r), _)| {
                        (i64::from(q) * i64::from(q) + i64::from(q) * i64::from(r) + i64::from(r) * i64::from(r)) as u64
                    })
                    .max()
                    .unwrap_or(0);
                assert_eq!(f.distance2_cristal(), attendu, "{bord:?} alpha {a} beta {b} gamma {g} pas {t}");
                assert!(attendu >= precedent, "{bord:?} pas {t} : distance2_cristal ne décroît pas");
                grandi |= attendu > precedent;
                precedent = attendu;
            }
            assert!(grandi && precedent > 0, "le cristal a grandi");
        }
    }
}

/// juste : le seuil d'arrêt de l'auteur à `R = 100` (V-BORD-26) : l'arrêt a lieu dès que `distance2_cristal() >= 7203`, c'est-à-dire `4 ·
/// distance2 >= 3 · 98²` (`98 = R - 2`). Des cases de `R = 100` choisies par l'outil indépendant : `(31, -96)` a `distance2` 7201 (strictement
/// en dessous), `(49, -98)` et `(49, 49)` ont exactement 7203 (`3 · 49²`), `(48, -98)` a 7204 ; sur chacune posée gelée, `distance2_cristal` vaut cette
/// valeur et la comparaison `>= 7203` donne faux, vrai, vrai, vrai. Il n'y a aucune case de `distance2` 7202 (`7202 = 2 · 13 · 277` et `2` n'a pas
/// de facteur parmi les normes `q² + q r + r²`, l'outil indépendant le confirme sur l'hexagone de rayon 100), donc le seuil se joue entre 7201 et 7203. Un seuil `>` à la place de `>=` ou un `distance2` décalé d'une unité échouent ici.
///
/// Ne couvre pas : la croissance jusqu'au seuil (voir `etape_3c_table`).
#[test]
#[cfg_attr(not(feature = "etape-3-bord"), ignore)]
fn seuil_de_l_auteur_a_rayon_100() {
    let (un, fond) = (Fixe::UN, Fixe::rapport(3, 10));
    let attendu =
        [((31, -96), 7201_u64, false), ((49, -98), 7203, true), ((49, 49), 7203, true), ((48, -98), 7204, true)];
    for (cellule, d2, arret) in attendu {
        let f = pose(100, Bord::Circulaire, fond, &[(cellule, un)]);
        assert_eq!(f.distance2_cristal(), d2, "{cellule:?}");
        assert_eq!(f.distance2_cristal() >= 7203, arret, "{cellule:?}");
        assert_eq!(4 * f.distance2_cristal() >= 3 * 98 * 98, arret, "{cellule:?} : 4 · distance2 >= 3 · (R - 2)²");
    }
    // une formule de masses sans case gelée : 0
    let masses = masses_formule(100, 37, 11, 97, 120, &[]);
    assert_eq!(
        Flocon::avec_bord_et_masses(100, parametres("1", "0.3", "0.01"), Bord::Circulaire, &masses).distance2_cristal(),
        0
    );
}

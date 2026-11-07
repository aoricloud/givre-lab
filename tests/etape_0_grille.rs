//! Étape 0 : `reseau`, la grille carrée, la géométrie du disque et l'enroulement d'une bande. Valeurs attendues :
//! `references/etape-0-valeurs.md`, identifiants `V-GRILLE-xx`.
//!
//! Ici la « source » est la définition écrite dans la documentation de `reseau` (une convention, pas une valeur
//! publiée : l'ordre des voisins, `y * largeur + x`) ou un calcul exact (la définition `dx² + dy² <= r²` réécrite dans le
//! test, et le nombre de points entiers d'un disque, suite A000328 de l'OEIS).

use givre_lab::reseau::{Grille, VOISINS4, VOISINS8, dans_disque};

/// juste : les déplacements des voisins à 4 et à 8 sont, dans cet ordre, ceux de la documentation : droite, bas, gauche,
/// haut ; droite, bas-droite, bas, bas-gauche, gauche, haut-gauche, haut, haut-droite (V-GRILLE-01).
///
/// Ne couvre pas : l'usage des constantes par `voisins4` et `voisins8` (voir les tests de voisinage).
#[test]
#[cfg_attr(not(feature = "etape-0"), ignore)]
fn ordre_des_deplacements() {
    assert_eq!(VOISINS4, [(1, 0), (0, 1), (-1, 0), (0, -1)]);
    assert_eq!(VOISINS8, [(1, 0), (1, 1), (0, 1), (-1, 1), (-1, 0), (-1, -1), (0, -1), (1, -1)]);
}

/// juste : les dimensions et le nombre de cases d'une grille de 5 colonnes et 4 lignes, d'une grille de 1 case, d'une
/// grille d'une colonne (V-GRILLE-02).
///
/// Ne couvre pas : les grandes grilles.
#[test]
#[cfg_attr(not(feature = "etape-0"), ignore)]
fn dimensions() {
    let g = Grille::new(5, 4);
    assert_eq!((g.largeur(), g.hauteur(), g.cases()), (5, 4, 20));
    let g = Grille::new(1, 1);
    assert_eq!((g.largeur(), g.hauteur(), g.cases()), (1, 1, 1));
    let g = Grille::new(1, 7);
    assert_eq!((g.largeur(), g.hauteur(), g.cases()), (1, 7, 7));
    assert_eq!(Grille::new(4096, 4096).cases(), 16_777_216);
}

/// juste : l'indice par lignes `y * largeur + x` sur une grille de 5 × 4 (non carrée : une confusion entre `x` et `y`, ou
/// entre largeur et hauteur, change des indices) ; `position` est son inverse ; quelques valeurs écrites à la main
/// (V-GRILLE-03).
///
/// Ne couvre pas : les grilles très grandes.
#[test]
#[cfg_attr(not(feature = "etape-0"), ignore)]
fn index_par_lignes_et_position() {
    let g = Grille::new(5, 4);
    assert_eq!(g.index(0, 0), 0);
    assert_eq!(g.index(4, 0), 4);
    assert_eq!(g.index(0, 1), 5);
    assert_eq!(g.index(2, 3), 17);
    assert_eq!(g.index(4, 3), 19);
    assert_eq!(g.position(17), (2, 3));
    assert_eq!(g.position(5), (0, 1));
    for y in 0..4 {
        for x in 0..5 {
            assert_eq!(g.index(x, y), y * 5 + x);
            assert_eq!(g.position(g.index(x, y)), (x, y));
        }
    }
    let etroite = Grille::new(2, 9);
    assert_eq!(etroite.index(1, 8), 17);
    assert_eq!(etroite.position(17), (1, 8));
}

/// juste : `index` panique pour une abscisse égale à la largeur (qui, sans contrôle, tomberait sur la ligne suivante) et
/// pour une ordonnée égale à la hauteur ; `position` panique pour `cases()` ; `new` panique pour une dimension nulle ou
/// un produit qui déborde (V-GRILLE-04, la doc dit « panique »).
///
/// Ne couvre pas : le message des paniques.
#[test]
#[cfg_attr(not(feature = "etape-0"), ignore)]
fn panique_hors_grille() {
    use std::panic::catch_unwind;
    let g = Grille::new(5, 4);
    assert!(catch_unwind(|| g.index(5, 0)).is_err(), "index(largeur, 0)");
    assert!(catch_unwind(|| g.index(0, 4)).is_err(), "index(0, hauteur)");
    assert!(catch_unwind(|| g.position(20)).is_err(), "position(cases)");
    assert!(catch_unwind(|| Grille::new(0, 3)).is_err(), "largeur nulle");
    assert!(catch_unwind(|| Grille::new(3, 0)).is_err(), "hauteur nulle");
    assert!(catch_unwind(|| Grille::new(usize::MAX, 2)).is_err(), "produit qui déborde");
    assert!(catch_unwind(|| g.voisins4(5, 0)).is_err(), "voisins4 hors grille");
    assert!(catch_unwind(|| g.voisins8(0, 4)).is_err(), "voisins8 hors grille");
}

/// juste : `contient` est vrai exactement pour `0 <= x < largeur` et `0 <= y < hauteur`, y compris pour des coordonnées
/// négatives et énormes (V-GRILLE-05).
///
/// Ne couvre pas : rien de plus que la définition.
#[test]
#[cfg_attr(not(feature = "etape-0"), ignore)]
fn contient_les_bords() {
    let g = Grille::new(5, 4);
    for (x, y, attendu) in [
        (0, 0, true),
        (4, 3, true),
        (2, 1, true),
        (5, 3, false),
        (4, 4, false),
        (-1, 0, false),
        (0, -1, false),
        (-1, -1, false),
        (5, 4, false),
        (isize::MAX, 0, false),
        (0, isize::MAX, false),
        (isize::MIN, 0, false),
        (0, isize::MIN, false),
    ] {
        assert_eq!(g.contient(x, y), attendu, "contient({x}, {y})");
    }
}

/// juste : les voisins à 4 sur une grille de 5 × 4, à l'intérieur, sur chaque bord et dans chaque coin, dans l'ordre
/// droite, bas, gauche, haut ; une case hors grille donne `None` à sa place (V-GRILLE-06). Cas dégénérés : la grille d'une
/// case, la grille d'une colonne.
///
/// Ne couvre pas : l'absence d'allocation (la signature rend un tableau).
#[test]
#[cfg_attr(not(feature = "etape-0"), ignore)]
fn voisins4_litteraux() {
    let g = Grille::new(5, 4);
    assert_eq!(g.voisins4(2, 1), [Some((3, 1)), Some((2, 2)), Some((1, 1)), Some((2, 0))]);
    assert_eq!(g.voisins4(0, 0), [Some((1, 0)), Some((0, 1)), None, None]);
    assert_eq!(g.voisins4(4, 3), [None, None, Some((3, 3)), Some((4, 2))]);
    assert_eq!(g.voisins4(4, 0), [None, Some((4, 1)), Some((3, 0)), None]);
    assert_eq!(g.voisins4(0, 3), [Some((1, 3)), None, None, Some((0, 2))]);
    assert_eq!(g.voisins4(4, 1), [None, Some((4, 2)), Some((3, 1)), Some((4, 0))]);
    assert_eq!(g.voisins4(2, 0), [Some((3, 0)), Some((2, 1)), Some((1, 0)), None]);
    assert_eq!(g.voisins4(2, 3), [Some((3, 3)), None, Some((1, 3)), Some((2, 2))]);
    assert_eq!(g.voisins4(0, 2), [Some((1, 2)), Some((0, 3)), None, Some((0, 1))]);
    assert_eq!(Grille::new(1, 1).voisins4(0, 0), [None, None, None, None]);
    assert_eq!(Grille::new(1, 3).voisins4(0, 1), [None, Some((0, 2)), None, Some((0, 0))]);
}

/// juste : les voisins à 8 sur une grille de 5 × 4, à l'intérieur, sur des bords et dans des coins, dans l'ordre droite,
/// bas-droite, bas, bas-gauche, gauche, haut-gauche, haut, haut-droite (V-GRILLE-06).
///
/// Ne couvre pas : l'absence d'allocation.
#[test]
#[cfg_attr(not(feature = "etape-0"), ignore)]
fn voisins8_litteraux() {
    let g = Grille::new(5, 4);
    assert_eq!(
        g.voisins8(2, 1),
        [
            Some((3, 1)),
            Some((3, 2)),
            Some((2, 2)),
            Some((1, 2)),
            Some((1, 1)),
            Some((1, 0)),
            Some((2, 0)),
            Some((3, 0))
        ]
    );
    assert_eq!(g.voisins8(0, 0), [Some((1, 0)), Some((1, 1)), Some((0, 1)), None, None, None, None, None]);
    assert_eq!(g.voisins8(4, 3), [None, None, None, None, Some((3, 3)), Some((3, 2)), Some((4, 2)), None]);
    assert_eq!(
        g.voisins8(4, 1),
        [None, None, Some((4, 2)), Some((3, 2)), Some((3, 1)), Some((3, 0)), Some((4, 0)), None]
    );
    assert_eq!(
        g.voisins8(2, 0),
        [Some((3, 0)), Some((3, 1)), Some((2, 1)), Some((1, 1)), Some((1, 0)), None, None, None]
    );
    assert_eq!(Grille::new(1, 1).voisins8(0, 0), [None; 8]);
    assert_eq!(Grille::new(1, 3).voisins8(0, 1), [None, None, Some((0, 2)), None, None, None, Some((0, 0)), None]);
}

/// juste : sur chaque case d'une grille de 7 × 6, les voisins coïncident avec un dénombrement naïf écrit ici (on essaie
/// chacun des déplacements de la doc et on garde ceux dont l'arrivée est dans la grille, à leur rang) ; la relation est
/// symétrique (`b` voisin de `a` ssi `a` voisin de `b`) ; les voisins à 4 sont les rangs pairs des voisins à 8 (V-GRILLE-06).
///
/// Ne couvre pas : les grilles d'autres tailles que 7 × 6.
#[test]
#[cfg_attr(not(feature = "etape-0"), ignore)]
fn voisinages_contre_denombrement_naif() {
    let g = Grille::new(7, 6);
    let dedans = |x: i64, y: i64| (0..7).contains(&x) && (0..6).contains(&y);
    for y in 0..6_usize {
        for x in 0..7_usize {
            let v4 = g.voisins4(x, y);
            let v8 = g.voisins8(x, y);
            for (k, &(dx, dy)) in VOISINS4.iter().enumerate() {
                let (nx, ny) = (x as i64 + dx as i64, y as i64 + dy as i64);
                let attendu = dedans(nx, ny).then_some((nx as usize, ny as usize));
                assert_eq!(v4[k], attendu, "voisins4({x}, {y})[{k}]");
                assert_eq!(v8[2 * k], attendu, "voisins8({x}, {y})[{}]", 2 * k);
            }
            for (k, &(dx, dy)) in VOISINS8.iter().enumerate() {
                let (nx, ny) = (x as i64 + dx as i64, y as i64 + dy as i64);
                let attendu = dedans(nx, ny).then_some((nx as usize, ny as usize));
                assert_eq!(v8[k], attendu, "voisins8({x}, {y})[{k}]");
            }
            for voisin in v8.into_iter().flatten() {
                assert!(g.voisins8(voisin.0, voisin.1).contains(&Some((x, y))), "symétrie de ({x}, {y}) et {voisin:?}");
            }
        }
    }
}

/// juste : `enrouler_x` est le reste euclidien par la largeur : `-1` donne `largeur - 1`, `largeur` donne 0, `-largeur`
/// donne 0 ; comparé à `((x % l) + l) % l` pour tout `x` de -50 à 50 et plusieurs largeurs, et sur de grandes valeurs
/// (V-GRILLE-07). Un `%` à la rust (reste de signe négatif) échoue sur les négatifs.
///
/// Ne couvre pas : les valeurs au-delà de 2^31 en valeur absolue (la doc les exclut).
#[test]
#[cfg_attr(not(feature = "etape-0"), ignore)]
fn enrouler_x_reste_euclidien() {
    let g = Grille::new(7, 3);
    assert_eq!(g.enrouler_x(-1), 6);
    assert_eq!(g.enrouler_x(7), 0);
    assert_eq!(g.enrouler_x(-7), 0);
    assert_eq!(g.enrouler_x(-8), 6);
    assert_eq!(g.enrouler_x(23), 2);
    for largeur in [1_usize, 2, 7, 64, 1000] {
        let g = Grille::new(largeur, 2);
        let l = largeur as isize;
        for x in -50..=50_isize {
            assert_eq!(g.enrouler_x(x), (((x % l) + l) % l) as usize, "largeur {largeur}, x {x}");
        }
        for x in [2_000_000_000_isize, -2_000_000_000, i32::MAX as isize, i32::MIN as isize] {
            assert_eq!(g.enrouler_x(x), (((x % l) + l) % l) as usize, "largeur {largeur}, x {x}");
        }
    }
}

/// juste : `dans_disque` est `dx² + dy² <= r²` : les triplets pythagoriciens `(3, 4, 5)`, `(5, 12, 13)`, `(8, 15, 17)`,
/// `(7, 24, 25)`, `(20, 21, 29)` sont dans le disque de rayon r et pas dans celui de r − 1, dans tous les signes et dans
/// les deux sens (le bord est dans le disque) ; `(1, 1)` est hors du disque de rayon 1 et dans celui de rayon 2 ; le rayon 0
/// ne contient que l'origine ; comparaison avec la définition réécrite ici sur `dx, dy` de -50 à 50 et `r` de 0 à 60
/// (V-GRILLE-08).
///
/// Ne couvre pas : les très grandes valeurs (voir `disque_sans_debordement`).
#[test]
#[cfg_attr(not(feature = "etape-0"), ignore)]
fn disque_definition() {
    for (a, b, c) in [(3_i32, 4_i32, 5_u32), (5, 12, 13), (8, 15, 17), (7, 24, 25), (20, 21, 29)] {
        for (sa, sb) in [(1, 1), (1, -1), (-1, 1), (-1, -1)] {
            assert!(dans_disque(sa * a, sb * b, c), "({}, {}) dans r = {c}", sa * a, sb * b);
            assert!(dans_disque(sb * b, sa * a, c), "({}, {}) dans r = {c}", sb * b, sa * a);
            assert!(!dans_disque(sa * a, sb * b, c - 1), "({}, {}) hors r = {}", sa * a, sb * b, c - 1);
        }
    }
    assert!(!dans_disque(1, 1, 1));
    assert!(dans_disque(1, 1, 2));
    assert!(dans_disque(0, 0, 0));
    assert!(!dans_disque(1, 0, 0));
    assert!(!dans_disque(0, -1, 0));
    assert!(dans_disque(5, 0, 5) && !dans_disque(6, 0, 5) && dans_disque(0, -5, 5) && !dans_disque(0, 6, 5));
    for dx in -50_i64..=50 {
        for dy in -50_i64..=50 {
            for r in 0_i64..=60 {
                assert_eq!(
                    dans_disque(dx as i32, dy as i32, r as u32),
                    dx * dx + dy * dy <= r * r,
                    "({dx}, {dy}), r = {r}"
                );
            }
        }
    }
}

/// juste : le nombre de points entiers du disque de rayon `r` (`dans_disque` compté sur le carré `[-r, r]²`) est, pour
/// `r` de 0 à 20, 1, 5, 13, 29, 49, 81, 113, 149, 197, 253, 317, 377, 441, 529, 613, 709, 797, 901, 1009, 1129, 1257
/// (V-GRILLE-09, suite A000328 de l'OEIS, « Number of points of norm <= n^2 in square lattice »).
///
/// Ne couvre pas : les rayons de plus de 20.
#[test]
#[cfg_attr(not(feature = "etape-0"), ignore)]
fn disque_nombre_de_points_entiers() {
    let a000328 = [1, 5, 13, 29, 49, 81, 113, 149, 197, 253, 317, 377, 441, 529, 613, 709, 797, 901, 1009, 1129, 1257];
    for (r, attendu) in a000328.into_iter().enumerate() {
        let r = r as i32;
        let mut compte = 0;
        for dx in -r..=r {
            for dy in -r..=r {
                if dans_disque(dx, dy, r as u32) {
                    compte += 1;
                }
            }
        }
        assert_eq!(compte, attendu, "rayon {r}");
    }
}

/// juste : aucun débordement ni flottant aux grandes valeurs : `i32::MIN` aux deux coordonnées avec le rayon `u32::MAX` est
/// dans le disque (`2^63 <= (2^32 - 1)²`), et `(i32::MAX, i32::MAX)` est dans le disque de rayon 3037000499 mais pas dans
/// celui de 3037000498 (`2·(2^31 - 1)² = 9223372028264841218`, entre `3037000498² = 9223372024852248004` et
/// `3037000499² = 9223372030926249001`) (V-GRILLE-10, calcul entier exact en Python).
///
/// Ne couvre pas : les valeurs intermédiaires entre les petites et les extrêmes (voir `disque_definition`).
#[test]
#[cfg_attr(not(feature = "etape-0"), ignore)]
fn disque_sans_debordement() {
    assert!(dans_disque(i32::MIN, i32::MIN, u32::MAX));
    assert!(dans_disque(i32::MIN, 0, u32::MAX));
    assert!(!dans_disque(i32::MIN, 0, 2_147_483_647));
    assert!(dans_disque(i32::MIN, 0, 2_147_483_648));
    assert!(dans_disque(i32::MAX, i32::MAX, 3_037_000_499));
    assert!(!dans_disque(i32::MAX, i32::MAX, 3_037_000_498));
    assert!(dans_disque(0, 0, u32::MAX));
}

//! Étape 0 : `reseau`, la grille hexagonale (coordonnées axiales `(q, r)`, `s = -q - r`). Valeurs attendues :
//! `references/etape-0-valeurs.md`, identifiants `V-HEX-xx`.
//!
//! Sources : la documentation de `reseau` (l'ordre des directions et des cases est une convention écrite) ; le nombre de
//! cases `3R(R+1) + 1` (suite A003215 de l'OEIS, « Hex (or centered hexagonal) numbers ») ; les définitions des
//! coordonnées cubiques, de la distance `max(|q|, |r|, |s|)` et de la rotation de 60 degrés `[q, r, s] → [-r, -s, -q]` de
//! Red Blob Games, « Hexagonal Grids » (redblobgames.com/grids/hexagons/, lu : « Rotation », « Distances »,
//! « Map storage » pour l'hexagone). Les symétries de l'étape 3 reposeront sur `tourner60` : ces tests sont les plus
//! stricts du module.

use givre_lab::reseau::{DIRECTIONS_HEX, Hexagone, distance_hex, tourner60};

/// L'énumération naïve de l'ordre canonique de la doc : `r` de `-R` à `R`, puis `q` de `max(-R, -R - r)` à
/// `min(R, R - r)`.
fn cases_dans_l_ordre(rayon: i32) -> Vec<(i32, i32)> {
    let mut cases = Vec::new();
    for r in -rayon..=rayon {
        for q in (-rayon).max(-rayon - r)..=rayon.min(rayon - r) {
            cases.push((q, r));
        }
    }
    cases
}

/// juste : les six déplacements sont, dans l'ordre de la doc, `(1, 0)`, `(0, 1)`, `(-1, 1)`, `(-1, 0)`, `(0, -1)`,
/// `(1, -1)` (V-HEX-01).
///
/// Ne couvre pas : que `voisins` s'en serve (voir `voisins_litteraux`).
#[test]
#[cfg_attr(not(feature = "etape-0"), ignore)]
fn ordre_des_directions() {
    assert_eq!(DIRECTIONS_HEX, [(1, 0), (0, 1), (-1, 1), (-1, 0), (0, -1), (1, -1)]);
}

/// juste : le nombre de cases est `3R(R+1) + 1` : 1, 7, 19, 37, 61, 91, 127, 169, 217, 271, 331, 397 pour `R` de 0 à 11
/// (V-HEX-02, OEIS A003215) ; pour le rayon maximal 32768, `3 · 32768 · 32769 + 1`.
///
/// Ne couvre pas : le contenu des cases (voir les tests d'indice).
#[test]
#[cfg_attr(not(feature = "etape-0"), ignore)]
fn nombre_de_cases() {
    let a003215 = [1, 7, 19, 37, 61, 91, 127, 169, 217, 271, 331, 397];
    for (rayon, attendu) in a003215.into_iter().enumerate() {
        let h = Hexagone::new(rayon as u32);
        assert_eq!(h.cases(), attendu, "rayon {rayon}");
        assert_eq!(h.rayon(), rayon as u32);
        assert_eq!(cases_dans_l_ordre(rayon as i32).len(), attendu);
    }
    assert_eq!(Hexagone::new(32768).cases() as u64, 3 * 32768 * 32769 + 1);
}

/// juste : `contient` est `max(|q|, |r|, |s|) <= R` : bord inclus, et pas `|q| <= R && |r| <= R` (qui accepterait `(R, 1)`).
/// Sur `R = 2` : dedans `(2, 0)`, `(2, -2)`, `(-2, 2)`, `(2, -1)`, `(0, 0)` ; dehors `(2, 1)`, `(1, 2)`, `(-2, -1)`,
/// `(3, 0)`, `(0, -3)`, `(-3, 3)` ; comparaison avec la définition réécrite ici pour `q, r` de -9 à 9 et `R` de 0 à 6 ;
/// aucune panique pour les valeurs extrêmes (V-HEX-03).
///
/// Ne couvre pas : l'orientation des axes (une convention).
#[test]
#[cfg_attr(not(feature = "etape-0"), ignore)]
fn contient_le_bord() {
    let h = Hexagone::new(2);
    for (q, r) in [(2, 0), (2, -2), (-2, 2), (2, -1), (0, 0), (0, 2), (-2, 0), (1, 1), (-1, -1), (-2, 1), (1, -2)] {
        assert!(h.contient(q, r), "({q}, {r}) doit être dedans");
    }
    for (q, r) in [(2, 1), (1, 2), (-2, -1), (-1, -2), (3, 0), (0, -3), (-3, 3), (2, 2), (-2, -2), (0, 3), (3, -3)] {
        assert!(!h.contient(q, r), "({q}, {r}) doit être dehors");
    }
    for rayon in 0..=6_i32 {
        let h = Hexagone::new(rayon as u32);
        let mut dedans = 0;
        for q in -9..=9_i32 {
            for r in -9..=9_i32 {
                let attendu = q.abs().max(r.abs()).max((q + r).abs()) <= rayon;
                assert_eq!(h.contient(q, r), attendu, "R = {rayon}, ({q}, {r})");
                dedans += usize::from(attendu);
            }
        }
        assert_eq!(dedans, h.cases());
    }
    let h = Hexagone::new(5);
    for (q, r) in
        [(i32::MIN, i32::MAX), (i32::MIN, i32::MIN), (i32::MAX, i32::MAX), (i32::MAX, i32::MIN), (i32::MAX, 0)]
    {
        assert!(!h.contient(q, r), "({q}, {r})");
    }
}

/// juste : l'ordre canonique pour `R = 0`, `R = 1` et `R = 2`, écrit en toutes lettres : `position(i)` est la i-ème case
/// et `index` est son inverse (V-HEX-04, ordre fixé par la doc : `r` croissant puis `q` croissant).
///
/// Ne couvre pas : les rayons de plus de 2 (voir `indice_contre_enumeration_naive`).
#[test]
#[cfg_attr(not(feature = "etape-0"), ignore)]
fn ordre_canonique_litteral() {
    let r0 = [(0, 0)];
    let r1 = [(0, -1), (1, -1), (-1, 0), (0, 0), (1, 0), (-1, 1), (0, 1)];
    let r2 = [
        (0, -2),
        (1, -2),
        (2, -2),
        (-1, -1),
        (0, -1),
        (1, -1),
        (2, -1),
        (-2, 0),
        (-1, 0),
        (0, 0),
        (1, 0),
        (2, 0),
        (-2, 1),
        (-1, 1),
        (0, 1),
        (1, 1),
        (-2, 2),
        (-1, 2),
        (0, 2),
    ];
    for (rayon, liste) in [(0_u32, &r0[..]), (1, &r1[..]), (2, &r2[..])] {
        let h = Hexagone::new(rayon);
        assert_eq!(h.cases(), liste.len());
        for (i, &(q, r)) in liste.iter().enumerate() {
            assert_eq!(h.position(i), (q, r), "R = {rayon}, position({i})");
            assert_eq!(h.index(q, r), i, "R = {rayon}, index({q}, {r})");
        }
    }
}

/// juste : pour `R` de 0 à 12, l'indice de chaque case est son rang dans l'énumération naïve de la doc (écrite dans le
/// test), `position` en est l'inverse, et `index` est une bijection de l'hexagone sur `0..cases()` (V-HEX-04).
///
/// Ne couvre pas : les rayons de plus de 12.
#[test]
#[cfg_attr(not(feature = "etape-0"), ignore)]
fn indice_contre_enumeration_naive() {
    for rayon in 0..=12_i32 {
        let h = Hexagone::new(rayon as u32);
        let liste = cases_dans_l_ordre(rayon);
        assert_eq!(liste.len(), h.cases());
        for (i, &(q, r)) in liste.iter().enumerate() {
            assert_eq!(h.index(q, r), i, "R = {rayon}, index({q}, {r})");
            assert_eq!(h.position(i), (q, r), "R = {rayon}, position({i})");
        }
    }
}

/// juste : `index` panique pour une case hors de l'hexagone, notamment `(R, 1)` (dont `|q|` et `|r|` sont au plus `R` mais
/// pas `|s|`) ; `position` panique pour `cases()` ; `Hexagone::new` panique au-delà de 32768 (V-HEX-05, la doc dit
/// « panique »).
///
/// Ne couvre pas : le message des paniques.
#[test]
#[cfg_attr(not(feature = "etape-0"), ignore)]
fn panique_hors_hexagone() {
    use std::panic::catch_unwind;
    let h = Hexagone::new(3);
    assert!(catch_unwind(|| h.index(3, 1)).is_err(), "index(R, 1)");
    assert!(catch_unwind(|| h.index(4, 0)).is_err(), "index(R + 1, 0)");
    assert!(catch_unwind(|| h.index(-3, -1)).is_err(), "index(-R, -1)");
    assert!(catch_unwind(|| h.position(h.cases())).is_err(), "position(cases)");
    assert!(catch_unwind(|| h.voisins(3, 1)).is_err(), "voisins hors hexagone");
    assert!(catch_unwind(|| Hexagone::new(32769)).is_err(), "rayon trop grand");
    assert!(catch_unwind(|| Hexagone::new(32768)).is_ok(), "32768 est permis");
}

/// juste : les six voisins, dans l'ordre de la doc, au centre, sur un coin, sur le milieu d'un côté ; `None` à sa place
/// pour les cases hors de l'hexagone (V-HEX-06). Pour `R = 2` : le coin `(2, 0)` n'a que les directions 2, 3, 4 ; le coin
/// `(-2, 2)` les directions 0, 4, 5 ; la case `(1, 1)` du bord les directions 2 à 5. Rayon 0 : aucun voisin.
///
/// Ne couvre pas : les grands rayons (voir `voisins_contre_definition`).
#[test]
#[cfg_attr(not(feature = "etape-0"), ignore)]
fn voisins_litteraux() {
    let h = Hexagone::new(2);
    assert_eq!(
        h.voisins(0, 0),
        [Some((1, 0)), Some((0, 1)), Some((-1, 1)), Some((-1, 0)), Some((0, -1)), Some((1, -1))]
    );
    assert_eq!(h.voisins(2, 0), [None, None, Some((1, 1)), Some((1, 0)), Some((2, -1)), None]);
    assert_eq!(h.voisins(-2, 2), [Some((-1, 2)), None, None, None, Some((-2, 1)), Some((-1, 1))]);
    assert_eq!(h.voisins(1, 1), [None, None, Some((0, 2)), Some((0, 1)), Some((1, 0)), Some((2, 0))]);
    assert_eq!(Hexagone::new(0).voisins(0, 0), [None; 6]);
    assert_eq!(
        Hexagone::new(1).voisins(0, 0),
        [Some((1, 0)), Some((0, 1)), Some((-1, 1)), Some((-1, 0)), Some((0, -1)), Some((1, -1))]
    );
}

/// juste : pour `R = 5`, les voisins de chaque case sont, au rang `k`, la case `(q + dq, r + dr)` pour la direction `k` si
/// `contient` (définition réécrite ici), sinon `None` ; la relation est symétrique ; une case intérieure (`distance < R`) a
/// six voisins, une case du bord en a trois (coin) ou quatre (côté) (V-HEX-06).
///
/// Ne couvre pas : les autres rayons que 5.
#[test]
#[cfg_attr(not(feature = "etape-0"), ignore)]
fn voisins_contre_definition() {
    let rayon = 5_i32;
    let h = Hexagone::new(rayon as u32);
    for (q, r) in cases_dans_l_ordre(rayon) {
        let voisins = h.voisins(q, r);
        let mut presents = 0;
        for (k, &(dq, dr)) in DIRECTIONS_HEX.iter().enumerate() {
            let (nq, nr) = (q + dq, r + dr);
            let dedans = nq.abs().max(nr.abs()).max((nq + nr).abs()) <= rayon;
            assert_eq!(voisins[k], dedans.then_some((nq, nr)), "voisins({q}, {r})[{k}]");
            presents += usize::from(dedans);
            if dedans {
                assert!(h.voisins(nq, nr).contains(&Some((q, r))), "symétrie ({q}, {r}) et ({nq}, {nr})");
            }
        }
        let distance = q.abs().max(r.abs()).max((q + r).abs());
        match (distance < rayon, (q.abs() == rayon) as u8 + (r.abs() == rayon) as u8 + ((q + r).abs() == rayon) as u8) {
            (true, _) => assert_eq!(presents, 6),
            (false, 1) => assert_eq!(presents, 4, "côté ({q}, {r})"),
            (false, _) => assert_eq!(presents, 3, "coin ({q}, {r})"),
        }
    }
}

/// juste : `tourner60` sur les six directions, écrite en toutes lettres : `(1, 0) → (0, 1) → (-1, 1) → (-1, 0) → (0, -1) →
/// (1, -1) → (1, 0)` ; et la formule cubique `(q, r, s) → (-r, -s, -q)` de Red Blob Games (« rotate 60° right ») : le
/// nouveau `q` est `-r`, le nouveau `r` est `-s = q + r` (V-HEX-07).
///
/// Ne couvre pas : l'invariance de l'hexagone (voir `rotation_invariants`).
#[test]
#[cfg_attr(not(feature = "etape-0"), ignore)]
fn tourner60_litteral() {
    let chaine = [(1, 0), (0, 1), (-1, 1), (-1, 0), (0, -1), (1, -1), (1, 0)];
    for paire in chaine.windows(2) {
        assert_eq!(tourner60(paire[0].0, paire[0].1), paire[1], "rotation de {:?}", paire[0]);
    }
    for q in -6..=6_i32 {
        for r in -6..=6_i32 {
            let s = -q - r;
            let (nq, nr) = tourner60(q, r);
            assert_eq!((nq, nr, -nq - nr), (-r, -s, -q), "rotation cubique de ({q}, {r}, {s})");
        }
    }
    assert_eq!(tourner60(3, 0), (0, 3));
    assert_eq!(tourner60(2, 1), (-1, 3));
    assert_eq!(tourner60(0, 0), (0, 0));
}

/// juste : le sens de `tourner60` est celui de l'ordre des voisins : tourner la direction `k` donne la direction
/// `(k + 1) % 6`, pour les six directions (V-HEX-07, convention de la doc).
///
/// Ne couvre pas : le sens horaire ou antihoraire à l'écran (la doc ne le dit pas).
#[test]
#[cfg_attr(not(feature = "etape-0"), ignore)]
fn tourner60_suit_l_ordre_des_voisins() {
    for k in 0..6 {
        let (dq, dr) = DIRECTIONS_HEX[k];
        assert_eq!(tourner60(dq, dr), DIRECTIONS_HEX[(k + 1) % 6], "direction {k}");
    }
}

/// juste : propriétés entières exactes de la rotation, sur tout l'hexagone de rayon 6 : six rotations ramènent à la case
/// (identité), trois donnent `(-q, -r)`, l'origine est fixe, chaque case reste dans l'hexagone (c'est une bijection de
/// l'hexagone), la distance à l'origine est conservée, et la distance entre deux cases aussi ; sur une case lointaine,
/// `(100, -37)`, six rotations ramènent aussi à elle (V-HEX-07 et V-HEX-08).
///
/// Ne couvre pas : les propriétés des flocons (étape 3).
#[test]
#[cfg_attr(not(feature = "etape-0"), ignore)]
fn rotation_invariants() {
    let h = Hexagone::new(6);
    let toutes = cases_dans_l_ordre(6);
    assert_eq!(tourner60(0, 0), (0, 0));
    let mut images = Vec::new();
    for &(q, r) in &toutes {
        let mut p = (q, r);
        let mut trois = (0, 0);
        for pas in 1..=6 {
            p = tourner60(p.0, p.1);
            assert!(h.contient(p.0, p.1), "({q}, {r}) tourné {pas} fois sort de l'hexagone");
            if pas == 3 {
                trois = p;
            }
        }
        assert_eq!(p, (q, r), "six rotations de ({q}, {r})");
        assert_eq!(trois, (-q, -r), "trois rotations de ({q}, {r})");
        let t = tourner60(q, r);
        assert_eq!(distance_hex((0, 0), t), distance_hex((0, 0), (q, r)));
        images.push(h.index(t.0, t.1));
    }
    images.sort_unstable();
    assert_eq!(images, (0..h.cases()).collect::<Vec<_>>(), "la rotation est une bijection de l'hexagone");
    for &a in toutes.iter().step_by(3) {
        for &b in toutes.iter().step_by(5) {
            assert_eq!(distance_hex(tourner60(a.0, a.1), tourner60(b.0, b.1)), distance_hex(a, b), "{a:?} {b:?}");
        }
    }
    let mut p = (100, -37);
    for _ in 0..6 {
        p = tourner60(p.0, p.1);
    }
    assert_eq!(p, (100, -37));
}

/// juste : la distance hexagonale `max(|dq|, |dr|, |ds|)` sur des cas écrits à la main : `((0,0),(1,0)) = 1`,
/// `((0,0),(1,1)) = 2` (car `s = -2`), `((0,0),(3,-5)) = 5`, `((2,-3),(-1,4)) = 7`, et aux grandes valeurs
/// `((0,0),(2^29,-2^29)) = 2^29`, `((-2^29,0),(2^29,0)) = 2^30` (V-HEX-08, Red Blob Games « Distances »).
///
/// Ne couvre pas : les distances au-delà de 2^30.
#[test]
#[cfg_attr(not(feature = "etape-0"), ignore)]
fn distance_litteral() {
    assert_eq!(distance_hex((0, 0), (0, 0)), 0);
    assert_eq!(distance_hex((0, 0), (1, 0)), 1);
    assert_eq!(distance_hex((0, 0), (2, -1)), 2);
    assert_eq!(distance_hex((0, 0), (1, 1)), 2);
    assert_eq!(distance_hex((0, 0), (-1, -1)), 2);
    assert_eq!(distance_hex((0, 0), (3, -5)), 5);
    assert_eq!(distance_hex((2, -3), (-1, 4)), 7);
    assert_eq!(distance_hex((-1, 4), (2, -3)), 7);
    assert_eq!(distance_hex((0, 0), (1 << 29, -(1 << 29))), 1 << 29);
    assert_eq!(distance_hex((-(1 << 29), 0), (1 << 29, 0)), 1 << 30);
}

/// juste : la distance hexagonale est le nombre minimal de pas de voisin à voisin : pour la case `(2, -3)` de l'hexagone
/// de rayon 10, elle est égale, pour chaque case de l'hexagone, à la longueur du plus court chemin trouvée par un
/// parcours en largeur écrit dans le test (qui n'utilise que `voisins`) ; les cases à distance `d` de l'origine sont au
/// nombre de `6d` ; `contient(q, r)` équivaut à `distance((0,0), (q,r)) <= R` (V-HEX-08).
///
/// Ne couvre pas : les autres cases de départ que `(2, -3)`.
#[test]
#[cfg_attr(not(feature = "etape-0"), ignore)]
fn distance_contre_parcours_en_largeur() {
    let rayon = 10_i32;
    let h = Hexagone::new(rayon as u32);
    let depart = (2, -3);
    let mut distances = vec![u32::MAX; h.cases()];
    distances[h.index(depart.0, depart.1)] = 0;
    let mut file = std::collections::VecDeque::from([depart]);
    while let Some((q, r)) = file.pop_front() {
        let d = distances[h.index(q, r)];
        for (nq, nr) in h.voisins(q, r).into_iter().flatten() {
            let i = h.index(nq, nr);
            if distances[i] == u32::MAX {
                distances[i] = d + 1;
                file.push_back((nq, nr));
            }
        }
    }
    let mut par_distance = vec![0_usize; rayon as usize + 1];
    for (q, r) in cases_dans_l_ordre(rayon) {
        assert_eq!(distance_hex(depart, (q, r)), distances[h.index(q, r)], "({q}, {r})");
        par_distance[distance_hex((0, 0), (q, r)) as usize] += 1;
    }
    assert_eq!(par_distance[0], 1);
    for (d, &nombre) in par_distance.iter().enumerate().skip(1) {
        assert_eq!(nombre, 6 * d, "cases à distance {d} de l'origine");
    }
    for q in -14..=14_i32 {
        for r in -14..=14_i32 {
            assert_eq!(h.contient(q, r), distance_hex((0, 0), (q, r)) <= rayon as u32, "({q}, {r})");
        }
    }
}

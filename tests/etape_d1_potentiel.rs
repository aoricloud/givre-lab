//! Étape D1 : le potentiel (`Claquage::potentiel`), tests EXACTS et de propriété. Valeurs attendues : `references/etape-d1-valeurs.md`,
//! identifiants `V-D1-11` à `V-D1-14`, et `V-D1-29`, `V-D1-31` (les bornes que la doc déduit du seuil).
//!
//! Ces tests ne regardent que ce que la doc de `src/claquage.rs` dit du champ : `φ = 0` sur l'amas, `φ = 1` hors du domaine, `0 <= φ <= 1`,
//! et chaque site libre est la moyenne de ses quatre voisins à `2 * SEUIL` près (la doc déduit cette borne du seuil d'arrêt). L'écart à
//! la solution EXACTE du problème de Laplace discret est au plus `2 * SEUIL * (d + 1)²` (principe du maximum, dit dans la doc) : c'est la
//! tolérance de ces tests, jamais un réglage. Le champ lui-même, bit pour bit, est jugé par `etape_d1_exact`.

mod etape_d1_commun;

use std::collections::BTreeMap;

use etape_d1_commun::SEUIL_DOC;
use givre_lab::claquage::Claquage;

/// La tolérance de la doc sur le champ, à `demi_cote = d` : `2 * SEUIL * (d + 1)²`.
fn tolerance_champ(d: i32) -> f64 {
    2.0 * SEUIL_DOC * f64::from((d + 1) * (d + 1))
}

/// Le potentiel d'un site, qui doit exister (`eta >= 1`).
fn phi(c: &Claquage, x: i32, y: i32) -> f64 {
    c.potentiel(x, y).expect("un champ à eta >= 1")
}

fn dans_omega(d: i32, x: i32, y: i32) -> bool {
    i64::from(x) * i64::from(x) + i64::from(y) * i64::from(y) <= i64::from(d) * i64::from(d)
}

/// La solution du problème de Laplace discret par élimination de Gauss à pivot partiel (dense, en `f64`) : pour chaque site libre `i`,
/// `4 φ_i - (somme des voisins libres) = (nombre de voisins hors du domaine)`, les voisins de l'amas valant 0. Écrite à part de la
/// relaxation de la doc (aucune itération), pour de petits domaines (`demi_cote` <= 10).
fn solution_directe(d: i32, amas: &[(i32, i32)]) -> BTreeMap<(i32, i32), f64> {
    let mut libres = Vec::new();
    for y in -d..=d {
        for x in -d..=d {
            if dans_omega(d, x, y) && !amas.contains(&(x, y)) {
                libres.push((x, y));
            }
        }
    }
    let n = libres.len();
    let indice: BTreeMap<(i32, i32), usize> = libres.iter().enumerate().map(|(k, &z)| (z, k)).collect();
    let mut a = vec![vec![0.0_f64; n + 1]; n];
    for (i, &(x, y)) in libres.iter().enumerate() {
        a[i][i] = 4.0;
        for (dx, dy) in [(1, 0), (0, 1), (-1, 0), (0, -1)] {
            let v = (x + dx, y + dy);
            if !dans_omega(d, v.0, v.1) {
                a[i][n] += 1.0;
            } else if let Some(&j) = indice.get(&v) {
                a[i][j] -= 1.0;
            }
        }
    }
    for colonne in 0..n {
        let pivot = (colonne..n).max_by(|&p, &q| a[p][colonne].abs().total_cmp(&a[q][colonne].abs())).unwrap();
        a.swap(colonne, pivot);
        let diviseur = a[colonne][colonne];
        for v in a[colonne].iter_mut() {
            *v /= diviseur;
        }
        let ligne_pivot = a[colonne].clone();
        for (ligne, rang) in a.iter_mut().enumerate() {
            if ligne != colonne && rang[colonne] != 0.0 {
                let facteur = rang[colonne];
                for (v, p) in rang.iter_mut().zip(&ligne_pivot) {
                    *v -= facteur * p;
                }
            }
        }
    }
    libres.iter().enumerate().map(|(i, &z)| (z, a[i][n])).collect()
}

/// L'écart maximal du champ de `c` à la solution directe de son amas, sur les sites libres du domaine.
fn ecart_a_la_solution_directe(c: &Claquage) -> f64 {
    let d = c.demi_cote() as i32;
    let exacte = solution_directe(d, c.sites());
    exacte.iter().map(|(&(x, y), &v)| (phi(c, x, y) - v).abs()).fold(0.0, f64::max)
}

/// juste : le champ du GERME SEUL est la solution exacte de Laplace (V-D1-11). Les valeurs ci-dessous sont les FRACTIONS EXACTES du
/// problème de Laplace discret (4 voisins, `φ = 1` hors du domaine `x² + y² <= d²`, `φ = 0` au germe), résolues en rationnels par le code
/// Python indépendant `d1_ref.py exact` (module `fractions`, élimination de Gauss exacte), pour `demi_cote` = 1 à 6, un représentant
/// `0 <= y <= x` de chaque orbite (les 8 symétries du carré donnent le reste). À la main : `d = 1` donne `3/4` (chaque voisin du germe a
/// trois voisins à 1 et le germe à 0) ; `d = 2` donne `7/11` en `(1,0)`, `9/11` en `(1,1)`, `10/11` en `(2,0)`
/// (`4a = c + 2b`, `4b = 2 + 2a`, `4c = 3 + a`). Pour chaque site libre et chacune de ses huit images par symétrie, `|φ - exact| <=
/// 2 * SEUIL * (d + 1)²` (la tolérance de la doc, jamais un réglage ; mesuré : l'écart réel est plus petit de plusieurs ordres de grandeur).
///
/// Ne couvre pas : un amas de plus d'un site (voir `champ_contre_la_solution_directe`).
#[test]
#[cfg_attr(not(feature = "etape-d1"), ignore)]
fn germe_seul_solutions_exactes() {
    #[allow(clippy::type_complexity)]
    let cas: [(i32, &[((i32, i32), i64, i64)]); 6] = [
        (1, &[((1, 0), 3, 4)]),
        (2, &[((1, 0), 7, 11), ((1, 1), 9, 11), ((2, 0), 10, 11)]),
        (
            3,
            &[
                ((1, 0), 93, 167),
                ((1, 1), 237, 334),
                ((2, 0), 135, 167),
                ((2, 1), 144, 167),
                ((2, 2), 311, 334),
                ((3, 0), 159, 167),
            ],
        ),
        (
            4,
            &[
                ((1, 0), 2411, 4711),
                ((1, 1), 3070, 4711),
                ((2, 0), 3504, 4711),
                ((2, 1), 3729, 4711),
                ((2, 2), 4085, 4711),
                ((3, 0), 4147, 4711),
                ((3, 1), 4257, 4711),
                ((3, 2), 4441, 4711),
                ((4, 0), 4570, 4711),
            ],
        ),
        (
            5,
            &[
                ((1, 0), 107275, 225429),
                ((1, 1), 45522, 75143),
                ((2, 0), 155968, 225429),
                ((2, 1), 165857, 225429),
                ((2, 2), 363557, 450858),
                ((3, 0), 184883, 225429),
                ((3, 1), 126077, 150286),
                ((3, 2), 65900, 75143),
                ((3, 3), 415391, 450858),
                ((4, 0), 205333, 225429),
                ((4, 1), 208022, 225429),
                ((4, 2), 424421, 450858),
                ((4, 3), 217691, 225429),
                ((5, 0), 220405, 225429),
            ],
        ),
        (
            6,
            &[
                ((1, 0), 3045331, 6713754),
                ((1, 1), 3877237, 6713754),
                ((2, 0), 2213425, 3356877),
                ((2, 1), 4709143, 6713754),
                ((2, 2), 20668795, 26855016),
                ((3, 0), 5243783, 6713754),
                ((3, 1), 7153715, 8951672),
                ((3, 2), 11250509, 13427508),
                ((3, 3), 11865539, 13427508),
                ((4, 0), 3878473, 4475836),
                ((4, 1), 11765929, 13427508),
                ((4, 2), 12071527, 13427508),
                ((4, 3), 12480569, 13427508),
                ((4, 4), 25908077, 26855016),
                ((5, 0), 1043521, 1118959),
                ((5, 1), 8417465, 8951672),
                ((5, 2), 12789101, 13427508),
                ((5, 3), 26062343, 26855016),
                ((6, 0), 2200199, 2237918),
            ],
        ),
    ];
    for eta in [1_u32, 2, 3] {
        for (d, valeurs) in cas {
            let c = Claquage::new(5, d as u32, eta);
            let mut vus = 0;
            for &((x, y), numerateur, denominateur) in valeurs {
                let exact = numerateur as f64 / denominateur as f64;
                for (a, b) in [(x, y), (y, x)] {
                    for (sx, sy) in [(1, 1), (-1, 1), (1, -1), (-1, -1)] {
                        let obtenu = phi(&c, sx * a, sy * b);
                        assert!(
                            (obtenu - exact).abs() <= tolerance_champ(d),
                            "d {d}, eta {eta} : φ({}, {}) = {obtenu}, exact {numerateur}/{denominateur} = {exact}",
                            sx * a,
                            sy * b
                        );
                        vus += 1;
                    }
                }
            }
            assert!(vus >= 8, "d {d} : des valeurs lues");
            assert_eq!(phi(&c, 0, 0), 0.0, "d {d} : le germe vaut 0");
        }
    }
}

/// juste : le champ d'un amas de plusieurs sites est la solution de Laplace (V-D1-12). Pour `η` = 1, 2, 3, quatre graines, `demi_cote` 8 et
/// 10, à 3, 8 puis 15 sites (le même `Claquage`, grandi), le champ de chaque site libre est à moins de `2 * SEUIL * (d + 1)²` de la
/// solution directe de `solution_directe` (une élimination de Gauss dense, écrite dans ce fichier, qui ne relaxe rien : un code
/// indépendant de celui de la doc). Une relaxation arrêtée trop tôt, un voisin diagonal, une frontière à 0, un site ajouté qui n'est pas
/// remis à 0 ou un champ qui n'est pas relaxé après le dernier site ajouté donnent un écart de l'ordre de `0,01` à `0,5`. (Mesuré : l'écart
/// maximal sur l'ensemble des cas est de `2,9e-6`, contre une tolérance de `1,5e-4` à `d = 8` et de `2,3e-4` à `d = 10` ; au germe seul, de `4e-8` à `1,1e-6`.)
///
/// Ne couvre pas : les domaines au-delà de `demi_cote = 10` (la solution directe y est lente) ; le champ bit pour bit (voir `etape_d1_exact`).
#[test]
#[cfg_attr(not(feature = "etape-d1"), ignore)]
fn champ_contre_la_solution_directe() {
    let mut verifies = 0;
    for eta in 1..=3_u32 {
        for d in [8_i32, 10] {
            for graine in 1..=4_u64 {
                let mut c = Claquage::new(graine, d as u32, eta);
                for n in [3_usize, 8, 15] {
                    while c.sites().len() < n && c.essayer_ajouter().is_some() {}
                    if c.sites().len() < n {
                        break;
                    }
                    let ecart = ecart_a_la_solution_directe(&c);
                    assert!(
                        ecart <= tolerance_champ(d),
                        "eta {eta}, d {d}, graine {graine}, {n} sites : écart à la solution directe {ecart} > {}",
                        tolerance_champ(d)
                    );
                    verifies += 1;
                }
            }
        }
    }
    // Un code qui ne fait jamais croître l'amas ne vérifierait rien : 71 des 72 états de la liste sont atteints (mesuré : un amas touche la frontière avant 15 sites).
    assert_eq!(verifies, 71, "nombre d'états comparés à la solution directe");
}

/// juste : les propriétés du champ (V-D1-13), à chaque pas de la croissance, pour `η` = 1, 2 et 3, `demi_cote` = 12 et 20, quatre graines :
/// `φ = 0` sur chaque site de l'amas ; `φ = 1.0` EXACTEMENT hors du domaine (sur le bord de la boîte, au-delà, et très loin : `(1000, 1000)`,
/// `(i32::MAX, i32::MIN)`, sans panique) ; `0 <= φ <= 1` sur chaque site libre (principe du maximum : la doc fait un `clamp`) ; et chaque
/// site libre vaut la moyenne de ses quatre voisins (hors du domaine : 1) à `2 * SEUIL` près (la borne que la doc déduit du seuil d'arrêt).
/// (Mesuré : l'écart à la moyenne atteint `0,60 SEUIL` au plus, la borne de la doc est `2 SEUIL`.) Cela dit aussi que le champ est RELAXÉ après chaque site ajouté, y compris le dernier. À `η = 0`, `potentiel` rend `None` partout.
///
/// Ne couvre pas : que le champ soit la bonne solution (voir `germe_seul_solutions_exactes` et `champ_contre_la_solution_directe`).
#[test]
#[cfg_attr(not(feature = "etape-d1"), ignore)]
fn proprietes_du_champ_a_chaque_pas() {
    for eta in 1..=3_u32 {
        for d in [12_i32, 20] {
            for graine in 1..=4_u64 {
                let mut c = Claquage::new(graine, d as u32, eta);
                let mut pas = 0;
                loop {
                    verifier_le_champ(&c, d, &format!("eta {eta}, d {d}, graine {graine}, {} sites", c.sites().len()));
                    pas += 1;
                    if pas > 40 || c.essayer_ajouter().is_none() {
                        break;
                    }
                }
            }
        }
    }
    let c = Claquage::new(1, 12, 0);
    assert_eq!(c.potentiel(0, 0), None);
    assert_eq!(c.potentiel(3, 4), None);
    assert_eq!(c.potentiel(1000, 1000), None);
}

/// Les propriétés de `proprietes_du_champ_a_chaque_pas` pour l'état courant de `c`.
fn verifier_le_champ(c: &Claquage, d: i32, contexte: &str) {
    for (x, y) in [(1000, 1000), (i32::MAX, i32::MIN), (d + 1, 0), (0, -d - 1), (d + 2, d + 2), (-d - 1, d + 1)] {
        assert_eq!(c.potentiel(x, y), Some(1.0), "{contexte} : hors du domaine, φ({x}, {y}) vaut 1.0");
    }
    for &(x, y) in c.sites() {
        assert_eq!(c.potentiel(x, y), Some(0.0), "{contexte} : φ vaut 0 sur l'amas, en ({x}, {y})");
    }
    for y in -d..=d {
        for x in -d..=d {
            if !dans_omega(d, x, y) || c.contient(x, y) {
                continue;
            }
            let v = phi(c, x, y);
            assert!((0.0..=1.0).contains(&v), "{contexte} : φ({x}, {y}) = {v} hors de [0, 1]");
            let moyenne = (((phi(c, x + 1, y) + phi(c, x, y + 1)) + phi(c, x - 1, y)) + phi(c, x, y - 1)) / 4.0;
            assert!(
                (v - moyenne).abs() <= 2.0 * SEUIL_DOC,
                "{contexte} : φ({x}, {y}) = {v} et la moyenne de ses voisins {moyenne} diffèrent de plus de 2 SEUIL"
            );
        }
    }
}

/// juste : le champ du germe seul est symétrique et croît vers la frontière (V-D1-14). Pour `η = 1`, `demi_cote` 12 et 20, les huit images
/// par symétrie du carré de chaque site libre ont le même potentiel à `1e-9` près (mesuré : `1,3e-15` au plus, les sommes de voisins ne
/// différant que par l'ordre d'addition), et le long de l'axe `φ(1, 0) < φ(2, 0) < … < φ(d, 0) < 1` (la différence de deux valeurs
/// consécutives dépasse `1e-3` : mesuré, la plus petite est `0,010` à `d = 20` et `0,019` à `d = 12`). Une frontière à 0, un côté oublié ou un
/// voisin diagonal faussent la symétrie ou la croissance (le premier perd la croissance, le deuxième la symétrie).
///
/// Ne couvre pas : la valeur du champ (voir `germe_seul_solutions_exactes`).
#[test]
#[cfg_attr(not(feature = "etape-d1"), ignore)]
fn germe_seul_symetrique_et_croissant() {
    for d in [12_i32, 20] {
        let c = Claquage::new(1, d as u32, 1);
        for y in -d..=d {
            for x in -d..=d {
                if !dans_omega(d, x, y) || (x, y) == (0, 0) {
                    continue;
                }
                let v = phi(&c, x, y);
                for (a, b) in [(x, y), (y, x)] {
                    for (sx, sy) in [(1, 1), (-1, 1), (1, -1), (-1, -1)] {
                        let w = phi(&c, sx * a, sy * b);
                        assert!(
                            (v - w).abs() <= 1e-9,
                            "d {d} : φ({x}, {y}) = {v} et son image ({}, {}) = {w}",
                            sx * a,
                            sy * b
                        );
                    }
                }
            }
        }
        let mut precedent = 0.0;
        for k in 1..=d {
            let v = phi(&c, k, 0);
            assert!(v > precedent + 1e-3, "d {d} : φ({k}, 0) = {v} ne croît pas après {precedent}");
            precedent = v;
        }
        assert!(precedent < 1.0);
    }
}

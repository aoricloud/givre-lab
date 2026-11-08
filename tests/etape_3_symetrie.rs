//! Étape 3 : la symétrie d'ordre 6 et la symétrie miroir sont EXACTES (la somme des six voisins est une somme d'entiers), et le
//! modèle est équivariant sous les douze symétries de l'hexagone. Valeurs attendues : `references/etape-3-valeurs.md`,
//! identifiants `V-SYM-xx`.

mod etape_3_commun;

use etape_3_commun::valeurs::{ASYM_AVANT, ASYM_DEFAUT_REFLEXION_15, ASYM_DEFAUT_ROTATION_15, P1_AVANT, P2_AVANT};
use etape_3_commun::{brut_de, cases, parametres, symetries};
use givre_lab::flocon::{Flocon, Parametres, reflechir};
use givre_lab::math::Fixe;
use givre_lab::reseau::tourner60;

fn depuis_brut(rayon: u32, p: Parametres, brut: &[i64]) -> Flocon {
    let masses: Vec<Fixe> = brut.iter().map(|&b| Fixe(b)).collect();
    Flocon::depuis_masses(rayon, p, &masses)
}

/// Les défauts de symétrie recomptés par le test : le nombre de cases dont la masse diffère de celle de leur image par `g`.
fn defaut(f: &Flocon, rayon: i32, g: fn(i32, i32) -> (i32, i32)) -> usize {
    cases(rayon)
        .iter()
        .filter(|&&(q, r)| {
            let (a, b) = g(q, r);
            f.masse(q, r) != f.masse(a, b)
        })
        .count()
}

/// juste : depuis le germe central, la symétrie est exacte à chaque pas (V-SYM-01). Neuf jeux `(α, β, γ)` (dendrites, étoiles, plaques,
/// `α = 2`, `β = 0`), `R = 15`, 60 pas, et deux dendrites fines sur `R = 40`, 300 pas : à chaque pas `defaut_rotation()` et
/// `defaut_reflexion()` valent 0, et à la fin chacune des **douze** symétries de l'hexagone (écrites dans le test, y compris
/// `(q, r) -> (-q - r, q)` et les cinq autres réflexions) laisse la masse de chaque case inchangée. Un voisinage faussé (même à six
/// voisins), un ordre de mise à jour qui dépend de la case, une mise à jour en place cassent l'égalité des `brut`.
///
/// Ne couvre pas : les états de départ sans symétrie (voir `equivariance`).
#[test]
#[cfg_attr(not(feature = "etape-3"), ignore)]
fn symetrie_exacte_depuis_le_germe() {
    let jeux = [
        (15, "1", "0.4", "0.001", 60),
        (15, "0.7", "0.3", "0.05", 60),
        (15, "2", "0.2", "0.3", 60),
        (15, "0.5", "0", "0.2", 60),
        (15, "1.5", "0.55", "0.0001", 60),
        (15, "1", "0.45", "0.1", 60),
        (15, "0.3", "0.8", "0.002", 60),
        (40, "1", "0.35", "0", 300),
        (40, "1", "0.4", "0.0001", 300),
    ];
    for (rayon, a, b, g, pas) in jeux {
        let mut f = Flocon::new(rayon, parametres(a, b, g));
        for t in 1..=pas {
            f.pas();
            assert_eq!(f.defaut_rotation(), 0, "R={rayon} alpha {a} beta {b} gamma {g} pas {t}");
            assert_eq!(f.defaut_reflexion(), 0, "R={rayon} alpha {a} beta {b} gamma {g} pas {t}");
        }
        for (nom, s) in symetries() {
            assert_eq!(defaut(&f, rayon as i32, s), 0, "R={rayon} alpha {a} beta {b} gamma {g} : {nom}");
        }
    }
}

/// juste : les défauts de symétrie sur des états à la main (V-SYM-02), `R = 1`, le centre à 1/2 et les six voisins `k` (dans l'ordre
/// de `DIRECTIONS_HEX`) à 1/4 ou 3/4. Rotation de 60° : la case `k` va sur `k + 1` ; réflexion `(q, r) -> (r, q)` : `k` va sur `1 - k`
/// (modulo 6 : `0 <-> 1`, `2 <-> 5`, `3 <-> 4`). (a) une seule voisine à 3/4 (`k = 0`) : rotation 2 (`k = 0` et `k = 5`), réflexion 2
/// (`k = 0` et `k = 1`) ; (b) alternance 3/4, 1/4 : rotation 6, réflexion 6 ; (c) `k = 0` et `k = 1` à 3/4 : rotation 2 (`k = 1` et
/// `k = 5`), réflexion 0 ; (d) toutes égales : 0 et 0, même avec un centre différent. Et `R = 2` : un état sans symétrie dont les
/// défauts, recomptés par le test avec `tourner60` et `reflechir`, sont ceux de la lib. Un défaut qui compte une case seulement dans
/// un sens, qui compare une case à elle-même, ou qui compte le centre échoue ici.
///
/// Ne couvre pas : les défauts d'un état qui évolue (voir `defauts_depuis_un_etat_sans_symetrie`).
#[test]
#[cfg_attr(not(feature = "etape-3"), ignore)]
fn defauts_a_la_main() {
    let p = parametres("1", "0.3", "0.01");
    let (quart, trois_quarts, demi) = (Fixe(1 << 30), Fixe(3 << 30), Fixe(1 << 31));
    let directions = [(1, 0), (0, 1), (-1, 1), (-1, 0), (0, -1), (1, -1)];
    let etat = |centre: Fixe, anneau: [Fixe; 6]| {
        let masses: Vec<Fixe> =
            cases(1).iter().map(|&c| directions.iter().position(|&d| d == c).map_or(centre, |k| anneau[k])).collect();
        Flocon::depuis_masses(1, p, &masses)
    };
    let (q, t) = (quart, trois_quarts);
    let f = etat(demi, [t, q, q, q, q, q]);
    assert_eq!((f.defaut_rotation(), f.defaut_reflexion()), (2, 2));
    let f = etat(demi, [t, q, t, q, t, q]);
    assert_eq!((f.defaut_rotation(), f.defaut_reflexion()), (6, 6));
    let f = etat(demi, [t, t, q, q, q, q]);
    assert_eq!((f.defaut_rotation(), f.defaut_reflexion()), (2, 0));
    let f = etat(demi, [q, q, q, q, q, q]);
    assert_eq!((f.defaut_rotation(), f.defaut_reflexion()), (0, 0));
    let f = etat(trois_quarts, [q, q, q, q, q, q]);
    assert_eq!((f.defaut_rotation(), f.defaut_reflexion()), (0, 0));

    let f = depuis_brut(2, p, &P1_AVANT);
    let rot = defaut(&f, 2, tourner60);
    let refl = defaut(&f, 2, reflechir);
    assert!(rot > 0 && refl > 0, "l'état P1 n'a aucune de ces symétries");
    assert_eq!((f.defaut_rotation(), f.defaut_reflexion()), (rot, refl));
}

/// juste : un état de départ sans symétrie garde ses défauts (V-SYM-03). `ASYM_AVANT` (`R = 6`, `α = 0.9`, `β = 0.35`, `γ = 0.02`, quatre
/// cases gelées placées sans symétrie) : au départ les défauts recomptés par le test sont ceux de la lib ; après 15 pas ils valent
/// `ASYM_DEFAUT_ROTATION_15 = 90` et `ASYM_DEFAUT_REFLEXION_15 = 86` (Python, `modele_independant.py`). Un défaut qui rendrait toujours 0 échoue.
///
/// Ne couvre pas : l'équivariance.
#[test]
#[cfg_attr(not(feature = "etape-3"), ignore)]
fn defauts_depuis_un_etat_sans_symetrie() {
    let p = parametres("0.9", "0.35", "0.02");
    let mut f = depuis_brut(6, p, &ASYM_AVANT);
    assert_eq!(f.defaut_rotation(), defaut(&f, 6, tourner60));
    assert_eq!(f.defaut_reflexion(), defaut(&f, 6, reflechir));
    assert!(f.defaut_rotation() > 0);
    f.avancer(15);
    assert_eq!(f.defaut_rotation(), ASYM_DEFAUT_ROTATION_15);
    assert_eq!(f.defaut_reflexion(), ASYM_DEFAUT_REFLEXION_15);
    assert_eq!(defaut(&f, 6, tourner60), ASYM_DEFAUT_ROTATION_15);
}

/// juste : le modèle est équivariant sous les douze symétries de l'hexagone (V-SYM-04) : pour tout état `S` et toute symétrie `g`, faire
/// un pas sur l'état `g(S)` (la masse de `c` posée sur `g(c)`) donne `g` de l'état d'un pas sur `S`, **exactement**, à chaque pas. Deux
/// états sans symétrie (`P2_AVANT` avec `α = 2, β = 0.5, γ = 0.25`, `R = 3`, et `ASYM_AVANT` avec `α = 0.9, β = 0.35, γ = 0.02`, `R = 6`),
/// douze symétries, trois pas : 72 comparaisons de tout l'état. C'est la propriété que la symétrie du germe ne voit pas : un voisinage
/// de six cases qui n'est pas celui de l'hexagone (par exemple `(1,1)` et `(-1,-1)` à la place de `(1,-1)` et `(-1,1)`) la viole, un germe
/// symétrique n'y verrait rien.
///
/// Ne couvre pas : les valeurs numériques d'un pas (voir `etape_3_pas`).
#[test]
#[cfg_attr(not(feature = "etape-3"), ignore)]
fn equivariance() {
    let cas: [(u32, &str, &str, &str, &[i64]); 2] =
        [(3, "2", "0.5", "0.25", &P2_AVANT), (6, "0.9", "0.35", "0.02", &ASYM_AVANT)];
    for (rayon, a, b, g, brut) in cas {
        let p = parametres(a, b, g);
        let toutes = cases(rayon as i32);
        let indice = |c: (i32, i32)| toutes.iter().position(|&x| x == c).expect("dans l'hexagone");
        for (nom, s) in symetries() {
            let mut transforme = vec![0_i64; brut.len()];
            for (i, &(q, r)) in toutes.iter().enumerate() {
                transforme[indice(s(q, r))] = brut[i];
            }
            let mut d_abord = depuis_brut(rayon, p, brut);
            let mut ensuite = depuis_brut(rayon, p, &transforme);
            for t in 1..=3 {
                d_abord.pas();
                ensuite.pas();
                let attendu = brut_de(&d_abord.masses());
                let mut attendu_transforme = vec![0_i64; attendu.len()];
                for (i, &(q, r)) in toutes.iter().enumerate() {
                    attendu_transforme[indice(s(q, r))] = attendu[i];
                }
                assert_eq!(brut_de(&ensuite.masses()), attendu_transforme, "R={rayon} {nom} pas {t}");
            }
        }
    }
}

/// juste : `reflechir(q, r)` est `(r, q)` (V-SYM-05), comme le dit sa doc : sur `q` et `r` de -9 à 9 ; c'est une involution ; elle envoie les six
/// directions `[(1, 0), (0, 1), (-1, 1), (-1, 0), (0, -1), (1, -1)]` sur `[(0, 1), (1, 0), (1, -1), (0, -1), (-1, 0), (-1, 1)]` ; elle garde la
/// distance au centre `max(|q|, |r|, |q + r|)` ; et elle est dans le groupe des douze symétries du test (`(q, r) -> (r, q)`).
///
/// Ne couvre pas : l'effet de `reflechir` sur un état (voir `equivariance`).
#[test]
#[cfg_attr(not(feature = "etape-3"), ignore)]
fn reflechir_est_q_r_vers_r_q() {
    for q in -9..=9 {
        for r in -9..=9 {
            assert_eq!(reflechir(q, r), (r, q));
            assert_eq!(reflechir(reflechir(q, r).0, reflechir(q, r).1), (q, r));
            assert_eq!(etape_3_commun::distance(q, r), etape_3_commun::distance(r, q));
        }
    }
    let directions = [(1, 0), (0, 1), (-1, 1), (-1, 0), (0, -1), (1, -1)];
    let images = [(0, 1), (1, 0), (1, -1), (0, -1), (-1, 0), (-1, 1)];
    for (d, image) in directions.iter().zip(images) {
        assert_eq!(reflechir(d.0, d.1), image);
    }
    assert!(symetries().iter().any(|(_, s)| (0..4).all(|k| s(k, 2 * k - 1) == reflechir(k, 2 * k - 1))));
}

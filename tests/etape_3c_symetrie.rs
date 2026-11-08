//! Étape 3c : la symétrie d'ordre 6 et la symétrie miroir restent EXACTES avec le bord circulaire (`distance2` est invariante par les douze symétries de
//! l'hexagone, donc le bord aussi), et le modèle est équivariant sous ces symétries. Comme `etape_3_symetrie`, avec `Bord::Circulaire` et des
//! rayons où il diffère de la couronne (`R >= 8`). Valeurs attendues : `references/etape-3-valeurs.md`, identifiants `V-BORD-xx`.
//!
//! Les fautes que ces tests voient : un bord défini par `q² - q r + r²` ou `q² + r²` (non symétrique) ; une mise à jour qui dépend du sens ; un
//! voisinage faussé aux cases du bord circulaire.

mod etape_3_commun;
mod etape_3c_commun;

use etape_3_commun::{brut_de, cases, masses_formule, parametres, symetries};
use givre_lab::flocon::{Bord, Flocon};
use givre_lab::math::Fixe;

/// Un état posé à la main : (rayon, α, β, γ, formule des masses `(a, b, c, d)`, cases gelées posées).
type PoseALaMain = (u32, &'static str, &'static str, &'static str, (i64, i64, i64, i64), &'static [(i32, i32)]);

/// juste : depuis le germe central, avec le bord circulaire, la symétrie est exacte à chaque pas (V-BORD-20). Sept jeux `(α, β, γ)` sur `R = 15` (bord
/// circulaire de 126 cases contre 90 pour la couronne), 60 pas, et deux dendrites fines sur `R = 40`, 300 pas : à chaque pas `defaut_rotation()` et
/// `defaut_reflexion()` valent 0, et à la fin chacune des **douze** symétries de l'hexagone laisse la masse de chaque case inchangée. Le test
/// n'est pas vide : pour tous les jeux avec `β > 0`, la vapeur atteint le bord et l'état final diffère de celui du bord hexagonal (le test le
/// vérifie, il recalcule le même jeu avec `Bord::Hexagonal`). Un bord non symétrique (`q² - q r + r²`) casse l'égalité des `brut`.
///
/// Ne couvre pas : les états de départ sans symétrie (voir `equivariance_a_bord_circulaire`).
#[test]
#[cfg_attr(not(feature = "etape-3-bord"), ignore)]
fn symetrie_exacte_a_bord_circulaire() {
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
        let mut f = Flocon::avec_bord(rayon, parametres(a, b, g), Bord::Circulaire);
        let mut hexagonal = Flocon::avec_bord(rayon, parametres(a, b, g), Bord::Hexagonal);
        for t in 1..=pas {
            f.pas();
            assert_eq!(f.defaut_rotation(), 0, "R={rayon} alpha {a} beta {b} gamma {g} pas {t}");
            assert_eq!(f.defaut_reflexion(), 0, "R={rayon} alpha {a} beta {b} gamma {g} pas {t}");
        }
        hexagonal.avancer(pas);
        if b != "0" {
            assert_ne!(f.masses(), hexagonal.masses(), "R={rayon} alpha {a} beta {b} gamma {g} : le bord compte");
        }
        for (nom, s) in symetries() {
            let defaut = cases(rayon as i32)
                .iter()
                .filter(|&&(q, r)| {
                    let (x, y) = s(q, r);
                    f.masse(q, r) != f.masse(x, y)
                })
                .count();
            assert_eq!(defaut, 0, "R={rayon} alpha {a} beta {b} gamma {g} : {nom}");
        }
    }
}

/// juste : le modèle à bord circulaire est équivariant sous les douze symétries de l'hexagone (V-BORD-21) : pour un état `S` sans symétrie et toute
/// symétrie `g`, faire un pas sur `g(S)` (la masse de `c` posée sur `g(c)`) donne `g` de l'état d'un pas sur `S`, **exactement**, à chaque pas. Deux
/// états sans symétrie posés par une formule sur `R = 9` et `R = 14` (cinq cases gelées posées sans symétrie), douze symétries, trois pas : 72
/// comparaisons de tout l'état (le test vérifie aussi que l'état à bord circulaire diffère de celui du bord hexagonal après trois pas : il n'est
/// pas vide). C'est la propriété que la symétrie du germe ne voit pas : un bord circulaire qui n'est pas stable par toutes les
/// symétries (par exemple `q² - q r + r²`) la viole, un germe symétrique n'y verrait rien.
///
/// Ne couvre pas : les valeurs numériques d'un pas (voir `etape_3c_pas`).
#[test]
#[cfg_attr(not(feature = "etape-3-bord"), ignore)]
fn equivariance_a_bord_circulaire() {
    let cas: [PoseALaMain; 2] = [
        (9, "0.9", "0.35", "0.02", (37, 11, 97, 120), &[(7, 0), (-3, 5), (-8, 0), (0, 0), (0, -9)]),
        (14, "1", "0.4", "0.05", (53, 7, 89, 110), &[(12, 0), (-6, 10), (-13, 0), (7, 7), (5, -9)]),
    ];
    for (rayon, a, b, g, (fa, fb, fc, fd), gel) in cas {
        let p = parametres(a, b, g);
        let surcharges: Vec<((i32, i32), Fixe)> = gel.iter().map(|&c| (c, Fixe::UN)).collect();
        let brut = brut_de(&masses_formule(rayon as i32, fa, fb, fc, fd, &surcharges));
        let toutes = cases(rayon as i32);
        let indice = |c: (i32, i32)| toutes.iter().position(|&x| x == c).expect("dans l'hexagone");
        let depuis = |brut: &[i64]| {
            let masses: Vec<Fixe> = brut.iter().map(|&b| Fixe(b)).collect();
            Flocon::avec_bord_et_masses(rayon, p, Bord::Circulaire, &masses)
        };
        // le test n'est pas vide : avec le bord hexagonal le même état évolue autrement
        let mut hexagonal =
            Flocon::avec_bord_et_masses(rayon, p, Bord::Hexagonal, &brut.iter().map(|&b| Fixe(b)).collect::<Vec<_>>());
        let mut rond = depuis(&brut);
        hexagonal.avancer(3);
        rond.avancer(3);
        assert_ne!(hexagonal.masses(), rond.masses(), "R={rayon} : le bord circulaire compte");
        for (nom, s) in symetries() {
            let mut transforme = vec![0_i64; brut.len()];
            for (i, &(q, r)) in toutes.iter().enumerate() {
                transforme[indice(s(q, r))] = brut[i];
            }
            let mut d_abord = depuis(&brut);
            let mut ensuite = depuis(&transforme);
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

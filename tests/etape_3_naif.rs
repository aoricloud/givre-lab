//! Étape 3 : le simulateur NAÏF du test (`etape_3_commun::Naif`, écrit d'après la doc de `flocon` : un dictionnaire de cases, des
//! entiers `i128`, des arrondis écrits à la main, les voisins dans un autre ordre) doit donner **exactement** les états de
//! `Flocon`, case par case, à chaque pas, pour plusieurs jeux de paramètres, plusieurs rayons et plusieurs états de départ. C'est
//! le test le plus fort contre une faute plausible : il fait de la doc, et non d'un prototype, la spécification. Les empreintes
//! de plusieurs trajectoires ont en plus été calculées par Python (`modele_independant.py`). Valeurs attendues :
//! `references/etape-3-valeurs.md`, identifiants `V-NAIF-xx`.

mod etape_3_commun;

use etape_3_commun::valeurs::{ASYM_AVANT, P1_AVANT, P2_AVANT, P3_EMPREINTE_20, TRAJECTOIRES};
use etape_3_commun::{Naif, brut_de, parametres};
use givre_lab::flocon::{Flocon, Parametres};
use givre_lab::math::Fixe;

/// Fait avancer `Flocon` et le naïf pas à pas depuis le même état et vérifie l'égalité des masses à chaque pas (et du temps).
fn comparer(mut f: Flocon, mut naif: Naif, pas: u64, etiquette: &str) {
    assert_eq!(brut_de(&f.masses()), naif.brut(), "{etiquette} : état de départ");
    for t in 1..=pas {
        f.pas();
        naif.pas();
        assert_eq!(f.temps(), t);
        assert_eq!(naif.temps, t);
        assert_eq!(brut_de(&f.masses()), naif.brut(), "{etiquette} : pas {t}");
    }
}

fn depuis_brut(rayon: u32, p: Parametres, brut: &[i64]) -> Flocon {
    let masses: Vec<Fixe> = brut.iter().map(|&b| Fixe(b)).collect();
    Flocon::depuis_masses(rayon, p, &masses)
}

/// juste : depuis l'état initial, sur sept jeux `(α, β, γ)` (diffusion forte et faible, `β` de 0 à 0.8, `γ` de 0 à 0.3, `α = 2`) et des
/// rayons de 5 à 9, 40 à 70 pas : tous les `brut` de toutes les cases sont ceux du naïf, à chaque pas (V-NAIF-01). Les dynamiques
/// couvertes : un germe qui ne grandit pas, des plaques, des étoiles, des branches ; la couronne atteinte pour certains jeux.
///
/// Ne couvre pas : les états de départ sans symétrie (voir `naif_depuis_etats_sans_symetrie`).
#[test]
#[cfg_attr(not(feature = "etape-3"), ignore)]
fn naif_depuis_l_etat_initial() {
    let jeux = [
        (5, "0.7", "0.3", "0.05", 40),
        (6, "1", "0.4", "0.001", 60),
        (5, "2", "0.2", "0.3", 40),
        (6, "0.5", "0", "0.2", 50),
        (7, "1.5", "0.55", "0.0001", 50),
        (8, "1", "0.45", "0.1", 60),
        (9, "0.3", "0.8", "0.002", 70),
    ];
    for (rayon, a, b, g, pas) in jeux {
        let p = parametres(a, b, g);
        comparer(
            Flocon::new(rayon, p),
            Naif::initial(rayon as i32, p),
            pas,
            &format!("R={rayon} alpha {a} beta {b} gamma {g}"),
        );
    }
}

/// juste : depuis trois états de départ sans symétrie, posés par `depuis_masses` (V-NAIF-02) : `P1_AVANT` (R=2, deux cases gelées dont
/// une sur la couronne), `P2_AVANT` (R=3) et `ASYM_AVANT` (R=6, quatre cases gelées dont une à `1 + 1 ulp`), 40 pas chacun : égalité
/// avec le naïf case par case à chaque pas. Les voisins d'une case de la couronne, la couronne qui n'est pas à `β` au départ, un
/// voisinage à six voisins mais mal orienté échouent ici.
///
/// Ne couvre pas : l'équivariance (voir `etape_3_symetrie`).
#[test]
#[cfg_attr(not(feature = "etape-3"), ignore)]
fn naif_depuis_etats_sans_symetrie() {
    let cas: [(u32, &str, &str, &str, &[i64]); 3] = [
        (2, "0.7", "0.3", "0.05", &P1_AVANT),
        (3, "2", "0.5", "0.25", &P2_AVANT),
        (6, "0.9", "0.35", "0.02", &ASYM_AVANT),
    ];
    for (rayon, a, b, g, brut) in cas {
        let p = parametres(a, b, g);
        comparer(
            depuis_brut(rayon, p, brut),
            Naif::depuis_brut(rayon as i32, p, brut),
            40,
            &format!("R={rayon} alpha {a} beta {b} gamma {g}"),
        );
    }
}

/// juste : des trajectoires dont l'empreinte (SHA-256 de l'écriture canonique de la doc) a été calculée par Python, l'outil
/// indépendant, depuis l'état initial (V-NAIF-03) : 6 jeux, 25 à 60 pas ; avec le nombre de cases gelées, le rayon du cristal et le
/// nombre de pointes à la fin. Et `P3_EMPREINTE_20` : `R=3`, `α=1`, `β=0.45`, `γ=0.1`, 20 pas.
///
/// Ne couvre pas : les états de départ sans symétrie.
#[test]
#[cfg_attr(not(feature = "etape-3"), ignore)]
fn trajectoires_calculees_par_python() {
    for &(rayon, a, b, g, pas, empreinte, gelees, rayon_cristal, pointes) in TRAJECTOIRES {
        let mut f = Flocon::new(rayon, parametres(a, b, g));
        f.avancer(pas);
        assert_eq!(f.empreinte(), empreinte, "R={rayon} alpha {a} beta {b} gamma {g} après {pas} pas");
        assert_eq!(f.gelees(), gelees);
        assert_eq!(f.rayon_cristal(), rayon_cristal);
        assert_eq!(f.pointes(), pointes);
    }
    let mut f = Flocon::new(3, parametres("1", "0.45", "0.1"));
    f.avancer(20);
    assert_eq!(f.empreinte(), P3_EMPREINTE_20);
}

//! Étape G1 : des propriétés QUALITATIVES vraies à tout réglage permis (les invariants, la permanence du rattachement, la connexité, la vitesse maximale du cristal, ce qui
//! ne nait jamais) ; aucune ne prétend à la forme d'un vrai flocon. Puis un test « régression » : un cristal du jeu de la figure 9 de l'article à petit rayon.
//! Valeurs attendues : `references/etape-g1-valeurs.md`, identifiants `V-G1-REG-xx`.

mod etape_g1_commun;

use etape_g1_commun::valeurs::{CRISTAL_R30, CRISTAL_R30_EMPREINTE, CRISTAL_R30_MASSE};
use etape_g1_commun::*;
use givre_lab::neige::Neige;
use std::collections::BTreeSet;

/// Vérifie (I1), (I2), (I3) sur l'état courant avec des voisins calculés ici ; panique sinon.
fn verifier_les_invariants(neige: &Neige, etiquette: &str) {
    let rayon = neige.rayon() as i32;
    for (q, r) in cases(rayon) {
        let c = neige.case(q, r).expect("case");
        assert!(c.b.0 >= 0 && c.c.0 >= 0 && c.d.0 >= 0, "{etiquette} : masse négative en ({q}, {r})");
        if c.a {
            assert!(c.b.0 == 0 && c.d.0 == 0, "{etiquette} : la case rattachée ({q}, {r}) a b ou d non nul : {c:?}");
        } else {
            let touche = DIRECTIONS.iter().any(|&(dq, dr)| neige.case(q + dq, r + dr).is_some_and(|v| v.a));
            if !touche {
                assert!(
                    c.b.0 == 0 && c.c.0 == 0,
                    "{etiquette} : la case lointaine ({q}, {r}) a b ou c non nul : {c:?}"
                );
            }
        }
    }
}

/// Les cases rattachées, par un parcours en largeur depuis l'origine à travers les cases rattachées seulement : doit être toutes.
fn composante_de_l_origine(neige: &Neige) -> usize {
    let mut vues: BTreeSet<(i32, i32)> = BTreeSet::new();
    let mut a_voir = vec![(0, 0)];
    vues.insert((0, 0));
    while let Some((q, r)) = a_voir.pop() {
        for &(dq, dr) in &DIRECTIONS {
            let y = (q + dq, r + dr);
            if neige.case(y.0, y.1).is_some_and(|c| c.a) && vues.insert(y) {
                a_voir.push(y);
            }
        }
    }
    vues.len()
}

/// juste : à chaque pas de dix suites (réglages de l'article, d'essai et extrêmes permis, des rayons de 4 à 10, 100 à 200 pas), depuis le germe (V-G1-REG-01) : (I1) une case
/// rattachée n'a ni `b` ni `d`, (I2) une case lointaine n'a ni `b` ni `c`, (I3) aucune masse n'est négative ; le rattachement est PERMANENT et la case rattachée ne change PLUS
/// (son contenu `(a, b, c, d)` est le même au pas suivant : « plus de dynamique aux cases rattachées ») ; le nombre de cases rattachées et la masse du cristal ne décroissent
/// jamais ; les cases rattachées forment une seule composante connexe contenant l'origine (une case ne se rattache qu'à côté du cristal) ; le rayon du cristal croît d'au plus
/// 1 par pas ; et, par la symétrie, le nombre de cases rattachées est congru à 1 modulo 6 (les orbites des cases non centrales ont 6 ou 12 éléments). Une fonte qui touche une case qui
/// vient de se rattacher, un rattachement qui lit les `a` du pas courant, un gel qui oublie une case de bord, une vapeur qui reste sur une case rattachée échouent ici.
///
/// Ne couvre pas : la forme du cristal ; une propriété qui ne serait vraie que pour certains réglages (voir les tests qui suivent, chacun avec sa condition).
#[test]
#[cfg_attr(not(feature = "etape-g1"), ignore)]
fn invariants_permanence_et_vitesse_a_chaque_pas() {
    let jeux: [(u32, Jeu, u64); 10] = [
        (10, FIG9_RHO_050, 200),
        (8, FIG9_RHO_040, 200),
        (8, FIG13_MILIEU, 200),
        (8, JEU_B, 200),
        (6, JEU_C, 100),
        (8, JEU_D, 150),
        (6, JEU_E, 100),
        (4, ["1", "1", "0", "8", "0", "0", "0"], 100),
        (4, ["1", "8", "1", "8", "1", "1", "1"], 100),
        (5, ["0.7", "1", "0.5", "1", "0.5", "0.5", "1"], 100),
    ];
    let mut a_grandi = 0;
    for (rayon, jeu, pas) in jeux {
        let mut neige = Neige::essayer_new(rayon, p(jeu)).expect("neige valide");
        let etiquette = format!("R={rayon} {jeu:?}");
        for t in 1..=pas {
            let avant = neige.cases();
            let (rattachees, cristal, rayon_cristal) =
                (neige.cases_rattachees(), neige.masse_du_cristal(), neige.rayon_cristal());
            neige.pas();
            let apres = neige.cases();
            for (i, (x, y)) in avant.iter().zip(&apres).enumerate() {
                if x.a {
                    assert_eq!(x, y, "{etiquette} pas {t} : la case rattachée d'indice {i} a changé");
                }
            }
            assert!(
                neige.cases_rattachees() >= rattachees,
                "{etiquette} pas {t} : le nombre de cases rattachées décroît"
            );
            assert!(neige.masse_du_cristal() >= cristal, "{etiquette} pas {t} : la masse du cristal décroît");
            assert!(
                neige.rayon_cristal() <= rayon_cristal + 1,
                "{etiquette} pas {t} : le cristal grandit de plus d'une case en un pas"
            );
            assert_eq!(
                (neige.cases_rattachees() - 1) % 6,
                0,
                "{etiquette} pas {t} : {} cases rattachées",
                neige.cases_rattachees()
            );
            verifier_les_invariants(&neige, &format!("{etiquette} pas {t}"));
            if t % 10 == 0 || t == pas {
                assert_eq!(
                    composante_de_l_origine(&neige),
                    neige.cases_rattachees(),
                    "{etiquette} pas {t} : le cristal n'est pas connexe"
                );
            }
        }
        a_grandi += usize::from(neige.cases_rattachees() > 7);
    }
    assert!(a_grandi >= 7, "trop peu de suites où le cristal a vraiment grandi : {a_grandi}");
}

/// juste : sans vapeur, rien ne nait (V-G1-REG-02) : `ρ = 0` (et `β >= 1`, la plage permise), pour quatre jeux des autres réglages (dont `α = 0, θ = 8` et tous les réglages à 1), `R = 5`,
/// 100 pas : l'état reste EXACTEMENT l'état de départ (le germe seul, `c = 1` ; tout le reste nul), la masse totale reste 1.
///
/// Ne couvre pas : un état sans germe (voir `sans_germe_rien_ne_nait_et_la_vapeur_se_moyenne`).
#[test]
#[cfg_attr(not(feature = "etape-g1"), ignore)]
fn sans_vapeur_rien_ne_nait() {
    let jeux: [Jeu; 4] = [
        ["0", "1", "0", "8", "0", "0", "0"],
        ["0", "1.3", "0.08", "0.025", "0.003", "0.07", "0.00005"],
        ["0", "1", "1", "8", "1", "1", "1"],
        ["0", "8", "0", "0", "0.5", "0.5", "0.5"],
    ];
    for jeu in jeux {
        let mut neige = Neige::essayer_new(5, p(jeu)).expect("neige valide");
        let depart = neige.cases();
        neige.avancer(100);
        assert_eq!(neige.cases(), depart, "{jeu:?}");
        assert_eq!(neige.masse_totale(), UN);
        assert_eq!(neige.cases_rattachees(), 1);
    }
}

/// juste : sans germe, rien ne nait et la vapeur se moyenne (V-G1-REG-03) : un état sans aucune case rattachée et de la vapeur quelconque (tirée par un congruentiel linéaire, graines 1 à
/// 10, `R = 5`), 60 pas, quatre jeux : aucune case ne se rattache jamais, `b` et `c` restent nuls (il n'y a aucune case de bord), et la vapeur reste entre son minimum et son
/// maximum de départ (chaque nouvelle vapeur est un arrondi de moyenne de valeurs de cet intervalle) ; la masse reste conservée à la borne de l'arrondi ; et
/// la vapeur s'est vraiment moyennée : après 60 pas l'étendue (maximum - minimum) est sous le quart de celle du départ (mesuré par le juge sur 200 graines : au
/// plus 0,065 fois l'étendue de départ, 0,025 en moyenne ; la borne 1/4 laisse un facteur 3,9). Un pas qui ne ferait rien échoue ici.
///
/// Ne couvre pas : une vapeur qui rencontre un cristal.
#[test]
#[cfg_attr(not(feature = "etape-g1"), ignore)]
fn sans_germe_rien_ne_nait_et_la_vapeur_se_moyenne() {
    let jeux = [FIG9_RHO_050, JEU_B, JEU_C, ["1", "1", "0", "8", "1", "1", "1"]];
    for graine in 1..=10_u64 {
        let jeu = jeux[(graine as usize) % jeux.len()];
        let mut g = Lcg(graine);
        let mut etat = etat_vide(5);
        for v in etat.values_mut() {
            v[3] = g.jusqu_a(UN as u64) as i128;
        }
        let (mut neige, _) = depuis_un_etat(5, &p(jeu), &etat);
        let (mini, maxi) = (etat.values().map(|v| v[3]).min().unwrap(), etat.values().map(|v| v[3]).max().unwrap());
        let depart = neige.masse_totale();
        assert_eq!(neige.cases().len(), 91);
        assert_eq!(depart, etat.values().map(|v| v[3]).sum::<i128>(), "graine {graine} : masse de départ");
        for t in 1..=60_i128 {
            neige.pas();
            assert_eq!(neige.cases_rattachees(), 0, "graine {graine} pas {t}");
            for c in neige.cases() {
                assert_eq!((c.b.0, c.c.0), (0, 0));
                assert!(
                    i128::from(c.d.0) >= mini && i128::from(c.d.0) <= maxi,
                    "graine {graine} pas {t} : vapeur {} hors de [{mini}, {maxi}]",
                    c.d.0
                );
            }
            assert!(7 * (neige.masse_totale() - depart).abs() <= 3 * 91 * t, "graine {graine} pas {t}");
        }
        // La vapeur s'est vraiment moyennée : l'étendue (maximum - minimum) est tombée sous le quart de celle du départ.
        let vapeurs: Vec<i128> = neige.cases().iter().map(|c| i128::from(c.d.0)).collect();
        let etendue = vapeurs.iter().max().unwrap() - vapeurs.iter().min().unwrap();
        assert!(
            4 * etendue < maxi - mini,
            "graine {graine} : étendue {etendue} après 60 pas, {} au départ",
            maxi - mini
        );
        assert_eq!(neige.temps(), 60);
    }
}

/// juste : avec `κ = 1` (toute la vapeur du bord gèle d'un coup, rien ne devient masse de bord), le germe ne grandit JAMAIS (V-G1-REG-04) : pour six jeux des autres réglages (dont `α = 0`,
/// `θ = 8`, `β = 1`), `R = 6`, 300 pas : `b = 0` partout à tout instant, une seule case rattachée (il faudrait `b >= β >= 1` avec `n <= 2`), et la glace s'accumule sur les six voisines
/// du germe (leur `c` est non nul). Un `κ` lu pour `1 - κ` (donc toute la vapeur devient masse de bord et le cristal grandit) échoue ici.
///
/// Ne couvre pas : `κ` strictement entre 0 et 1.
#[test]
#[cfg_attr(not(feature = "etape-g1"), ignore)]
fn kappa_un_et_le_germe_ne_grandit_pas() {
    let jeux: [Jeu; 6] = [
        ["0.5", "1.3", "0.08", "0.025", "1", "0.07", "0.00005"],
        ["1", "1", "0", "8", "1", "0", "0"],
        ["1", "1", "1", "8", "1", "1", "1"],
        ["0.9", "1.05", "0.5", "0.5", "1", "0.05", "0.001"],
        ["0.3", "2", "0.2", "0.1", "1", "0.5", "0"],
        ["0.7", "1", "0", "0", "1", "0.2", "0.2"],
    ];
    for jeu in jeux {
        let mut neige = Neige::essayer_new(6, p(jeu)).expect("neige valide");
        for t in 1..=300 {
            neige.pas();
            assert!(neige.cases().iter().all(|c| c.b.0 == 0), "{jeu:?} pas {t} : b non nul");
            assert_eq!(neige.cases_rattachees(), 1, "{jeu:?} pas {t}");
        }
        let c_voisines: i64 = DIRECTIONS.iter().map(|&(q, r)| neige.case(q, r).expect("voisine").c.0).sum();
        assert!(c_voisines > 0 || jeu[6] == "1", "{jeu:?} : la glace ne s'accumule pas sur les voisines du germe");
    }
}

/// juste : avec `κ = 0` (rien ne gèle d'un coup), il n'y a jamais de glace hors du cristal (V-G1-REG-05) : `c = 0` sur toute case NON rattachée, à chaque pas, pour cinq jeux (dont les jeux de l'article
/// avec `κ = 0`), `R = 8`, 200 pas ; et le cristal grandit quand même. Un `κ` lu pour `1 - κ` échoue ici.
///
/// Ne couvre pas : `κ > 0`.
#[test]
#[cfg_attr(not(feature = "etape-g1"), ignore)]
fn kappa_zero_pas_de_glace_hors_du_cristal() {
    let jeux: [Jeu; 5] = [
        ["0.5", "1.3", "0.08", "0.025", "0", "0.07", "0.00005"],
        ["0.4", "1.3", "0.08", "0.025", "0", "0.07", "0.00005"],
        ["0.9", "1.05", "0.5", "0.5", "0", "0.05", "0.001"],
        ["0.6", "1.1", "0.3", "0.1", "0", "0.5", "1"],
        ["1", "1", "0", "8", "0", "0", "0"],
    ];
    for jeu in jeux {
        let mut neige = Neige::essayer_new(8, p(jeu)).expect("neige valide");
        for t in 1..=200 {
            neige.pas();
            assert!(neige.cases().iter().all(|c| c.a || c.c.0 == 0), "{jeu:?} pas {t} : de la glace hors du cristal");
        }
        assert!(neige.cases_rattachees() > 1, "{jeu:?} : le cristal n'a pas grandi");
    }
}

/// juste : sans fonte (`μ = γ = 0`), la masse gelée `Σ (b + c)` sur toutes les cases ne décroît JAMAIS, exactement, à chaque pas (V-G1-REG-06) : le gel ne fait que prendre à la vapeur, le
/// rattachement déplace `b` dans `c`, rien ne rend à la vapeur. Quatre jeux (`FIG9_RHO_050`, `JEU_C`, `JEU_D` et un jeu à `κ = 0,5`), `R = 8`, 200 pas. Une fonte qui s'applique malgré `μ = 0`,
/// un `b` perdu au rattachement échouent ici.
///
/// Ne couvre pas : la fonte non nulle (aucune monotonie n'y est vraie en général).
#[test]
#[cfg_attr(not(feature = "etape-g1"), ignore)]
fn sans_fonte_la_masse_gelee_ne_decroit_pas() {
    let jeux: [Jeu; 4] = [
        ["0.5", "1.3", "0.08", "0.025", "0.003", "0", "0"],
        ["0.9", "1.05", "0.5", "0.5", "0.1", "0", "0"],
        ["0.65", "1.75", "0.2", "0.026", "0.15", "0", "0"],
        ["0.8", "1.2", "0.4", "0.3", "0.5", "0", "0"],
    ];
    for jeu in jeux {
        let mut neige = Neige::essayer_new(8, p(jeu)).expect("neige valide");
        let gelee = |n: &Neige| -> i128 { n.cases().iter().map(|c| i128::from(c.b.0) + i128::from(c.c.0)).sum() };
        let mut avant = gelee(&neige);
        for t in 1..=200 {
            neige.pas();
            let apres = gelee(&neige);
            assert!(apres >= avant, "{jeu:?} pas {t} : la masse gelée décroît de {avant} à {apres}");
            avant = apres;
        }
        assert!(neige.cases_rattachees() > 1, "{jeu:?} : le cristal n'a pas grandi");
    }
}

/// régression : un cristal du jeu de la figure 9 de l'article à petit rayon (V-G1-REG-07). `R = 30`, `ρ = 0,5`, `β = 1,3`, `α = 0,08`, `θ = 0,025`, `κ = 0,003`, `μ = 0,07`, `γ = 0,00005`
/// (annexe 8 de Gravner et Griffeath, ligne « 9 », et `ρ` de la figure 9 : `FIG9_RHO_050`), de 100 en 100 pas jusqu'à 500 : le nombre de cases rattachées, le rayon du cristal et la masse du cristal
/// sont ceux calculés par le code Python indépendant (109, 235, 397, 523, 709 cases ; rayons 6, 10, 13, 16, 19 ; la masse du cristal croît strictement) ; l'empreinte et la masse totale à
/// 500 pas aussi ; le cristal a SIX BRANCHES : les six cases à `rayon_cristal()` dans les six directions de la grille sont rattachées (la symétrie est prouvée par ailleurs) ; il est loin du
/// mur (`rayon_cristal < R`). **C'est une régression de NOTRE code, avec un deuxième calcul indépendant : ce n'est PAS la preuve que ce cristal ressemble à la figure 9** (qui, à plus grande
/// taille et plus de pas, montre une dendrite ; l'article ne donne aucun nombre pour ses figures).
///
/// Ne couvre pas : la forme ; ni la ressemblance à l'article ; ni un autre jeu de réglages ; ni une taille d'article (400 à 600 cases de diamètre).
#[test]
#[cfg_attr(not(feature = "etape-g1"), ignore)]
fn cristal_du_jeu_de_la_figure_9_a_petit_rayon() {
    let mut neige = Neige::essayer_new(30, p(FIG9_RHO_050)).expect("neige valide");
    let mut cristal_avant = 0;
    for &(pas, rattachees, rayon_cristal, masse_cristal) in &CRISTAL_R30 {
        neige.avancer(pas - neige.temps());
        assert_eq!(neige.temps(), pas);
        assert_eq!(neige.cases_rattachees(), rattachees, "pas {pas}");
        assert_eq!(neige.rayon_cristal(), rayon_cristal, "pas {pas}");
        assert_eq!(neige.masse_du_cristal(), masse_cristal, "pas {pas}");
        assert!(neige.masse_du_cristal() > cristal_avant, "la masse du cristal doit croître : pas {pas}");
        cristal_avant = neige.masse_du_cristal();
        assert!(neige.rayon_cristal() < 30, "le cristal ne doit pas toucher le mur");
    }
    assert_eq!(neige.empreinte(), CRISTAL_R30_EMPREINTE);
    assert_eq!(neige.masse_totale(), CRISTAL_R30_MASSE);
    let l = neige.rayon_cristal() as i32;
    for (k, &(dq, dr)) in DIRECTIONS.iter().enumerate() {
        assert!(
            neige.case(l * dq, l * dr).expect("case").a,
            "la pointe de la branche {k} (distance {l}) n'est pas rattachée"
        );
    }
}

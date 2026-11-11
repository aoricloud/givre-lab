//! Étape G1 : sans bruit, les douze symétries de l'hexagone (six rotations, six miroirs) laissent l'état invariant AU BIT PRÈS, à chaque pas, et le modèle
//! est équivariant : le pas d'un état tourné est l'état tourné du pas. Les symétries sont écrites dans le test (`etape_g1_commun::symetries`), pas déduites de
//! l'empreinte. Valeurs attendues : `references/etape-g1-valeurs.md`, identifiants `V-G1-SYM-xx`.

mod etape_g1_commun;

use etape_g1_commun::*;
use givre_lab::neige::Neige;

/// Compte les cases dont le contenu (`a`, `b`, `c`, `d`) diffère de celui de leur image par `g`.
fn defaut(neige: &Neige, g: Symetrie) -> usize {
    cases(neige.rayon() as i32).iter().filter(|&&(q, r)| neige.case(q, r) != neige.case(g(q, r).0, g(q, r).1)).count()
}

/// juste : depuis le germe, la symétrie est exacte à CHAQUE pas (V-G1-SYM-01). Sept jeux de réglages (dont trois de l'article), des rayons de 10 à 14, 60 à 120 pas :
/// à chaque pas, pour chacune des douze symétries de l'hexagone (une rotation de 60°, ses puissances, et leur composée avec la réflexion `(q, r) -> (r, q)`, écrites dans le test),
/// le contenu `(a, b, c, d)` de chaque case est celui de son image, au bit près ; et le cristal a bien grandi (il ne s'agit pas de la symétrie d'un germe seul). Un voisinage
/// faussé (même à six voisins), un ordre de mise à jour qui dépend de la case, une mise à jour en place d'un sous-pas, un mur qui ne réfléchit pas symétriquement cassent
/// l'égalité.
///
/// Ne couvre pas : un état de départ sans symétrie (voir `equivariance_depuis_un_etat_sans_symetrie`).
#[test]
#[cfg_attr(not(feature = "etape-g1"), ignore)]
fn symetrie_exacte_depuis_le_germe() {
    let jeux: [(u32, Jeu, u64); 7] = [
        (14, FIG9_RHO_050, 80),
        (12, FIG9_RHO_040, 120),
        (10, FIG13_MILIEU, 120),
        (12, JEU_B, 100),
        (12, JEU_C, 60),
        (10, JEU_D, 80),
        (10, JEU_E, 60),
    ];
    for (rayon, jeu, pas) in jeux {
        let mut neige = Neige::essayer_new(rayon, p(jeu)).expect("neige valide");
        for t in 1..=pas {
            neige.pas();
            for (nom, g) in symetries() {
                assert_eq!(defaut(&neige, g), 0, "R={rayon} {jeu:?} pas {t} : {nom}");
            }
        }
        assert!(neige.cases_rattachees() > 1, "R={rayon} {jeu:?} : le cristal n'a pas grandi en {pas} pas");
    }
}

/// juste : le modèle est ÉQUIVARIANT (V-G1-SYM-02) : pour des états de départ sans symétrie, tirés par un congruentiel linéaire (graines 1 à 24, rayon 4 à 6, des
/// réglages variés), et chacune des douze symétries `g`, la neige du `g(état)` après un, deux et trois pas est exactement le `g` de la neige de l'état après autant de pas
/// (case par case, `(a, b, c, d)`). Une voisine oubliée ou mal orientée (une direction de la grille échangée), un voisinage non symétrique, un germe décalé, un ordre des
/// voisines qui compte, une règle de rattachement qui dépend d'une direction échouent ici même quand le cristal n'a aucune symétrie.
///
/// L'état doit avoir changé en trois pas (un pas qui ne ferait rien serait équivariant à vide).
///
/// Ne couvre pas : un pas long ; la symétrie propre du germe (voir `symetrie_exacte_depuis_le_germe`).
#[test]
#[cfg_attr(not(feature = "etape-g1"), ignore)]
fn equivariance_depuis_un_etat_sans_symetrie() {
    let jeux = [FIG9_RHO_050, FIG13_MILIEU, JEU_B, JEU_C, JEU_D, JEU_E];
    for graine in 1..=24_u64 {
        let rayon = 4 + (graine % 3) as u32;
        let jeu = jeux[(graine as usize) % jeux.len()];
        let etat = etat_au_hasard(rayon as i32, graine, false);
        let (droit, _) = depuis_un_etat(rayon, &p(jeu), &etat);
        let mut asymetrique = false;
        for (nom, g) in symetries() {
            let (mut tourne, _) = depuis_un_etat(rayon, &p(jeu), &image_de_l_etat(&etat, g));
            let mut reference = droit.clone();
            for t in 1..=3 {
                reference.pas();
                tourne.pas();
                for &(q, r) in &cases(rayon as i32) {
                    let (gq, gr) = g(q, r);
                    assert_eq!(
                        tourne.case(gq, gr),
                        reference.case(q, r),
                        "graine {graine} R={rayon} {jeu:?} : {nom}, pas {t}, case ({q}, {r})"
                    );
                }
            }
            asymetrique |= defaut(&reference, g) > 0;
            assert_ne!(
                reference.cases(),
                droit.cases(),
                "graine {graine} : trois pas n'ont rien changé, le test serait à vide"
            );
            assert_eq!(reference.temps(), 3);
        }
        assert!(asymetrique, "graine {graine} : l'état tiré est symétrique, le test serait à vide");
    }
}

/// juste : la symétrie est aussi exacte pour des réglages extrêmes permis et pour un cristal qui TOUCHE le mur (V-G1-SYM-03) : `R = 3`, 150 pas, quatre jeux
/// (dont `α = 0, θ = 8, κ = μ = γ = 0` : tout l'hexagone se rattache), chacun : défaut de symétrie nul à chaque pas pour les douze symétries ; au moins un cristal atteint la couronne (`rayon_cristal() = R`).
///
/// Ne couvre pas : la symétrie d'un grand cristal (voir `symetrie_exacte_depuis_le_germe`).
#[test]
#[cfg_attr(not(feature = "etape-g1"), ignore)]
fn symetrie_exacte_contre_le_mur() {
    let jeux: [Jeu; 4] = [
        JEU_C,
        ["1", "1", "0", "8", "0.2", "0.1", "0.01"],
        ["1", "1", "0", "8", "0", "0", "0"],
        ["1", "1", "0.3", "2", "0.02", "0.01", "0.0001"],
    ];
    let mut touche = false;
    for jeu in jeux {
        let mut neige = Neige::essayer_new(3, p(jeu)).expect("neige valide");
        for t in 1..=150 {
            neige.pas();
            for (nom, g) in symetries() {
                assert_eq!(defaut(&neige, g), 0, "{jeu:?} pas {t} : {nom}");
            }
        }
        touche |= neige.rayon_cristal() == 3;
    }
    assert!(touche, "au moins un jeu doit faire toucher le mur au cristal");
}

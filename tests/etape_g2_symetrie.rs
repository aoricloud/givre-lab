//! Étape G2 : les mesures et l'étiquette d'un cristal ne changent pas quand on applique à tout l'état une des douze symétries de l'hexagone (six rotations, six miroirs). Les symétries sont écrites dans le test
//! (`etape_g2_commun::symetries`), pas déduites d'une empreinte. Valeurs attendues : `references/etape-g2-valeurs.md`, identifiants `V-G2-SYM-xx`.

mod etape_g2_commun;

use etape_g2_commun::*;
use givre_lab::balayage::{classer, mesurer};
use givre_lab::math::Fixe;
use givre_lab::neige::{Case, Neige};

/// juste : pour des ensembles de cases rattachées SANS symétrie (tirés par un congruentiel : 60 ensembles épars de densité 15 à 90 %, rayon de domaine 4 à 11, qui touchent le mur ou non, et 40 cristaux connexes) et
/// pour chacune des douze symétries `g`, la neige de l'ensemble `g(A)` a EXACTEMENT les mêmes mesures et la même étiquette que celle de `A` (V-G2-SYM-01). Chaque ensemble tiré a au moins une image différente de lui-même
/// (le test ne serait pas à vide). Une mesure qui dépendrait de l'ordre des voisines, d'une direction (une voisine oubliée), de l'orientation de la grille ou de la case de départ d'un parcours échoue.
///
/// Ne couvre pas : l'équivariance du modèle (G1 : `etape_g1_symetrie`) ; des cristaux de G1 qui n'ont pas de symétrie (voir `mesures_invariantes_pour_des_cristaux_de_g1_sans_symetrie`).
#[test]
#[cfg_attr(not(feature = "etape-g2"), ignore)]
fn mesures_et_forme_invariantes_pour_des_ensembles_sans_symetrie() {
    let mut symetriques = 0;
    for graine in 1..=100_u64 {
        let domaine = 4 + (graine % 8) as u32;
        let ensemble = if graine <= 60 {
            ensemble_au_hasard(domaine as i32, 15 + (graine * 7) % 76, graine)
        } else {
            blob_au_hasard(domaine as i32, 3 + (graine % 6) as u32, 40 + (graine % 5) * 10, 500 + graine)
        };
        let neige = neige_de(domaine, &ensemble, fig9());
        let attendu = mesurer(&neige);
        // Le calcul naïf du test : une fonction constante serait invariante, mais pas égale au calcul naïf.
        assert_eq!(attendu, mesures_naives(&ensemble), "graine {graine}, domaine {domaine} : le calcul naïf");
        let mut une_image_differe = false;
        for (nom, g) in symetries() {
            let image = image(&ensemble, g);
            une_image_differe |= image != ensemble;
            let tournee = neige_de(domaine, &image, fig9());
            let obtenu = mesurer(&tournee);
            assert_eq!(obtenu, attendu, "graine {graine}, domaine {domaine} : {nom}");
            assert_eq!(
                obtenu,
                mesures_naives(&image),
                "graine {graine}, domaine {domaine} : {nom} (le calcul naïf de l'image)"
            );
            assert_eq!(classer(&obtenu), classer(&attendu), "graine {graine} : {nom} (étiquette)");
        }
        symetriques += usize::from(!une_image_differe);
    }
    // Un ensemble tiré peut être symétrique par hasard (un germe seul, un hexagone plein) : très peu, ou le test serait à vide.
    assert!(symetriques <= 3, "{symetriques} ensembles tirés sur 100 sont symétriques : le test serait à vide");
}

/// juste : les cristaux construits à la main et sans symétrie (un bras, deux composantes de tailles différentes, l'étoile à six bras d'où on retire une pointe, un anneau privé de deux cases voisines et une case
/// ajoutée hors de l'anneau) ont les mêmes mesures sous les douze symétries (V-G2-SYM-02), et les valeurs de l'étoile privée d'une pointe sont calculées à la main : `etoile(12)` sans la case `(12, 0)` a 72 cases, le
/// rayon 12 atteint encore par cinq pointes, une compacité `floor(72000 / 469) = 153` et, pour `pointes` comme pour `aiguilles`, 65 (la pointe retirée ôte une case à quatre voisines libres au moins, et la case d'avant, qui devient la pointe,
/// l'était déjà : `66 - 1 + 0`).
///
/// Ne couvre pas : des cristaux issus du modèle.
#[test]
#[cfg_attr(not(feature = "etape-g2"), ignore)]
fn mesures_invariantes_pour_des_formes_construites_asymetriques() {
    let mut etoile_privee = etoile(12);
    etoile_privee.remove(&(12, 0));
    let mut anneau_troue = anneau(7);
    anneau_troue.remove(&(7, 0));
    anneau_troue.remove(&(6, 1));
    anneau_troue.insert((2, 1));
    let deux: Ensemble = disque(3).into_iter().chain(disque(1).into_iter().map(|(q, r)| (8 + q, -4 + r))).collect();
    let formes: Vec<(&str, Ensemble)> = vec![
        ("bras", bras(9)),
        ("étoile privée d'une pointe", etoile_privee),
        ("anneau troué", anneau_troue),
        ("deux composantes", deux),
    ];
    for (nom, ensemble) in &formes {
        let attendu = mesurer(&neige_de(14, ensemble, fig9()));
        assert_eq!(attendu, mesures_naives(ensemble), "{nom} : le calcul naïf");
        for (symetrie, g) in symetries() {
            let tournee = neige_de(14, &image(ensemble, g), fig9());
            assert_eq!(mesurer(&tournee), attendu, "{nom} : {symetrie}");
            assert_eq!(classer(&mesurer(&tournee)), classer(&attendu), "{nom} : {symetrie} (étiquette)");
        }
    }
    let etoile_privee = &formes[1].1;
    let a_la_main = mesurer(&neige_de(14, etoile_privee, fig9()));
    assert_eq!((a_la_main.rattachees, a_la_main.rayon, a_la_main.compacite), (72, 12, 153));
    assert_eq!((a_la_main.pointes, a_la_main.aiguilles), (65, 65));
    // Cinq cases à la distance 12 sur les 72 de l'anneau : `floor(5000 / 72) = 69`.
    assert_eq!(a_la_main.hexagonalite, 69);
}

/// juste : des cristaux de G1 qui ne sont PAS symétriques (V-G2-SYM-03). On part d'un état sans symétrie (un petit amas de 5 à 12 cases rattachées tiré par un congruentiel, de la vapeur `ρ` ailleurs), on le tourne
/// ou on le retourne par chacune des douze symétries, on fait avancer chacun de 70 pas avec le modèle de G1 (réglages de la figure 9 ; un jeu dense à nous, 25 pas), et les mesures et l'étiquette du résultat sont les mêmes
/// pour les douze (le modèle de G1 est équivariant : ses états sont images les uns des autres ; les mesures doivent l'être de même). Six graines, deux jeux. Le cristal a grandi, et n'est pas symétrique.
///
/// Ne couvre pas : l'équivariance du modèle elle-même (G1).
#[test]
#[cfg_attr(not(feature = "etape-g2"), ignore)]
fn mesures_invariantes_pour_des_cristaux_de_g1_sans_symetrie() {
    for graine in 1..=6_u64 {
        for (jeu, p, pas) in [("figure 9", fig9(), 70), ("jeu dense", jeu_dense(), 25)] {
            let depart = blob_au_hasard(9, 2 + (graine % 3) as u32, 40, 90 + graine);
            let mut attendu = None;
            for (nom, g) in symetries() {
                let rattachees = image(&depart, g);
                let liste: Vec<Case> = cases(14)
                    .iter()
                    .map(|c| {
                        if rattachees.contains(c) {
                            Case { a: true, b: Fixe::ZERO, c: Fixe::UN, d: Fixe::ZERO }
                        } else {
                            Case { a: false, b: Fixe::ZERO, c: Fixe::ZERO, d: p.rho }
                        }
                    })
                    .collect();
                let mut neige = Neige::essayer_depuis_cases(14, p, &liste).expect("état valide");
                neige.avancer(pas);
                let obtenu = mesurer(&neige);
                assert_eq!(
                    obtenu,
                    mesures_naives(&rattachees_de(&neige)),
                    "graine {graine}, {jeu}, {nom} : le calcul naïf"
                );
                match attendu {
                    None => {
                        assert!(
                            obtenu.rattachees as usize > depart.len() + 10,
                            "graine {graine}, {jeu} : le cristal n'a pas grandi"
                        );
                        let a = rattachees_de(&neige);
                        assert!(
                            symetries().iter().any(|&(_, g2)| image(&a, g2) != a),
                            "graine {graine}, {jeu} : le cristal est symétrique"
                        );
                        attendu = Some(obtenu);
                    }
                    Some(reference) => {
                        assert_eq!(obtenu, reference, "graine {graine}, {jeu}, {nom}");
                        assert_eq!(classer(&obtenu), classer(&reference), "graine {graine}, {jeu}, {nom} (étiquette)");
                    }
                }
            }
        }
    }
}

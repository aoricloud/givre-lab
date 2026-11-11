//! Étape G2 : les cinq mesures d'un cristal (`balayage::mesurer`) sont des entiers EXACTS. Des cristaux construits à la main dont les valeurs sont calculées à la main (et recomptées par un outil
//! Python indépendant), un calcul naïf écrit dans le test sur des milliers d'ensembles tirés et sur des cristaux obtenus par `Neige::avancer`, et la propriété que seul `a` compte.
//! Valeurs attendues : `references/etape-g2-valeurs.md`, identifiants `V-G2-MES-xx` et `V-G2-NAIF-xx`.

mod etape_g2_commun;

use etape_g2_commun::*;
use givre_lab::balayage::{Mesures, mesurer};
use givre_lab::math::Fixe;
use givre_lab::neige::{Case, Neige};

/// Les mesures attendues, dans l'ordre `(rattachees, rayon, compacite, pointes, aiguilles, hexagonalite)`.
fn m(rattachees: u32, rayon: u32, compacite: u32, pointes: u32, aiguilles: u32, hexagonalite: u32) -> Mesures {
    Mesures { rattachees, rayon, compacite, pointes, aiguilles, hexagonalite }
}

/// juste : les six mesures de cristaux CONSTRUITS À LA MAIN, valeurs calculées à la main d'après les définitions de la doc (V-G2-MES-01 à V-G2-MES-12 ; recomptées par `mesures_g2.py`, un code Python
/// écrit d'après la doc seule). `compacite = floor(1000 n / (3 R (R + 1) + 1))`, `pointes` = les cases rattachées à au moins trois voisines libres, `aiguilles` = celles qui en ont au moins quatre, `hexagonalite = floor(1000 m / max(1, 6 R))`.
/// Chaque cas attrape une faute plausible : le germe seul (le disque de rayon 0 a UNE case : `R (R + 1)` sans le `+ 1`, `6 R` sans le `max(1, …)`) ; le disque plein (un coin a TROIS voisines libres, un côté
/// DEUX : un seuil `> 3` ou `>= 2` change `pointes`) ; le bras (chaque case est une pointe et une aiguille, même le germe) ; le disque percé et le disque plein n'ont aucune aiguille (un coin a trois voisines libres, pas quatre) ; l'anneau (l'anneau est à la distance `R`, pas `R - 1`) ; le disque perce (les voisines libres
/// de l'intérieur ne font pas de pointe) ; deux composantes ; un cristal dont le rayon vient d'une case qui n'est pas le germe ; l'ensemble vide (jamais de panique) ; le mur (V-G2-MES-10).
///
/// Ne couvre pas : des cristaux issus du modèle (voir `naif_contre_la_bibliotheque_sur_des_cristaux_de_g1`) ; la forme (voir `etape_g2_forme`).
#[test]
#[cfg_attr(not(feature = "etape-g2"), ignore)]
fn mesures_de_cristaux_construits_a_la_main() {
    let deux_composantes: Ensemble =
        disque(2).into_iter().chain(cases(1).into_iter().map(|(q, r)| (6 + q, -3 + r))).collect();
    let perce: Ensemble = disque(4).difference(&disque(1)).copied().collect();
    let une_case: Ensemble = [(3, -1)].into_iter().collect();
    // (nom, rayon du domaine, cases rattachées, mesures attendues)
    let cas: Vec<(&str, u32, Ensemble, Mesures)> = vec![
        ("germe seul (V-G2-MES-01)", 12, disque(0), m(1, 0, 1000, 1, 1, 1000)),
        ("disque de rayon 1 (V-G2-MES-02)", 12, disque(1), m(7, 1, 1000, 6, 0, 1000)),
        ("disque de rayon 4 (V-G2-MES-03)", 12, disque(4), m(61, 4, 1000, 6, 0, 1000)),
        ("disque de rayon 10 (V-G2-MES-04)", 12, disque(10), m(331, 10, 1000, 6, 0, 1000)),
        ("bras de longueur 7 (V-G2-MES-05)", 12, bras(7), m(8, 7, 47, 8, 8, 23)),
        ("anneau de rayon 5 (V-G2-MES-06)", 12, anneau(5), m(30, 5, 329, 30, 30, 1000)),
        ("disque de rayon 4 perce d'un trou de rayon 1 (V-G2-MES-07)", 12, perce, m(54, 4, 885, 6, 0, 1000)),
        (
            "deux composantes : un disque de rayon 2 et un de rayon 1 en (6, -3) (V-G2-MES-08)",
            12,
            deux_composantes,
            m(26, 7, 153, 12, 0, 47),
        ),
        (
            "étoile à six bras d'une case de large, longueur 12 (V-G2-MES-09)",
            12,
            etoile(12),
            m(73, 12, 155, 66, 66, 83),
        ),
        ("tout l'hexagone du domaine, rayon 3 : le mur (V-G2-MES-10)", 3, disque(3), m(37, 3, 1000, 6, 0, 1000)),
        ("aucune case rattachée (V-G2-MES-11)", 12, Ensemble::new(), m(0, 0, 0, 0, 0, 0)),
        ("une seule case, loin du centre, en (3, -1) (V-G2-MES-12)", 12, une_case, m(1, 3, 27, 1, 1, 55)),
    ];
    for (nom, domaine, ensemble, attendu) in cas {
        let neige = neige_de(domaine, &ensemble, fig9());
        assert_eq!(mesurer(&neige), attendu, "{nom}");
    }
}

/// juste : le MUR n'est pas du cristal (V-G2-MES-13). Le disque plein de rayon `r` posé dans un domaine de rayon `r` (toutes les cases de l'hexagone sont rattachées) a six pointes : ses cases d'angle
/// ont trois voisines au-delà du mur, ses cases de côté deux, et une voisine hors de l'hexagone est libre. Un code qui compterait le mur comme du cristal donnerait zéro pointe ; le même disque dans un domaine
/// plus grand donne le même résultat. De même, une case rattachée au centre d'un domaine de rayon 1 a six voisines dans l'hexagone, une case rattachée du bord n'en a que 3 ou 4 : `libres` compte les cases hors de
/// l'hexagone.
///
/// Ne couvre pas : un cristal qui touche le mur en n'étant pas un disque plein (les ensembles tirés de `naif_contre_la_bibliotheque_sur_des_ensembles_tires` en touchent beaucoup).
#[test]
#[cfg_attr(not(feature = "etape-g2"), ignore)]
fn le_mur_n_est_pas_du_cristal() {
    for r in 1..=6_u32 {
        let dans_le_domaine = mesurer(&neige_de(r, &disque(r as i32), fig9()));
        let dans_un_grand_domaine = mesurer(&neige_de(r + 5, &disque(r as i32), fig9()));
        assert_eq!(dans_le_domaine, dans_un_grand_domaine, "r = {r}");
        assert_eq!(dans_le_domaine.pointes, 6, "r = {r}");
    }
    // Un hexagone de rayon 1 (7 cases) entièrement rattaché : les six cases du bord ont chacune trois voisines hors du mur ou plus.
    let tout = mesurer(&neige_de(1, &disque(1), fig9()));
    assert_eq!(tout, m(7, 1, 1000, 6, 0, 1000));
    // Une seule case au bord d'un domaine de rayon 2 : six voisines libres, dont trois hors de l'hexagone ; c'est une pointe.
    let un_coin: Ensemble = [(2, 0)].into_iter().collect();
    assert_eq!(mesurer(&neige_de(2, &un_coin, fig9())), m(1, 2, 1000 / 19, 1, 1, 1000 / 12));
}

/// juste : le calcul NAÏF du test (`mesures_naives` : des boucles sur un ensemble de cases, les voisines parcourues à l'envers, des divisions entières en `u64`, écrit d'après la doc) donne EXACTEMENT les mesures de
/// la bibliothèque sur 400 ensembles tirés (V-G2-NAIF-01) : des rayons de domaine de 1 à 13, des densités de 5 à 100 % (donc des cristaux creux, troués, en morceaux, sans symétrie, qui touchent le mur ou non), et 150
/// cristaux connexes qui grandissent depuis l'origine. Les graines sont fixes et écrites ici ; aucune n'est choisie parce qu'elle passe. Une direction oubliée (voisine non comptée), un mur compté comme du cristal,
/// une compacité arrondie autrement, un rayon décalé de 1, un `>` pour `>=` au seuil de trois voisines (ou de quatre pour les aiguilles), l'anneau pris à la distance `R - 1`, un dénominateur `6 R` sans `max(1, …)` échouent.
///
/// Ne couvre pas : les cristaux du modèle (voir `naif_contre_la_bibliotheque_sur_des_cristaux_de_g1`).
#[test]
#[cfg_attr(not(feature = "etape-g2"), ignore)]
fn naif_contre_la_bibliotheque_sur_des_ensembles_tires() {
    let densites = [5_u64, 20, 35, 50, 65, 80, 95, 100];
    for graine in 1..=400_u64 {
        let domaine = 1 + (graine % 13) as u32;
        let densite = densites[(graine as usize) % densites.len()];
        let ensemble = ensemble_au_hasard(domaine as i32, densite, graine);
        let neige = neige_de(domaine, &ensemble, fig9());
        assert_eq!(mesurer(&neige), mesures_naives(&ensemble), "graine {graine}, domaine {domaine}, densité {densite}");
    }
    for graine in 1..=150_u64 {
        let domaine = 3 + (graine % 11) as u32;
        let tours = 2 + (graine % 9) as u32;
        let densite = 30 + 10 * (graine % 6);
        let ensemble = blob_au_hasard(domaine as i32, tours, densite, 1000 + graine);
        let neige = neige_de(domaine, &ensemble, fig9());
        assert_eq!(mesurer(&neige), mesures_naives(&ensemble), "blob graine {graine}, domaine {domaine}");
    }
}

/// juste : le calcul naïf du test donne EXACTEMENT les mesures de la bibliothèque sur des cristaux OBTENUS PAR `Neige::avancer` (V-G2-NAIF-02) : les trois jeux de l'article et trois jeux à nous (dont deux
/// qui touchent le mur), des domaines de rayon 12 à 40, de 40 à 400 pas, mesurés à plusieurs instants de la croissance (tous les 40 pas). L'ensemble `A` est relu case par case dans l'ordre canonique écrit
/// dans le test ; le rayon et le nombre de cases rattachées sont aussi comparés à `Neige::rayon_cristal` et `Neige::cases_rattachees` (G1, un autre calcul). Chaque cristal a grandi (rayon d'au moins 5 à la fin).
///
/// Ne couvre pas : la tendance le long d'un balayage (voir `etape_g2_ordre`) ; des tailles de l'article.
#[test]
#[cfg_attr(not(feature = "etape-g2"), ignore)]
fn naif_contre_la_bibliotheque_sur_des_cristaux_de_g1() {
    let jeux = [
        (24, fig9(), 400),
        (24, parametres("0.40", "1.3", "0.08", "0.025", "0.003", "0.07", "0.00005"), 400),
        (24, parametres("0.8", "1.9", "0.004", "0.001", "0.05", "0.015", "0.0001"), 240),
        (24, parametres("0.8", "2.7", "0.004", "0.001", "0.05", "0.015", "0.0001"), 240),
        (24, parametres("0.5", "1.4", "0.1", "0.005", "0.001", "0.09", "0.001"), 400),
        (12, jeu_dense(), 160),
        (40, parametres("0.65", "1.75", "0.2", "0.026", "0.15", "0.015", "0.00001"), 200),
        (10, parametres("1", "1", "0.3", "2", "0.02", "0.01", "0.0001"), 120),
    ];
    for (domaine, p, pas) in jeux {
        let mut neige = Neige::essayer_new(domaine, p).expect("neige valide");
        let mut t = 0;
        while t < pas {
            neige.avancer(40);
            t += 40;
            let ensemble = rattachees_de(&neige);
            let obtenu = mesurer(&neige);
            assert_eq!(obtenu, mesures_naives(&ensemble), "domaine {domaine}, {p:?}, après {t} pas");
            assert_eq!(obtenu.rattachees as usize, neige.cases_rattachees(), "domaine {domaine}, après {t} pas");
            assert_eq!(obtenu.rayon, neige.rayon_cristal(), "domaine {domaine}, après {t} pas");
        }
        assert!(mesurer(&neige).rayon >= 5, "domaine {domaine}, {p:?} : le cristal n'a pas grandi");
    }
}

/// juste : les mesures ne regardent que `a` (V-G2-NAIF-03). Deux neiges qui ont les mêmes cases rattachées mais des masses de bord, de glace et de vapeur différentes (et des réglages différents) ont les mêmes
/// mesures ; la neige qui a une vapeur partout et la neige vide autour du même cristal aussi. Un code qui lirait `c`, `b` ou `d` (par exemple « rattachée = `c > 0` ») échoue : une case de bord peut avoir
/// `c > 0` sans être rattachée.
///
/// Ne couvre pas : la règle de rattachement du modèle (G1).
#[test]
#[cfg_attr(not(feature = "etape-g2"), ignore)]
fn les_mesures_ne_regardent_que_les_cases_rattachees() {
    for graine in 1..=30_u64 {
        let domaine = 5 + (graine % 6) as u32;
        let ensemble = blob_au_hasard(domaine as i32, 2 + (graine % 5) as u32, 55, 77 * graine);
        let attendu = mesures_naives(&ensemble);
        // Même ensemble, masses quelconques : une case non rattachée voisine d'une case rattachée porte du bord, de la glace et de la vapeur ; les autres de la vapeur.
        let mut g = Lcg(graine);
        let liste: Vec<Case> = cases(domaine as i32)
            .iter()
            .map(|&(q, r)| {
                if ensemble.contains(&(q, r)) {
                    Case { a: true, b: Fixe::ZERO, c: Fixe(1 + g.jusqu_a(1 << 33) as i64), d: Fixe::ZERO }
                } else if DIRECTIONS.iter().any(|&(dq, dr)| ensemble.contains(&(q + dq, r + dr))) {
                    Case {
                        a: false,
                        b: Fixe(g.jusqu_a(1 << 33) as i64),
                        c: Fixe(1 + g.jusqu_a(1 << 33) as i64),
                        d: Fixe(g.jusqu_a(1 << 32) as i64),
                    }
                } else {
                    Case { a: false, b: Fixe::ZERO, c: Fixe::ZERO, d: Fixe(g.jusqu_a(1 << 32) as i64) }
                }
            })
            .collect();
        let chargee = Neige::essayer_depuis_cases(domaine, jeu_dense(), &liste).expect("état valide");
        assert_eq!(mesurer(&chargee), attendu, "graine {graine} : masses quelconques");
        assert_eq!(mesurer(&neige_de(domaine, &ensemble, fig9())), attendu, "graine {graine} : masses nulles");
    }
}

/// juste : un GRAND cristal ne fait rien déborder (V-G2-MES-14). L'hexagone de rayon 1197 entièrement rattaché a `3 · 1197 · 1198 + 1 = 4 302 019` cases : `1000 · rattachees = 4 302 019 000` dépasse un `u32`
/// (`4 294 967 295`). La doc demande les produits en `u64` : les mesures sont celles du disque plein, calculées à la main (`rattachees = 4 302 019`, `rayon = 1197`, `compacite = 1000`, six pointes, aucune aiguille,
/// `hexagonalite = 1000` : le mur n'est pas du cristal). Un produit fait en `u32` panique ici (le profil de test garde les vérifications de débordement) ou rend une compacité fausse. Le même hexagone privé de sa
/// dernière case (un coin) : une case de moins, `compacite = floor(4 302 018 000 / 4 302 019) = 999`, sept pointes (le coin ôté découvre ses deux voisines de côté), `hexagonalite = floor(7 181 000 / 7182) = 999`.
/// Le test prend environ 550 Mo et trois secondes.
///
/// Ne couvre pas : le plus grand rayon permis par `Neige` (2047 : 12 576 769 cases, plus d'un gigaoctet : mesuré à part par le juge, pas dans un test).
#[test]
#[cfg_attr(not(feature = "etape-g2"), ignore)]
fn un_grand_cristal_ne_deborde_pas() {
    let rayon = 1197_u32;
    let nombre = 3 * 1197 * 1198 + 1;
    assert_eq!(nombre, 4_302_019_usize);
    let pleine = Case { a: true, b: Fixe::ZERO, c: Fixe::UN, d: Fixe::ZERO };
    let mut liste = vec![pleine; nombre];
    let neige = Neige::essayer_depuis_cases(rayon, fig9(), &liste).expect("état valide");
    assert_eq!(mesurer(&neige), m(4_302_019, 1197, 1000, 6, 0, 1000));
    drop(neige);
    // La dernière case de l'ordre canonique est le coin `(0, 1197)` : on la détache (elle garde sa glace : une case non rattachée voisine du cristal peut en porter).
    liste[nombre - 1] = Case { a: false, b: Fixe::ZERO, c: Fixe::UN, d: Fixe::ZERO };
    let neige = Neige::essayer_depuis_cases(rayon, fig9(), &liste).expect("état valide");
    assert_eq!(neige.case(0, 1197).map(|case| case.a), Some(false));
    assert_eq!(mesurer(&neige), m(4_302_018, 1197, 999, 7, 0, 999));
}

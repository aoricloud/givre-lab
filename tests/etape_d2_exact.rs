//! Étape D2 : l'exactitude de `givre_lab::courbe` : la fenêtre des petits amas, la dimension d'un amas, la moyenne et l'erreur-type, le point de la
//! courbe, le demi-côté, la courbe. Valeurs attendues : `references/etape-d2-valeurs.md`, identifiants `V-D2-01` à `V-D2-09`.
//!
//! **Ce sont des tests « juste »** : chaque comportement est comparé à un calcul exact (fait à la main ou par `d2_ref.py`, un code Python indépendant
//! écrit d'après la documentation seule, en `decimal` à 60 chiffres) ou à un calcul **naïf écrit ici** (`tests/etape_d2_commun/mod.rs` : des boucles
//! simples, rien d'optimisé) qui répète la documentation de `src/courbe.rs` mot pour mot. La bibliothèque doit donner EXACTEMENT les mêmes bits que le
//! naïf. Les mesures statistiques de la courbe (l'ordre, les bandes) sont dans `etape_d2_ordre.rs`.

mod etape_d2_commun;

use etape_d2_commun::*;
use givre_lab::correlation::dimension_correlation;
use givre_lab::courbe::{
    courbe, demi_cote_de_la_courbe, dimension_d_un_amas, fenetre_petit_amas, moyenne_et_erreur_type, point_de_la_courbe,
};

/// Une forme de test : son nom, ses sites, la dimension que l'outil Python indépendant lui donne.
type Forme = (&'static str, Vec<(i32, i32)>, f64);

/// Un cas de point de la courbe : `(eta, graines, taille, demi_cote, moyenne de Python, erreur-type de Python)`.
type CasPoint = (u32, &'static [u64], usize, u32, f64, f64);

fn bits(valeur: f64) -> String {
    format!("{valeur:?} ({:#018x})", valeur.to_bits())
}

/// Les amas de claquage des cas ci-dessous : `(eta, graine, demi_cote, taille)`.
const AMAS: [(u32, u64, u32, usize); 6] =
    [(1, 3, 16, 40), (2, 5, 20, 40), (3, 2, 24, 40), (0, 7, 40, 100), (3, 1, 48, 100), (1, 4, 40, 100)];

/// juste : la fenêtre des petits amas suit sa formule (V-D2-01). Pour une étendue `e` de 0 à 300 (deux sites `(0, 0)` et `(e, 0)`) la fenêtre est celle de
/// la documentation, `2..=max(4, e / 6)` vide si ce dernier rayon dépasse `e / 2` ou 4096, recalculée dans `tests/etape_d2_commun/mod.rs` ; elle est aussi
/// égale à des listes écrites à la main : vide pour `e <= 7` (une étendue de 7 laisse `4 > 3`), `[2, 3, 4]` de 8 à 29, `[2, 3, 4, 5]` de 30 à 35,
/// `[2..=6]` de 36 à 41, `[2..=7]` à 42 ; et aux bords de `RAYON_MAX` : à l'étendue 24 581 le dernier rayon est 4096 (4095 rayons), à 24 582 il serait 4097 : vide ;
/// une étendue de `2^32 - 1` (les extrémités de l'`i32`) : vide, sans panique. Une liste vide ou d'un site : vide.
///
/// Ne couvre pas : la dépendance à autre chose que l'étendue (voir le test suivant) ; la valeur de la dimension.
#[test]
#[cfg_attr(not(feature = "etape-d2"), ignore)]
fn fenetre_suit_sa_formule() {
    for e in 0..=300_u32 {
        let sites = [(0, 0), (e as i32, 0)];
        assert_eq!(fenetre_petit_amas(&sites), fenetre_doc(e), "étendue {e}");
    }
    let a_la_main = |e: i32, attendu: Vec<u32>| {
        assert_eq!(fenetre_petit_amas(&[(0, 0), (e, 0)]), attendu, "étendue {e}");
    };
    for e in [0, 1, 4, 7] {
        a_la_main(e, vec![]);
    }
    for e in [8, 12, 23, 29] {
        a_la_main(e, vec![2, 3, 4]);
    }
    for e in [30, 33, 35] {
        a_la_main(e, vec![2, 3, 4, 5]);
    }
    for e in [36, 41] {
        a_la_main(e, vec![2, 3, 4, 5, 6]);
    }
    a_la_main(42, vec![2, 3, 4, 5, 6, 7]);
    a_la_main(299, (2..=49).collect());
    // les bords de RAYON_MAX (4096) : le dernier rayon est `e / 6`
    let grande = fenetre_petit_amas(&[(0, 0), (24_581, 0)]);
    assert_eq!((grande.first(), grande.last(), grande.len()), (Some(&2), Some(&4096), 4095));
    assert!(fenetre_petit_amas(&[(0, 0), (24_582, 0)]).is_empty(), "le dernier rayon 4097 dépasse RAYON_MAX");
    assert!(fenetre_petit_amas(&[(0, 0), (100_000, 0)]).is_empty());
    assert!(fenetre_petit_amas(&[(i32::MIN, 0), (i32::MAX, 0)]).is_empty());
    assert!(fenetre_petit_amas(&[]).is_empty());
    assert!(fenetre_petit_amas(&[(5, 5)]).is_empty());
    // la limite `dernier > e / 2` : une étendue de 7 vide, de 8 non (le dernier rayon 4 vaut exactement la moitié)
    assert!(fenetre_petit_amas(&[(0, 0), (0, 7)]).is_empty() && !fenetre_petit_amas(&[(0, 0), (0, 8)]).is_empty());
}

/// juste : la fenêtre ne dépend QUE de l'étendue (V-D2-02) : pour les étendues 8, 29, 30, 36 et 60, des amas de formes, de nombres de sites, de
/// positions, d'orientations et d'ordres très différents (un trait, un L, deux sites, un cadre, un nuage de sites pseudo-aléatoires dans le carré,
/// chacun translaté, tourné d'un quart de tour, retourné en miroir, remis à l'envers) ont exactement la fenêtre de la documentation ; et la fenêtre
/// ne change pas quand on ajoute un site à l'intérieur de la boîte (ni son nombre de sites ni sa dimension n'y entrent).
///
/// Ne couvre pas : le cas de bords de `RAYON_MAX` (voir `fenetre_suit_sa_formule`).
#[test]
#[cfg_attr(not(feature = "etape-d2"), ignore)]
fn fenetre_ne_depend_que_de_l_etendue() {
    for e in [8_i32, 29, 30, 36, 60] {
        let attendu = fenetre_doc(e as u32);
        let mut nuage = vec![(0, 0), (e, e)];
        let mut etat = 12345_u64 + e as u64;
        for _ in 0..40 {
            etat = etat.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            let x = ((etat >> 33) % (e as u64 + 1)) as i32;
            let y = ((etat >> 13) % (e as u64 + 1)) as i32;
            nuage.push((x, y));
        }
        let cadre: Vec<(i32, i32)> = (0..=e).flat_map(|k| [(k, 0), (k, e), (0, k), (e, k)]).collect();
        let en_l: Vec<(i32, i32)> = (0..=e).map(|k| (k, 0)).chain((0..=e).map(|k| (0, k))).collect();
        let familles = [vec![(0, 0), (e, 0)], trait_(e + 1), en_l, cadre, nuage];
        for famille in &familles {
            let transformees: Vec<Vec<(i32, i32)>> = vec![
                famille.clone(),
                famille.iter().map(|&(x, y)| (x - 1000, y + 777)).collect(),
                famille.iter().map(|&(x, y)| (-y, x)).collect(),
                famille.iter().map(|&(x, y)| (-x, y)).collect(),
                famille.iter().rev().copied().collect(),
            ];
            for t in transformees {
                assert_eq!(etendue_doc(&t), e as u32);
                assert_eq!(fenetre_petit_amas(&t), attendu, "étendue {e}, {} sites", t.len());
            }
        }
        // un site de plus dans la boîte, ou le même site répété : la même fenêtre
        let mut plus = familles[2].clone();
        plus.push((e / 2, e / 3));
        plus.push((0, 0));
        assert_eq!(fenetre_petit_amas(&plus), attendu);
    }
}

/// juste : la dimension d'un amas est celle de la corrélation sur la fenêtre, bit pour bit (V-D2-03). Pour six amas de claquage (`η` de 0 à 3, de 40 à 100
/// sites) et six formes, `dimension_d_un_amas` est `Some` et égale, à `to_bits` près, à `givre_lab::correlation::dimension_correlation` sur la fenêtre de la
/// documentation (`2..=max(4, e / 6)`) ET au calcul naïf écrit dans `tests/etape_d2_commun/mod.rs` (paires comptées une à une, disque compté case par case,
/// une seule division, `logarithme`, pente des moindres carrés dans l'ordre de la documentation) ; la plus grande fenêtre essayée a 15 rayons (le trait de 100 sites,
/// étendue 99 : `2..=16`), la plus petite 3.
///
/// Ne couvre pas : les valeurs elles-mêmes (voir `dimension_des_formes_contre_l_outil_independant`) ; la dimension d'une liste sans fenêtre (voir le test suivant).
#[test]
#[cfg_attr(not(feature = "etape-d2"), ignore)]
fn dimension_egale_la_correlation_sur_la_fenetre() {
    let mut cas: Vec<(String, Vec<(i32, i32)>)> = AMAS
        .iter()
        .map(|&(eta, graine, d, taille)| {
            (
                format!("claquage η={eta} graine={graine}"),
                amas_naif(graine, d, eta, taille).expect("l'amas atteint sa taille"),
            )
        })
        .collect();
    cas.push(("trait 100".into(), trait_(100)));
    cas.push(("carré 12".into(), carre(12)));
    cas.push(("disque 9".into(), disque_plein(9)));
    cas.push(("croix 9".into(), croix(9)));
    cas.push(("Vicsek 4".into(), vicsek(4)));
    cas.push(("tapis 2".into(), tapis(2)));
    let mut plus_grande_fenetre = 0;
    for (nom, sites) in cas {
        let rayons = fenetre_doc(etendue_doc(&sites));
        assert!(!rayons.is_empty(), "{nom} : la fenêtre est vide, le cas ne mesure rien");
        plus_grande_fenetre = plus_grande_fenetre.max(rayons.len());
        let d = dimension_d_un_amas(&sites).unwrap_or_else(|| panic!("{nom} : aucune dimension"));
        let attendue = dimension_correlation(&sites, &rayons);
        assert_eq!(d.to_bits(), attendue.to_bits(), "{nom} : {} contre la corrélation {}", bits(d), bits(attendue));
        let naive = dimension_naive(&sites).expect("le naïf mesure");
        assert_eq!(d.to_bits(), naive.to_bits(), "{nom} : {} contre le naïf {}", bits(d), bits(naive));
    }
    assert_eq!(plus_grande_fenetre, 15, "le trait de 100 sites (étendue 99) doit avoir la fenêtre 2..=16");
}

/// juste : `dimension_d_un_amas` rend `None` quand il n'y a rien à mesurer et ne panique jamais (V-D2-03) : une liste vide ; un seul site ; sept sites en ligne
/// (étendue 6 : fenêtre vide) ; trois sites éloignés (étendue 100 : la fenêtre est `2..=16` mais `C(r) = 0` pour tous ses rayons) ; **une seule `C(r)` nulle suffit** :
/// quatre sites `(0,0) (3,0) (50,0) (53,0)` (étendue 53, fenêtre `2..=8`) ont `C(2) = 0` et `C(3) > 0` : `None` ; les mêmes à distance 2 `(0,0) (2,0) (50,0) (52,0)`
/// ont tous leurs `C(r)` non nuls : `Some`, égale au calcul naïf ; des sites répétés (dix fois le même site, plus un autre à distance 20) : `Some` ou `None`
/// sans panique, et `None` ici car la fenêtre `2..=4` a `C(2) = 0` (les répétitions sont à distance 0, hors du cumul, et l'autre site est à 20).
///
/// Ne couvre pas : la valeur de `Some` pour des amas en morceaux (le naïf la recoupe seulement pour le dernier cas).
#[test]
#[cfg_attr(not(feature = "etape-d2"), ignore)]
fn dimension_est_none_quand_rien_n_est_mesurable() {
    assert_eq!(dimension_d_un_amas(&[]), None);
    assert_eq!(dimension_d_un_amas(&[(3, 4)]), None);
    assert_eq!(dimension_d_un_amas(&trait_(7)), None);
    assert_eq!(dimension_d_un_amas(&[(0, 0), (50, 0), (100, 0)]), None);
    assert_eq!(dimension_d_un_amas(&[(0, 0), (3, 0), (50, 0), (53, 0)]), None, "C(2) = 0 dans la fenêtre 2..=8");
    let deux_paires = [(0, 0), (2, 0), (50, 0), (52, 0)];
    let d = dimension_d_un_amas(&deux_paires).expect("tous les C(r) de la fenêtre sont non nuls");
    assert_eq!(d.to_bits(), dimension_naive(&deux_paires).unwrap().to_bits());
    let mut repetes = vec![(0, 0); 10];
    repetes.push((20, 0));
    assert_eq!(dimension_d_un_amas(&repetes), None);
}

/// juste : les dimensions de douze formes et de quatre amas de claquage sont celles de l'outil Python indépendant `d2_ref.py` (V-D2-04 et V-D2-05). L'outil est écrit d'après la documentation seule :
/// fenêtre `2..=max(4, e // 6)`, `C(r)` en fractions exactes, logarithmes en `decimal` à 60 chiffres, pente des moindres carrés en `decimal` ; ses valeurs sont
/// données ici en doubles (le plus court décimal qui revient au même `f64`, 17 chiffres au plus). Tolérance `1e-12` (la précision de `math::logarithme` est de `5e-16` en relatif, l'écart mesuré entre le prototype et l'outil est
/// de l'ordre de `1e-15`). **Ces valeurs ne sont PAS les dimensions exactes des formes** (un trait n'est pas à 1, un carré de 12 sites de côté est à 1,77 :
/// l'effet de bord d'un petit objet), seulement ce que la FORMULE donne ; les dimensions de similarité (`ln 5 / ln 3`, `ln 8 / ln 3`) sont dans `MODELE.md`,
/// avec l'erreur de la fenêtre.
///
/// Ne couvre pas : la justesse de la fenêtre pour mesurer la dimension d'une forme (c'est la sensibilité de `MODELE.md`, mesurée et non testée) ; les amas de claquage.
#[test]
#[cfg_attr(not(feature = "etape-d2"), ignore)]
fn dimension_des_formes_contre_l_outil_independant() {
    let cas: Vec<Forme> = vec![
        ("trait(40)", trait_(40), 0.9143974384063535),
        ("trait(100)", trait_(100), 0.9506966328960098),
        ("trait(300)", trait_(300), 0.9626343372328691),
        ("diagonale(30)", diagonale(30), 1.0144221606315442),
        ("carre(12)", carre(12), 1.7746942860644626),
        ("carre(25)", carre(25), 1.8987964383688234),
        ("disque_plein(9)", disque_plein(9), 1.85669608154284),
        ("croix(9)", croix(9), 1.1423905844403575),
        ("vicsek(3)", vicsek(3), 1.4862797903632097),
        ("vicsek(4)", vicsek(4), 1.4845430757472124),
        ("tapis(2)", tapis(2), 1.5798417941054668),
        ("tapis(3)", tapis(3), 1.7997313914180921),
    ];
    for (nom, sites, attendu) in cas {
        let d = dimension_d_un_amas(&sites).unwrap_or_else(|| panic!("{nom} : aucune dimension"));
        assert!((d - attendu).abs() <= 1e-12, "{nom} : {d} contre {attendu} (outil indépendant)");
    }
    // quatre amas de claquage (`d1_ref.py` fait croître l'amas, `d2_ref.py` mesure)
    for ((eta, graine, d, taille), attendu) in
        AMAS.iter().take(4).zip([1.4114199816896942, 1.4446687092196293, 1.4678960309055713, 1.7701724438824527])
    {
        let sites = amas_naif(*graine, *d, *eta, *taille).expect("l'amas atteint sa taille");
        let mesuree = dimension_d_un_amas(&sites).expect("une dimension");
        assert!(
            (mesuree - attendu).abs() <= 1e-12,
            "claquage η={eta} graine={graine} : {mesuree} contre {attendu} (outil indépendant)"
        );
    }
}

/// juste : `moyenne_et_erreur_type` à la main et en bits (V-D2-06). (1) `[1,3]` : moyenne 2, variance corrigée `(1 + 1) / 1 = 2`, erreur-type `sqrt(2 / 2) = 1`
/// EXACTEMENT ; `[1,2,3,4]` : moyenne `2,5`, variance `5/3`, erreur-type `sqrt(5/12) = 0,6454972243679028` ; trois valeurs égales : erreur-type 0 ; (2) deux listes de
/// bits donnés par Python (mêmes opérations, mêmes arrondis : `chercher_stats.py`) : `[1.103, 1.962, 1.04, 1.737, 1.679]` donne `(1.5042000000000002, 0.18313585121433762)`,
/// `[1.887, 1.306, 1.898]` donne `(1.697, 0.19552578687562755)` ; (3) ces listes **voient chaque faute** : la somme de la moyenne à l'envers, la somme des carrés à l'envers, la racine
/// de la variance divisée par `sqrt(n)`, le diviseur `n` au lieu de `n - 1` donnent d'autres bits, calculés ici.
///
/// Ne couvre pas : les entrées non finies (`NaN`, infinis) ; des milliers de valeurs.
#[test]
#[cfg_attr(not(feature = "etape-d2"), ignore)]
fn moyenne_et_erreur_type_a_la_main_et_en_bits() {
    assert_eq!(moyenne_et_erreur_type(&[1.0, 3.0]), (2.0, 1.0));
    let (m, s) = moyenne_et_erreur_type(&[1.0, 2.0, 3.0, 4.0]);
    assert_eq!(m, 2.5);
    assert_eq!(s.to_bits(), 0.6454972243679028_f64.to_bits(), "sqrt((5/3) / 4) : {}", bits(s));
    assert_eq!(moyenne_et_erreur_type(&[0.5, 0.5, 0.5]), (0.5, 0.0));
    assert_eq!(moyenne_et_erreur_type(&[-1.0, 1.0]), (0.0, 1.0));
    for (valeurs, moyenne, erreur) in [
        (vec![1.103, 1.962, 1.04, 1.737, 1.679], 1.5042000000000002_f64, 0.18313585121433762_f64),
        (vec![1.887, 1.306, 1.898], 1.697, 0.19552578687562755),
    ] {
        let (m, s) = moyenne_et_erreur_type(&valeurs);
        assert_eq!(
            (m.to_bits(), s.to_bits()),
            (moyenne.to_bits(), erreur.to_bits()),
            "{valeurs:?} : {} {}",
            bits(m),
            bits(s)
        );
        // les fautes, une à une : toutes donnent d'autres bits
        let n = valeurs.len() as f64;
        let somme = |v: &[f64]| v.iter().fold(0.0, |a, &x| a + x);
        let envers: Vec<f64> = valeurs.iter().rev().copied().collect();
        let moyenne_envers = somme(&envers) / n;
        assert_ne!(moyenne_envers.to_bits(), m.to_bits(), "{valeurs:?} ne voit pas une somme à l'envers");
        let carres = |v: &[f64]| v.iter().fold(0.0, |a, &x| a + (x - m) * (x - m));
        let carres_envers = carres(&envers);
        assert_ne!(
            (carres_envers / (n - 1.0) / n).sqrt().to_bits(),
            s.to_bits(),
            "{valeurs:?} ne voit pas des carrés à l'envers"
        );
        let variance = carres(&valeurs) / (n - 1.0);
        assert_ne!((variance.sqrt() / n.sqrt()).to_bits(), s.to_bits(), "{valeurs:?} ne voit pas sqrt(v) / sqrt(n)");
        assert_ne!((carres(&valeurs) / n / n).sqrt().to_bits(), s.to_bits(), "{valeurs:?} ne voit pas le diviseur n");
    }
}

/// juste : `moyenne_et_erreur_type` refuse moins de deux valeurs (V-D2-10) : une liste vide et une liste d'une valeur paniquent (message contenant « au moins deux »).
///
/// Ne couvre pas : la valeur du message au-delà de ces mots.
#[test]
#[cfg_attr(not(feature = "etape-d2"), ignore)]
#[should_panic(expected = "au moins deux")]
fn moyenne_et_erreur_type_refuse_une_valeur() {
    let _ = moyenne_et_erreur_type(&[1.0]);
}

/// juste : idem pour une liste vide (V-D2-10).
///
/// Ne couvre pas : le message au-delà de ces mots.
#[test]
#[cfg_attr(not(feature = "etape-d2"), ignore)]
#[should_panic(expected = "au moins deux")]
fn moyenne_et_erreur_type_refuse_aucune_valeur() {
    let _ = moyenne_et_erreur_type(&[]);
}

/// juste : `point_de_la_courbe` égale le calcul naïf, bit pour bit (V-D2-07). Pour cinq listes de graines (`η = 0` : huit graines dans un ordre où une somme à
/// l'envers change les bits de la moyenne ET de l'erreur-type, et cinq graines dont `1` est répétée ; `η = 1` à 3 : quatre et cinq graines de 40 sites, demi-côtés 16,
/// 20 et 24), la moyenne et l'erreur-type sont exactement celles du naïf écrit dans `tests/etape_d2_commun/mod.rs` (chaque amas par `Claquage`, sa dimension par le
/// naïf, la somme de gauche à droite, la variance corrigée `n - 1`, `sqrt(variance / n)`), ET celles de l'outil Python indépendant à `1e-12` près (`point_eta*.txt`),
/// ET `eta`, `taille` et `graines` sont ceux de l'appel (`graines` = le nombre de graines données, répétées comprises). Le cas `η = 0` est **vérifié sensible à l'ordre**
/// (la liste inversée donne d'autres bits dans le naïf lui-même).
///
/// Ne couvre pas : les amas de plus de 100 sites ; `η` au-delà de 3.
#[test]
#[cfg_attr(not(feature = "etape-d2"), ignore)]
fn point_de_la_courbe_egal_au_calcul_naif() {
    // (eta, graines, taille, demi_cote, moyenne et erreur-type de l'outil Python)
    let cas: [CasPoint; 5] = [
        (0, &[40, 6, 14, 24, 36, 10, 26, 4], 100, 40, 1.7594072119614177, 0.011246069991299637),
        (0, &[3, 1, 4, 1, 5], 100, 40, 1.760190321639205, 0.0037670743720508307),
        (1, &[2, 7, 11, 5], 40, 16, 1.5660696599687842, 0.04467801867048171),
        (2, &[5, 3, 9, 1, 6], 40, 20, 1.384847739657386, 0.04578514750363673),
        (3, &[4, 10, 2, 8], 40, 24, 1.4678813207292427, 0.009843395103462564),
    ];
    for (eta, graines, taille, d, python_m, python_s) in cas {
        let p = point_de_la_courbe(eta, graines, taille, d);
        let (m, s) = point_naif(eta, graines, taille, d);
        assert_eq!(p.eta, eta);
        assert_eq!(p.taille, taille);
        assert_eq!(p.graines, graines.len());
        assert_eq!(
            p.moyenne.to_bits(),
            m.to_bits(),
            "η={eta} {graines:?} : moyenne {} contre le naïf {}",
            bits(p.moyenne),
            bits(m)
        );
        assert_eq!(
            p.erreur_type.to_bits(),
            s.to_bits(),
            "η={eta} {graines:?} : erreur-type {} contre le naïf {}",
            bits(p.erreur_type),
            bits(s)
        );
        assert!((p.moyenne - python_m).abs() <= 1e-12, "η={eta} : moyenne {} contre Python {python_m}", p.moyenne);
        assert!(
            (p.erreur_type - python_s).abs() <= 1e-12,
            "η={eta} : erreur-type {} contre Python {python_s}",
            p.erreur_type
        );
    }
    // le cas `η = 0` à huit graines voit l'ordre : la liste inversée donne d'autres bits, et `point_de_la_courbe` les suit
    let graines: [u64; 8] = [40, 6, 14, 24, 36, 10, 26, 4];
    let inversees: Vec<u64> = graines.iter().rev().copied().collect();
    let (m, s) = point_naif(0, &graines, 100, 40);
    let (m2, s2) = point_naif(0, &inversees, 100, 40);
    assert_ne!(m.to_bits(), m2.to_bits(), "le cas ne voit pas l'ordre de la moyenne");
    assert_ne!(s.to_bits(), s2.to_bits(), "le cas ne voit pas l'ordre de l'erreur-type");
    let p = point_de_la_courbe(0, &inversees, 100, 40);
    assert_eq!(
        (p.moyenne.to_bits(), p.erreur_type.to_bits()),
        (m2.to_bits(), s2.to_bits()),
        "la liste inversée est sommée dans SON ordre"
    );
}

/// juste : `demi_cote_de_la_courbe` suit la règle de la documentation (V-D2-08) : `min(255, A + B * taille / 100)` avec `A = [30, 32, 22, 12]` et `B = [10, 8, 18, 36]`
/// pour `η = 0..=3`, pour 15 tailles de 0 à 100 000 (recalculé dans `tests/etape_d2_commun/mod.rs`) ; et les valeurs écrites à la main : à 100 sites `40, 40, 40, 48`, à 300 sites
/// `60, 56, 76, 120`, le plafond 255 atteint pour `η = 3` à 675 sites (`12 + 243`) et dépassé à 700 (`12 + 252 = 264`), la division entière (`18 * 101 / 100 = 18`,
/// donc `22 + 18 = 40` à 101 sites pour `η = 2`) ; **`η = 4` panique** (message contenant « η »).
///
/// Ne couvre pas : que ces demi-côtés soient assez grands (c'est le panique « frontière » de `point_de_la_courbe`, mesuré dans `MODELE.md`).
#[test]
#[cfg_attr(not(feature = "etape-d2"), ignore)]
fn demi_cote_de_la_courbe_suit_la_regle() {
    for eta in 0..=3_u32 {
        for taille in [0_usize, 1, 50, 99, 100, 101, 150, 199, 200, 300, 500, 674, 675, 700, 100_000] {
            assert_eq!(demi_cote_de_la_courbe(eta, taille), demi_cote_doc(eta, taille), "η={eta} taille={taille}");
        }
    }
    let a_100: Vec<u32> = (0..=3).map(|eta| demi_cote_de_la_courbe(eta, 100)).collect();
    let a_300: Vec<u32> = (0..=3).map(|eta| demi_cote_de_la_courbe(eta, 300)).collect();
    assert_eq!(a_100, [40, 40, 40, 48]);
    assert_eq!(a_300, [60, 56, 76, 120]);
    assert_eq!(demi_cote_de_la_courbe(3, 675), 255);
    assert_eq!(demi_cote_de_la_courbe(3, 700), 255);
    assert_eq!(demi_cote_de_la_courbe(3, 674), 254, "12 + 36 * 674 / 100 = 12 + 242");
    assert_eq!(demi_cote_de_la_courbe(2, 101), 40, "18 * 101 / 100 = 18 en entiers");
    assert_eq!(demi_cote_de_la_courbe(0, 0), 30);
    let refus = std::panic::catch_unwind(|| demi_cote_de_la_courbe(4, 100));
    let message = refus.expect_err("η = 4 doit paniquer");
    let texte = message
        .downcast_ref::<String>()
        .cloned()
        .or_else(|| message.downcast_ref::<&str>().map(|s| s.to_string()))
        .unwrap_or_default();
    assert!(texte.contains('η'), "message de la panique : {texte:?}");
}

/// juste : `courbe` donne un point par `η`, dans l'ordre donné, chacun égal au calcul naïf avec le demi-côté de la règle (V-D2-09). Avec `etas = [3, 0, 2, 0]`, 3 graines
/// (1 à 3) et 60 sites, la courbe a quatre points d'`eta` 3, 0, 2, 0 (l'ordre n'est ni trié ni dédoublonné), le point d'`η = 0` apparaît deux fois à l'identique, et chaque
/// point (`eta`, `taille`, `graines`, bits de la moyenne et de l'erreur-type) est celui du naïf avec `demi_cote_doc(eta, 60)` ; `courbe(&[], …)` est vide ; `courbe(&[2], …)`
/// est le point d'`η = 2` de la grande courbe, à l'identique.
///
/// Ne couvre pas : les tailles de plus de 100 sites ; l'ordre de calcul entre les `η` (sans effet, les amas sont indépendants).
#[test]
#[cfg_attr(not(feature = "etape-d2"), ignore)]
fn courbe_donne_les_points_dans_l_ordre_donne() {
    let graines = [1_u64, 2, 3];
    let points = courbe(&[3, 0, 2, 0], &graines, 60);
    assert_eq!(points.iter().map(|p| p.eta).collect::<Vec<_>>(), [3, 0, 2, 0]);
    assert_eq!(points[1], points[3], "le même η deux fois donne le même point");
    for p in &points {
        let (m, s) = point_naif(p.eta, &graines, 60, demi_cote_doc(p.eta, 60));
        assert_eq!((p.taille, p.graines), (60, 3));
        assert_eq!(
            p.moyenne.to_bits(),
            m.to_bits(),
            "η={} : moyenne {} contre le naïf {}",
            p.eta,
            bits(p.moyenne),
            bits(m)
        );
        assert_eq!(
            p.erreur_type.to_bits(),
            s.to_bits(),
            "η={} : erreur-type {} contre le naïf {}",
            p.eta,
            bits(p.erreur_type),
            bits(s)
        );
    }
    assert!(courbe(&[], &graines, 60).is_empty());
    // la courbe d'un seul η est le point de cet η dans la grande courbe (ni l'ordre ni les voisins n'y changent rien)
    let seul = courbe(&[2], &graines, 60);
    assert_eq!(seul.len(), 1);
    assert_eq!(seul[0], points[2]);
}

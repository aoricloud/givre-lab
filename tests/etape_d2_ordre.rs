//! Étape D2 : l'ORDRE et la BAISSE de `D(η)` (`givre_lab::courbe`) : `D(0) > D(1) > D(2) > D(3)` à 100 sites, EN MOYENNE sur 24 graines, chaque écart plus grand que ce que
//! l'erreur-type permet. Valeurs attendues : `references/etape-d2-valeurs.md`, identifiants `V-D2-18` à `V-D2-20`.
//!
//! **Ce sont des contrôles croisés de NOTRE modèle, étiquetés « régression »** : c'est le seul énoncé « physique » testé, et il est vrai en D1 pour `η = 1, 2, 3` (voir
//! `etape_d1_dimensions.rs`) ; D2 y ajoute `η = 0` et la fenêtre des petits amas. **Aucune valeur publiée n'entre dans ces tests** : ni `1,433`, ni `1,263`, ni `1,71`
//! ne sont comparés à une mesure (à 100 sites le banc est loin de sa valeur limite ; voir `MODELE.md`, qui écrit l'écart tel qu'il est). Les graines sont 1 à 24 (écrites ici),
//! 100 sites, les demi-côtés de `demi_cote_de_la_courbe` (40, 40, 40, 48). **Ce qui a été mesuré** (prototype, sorties dans `essai-d2-outils/`), sur 170 graines : les 120
//! graines 1 à 120 (cinq blocs de 24) et 50 graines neuves 1001 à 1050 (deux blocs de 25) ; `m` est la dimension moyenne d'un bloc, σ l'écart-type d'UNE graine :
//!
//! | `η` | σ (120 graines) | `m` des cinq blocs de 24 | `m` des deux blocs de 25 neufs | bande de `m` : moyenne ± 3 σ / √24 |
//! |---|---|---|---|---|
//! | 0 | 0,0219 | 1,7552 à 1,7645 | 1,7600 ; 1,7589 | `[1,746 ; 1,773]` |
//! | 1 | 0,0583 | 1,6453 à 1,6704 | 1,6784 ; 1,6679 | `[1,626 ; 1,698]` |
//! | 2 | 0,0698 | 1,4704 à 1,5110 | 1,5225 ; 1,4962 | `[1,452 ; 1,538]` |
//! | 3 | 0,0709 | 1,3258 à 1,3651 | 1,3435 ; 1,3501 | `[1,305 ; 1,392]` |
//!
//! Écarts `D(η) - D(η + 1)` des blocs : de `0,082` à `0,114` (0-1), de `0,156` à `0,189` (1-2), de `0,119` à `0,179` (2-3) ; leur écart en unités d'erreur-type (`z`, avec
//! `sqrt(e_a² + e_b²)`) est au moins `6,1`.
//!
//! **Recoupement du juge** (un second code écrit d'après la doc seule, mêmes bits ; 100 graines neuves, 7001 à 7050 et 9001 à 9050, huit blocs de 24, dont quatre disjoints) :
//! `m` de `1,7524` à `1,7617`, de `1,6404` à `1,6728`, de `1,4847` à `1,5363` et de `1,3388` à `1,3757` (tous dans les bandes ; la marge la plus faible est celle du haut de `η = 2`,
//! `0,0017`) ; écarts d'au moins `0,086`, `0,129` et `0,109` ; `z` d'au moins `6,7`, `6,5` et **`4,2`** : le critère « 4 fois l'erreur-type » est donc tout près de ce qu'un bloc honnête
//! donne parfois, c'est un plancher et non une marge confortable. Sur les graines des tests (1 à 24) : écarts `0,100`, `0,159`, `0,140`, `z` de `8,2`, `8,2` et `7,0`.
//! Les graines étant fixes et la doc fixant les bits, ces tests ne peuvent pas échouer au hasard : ils ne voient que des fautes.

use std::sync::OnceLock;

use givre_lab::courbe::{PointDeLaCourbe, courbe, csv_de_la_courbe};

const GRAINES: std::ops::RangeInclusive<u64> = 1..=24;
const TAILLE: usize = 100;

/// La courbe de `η = 0, 1, 2, 3` sur les graines 1 à 24 à 100 sites : un fil par `η` (chaque point ne dépend que de lui ; le résultat est le même au bit près que
/// `courbe(&[0, 1, 2, 3], …)`), rendue dans l'ordre des `η`. Quatre fils quel que soit le nombre de cœurs : le système les répartit.
fn serie() -> &'static [PointDeLaCourbe] {
    static SERIE: OnceLock<Vec<PointDeLaCourbe>> = OnceLock::new();
    SERIE.get_or_init(|| {
        let graines: Vec<u64> = GRAINES.collect();
        let graines = &graines;
        std::thread::scope(|s| {
            let fils: Vec<_> = (0..=3_u32)
                .map(|eta| {
                    s.spawn(move || {
                        let points = courbe(&[eta], graines, TAILLE);
                        assert_eq!(points.len(), 1);
                        points[0].clone()
                    })
                })
                .collect();
            fils.into_iter().map(|f| f.join().expect("un fil de mesure a paniqué")).collect()
        })
    })
}

/// régression : (propriété de NOTRE modèle, mesurée sur NOTRE code) `D(0) > D(1) > D(2) > D(3)`, et chaque écart est grand devant l'erreur-type (V-D2-18). Sur les graines 1 à 24, 100 sites,
/// la courbe a quatre points (`eta` 0 à 3, `taille` 100, 24 graines, erreurs-types dans `]0 ; 0,05[`) ; chaque écart `D(η) - D(η + 1)` est au moins `0,058`, `0,11` et `0,08`
/// (mesuré sur 170 graines : de `0,082` à `0,114`, de `0,156` à `0,189`, de `0,119` à `0,179` ; chaque borne est la moyenne mesurée de l'écart moins 3 σ_écart / √24, aucune n'a été élargie) ET au moins
/// 4 fois `sqrt(e_η² + e_{η+1}²)` (mesuré : au moins 6,1 fois dans les sept blocs de l'écrivain, au moins 4,2 fois dans les huit blocs neufs du juge ; 7,0 sur les graines 1 à 24). Un `D` qui ne baisse pas avec `η` (poids qui ne dépendent pas de `η`, un `η` décalé), un `η = 0` oublié
/// ou une fenêtre qui mesure autre chose que la corrélation locale échouent ici. Les écarts sont plus grands que ce que l'erreur-type permet, donc la baisse est celle du modèle et non du hasard
/// des graines.
///
/// Ne couvre pas : les valeurs publiées (jamais comparées) ; `η` au-delà de 3 ; les amas de plus de 100 sites (à 300 sites, mesuré 24 graines : `1,880 ; 1,684 ; 1,456 ; 1,315`, même ordre, voir
/// `MODELE.md`) ; la valeur d'UNE graine.
#[test]
#[cfg_attr(not(feature = "etape-d2"), ignore)]
fn la_dimension_baisse_avec_eta() {
    let c = serie();
    assert_eq!(c.len(), 4);
    for (k, p) in c.iter().enumerate() {
        assert_eq!((p.eta, p.taille, p.graines), (k as u32, TAILLE, 24));
        assert!(p.erreur_type > 0.0 && p.erreur_type < 0.05, "η={k} : erreur-type {}", p.erreur_type);
    }
    for (k, borne) in [0.058, 0.11, 0.08].into_iter().enumerate() {
        let ecart = c[k].moyenne - c[k + 1].moyenne;
        let erreur = (c[k].erreur_type * c[k].erreur_type + c[k + 1].erreur_type * c[k + 1].erreur_type).sqrt();
        assert!(
            ecart >= borne,
            "D({k}) - D({}) = {ecart} < {borne} ({} contre {})",
            k + 1,
            c[k].moyenne,
            c[k + 1].moyenne
        );
        assert!(
            ecart >= 4.0 * erreur,
            "D({k}) - D({}) = {ecart} n'est que {} fois l'erreur-type {erreur}",
            k + 1,
            ecart / erreur
        );
    }
}

/// régression : la dimension moyenne de chaque `η` est dans sa bande (V-D2-19). Sur les graines 1 à 24, 100 sites : `D(0)` dans `[1,746 ; 1,773]`, `D(1)` dans `[1,626 ; 1,698]`, `D(2)` dans
/// `[1,452 ; 1,538]`, `D(3)` dans `[1,305 ; 1,392]` (chaque bande est la moyenne mesurée sur 170 graines ± 3 σ / √24, voir l'en-tête ; les sept blocs mesurés y tombent tous ; aucune n'a été élargie).
/// Elles disent que la dimension est PLAUSIBLE à cette taille (un amas plein de 100 sites mesure 1,76 et non 2 : l'effet de bord d'un disque de rayon de l'ordre de 10 ; `η = 1` à 1,66 contre la DLA limite
/// à 1,71 ; `η = 3` à 1,35), pas qu'elle soit juste : **ces bandes ne sont pas des valeurs publiées**, et `1,433` et `1,263` n'y figurent pas.
///
/// Ne couvre pas : la valeur de la dimension limite ; la comparaison à Hastings (jamais testée) ; une autre taille que 100 sites.
#[test]
#[cfg_attr(not(feature = "etape-d2"), ignore)]
fn la_dimension_de_chaque_eta_est_dans_sa_bande() {
    let c = serie();
    for (k, (bas, haut)) in [(1.746, 1.773), (1.626, 1.698), (1.452, 1.538), (1.305, 1.392)].into_iter().enumerate() {
        assert!((bas..=haut).contains(&c[k].moyenne), "D({k}) = {}, hors de [{bas} ; {haut}]", c[k].moyenne);
    }
}

/// régression : la courbe de 100 sites sur les graines 1 à 24 est figée, au CSV près (V-D2-20) : le texte de `csv_de_la_courbe` de cette courbe est
/// `eta,taille,graines,D,erreur,publie,ecart` puis `0,100,24,1.757889,0.003558,,`, `1,100,24,1.657425,0.011674,1.710,-0.052575`, `2,100,24,1.498010,0.015516,1.433,0.065010`,
/// `3,100,24,1.358243,0.012358,1.263,0.095243` (chaque ligne finie par `\n`). La doc de `src/courbe.rs` fixe tous les bits (fenêtre, demi-côtés, croissance de `Claquage`, somme, variance) : une
/// implémentation fidèle donne exactement ce texte, qui est celui de `cargo run --example courbe_d2 -- <dossier> 100 24`. **Ce n'est pas une comparaison au publié** : la colonne `ecart`
/// est l'arithmétique `D - publie` de NOTRE sortie, figée avec le reste ; le test ne dit pas que l'écart est petit, ni grand. Il ne prouve que la stabilité (les tests `juste` des autres fichiers
/// prouvent, eux, que chaque nombre est celui de la doc).
///
/// Ne couvre pas : la justesse (c'est `etape_d2_exact`) ; une autre graine, taille ou machine (la doc fixe les bits : même résultat sous Linux et Windows).
#[test]
#[cfg_attr(not(feature = "etape-d2"), ignore)]
fn courbe_de_100_sites_figee() {
    let attendu = "eta,taille,graines,D,erreur,publie,ecart\n\
                   0,100,24,1.757889,0.003558,,\n\
                   1,100,24,1.657425,0.011674,1.710,-0.052575\n\
                   2,100,24,1.498010,0.015516,1.433,0.065010\n\
                   3,100,24,1.358243,0.012358,1.263,0.095243\n";
    assert_eq!(csv_de_la_courbe(serie()), attendu);
}

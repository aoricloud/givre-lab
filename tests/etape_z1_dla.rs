//! Étape Z1 : la corrélation densité-densité sur la DLA du banc (`givre_lab::dla`), et son accord avec les deux mesures de l'étape 1.
//! Valeurs attendues : `references/etape-z1-valeurs.md`, identifiants `V-Z1-xx`.
//!
//! Sur chaque amas on mesure trois fois sa dimension :
//!
//! - la **corrélation** : `mesure(sites).dimension`, la fenêtre de la doc (4 à `étendue / 8`) ;
//! - la **masse-rayon** `D_mr` : la pente de `ln N(r)` contre `ln r` depuis le germe, sur TOUS les rayons entiers de 6 à `rayon / 2`
//!   (`rayon` = `Amas::rayon()`), exactement comme `etape_1_dla_statistique` ;
//! - le **rayon de giration** : `1/β`, `β` la pente de `ln R_g` contre `ln N` (`R_g` celui des `N` premiers sites du même amas), pour les
//!   tailles `N = 125, 250, 500, 1000, 2000, 4000` (amas de 4000 sites) et `313, 625, 1250, 2500, 5000, 10000` (amas de 10 000 sites : mêmes
//!   rapports, on part de `10000 / 32`).
//!
//! Sur 24 graines (1 à 24, écrites ici), à 4000 et à 10 000 sites, `Amas::new(graine, 400)`. **Ce qui a été mesuré** (prototype, outils et
//! sorties dans `essai-z1-outils/`) sur les graines 1 à 120 (cinq blocs de 24) :
//!
//! | taille | corrélation | masse-rayon | 1/β | corr − mr | corr − 1/β | mr − 1/β |
//! |---|---|---|---|---|---|---|
//! | 4000 sites | 1,6486 (σ 0,0215) | 1,6737 (σ 0,047) | 1,7101 (σ 0,060) | −0,0251 (σ 0,046) | −0,0615 (σ 0,061) | −0,0364 (σ 0,044) |
//! | 10 000 sites | 1,6587 (σ 0,0152) | 1,6803 (σ 0,035) | 1,7127 (σ 0,054) | −0,0216 (σ 0,037) | −0,0540 (σ 0,058) | −0,0324 (σ 0,044) |
//!
//! (σ est l'écart-type d'UNE graine.) Les moyennes des cinq blocs de 24 graines : à 4000 sites corrélation de 1,6427 à 1,6537, écart
//! `corr − mr` de −0,042 à −0,014, `corr − 1/β` de −0,087 à −0,041, `mr − 1/β` de −0,045 à −0,027 ; à 10 000 sites corrélation de 1,6530 à 1,6633,
//! `corr − mr` de −0,035 à −0,013, `corr − 1/β` de −0,072 à −0,042, `mr − 1/β` de −0,037 à −0,024. Les trois mesures s'accordent donc à un dixième
//! près, mais **pas à la précision d'une graine** (l'écart-type d'une graine est 0,02 pour la corrélation, 0,04 pour la masse-rayon, 0,06 pour la
//! giration) et **pas au centième** : elles s'ordonnent `corrélation < masse-rayon < 1/β` dans tous les blocs. Les bandes ci-dessous sont
//! `moyenne ± 3 σ / √24` sur les 120 graines, les moyennes des blocs de 24 graines y tombent toutes, et aucune n'a été élargie.
//!
//! Ce qui n'est PAS testé : la valeur publiée 1,713 (Davidovitch, Levermann et Procaccia, 2000, un calcul de renormalisation sur de petits amas) ni
//! le zinc 1,66 ± 0,03 (Matsushita et coll., 1984) : un test « dans la barre du zinc » se réglerait pour passer, et la corrélation du banc (1,65 à
//! 1,66 à ces tailles) y est de toute façon, ce qui n'est pas une preuve.

mod etape_z1_commun;

use std::sync::OnceLock;

use etape_z1_commun::{melanger, translater};
use givre_lab::correlation::{correlation, fenetre, mesure, paires_par_distance2};
use givre_lab::dla::Amas;
use givre_lab::math::logarithme;
use givre_lab::mesures::{dimension_masse_rayon, pente, rayon_de_giration};

const GRAINES: std::ops::RangeInclusive<u64> = 1..=24;
const TAILLES_GIRATION_4000: [usize; 6] = [125, 250, 500, 1000, 2000, 4000];
const TAILLES_GIRATION_10000: [usize; 6] = [313, 625, 1250, 2500, 5000, 10000];

/// Les trois dimensions d'un amas, et la courbe `C(r)` pour `r = 1..=60` (vide si elle n'est pas demandée : seule la série de 4000 sites la
/// demande, pour `la_courbe_decroit`).
struct Mesures {
    correlation: f64,
    masse_rayon: f64,
    giration: f64,
    courbe: Vec<f64>,
}

fn mesurer(graine: u64, particules: usize, tailles: &[usize], avec_courbe: bool) -> Mesures {
    let mut amas = Amas::new(graine, 400);
    amas.croitre_jusqu_a(particules);
    let sites = amas.sites();
    let correlation_mesuree = mesure(sites).expect("un amas de 4000 sites a une fenêtre").dimension;
    let rayons: Vec<u32> = (6..=amas.rayon() / 2).collect();
    let masse_rayon = dimension_masse_rayon(sites, (0, 0), &rayons);
    let xs: Vec<f64> = tailles.iter().map(|&n| logarithme(n as f64)).collect();
    let ys: Vec<f64> = tailles.iter().map(|&n| logarithme(rayon_de_giration(&sites[..n]))).collect();
    Mesures {
        correlation: correlation_mesuree,
        masse_rayon,
        giration: 1.0 / pente(&xs, &ys),
        courbe: if avec_courbe { correlation(sites, 60) } else { Vec::new() },
    }
}

/// Les mesures des 24 graines pour une taille d'amas, dans l'ordre des graines. Les amas sont indépendants : autant de fils que la machine en
/// offre (de 1 à 8) se les partagent (le résultat de chaque graine ne dépend que d'elle, et les moyennes se font ensuite dans l'ordre des graines :
/// même résultat, au bit près, quel que soit le nombre de fils).
fn serie(particules: usize, tailles: &'static [usize], avec_courbe: bool) -> Vec<Mesures> {
    let graines: Vec<u64> = GRAINES.collect();
    let fils = std::thread::available_parallelism().map_or(1, |n| n.get()).clamp(1, 8);
    std::thread::scope(|s| {
        let fils: Vec<_> = graines
            .chunks(graines.len().div_ceil(fils))
            .map(|part| {
                s.spawn(move || part.iter().map(|&g| mesurer(g, particules, tailles, avec_courbe)).collect::<Vec<_>>())
            })
            .collect();
        fils.into_iter().flat_map(|f| f.join().expect("un fil de mesure a paniqué")).collect()
    })
}

fn serie_4000() -> &'static [Mesures] {
    static SERIE: OnceLock<Vec<Mesures>> = OnceLock::new();
    SERIE.get_or_init(|| serie(4000, &TAILLES_GIRATION_4000, true))
}

fn serie_10000() -> &'static [Mesures] {
    static SERIE: OnceLock<Vec<Mesures>> = OnceLock::new();
    SERIE.get_or_init(|| serie(10_000, &TAILLES_GIRATION_10000, false))
}

fn moyenne(valeurs: impl Iterator<Item = f64>) -> f64 {
    let (somme, n) = valeurs.fold((0.0, 0.0), |(s, n), v| (s + v, n + 1.0));
    somme / n
}

/// Vérifie les bandes et les accords d'une série de 24 graines. `bande_correlation` est la bande de la moyenne de la corrélation.
fn verifier_les_trois_mesures(serie: &[Mesures], taille: usize, bande_correlation: (f64, f64)) {
    let corr = moyenne(serie.iter().map(|m| m.correlation));
    let mr = moyenne(serie.iter().map(|m| m.masse_rayon));
    let gir = moyenne(serie.iter().map(|m| m.giration));
    assert!(
        (bande_correlation.0..=bande_correlation.1).contains(&corr),
        "{taille} sites : corrélation moyenne {corr} hors de [{} ; {}]",
        bande_correlation.0,
        bande_correlation.1
    );
    assert!(
        (corr - mr).abs() <= 0.055,
        "{taille} sites : corrélation {corr} et masse-rayon {mr} s'écartent de plus de 0,055"
    );
    assert!((corr - gir).abs() <= 0.10, "{taille} sites : corrélation {corr} et 1/β {gir} s'écartent de plus de 0,10");
    assert!((mr - gir).abs() <= 0.065, "{taille} sites : masse-rayon {mr} et 1/β {gir} s'écartent de plus de 0,065");
    // Les trois mesures sont de vraies dimensions fractales : entre 1 et 2, loin de 1 comme de 2, pour chacune des graines.
    for (graine, m) in GRAINES.zip(serie) {
        for (nom, valeur) in [("corrélation", m.correlation), ("masse-rayon", m.masse_rayon), ("1/β", m.giration)] {
            assert!((1.45..=1.95).contains(&valeur), "graine {graine}, {taille} sites : {nom} = {valeur}");
        }
    }
}

/// régression : (bande mesurée sur NOTRE modèle, accord de trois estimateurs) sur les graines 1 à 24, 4000 sites chacune, les trois mesures de
/// la dimension de l'amas s'accordent : la corrélation moyenne est dans `[1,635 ; 1,662]` (`1,6486 ± 3 σ/√24`, V-Z1-18) ; la corrélation et la
/// masse-rayon moyennes diffèrent d'au plus `0,055`, la corrélation et `1/β` d'au plus `0,10`, la masse-rayon et `1/β` d'au plus `0,065`
/// (les trois bandes `|écart moyen| + 3 σ/√24` mesurées sur 120 graines, V-Z1-19) ; chaque graine donne trois dimensions entre `1,45` et `1,95`.
/// C'est un contrôle croisé de la mesure, pas une preuve de la dimension de la DLA (voir l'en-tête).
///
/// Ne couvre pas : la valeur publiée 1,713 ni le zinc 1,66 (aucun test ne s'y appuie) ; que chaque graine s'accorde (l'écart-type d'une graine
/// est de 0,02 à 0,06) ; la dynamique de la DLA (les tests de l'étape 1).
#[test]
#[cfg_attr(not(feature = "etape-z1"), ignore)]
fn trois_mesures_a_4000_sites() {
    verifier_les_trois_mesures(serie_4000(), 4000, (1.635, 1.662));
}

/// régression : comme `trois_mesures_a_4000_sites`, à 10 000 sites (graines 1 à 24) : la corrélation moyenne est dans `[1,649 ; 1,668]`
/// (`1,6587 ± 3 σ/√24`), les trois écarts sont bornés par `0,055`, `0,10` et `0,065` (mesurés : `0,044`, `0,090` et `0,059` au plus pour la
/// bande, `0,035`, `0,072` et `0,037` au plus pour un bloc de 24 graines), chaque graine donne des dimensions entre `1,45` et `1,95` (V-Z1-20).
///
/// Ne couvre pas : les amas de plus de 10 000 sites (la corrélation monte encore, à 1,667 pour 30 000 sites sur 12 graines).
#[test]
#[cfg_attr(not(feature = "etape-z1"), ignore)]
fn trois_mesures_a_10000_sites() {
    verifier_les_trois_mesures(serie_10000(), 10_000, (1.649, 1.668));
}

/// régression : (propriété vraie en moyenne et mesurée sur NOTRE modèle) sur les graines 1 à 24, 4000 sites, la courbe `C(r)` DÉCROÎT : la
/// moyenne sur les graines de `C(r)` est strictement décroissante pour `r = 1..=60` (0,546 à `r = 1`, 0,335 à `r = 4`, 0,127 à `r = 60`) ;
/// pour chaque graine `C(60) < C(4)`. (Mesuré : sur 120 graines la courbe de chaque graine est même strictement décroissante de 1 à 60, mais
/// c'est une propriété de réseau que le test ne demande pas.) Une courbe qui monterait, ou une pente de `+2` oubliée, se voit ici (V-Z1-21).
///
/// Ne couvre pas : la pente (voir `trois_mesures_a_4000_sites`) ; les rayons au-delà de 60.
#[test]
#[cfg_attr(not(feature = "etape-z1"), ignore)]
fn la_courbe_decroit() {
    let serie = serie_4000();
    let moyenne_par_rayon: Vec<f64> = (0..60).map(|i| moyenne(serie.iter().map(|m| m.courbe[i]))).collect();
    for r in 1..60 {
        assert!(
            moyenne_par_rayon[r] < moyenne_par_rayon[r - 1],
            "la courbe moyenne ne décroît pas de r = {r} à r = {} : {} puis {}",
            r + 1,
            moyenne_par_rayon[r - 1],
            moyenne_par_rayon[r]
        );
    }
    assert!(moyenne_par_rayon[0] > 0.5 && moyenne_par_rayon[0] < 0.6, "C(1) moyen : {}", moyenne_par_rayon[0]);
    for (graine, m) in GRAINES.zip(serie) {
        assert!(
            m.courbe[59] < m.courbe[3],
            "graine {graine} : C(60) = {} n'est pas sous C(4) = {}",
            m.courbe[59],
            m.courbe[3]
        );
    }
}

/// juste : la corrélation compte TOUTES les paires, pas seulement celles qui partent du germe ou du centre de masse (V-Z1-22) : pour un amas de
/// la DLA de 1500 sites (graine 3), avec un `rayon_max` qui contient l'amas entier (le double du rayon de l'amas plus un), la somme des
/// comptes de `paires_par_distance2` est `1500 × 1499` ; la mesure ne change pas, au bit près, quand on translate l'amas de `(500, -300)`,
/// quand on retourne la liste de ses sites ou quand on la mélange (le germe n'est plus le premier site) ; et `dimension_masse_rayon` depuis
/// le germe, elle, donne autre chose (elle est mesurée depuis un centre).
///
/// Ne couvre pas : l'autre moitié du lien avec la masse-rayon (les écarts de moyenne : voir `trois_mesures_a_4000_sites`).
#[test]
#[cfg_attr(not(feature = "etape-z1"), ignore)]
fn toutes_les_paires_sont_comptees() {
    let mut amas = Amas::new(3, 400);
    amas.croitre_jusqu_a(1500);
    let sites = amas.sites().to_vec();
    let rayon_max = 2 * amas.rayon() + 1;
    let toutes = paires_par_distance2(&sites, rayon_max);
    assert_eq!(toutes.iter().sum::<u64>(), 1500 * 1499);

    let reference = mesure(&sites).expect("1500 sites ont une fenêtre");
    assert_eq!(reference.rayons, fenetre(&sites));
    let mut inverse = sites.clone();
    inverse.reverse();
    for (nom, autres) in [
        ("translation", translater(&sites, (500, -300))),
        ("ordre inversé", inverse),
        ("ordre mélangé", melanger(&sites, 9)),
    ] {
        let m = mesure(&autres).expect("même fenêtre");
        assert_eq!(m.dimension.to_bits(), reference.dimension.to_bits(), "{nom} : dimension");
        assert_eq!(m.rayons, reference.rayons, "{nom} : rayons");
    }
    let depuis_le_germe = dimension_masse_rayon(&sites, (0, 0), &(6..=amas.rayon() / 2).collect::<Vec<u32>>());
    assert!(
        (depuis_le_germe - reference.dimension).abs() > 1e-6,
        "la masse-rayon depuis le germe ne devrait pas être la corrélation : {depuis_le_germe}"
    );
}

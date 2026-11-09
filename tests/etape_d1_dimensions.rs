//! Étape D1 : la forme des amas de claquage (`givre_lab::claquage`) : plein à `η = 0`, voisin de la DLA à `η = 1`, de plus en plus fin quand
//! `η` monte. Valeurs attendues : `references/etape-d1-valeurs.md`, identifiants `V-D1-16` à `V-D1-18` (V-D1-19 : les repères publiés, non testés).
//!
//! **Ce sont des contrôles croisés de NOTRE modèle, étiquetés « régression »** : aucune valeur publiée n'y entre (Niemeyer et coll. 1984 ;
//! Hastings 2001, tableau II : `η = 2 → 1,433`, `η = 3 → 1,263` ; la DLA à 1,713 : repères non testés, dans la fiche ; la courbe `D(η)` contre
//! Hastings est l'étape D2). Les tailles sont petites (100 à 2000 sites) parce que le champ de Laplace est calculé à chaque pas : à ces tailles la
//! dimension est bien en dessous de sa valeur limite, et ces tests ne prouvent pas une dimension, seulement l'ordre et la compacité.
//!
//! Sur 24 graines (1 à 24, écrites ici), `Claquage::new(graine, demi_cote, eta)` ; le `demi_cote` est choisi pour que l'amas n'atteigne pas la frontière
//! (aucune des 120 graines mesurées ne l'atteint). **Ce qui a été mesuré** (prototype, outils et sorties dans `essai-d1-outils/`) sur les graines 1 à 120
//! (cinq blocs de 24) ; σ est l'écart-type d'UNE graine ; les bandes sont `moyenne ± 3 σ / √24` (ou, pour une différence, `moyenne ± 3 σ_diff / √24`) :
//!
//! | série | `corr` (rayons 2 à 8 ou 2 à 6) | masse-rayon | `1/β` | rayon de l'amas |
//! |---|---|---|---|---|
//! | `η = 1`, 300 sites, `demi_cote` 45 | 1,6767 (σ 0,041) | 1,614 (σ 0,168) | 1,800 (σ 0,158) | 23,5 (σ 2,8) |
//! | DLA (étape 1), 300 sites | 1,6027 (σ 0,043) | 1,580 (σ 0,150) | 1,687 (σ 0,128) | 27,6 (σ 3,0) |
//! | `η = 1`, 100 sites, `demi_cote` 40 | 1,6060 (σ 0,056) | 1,173 (σ 0,255) | 1,767 (σ 0,164) | 13,1 (σ 1,7) |
//! | `η = 2`, 100 sites, `demi_cote` 40 | 1,4548 (σ 0,068) | 1,247 (σ 0,268) | 1,503 (σ 0,133) | 17,1 (σ 2,4) |
//! | `η = 3`, 100 sites, `demi_cote` 40 | 1,3370 (σ 0,062) | 1,184 (σ 0,249) | 1,314 (σ 0,132) | 23,0 (σ 4,1) |
//!
//! La masse-rayon (rayons 6 à rayon/2) est trop bruitée à 100 sites (quelques rayons seulement) pour ordonner quoi que ce soit : elle n'est utilisée qu'à
//! 300 sites, avec une bande large. À 300 sites, `η = 1` est PLUS COMPACT que la DLA de l'étape 1 (corrélation +0,074 ; giration +0,11 ; rayon -4,1) :
//! les deux modèles ne sont pas équivalents sur la grille à ces tailles (voir `MODELE.md`). Le test dit l'écart mesuré, il ne l'efface pas.

use std::sync::OnceLock;

use givre_lab::claquage::Claquage;
use givre_lab::correlation::{dimension_correlation, sites_du_disque};
use givre_lab::dla::Amas;
use givre_lab::math::logarithme;
use givre_lab::mesures::{dimension_masse_rayon, masse_dans_rayon, pente, rayon_de_giration};

const GRAINES: std::ops::RangeInclusive<u64> = 1..=24;

/// Les trois dimensions et le rayon d'un amas.
struct Mesure {
    corr: f64,
    mr: f64,
    gir: f64,
    rayon: f64,
}

/// Les mesures d'un amas de `n` sites : la corrélation sur les rayons entiers `rmin` à `rmax`, la masse-rayon depuis le germe sur les rayons entiers de 6 à
/// `rayon / 2` (au moins 7), et `1/β`, `β` la pente de `ln R_g` contre `ln k` pour `k = n/8, n/4, n/2, n` (les `k` premiers sites).
fn mesurer(sites: &[(i32, i32)], rayon: u32, rmin: u32, rmax: u32) -> Mesure {
    let rayons_corr: Vec<u32> = (rmin..=rmax).collect();
    let rayons_mr: Vec<u32> = (6..=(rayon / 2).max(7)).collect();
    let n = sites.len();
    let tailles = [n / 8, n / 4, n / 2, n];
    let xs: Vec<f64> = tailles.iter().map(|&k| logarithme(k as f64)).collect();
    let ys: Vec<f64> = tailles.iter().map(|&k| logarithme(rayon_de_giration(&sites[..k]))).collect();
    Mesure {
        corr: dimension_correlation(sites, &rayons_corr),
        mr: dimension_masse_rayon(sites, (0, 0), &rayons_mr),
        gir: 1.0 / pente(&xs, &ys),
        rayon: f64::from(rayon),
    }
}

/// Fait `calcul(graine)` pour chaque graine de `GRAINES` : autant de fils que la machine en offre (de 1 à 8) se les partagent ; le résultat de chaque graine ne
/// dépend que d'elle, et il est rendu dans l'ordre des graines (même résultat, au bit près, quel que soit le nombre de fils).
fn pour_chaque_graine<T: Send>(calcul: impl Fn(u64) -> T + Sync) -> Vec<T> {
    let graines: Vec<u64> = GRAINES.collect();
    let fils = std::thread::available_parallelism().map_or(1, |n| n.get()).clamp(1, 8);
    let calcul = &calcul;
    std::thread::scope(|s| {
        let fils: Vec<_> = graines
            .chunks(graines.len().div_ceil(fils))
            .map(|part| s.spawn(move || part.iter().map(|&g| calcul(g)).collect::<Vec<_>>()))
            .collect();
        fils.into_iter().flat_map(|f| f.join().expect("un fil de mesure a paniqué")).collect()
    })
}

/// Un amas de claquage de `n` sites (il doit les atteindre : la frontière ne doit pas arrêter la croissance).
fn claquage(graine: u64, demi_cote: u32, eta: u32, n: usize) -> Claquage {
    let mut c = Claquage::new(graine, demi_cote, eta);
    while c.sites().len() < n {
        assert!(
            c.essayer_ajouter().is_some(),
            "eta {eta}, graine {graine} : l'amas touche la frontière à {} sites",
            c.sites().len()
        );
    }
    c
}

fn serie_claquage(demi_cote: u32, eta: u32, n: usize, rmax: u32) -> Vec<Mesure> {
    pour_chaque_graine(|graine| {
        let c = claquage(graine, demi_cote, eta, n);
        mesurer(c.sites(), c.rayon(), 2, rmax)
    })
}

fn claquage_eta1_300() -> &'static [Mesure] {
    static SERIE: OnceLock<Vec<Mesure>> = OnceLock::new();
    SERIE.get_or_init(|| serie_claquage(45, 1, 300, 8))
}

fn dla_300() -> &'static [Mesure] {
    static SERIE: OnceLock<Vec<Mesure>> = OnceLock::new();
    SERIE.get_or_init(|| {
        pour_chaque_graine(|graine| {
            let mut amas = Amas::new(graine, 200);
            amas.croitre_jusqu_a(300);
            mesurer(amas.sites(), amas.rayon(), 2, 8)
        })
    })
}

fn claquage_100(eta: u32) -> &'static [Mesure] {
    static SERIES: [OnceLock<Vec<Mesure>>; 3] = [OnceLock::new(), OnceLock::new(), OnceLock::new()];
    SERIES[eta as usize - 1].get_or_init(|| serie_claquage(40, eta, 100, 6))
}

fn moyenne(serie: &[Mesure], champ: impl Fn(&Mesure) -> f64) -> f64 {
    serie.iter().map(champ).sum::<f64>() / serie.len() as f64
}

/// L'écart des moyennes `a - b` d'un champ.
fn ecart(a: &[Mesure], b: &[Mesure], champ: impl Fn(&Mesure) -> f64 + Copy) -> f64 {
    moyenne(a, champ) - moyenne(b, champ)
}

/// régression : (propriété de NOTRE modèle, mesurée sur NOTRE code) un amas d'Eden (`η = 0`) est PLEIN (V-D1-16). Sur les graines 1 à 24, 2000 sites,
/// `demi_cote` 50 (le rayon des amas va de 30 à 41) : la dimension de masse-rayon (rayons 6 à rayon/2, depuis le germe) est celle du disque plein de
/// mêmes rayons (`sites_du_disque`, la pente de `ln N(r)` pour les nombres exacts de points du disque) à `0,02` près pour chaque graine (mesuré sur 120
/// graines : de `-0,0148` à `+0,0015`), et la moyenne sur les 24 graines de cet écart est dans `[-0,004 ; +0,0005]` (`-0,0020 ± 3 σ/√24` avec `σ = 0,0023` ;
/// les cinq blocs de 24 graines : de `-0,0013` à `-0,0025`) ; la fraction de sites occupés du disque de rayon 10 est au moins `0,99` (mesuré : au moins
/// `0,9968`, en moyenne `0,9999`) et celle du disque de rayon 16 au moins `0,975` (mesuré : au moins `0,986`, en moyenne `0,998`), pour chaque graine. Un amas
/// ramifié (dimension 1,7) a une masse-rayon de 1,7 et un disque de rayon 10 à demi vide : il échoue ici. La dimension est de 2 à `0,01` près : l'amas d'Eden est
/// compact (voir la fiche, repère non testé).
///
/// Ne couvre pas : la rugosité de la frontière de l'amas ; que `η = 0` soit l'Eden de la littérature (la règle des candidates équiprobables est jugée par
/// `etape_d1_exact`) ; les amas de plus de 2000 sites.
#[test]
#[cfg_attr(not(feature = "etape-d1"), ignore)]
fn eden_est_plein() {
    static SERIE: OnceLock<Vec<(f64, f64, f64)>> = OnceLock::new();
    let serie = SERIE.get_or_init(|| {
        pour_chaque_graine(|graine| {
            let c = claquage(graine, 50, 0, 2000);
            let rayons: Vec<u32> = (6..=c.rayon() / 2).collect();
            let d = dimension_masse_rayon(c.sites(), (0, 0), &rayons);
            let xs: Vec<f64> = rayons.iter().map(|&r| logarithme(f64::from(r))).collect();
            let ys: Vec<f64> = rayons.iter().map(|&r| logarithme(sites_du_disque(r) as f64)).collect();
            let plein = |r: u32| masse_dans_rayon(c.sites(), (0, 0), r) as f64 / sites_du_disque(r) as f64;
            (d - pente(&xs, &ys), plein(10), plein(16))
        })
    });
    for (graine, &(ecart_au_plein, plein_10, plein_16)) in GRAINES.zip(serie) {
        assert!(
            (-0.02..=0.005).contains(&ecart_au_plein),
            "graine {graine} : masse-rayon moins disque plein = {ecart_au_plein}, hors de [-0,02 ; 0,005]"
        );
        assert!(plein_10 >= 0.99, "graine {graine} : le disque de rayon 10 n'est rempli qu'à {plein_10}");
        assert!(plein_16 >= 0.975, "graine {graine} : le disque de rayon 16 n'est rempli qu'à {plein_16}");
    }
    let moyen = serie.iter().map(|m| m.0).sum::<f64>() / serie.len() as f64;
    assert!((-0.004..=0.0005).contains(&moyen), "écart moyen au disque plein = {moyen}, hors de [-0,004 ; 0,0005]");
}

/// régression : (accord de trois estimateurs avec la DLA de l'étape 1, mesuré sur NOTRE modèle) `η = 1` est proche de la DLA mais PLUS COMPACT (V-D1-17).
/// Sur les graines 1 à 24, 300 sites chacun (`demi_cote` 45 pour le claquage, 200 pour la DLA), aucun amas n'atteint la frontière ; la corrélation (rayons 2 à
/// 8) moyenne du claquage est dans `[1,651 ; 1,703]` (`1,6767 ± 3 σ/√24`) ; les écarts de moyennes `claquage - DLA` sont dans `[0,039 ; 0,109]` pour la
/// corrélation (mesuré : `+0,074`, blocs de 24 de `+0,056` à `+0,095`), dans `[-0,097 ; +0,165]` pour la masse-rayon (mesuré : `+0,034`, un estimateur
/// bruité) et dans `[-0,018 ; +0,243]` pour `1/β` (mesuré : `+0,11`), et le rayon moyen de l'amas est inférieur de 1,5 à 6,7 sites à celui de la DLA
/// (mesuré : `-4,1`). Chaque bande est `écart mesuré ± 3 σ/√24` sur 120 graines, aucune n'a été élargie (le juge, sur 100 graines neuves 5001 à 5100 :
/// corrélation `1,6797`, écarts `+0,071`, `+0,027`, `+0,066`, `-3,9` : dans les bandes). Le sens de l'écart (plus compact) est vrai à 300, 600 et 1000
/// sites, et sa CAUSE est mesurée (voir `MODELE.md`) : la DLA de l'étape 1 colle dès qu'un voisin est dans l'amas, son ensemble absorbant est donc
/// l'amas ET ses sites périmètre, alors que le claquage met `φ = 0` sur l'amas seul ; un claquage où `C ∪ P` est absorbant (poids = flux) redonne la
/// DLA à `0,005` près en corrélation. Ce test rejette donc un modèle qui redeviendrait la DLA (écart `0`, hors de la bande) autant qu'un `η` décalé
/// d'un ou un champ oublié (poids uniformes : un amas plein ou une dimension de 1,45).
///
/// Ne couvre pas : la valeur d'UNE graine (la corrélation d'une graine neuve descend à `1,53` : seule la moyenne est testée) ; les amas de plus
/// de 300 sites ; la dimension 1,713 (non testée).
#[test]
#[cfg_attr(not(feature = "etape-d1"), ignore)]
fn eta_un_est_proche_de_la_dla_mais_plus_compact() {
    let (claq, dla) = (claquage_eta1_300(), dla_300());
    let corr = moyenne(claq, |m| m.corr);
    assert!((1.651..=1.703).contains(&corr), "corrélation moyenne du claquage η = 1 : {corr}, hors de [1,651 ; 1,703]");
    for (nom, valeur, bas, haut) in [
        ("corrélation", ecart(claq, dla, |m| m.corr), 0.039, 0.109),
        ("masse-rayon", ecart(claq, dla, |m| m.mr), -0.097, 0.165),
        ("1/β", ecart(claq, dla, |m| m.gir), -0.018, 0.243),
        ("rayon", ecart(claq, dla, |m| m.rayon), -6.7, -1.5),
    ] {
        assert!((bas..=haut).contains(&valeur), "claquage η = 1 moins DLA, {nom} : {valeur}, hors de [{bas} ; {haut}]");
    }
}

/// régression : plus `η` monte, plus l'amas est fin (V-D1-18). Sur les graines 1 à 24, 100 sites chacun, `demi_cote` 40 (aucun amas n'atteint la frontière, même à
/// `η = 3` : le rayon va jusqu'à 35), les moyennes sur les 24 graines de la corrélation (rayons 2 à 6) et de `1/β` BAISSENT quand `η` passe de 1 à 2 puis à 3,
/// et le rayon de l'amas AUGMENTE : corrélation `η = 1` moins `η = 2` au moins `0,10` (mesuré : `0,151`, blocs de 24 de `0,135` à `0,173`, bande `0,108` à
/// `0,194`), `η = 2` moins `η = 3` au moins `0,065` (mesuré : `0,118`, blocs `0,087` à `0,135`, bande `0,069` à `0,167`) ; `1/β` au moins `0,16` puis `0,11`
/// (mesuré : `0,264` et `0,189`, bandes `0,169` à `0,359` et `0,119` à `0,260`) ; rayon moyen au moins `2,5` puis `3,5` de plus (mesuré : `+4,1` et `+5,9`, bandes
/// `2,7` à `5,5` et `3,6` à `8,2`). Les tailles sont petites, le sens est celui de la théorie (plus `η` monte, plus la croissance se concentre aux pointes), pas
/// leurs valeurs : D2 compare les dimensions à Hastings. Un `η` décalé d'un ou un exposant sans effet (poids ne dépendant pas de `η`) annule ces écarts.
///
/// Ne couvre pas : les valeurs de `D(η)` (étape D2) ; `η` au-delà de 3 ; les amas de plus de 100 sites ; la masse-rayon (trop bruitée à 100 sites).
#[test]
#[cfg_attr(not(feature = "etape-d1"), ignore)]
fn plus_eta_monte_plus_l_amas_est_fin() {
    let (un, deux, trois) = (claquage_100(1), claquage_100(2), claquage_100(3));
    for (nom, un_deux, deux_trois, bas_un_deux, bas_deux_trois) in [
        ("corrélation", ecart(un, deux, |m| m.corr), ecart(deux, trois, |m| m.corr), 0.10, 0.065),
        ("1/β", ecart(un, deux, |m| m.gir), ecart(deux, trois, |m| m.gir), 0.16, 0.11),
        (
            "rayon (η plus grand moins η plus petit)",
            ecart(deux, un, |m| m.rayon),
            ecart(trois, deux, |m| m.rayon),
            2.5,
            3.5,
        ),
    ] {
        assert!(un_deux >= bas_un_deux, "{nom} : η = 1 contre η = 2, écart {un_deux} < {bas_un_deux}");
        assert!(deux_trois >= bas_deux_trois, "{nom} : η = 2 contre η = 3, écart {deux_trois} < {bas_deux_trois}");
    }
}

//! Étape 1 : la DLA, tests STATISTIQUES (une propriété vraie seulement en moyenne, testée en moyenne sur plusieurs graines, avec une
//! tolérance MESURÉE). Valeurs attendues : `references/etape-1-valeurs.md`, identifiants `V-STAT-xx`.
//!
//! La dynamique est fixée appel par appel par la doc de `src/dla.rs` : toute implémentation fidèle donne EXACTEMENT les mêmes amas, donc
//! EXACTEMENT les mêmes nombres ici (deux prototypes écrits séparément les ont donnés au bit près). Ces tests ne vérifient donc pas la
//! dynamique (les tests exacts le font) : ils disent ce que le modèle donne pour les deux grandeurs de la preuve de l'étape 1
//! (`FEUILLE-DE-ROUTE.md`), sur deux séries de 20 graines (1 à 20 et 101 à 120), avec 4000 sites par amas :
//!
//! - l'**exposant du rayon de giration** `β` : la pente de `ln R_g` contre `ln N`, `R_g` celui des `N` premiers sites du même amas, pour
//!   `N = 125, 250, 500, 1000, 2000, 4000` ; la dimension qu'il donne est `1/β` (`R_g ∝ N^β`, Meakin, Phys. Rev. A 27, 604, 1983) ;
//! - la **dimension de masse-rayon** `D_mr` : la pente de `ln N(r)` contre `ln r`, `N(r)` le nombre de sites à distance `<= r` du germe,
//!   sur TOUS les rayons entiers de 6 à `rayon / 2` (`rayon` = `Amas::rayon()`).
//!
//! **Ce qui a été mesuré** (deux prototypes indépendants, résultats identiques ; outils et sorties : `essai-1-outils/` et le rapport du
//! juge, `essai-1-juge.md`). Sur 250 graines (1 à 30, 101 à 120, 201 à 400), une graine : `D_mr` moyenne 1,670, écart-type 0,047 ;
//! `β` moyenne 0,5848, écart-type 0,020. Les dix moyennes de 20 graines disjointes des graines 201 à 400 vont de 1,659 à 1,684 pour
//! `D_mr` et de 0,5747 à 0,5906 pour `β` ; les séries des tests donnent 1,6625 et 0,5880 (1 à 20), 1,6796 et 0,5861 (101 à 120). Les
//! bandes, fixées par l'écrivain sur les graines 1 à 30 (moyenne `± 3 σ / √20`), sont `[1,642 ; 1,699]` et `[0,572 ; 0,598]` ; les dix
//! blocs des graines 201 à 400, qui n'ont pas servi à les fixer, y tombent tous ; aucune bande n'a été élargie, aucune graine choisie.
//!
//! **L'écart à la valeur publiée, dit franchement.**
//! - `β` : la moyenne 0,585 donne `1/β = 1,71`. Publié : Meakin (Phys. Rev. A 27, 604, 1983, résumé) donne en dimension 2 `D/d = 0,847 ±
//!   0,01`, soit `D = 1,694 ± 0,02` ; Tolman et Meakin (Phys. Rev. A 40, 428, 1989, résumé) `D = 1,715 ± 0,004` (hors réseau, jusqu'à
//!   10^6 particules) ; Loh (arXiv:1407.2586, 2014, texte intégral) : les petits amas circulaires de la DLA sur grille carrée ont
//!   `β = 0,585 = 1/1,71`. La bande `[0,572 ; 0,598]` (soit `1/β` dans `[1,672 ; 1,748]`) contient ces trois valeurs : c'est le test « juste ».
//! - `D_mr` : 1,670 à 4000 sites, **0,04 sous 1,71**. Ce n'est pas une faute des règles : le juge a refait la mesure avec un simulateur
//!   écrit à part, et (a) `D_mr` MONTE avec la taille de l'amas : 1,663 ± 0,007 (4000 sites, 40 graines), 1,682 ± 0,006 (10 000, 30
//!   graines), 1,689 ± 0,006 (30 000, 24 graines) ; (b) elle ne bouge pas (à l'erreur-type près) quand on remplace l'abandon à `2R` par un
//!   abandon à `10R`, par une marge de lancement de 20, ou par le retour exact du marcheur sur le cercle de lancement (noyau de Poisson,
//!   sans abandon) : 1,661, 1,670 et 1,667 à 4000 sites ; (c) elle dépend peu de la plage de rayons (1,64 à 1,68 à 4000 sites). C'est
//!   donc un effet de taille finie de la mesure de masse-rayon depuis le germe sur un amas de 4000 sites ; aucune source lue ne donne la
//!   valeur attendue à cette taille. Le test de `D_mr` est donc une **régression** : il fige ce que NOTRE modèle donne à 4000 sites,
//!   avec une bande mesurée ; il ne dit pas que la dimension de la DLA vaut 1,67.
//! - Le comptage de boîtes (côtés 2 à 32) de ces amas donne 1,49 et ne monte pas avec la taille (1,490, 1,496, 1,496 à 4000, 10 000,
//!   30 000 sites) : c'est une propriété de l'estimateur sur cette plage de côtés, pour laquelle aucune source n'a été trouvée (Sander,
//!   Contemp. Phys. 41, 203, 2000, dit la dimension de boîtes de la DLA « proche de 1,7 » pour de grands amas). Elle n'est donc PAS
//!   testée sur l'amas ; `dimension_boites` est testée sur des images exactes (`etape_1_mesures`).

use givre_lab::dla::Amas;
use givre_lab::math::logarithme;
use givre_lab::mesures::{dimension_masse_rayon, pente, rayon_de_giration};
use std::sync::OnceLock;

const PARTICULES: usize = 4000;
const TAILLES_GIRATION: [usize; 6] = [125, 250, 500, 1000, 2000, 4000];

/// `(D_mr, β)` pour la graine `graine`.
fn mesurer(graine: u64) -> (f64, f64) {
    let mut amas = Amas::new(graine, 400);
    amas.croitre_jusqu_a(PARTICULES);
    let sites = amas.sites();
    let rayons: Vec<u32> = (6..=amas.rayon() / 2).collect();
    let dimension = dimension_masse_rayon(sites, (0, 0), &rayons);
    let xs: Vec<f64> = TAILLES_GIRATION.iter().map(|&n| logarithme(n as f64)).collect();
    let ys: Vec<f64> = TAILLES_GIRATION.iter().map(|&n| logarithme(rayon_de_giration(&sites[..n]))).collect();
    (dimension, pente(&xs, &ys))
}

/// Les moyennes `(D_mr, β)` des deux séries de 20 graines, `1..=20` puis `101..=120`, calculées une seule fois pour les deux tests.
/// Les 40 amas sont indépendants : quatre fils en font chacun dix (le résultat de chaque graine ne dépend que d'elle, et les
/// moyennes se font ensuite dans l'ordre des graines : le résultat est le même, au bit près, qu'avec un seul fil).
fn moyennes() -> &'static [(f64, f64); 2] {
    static MOYENNES: OnceLock<[(f64, f64); 2]> = OnceLock::new();
    MOYENNES.get_or_init(|| {
        let graines: Vec<u64> = (1..=20).chain(101..=120).collect();
        let parts: Vec<Vec<(f64, f64)>> = std::thread::scope(|s| {
            let fils: Vec<_> =
                graines.chunks(10).map(|part| s.spawn(move || part.iter().map(|&g| mesurer(g)).collect())).collect();
            fils.into_iter().map(|f| f.join().expect("un fil de mesure a paniqué")).collect()
        });
        let mesures: Vec<(f64, f64)> = parts.concat();
        [&mesures[..20], &mesures[20..]].map(|serie| {
            let n = serie.len() as f64;
            (serie.iter().map(|m| m.0).sum::<f64>() / n, serie.iter().map(|m| m.1).sum::<f64>() / n)
        })
    })
}

/// juste : (propriété vraie en moyenne, tolérance mesurée, valeur publiée) sur les graines 1 à 20 puis 101 à 120 (4000 sites chacune),
/// l'exposant moyen du rayon de giration `β` est dans `[0,572 ; 0,598]`, soit `1/β` dans `[1,672 ; 1,748]`, qui contient les valeurs
/// publiées `1,694 ± 0,02` (Meakin 1983), `1,715 ± 0,004` (Tolman et Meakin 1989) et `β = 0,585` des petits amas sur grille carrée (Loh
/// 2014) ; la largeur de la bande est mesurée (voir l'en-tête du fichier) (V-STAT-01).
///
/// Ne couvre pas : la valeur asymptotique (la DLA sur grille carrée devient anisotrope au-delà de 10^5 sites et `β` y dérive vers 2/3,
/// Loh 2014) ; que chaque graine soit dans la bande (elle ne l'est pas : l'écart-type d'une graine est 0,02).
#[test]
#[cfg_attr(not(feature = "etape-1"), ignore)]
fn exposant_de_giration() {
    for (serie, &(_, beta)) in ["1 à 20", "101 à 120"].iter().zip(moyennes()) {
        assert!(
            (0.572..=0.598).contains(&beta),
            "graines {serie} : exposant du rayon de giration moyen {beta} hors de [0,572 ; 0,598]"
        );
    }
}

/// régression : (propriété vraie en moyenne, bande mesurée sur NOTRE modèle) sur les graines 1 à 20 puis 101 à 120 (4000 sites chacune),
/// la dimension de masse-rayon moyenne (rayons 6 à `rayon / 2`, depuis le germe) est dans `[1,642 ; 1,699]`. Elle est 0,04 sous la
/// valeur publiée 1,71 : un effet de taille finie mesuré par le juge (elle monte à 1,689 pour 30 000 sites, et ne dépend pas des
/// règles d'abandon), voir l'en-tête du fichier (V-STAT-02).
///
/// Ne couvre pas : la dimension de la DLA (ce test fige ce que le modèle donne à 4000 sites, il n'en fait pas une preuve) ; la
/// dynamique (les tests exacts) ; que chaque graine soit dans la bande (l'écart-type d'une graine est 0,047).
#[test]
#[cfg_attr(not(feature = "etape-1"), ignore)]
fn dimension_de_masse_rayon() {
    for (serie, &(dimension, _)) in ["1 à 20", "101 à 120"].iter().zip(moyennes()) {
        assert!(
            (1.642..=1.699).contains(&dimension),
            "graines {serie} : dimension de masse-rayon moyenne {dimension} hors de [1,642 ; 1,699]"
        );
    }
}

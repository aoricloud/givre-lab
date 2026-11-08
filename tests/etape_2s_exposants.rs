//! Étape 2s : l'exposant de croissance β des quatre modèles, mesuré par la pente de `ln w` contre `ln t` sur la courbe moyenne de
//! `w²`. Valeurs attendues : `references/etape-2s-valeurs.md`, identifiants `V-EXP-xx` (valeurs publiées, mesures du prototype et
//! tolérances).
//!
//! LA MESURE (identique pour les quatre modèles) : `L = 1024` colonnes ; la fenêtre, 11 instants `t` = 16, 24, 32, 48, 64, 96, 128, 192,
//! 256, 384, 512 (avant la saturation proprement dite, qui vient vers 4 000 à 8 000 couches pour le balistique, MAIS PAS avant l'effet de la
//! taille finie : à `t = 512` le `w²` moyen à `L = 1024` est déjà sous celui d'une bande huit fois plus large, de 10,5 % pour le balistique,
//! 6,9 % pour Eden (`L = 4096`), 4,3 % pour la relaxation ; mesure du juge, voir V-EXP-06) ; les graines 1 à `G`
//! avec `G` = 25 (dépôt aléatoire), 100 (relaxation), 300 (balistique), 150 (Eden) ; UNE pente sur la moyenne des `w²` des `G` graines
//! (`surface::exposant`). Les nombres de graines différent parce que le bruit de la mesure diffère d'un modèle à l'autre : le
//! dépôt aléatoire (β exact 1/2) en demande peu, le balistique le plus.
//!
//! LES TOLÉRANCES sont MESURÉES sur le prototype : 60 groupes indépendants de graines (le groupe `r` utilise les graines
//! `100000 + 1000 r + 1 …`, aucune ne chevauche celles du test), un exposant par groupe, avec les mêmes `G` ; chaque tolérance est
//! 5 écarts-types observés entre groupes, arrondis vers le haut au millième. Résultats (moyenne ± écart-type, minimum, maximum sur
//! les 60 groupes) :
//!
//! | modèle | β publié | mesuré (moyenne ± sd) | min – max | tolérance retenue | écart à la valeur publiée |
//! |---|---|---|---|---|---|
//! | dépôt aléatoire | 1/2 exact | 0,50036 ± 0,00181 | 0,4958 – 0,5046 | 0,5 ± 0,010 | +0,0004 |
//! | relaxation (Family) | 1/4 (EW) | 0,24631 ± 0,00364 | 0,2378 – 0,2531 | 0,246 ± 0,019 | -0,0037 |
//! | balistique | 1/3 (KPZ) | 0,28249 ± 0,00279 | 0,2768 – 0,2887 | 0,282 ± 0,014 | -0,0508 |
//! | Eden | 1/3 (KPZ) | 0,30300 ± 0,00410 | 0,2942 – 0,3105 | 0,303 ± 0,021 | -0,0303 |
//!
//! L'écart à la valeur publiée est franc pour les deux modèles KPZ : à cette taille et sur cette fenêtre (`t` de 16 à 512) l'exposant
//! effectif est sous 1/3 ; l'intervalle de tolérance ne contient donc PAS 1/3 pour ces deux modèles, et c'est dit : le test mesure ce que
//! fait le modèle à `L = 1024` sur cette fenêtre, pas la valeur asymptotique (règle 6 du brief commun). C'est le modèle et non une faute :
//! pour le balistique, Farnudi et Vvedensky (Phys. Rev. E 83 (2011) 020103, tableau II, texte lu) publient `β_L = 0,280 ± 0,002` à
//! `L = 1024` (fenêtre de croissance jusqu'à 284 couches) et un `β_L` qui ne monte vers 1/3 que lentement avec `L` (0,302 à 4096, 0,322 à
//! 65 536, 0,334 à 2^19), selon `β_L = β + A L^(-λ)`, `A < 0`. Le juge a refait la mesure avec un simulateur indépendant (autre générateur,
//! autre code, 60 groupes) : 0,4998 ; 0,2461 ; 0,2830 ; 0,3040 (V-EXP-06). Ce que le test N'affirme PAS : que `β` tende vers 1/3 (la même
//! fenêtre à `L = 8192` donne 0,2965 pour le balistique et 0,313 pour Eden à `L = 4096`, mais ce n'est pas testé : un test de tendance
//! assez sûr coûterait plusieurs fois le temps de ce fichier). Le dépôt aléatoire (exact) et la relaxation (1/4 dans l'intervalle) sont bien
//! dans leur intervalle.
//!
//! L'ordre (V-EXP-05) se teste sur des DIFFÉRENCES de groupes (mêmes graines pour les quatre modèles, comme ici), dont la moyenne et
//! l'écart-type sont aussi mesurés sur les 60 groupes : balistique - relaxation 0,03617 ± 0,00457 (minimum 0,0259), Eden - relaxation
//! 0,05669 ± 0,00579 (minimum 0,0442), dépôt - max(balistique, Eden) 0,19736 ± 0,00459 (minimum 0,1871), Eden - balistique
//! 0,02051 ± 0,00497 (de 0,0110 à 0,0330 : les deux modèles KPZ ne sont pas égaux à cette taille ; la différence est petite devant
//! celle qui les sépare de la relaxation). Les marges retenues sont moyenne - 5 écarts-types, arrondies à la baisse : 0,013, 0,027,
//! 0,17 ; la borne de |Eden - balistique| est moyenne + 5 écarts-types, arrondie à la hausse : 0,046.
//!
//! Ces tests sont lourds (de l'ordre de 10 s de calcul en tout) : les quatre mesures se font une fois, en parallèle, partagées par
//! tous les tests du fichier.

use givre_lab::surface::{Modele, exposant};
use std::sync::OnceLock;

/// `L`, les instants de la fenêtre et les nombres de graines, comme dans la doc du fichier.
const LARGEUR: usize = 1024;
const INSTANTS: [u64; 11] = [16, 24, 32, 48, 64, 96, 128, 192, 256, 384, 512];
const GRAINES: [u64; 4] = [25, 100, 300, 150];

/// Les quatre exposants mesurés, dans l'ordre dépôt, relaxation, balistique, Eden.
fn exposants() -> &'static [f64; 4] {
    static MESURES: OnceLock<[f64; 4]> = OnceLock::new();
    MESURES.get_or_init(|| {
        std::thread::scope(|portee| {
            let fils: Vec<_> = Modele::TOUS
                .into_iter()
                .zip(GRAINES)
                .map(|(modele, g)| {
                    portee.spawn(move || {
                        let graines: Vec<u64> = (1..=g).collect();
                        exposant(modele, LARGEUR, &graines, &INSTANTS)
                    })
                })
                .collect();
            let mut mesures = [0.0; 4];
            for (i, f) in fils.into_iter().enumerate() {
                mesures[i] = f.join().expect("mesure");
            }
            mesures
        })
    })
}

fn dans(valeur: f64, centre: f64, tolerance: f64, nom: &str) {
    assert!(
        (valeur - centre).abs() <= tolerance,
        "{nom} : β mesuré {valeur:.5}, attendu {centre} ± {tolerance} (L = {LARGEUR}, fenêtre {INSTANTS:?}, G = {GRAINES:?})"
    );
}

/// juste : (mesure statistique ; valeur publiée exacte) le dépôt aléatoire a `β = 1/2` (V-EXP-01) : `w² = t (1 - 1/L)` en moyenne, donc la pente de `ln w`
/// contre `ln t` vaut 0,5 aux fluctuations près ; le prototype mesure 0,50036 ± 0,00181 sur 60 groupes (écart à 1/2 : +0,0004) ;
/// tolérance 0,010 autour de 0,5 (5 écarts-types arrondis).
///
/// Ne couvre pas : la valeur à plus de 0,01 près ; une autre taille ou une autre fenêtre.
#[test]
#[cfg_attr(not(feature = "etape-2s"), ignore)]
fn exposant_du_depot_aleatoire() {
    dans(exposants()[0], 0.5, 0.010, "dépôt aléatoire");
}

/// juste : (mesure statistique ; valeur publiée : environ 1/4, classe d'Edwards-Wilkinson ; Family 1986 : 0,25 ± 0,01 selon une source
/// secondaire) la relaxation a `β ≈ 1/4` (V-EXP-02) ; le prototype mesure 0,24631 ± 0,00364 sur 60 groupes (écart à 1/4 : -0,0037) ; tolérance 0,019
/// autour de 0,246 (5 écarts-types arrondis). L'intervalle [0,227 ; 0,265] contient 1/4.
///
/// Ne couvre pas : l'exposant de rugosité α, ni le temps de saturation.
#[test]
#[cfg_attr(not(feature = "etape-2s"), ignore)]
fn exposant_de_la_relaxation() {
    dans(exposants()[1], 0.246, 0.019, "relaxation");
}

/// juste : (mesure statistique ; valeur publiée à cette taille : `β_L = 0,280 ± 0,002` à `L = 1024`, Farnudi et Vvedensky 2011,
/// tableau II ; asymptotique 1/3, classe KPZ ; mesure indépendante du juge : 0,2830) le dépôt balistique (V-EXP-03) ; le prototype mesure 0,28249 ± 0,00279 sur 60 groupes :
/// SOUS 1/3, de 0,051 (exposant effectif à `L = 1024` et `t` de 16 à 512, avec des corrections connues) ; tolérance 0,014 autour de
/// 0,282 (5 écarts-types arrondis) : l'intervalle [0,268 ; 0,296] ne contient pas 1/3 et le test ne prétend pas le contraire.
///
/// Ne couvre pas : l'approche de 1/3 aux grands temps et aux grandes tailles (mesurée par le juge, non testée).
#[test]
#[cfg_attr(not(feature = "etape-2s"), ignore)]
fn exposant_du_balistique() {
    dans(exposants()[2], 0.282, 0.014, "balistique");
}

/// juste : (mesure statistique ; asymptotique publiée 1/3, classe KPZ ; aucune valeur publiée à cette taille n'a été lue ; mesure
/// indépendante du juge, autre générateur : 0,3040) Eden (V-EXP-04) ; le prototype mesure 0,30300 ± 0,00410 sur 60 groupes : sous 1/3, de
/// 0,030 (mêmes raisons que le balistique) ; tolérance 0,021 autour de 0,303 (5 écarts-types arrondis) : [0,282 ; 0,324] ne contient
/// pas 1/3.
///
/// Ne couvre pas : l'approche de 1/3 aux grands temps ; les autres variantes d'Eden (B, C).
#[test]
#[cfg_attr(not(feature = "etape-2s"), ignore)]
fn exposant_d_eden() {
    dans(exposants()[3], 0.303, 0.021, "Eden");
}

/// juste : (mesure statistique ; recoupée par le simulateur indépendant du juge, V-EXP-06) l'ordre `β_dépôt > β_balistique ≈ β_Eden > β_relaxation` avec une marge mesurée (V-EXP-05, voir la doc du fichier) :
/// `β_dépôt - max(β_balistique, β_Eden) >= 0,17` (mesuré 0,197 ± 0,005) ; `β_balistique - β_relaxation >= 0,013` (0,0362 ± 0,0046) ;
/// `β_Eden - β_relaxation >= 0,027` (0,0567 ± 0,0058) ; `|β_Eden - β_balistique| <= 0,046` (0,0205 ± 0,0050). Chaque marge est la
/// moyenne mesurée moins 5 écarts-types (la borne de la dernière, plus 5), arrondie dans le sens qui élargit la tolérance de peu.
///
/// Ne couvre pas : que les deux modèles KPZ aient le même exposant (à cette taille, Eden est au-dessus de 0,02 en moyenne).
#[test]
#[cfg_attr(not(feature = "etape-2s"), ignore)]
fn ordre_des_exposants() {
    let [depot, relaxation, balistique, eden] = *exposants();
    assert!(depot - balistique.max(eden) >= 0.17, "dépôt {depot:.4}, balistique {balistique:.4}, Eden {eden:.4}");
    assert!(balistique - relaxation >= 0.013, "balistique {balistique:.4}, relaxation {relaxation:.4}");
    assert!(eden - relaxation >= 0.027, "Eden {eden:.4}, relaxation {relaxation:.4}");
    assert!((eden - balistique).abs() <= 0.046, "Eden {eden:.4}, balistique {balistique:.4}");
}

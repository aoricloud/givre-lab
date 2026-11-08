//! Étape 4 : la physique de la face, MESURÉE sur le prototype (au moins vingt graines par point) : de la nucléation sous
//! un seuil quand la face est lisse, de la linéarité quand elle est rugueuse, et de la rugosité qui croît avec la
//! température. Valeurs attendues : `references/etape-4-valeurs.md`, identifiants `V-PHY-xx`.
//!
//! **Ce que ces tests sont.** Ce sont quatre tests `régression :` (au sens de `AGENTS.md` : des résultats mesurés sur le
//! prototype, non des calculs exacts) : ils ne sont pas la preuve du modèle. La preuve (les tests `juste :`) est ailleurs :
//! `etape_4_exact.rs` (la dynamique égale à un simulateur naïf, essai après essai), `etape_4_mesures.rs` (la marche aléatoire
//! `J = 0` à son calcul exact, la face de 3 × 3 à l'énumération exacte de sa loi de Boltzmann). Les tests de ce fichier disent
//! que la physique que ces calculs exacts fixent a bien la forme que les sources décrivent.
//!
//! **Ce que disent les sources** (voir la fiche) : le modèle est le SOS à valeur absolue (ASOS), `H = K Σ |h_x − h_y|` avec
//! `K = J/kT` ; sa transition rugueuse est à `K_R = 0,80608(2)` (Hasenbusch et Pinn, J. Phys. A 30, 63 (1997), éq. 4 et
//! résultats ; recoupé par `u_r = e^(−2K_R) = 0,19945(1)`, arXiv hep-lat/9710079), soit `kT_R ≈ 1,24 J` ; à la transition,
//! la largeur croît comme `w² ≈ const + (1/π²) ln L` (Hasenbusch, Meyer, Pütz, arXiv hep-lat/9601011, éq. 20 et texte),
//! avec des corrections en `1/ln L`. Gilmer et Bennema (J. Appl. Phys. 43, 1347 (1972), résumé) observent, sous la
//! température de rugosité, une barrière de nucléation bidimensionnelle à faible sursaturation, et, au-dessus, une
//! croissance presque linéaire en la sursaturation ; Libbrecht (Rep. Prog. Phys. 68, 855 (2005), § 2.5, éq. 19-20) écrit la
//! croissance limitée par la nucléation `α ≈ A exp(−σ_crit/σ)` : **une vitesse qui n'est nulle pour aucune sursaturation**,
//! mais exponentiellement petite sous `σ_crit`. Les points choisis ici sont de part et d'autre de `K_R` : `J/kT = 2`, `1,5`
//! (24 seizièmes), `1,25`, `1` donnent `K > K_R` (face lisse, basse température), `0,75`, `0,5` (8 seizièmes), `0,375`
//! donnent `K < K_R` (face rugueuse, haute température). **Une seule valeur publiée est confrontée au modèle en nombres :
//! la pente `1/π²` de la largeur à la transition** (`pente_logarithmique_encadre_la_transition_rugueuse`) ; les vitesses
//! et les seuils ne le sont pas (les sources lisibles n'en donnent pas pour cette dynamique) ; chaque seuil de ce fichier
//! est une mesure du prototype, rapportée à côté de sa tolérance.
//!
//! **Ce que ces tests établissent, et ce qu'ils n'établissent pas.** La dynamique est fixée au bit par la doc et par
//! `etape_4_exact.rs` : une implémentation qui passe le simulateur naïf donne exactement les nombres du prototype, et ces
//! tests ne peuvent alors plus échouer. Ils ne gardent donc pas le code (le simulateur naïf le fait) : ils disent que **le
//! modèle ainsi spécifié** a, aux points choisis et aux tailles et durées écrites, la forme que les sources décrivent. Ils
//! n'établissent ni un seuil de nucléation (il n'y en a pas au sens strict : `v ∝ exp(−B/Δμ)` ; « quasi nulle » dépend de la
//! taille et de la durée), ni la loi en `exp(−B/Δμ)`, ni une pente de linéarité publiée, ni la position de `K_R` mieux qu'au
//! seizième.
//!
//! **Comment les tolérances ont été fixées.** Chaque point est la moyenne sur 20 graines fixes (1 à 20 : jamais une graine
//! choisie parce qu'elle passe). Les valeurs et les écarts rapportés dans chaque test viennent d'une calibration sur
//! **30 autres graines** (1001 à 1030) du prototype, faite avant d'écrire les seuils ; les tests passent aussi sur ces
//! 30 graines et sur les graines 1 à 20. **Robustesse** : les critères des quatre tests de ce fichier ont été rejoués sur 25
//! blocs de 20 graines que les tests ne contiennent pas (graines de 100000 à 102419, par blocs consécutifs de 100) : aucun
//! échec ; marges observées sur ces 25 blocs : rapport maximum sur minimum de `v/Δμ` de `1,012` à `1,047` (seuil `1,10`) ; plus petit
//! pas de `w²` selon la température `1,82` à `1,94` (seuil `1,3`) ; rapport rugueux 24/6 `1,66` à `1,77` (seuil `1,4`) ; rapport
//! lisse 24/12 `0,990` à `1,055` (plage `0,90 à 1,10`). Toute graine donne le même résultat à chaque lancer (le générateur est
//! déterministe) : la tolérance ne sert pas à absorber un bruit d'un lancer à l'autre, mais à dire quel écart d'une
//! implémentation fidèle à la doc serait physiquement acceptable, et quel écart ne l'est pas.

use givre_lab::cinetique::{Face, rugosite_moyenne, vitesse_croissance};
use givre_lab::math::EXP_NEG;

/// Les vingt graines des tests.
pub const GRAINES: [u64; 20] = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20];

/// Moyenne, écart-type (de l'échantillon, `n − 1`) et erreur-type de la moyenne d'une liste de mesures.
#[derive(Clone, Copy, Debug)]
pub struct Stat {
    pub moyenne: f64,
    pub ecart_type: f64,
    pub erreur: f64,
}

pub fn stat(valeurs: &[f64]) -> Stat {
    let n = valeurs.len() as f64;
    let moyenne = valeurs.iter().sum::<f64>() / n;
    let variance = valeurs.iter().map(|v| (v - moyenne) * (v - moyenne)).sum::<f64>() / (n - 1.0);
    Stat { moyenne, ecart_type: variance.sqrt(), erreur: (variance / n).sqrt() }
}

/// Une mesure par graine, les graines en parallèle (chaque mesure est indépendante : le résultat ne dépend pas de l'ordre).
fn par_graines<F: Fn(u64) -> f64 + Sync>(graines: &[u64], mesure: F) -> Vec<f64> {
    let mut sortie = vec![0.0; graines.len()];
    std::thread::scope(|s| {
        for (case, &graine) in sortie.iter_mut().zip(graines) {
            let mesure = &mesure;
            s.spawn(move || *case = mesure(graine));
        }
    });
    sortie
}

/// La vitesse de croissance de chaque graine : face de `l × l`, `transitoire` balayages jetés, puis la vitesse sur `fenetre`
/// balayages (`vitesse_croissance`).
pub fn vitesses(l: usize, j: u32, mu: i32, transitoire: usize, fenetre: usize, graines: &[u64]) -> Vec<f64> {
    par_graines(graines, |graine| {
        let mut face = Face::new(l, j, mu, graine);
        face.balayages(transitoire);
        vitesse_croissance(&mut face, fenetre)
    })
}

/// La rugosité moyenne `w²` d'équilibre (`Δμ = 0`) de chaque graine : `transitoire` balayages jetés, puis `rugosite_moyenne`
/// sur `fenetre` balayages.
pub fn rugosites(l: usize, j: u32, transitoire: usize, fenetre: usize, graines: &[u64]) -> Vec<f64> {
    par_graines(graines, |graine| {
        let mut face = Face::new(l, j, 0, graine);
        face.balayages(transitoire);
        rugosite_moyenne(&mut face, fenetre)
    })
}

/// Vrai si `b` dépasse `a` de plus de cinq erreurs-types combinées : `b.moyenne − a.moyenne > 5 · √(a.erreur² + b.erreur²)`.
pub fn depasse(a: Stat, b: Stat) -> bool {
    b.moyenne - a.moyenne > 5.0 * (a.erreur * a.erreur + b.erreur * b.erreur).sqrt()
}

// --- les paramètres des mesures (utilisés par les tests et par la calibration) ---

/// Face de la nucléation et de la linéarité : 16 × 16, 300 balayages jetés, vitesse sur 1000.
pub const L_VITESSE: usize = 16;
pub const TRANSITOIRE_VITESSE: usize = 300;
pub const FENETRE_VITESSE: usize = 1000;
/// Face de la rugosité en fonction de la température : 12 × 12, 1000 balayages jetés, moyenne sur 1000.
pub const L_RUGOSITE: usize = 12;
pub const TRANSITOIRE_RUGOSITE: usize = 1000;
pub const FENETRE_RUGOSITE: usize = 1000;

/// régression : valeurs mesurées sur le prototype, forme qualitative donnée par Gilmer et Bennema 1972 (résumé) : **à basse
/// température la face est lisse et ne croît pas sous un seuil de sursaturation** (V-PHY-01). Face lisse `J/kT = 1,5` (24
/// seizièmes, au-dessus de `K_R = 0,806`), `L = 16`, graines 1 à 20 : à `Δμ/kT = 0,25` (4 seizièmes) la vitesse est quasi
/// nulle ; à `Δμ/kT = 1,5` (24) elle vaut 0,033. On teste la signature de la nucléation, une vitesse par unité de sursaturation
/// `v/Δμ` qui **croît** avec `Δμ` (alors qu'elle est constante sur une face rugueuse, test suivant) : (i) `v/Δμ` en
/// `Δμ = 0,25` ne dépasse pas `1 %` de `v/Δμ` en `Δμ = 1,5` ; (ii) `v/Δμ` croît strictement (de plus de cinq erreurs-types
/// combinées) pour `Δμ/kT = 0,25, 1, 1,5, 2` ; (iii) la vitesse à `Δμ/kT = 1,5` dépasse 0,02.
///
/// Calibration (30 graines, 1001 à 1030 ; cinq blocs de 30 graines donnent les mêmes valeurs à l'erreur-type près) : `v/Δμ =
/// -0,000006 (écart-type 0,00003) ; 0,0098 (0,0013) ; 0,0219 (0,0010) ; 0,0346 (0,0006)` pour `Δμ/kT = 0,25 ; 1 ; 1,5 ; 2` ;
/// rapport `(v/Δμ)(0,25) / (v/Δμ)(1,5)` : au plus `3e-4` sur les cinq blocs. Tolérances : `1 %` pour le rapport (plus de 30
/// fois la valeur mesurée la plus grande, et 100 fois moins que la valeur `1` qu'une loi linéaire donnerait) ; `0,02` pour
/// (iii) (60 % de la mesure `0,033` : il refuse une face qui ne croît pas du tout).
///
/// **Ce que « sous un seuil » veut dire.** La croissance d'une face lisse par nucléation n'a pas de seuil strict : sa vitesse
/// suit `exp(−σ_crit/Δμ)` (Libbrecht 2005, éq. 19), jamais nulle. « Quasi nulle » veut dire ici : aucune croissance mesurable
/// sur `16 × 16` colonnes et 1000 balayages ; une face plus grande ou observée plus longtemps croîtrait aussi à `Δμ/kT = 0,25`.
/// Ce que le test établit est la **signature** de la nucléation : `v/Δμ` croît fortement avec `Δμ` (au lieu d'être constante).
///
/// Ne couvre pas : la valeur du seuil (une notion de taille et de durée finies), la loi en `exp(−B/Δμ)` et la valeur de `B`,
/// la dépendance en taille ou en durée de la nucléation (une face plus grande, ou observée plus longtemps, nuclée davantage).
#[test]
#[cfg_attr(not(feature = "etape-4"), ignore)]
fn face_lisse_ne_croit_pas_sous_un_seuil() {
    let j = 24;
    let point = |mu: i32| {
        let v = vitesses(L_VITESSE, j, mu, TRANSITOIRE_VITESSE, FENETRE_VITESSE, &GRAINES);
        stat(&v.iter().map(|x| x / (f64::from(mu) / 16.0)).collect::<Vec<f64>>())
    };
    let (bas, un, haut, deux) = (point(4), point(16), point(24), point(32));
    eprintln!(
        "lisse J=24 : v/Δμ à Δμ = 0,25 ; 1 ; 1,5 ; 2 : {:.6} ; {:.6} ; {:.6} ; {:.6}",
        bas.moyenne, un.moyenne, haut.moyenne, deux.moyenne
    );
    let vitesse_haut = haut.moyenne * 1.5;
    assert!(vitesse_haut > 0.02, "la face lisse ne croît pas du tout à Δμ/kT = 1,5 : v = {vitesse_haut}");
    assert!(bas.moyenne <= 0.01 * haut.moyenne, "v/Δμ à 0,25 = {} contre {} à 1,5", bas.moyenne, haut.moyenne);
    assert!(
        depasse(bas, un) && depasse(un, haut) && depasse(haut, deux),
        "v/Δμ ne croît pas : {bas:?} {un:?} {haut:?} {deux:?}"
    );
}

/// régression : valeurs mesurées sur le prototype (seul le cas `J = 0` est un calcul exact) : **à haute température la face est
/// rugueuse et la vitesse est linéaire en la sursaturation** (V-PHY-02). Face rugueuse `J/kT = 0,5` (8 seizièmes, sous `K_R`),
/// `L = 16`, graines 1 à 20, `Δμ/kT = 0,125 ; 0,25 ; 0,5 ; 1` (2, 4, 8, 16 seizièmes) : `v/Δμ` est constante. Critère : le plus
/// grand des quatre `v/Δμ` divisé par le plus petit vaut au plus `1,10`.
///
/// Calibration (30 graines, 1001 à 1030) : `v/Δμ = 0,2452 ; 0,2391 ; 0,2348 ; 0,2348` (écart-type de `v/Δμ` pour une graine : 0,012
/// à `Δμ = 0,125`, 0,0014 à `Δμ = 1`), rapport `1,044`. Sur cinq blocs de 30 graines : rapports `1,044 ; 1,021 ; 1,013 ; 1,029 ;
/// 1,041` (moyenne `1,03`, écart-type `0,013`). Tolérance `1,10` : plus de quatre écarts-types au-dessus de la moyenne (la
/// partie vraie de l'écart, `+3 %`, est la courbure de la loi, qui sature vers 1/2 ; le reste est du bruit de la plus faible
/// sursaturation) ; une loi qui sature à `J = 0` (`(1 − e^(−Δμ))/Δμ`) donnerait `1,49`, une face lisse plusieurs centaines. Et
/// la **mobilité** `v/Δμ` à `Δμ/kT = 0,5` **décroît quand `J` croît** : `J = 0` (valeur exacte `(1 − p)/2` divisée par `Δμ`,
/// `p = EXP_NEG[8] / 2^32` : un calcul, pas une mesure) `> J = 4 > J = 8`, chaque écart de plus de cinq erreurs-types
/// (calibration : `0,3935` exact ; `0,3468` (écart-type 0,0029) ; `0,2348` (0,0027)).
///
/// Ne couvre pas : la valeur de la pente (une mesure du prototype, non publiée), la linéarité pour `Δμ` grand (la loi sature
/// vers `1/2` : `v/Δμ` finit par décroître, déjà de 18 % entre `Δμ/kT = 0,0625` et `1` à `J = 4`). La linéarité « à 10 % sur
/// `Δμ/kT` de `0,125` à `1` » est un fait mesuré à `J/kT = 0,5` et `L = 16`, non un théorème : la réponse linéaire d'une face
/// rugueuse n'est attendue que pour `Δμ` petit, et le cas exact `J = 0` (`(1 − e^(−Δμ))/2`) n'est pas linéaire sur cette plage.
#[test]
#[cfg_attr(not(feature = "etape-4"), ignore)]
fn face_rugueuse_croit_lineairement() {
    let point = |j: u32, mu: i32| {
        let v = vitesses(L_VITESSE, j, mu, TRANSITOIRE_VITESSE, FENETRE_VITESSE, &GRAINES);
        stat(&v.iter().map(|x| x / (f64::from(mu) / 16.0)).collect::<Vec<f64>>())
    };
    let pentes: Vec<Stat> = [2, 4, 8, 16].into_iter().map(|mu| point(8, mu)).collect();
    let (plus_grande, plus_petite) = (
        pentes.iter().map(|s| s.moyenne).fold(f64::MIN, f64::max),
        pentes.iter().map(|s| s.moyenne).fold(f64::MAX, f64::min),
    );
    eprintln!(
        "rugueuse J=8 : v/Δμ = {:?}, rapport {:.4}",
        pentes.iter().map(|s| s.moyenne).collect::<Vec<f64>>(),
        plus_grande / plus_petite
    );
    assert!(plus_petite > 0.1, "la face rugueuse ne croît pas : v/Δμ = {plus_petite}");
    assert!(plus_grande / plus_petite <= 1.10, "v/Δμ n'est pas constante : rapport {}", plus_grande / plus_petite);
    // la mobilité décroît avec J, à Δμ/kT = 0,5 : J = 0 exact, puis J = 4 et J = 8 mesurés
    let p = EXP_NEG[8] as f64 / 4_294_967_296.0;
    let mobilite_j0 = (1.0 - p) / 2.0 / 0.5;
    let (j4, j8) = (point(4, 8), pentes[2]);
    eprintln!("mobilité à Δμ = 0,5 : J=0 {mobilite_j0:.4} ; J=4 {:.4} ; J=8 {:.4}", j4.moyenne, j8.moyenne);
    let exact = Stat { moyenne: mobilite_j0, ecart_type: 0.0, erreur: 0.0 };
    assert!(depasse(j4, exact) && depasse(j8, j4), "la mobilité ne décroît pas avec J");
}

/// régression : valeurs mesurées sur le prototype : **la rugosité d'équilibre `w²` croît avec la température** (V-PHY-03) : sur
/// la série `J/kT = 2 ; 1,5 ; 1 ; 0,75 ; 0,5 ; 0,375` (32, 24, 16, 12, 8, 6 seizièmes ; de la face lisse à la face rugueuse,
/// en passant `K_R = 0,806`), `Δμ = 0`, `L = 12`, graines 1 à 20, `w²` croît **strictement, chaque pas de plus de cinq
/// erreurs-types combinées**, et chaque pas multiplie `w²` par au moins `1,3`.
///
/// Calibration (30 graines, 1001 à 1030) : `w² = 0,00074 ; 0,0062 ; 0,090 ; 0,46 ; 1,33 ; 2,53` (écarts-types : 0,00014 ; 0,0006 ;
/// 0,010 ; 0,023 ; 0,075 ; 0,15) ; rapports entre températures consécutives `8,4 ; 14,5 ; 5,1 ; 2,9 ; 1,9`, le plus petit
/// `1,9`. Tolérance `1,3` : 68 % du plus petit pas mesuré ; elle refuse une rugosité qui ne bougerait presque pas, mais ne
/// dépend d'aucune valeur absolue.
///
/// Ne couvre pas : la loi de `w²` (en `ln L` au-dessus de `K_R`, voir le test suivant), la valeur de `K_R` (la grille de `J`
/// est au seizième : `K` varie de `0,0625`, trop grossier pour placer la transition), l'équilibre exact (`L = 12`, 1000
/// balayages : le test exact est celui de la face de 3 × 3 de `etape_4_mesures`).
#[test]
#[cfg_attr(not(feature = "etape-4"), ignore)]
fn rugosite_croit_avec_la_temperature() {
    let serie: Vec<(u32, Stat)> = [32_u32, 24, 16, 12, 8, 6]
        .into_iter()
        .map(|j| (j, stat(&rugosites(L_RUGOSITE, j, TRANSITOIRE_RUGOSITE, FENETRE_RUGOSITE, &GRAINES))))
        .collect();
    eprintln!("w² selon J : {:?}", serie.iter().map(|(j, s)| (*j, s.moyenne)).collect::<Vec<_>>());
    assert!(serie[0].1.moyenne > 0.0, "la face la plus lisse a une rugosité nulle : le modèle ne bouge pas");
    for paire in serie.windows(2) {
        let ((j_a, a), (j_b, b)) = (paire[0], paire[1]);
        assert!(depasse(a, b), "w² ne croît pas de J={j_a} à J={j_b} : {a:?} puis {b:?}");
        assert!(b.moyenne >= 1.3 * a.moyenne, "pas trop petit de J={j_a} à J={j_b} : {} puis {}", a.moyenne, b.moyenne);
    }
}

/// régression : valeurs mesurées sur le prototype ; la forme qualitative est celle de la rugosité, citée par Gilmer et Bennema 1972 et
/// Weeks et Gilmer 1979, **non lue** ; Hasenbusch et Pinn 1997 : « la largeur diverge en `ln L` dans la phase rugueuse et reste
/// finie dans la phase lisse ») : **au-dessus de la température de rugosité (`K < K_R`), la largeur de la face croît avec la
/// taille ; au-dessous (`K > K_R`), elle sature** (V-PHY-04). `Δμ = 0`, `L = 6, 12, 24`, graines 1 à 20 (1000 balayages
/// jetés, moyenne sur 1500). Face rugueuse `J/kT = 0,5` : `w²` croît de chaque doublement de `L` de plus de cinq erreurs-types
/// combinées, et `w²(24)/w²(6) >= 1,4`. Face lisse `J/kT = 1` : `w²(24)/w²(12)` reste dans `[0,90 ; 1,10]`.
///
/// Calibration (30 graines, 1001 à 1030) : rugueuse `w² = 0,988 ; 1,331 ; 1,696` (rapport `24/6` : `1,72` ; sur cinq blocs de 30
/// graines `1,68` à `1,72`) ; lisse `w² = 0,0819 ; 0,0889 ; 0,0881`, rapport `24/12` : `0,992` (sur cinq blocs : `0,992 ; 1,004 ;
/// 0,998 ; 1,030 ; 1,010`, écart-type `0,014`). Tolérances : `1,4` pour le rapport rugueux (82 % de la plus petite mesure) ;
/// `10 %` autour de 1 pour le lisse (plus de six écarts-types de ce rapport). À `L = 6` la face lisse est encore sous sa valeur
/// de saturation (`+8 %` jusqu'à `L = 12`) : le rapport lisse se prend entre 12 et 24.
///
/// Ne couvre pas : le coefficient du logarithme (la théorie le donne universel à `K_R` ; le prototype donne environ 0,1 par
/// doublement de `L` à `K = 0,8125`, non retenu faute de source lue), des tailles au-delà de 24, la position de `K_R`.
#[test]
#[cfg_attr(not(feature = "etape-4"), ignore)]
fn largeur_croit_avec_la_taille_au_dessus_de_la_transition() {
    let w2 = |l: usize, j: u32| stat(&rugosites(l, j, 1000, 1500, &GRAINES));
    let rugueuse: Vec<Stat> = [6, 12, 24].into_iter().map(|l| w2(l, 8)).collect();
    let lisse: Vec<Stat> = [6, 12, 24].into_iter().map(|l| w2(l, 16)).collect();
    eprintln!(
        "rugueuse w² : {:?} ; lisse w² : {:?}",
        rugueuse.iter().map(|s| s.moyenne).collect::<Vec<_>>(),
        lisse.iter().map(|s| s.moyenne).collect::<Vec<_>>()
    );
    assert!(depasse(rugueuse[0], rugueuse[1]) && depasse(rugueuse[1], rugueuse[2]), "w² rugueuse ne croît pas avec L");
    assert!(
        rugueuse[2].moyenne / rugueuse[0].moyenne >= 1.4,
        "rapport rugueux {}",
        rugueuse[2].moyenne / rugueuse[0].moyenne
    );
    let rapport_lisse = lisse[2].moyenne / lisse[1].moyenne;
    assert!((0.90..=1.10).contains(&rapport_lisse), "la face lisse ne sature pas : w²(24)/w²(12) = {rapport_lisse}");
}

/// régression, confrontée à une valeur publiée (critères mesurés sur le prototype ; références : `K_R = 0,80608(2)`, Hasenbusch
/// et Pinn 1997 ; pente `1/π²` à la transition, Hasenbusch, Meyer et Pütz, arXiv hep-lat/9601011, éq. 20) : **la pente de la
/// largeur en `ln L` passe par la valeur universelle `1/π²` entre `J/kT = 0,75` et `0,875`, là où les sources placent la
/// transition rugueuse** (V-PHY-05). Dans la phase rugueuse, `w²` croît comme `c · ln L` avec `c > 1/π²` ; à la transition,
/// `c = 1/π²` (aux corrections en `1/ln L` près : la pente mesurée sur de petites faces dépasse la valeur asymptotique) ; dans la
/// phase lisse, `c → 0`. On mesure `c = (w²(32) − w²(8)) / ln 4` (`Δμ = 0`, graines 1 à 20, 1000 balayages jetés, moyenne sur
/// 2000) à `J = 12, 13, 14` seizièmes (`K = 0,75 ; 0,8125 ; 0,875`) et on exige : `c(12) · π² >= 1,3` (rugueuse) ;
/// `c(13) · π²` dans `[0,9 ; 1,6]` (à `0,8 %` de `K_R`) ; `c(14) · π² <= 0,75` (lisse).
///
/// Calibration du juge (12 blocs de 20 graines, 7 600 000 et suivantes, mêmes durées) : `c · π² = 1,64 à 1,80` (`J = 12`),
/// `1,12 à 1,29` (`J = 13`, moyenne `1,19` : 19 % au-dessus de `1/π²`, dans le sens des corrections annoncées par la source),
/// `0,41 à 0,56` (`J = 14`) ; erreur-type d'un bloc `0,03 à 0,07`. Marges : au moins 4 erreurs-types de chaque borne ; un
/// seizième de `J` en plus ou en moins déplace `c(13) · π²` vers `0,47` ou `1,75`, hors de la plage. Avec la convention
/// « chaque paire comptée deux fois » (`K` doublé), les trois faces seraient lisses et `c(12)` échouerait : ce test est
/// l'appui numérique de la convention de la doc, que le texte de Hasenbusch et Pinn n'écrit en toutes lettres que pour Ising.
///
/// Ne couvre pas : la valeur de `K_R` mieux qu'au seizième (la grille de `J`), l'amplitude exacte des corrections en
/// `1/ln L`, des tailles au-delà de 32, la dynamique (une propriété d'équilibre : toute dynamique de même loi d'équilibre la
/// donnerait).
#[test]
#[cfg_attr(not(feature = "etape-4"), ignore)]
fn pente_logarithmique_encadre_la_transition_rugueuse() {
    let pi2 = std::f64::consts::PI * std::f64::consts::PI;
    let pente = |j: u32| {
        let petite = stat(&rugosites(8, j, 1000, 2000, &GRAINES)).moyenne;
        let grande = stat(&rugosites(32, j, 1000, 2000, &GRAINES)).moyenne;
        (grande - petite) / 4.0_f64.ln() * pi2
    };
    let (rugueuse, transition, lisse) = (pente(12), pente(13), pente(14));
    eprintln!("c · π² : J=12 {rugueuse:.3} ; J=13 {transition:.3} ; J=14 {lisse:.3}");
    assert!(rugueuse >= 1.3, "J/kT = 0,75 : la face n'est pas rugueuse, c · π² = {rugueuse}");
    assert!(
        (0.9..=1.6).contains(&transition),
        "J/kT = 0,8125 : c · π² = {transition}, loin de 1 (transition rugueuse)"
    );
    assert!(lisse <= 0.75, "J/kT = 0,875 : la face n'est pas lisse, c · π² = {lisse}");
}

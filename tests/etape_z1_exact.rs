//! Étape Z1 : la corrélation densité-densité (`givre_lab::correlation`), tests EXACTS. Valeurs attendues :
//! `references/etape-z1-valeurs.md`, identifiants `V-Z1-xx` cités dans chaque test.
//!
//! Tout ce fichier est exact : des entiers (comptes de paires, cases du disque), des fractions écrites à la main, des formules
//! fermées calculées dans le test par des boucles de quelques lignes, et des valeurs calculées par un outil indépendant (Python
//! 3.13, `fractions` et `decimal` à 60 chiffres, un compteur de paires naïf écrit d'après la doc seule : `essai-z1-outils/z1_ref.py`
//! des notes de la session). Aucune valeur ne vient du code jugé. Les seules tolérances sont `1e-12` sur des dimensions, qui valent
//! exactement une valeur calculée à l'arrondi près du `logarithme` (`5e-16`).
//!
//! Le compteur naïf de paires, la formule de `C(r)` et les formes sont dans `etape_z1_commun`.

mod etape_z1_commun;

use std::panic::catch_unwind;

use etape_z1_commun::{
    amas_au_hasard, bits, carre, correlation_attendue, disque_naif, paires_naives, proche, trait_horizontal,
};
use givre_lab::correlation::{
    RAYON_DISQUE_MAX, RAYON_MAX, correlation, dimension_correlation, etendue, fenetre, mesure, paires_par_distance2,
    sites_du_disque,
};

/// Les dimensions calculées par Python pour les cas à la main de `dimension_correlation` (`decimal`, 60 chiffres, moindres carrés sur
/// les points `(ln r, ln C(r))` exacts), arrondies à 17 chiffres.
const D_CARRE_3_RAYONS_1_2: f64 = 1.530_514_716_698_779_8;
const D_CARRE_3_RAYONS_1_2_3: f64 = 1.261_250_216_840_658_3;
const D_CARRE_5_RAYONS_1_A_4: f64 = 1.556_078_371_491_421_2;
const D_TRAIT_101_RAYONS_2_4_8_16: f64 = 0.946_994_203_486_816_4;
const D_TRAIT_201_FENETRE: f64 = 0.959_696_525_037_499_8;

/// Un `Result` de `catch_unwind` : vrai si le calcul a paniqué.
fn panique<R>(calcul: impl FnOnce() -> R + std::panic::UnwindSafe) -> bool {
    catch_unwind(calcul).is_err()
}

// ------------------------------------------------------------------------------------------------------------- le disque

/// juste : `sites_du_disque` pour les rayons 0 à 12 (la suite A000328 de l'OEIS : `1, 5, 13, 29, 49, 81, 113, 149, 197, 253, 317, 377,
/// 441`), puis contre le compteur case par case pour tous les rayons de 0 à 60, puis les valeurs de Python (comptage par colonnes
/// avec `isqrt`, recomptées case par case jusqu'à 1000) pour les rayons 100, 370, 400, 1000, 4096 (les rayons de la fenêtre et du
/// plafond de la page) (V-Z1-01). Un disque qui oublie son centre, qui exclut son bord (`<` pour `<=`) ou qui compte un quart de
/// disque se voit dès le rayon 1.
///
/// Ne couvre pas : les très grands rayons (voir `disque_au_dela_de_u32`).
#[test]
#[cfg_attr(not(feature = "etape-z1"), ignore)]
fn disque_petits_et_moyens_rayons() {
    let gauss: Vec<u64> = (0..=12).map(sites_du_disque).collect();
    assert_eq!(gauss, [1, 5, 13, 29, 49, 81, 113, 149, 197, 253, 317, 377, 441]);
    for r in 0..=60 {
        assert_eq!(sites_du_disque(r), disque_naif(r), "rayon {r}");
    }
    for (r, attendu) in [(100, 31_417), (370, 430_085), (400, 502_625), (1000, 3_141_549), (4096, 52_706_921)] {
        assert_eq!(sites_du_disque(r), attendu, "rayon {r}");
    }
}

/// juste : `sites_du_disque` ne déborde pas un `u32` : le rayon 36 974 donne `4 294 797 601` (juste sous `u32::MAX`), le rayon 36 975
/// donne `4 295 030 009` (juste au-dessus), puis 65 535 et 65 536 (où `r²` ne tient plus dans un `u32`), 100 000 et `2^20` (valeurs de
/// Python, comptage par colonnes avec `isqrt`) (V-Z1-02). `RAYON_DISQUE_MAX` vaut `2^20` ; au-delà, panique.
///
/// Ne couvre pas : le temps du calcul pour de très grands rayons.
#[test]
#[cfg_attr(not(feature = "etape-z1"), ignore)]
fn disque_au_dela_de_u32() {
    assert_eq!(RAYON_DISQUE_MAX, 1 << 20);
    for (r, attendu) in [
        (36_974, 4_294_797_601_u64),
        (36_975, 4_295_030_009),
        (65_535, 13_492_624_829),
        (65_536, 13_493_036_169),
        (100_000, 31_415_925_457),
        (1 << 20, 3_454_217_649_829),
    ] {
        assert_eq!(sites_du_disque(r), attendu, "rayon {r}");
    }
    assert!(panique(|| sites_du_disque(RAYON_DISQUE_MAX + 1)), "rayon au-delà de RAYON_DISQUE_MAX");
}

// ------------------------------------------------------------------------------------------------------------- les paires

/// Un tableau de paires où seuls certains indices sont non nuls : `(indice, valeur)`, le tableau ayant `rayon_max² + 1` éléments.
fn tableau(rayon_max: u32, non_nuls: &[(usize, u64)]) -> Vec<u64> {
    let mut paires = vec![0; (rayon_max * rayon_max + 1) as usize];
    for &(indice, valeur) in non_nuls {
        paires[indice] = valeur;
    }
    paires
}

/// juste : `paires_par_distance2` sur des petits amas comptés à la main (et recomptés par le compteur de Python) (V-Z1-03) : deux sites
/// `(0,0)`, `(3,4)` et `rayon_max 5` : 2 paires ordonnées à l'indice 25 ; trois sites alignés `(0,0)`, `(1,0)`, `(3,0)` et `rayon_max 3` :
/// 2 aux indices 1, 4 et 9 ; le L `(0,0)`, `(1,0)`, `(0,1)` et `rayon_max 2` : 4 à l'indice 1 et 2 à l'indice 2 ; trois fois `(5,5)` et
/// `(6,5)` avec `rayon_max 1` : 6 à l'indice 0 (les trois répétitions forment 3 paires, donc 6 ordonnées) et 6 à l'indice 1 ; le carré 2 × 2
/// avec `rayon_max 2` : 8 à l'indice 1, 4 à l'indice 2 ; le carré 3 × 3 avec `rayon_max 3` : 24, 16, 12, 16, 4 aux indices 1, 2, 4, 5, 8.
/// Une paire compte deux fois (ordonnée), un site n'est pas apparié à lui-même, une paire à distance exactement `rayon_max` est comptée, une
/// paire à distance supérieure ne l'est pas (le trait `(0,0)`, `(3,4)` avec `rayon_max 4` ne compte rien).
///
/// Ne couvre pas : les gros amas (voir `paires_contre_le_compteur_naif`).
#[test]
#[cfg_attr(not(feature = "etape-z1"), ignore)]
fn paires_a_la_main() {
    assert_eq!(paires_par_distance2(&[(0, 0), (3, 4)], 5), tableau(5, &[(25, 2)]));
    assert_eq!(paires_par_distance2(&[(0, 0), (3, 4)], 4), tableau(4, &[]));
    assert_eq!(paires_par_distance2(&[(0, 0), (1, 0), (3, 0)], 3), tableau(3, &[(1, 2), (4, 2), (9, 2)]));
    assert_eq!(paires_par_distance2(&[(0, 0), (1, 0), (0, 1)], 2), tableau(2, &[(1, 4), (2, 2)]));
    assert_eq!(paires_par_distance2(&[(5, 5), (5, 5), (5, 5), (6, 5)], 1), tableau(1, &[(0, 6), (1, 6)]));
    assert_eq!(paires_par_distance2(&carre(2), 2), tableau(2, &[(1, 8), (2, 4)]));
    assert_eq!(paires_par_distance2(&carre(3), 3), tableau(3, &[(1, 24), (2, 16), (4, 12), (5, 16), (8, 4)]));
    // Le compteur naïf des tests est lui-même ancré sur ces mêmes tableaux (il sert de référence aux amas au hasard).
    assert_eq!(paires_naives(&[(0, 0), (1, 0), (3, 0)], 3), tableau(3, &[(1, 2), (4, 2), (9, 2)]));
    assert_eq!(paires_naives(&[(5, 5), (5, 5), (5, 5), (6, 5)], 1), tableau(1, &[(0, 6), (1, 6)]));
    assert_eq!(paires_naives(&carre(3), 3), tableau(3, &[(1, 24), (2, 16), (4, 12), (5, 16), (8, 4)]));
}

/// juste : `paires_par_distance2` donne EXACTEMENT les comptes du compteur naïf (double boucle sur les paires ordonnées, `i128`, écrit dans
/// `etape_z1_commun`) sur des amas au hasard (V-Z1-04) : 0, 1, 2, 5, 40, 150 et 300 sites tirés dans des carrés de demi-côté 1, 4, 12 et 40
/// (donc beaucoup de répétitions dans les petits carrés, des égalités de distances dans les grands), pour `rayon_max` 0, 1, 2, 3, 7, 12 et 30,
/// sur deux graines chacun (les graines 1 et 2, écrites ici, jamais choisies). Une paire comptée une fois ou trois fois, un `i == j` compté, une
/// distance décalée d'une case, la distance prise au lieu de son carré, un `<` pour un `<=` au bord du disque, un ordre des sites qui change le
/// compte, un site de départ oublié : chacun change au moins un de ces tableaux.
///
/// Ne couvre pas : les coordonnées de l'ordre de `2^31` (voir `paires_limites`), la vitesse.
#[test]
#[cfg_attr(not(feature = "etape-z1"), ignore)]
fn paires_contre_le_compteur_naif() {
    for n in [0_usize, 1, 2, 5, 40, 150, 300] {
        for demi in [1_i32, 4, 12, 40] {
            for graine in [1_u64, 2] {
                let sites = amas_au_hasard(graine * 1000 + n as u64 * 7 + demi as u64, n, demi);
                for rayon_max in [0_u32, 1, 2, 3, 7, 12, 30] {
                    assert_eq!(
                        paires_par_distance2(&sites, rayon_max),
                        paires_naives(&sites, rayon_max),
                        "n {n}, demi-côté {demi}, graine {graine}, rayon_max {rayon_max}"
                    );
                }
            }
        }
    }
}

/// juste : les cas limites que la doc fixe (V-Z1-05) : une liste vide ou d'un seul site donne un tableau de zéros de `rayon_max² + 1`
/// éléments ; `rayon_max 0` donne un tableau d'un élément (les répétitions) ; avec `rayon_max` assez grand pour tout contenir, la somme des
/// éléments est `n (n - 1)` ; des sites en `i32::MIN` et `i32::MAX` (écart de `2^32 - 1`, que ni un `i32` ni un `u32` ne portent au carré) sont
/// trop éloignés pour compter et ne font pas paniquer, et la paire proche voisine d'eux est comptée ; `rayon_max = RAYON_MAX` (4096) est
/// accepté (le tableau a `4096² + 1` éléments) et `RAYON_MAX + 1` fait paniquer, de même que `correlation` d'une liste vide.
///
/// Ne couvre pas : la valeur exacte de `correlation` (voir `correlation_a_la_main`).
#[test]
#[cfg_attr(not(feature = "etape-z1"), ignore)]
fn paires_limites() {
    assert_eq!(paires_par_distance2(&[], 3), vec![0; 10]);
    assert_eq!(paires_par_distance2(&[(4, -2)], 3), vec![0; 10]);
    assert_eq!(paires_par_distance2(&[(1, 1), (1, 1)], 0), vec![2]);
    assert_eq!(paires_par_distance2(&[(1, 2), (3, 4), (5, 6)], 0), vec![0]);

    let sites = amas_au_hasard(9, 60, 5);
    let toutes = paires_par_distance2(&sites, 15);
    assert_eq!(
        toutes.iter().sum::<u64>(),
        60 * 59,
        "avec un rayon qui contient tout, chaque paire ordonnée est comptée une fois"
    );

    let extremes = [(i32::MIN, i32::MIN), (i32::MAX, i32::MAX), (i32::MIN, i32::MAX), (0, 0), (1, 0), (i32::MAX, 0)];
    assert_eq!(paires_par_distance2(&extremes, 2), tableau(2, &[(1, 2)]));
    assert_eq!(paires_par_distance2(&[(i32::MIN, 0), (i32::MAX, 0)], 4), tableau(4, &[]));

    let grand = paires_par_distance2(&[(0, 0), (1, 0)], RAYON_MAX);
    assert_eq!(RAYON_MAX, 4096);
    assert_eq!(grand.len(), 4096 * 4096 + 1);
    assert_eq!(grand[1], 2);
    assert_eq!(grand.iter().sum::<u64>(), 2);
    assert!(panique(|| paires_par_distance2(&[(0, 0)], RAYON_MAX + 1)), "rayon_max au-delà de RAYON_MAX");
    assert!(panique(|| correlation(&[], 3)), "aucun site");
    assert!(panique(|| correlation(&[(0, 0)], RAYON_MAX + 1)), "rayon_max au-delà de RAYON_MAX");
}

// ------------------------------------------------------------------------------------------------------------- C(r)

/// juste : `correlation` sur des petits amas dont la réponse est une fraction écrite à la main (V-Z1-06), au bit près (une seule division de
/// deux entiers exacts, qui donne l'arrondi correct de la fraction) : un site seul `0, 0, 0` ; deux sites `(0,0)`, `(3,4)` : `0, 0, 0, 0, 1/80`
/// (le disque de rayon 5 a 81 cases) ; trois alignés `(0,0)`, `(1,0)`, `(3,0)` : `1/6, 1/9, 1/14` ; le L : `1/3, 1/6` ; trois fois `(5,5)` et
/// `(6,5)` : `3/8` (les répétitions comptent dans `n = 4` mais pas dans le numérateur : `6 / (4 × 4)`) ; le carré 2 × 2 : `1/2, 1/4` ; le
/// carré 3 × 3 : `2/3, 13/27, 2/7`. Un diviseur `n - 1`, un disque sans le retrait du centre, ou un disque trop petit change au moins une de ces
/// fractions ; `rayon_max 0` donne une liste vide.
///
/// Ne couvre pas : les grands amas (voir `correlation_formules_fermees`).
#[test]
#[cfg_attr(not(feature = "etape-z1"), ignore)]
fn correlation_a_la_main() {
    assert_eq!(correlation(&[(7, -3)], 3), [0.0, 0.0, 0.0]);
    assert_eq!(correlation(&[(0, 0), (3, 4)], 5), [0.0, 0.0, 0.0, 0.0, 1.0 / 80.0]);
    assert_eq!(correlation(&[(0, 0), (1, 0), (3, 0)], 3), [1.0 / 6.0, 1.0 / 9.0, 1.0 / 14.0]);
    assert_eq!(correlation(&[(0, 0), (1, 0), (0, 1)], 2), [1.0 / 3.0, 1.0 / 6.0]);
    assert_eq!(correlation(&[(5, 5), (5, 5), (5, 5), (6, 5)], 1), [3.0 / 8.0]);
    assert_eq!(correlation(&carre(2), 2), [1.0 / 2.0, 1.0 / 4.0]);
    assert_eq!(correlation(&carre(3), 3), [2.0 / 3.0, 13.0 / 27.0, 2.0 / 7.0]);
    assert!(correlation(&carre(3), 0).is_empty());
}

/// juste : `correlation` d'un carré plein de côté `s` pour `s = 1..=12` et `s = 20`, et d'un trait de `L` sites pour `L = 2, 3, 10, 101`, au
/// bit près, avec les formules fermées de la doc (V-Z1-07) : le déplacement `(dx, dy)` non nul apparaît `(s - |dx|)(s - |dy|)` fois dans le
/// carré (pour `|dx|, |dy| < s`), le déplacement `(dx, 0)` apparaît `L - |dx|` fois dans le trait ; `cumul(r)` est la somme de ces nombres sur
/// les déplacements de longueur `<= r`, le dénominateur est `n × (disque(r) - 1)` (disque compté case par case). Les rayons vont de 1 à `2 s + 1`
/// (jusqu'au-delà du carré, où le cumul est constant : `n (n - 1)`). Les sites du carré sont rangés par colonnes, ceux du trait par abscisse
/// croissante, et le calcul les mélange (V-Z1-07 : l'ordre ne compte pas).
///
/// Ne couvre pas : une forme sans formule fermée (voir `etape_z1_formes`).
#[test]
#[cfg_attr(not(feature = "etape-z1"), ignore)]
fn correlation_formules_fermees() {
    for s in (1..=12_i32).chain([20]) {
        let n = (s * s) as usize;
        let rayon_max = (2 * s + 1) as u32;
        let mut cumul_par_carre = vec![0_u64; (rayon_max * rayon_max + 1) as usize];
        for dx in -(s - 1)..s {
            for dy in -(s - 1)..s {
                let d2 = (dx * dx + dy * dy) as usize;
                if (dx, dy) != (0, 0) && d2 < cumul_par_carre.len() {
                    cumul_par_carre[d2] += ((s - dx.abs()) * (s - dy.abs())) as u64;
                }
            }
        }
        let attendu = correlation_attendue(n, &cumul_par_carre, rayon_max);
        let obtenu = correlation(&etape_z1_commun::melanger(&carre(s), s as u64), rayon_max);
        assert_eq!(bits(&obtenu), bits(&attendu), "carré de côté {s}");
        assert_eq!(bits(&correlation(&carre(s), rayon_max)), bits(&attendu), "carré de côté {s}, rangé par colonnes");
    }
    for longueur in [2_i32, 3, 10, 101] {
        let rayon_max = 12_u32.min(longueur as u32 + 2);
        let mut paires = vec![0_u64; (rayon_max * rayon_max + 1) as usize];
        for dx in 1..longueur {
            if (dx * dx) as usize <= (rayon_max * rayon_max) as usize {
                paires[(dx * dx) as usize] = 2 * (longueur - dx) as u64;
            }
        }
        let attendu = correlation_attendue(longueur as usize, &paires, rayon_max);
        assert_eq!(bits(&correlation(&trait_horizontal(longueur), rayon_max)), bits(&attendu), "trait de {longueur}");
    }
}

/// juste : le dénominateur `n × (disque(r) - 1)` dépasse `u32::MAX` dès 10 000 sites et `r = 370` : pour le carré plein de 100 × 100 sites (10 000
/// sites) et `rayon_max 400`, tous les `C(r)` pour `r >= 141` (où le cumul est constant : `n (n - 1) = 99 990 000`) valent
/// `99 990 000 / (10 000 × (disque(r) - 1))`, au bit près ; en particulier `C(370) = 9999/430084` et `C(400) = 9999/502624` (dénominateurs
/// `4 300 840 000` et `5 026 240 000`, tous deux `> 2^32`, valeurs de Python) (V-Z1-08). Un produit calculé en `u32` déborde (le test échoue par
/// un résultat faux, ou une panique de débordement).
///
/// Ne couvre pas : un amas plus grand (le coût est en `n²`).
#[test]
#[cfg_attr(not(feature = "etape-z1"), ignore)]
fn denominateur_au_dela_de_u32() {
    let sites = carre(100);
    let c = correlation(&sites, 400);
    assert_eq!(c.len(), 400);
    assert_eq!(c[370 - 1], 99_990_000.0 / 4_300_840_000.0);
    assert_eq!(c[400 - 1], 99_990_000.0 / 5_026_240_000.0);
    for r in 141..=400_u32 {
        let denominateur = 10_000_u128 * u128::from(disque_naif(r) - 1);
        let attendu = 99_990_000.0_f64 / (denominateur as f64);
        assert_eq!(c[(r - 1) as usize].to_bits(), attendu.to_bits(), "rayon {r}");
    }
}

/// juste : `dimension_correlation` sur des cas dont la dimension est calculée à part (V-Z1-09), à `1e-12` : le carré 2 × 2 avec les rayons `[1, 2]`
/// vaut exactement 1 (`C(1) = 1/2`, `C(2) = 1/4`, la pente de `ln C` contre `ln r` est `-1`) ; le carré 3 × 3 avec `[1, 2]`, puis `[1, 2, 3]` ;
/// le carré 5 × 5 avec `[1, 2, 3, 4]` ; le trait de 101 sites avec `[2, 4, 8, 16]` (valeurs de Python : fractions exactes, `ln` à 60 chiffres,
/// moindres carrés). Les rayons donnés dans un autre ordre, ou non consécutifs, donnent la même valeur : `[3, 1, 2]` pour le carré 3 × 3.
/// Le résultat est `2 +` la pente, et non la pente (le carré 2 × 2 donnerait `-1`).
///
/// Ne couvre pas : la valeur sur un amas fractal (voir `etape_z1_formes`).
#[test]
#[cfg_attr(not(feature = "etape-z1"), ignore)]
fn dimension_a_la_main() {
    proche(dimension_correlation(&carre(2), &[1, 2]), 1.0, 1e-12, "carré 2 × 2, rayons 1 et 2");
    proche(dimension_correlation(&carre(3), &[1, 2]), D_CARRE_3_RAYONS_1_2, 1e-12, "carré 3 × 3, rayons 1 et 2");
    proche(dimension_correlation(&carre(3), &[1, 2, 3]), D_CARRE_3_RAYONS_1_2_3, 1e-12, "carré 3 × 3, rayons 1 à 3");
    proche(
        dimension_correlation(&carre(3), &[3, 1, 2]),
        D_CARRE_3_RAYONS_1_2_3,
        1e-12,
        "carré 3 × 3, rayons dans le désordre",
    );
    proche(dimension_correlation(&carre(5), &[1, 2, 3, 4]), D_CARRE_5_RAYONS_1_A_4, 1e-12, "carré 5 × 5, rayons 1 à 4");
    proche(
        dimension_correlation(&trait_horizontal(101), &[2, 4, 8, 16]),
        D_TRAIT_101_RAYONS_2_4_8_16,
        1e-12,
        "trait de 101 sites, rayons 2, 4, 8, 16",
    );
}

/// juste : `dimension_correlation` fait paniquer les entrées sans dimension (doc) : moins de deux rayons, deux rayons égaux, un rayon nul, un
/// rayon au-delà de `RAYON_MAX`, aucun site, et un `C(r)` nul (deux sites `(0,0)`, `(50,0)` et les rayons `[1, 2]` : aucune paire à distance
/// `<= 2`, le logarithme de 0 n'existe pas) ; et deux entrées valides au bord du refus sont acceptées (deux rayons distincts `[1, 2]` d'un carré
/// 2 × 2 ; les rayons `[2, 2, 3]`, dont deux égaux mais pas tous : la pente existe), pour qu'un corps qui panique toujours n'y passe pas (V-Z1-10).
///
/// Ne couvre pas : le texte des messages.
#[test]
#[cfg_attr(not(feature = "etape-z1"), ignore)]
fn dimension_entrees_refusees() {
    let petit = carre(2);
    assert!(panique(|| dimension_correlation(&carre(4), &[2])), "un seul rayon");
    assert!(panique(|| dimension_correlation(&carre(4), &[])), "aucun rayon");
    assert!(panique(|| dimension_correlation(&carre(4), &[2, 2])), "deux rayons égaux");
    assert!(panique(|| dimension_correlation(&carre(4), &[0, 2])), "un rayon nul");
    assert!(panique(|| dimension_correlation(&carre(4), &[2, RAYON_MAX + 1])), "un rayon au-delà de RAYON_MAX");
    assert!(panique(|| dimension_correlation(&[], &[1, 2])), "aucun site");
    assert!(panique(|| dimension_correlation(&[(0, 0), (50, 0)], &[1, 2])), "C(r) nul");
    assert!(dimension_correlation(&petit, &[1, 2]).is_finite());
    assert!(dimension_correlation(&carre(3), &[2, 2, 3]).is_finite());
}

// ------------------------------------------------------------------------------------------------------------- la fenêtre

/// Deux sites dont l'étendue est `e` : `(0,0)` et `(e,0)`.
fn deux_sites_a(e: i32) -> Vec<(i32, i32)> {
    vec![(0, 0), (e, 0)]
}

/// juste : `etendue` et `fenetre` (doc de `correlation`, V-Z1-11). `etendue` : vide et un site : 0 ; `(0,0)`, `(3,10)` : 10 ; `(-5,2)`, `(5,3)` : 10 (le
/// plus grand des deux écarts, en abscisse ici) ; `(0,0)`, `(2,9)`, `(-4,-1)` : 10 (en ordonnée) ; un trait de 201 sites : 200 (en nombre de pas) ;
/// `i32::MIN` et `i32::MAX` : `4 294 967 295` (un `u32` qui tient l'écart). `fenetre` : les entiers de 4 à `étendue / 8` si cela fait au moins 6
/// rayons : étendue 71 : vide (`71 / 8 = 8`, rayons 4 à 8, soit 5), 72 à 79 : 4 à 9, 80 : 4 à 10, 242 : 4 à 30, 32 768 : 4 à 4096 (le plus grand
/// rayon permis, 4093 rayons), 32 776 (`/ 8 = 4097 > RAYON_MAX`) : vide ; étendue 0 et 1 site : vide ; `i32::MIN` à `i32::MAX` : vide. La fenêtre
/// d'un trait, d'un L et de deux sites de même étendue est la même (elle ne dépend que de l'étendue). Une fenêtre qui part de 5 (`FENETRE_DEBUT`)
/// ou qui se termine à `étendue / 6` (`FENETRE_DIVISEUR`) se voit à ces bornes.
///
/// Ne couvre pas : l'invariance par les transformations du plan (voir `etape_z1_formes`).
#[test]
#[cfg_attr(not(feature = "etape-z1"), ignore)]
fn etendue_et_fenetre() {
    assert_eq!(etendue(&[]), 0);
    assert_eq!(etendue(&[(7, -7)]), 0);
    assert_eq!(etendue(&[(0, 0), (3, 10)]), 10);
    assert_eq!(etendue(&[(-5, 2), (5, 3)]), 10);
    assert_eq!(etendue(&[(0, 0), (2, 9), (-4, -1)]), 10);
    assert_eq!(etendue(&trait_horizontal(201)), 200);
    assert_eq!(etendue(&[(i32::MIN, 0), (i32::MAX, 0)]), u32::MAX);
    assert_eq!(etendue(&[(5, i32::MIN), (5, i32::MAX), (0, 0)]), u32::MAX);

    assert!(fenetre(&[]).is_empty());
    assert!(fenetre(&[(3, 3)]).is_empty());
    assert!(fenetre(&deux_sites_a(0)).is_empty());
    assert!(fenetre(&deux_sites_a(71)).is_empty());
    for e in 72..=79 {
        assert_eq!(fenetre(&deux_sites_a(e)), (4..=9).collect::<Vec<u32>>(), "étendue {e}");
    }
    assert_eq!(fenetre(&deux_sites_a(80)), (4..=10).collect::<Vec<u32>>());
    assert_eq!(fenetre(&deux_sites_a(242)), (4..=30).collect::<Vec<u32>>());
    assert_eq!(fenetre(&[(0, 0), (0, 242)]), (4..=30).collect::<Vec<u32>>(), "l'étendue en ordonnée");
    assert_eq!(
        fenetre(&[(-100, 5), (0, 5), (50, 40), (100, 90)]),
        (4..=25).collect::<Vec<u32>>(),
        "étendue 200 (l'abscisse)"
    );
    assert_eq!(fenetre(&deux_sites_a(32_768)), (4..=4096).collect::<Vec<u32>>());
    assert!(fenetre(&deux_sites_a(32_776)).is_empty());
    assert!(fenetre(&[(i32::MIN, 0), (i32::MAX, 0)]).is_empty());
}

/// juste : `mesure` (V-Z1-12). Le trait de 201 sites (étendue 200, fenêtre 4 à 25) donne `Some` : les rayons sont 4 à 25, chaque valeur est BIT POUR BIT
/// le `C(r)` de `correlation` au même rayon, la dimension est BIT POUR BIT `dimension_correlation(sites, &rayons)`, et elle vaut `0,9596965250374998`
/// à `1e-12` (Python, comptage naïf des paires et `decimal` à 60 chiffres : le trait donne un peu moins que 1 sur cette fenêtre). `None` pour
/// un amas trop petit (étendue 71), pour deux sites à distance 100 (la fenêtre 4 à 12 n'a aucune paire : `C(r) = 0`, là où `dimension_correlation`
/// paniquerait) et pour un seul site ; et `mesure` ne panique pour aucune des entrées, y compris deux sites aux extrémités de l'`i32`.
///
/// Ne couvre pas : la valeur sur une fractale (voir `etape_z1_formes`).
#[test]
#[cfg_attr(not(feature = "etape-z1"), ignore)]
fn mesure_est_coherente() {
    let trait_201 = trait_horizontal(201);
    let m = mesure(&trait_201).expect("un trait de 201 sites se mesure");
    assert_eq!(m.rayons, (4..=25).collect::<Vec<u32>>());
    assert_eq!(m.valeurs.len(), m.rayons.len());
    let toutes = correlation(&trait_201, 25);
    for (rayon, valeur) in m.rayons.iter().zip(&m.valeurs) {
        assert_eq!(valeur.to_bits(), toutes[(*rayon - 1) as usize].to_bits(), "C({rayon})");
    }
    assert_eq!(m.dimension.to_bits(), dimension_correlation(&trait_201, &m.rayons).to_bits());
    proche(m.dimension, D_TRAIT_201_FENETRE, 1e-12, "dimension du trait de 201 sites");

    assert_eq!(mesure(&deux_sites_a(71)), None, "étendue 71 : amas trop petit");
    assert_eq!(mesure(&deux_sites_a(100)), None, "aucune paire dans la fenêtre");
    assert!(panique(|| dimension_correlation(&deux_sites_a(100), &(4..=12).collect::<Vec<u32>>())));
    assert_eq!(mesure(&[(0, 0)]), None);
    assert_eq!(mesure(&[]), None);
    assert_eq!(mesure(&[(i32::MIN, i32::MIN), (i32::MAX, i32::MAX)]), None);
}

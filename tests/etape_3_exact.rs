//! Étape 3 : des cas à solution connue (formes closes), vérifiés sur **tout l'état à chaque pas**. Avec `α = 0` (aucune
//! diffusion) ou `β = 0` (aucune vapeur : la diffusion est nulle partout), la dynamique se résout à la main : chaque anneau de
//! cellules devient de bord quand l'anneau intérieur gèle, reçoit `γ` à chaque pas, et gèle quand sa masse atteint 1.
//! Valeurs attendues : `references/etape-3-valeurs.md`, identifiants `V-EXACT-xx`.
//!
//! Notation : `m` est le nombre de pas qu'un anneau met à geler, `m = ceil((UN - β) / γ)` calculé en entiers sur les `brut`
//! (`UN = 2^32`) ; l'anneau `k` (distance `k` du centre, `1 <= k <= R - 1`) vaut `β` jusqu'au temps `m (k - 1)`, puis
//! `β + (t - m (k - 1)) γ` ; le centre vaut `UN + t γ` ; la couronne (distance `R`) vaut `β`.

mod etape_3_commun;

use etape_3_commun::{cases, distance, parametres};
use givre_lab::flocon::{Etat, Flocon};
use givre_lab::math::Fixe;

fn cases_hexagone(r: i64) -> usize {
    (3 * r * (r + 1) + 1) as usize
}

/// La masse (brut) attendue de la case `(q, r)` au temps `t`, pour le germe central sur un hexagone de rayon `rayon` : voir la
/// notation en tête de fichier.
fn attendue(rayon: i32, beta: i64, gamma: i64, m: i64, t: i64, q: i32, r: i32) -> i64 {
    let k = i64::from(distance(q, r));
    if k == 0 {
        Fixe::UN.0 + t * gamma
    } else if k == i64::from(rayon) {
        beta
    } else {
        beta + (t - m * (k - 1)).max(0) * gamma
    }
}

/// Vérifie tout l'état, les classes et les mesures de `f` à chaque pas de 0 à `pas_max` contre la forme close.
fn verifier(f: &mut Flocon, rayon: i32, m: i64, pas_max: i64, etiquette: &str) {
    let (beta, gamma) = (f.parametres().beta.0, f.parametres().gamma.0);
    for t in 0..=pas_max {
        if t > 0 {
            f.pas();
        }
        for &(q, r) in &cases(rayon) {
            assert_eq!(f.masse(q, r).0, attendue(rayon, beta, gamma, m, t, q, r), "{etiquette} t={t} case ({q}, {r})");
        }
        // les classes : gelée si la masse est au moins 1, de bord si une voisine est gelée
        let gelees_attendues =
            cases(rayon).iter().filter(|&&(q, r)| attendue(rayon, beta, gamma, m, t, q, r) >= Fixe::UN.0).count();
        assert_eq!(f.gelees(), gelees_attendues, "{etiquette} t={t}");
        // l'anneau k est gelé dès le temps m k (et pas avant) : le rayon du cristal est min(t / m, R - 1) si beta + m * gamma >= 1
        let rayon_attendu = (t / m).min(i64::from(rayon) - 1);
        assert_eq!(i64::from(f.rayon_cristal()), rayon_attendu, "{etiquette} t={t}");
        assert_eq!(f.gelees(), cases_hexagone(rayon_attendu), "{etiquette} t={t} : le cristal est un hexagone plein");
        assert_eq!(f.compacite(), Fixe::UN, "{etiquette} t={t}");
        assert_eq!(f.pointes(), 0, "{etiquette} t={t}");
        if rayon_attendu >= 1 {
            let k = rayon_attendu as i32;
            assert_eq!(f.etat(k, 0), Etat::Gelee);
            // un anneau de plus : de bord (si ce n'est pas la couronne, c'est aussi de bord : elle est voisine de l'anneau gelé)
            assert_eq!(f.etat(k + 1, 0), Etat::DeBord, "{etiquette} t={t}");
        }
        assert_eq!(f.temps(), t as u64);
    }
}

/// juste : `α = 0`, `β = 1/2`, `γ = 1/4`, `R = 8`, tout l'état à chaque pas de 0 à 17 (V-EXACT-01). Un anneau gèle exactement
/// 2 pas après être devenu de bord : `1/2 + 2 * 1/4 = 1` pile (la masse égale à 1 est gelée : `>=`, pas `>`). Les cases non
/// réceptives ne bougent jamais (`α = 0`), `γ` ne va qu'aux réceptives. Un `>` pour `>=` retarde chaque anneau d'un pas ; un `γ`
/// ajouté aux non réceptives ou aux seules gelées, une classe de bord fausse, un voisinage faussé échouent ici.
///
/// Ne couvre pas : la diffusion (voir `plaque_sans_vapeur`).
#[test]
#[cfg_attr(not(feature = "etape-3"), ignore)]
fn alpha_nul_dyadique() {
    let mut f = Flocon::new(8, parametres("0", "0.5", "0.25"));
    verifier(&mut f, 8, 2, 17, "alpha 0, beta 1/2, gamma 1/4");
}

/// juste : `α = 0`, `β = 0.3`, `γ = 0.07` (décimaux non dyadiques lus par `Parametres::lire`), `R = 6` (V-EXACT-02). Les `brut`
/// lus sont `1288490189` et `300647711` (Python : `floor(0.3 * 2^32 + 1/2)` et `floor(0.07 * 2^32 + 1/2)`) ; `m = ceil((2^32 - 1288490189)
/// / 300647711) = 10` par entiers (`9 γ` ne suffit pas, `10 γ` dépasse de 3 `ulp`). Tout l'état à chaque pas de 0 à 5 `m + 2`.
///
/// Ne couvre pas : la diffusion.
#[test]
#[cfg_attr(not(feature = "etape-3"), ignore)]
fn alpha_nul_decimaux() {
    let p = parametres("0", "0.3", "0.07");
    assert_eq!((p.beta.0, p.gamma.0), (1_288_490_189, 300_647_711));
    let m = (Fixe::UN.0 - p.beta.0 + p.gamma.0 - 1) / p.gamma.0;
    assert_eq!(m, 10);
    let mut f = Flocon::new(6, p);
    verifier(&mut f, 6, m, 5 * m + 2, "alpha 0, beta 0.3, gamma 0.07");
}

/// juste : une plaque sans vapeur (V-EXACT-03) : `β = 0`. Les cellules non réceptives valent 0 et restent à 0 (leur moyenne est
/// 0) ; une cellule de bord ne reçoit que `γ` ; **la diffusion n'a aucun effet, quel que soit `α`**. `R = 5`, `α` dans
/// `{1, 0.7, 2, 0.3}`, `γ` dans `{0.25, 0.1, 0.05}` : tout l'état à chaque pas, un anneau tous les `m = ceil(2^32 / γ)` pas (4, 10,
/// 20), le cristal est toujours un hexagone plein (compacité 1, aucune pointe). Une diffusion qui se trompe de cellules (la masse
/// d'une réceptive qui diffuse, une réceptive qui ne reçoit pas) donne une masse non nulle ailleurs et échoue ici.
///
/// Ne couvre pas : `β > 0` avec diffusion.
#[test]
#[cfg_attr(not(feature = "etape-3"), ignore)]
fn plaque_sans_vapeur() {
    for alpha in ["1", "0.7", "2", "0.3"] {
        for gamma in ["0.25", "0.1", "0.05"] {
            let p = parametres(alpha, "0", gamma);
            let m = (Fixe::UN.0 + p.gamma.0 - 1) / p.gamma.0;
            assert!(matches!((gamma, m), ("0.25", 4) | ("0.1", 10) | ("0.05", 20)), "m = {m} pour gamma {gamma}");
            let mut f = Flocon::new(5, p);
            verifier(&mut f, 5, m, 4 * m + 3, &format!("alpha {alpha} beta 0 gamma {gamma}"));
        }
    }
}

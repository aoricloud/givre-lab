//! Étape P1 : les trois cas à solution exacte du champ de phase de Karma et Rappel (Phys. Rev. E 57, 4323, 1998), en `f64` ET en
//! virgule fixe. Valeurs attendues : `references/etape-p1-valeurs.md`, identifiants `V-P1-01` (le front au repos), `V-P1-02` et
//! `V-P1-ONDE` (le front poussé), `V-P1-03` (le front de Neumann), `V-P1-K` (les `k`), `V-P1-ARITH` (flottant contre virgule fixe).
//! Les tolérances sont celles de `etape_p1_commun` (`TOL_CAS1`, `TOL_CAS2`, `TOL_CAS3`) : mesurées, 1,14 à 1,25 fois la valeur
//! observée ; chaque cas porte en plus un critère qui ne dépend pas de la mesure (l'ordre de convergence).

mod etape_p1_commun;

use etape_p1_commun::*;

const MOTEURS: [(&str, Moteur); 2] = [("f64", Moteur::Lib64), ("Fixe", Moteur::LibFixe)];

/// juste : (V-P1-01) le front au repos (`u = 0`) : départ en marche `+1 | -1` au milieu d'un domaine de largeur 40 sans flux,
/// `W = 1,25`, `tau = 1,6`, relaxation pendant `60 tau` ; l'écart maximal (sur les cases) au profil exact `-tanh(x / (W sqrt2))`
/// (référence calculée dans le test avec `tanh` de la bibliothèque standard, permise ici) est sous `TOL_CAS1[dx]` pour
/// `dx` = 1 ; 0,5 ; 0,25 ; 0,125, en `f64` ET en `Fixe` ; et le rapport des écarts `f64` de deux pas successifs est entre 3,4 et 4,6
/// (l'ordre 2 du laplacien à trois points : observé 3,8 ; 3,9 ; 4,0).
///
/// Ne couvre pas : un front en mouvement (voir `front_pousse_vitesse`) ; un autre `W` que 1,25.
#[test]
#[cfg_attr(not(feature = "etape-p1"), ignore)]
fn front_au_repos_profil_exact() {
    let mut precedent: Option<f64> = None;
    for (dx, tol) in TOL_CAS1 {
        for (nom, m) in MOTEURS {
            let (ecart, _) = cas1(m, dx);
            assert!(ecart <= tol, "{nom} dx={dx} : écart {ecart:e} > tolérance {tol:e}");
            if nom == "f64" {
                if let Some(avant) = precedent {
                    let rapport = avant / ecart;
                    assert!((3.4..4.6).contains(&rapport), "dx={dx} : rapport des écarts {rapport} (ordre 2 attendu)");
                }
                precedent = Some(ecart);
            }
        }
    }
}

/// juste : (V-P1-01) le front au repos ne bouge pas, et il est au milieu : la position (doc) est `20 = n dx / 2` à `1e-12` près
/// (l'antisymétrie `φ[n-1-i] = -φ[i]`, voir `etape_p1_symetrie.rs`), en `f64` ET en `Fixe` (à `1e-8` près : l'arrondi de `Fixe` demi-cas vers +∞ n'est pas symétrique ; observé 2,5e-9 à dx = 0,5).
///
/// Ne couvre pas : un front décentré.
#[test]
#[cfg_attr(not(feature = "etape-p1"), ignore)]
fn front_au_repos_reste_au_milieu() {
    for (dx, tol) in [(0.5, 1e-12), (0.25, 1e-12)] {
        let (_, e) = cas1(Moteur::Lib64, dx);
        let x = position(&e.phi, dx).expect("front");
        assert!((x - 20.0).abs() <= tol, "f64 dx={dx} : {x}");
        let (_, e) = cas1(Moteur::LibFixe, dx);
        let x = position(&e.phi, dx).expect("front");
        assert!((x - 20.0).abs() <= 1e-8, "Fixe dx={dx} : {x}");
    }
}

/// juste : (V-P1-02, V-P1-ONDE) le front poussé à température fixée : `u = -Delta` gelé, la vitesse mesurée (position par
/// interpolation de la doc, sur au moins 1,5 de course) est la vitesse EXACTE de l'onde progressive du modèle,
/// `v = c(lambda Delta) W / tau` (`C_EXACT` : calculée à tout ordre par un tir dans le plan de phase, `juge_onde.py` ; au
/// premier ordre `c = lambda Delta / a1`, la cinétique de l'article quand `D` est infini, `a1 = 5 sqrt2 / 8`) : l'écart relatif
/// est sous `TOL_CAS2` pour `dx` = 0,5 ; 0,25 ; 0,125 et `lambda Delta` = 0,01 ; 0,1 (avec `lambda = 2` : la vitesse ne dépend
/// que du produit) ; 0,3 ; 1, en `f64` ET en `Fixe`. Et, critère qui ne dépend pas de la mesure : l'écart est une erreur de
/// grille d'ordre 2 : le rapport des écarts de deux pas successifs est entre 3,4 et 4,6 (observé : de 3,8 à 4,1), pour chaque
/// `lambda Delta`, en `f64`.
///
/// Ne couvre pas : `u` joué ; un `lambda Delta` négatif ; la convergence en `dt` (à `dt` quatre fois plus petit l'écart de
/// `lambda Delta = 1` diminue d'un tiers : mesuré par le juge, non gelé).
#[test]
#[cfg_attr(not(feature = "etape-p1"), ignore)]
fn front_pousse_vitesse() {
    let mut ecarts_f64: Vec<(f64, f64, f64)> = Vec::new();
    for (dx, lambda, ld, tol) in TOL_CAS2 {
        for (nom, m) in MOTEURS {
            let rel = cas2(m, dx, lambda, ld);
            assert!(
                rel.abs() <= tol,
                "{nom} dx={dx} lambda={lambda} lambda*Delta={ld} : écart relatif à la vitesse exacte {rel:+e} (tolérance {tol:e})"
            );
            if nom == "f64" {
                ecarts_f64.push((dx, ld, rel.abs()));
            }
        }
    }
    for &(dx, ld, ecart) in &ecarts_f64 {
        if let Some(&(_, _, plus_fin)) = ecarts_f64.iter().find(|t| t.0 == dx / 2.0 && t.1 == ld) {
            let rapport = ecart / plus_fin;
            assert!(
                (3.4..4.6).contains(&rapport),
                "lambda*Delta={ld} dx={dx} : rapport des écarts {rapport} (ordre 2 attendu)"
            );
        }
    }
}

/// juste : (V-P1-03, V-P1-K) le front de Neumann : le modèle complet (`φ` et `u` joués), `D = 1,5`, `tau = 2 W²`,
/// `lambda = D tau / (a2 W²) = 4,787` (la cinétique `beta = a1 (tau / (lambda W) - a2 W / D)` de l'interface mince est alors
/// nulle), solide contre un mur sans flux, liquide surfondu `u = -Delta` ; départ à `x0 = 10` sur le profil exact de `u` de la
/// solution de Neumann au temps `t0 = (x0 / 2k)² / D` ; après 400 de temps, `X / (2 k sqrt(D (t0 + 400))) - 1` est sous
/// `TOL_CAS3`, pour `Delta` = 0,25 ; 0,5 ; 0,75 (`k` = 0,168 999 042 462 ; 0,432 751 599 366 ; 0,971 850 282 583, racines de
/// `Delta = sqrt(pi) k e^(k²) erfc(k)`), `W` = 1 (`dx` = 0,5 et 0,25) et `W` = 0,5 (`dx` = 0,25, pour `Delta` = 0,5 et 0,75), en
/// `f64` ET en `Fixe`. Et, critère qui ne dépend pas de la mesure : l'écart est celui de l'interface mince (il ne vient ni de
/// `dx` ni de `dt`), il diminue donc avec `W` à `lambda` fixé : à `Delta = 0,75`, où il est le plus grand, l'écart à `W = 0,5`
/// (`dx = 0,25`) est moins de la moitié de celui à `W = 1` (`dx = 0,5`), en `f64` (observé : 6,5 fois moins).
///
/// Ne couvre pas : un `lambda` plus grand (à `tau` fixé, diviser `W` par 2 multiplie `lambda` par 4 et l'écart par 6 : +6 % à
/// `Delta` = 0,75, `W` = 0,5, `tau` = 2 : voir `MODELE.md`) ; un `Delta` plus proche de 1 ; l'accord avec l'article (qui ne donne
/// pas ce cas).
#[test]
#[cfg_attr(not(feature = "etape-p1"), ignore)]
fn front_de_neumann_position() {
    for (idx, w, dx, tol) in TOL_CAS3 {
        let (delta, k) = K_NEUMANN[idx];
        for (nom, m) in MOTEURS {
            let rel = cas3(m, delta, k, w, dx);
            assert!(rel.abs() <= tol, "{nom} Delta={delta} W={w} dx={dx} : écart relatif {rel:+e} (tolérance {tol:e})");
        }
    }
    let (delta, k) = K_NEUMANN[2];
    let (gros, fin) = (cas3(Moteur::Lib64, delta, k, 1.0, 0.5).abs(), cas3(Moteur::Lib64, delta, k, 0.5, 0.25).abs());
    assert!(
        fin < 0.5 * gros,
        "Delta={delta} : l'écart à W=0,5 ({fin:e}) devait être moins de la moitié de celui à W=1 ({gros:e})"
    );
}

/// juste : (V-P1-ARITH) flottant contre virgule fixe sur le MÊME schéma : au repos, `f64` et `Fixe` diffèrent de moins de `2e-8` en `phi`
/// (`dx` = 0,5 et 0,25 : observé 2,6e-9 et 8,3e-9) ; le front poussé (`dx` = 0,25, `lambda Delta` = 0,1) : leurs écarts relatifs
/// à la vitesse exacte diffèrent de moins de `1e-7` (observé 5e-9) ; Neumann (`Delta` = 0,5, `W` = 1, `dx` = 0,25) : de moins de `1,2e-5`
/// (observé 4,8e-6). C'est la mesure qui dit que la virgule fixe Q31.32 suit le `f64` à ces pas, pas qu'elle le suivra à un pas plus fin
/// (l'écart croît comme `1 / dt` : voir la doc de `Front1dFixe`).
///
/// Ne couvre pas : les pas plus fins, les petits incréments (voir `MODELE.md`) ; la vitesse (le coût n'est pas un test).
#[test]
#[cfg_attr(not(feature = "etape-p1"), ignore)]
fn flottant_et_virgule_fixe_s_accordent() {
    for dx in [0.5, 0.25] {
        let (_, a) = cas1(Moteur::Lib64, dx);
        let (_, b) = cas1(Moteur::LibFixe, dx);
        let diff = a.phi.iter().zip(&b.phi).map(|(x, y)| (x - y).abs()).fold(0.0, f64::max);
        assert!(diff <= 2e-8, "dx={dx} : f64 contre Fixe {diff:e}");
        assert!(diff > 0.0, "dx={dx} : f64 et Fixe ne peuvent pas être égaux au bit près (le test serait à vide)");
    }
    let a = cas2(Moteur::Lib64, 0.25, 2.0, 0.1);
    let b = cas2(Moteur::LibFixe, 0.25, 2.0, 0.1);
    assert!((a - b).abs() <= 1e-7, "front poussé : {a:e} contre {b:e}");
    let (delta, k) = K_NEUMANN[1];
    let (a, b) = (cas3(Moteur::Lib64, delta, k, 1.0, 0.25), cas3(Moteur::LibFixe, delta, k, 1.0, 0.25));
    assert!((a - b).abs() <= 1.2e-5, "Neumann : {a:e} contre {b:e}");
}

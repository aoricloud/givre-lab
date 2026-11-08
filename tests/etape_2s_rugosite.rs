//! Étape 2s : la rugosité `w`, sa solution exacte pour le dépôt aléatoire, la conservation des hauteurs, la pente des moindres
//! carrés et les courbes moyennes. Valeurs attendues : `references/etape-2s-valeurs.md`, identifiants `V-RUG-xx`, `V-RD-xx`,
//! `V-PENTE-xx`, `V-MOY-xx`.

mod etape_2s_commun;

use etape_2s_commun::{NOMS, Naif, rugosite_naive};
use givre_lab::math::logarithme;
use givre_lab::surface::{
    Modele, Surface, exposant, pente, rugosite_carre_de, rugosite_de, rugosite_exacte, rugosites_moyennes,
    variance_rugosite_carre_depot,
};

fn modele(nom: &str) -> Modele {
    Modele::depuis_nom(nom).expect("nom de modèle")
}

/// juste : `rugosite_exacte` rend `(L·S2 - S1², L²)` en entiers (V-RUG-01, la doc), sur des profils dont on connaît la variance
/// à la main : `[0, 2]` : moyenne 1, `w² = 1` (4/4) ; `[1, 1, 1]` : 0 ; `[0, 3, 0, 3]` : moyenne 1,5, `w² = 2,25` (36/16) ;
/// `[0, 0, 0, 9]` : moyenne 2,25, Σ(h - moyenne)² = 60,75, `w² = 15,1875` (243/16) ; `[1, 2, 3, 4, 5]` : variance 2 (50/25) ; une
/// colonne : 0/1 ; `[10^12 + 1, 10^12, 10^12]` : `w² = 2/9` EXACTEMENT, que des flottants sur la moyenne ne donnent pas (le
/// numérateur est 2 : `L·S2 - S1² = 9·10^24 + 6·10^12 + 3 - (9·10^24 + 6·10^12 + 1)`) ; `[2^40, 0]` : numérateur `2^80`.
/// Le rapport n'est PAS simplifié (le dénominateur est toujours `L²`).
///
/// Ne couvre pas : les hauteurs au-delà de `2^40` (débordement non fixé par la doc).
#[test]
#[cfg_attr(not(feature = "etape-2s"), ignore)]
fn rugosite_exacte_a_la_main() {
    let cas: [(&[u64], (u128, u128)); 9] = [
        (&[0, 2], (4, 4)),
        (&[1, 1, 1], (0, 9)),
        (&[0, 3, 0, 3], (36, 16)),
        (&[0, 0, 0, 9], (243, 16)),
        (&[1, 2, 3, 4, 5], (50, 25)),
        (&[7], (0, 1)),
        (&[1_000_000_000_001, 1_000_000_000_000, 1_000_000_000_000], (2, 9)),
        (&[1 << 40, 0], (1 << 80, 4)),
        (&[5, 5, 5, 5, 6, 5, 5, 5], (7, 64)), // S1 = 41, S2 = 211 : 8 · 211 - 41² = 7
    ];
    for (profil, attendu) in cas {
        assert_eq!(rugosite_exacte(profil), attendu, "{profil:?}");
        assert_eq!(rugosite_naive(profil), attendu, "{profil:?} (le calcul du test)");
    }
}

/// juste : `rugosite_carre_de` est `numérateur as f64 / dénominateur as f64` et `rugosite_de` sa racine, au bit près (V-RUG-02, la
/// doc) : `[0, 3, 0, 3]` donne exactement 2,25 et 1,5 ; `[0, 0, 0, 9]` donne `243 / 16 = 15,1875` ; `[10^12 + 1, 10^12, 10^12]`
/// donne `2.0 / 9.0` (aux bits : `0x3FCC71C71C71C71C`) et sa racine `(2/9).sqrt()` ; un profil plat donne exactement 0. Les
/// méthodes `rugosite_carre()` et `rugosite()` d'une `Surface` rendent les mêmes valeurs que ces fonctions sur ses hauteurs,
/// et, sur `depuis_hauteurs`, la même chose que la main.
///
/// Ne couvre pas : la précision relative de la racine pour des valeurs énormes (c'est `sqrt`, correct à l'arrondi).
#[test]
#[cfg_attr(not(feature = "etape-2s"), ignore)]
fn rugosite_flottante_est_la_formule_de_la_doc() {
    assert_eq!(rugosite_carre_de(&[0, 3, 0, 3]), 2.25);
    assert_eq!(rugosite_de(&[0, 3, 0, 3]), 1.5);
    assert_eq!(rugosite_carre_de(&[0, 0, 0, 9]), 15.1875);
    assert_eq!(rugosite_carre_de(&[3, 3, 3]), 0.0);
    assert_eq!(rugosite_de(&[3, 3, 3]), 0.0);
    let ecart = [1_000_000_000_001_u64, 1_000_000_000_000, 1_000_000_000_000];
    assert_eq!(rugosite_carre_de(&ecart).to_bits(), (2.0_f64 / 9.0).to_bits());
    assert_eq!(rugosite_carre_de(&ecart).to_bits(), 0x3FCC_71C7_1C71_C71C);
    assert_eq!(rugosite_de(&ecart).to_bits(), (2.0_f64 / 9.0).sqrt().to_bits());
    for (profil, w2, w) in
        [(&[0_u64, 2][..], 1.0, 1.0), (&[0, 3, 0, 3], 2.25, 1.5), (&[0, 0, 0, 9], 15.1875, 15.1875_f64.sqrt())]
    {
        for nom in ["depot", "relaxation", "balistique"] {
            let s = Surface::depuis_hauteurs(modele(nom), profil, 1);
            assert_eq!(s.rugosite_carre(), w2, "{nom} {profil:?}");
            assert_eq!(s.rugosite(), w, "{nom} {profil:?}");
        }
    }
    for nom in NOMS {
        let mut s = Surface::new(modele(nom), 9, 4);
        for _ in 0..200 {
            s.deposer();
        }
        let (num, den) = rugosite_naive(&s.hauteurs());
        assert_eq!(s.rugosite_carre(), num as f64 / den as f64, "{nom}");
        assert_eq!(s.rugosite(), (num as f64 / den as f64).sqrt(), "{nom}");
        assert_eq!(rugosite_exacte(&s.hauteurs()), (num, den), "{nom}");
    }
}

/// juste : la solution exacte du dépôt aléatoire, VÉRIFIÉE PAR ÉNUMÉRATION (V-RD-01, V-RD-02) : pour `(n, L)` dans (4,2), (4,3), (5,3),
/// (6,3), (6,4), (7,4), (8,3), (7,5), (9,3), on énumère les `L^n` suites de colonnes (code du test, entiers `u128`), on calcule le
/// numérateur `L·S2 - S1²` de chaque suite, et l'on vérifie (a) `Σ numérateurs = L^n · n · (L - 1)` EXACTEMENT, c'est-à-dire
/// `E[w²] = n (L - 1) / L² = t (1 - 1/L)` pour `n = t·L` ; (b) la variance exacte, `Var(w²) = (L^n Σ num² - (Σ num)²) / (L^(2n) L⁴)`,
/// est celle de `variance_rugosite_carre_depot(n, L)` (`2 n (n-1) (L-1) / L⁴`) à `1e-12` en relatif. (Le calcul en rationnels de
/// Python `fractions` donne les mêmes variances : 3/2 pour (4,2), 16/27 pour (4,3), 45/64 pour (6,4).)
///
/// Ne couvre pas : `Surface` (le test suivant la compare à cette solution).
#[test]
#[cfg_attr(not(feature = "etape-2s"), ignore)]
fn depot_solution_exacte_par_enumeration() {
    for (n, l) in [(4_u32, 2_usize), (4, 3), (5, 3), (6, 3), (6, 4), (7, 4), (8, 3), (7, 5), (9, 3)] {
        let total = (l as u64).pow(n);
        let (mut somme, mut somme2) = (0_u128, 0_u128);
        for code in 0..total {
            let mut h = vec![0_u64; l];
            let mut reste = code;
            for _ in 0..n {
                h[(reste % l as u64) as usize] += 1;
                reste /= l as u64;
            }
            let (num, _) = rugosite_naive(&h);
            somme += num;
            somme2 += num * num;
        }
        let total = u128::from(total);
        assert_eq!(somme, total * u128::from(n) * (l as u128 - 1), "n={n} L={l} : E[w²]");
        let denominateur = (total * total * (l as u128).pow(4)) as f64;
        let variance = (total * somme2 - somme * somme) as f64 / denominateur;
        let formule = variance_rugosite_carre_depot(u64::from(n), l as u64);
        assert!(
            (variance - formule).abs() <= 1e-12 * variance.abs(),
            "n={n} L={l} : énumération {variance}, formule {formule}"
        );
    }
    // trois valeurs de la variance recoupées par Python (`variance_depot.py`, fractions) : 3/2, 16/27, 45/64
    assert!((variance_rugosite_carre_depot(4, 2) - 1.5).abs() < 1e-12);
    assert!((variance_rugosite_carre_depot(4, 3) - 16.0 / 27.0).abs() < 1e-12);
    assert!((variance_rugosite_carre_depot(6, 4) - 45.0 / 64.0).abs() < 1e-12);
    // L = 1 : une seule colonne, w² est toujours 0, donc de variance nulle
    assert_eq!(variance_rugosite_carre_depot(50, 1), 0.0);
}

/// juste : `Surface` dépôt aléatoire suit la solution exacte (V-RD-03) : `E[w²] = t (1 - 1/L)`, vérifiée en moyenne sur `G`
/// graines (1 à `G`) avec une tolérance CALCULÉE, non mesurée : six écarts-types de la moyenne, `6 · sqrt(Var(w²) / G)`, avec
/// `Var(w²) = 2 n (n-1) (L-1) / L⁴` (`variance_rugosite_carre_depot`, prouvée par énumération au test précédent). `(L, t, G)` :
/// (2, 60, 2000), (3, 60, 2000), (4, 60, 2000), (8, 60, 3000), (32, 40, 6000). Le test impose en plus que cette tolérance soit
/// plus étroite que `t / L`, l'écart entre `t (1 - 1/L)` et `t` : une implémentation qui oublierait le facteur `1 - 1/L` (ou
/// qui n'aurait que `L - 1` colonnes) sortirait de la bande. Faux d'une probabilité d'environ 2e-9 par couple pour des graines
/// indépendantes (six écarts-types) ; les graines sont fixes, donc le résultat est déterministe.
///
/// Ne couvre pas : la loi de `w²` autrement que par sa moyenne ; l'indépendance des graines voisines de PCG32 (même flux),
/// supposée : les z-scores observés sont dans `references/etape-2s-valeurs.md`.
#[test]
#[cfg_attr(not(feature = "etape-2s"), ignore)]
fn depot_esperance_de_la_rugosite() {
    for (l, t, g) in [(2_usize, 60_u64, 2000_u64), (3, 60, 2000), (4, 60, 2000), (8, 60, 3000), (32, 40, 6000)] {
        let n = t * l as u64;
        let attendu = t as f64 * (1.0 - 1.0 / l as f64);
        let sigma_moyenne = (variance_rugosite_carre_depot(n, l as u64) / g as f64).sqrt();
        let mut somme = 0.0;
        for graine in 1..=g {
            let mut s = Surface::new(Modele::Depot, l, graine);
            for _ in 0..t {
                s.deposer_couche();
            }
            assert_eq!(s.particules(), n);
            somme += s.rugosite_carre();
        }
        let moyenne = somme / g as f64;
        let z = (moyenne - attendu) / sigma_moyenne;
        assert!(
            z.abs() < 6.0,
            "L={l} t={t} G={g} : moyenne {moyenne}, attendu {attendu}, écart-type {sigma_moyenne}, z = {z}"
        );
        assert!(
            6.0 * sigma_moyenne < t as f64 / l as f64,
            "L={l} : la tolérance 6σ n'est pas étroite ({})",
            6.0 * sigma_moyenne
        );
    }
}

/// juste : la conservation des hauteurs (V-CONS-01) : pour le dépôt aléatoire et la relaxation, chaque pas fait monter UNE colonne
/// de EXACTEMENT 1 (donc `Σ h = particules` à chaque pas) ; pour le balistique, chaque pas fait monter UNE colonne d'au moins 1
/// (`Σ h` croît d'au moins 1 par pas et aucune hauteur ne baisse) ; pour Eden, aucune hauteur ne baisse, le nombre de sites
/// occupés vaut le nombre de particules et `Σ h` (le profil du dessus) est au moins ce nombre. L = 11, 6 graines, 500
/// particules, pas à pas.
///
/// Ne couvre pas : que la colonne qui monte soit la bonne (les tests exacts contre le naïf).
#[test]
#[cfg_attr(not(feature = "etape-2s"), ignore)]
fn conservation_des_hauteurs() {
    for nom in NOMS {
        for graine in 1..=6_u64 {
            let mut s = Surface::new(modele(nom), 11, graine);
            let mut avant = s.hauteurs();
            for p in 1..=500_u64 {
                s.deposer();
                let apres = s.hauteurs();
                assert!(avant.iter().zip(&apres).all(|(a, b)| b >= a), "{nom} : une hauteur a baissé au pas {p}");
                let montees: Vec<u64> = avant.iter().zip(&apres).map(|(a, b)| b - a).filter(|&d| d > 0).collect();
                match nom {
                    "depot" | "relaxation" => assert_eq!(montees, vec![1], "{nom} graine {graine} pas {p}"),
                    "balistique" => assert!(montees.len() == 1 && montees[0] >= 1, "{nom} pas {p} : {montees:?}"),
                    _ => assert!(montees.len() <= 1, "{nom} pas {p} : {montees:?}"),
                }
                avant = apres;
                let somme: u64 = avant.iter().sum();
                match nom {
                    "depot" | "relaxation" => assert_eq!(somme, p, "{nom} graine {graine}"),
                    _ => assert!(somme >= p, "{nom} graine {graine} pas {p} : somme {somme}"),
                }
            }
            if nom == "eden" {
                let h_max = *avant.iter().max().unwrap() as usize;
                let occupes: usize = (0..11).map(|x| (0..h_max + 1).filter(|&y| s.occupe(x, y)).count()).sum();
                assert_eq!(occupes as u64, 500);
            }
        }
    }
}

/// juste : la pente des moindres carrés `Σ(x - x̄)(y - ȳ) / Σ(x - x̄)²` sur des points écrits à la main (V-PENTE-01) :
/// `(0;1) (1;3) (2;5) (3;7)` donnent exactement 2 ; `(1; 2,2) (2; 2,8) (3; 4,4) (4; 4,0) (5; 5,6)` donnent `8/10 = 0,8` ;
/// `(0;0) (1;1) (2;1) (3;3) (4;3)` donnent `8/10 = 0,8` ; une pente négative `(0;10) (2;6) (4;2)` donne -2 ; des `x` NON
/// régulièrement espacés `(0;0) (1;1) (10;10)` donnent 1 et `(0;0) (1;0) (10;9)` donnent `171/182` (`x̄ = 11/3`, `ȳ = 3`,
/// `Sxy = 11 + 8 + 38 = 57`, `Sxx = 546/9`) : une pente prise sur les indices et non sur les `x` ne les donne pas ; ajouter une
/// constante aux `y` ne change pas la pente, les multiplier par 3 la multiplie par 3 ; la formule développée `(n Σxy - Σx Σy) /
/// (n Σx² - (Σx)²)` donne la même valeur. La tolérance est `1e-12` en relatif (les sommes en `f64` de nombres de cet ordre).
/// Tous les `x` égaux donnent `NaN` (`0 / 0`, la doc).
///
/// Ne couvre pas : les très grands jeux de points (la somme en `f64` n'est pas compensée : la doc le dit).
#[test]
#[cfg_attr(not(feature = "etape-2s"), ignore)]
fn pente_a_la_main() {
    assert_eq!(pente(&[0.0, 1.0, 2.0, 3.0], &[1.0, 3.0, 5.0, 7.0]), 2.0);
    let proche = |a: f64, b: f64| (a - b).abs() <= 1e-12 * b.abs().max(1.0);
    assert!(proche(pente(&[1.0, 2.0, 3.0, 4.0, 5.0], &[2.2, 2.8, 4.4, 4.0, 5.6]), 0.8));
    assert!(proche(pente(&[0.0, 1.0, 2.0, 3.0, 4.0], &[0.0, 1.0, 1.0, 3.0, 3.0]), 0.8));
    assert!(proche(pente(&[0.0, 2.0, 4.0], &[10.0, 6.0, 2.0]), -2.0));
    assert!(proche(pente(&[0.0, 1.0, 10.0], &[0.0, 1.0, 10.0]), 1.0));
    // x = 0, 1, 10 ; y = 0, 0, 9 : x̄ = 11/3, ȳ = 3 ; Sxy = (-11/3)(-3) + (-8/3)(-3) + (19/3)(6) = 11 + 8 + 38 = 57 ;
    // Sxx = (121 + 64 + 361) / 9 = 546/9 ; pente = 57 · 9 / 546 = 513/546 = 171/182
    assert!(proche(pente(&[0.0, 1.0, 10.0], &[0.0, 0.0, 9.0]), 171.0 / 182.0));
    let xs = [0.5, 1.0, 2.5, 4.0, 4.5];
    let ys = [3.0, 2.0, 2.5, 0.5, 1.5];
    let base = pente(&xs, &ys);
    let decalees: Vec<f64> = ys.iter().map(|y| y + 100.0).collect();
    let triplees: Vec<f64> = ys.iter().map(|y| 3.0 * y).collect();
    assert!(proche(pente(&xs, &decalees), base));
    assert!(proche(pente(&xs, &triplees), 3.0 * base));
    // le même calcul par la formule développée (n Σxy - Σx Σy) / (n Σx² - (Σx)²)
    let n = 5.0;
    let (sx, sy): (f64, f64) = (xs.iter().sum(), ys.iter().sum());
    let sxy: f64 = xs.iter().zip(&ys).map(|(x, y)| x * y).sum();
    let sxx: f64 = xs.iter().map(|x| x * x).sum();
    assert!(proche(base, (n * sxy - sx * sy) / (n * sxx - sx * sx)));
    assert!(pente(&[2.0, 2.0, 2.0], &[1.0, 5.0, 9.0]).is_nan());
}

/// juste : les paniques de `pente`, `rugosite_exacte`, `rugosites_moyennes` et `exposant` (V-PENTE-02, la doc) : listes de longueurs
/// différentes ; un seul point ; aucune colonne ; aucune graine ; aucun instant ; instants non strictement croissants (répétés,
/// décroissants) ; premier instant 0 ; un seul instant pour `exposant`.
///
/// Ne couvre pas : les messages.
#[test]
#[cfg_attr(not(feature = "etape-2s"), ignore)]
fn paniques_des_mesures() {
    fn panique<F: FnOnce()>(f: F) -> bool {
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(f)).is_err()
    }
    let m = Modele::Depot;
    assert!(panique(|| _ = pente(&[1.0, 2.0], &[1.0])));
    assert!(panique(|| _ = pente(&[1.0], &[1.0])));
    assert!(panique(|| _ = pente(&[], &[])));
    assert!(!panique(|| _ = pente(&[1.0, 2.0], &[1.0, 3.0])));
    assert!(panique(|| _ = rugosite_exacte(&[])));
    assert!(panique(|| _ = rugosites_moyennes(m, 4, &[], &[1, 2])));
    assert!(panique(|| _ = rugosites_moyennes(m, 4, &[1], &[])));
    assert!(panique(|| _ = rugosites_moyennes(m, 4, &[1], &[2, 2])));
    assert!(panique(|| _ = rugosites_moyennes(m, 4, &[1], &[3, 2])));
    assert!(panique(|| _ = rugosites_moyennes(m, 4, &[1], &[0, 2])));
    assert!(!panique(|| _ = rugosites_moyennes(m, 4, &[1], &[1, 2])));
    assert!(!panique(|| _ = rugosites_moyennes(m, 4, &[1], &[5])));
    assert!(panique(|| _ = exposant(m, 4, &[1], &[3])));
    assert!(!panique(|| _ = exposant(m, 4, &[1], &[3, 4])));
}

/// juste : `rugosites_moyennes` donne, pour chacun des quatre modèles, EXACTEMENT les `w²` moyens qu'un simulateur naïf calcule
/// (V-MOY-01, la doc) : trois graines (1, 2, 3), L = 6, instants 1, 2, 5, 9 ; pour chaque graine, `t·L` pas du naïf, `w² =
/// numérateur as f64 / dénominateur as f64`, somme des trois dans l'ordre des graines à partir de 0,0, divisée par `3.0` ; les
/// deux listes sont égales bit pour bit. Le même calcul avec une seule graine ou des instants non consécutifs (3, 4) donne les
/// mêmes valeurs aux mêmes instants (l'ordre des opérations de la doc : pas de relevé décalé d'un instant).
///
/// Ne couvre pas : `exposant` (le test suivant).
#[test]
#[cfg_attr(not(feature = "etape-2s"), ignore)]
fn moyennes_contre_naif() {
    fn naif_moyennes(nom: &'static str, l: usize, graines: &[u64], instants: &[u64]) -> Vec<f64> {
        let mut sommes = vec![0.0_f64; instants.len()];
        for &graine in graines {
            let mut naif = Naif::new(nom, l, graine);
            let mut fait = 0_u64;
            for (k, &t) in instants.iter().enumerate() {
                while fait < t * l as u64 {
                    naif.pas();
                    fait += 1;
                }
                let (num, den) = rugosite_naive(&naif.hauteurs());
                sommes[k] += num as f64 / den as f64;
            }
        }
        sommes.iter().map(|s| s / graines.len() as f64).collect()
    }
    for nom in NOMS {
        for (graines, instants) in
            [(&[1_u64, 2, 3][..], &[1_u64, 2, 5, 9][..]), (&[7][..], &[3, 4][..]), (&[4, 5][..], &[1][..])]
        {
            let lu = rugosites_moyennes(modele(nom), 6, graines, instants);
            assert_eq!(lu, naif_moyennes(nom, 6, graines, instants), "{nom} {graines:?} {instants:?}");
        }
    }
}

/// juste : `exposant` est la `pente` de `0,5 · ln(w²moyen)` contre `ln(t)` (V-MOY-02, la doc) : pour les quatre modèles, L = 8,
/// graines 1 à 4, instants 2, 4, 8, 16, 32, le test refait le calcul avec les `w²` moyens du naïf, `logarithme` de
/// `givre_lab::math` et une pente par la formule développée `(n Σxy - Σx Σy) / (n Σxx - (Σx)²)` ; l'accord est à `1e-9`
/// près en relatif (autre ordre de sommation). Le dépôt aléatoire L = 8 donne un exposant proche de 1/2 (à 0,1 : garde-fou
/// grossier, la mesure est dans `etape_2s_exposants`).
///
/// Ne couvre pas : la valeur de l'exposant des modèles (`etape_2s_exposants`).
#[test]
#[cfg_attr(not(feature = "etape-2s"), ignore)]
fn exposant_est_la_pente_de_ln_w_contre_ln_t() {
    let instants = [2_u64, 4, 8, 16, 32];
    let graines = [1_u64, 2, 3, 4];
    for nom in NOMS {
        let mut sommes = vec![0.0_f64; instants.len()];
        for &graine in &graines {
            let mut naif = Naif::new(nom, 8, graine);
            let mut fait = 0_u64;
            for (k, &t) in instants.iter().enumerate() {
                while fait < t * 8 {
                    naif.pas();
                    fait += 1;
                }
                let (num, den) = rugosite_naive(&naif.hauteurs());
                sommes[k] += num as f64 / den as f64;
            }
        }
        let xs: Vec<f64> = instants.iter().map(|&t| logarithme(t as f64)).collect();
        let ys: Vec<f64> = sommes.iter().map(|s| 0.5 * logarithme(s / 4.0)).collect();
        let n = xs.len() as f64;
        let (sx, sy): (f64, f64) = (xs.iter().sum(), ys.iter().sum());
        let sxy: f64 = xs.iter().zip(&ys).map(|(x, y)| x * y).sum();
        let sxx: f64 = xs.iter().map(|x| x * x).sum();
        let attendu = (n * sxy - sx * sy) / (n * sxx - sx * sx);
        let lu = exposant(modele(nom), 8, &graines, &instants);
        assert!((lu - attendu).abs() <= 1e-9 * attendu.abs().max(1e-3), "{nom} : exposant {lu}, attendu {attendu}");
        if nom == "depot" {
            assert!((lu - 0.5).abs() < 0.1, "dépôt : exposant {lu}");
        }
    }
    // une seule colonne : la rugosité est nulle, l'exposant n'existe pas (NaN, la doc)
    assert!(exposant(Modele::Depot, 1, &[1, 2], &[2, 4]).is_nan());
}

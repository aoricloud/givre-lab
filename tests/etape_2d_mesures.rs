//! Étape 2d : les mesures propres aux variantes (`dla_variantes::densite` et `anisotropie`), la conversion de `p` en Q0.32
//! (`collage_depuis_flottant`) et la lecture des directives `collage` et `bruit` (`variante_du_scenario`). Valeurs attendues :
//! `references/etape-2d-valeurs.md`, identifiants `V-ANI-xx`, `V-DEN-xx`, `V-CONV-xx`, `V-DIR2-xx`.
//!
//! Tout est EXACT : des ensembles de sites écrits à la main (une croix sur les axes, une croix sur les diagonales, un carré, un disque, un
//! segment, deux sites) dont les valeurs sont des rationnels calculés par Python (`Fraction`, `valeurs_2d.py`), et des tables de conversion
//! calculées en décimal exact (`Decimal`). Les attendus sont écrits sous forme `a / b` : numérateur et dénominateur sont des entiers exacts en
//! `f64`, donc la division IEEE rend l'arrondi correct du rationnel, le même nombre que `float(Fraction(a, b))` de Python.

use givre_lab::dla::Amas;
use givre_lab::dla_variantes::{
    BRUIT_MAX, TOUJOURS, Variante, anisotropie, collage_depuis_flottant, densite, variante_du_scenario,
};
use givre_lab::mesures::rayon_de_giration;
use givre_lab::scenario::{ErreurLecture, lire_scenario};

type Sites = Vec<(i32, i32)>;

/// Le germe et, sur chaque demi-axe, les sites `1..=l` : quatre bras sur les axes.
fn croix_axes(l: i32) -> Sites {
    let mut sites = vec![(0, 0)];
    for k in 1..=l {
        sites.extend([(k, 0), (-k, 0), (0, k), (0, -k)]);
    }
    sites
}

/// Le germe et, sur chaque demi-diagonale, les sites `(k, k)` à `(l, l)` : quatre bras sur les diagonales.
fn croix_diagonales(l: i32) -> Sites {
    let mut sites = vec![(0, 0)];
    for k in 1..=l {
        sites.extend([(k, k), (k, -k), (-k, k), (-k, -k)]);
    }
    sites
}

/// Le carré plein `[-a, a]²`.
fn carre(a: i32) -> Sites {
    (-a..=a).flat_map(|y| (-a..=a).map(move |x| (x, y))).collect()
}

/// Le disque plein `x² + y² <= r²`.
fn disque(r: i32) -> Sites {
    (-r..=r).flat_map(|y| (-r..=r).map(move |x| (x, y))).filter(|&(x, y)| x * x + y * y <= r * r).collect()
}

fn proche(a: f64, b: f64, tolerance_relative: f64) -> bool {
    (a - b).abs() <= tolerance_relative * b.abs()
}

// ---------------------------------------------------------------------------------------------------------- l'anisotropie

/// juste : `anisotropie` (`S4 = Σ (x⁴ − 6x²y² + y⁴) / Σ (x² + y²)²` autour de l'origine) sur des ensembles écrits à la main, avec des valeurs
/// calculées par Python en rationnels exacts (V-ANI-01) : une croix à quatre bras sur les axes (le germe et `(±k, 0)`, `(0, ±k)`, `k = 1..3`) donne
/// EXACTEMENT `+1` ; une croix sur les diagonales `(±k, ±k)` donne EXACTEMENT `−1` ; le germe seul et la liste vide donnent `0.0` (le
/// dénominateur est nul) ; un site seul `(1, 2)` donne `−7/25`, `(3, 4)` donne `−527/625` ; le carré plein `[-1, 1]²` donne `−3/5` et `[-2, 2]²`
/// `−13/27` ; une croix axes (bras de 2) plus les quatre sites `(±1, ±1)`, `13/21` ; un disque plein de rayon 4 (49 sites) `9/101`, de rayon 10
/// (317 sites) `−18953/269911`, de rayon 30 `−2888367/189585641`, de rayon 100 (31 417 sites) `−47403637/21818982569`, soit environ `−0,002` : « 0
/// pour un disque » est une limite, un disque de rayon fini n'est pas exactement isotrope. Le test voit : un mauvais signe de `6 x² y²`, une
/// moyenne des `cos 4θ` à la place du rapport des sommes (`(1, 0)` et `(4, 4)` donnent `−1023/1025`, pas `0`), un centre de masse à la place de
/// l'origine (`(10, 0)` et `(10, 1)` donnent `19401/20201`, pas `1`), un dénominateur oublié (`r⁴` ou non), le rôle de `x` et `y` échangés sans effet.
///
/// Ne couvre pas : les grandes coordonnées (voir `anisotropie_invariances_et_grands_nombres`).
#[test]
#[cfg_attr(not(feature = "etape-2"), ignore)]
fn anisotropie_a_la_main() {
    assert_eq!(anisotropie(&croix_axes(3)), 1.0);
    assert_eq!(anisotropie(&croix_diagonales(3)), -1.0);
    assert_eq!(anisotropie(&[(0, 0)]), 0.0);
    assert_eq!(anisotropie(&[]), 0.0);
    assert_eq!(anisotropie(&[(1, 2)]), -7.0 / 25.0);
    assert_eq!(anisotropie(&[(3, 4)]), -527.0 / 625.0);
    assert_eq!(anisotropie(&[(2, 1)]), -7.0 / 25.0);
    assert_eq!(anisotropie(&carre(1)), -3.0 / 5.0);
    assert_eq!(anisotropie(&carre(2)), -13.0 / 27.0);
    assert_eq!(anisotropie(&[(1, 0), (4, 4)]), -1023.0 / 1025.0);
    assert_eq!(anisotropie(&[(10, 0), (10, 1)]), 19401.0 / 20201.0);
    let mut mixte = croix_axes(2);
    mixte.extend([(1, 1), (1, -1), (-1, 1), (-1, -1)]);
    assert_eq!(anisotropie(&mixte), 13.0 / 21.0);
    assert!(proche(anisotropie(&disque(4)), 9.0 / 101.0, 1e-14));
    assert!(proche(anisotropie(&disque(10)), -18953.0 / 269911.0, 1e-14));
    assert!(proche(anisotropie(&disque(30)), -2_888_367.0 / 189_585_641.0, 1e-14));
    assert!(proche(anisotropie(&disque(100)), -47_403_637.0 / 21_818_982_569.0, 1e-14));
    assert_eq!(disque(100).len(), 31_417);
    // un segment sur un axe : +1, et sur une diagonale : −1, quelle que soit la longueur
    let segment_x: Sites = (-9..=9).map(|x| (x, 0)).collect();
    let segment_y: Sites = (1..=9).map(|y| (0, -y)).collect();
    let segment_diagonal: Sites = (1..=9).map(|k| (-k, k)).collect();
    assert_eq!((anisotropie(&segment_x), anisotropie(&segment_y), anisotropie(&segment_diagonal)), (1.0, 1.0, -1.0));
}

/// juste : propriétés de `S4` (V-ANI-02), sur les 500 premiers sites de `Amas::new(3, 100)` (l'étape 1) : ni l'échange `x ↔ y`, ni `x → −x`, ni
/// `y → −y`, ni la rotation d'un quart de tour `(x, y) → (−y, x)`, ni l'ordre des sites, ni leur répétition (chacun deux fois), ni
/// l'homothétie `(x, y) → (2x, 2y)` ne changent `S4`, au bit près (les sommes sont des entiers exacts) ; une translation la change (elle est
/// autour de l'origine, pas du centre de masse). Puis les grandes coordonnées, qui débordent un `i64` : `(100000, 0)` et le germe donnent
/// `+1`, `(0, 100000)` et `(100000, 0)` aussi, `(100000, 100000)` donne `−1` (`x⁴ = 10^20 > 2^63`) ; `(16000000, 0), (0, 16000000)` et
/// `(1, 1)` donnent `+1` à l'arrondi près (`1 − 6·10^-29`), calculé en `i128`.
///
/// Ne couvre pas : les coordonnées au-delà de `2^24` (la doc ne le garantit pas).
#[test]
#[cfg_attr(not(feature = "etape-2"), ignore)]
fn anisotropie_invariances_et_grands_nombres() {
    let mut amas = Amas::new(3, 100);
    amas.croitre_jusqu_a(500);
    let sites: Sites = amas.sites().to_vec();
    let s4 = anisotropie(&sites);
    assert!(s4 != 0.0 && s4.abs() < 1.0, "un amas de DLA : S4 = {s4}");
    let transforme = |f: &dyn Fn(i32, i32) -> (i32, i32)| -> f64 {
        let t: Sites = sites.iter().map(|&(x, y)| f(x, y)).collect();
        anisotropie(&t)
    };
    assert_eq!(transforme(&|x, y| (y, x)), s4, "x <-> y");
    assert_eq!(transforme(&|x, y| (-x, y)), s4, "x -> -x");
    assert_eq!(transforme(&|x, y| (x, -y)), s4, "y -> -y");
    assert_eq!(transforme(&|x, y| (-y, x)), s4, "rotation d'un quart de tour");
    assert_eq!(transforme(&|x, y| (2 * x, 2 * y)), s4, "homothétie de rapport 2");
    assert_ne!(transforme(&|x, y| (x + 3, y)), s4, "une translation change S4 (autour de l'origine)");
    let mut renverse = sites.clone();
    renverse.reverse();
    assert_eq!(anisotropie(&renverse), s4, "ordre des sites");
    let mut doubles = sites.clone();
    doubles.extend(sites.iter().copied());
    assert_eq!(anisotropie(&doubles), s4, "sites répétés");

    assert_eq!(anisotropie(&[(100_000, 0), (0, 0)]), 1.0);
    assert_eq!(anisotropie(&[(0, 100_000), (100_000, 0)]), 1.0);
    assert_eq!(anisotropie(&[(100_000, 100_000)]), -1.0);
    assert_eq!(anisotropie(&[(16_000_000, 0), (0, 16_000_000), (1, 1)]), 1.0);
}

// ------------------------------------------------------------------------------------------------------------- la densité

/// juste : `densite` (`N / Rg²` = `n³ / num`) sur des ensembles écrits à la main, valeurs calculées par Python en rationnels exacts
/// (V-DEN-01) : `(0, 0)` et `(3, 4)` : `8/25` (`Rg² = 25/4`, `N = 2`) ; le carré plein `[-2, 2]²` : `25/4` (`Rg² = (s² − 1)/6 = 4`, `N = 25`) ;
/// `[-6, 6]²` : `169/28` ; la croix de quatre bras de 3 : `169/56` ; le segment `(0..9, 0)` : `40/33` ; le disque de rayon 10 (317 sites) :
/// `100489/16012` ; celui de rayon 100 (31 417 sites) : `987027889/157090680`, soit `6,28317…`, à moins de `10^-3` de `2π` (`N = πR²`,
/// `Rg² = R²/2`). La densité ne change pas par translation (le rayon de giration est celui du centre de masse : sommes entières exactes, bit à
/// bit), et `densite × rayon_de_giration² = N` à `10^-12` relatif.
///
/// Ne couvre pas : les amas de DLA (voir `etape_2d_statistique`).
#[test]
#[cfg_attr(not(feature = "etape-2"), ignore)]
fn densite_a_la_main() {
    assert_eq!(densite(&[(0, 0), (3, 4)]), 8.0 / 25.0);
    assert_eq!(densite(&carre(2)), 25.0 / 4.0);
    assert_eq!(densite(&carre(6)), 169.0 / 28.0);
    assert_eq!(densite(&croix_axes(3)), 169.0 / 56.0);
    let segment: Sites = (0..10).map(|x| (x, 0)).collect();
    assert_eq!(densite(&segment), 40.0 / 33.0);
    assert_eq!(densite(&disque(10)), 100_489.0 / 16_012.0);
    let grand = disque(100);
    assert_eq!(densite(&grand), 987_027_889.0 / 157_090_680.0);
    assert!((densite(&grand) - 2.0 * std::f64::consts::PI).abs() < 1e-3, "disque : {}", densite(&grand));
    let decale: Sites = disque(10).iter().map(|&(x, y)| (x + 37, y - 12)).collect();
    assert_eq!(densite(&decale), densite(&disque(10)), "translation : mêmes sommes entières exactes");
    for sites in [carre(3), disque(7), croix_axes(5), segment] {
        let rg = rayon_de_giration(&sites);
        assert!(proche(densite(&sites) * rg * rg, sites.len() as f64, 1e-12), "N = densité × Rg²");
    }
    // la densité d'un amas plus compact est plus grande : le carré plein de côté 5 a plus de sites par Rg² que la croix de même rayon
    assert!(densite(&carre(2)) > densite(&croix_axes(2)));
}

/// juste : `densite` PANIQUE quand le rayon de giration est nul ou n'existe pas (V-DEN-02) : la liste vide, un site seul, deux sites confondus
/// (le message est libre) ; et ne panique pas pour deux sites distincts (`(0, 0)` et `(3, 4)` : `8/25`).
///
/// Ne couvre pas : le message de la panique.
#[test]
#[cfg_attr(not(feature = "etape-2"), ignore)]
fn densite_panique_sans_rayon_de_giration() {
    assert_eq!(densite(&[(0, 0), (3, 4)]), 8.0 / 25.0, "deux sites distincts : la densité existe");
    assert!(std::panic::catch_unwind(|| densite(&[])).is_err());
    assert!(std::panic::catch_unwind(|| densite(&[(0, 0)])).is_err());
    assert!(std::panic::catch_unwind(|| densite(&[(4, 4), (4, 4)])).is_err());
}

// ------------------------------------------------------------------------------------------ la conversion de p en Q0.32

/// juste : `collage_depuis_flottant` rend `round(p · 2^32)`, arrondi au plus proche, les demi vers le haut, pour `0 < p <= 1` et `q >= 1` (V-CONV-01),
/// avec les valeurs de Python en décimal exact : `1` donne `2^32` ; `0,5`, `2^31` ; `0,25`, `2^30` ; `0,75`, `3221225472` ; `0,1`, `429496730`
/// (`429496729,6` : un arrondi par défaut donnerait `…729`) ; `0,2`, `858993459` ; `0,3`, `1288490189` ; `0,9`, `3865470566` ; `0,05`, `214748365`
/// (`214748364,8`) ; `0,01`, `42949673` ; les demis EXACTS : `2^-33` donne `1` (`0,5` arrondi vers le haut ; l'arrondi au pair donnerait 0),
/// `3·2^-33` donne `2`, `5·2^-33` donne `3` (`2,5` : l'arrondi au pair donnerait 2) ; `0,9999999999` donne `2^32` (`4294967295,57`) et
/// `0,99999999988` donne `4294967295` ; `1,2e-10` donne `1` (`0,515`). Refusés (`None`) : `0`, `-0.0`, `-0,5`, `1,5`, `2`, `1,0000000000000002` (le
/// `f64` qui suit 1), `NaN`, `±∞`, `1,1e-10` (`0,47` : `q` serait 0), `5e-11`.
///
/// Ne couvre pas : la lecture du texte `p` (voir `variante_du_scenario_lit_les_directives`).
#[test]
#[cfg_attr(not(feature = "etape-2"), ignore)]
fn collage_depuis_flottant_a_la_main() {
    let deux_puissance = |k: i32| -> f64 { (0..k.abs()).fold(1.0, |a, _| if k < 0 { a / 2.0 } else { a * 2.0 }) };
    let table: [(f64, u64); 17] = [
        (1.0, 4_294_967_296),
        (0.5, 2_147_483_648),
        (0.25, 1_073_741_824),
        (0.75, 3_221_225_472),
        (0.1, 429_496_730),
        (0.2, 858_993_459),
        (0.3, 1_288_490_189),
        (0.9, 3_865_470_566),
        (0.05, 214_748_365),
        (0.01, 42_949_673),
        (deux_puissance(-33), 1),
        (3.0 * deux_puissance(-33), 2),
        (5.0 * deux_puissance(-33), 3),
        (0.9999999999, 4_294_967_296),
        (0.99999999988, 4_294_967_295),
        (1.2e-10, 1),
        (deux_puissance(-32), 1),
    ];
    for (p, attendu) in table {
        assert_eq!(collage_depuis_flottant(p), Some(attendu), "p = {p:e}");
    }
    assert_eq!(collage_depuis_flottant(1.0), Some(TOUJOURS));
    assert_eq!(deux_puissance(-33), 1.0 / 8_589_934_592.0);
    for p in
        [0.0, -0.0, -0.5, 1.5, 2.0, 1.000_000_000_000_000_2, f64::NAN, f64::INFINITY, f64::NEG_INFINITY, 1.1e-10, 5e-11]
    {
        assert_eq!(collage_depuis_flottant(p), None, "p = {p:e}");
    }
}

// ---------------------------------------------------------------------------------------------- les directives

fn variante_de(texte: &str) -> Result<Option<Variante>, ErreurLecture> {
    variante_du_scenario(&lire_scenario(texte).expect("scénario valide"))
}

/// juste : `variante_du_scenario` lit la directive `collage <p>` ou `bruit <m>` (V-DIR2-01) : aucune des deux donne `Ok(None)` (les autres directives,
/// `dla` comprise, sont ignorées) ; `collage 0.5` donne `Collage(2^31)`, `collage 1` et `collage 1.0` et `collage 01` donnent `Collage(TOUJOURS)`,
/// `collage 0.05` donne `214748365`, `collage 0.9999999999` donne `TOUJOURS` ; `bruit 1`, `bruit 05` (zéros de tête : 5) et `bruit 1000000`
/// (`BRUIT_MAX`) sont acceptés ; la directive peut venir avant ou après la forme et la directive `dla`, et se lit parmi des commentaires
/// et d'autres directives.
///
/// Ne couvre pas : les refus (voir `variante_du_scenario_refuse`), ni l'usage par `lancer`.
#[test]
#[cfg_attr(not(feature = "etape-2"), ignore)]
fn variante_du_scenario_lit_les_directives() {
    assert_eq!(variante_de("boite 5 5\n"), Ok(None));
    assert_eq!(variante_de("boite 5 5\ndla 10\ninconnue 1\n"), Ok(None));
    assert_eq!(variante_de("boite 5 5\ncollage 0.5\n"), Ok(Some(Variante::Collage(2_147_483_648))));
    assert_eq!(variante_de("boite 5 5\ncollage 1\n"), Ok(Some(Variante::Collage(TOUJOURS))));
    assert_eq!(variante_de("boite 5 5\ncollage 1.0\n"), Ok(Some(Variante::Collage(TOUJOURS))));
    assert_eq!(variante_de("boite 5 5\ncollage 01\n"), Ok(Some(Variante::Collage(TOUJOURS))));
    assert_eq!(variante_de("boite 5 5\ncollage 0.05\n"), Ok(Some(Variante::Collage(214_748_365))));
    assert_eq!(variante_de("boite 5 5\ncollage 0.9999999999\n"), Ok(Some(Variante::Collage(TOUJOURS))));
    assert_eq!(variante_de("boite 5 5\ncollage 0.1\n"), Ok(Some(Variante::Collage(429_496_730))));
    assert_eq!(variante_de("boite 5 5\nbruit 1\n"), Ok(Some(Variante::Bruit(1))));
    assert_eq!(variante_de("boite 5 5\nbruit 05\n"), Ok(Some(Variante::Bruit(5))));
    assert_eq!(variante_de("boite 5 5\nbruit 1000000\n"), Ok(Some(Variante::Bruit(BRUIT_MAX))));
    assert_eq!(variante_de("bruit 3\nboite 5 5\ndla 40\n"), Ok(Some(Variante::Bruit(3))));
    assert_eq!(
        variante_de("# c\nboite 5 5\n\ninconnue\ncollage 0.25\ndla 12\n"),
        Ok(Some(Variante::Collage(1_073_741_824)))
    );
}

/// juste : les refus de `variante_du_scenario` (V-DIR2-02), toujours `Err(ErreurLecture::LigneInvalide(ligne))` avec le numéro de la ligne fautive
/// (lignes ignorées et commentaires comptés ; ici la directive est ligne 3 après `# c` et `boite`). `collage` : sans argument, deux arguments, `0`,
/// `0.0`, `-0.5`, `1.5`, `.5`, `5.`, `1e-1`, `+0.5`, `x`, `inf`, `0.00000000001` (`q` serait 0). `bruit` : sans argument, deux arguments, `0`, `1000001`, `+2`,
/// `-2`, `2.0`, `x`, `99999999999999999999`. Une SECONDE directive `collage` ou `bruit`, de l'une ou l'autre sorte, valide ou non, est la fautive ;
/// si la première est fautive, c'est la première.
///
/// Ne couvre pas : l'interprétation par `lancer` (voir `etape_2d_lancer`).
#[test]
#[cfg_attr(not(feature = "etape-2"), ignore)]
fn variante_du_scenario_refuse() {
    for mauvais in [
        "collage",
        "collage 0.5 0.5",
        "collage 0",
        "collage 0.0",
        "collage -0.5",
        "collage 1.5",
        "collage .5",
        "collage 5.",
        "collage 1e-1",
        "collage +0.5",
        "collage x",
        "collage inf",
        "collage 0.00000000001",
        "bruit",
        "bruit 2 3",
        "bruit 0",
        "bruit 1000001",
        "bruit +2",
        "bruit -2",
        "bruit 2.0",
        "bruit x",
        "bruit 99999999999999999999",
    ] {
        assert_eq!(
            variante_de(&format!("# c\nboite 5 5\n{mauvais}\n")),
            Err(ErreurLecture::LigneInvalide(3)),
            "{mauvais}"
        );
    }
    for (texte, ligne) in [
        ("boite 5 5\ncollage 0.5\nbruit 2\n", 3),
        ("boite 5 5\nbruit 2\ncollage 0.5\n", 3),
        ("boite 5 5\nbruit 2\n\nbruit 3\n", 4),
        ("boite 5 5\ncollage 0.5\ncollage 0.4\n", 3),
        ("boite 5 5\ncollage 0.5\ndla 3\nbruit 4\n", 4),
        ("boite 5 5\ncollage 0\nbruit 2\n", 2),
        ("boite 5 5\ncollage 0\ncollage 0.5\n", 2),
        ("boite 5 5\nbruit 2\ncollage 0\n", 3),
        ("boite 5 5\ncollage 0.5\nbruit 0\n", 3),
    ] {
        assert_eq!(variante_de(texte), Err(ErreurLecture::LigneInvalide(ligne)), "{texte:?}");
    }
}

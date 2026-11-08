//! Étape 3 : la lecture des paramètres (`lire_decimal`, `Parametres::lire`, `Parametres::sont_valides`) et le refus des
//! paramètres invalides par `Flocon::new`. Valeurs attendues : `references/etape-3-valeurs.md`, identifiants `V-LECT-xx`.
//!
//! Les `brut` attendus viennent de Python (`fractions`, entiers de taille quelconque) : `floor(N * 2^32 / 10^k + 1/2)`,
//! sans flottant ; un `brut` écrit à la main (`0.5` donne `2^31`, `0.1` donne `429496730`) en recoupe quelques-uns.

mod etape_3_commun;

use etape_3_commun::valeurs::DECIMAUX;
use givre_lab::flocon::{Flocon, Parametres, lire_decimal};
use givre_lab::math::Fixe;

/// juste : `lire_decimal` rend l'arrondi UNIQUE de `N / 10^k` (au plus proche, demi-cas vers `+∞`), calculé en entiers
/// exacts par Python (V-LECT-01) : 27 textes dont `0.5`, `0.1`, `0.0001`, `0.35`, `1.5`, `007`, `1.00`, 18 chiffres, un texte
/// juste sous et un texte juste au-dessus de `2^-33` (`0.00000000011641532` donne 0, `0.00000000011641533` donne 1), le plus
/// grand entier permis, et deux textes qu'une conversion par flottant arrondit à l'envers : `1.00000000011641532` (un peu sous
/// `1 + 2^-33` : exactement `2^32`, mais le flottant le plus proche est `1 + 2^-33` exactement, donc `2^32 + 1`) et
/// `1.00000000034924596` (un peu sous `1 + 3 * 2^-33` : `2^32 + 1`, pas `2^32 + 2`). Recoupé à la main : `0.5` a pour `brut` `2^31`,
/// `0.1` a pour `brut` 429496730, `1` vaut `Fixe::UN`. Une conversion par un flottant, une troncature ou un arrondi par défaut
/// échouent ici.
///
/// Ne couvre pas : les textes refusés (voir `lire_decimal_refuse`).
#[test]
#[cfg_attr(not(feature = "etape-3"), ignore)]
fn lire_decimal_valeurs_exactes() {
    for &(texte, brut) in DECIMAUX {
        assert_eq!(lire_decimal(texte), Some(Fixe(brut)), "texte {texte:?}");
    }
    assert_eq!(lire_decimal("0.5"), Some(Fixe(1 << 31)));
    assert_eq!(lire_decimal("0.25"), Some(Fixe(1 << 30)));
    assert_eq!(lire_decimal("0.1"), Some(Fixe(429_496_730)));
    assert_eq!(lire_decimal("1"), Some(Fixe::UN));
    assert_eq!(lire_decimal("0"), Some(Fixe::ZERO));
    assert_eq!(lire_decimal("2"), Some(Fixe::entier(2)));
}

/// juste : les textes qui ne sont pas un décimal simple sont refusés (V-LECT-02) : vide, `.5`, `5.`, signe (`-0.5`, `+1`),
/// exposant (`1e3`), virgule, espace avant ou après, deux points, lettres, préfixe `0x`, chiffres non ASCII (arabe-indien,
/// pleine largeur), plus de 18 chiffres (19 : `0.123456789012345678`), et les valeurs `2^31` ou plus (`2147483648`,
/// `9999999999`) ; et acceptés : `2147483647` (le plus grand entier), un texte de 18 chiffres, `007`.
///
/// Ne couvre pas : la validité des paramètres (voir `parametres_bornes`).
#[test]
#[cfg_attr(not(feature = "etape-3"), ignore)]
fn lire_decimal_refuse() {
    let refuses = [
        "",
        ".5",
        "5.",
        ".",
        "-0.5",
        "-0",
        "+1",
        "1e3",
        "1E3",
        "0,5",
        " 0.5",
        "0.5 ",
        "0 .5",
        "0..5",
        "1.2.3",
        "abc",
        "0x10",
        "inf",
        "nan",
        "\u{663}",
        "\u{ff11}",
        "0.\u{ff11}",
        "0.123456789012345678",
        "1234567890123456789",
        "2147483648",
        "9999999999",
        "2147483648.5",
    ];
    for texte in refuses {
        assert_eq!(lire_decimal(texte), None, "texte {texte:?} refusé");
    }
    assert_eq!(lire_decimal("2147483647"), Some(Fixe((2_147_483_647_i64) << 32)));
    assert_eq!(lire_decimal("0.00000000000000001"), Some(Fixe::ZERO)); // 18 chiffres, arrondi à 0
    assert_eq!(lire_decimal("007"), Some(Fixe::entier(7)));
}

/// juste : `Parametres::lire` rend les trois `Fixe` lus et rien d'autre (V-LECT-03) : `1`, `0.4`, `0.001` donnent les `brut`
/// `2^32`, 1717986918 et 4294967 (Python : `floor(0.4 * 2^32 + 1/2)` et `floor(0.001 * 2^32 + 1/2)`) ; un texte mal écrit dans
/// l'un des trois champs donne `None`.
///
/// Ne couvre pas : les bornes (voir `parametres_bornes`).
#[test]
#[cfg_attr(not(feature = "etape-3"), ignore)]
fn parametres_lire() {
    let p = Parametres::lire("1", "0.4", "0.001").expect("valide");
    assert_eq!(p, Parametres { alpha: Fixe::UN, beta: Fixe(1_717_986_918), gamma: Fixe(4_294_967) });
    assert_eq!(Parametres::lire("x", "0.4", "0.001"), None);
    assert_eq!(Parametres::lire("1", "0,4", "0.001"), None);
    assert_eq!(Parametres::lire("1", "0.4", ""), None);
    assert_eq!(Parametres::lire("1", "0.4", "-0.001"), None);
}

/// juste : les bornes de validité (V-LECT-04) : `0 <= α <= 2`, `0 <= β < 1`, `0 <= γ <= 1`, jugées sur le `Fixe` lu. `lire`
/// accepte `α = 2` et `0`, `β = 0`, `γ = 0` et `1`, et refuse `α > 2` (`2.0000001` dépasse `2^33` en `brut`, `3`), `β = 1` et
/// `β > 1`, `γ > 1`. Un cas d'arrondi : `0.9999999999` s'arrondit à `Fixe::UN` (refusé : `β < 1`), `0.9999999998` à `UN - 1`
/// (accepté). `sont_valides` donne les mêmes verdicts sur des `Fixe` construits à la main (`UN - 1` accepté, `UN` refusé pour
/// `β`, `2^33 + 1` refusé pour `α`, `UN + 1` refusé pour `γ`, `-1` refusé partout).
///
/// Ne couvre pas : les paniques de `Flocon::new` (voir `new_refuse_les_parametres_invalides`).
#[test]
#[cfg_attr(not(feature = "etape-3"), ignore)]
fn parametres_bornes() {
    assert!(Parametres::lire("2", "0", "0").is_some());
    assert!(Parametres::lire("0", "0.99", "1").is_some());
    assert_eq!(Parametres::lire("2.0000001", "0.5", "0.5"), None);
    assert_eq!(Parametres::lire("3", "0.5", "0.5"), None);
    assert_eq!(Parametres::lire("1", "1", "0.5"), None);
    assert_eq!(Parametres::lire("1", "1.5", "0.5"), None);
    assert_eq!(Parametres::lire("1", "0.5", "1.0000001"), None);
    assert_eq!(Parametres::lire("1", "0.5", "2"), None);
    assert_eq!(Parametres::lire("1", "0.9999999999", "0.1"), None);
    let presque = Parametres::lire("1", "0.9999999998", "0.1").expect("valide : UN - 1");
    assert_eq!(presque.beta, Fixe(Fixe::UN.0 - 1));

    let un = Fixe::UN;
    let deux = Fixe::entier(2);
    let valide = |a: Fixe, b: Fixe, g: Fixe| Parametres { alpha: a, beta: b, gamma: g }.sont_valides();
    assert!(valide(un, Fixe::ZERO, Fixe::ZERO));
    assert!(valide(deux, Fixe(un.0 - 1), un));
    assert!(valide(Fixe::ZERO, Fixe::ZERO, Fixe::ZERO));
    assert!(!valide(Fixe(deux.0 + 1), Fixe::ZERO, Fixe::ZERO));
    assert!(!valide(un, un, Fixe::ZERO));
    assert!(!valide(un, Fixe::ZERO, Fixe(un.0 + 1)));
    assert!(!valide(Fixe(-1), Fixe::ZERO, Fixe::ZERO));
    assert!(!valide(un, Fixe(-1), Fixe::ZERO));
    assert!(!valide(un, Fixe::ZERO, Fixe(-1)));
}

/// juste : `Flocon::new` panique pour un rayon 0 et pour des paramètres invalides (`β = 1`, `α > 2`, `γ > 1`, un champ négatif),
/// comme le dit sa doc (V-LECT-05) ; et il accepte les paramètres valides aux bornes.
///
/// Ne couvre pas : le contenu de l'état (voir `etape_3_etat`).
#[test]
#[cfg_attr(not(feature = "etape-3"), ignore)]
fn new_refuse_les_parametres_invalides() {
    let valides = Parametres { alpha: Fixe::UN, beta: Fixe::depuis_f64(0.5), gamma: Fixe::ZERO };
    let paniquent = |rayon: u32, p: Parametres| std::panic::catch_unwind(|| Flocon::new(rayon, p)).is_err();
    assert!(paniquent(0, valides));
    assert!(paniquent(3, Parametres { beta: Fixe::UN, ..valides }));
    assert!(paniquent(3, Parametres { alpha: Fixe(Fixe::entier(2).0 + 1), ..valides }));
    assert!(paniquent(3, Parametres { gamma: Fixe(Fixe::UN.0 + 1), ..valides }));
    assert!(paniquent(3, Parametres { alpha: Fixe(-1), ..valides }));
    assert!(paniquent(3, Parametres { beta: Fixe(-1), ..valides }));
    assert!(paniquent(3, Parametres { gamma: Fixe(-1), ..valides }));
    assert!(!paniquent(1, valides));
    assert!(!paniquent(3, Parametres { alpha: Fixe::entier(2), beta: Fixe(Fixe::UN.0 - 1), gamma: Fixe::UN }));
    assert!(!paniquent(3, Parametres { alpha: Fixe::ZERO, beta: Fixe::ZERO, gamma: Fixe::ZERO }));
}

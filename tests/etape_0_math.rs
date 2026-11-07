//! Étape 0 : `math`, la racine entière, le logarithme et la virgule fixe Q31.32. Valeurs attendues :
//! `references/etape-0-valeurs.md`, identifiants `V-RACINE-xx`, `V-LOG-xx`, `V-FIXE-xx`. (La table `EXP_NEG` est dans
//! `etape_0_exp_neg.rs`.)
//!
//! Sources : la définition de la partie entière de la racine, vérifiée en entiers exacts ; des logarithmes exacts calculés
//! en 50 chiffres décimaux (module `decimal` de Python, fonction `ln`) et arrondis au `f64` le plus proche ; pour la virgule
//! fixe, `floor(valeur exacte + 1/2)` calculé en rationnels exacts (module `fractions` de Python).

mod common;

use givre_lab::alea::Alea;
use givre_lab::math::{Fixe, logarithme, racine_entiere};

// ---------------------------------------------------------------------------------------------------------------------
// racine_entiere
// ---------------------------------------------------------------------------------------------------------------------

/// Vrai si `r` est la partie entière exacte de la racine de `n` : `r² <= n < (r + 1)²`, en `u128` (aucun débordement).
fn est_racine_entiere(n: u64, r: u64) -> bool {
    let r = u128::from(r);
    r * r <= u128::from(n) && u128::from(n) < (r + 1) * (r + 1)
}

/// juste : pour tout `n` de 0 à 70 000, le résultat `r` vérifie `r² <= n < (r + 1)²` (la définition de la partie entière,
/// comparée en `u128`) (V-RACINE-01).
///
/// Ne couvre pas : les grandes valeurs (voir `racine_carres_et_voisins` et `racine_extremes`).
#[test]
#[cfg_attr(not(feature = "etape-0"), ignore)]
fn racine_petites_valeurs_exhaustif() {
    for n in 0..=70_000_u64 {
        let r = racine_entiere(n);
        assert!(est_racine_entiere(n, r), "racine_entiere({n}) = {r}");
    }
}

/// juste : pour des `k` de 0 à 3 000, des puissances de 2 jusqu'à `2^32 - 1`, les quatre plus grands entiers de 32 bits et
/// 5 000 entiers de 32 bits pseudo-aléatoires, on a `racine(k²) = k`, `racine(k² - 1) = k - 1`, `racine(k² + 2k) = k` (le
/// plus grand `n` de racine `k`) et, quand `(k + 1)²` tient dans un `u64`, `racine(k² + 2k + 1) = k + 1` : les deux côtés de
/// chaque carré parfait, où une racine par `f64` se trompe (V-RACINE-01).
///
/// Ne couvre pas : les `n` intermédiaires entre ces carrés (le test exhaustif jusqu'à 70 000 s'en charge pour les petits).
#[test]
#[cfg_attr(not(feature = "etape-0"), ignore)]
fn racine_carres_et_voisins() {
    let mut ks: Vec<u64> = (0..=3000).collect();
    ks.extend((0..32).map(|e| 1_u64 << e));
    ks.extend([u64::from(u32::MAX), u64::from(u32::MAX) - 1, u64::from(u32::MAX) - 2, u64::from(u32::MAX) - 3]);
    let mut alea = Alea::new(77);
    ks.extend((0..5000).map(|_| u64::from(alea.suivant())));
    for k in ks {
        let carre = k * k;
        assert_eq!(racine_entiere(carre), k, "racine({carre})");
        if k >= 1 {
            assert_eq!(racine_entiere(carre - 1), k - 1, "racine({})", carre - 1);
        }
        assert_eq!(racine_entiere(carre + 2 * k), k, "racine({})", carre + 2 * k);
        if k < u64::from(u32::MAX) {
            assert_eq!(racine_entiere(carre + 2 * k + 1), k + 1, "racine({})", carre + 2 * k + 1);
        }
    }
}

/// juste : les valeurs extrêmes : `racine(u64::MAX) = 4294967295`, `racine(4294967295²) = 4294967295` avec
/// `4294967295² = 18446744065119617025`, `racine(4294967295² - 1) = 4294967294`, `racine(2^63) = 3037000499`
/// (`3037000499² = 9223372030926249001 <= 2^63 < 3037000500² = 9223372037000250000`), `racine(2^62) = 2^31`, et les petites
/// valeurs 0 à 4 (V-RACINE-02, calcul entier exact).
///
/// Ne couvre pas : les valeurs entre 2^32 et 2^62 en dehors des carrés (une propriété générale, que les autres tests voient).
#[test]
#[cfg_attr(not(feature = "etape-0"), ignore)]
fn racine_extremes() {
    assert_eq!(racine_entiere(u64::MAX), 4_294_967_295);
    assert_eq!(racine_entiere(18_446_744_065_119_617_025), 4_294_967_295);
    assert_eq!(racine_entiere(18_446_744_065_119_617_024), 4_294_967_294);
    assert_eq!(racine_entiere(1 << 63), 3_037_000_499);
    assert_eq!(racine_entiere(9_223_372_030_926_249_001), 3_037_000_499);
    assert_eq!(racine_entiere(9_223_372_030_926_249_000), 3_037_000_498);
    assert_eq!(racine_entiere(1 << 62), 1 << 31);
    assert_eq!(racine_entiere(u64::MAX - 1), 4_294_967_295);
    for (n, r) in [(0, 0), (1, 1), (2, 1), (3, 1), (4, 2)] {
        assert_eq!(racine_entiere(n), r, "racine({n})");
    }
}

// ---------------------------------------------------------------------------------------------------------------------
// logarithme
// ---------------------------------------------------------------------------------------------------------------------

/// Logarithmes exacts (V-LOG-01) : `(x, ln x)` où `ln x` est le logarithme exact de `x` (la valeur exacte du `f64` `x`),
/// calculé en 50 chiffres décimaux et arrondi au `f64` le plus proche.
const LOGARITHMES_EXACTS: &[(f64, f64)] = &[
    (2.0, std::f64::consts::LN_2),
    (10.0, std::f64::consts::LN_10),
    (0.5, -std::f64::consts::LN_2),
    (3.0, 1.0986122886681098),
    (7.0, 1.9459101490553132),
    (0.1, -2.3025850929940455),
    (1e-10, -23.025850929940457),
    (123456.789, 11.723646487185881),
    (1e+300, 690.7755278982137),
    (1e-300, -690.7755278982137),
    (1.7976931348623157e+308, 709.782712893384),
    (2.2250738585072014e-308, -708.3964185322641),
    (5e-324, -744.4400719213812),
    (1.0000000000000002, 2.2204460492503128e-16),
    (0.9999999999999999, -1.1102230246251565e-16),
    (1.0000001, 9.999999505838704e-08),
    (0.9999999, -1.0000000494736474e-07),
    (1.5, 0.4054651081081644),
    (0.75, -0.2876820724517809),
    (4.0, 1.3862943611198906),
    (1024.0, 6.931471805599453),
    (std::f64::consts::E, 1.0),
    (1e+100, 230.25850929940458),
    (0.3333333333333333, -1.0986122886681098),
    (std::f64::consts::SQRT_2, 0.3465735902799727),
    (std::f64::consts::FRAC_1_SQRT_2, -0.3465735902799726),
];

/// juste : `logarithme(x)` est dans `5e-16` en erreur relative du logarithme exact, sur les 26 valeurs de V-LOG-01, qui
/// couvrent `1 ± 2^-52`, `1e±300`, `f64::MAX`, `f64::MIN_POSITIVE`, le plus petit sous-normal `5e-324`, et les deux côtés de
/// `√2` et `√2/2`. Tolérance : la précision annoncée dans la doc de `logarithme` (`5e-16`) ; le prototype mesure 3,0e-16 au
/// pire contre le logarithme exact (moyenne 5,4e-17, écart-type 4,2e-17 sur 13 402 points, voir
/// `logarithme_contre_ln_du_systeme`) ; la valeur écrite ici est elle-même arrondie (au plus `1,1e-16` d'erreur relative),
/// ce que la marge couvre.
///
/// Ne couvre pas : les valeurs qui ne sont pas dans la liste (voir `logarithme_contre_ln_du_systeme`).
#[test]
#[cfg_attr(not(feature = "etape-0"), ignore)]
fn logarithme_valeurs_exactes() {
    for &(x, attendu) in LOGARITHMES_EXACTS {
        let obtenu = logarithme(x);
        let erreur = ((obtenu - attendu) / attendu).abs();
        assert!(erreur <= 5e-16, "logarithme({x:e}) = {obtenu:e}, attendu {attendu:e}, erreur relative {erreur:e}");
    }
}

/// juste : `logarithme(1.0)` est exactement 0 (V-LOG-02, la doc le dit : « exactement 0 »), alors que ses deux voisins
/// immédiats `1 + 2^-52` et `1 - 2^-53` ont un logarithme non nul, du bon signe (un code qui rend toujours 0 échoue ici).
///
/// Ne couvre pas : le signe du zéro.
#[test]
#[cfg_attr(not(feature = "etape-0"), ignore)]
fn logarithme_de_un_est_zero() {
    assert_eq!(logarithme(1.0), 0.0);
    assert!(logarithme(1.0000000000000002) > 0.0);
    assert!(logarithme(0.9999999999999999) < 0.0);
}

/// juste : le domaine documenté : NaN pour `0.0`, `-0.0`, les négatifs, `-∞` et NaN ; `+∞` pour `+∞` (V-LOG-03, la doc).
///
/// Ne couvre pas : les valeurs strictement positives finies (autres tests).
#[test]
#[cfg_attr(not(feature = "etape-0"), ignore)]
fn logarithme_domaine() {
    for x in [0.0, -0.0, -1.0, -f64::MIN_POSITIVE, -1e300, f64::NEG_INFINITY, f64::NAN, -f64::NAN, -5e-324] {
        assert!(logarithme(x).is_nan(), "logarithme({x:e}) doit être NaN, trouvé {:e}", logarithme(x));
    }
    assert_eq!(logarithme(f64::INFINITY), f64::INFINITY);
}

/// juste : les sous-normaux sont traités (la doc : « traités, non refusés ») : le logarithme exact de huit sous-normaux
/// (`5e-324` = `2^-1074`, `1e-323`, `1,5e-323`, `0x3E8`, `0x40000000`, `0x0000100000`, `0x0008000000000000` et le plus
/// grand `0x000FFFFFFFFFFFFF`), calculé en 50 chiffres décimaux, à `5e-16` près en relatif (V-LOG-04).
///
/// Ne couvre pas : les sous-normaux autres que ceux-là (le test `logarithme_contre_ln_du_systeme` en prend 500 au hasard).
#[test]
#[cfg_attr(not(feature = "etape-0"), ignore)]
fn logarithme_sous_normaux() {
    let attendu: [(u64, f64); 8] = [
        (0x0000_0000_0000_0001, -744.4400719213812),
        (0x0000_0000_0000_0002, -743.7469247408213),
        (0x0000_0000_0000_0003, -743.3414596327132),
        (0x0000_0000_0000_03E8, -737.5323166423991),
        (0x0000_0000_4000_0000, -723.6456565045829),
        (0x0000_0000_0010_0000, -730.5771283101824),
        (0x0008_0000_0000_0000, -709.0895657128241),
        (0x000F_FFFF_FFFF_FFFF, -708.3964185322641),
    ];
    for (bits, ln_x) in attendu {
        let x = f64::from_bits(bits);
        let obtenu = logarithme(x);
        let erreur = ((obtenu - ln_x) / ln_x).abs();
        assert!(erreur <= 5e-16, "logarithme({x:e}) = {obtenu:e}, attendu {ln_x:e}");
    }
    let obtenu = logarithme(1e-310);
    assert!(((obtenu + 713.8013788281542) / 713.8013788281542).abs() <= 5e-16, "logarithme(1e-310) = {obtenu:e}");
}

/// juste : sur 13 402 points fixes (des exposants et des mantisses au hasard, 500 sous-normaux, toutes les puissances de 2,
/// l'entourage de 1, de `√2` et de `√2/2`, une grille de 1/1000 à 1000 ; voir `common::points_logarithme`, V-LOG-05),
/// `|logarithme(x) - f64::ln(x)| <= 5e-16 · |f64::ln(x)|`, et `logarithme(x) = 0` quand `f64::ln(x) = 0` (seulement `x = 1`).
/// Mesure sur le prototype : 3,2e-16 au pire contre `f64::ln` (Windows, UCRT), 3,0e-16 contre le logarithme exact en 50
/// chiffres, atteint juste sous `√2/2` ; `f64::ln` lui-même est à 1,1e-16 du logarithme exact au pire. La tolérance `5e-16`
/// est la précision annoncée par la doc, et elle contient l'erreur du prototype plus celle d'une bibliothèque mathématique
/// d'un autre système (au pire une unité du dernier chiffre, 2,2e-16 en relatif) sans qu'on l'ait élargie pour un cas.
/// `f64::ln` n'est permis que dans ce test (et `regles` vérifie qu'il n'est pas dans `src/`).
///
/// Ne couvre pas : l'exactitude absolue de `f64::ln` (c'est la référence du test, pas une vérité) ; les valeurs hors de ces
/// points.
#[test]
#[cfg_attr(not(feature = "etape-0"), ignore)]
fn logarithme_contre_ln_du_systeme() {
    let points = common::points_logarithme();
    assert_eq!(points.len(), 13_402, "le jeu de points a changé");
    let mut pire = 0.0_f64;
    for &x in &points {
        let reference = x.ln();
        let obtenu = logarithme(x);
        if reference == 0.0 {
            assert_eq!(obtenu, 0.0, "logarithme({x:e})");
            continue;
        }
        let erreur = ((obtenu - reference) / reference).abs();
        assert!(erreur <= 5e-16, "logarithme({x:e}) = {obtenu:e}, f64::ln = {reference:e}, erreur relative {erreur:e}");
        pire = pire.max(erreur);
    }
    assert!(pire > 0.0, "un logarithme qui copie f64::ln à l'identique n'est pas écrit avec + - * /");
}

// ---------------------------------------------------------------------------------------------------------------------
// Fixe
// ---------------------------------------------------------------------------------------------------------------------

/// `(num, den, brut)` : `Fixe::rapport(num, den)` a pour champ brut `floor(num · 2^32 / den + 1/2)` (V-FIXE-02).
const RAPPORTS: &[(i64, i64, i64)] = &[
    (1, 3, 1431655765),
    (2, 3, 2863311531),
    (-1, 3, -1431655765),
    (-2, 3, -2863311531),
    (1, -3, -1431655765),
    (-1, -3, 1431655765),
    (1, 2, 2147483648),
    (1, 8589934592, 1),
    (-1, 8589934592, 0),
    (3, 8589934592, 2),
    (-3, 8589934592, -1),
    (7, 5, 6012954214),
    (-7, 5, -6012954214),
    (1, 7, 613566757),
    (22, 7, 13498468645),
    (2147483647, 1, 9223372032559808512),
    (-2147483648, 1, -9223372036854775808),
    (355, 113, 13493038850),
    (1, 10, 429496730),
    (-1, 10, -429496730),
    (5, 17179869184, 1),
    (-5, 17179869184, -1),
    (268680898207, 437, 2640676592354622513),
    (-771373449553, -2714, 1220716189695225062),
    (-331647595755, -330826371945, 4305628869),
    (863494752763, 284051021305, 13056392849),
    (-779466883764, -1810032408223, 1849571731),
    (-811173594157, -6973, 499636319845560259),
    (47294393188, -1507759, -134721710848103),
    (-997703015442, -9555295, 448453116564583),
    (-8573014605, 16242747997663, -2266908),
    (889750503632, -9099690742991, -419953757),
    (-1058068527941, -1477263870588, 3076207179),
    (-816517286719, -374851770, 9355471478971),
    (-744121884749, 14134, -226119934854594324),
    (-576790637954, -2463897503, 1005438304002),
    (289084776842, 133406997, 9306930597560),
    (-889985842670, 12969594102, -294724727552),
    (-534646870977, 30069390, -76366392060329),
    (-823630863024, -22481726, 157348578159183),
    (682691964908, 9464786814, 309794580707),
    (1045137704207, 55293, 81182649872236750),
    (-997585515750, -1218, 3517731662651513064),
    (-30492984435, -216, 606325791230379804),
    (633035255678, -24266, -112044247933405106),
    (27023151906, -272, -426704241437904655),
    (385374271058, -3595435, -460353167534373),
    (-1055341239748, 13630, -332549971448111173),
    (990607083674, -1325967197030, -3208695537),
    (14636059701, -664316392648, -94625691),
    (939626766580, -10496565330338, -384474931),
    (-41977045666, 100979161, -1785418263855),
    (-794353405599, -52337759, 65186625558689),
    (617914711759, 10950004731, 242367336263),
    (1007405228018, 10552708042, 410015371499),
    (-301932164004, -4101588147, 316167475507),
    (-640910973763, 15685241, -175495656838145),
    (-970449899803, -379697116530, 10977303752),
    (-325559159281, -100300, 13940836909522930),
    (526731953755, -153069785858, -14779510551),
];

/// `(a, b, brut)` : `Fixe(a) * Fixe(b)` a pour champ brut `floor(a · b / 2^32 + 1/2)` (V-FIXE-03).
const PRODUITS: &[(i64, i64, i64)] = &[
    (4294967296, 4294967296, 4294967296),
    (6442450944, 6442450944, 9663676416),
    (2147483648, 1, 1),
    (-2147483648, 1, 0),
    (1, 2147483648, 1),
    (1, -2147483648, 0),
    (3, 2147483648, 2),
    (-3, 2147483648, -1),
    (9223372036854775807, 4294967296, 9223372036854775807),
    (-9223372036854775808, 4294967296, -9223372036854775808),
    (9223372036854775807, -4294967296, -9223372036854775807),
    (-4294967296, -4294967296, 4294967296),
    (8589934592, 2305843009213693951, 4611686018427387902),
    (1, 1, 0),
    (-1, -1, 0),
    (-1, 1, 0),
    (1431655765, 1431655765, 477218588),
    (-252718, 245367, -14),
    (9802, -30990511868460, -70726731),
    (196378584, -10847, -496),
    (278507236, -915, -59),
    (11412725021999, 12603489299, 33490396497583),
    (394370451498, 127509891798856, 11708152852760620),
    (8139868, -136745315, -259161),
    (2802363085543, 258271393537, 168515886017084),
    (56778928, -65684181429, -868336625),
    (-4323, 57523094964867, -57898541),
    (84219591, 111729972173329, 2190902027972),
    (51, -436529, 0),
    (-61429773, -941296023, 13463106),
    (2008037782, 29800651, 13932780),
    (-46154658526, -15, 161),
    (-17918370411, 3351629195459, -13982815063688),
    (5907921805909, 14743518, 20280375957),
    (-174234495865305, 41140434328342, -1668949340208982137),
    (-4279359170987, 151602682550, -151051750852860),
    (-836555543226, -31719781062, 6178244640429),
    (7270092592194, -1788251764, -3026974364751),
    (63863300436, 310873400146708, 4622480215302217),
    (534251362462568, 44008002328, 5474159308474014),
    (738646963235, 590276, 101515459),
    (301947147615, 3387221885, 238130331625),
    (-25220, -55089076805561, 323482444),
    (16755619258564, -2590394202, -10105701856852),
    (-10782796783, 17677258575973, -44379915787201),
    (32256461793, -313674, -2355784),
    (-203355603, -14714314891474, 696684787396),
    (-3158805, -11263002821887, 8283562406),
    (537235, -81867829, -10240),
    (-98589961335195, -230578809, 5292882179847),
    (1986311187532, -1061715473544, -491015921131773),
    (36040253963110, -20409120347, -171258552109254),
    (7198600144, -7014272243487, -11756303995387),
    (4233368057275, -122050391067, -120299921119809),
    (2519873742910, 2808, 1647464),
    (1641441155, -102052739837085, -39002291660081),
    (-1878678, -6962374670881, 3045438817),
    (-8294979436456, -440435925944, 850625091414418),
    (102, 343316, 0),
    (-33364628001, 7364844402703, -57212378313167),
];

/// `(brut, n, résultat)` : `Fixe(brut).div_entier(n)` a pour champ brut `floor(brut / n + 1/2)` (V-FIXE-04).
const QUOTIENTS: &[(i64, i64, i64)] = &[
    (5, 2, 3),
    (-5, 2, -2),
    (7, 2, 4),
    (-7, 2, -3),
    (-3, 2, -1),
    (-1, 2, 0),
    (1, 2, 1),
    (-8, 3, -3),
    (8, 3, 3),
    (-8, -3, 3),
    (8, -3, -3),
    (21, 6, 4),
    (-21, 6, -3),
    (4294967296, 3, 1431655765),
    (4294967296, 6, 715827883),
    (-4294967296, 6, -715827883),
    (9223372036854775807, 2, 4611686018427387904),
    (-9223372036854775808, 2, -4611686018427387904),
    (-9223372036854775808, 3, -3074457345618258603),
    (9223372036854775807, -7, -1317624576693539401),
    (0, 5, 0),
    (4294967296, 1, 4294967296),
    (4294967296, -1, -4294967296),
    (1, 1099511627776, 0),
    (1, 4611686018427387904, 0),
    (149574129925644, -782565, -191133171),
    (4128693804351338454, -5, -825738760870267691),
    (-558586638052994393, 6951004, -80360569215),
    (52525047084453261, 1, 52525047084453261),
    (-201656998939446, -26, 7756038420748),
    (36149667, 7166, 5045),
    (-4372, 1311626, 0),
    (-23932672879801293, -8056996, 2970421343),
    (812996441881247, 140, 5807117442009),
    (-597229763665575426, 409243, -1459352423048),
    (-11327649384, 16828260, -673),
    (92456426, -4, -23114106),
    (2204820993021250, 787217237303, 2801),
    (-547490235248135372, 239610, -2284922312291),
    (820360587, -119, -6893786),
    (-96405858071593, -6293, 15319538864),
    (117667771836, -18665029510, -6),
    (-1973100448584182, 7, -281871492654883),
    (-64013458, 10332324358, 0),
    (-7676392580, 78, -98415289),
    (-134481, 15, -8965),
    (2104327979997, 12852403, 163730),
    (-9709664398396, 7166133, -1354938),
    (-959782, 69243946, 0),
    (893017312010, -967, -923492567),
    (925845741492250, -82, -11290801725515),
    (568, -28143478, 0),
    (16159980754378405, 872995, 18510965990),
    (-569015485731, 78875698, -7214),
    (-17253459912882657, 2970552333, -5808166),
    (-11548358181548, 25, -461934327262),
    (769078139849312, -220320512607, -3491),
    (-11870669, 2196396, -5),
    (-16201642520756, 6869588992, -2358),
    (8509609756353963, 50689741478, 167876),
];

/// `(bits de x, brut)` : `Fixe::depuis_f64(x)` a pour champ brut `floor(x · 2^32 + 1/2)` (V-FIXE-05).
const CONVERSIONS: &[(u64, i64)] = &[
    (0x3FE0000000000000, 2147483648),
    (0x3FF0000000000000, 4294967296),
    (0xBFF8000000000000, -6442450944),
    (0x3FB999999999999A, 429496730),
    (0xBFB999999999999A, -429496730),
    (0x4008000000000000, 12884901888),
    (0x41DFFFFFFFE00000, 9223372034707292160),
    (0xC1DFFFFFFFE00000, -9223372034707292160),
    (0x3DF0000000000000, 1),
    (0xBDF0000000000000, -1),
    (0x3DE0000000000000, 1),
    (0xBDE0000000000000, 0),
    (0x3DF8000000000000, 2),
    (0xBDF8000000000000, -1),
    (0x3E04000000000000, 3),
    (0x3FD3333333333333, 1288490189),
    (0x3FD5555555555555, 1431655765),
    (0x3DDFFFFFFF768FA1, 0),
    (0x3D719799812DEA11, 0),
    (0xBD719799812DEA11, 0),
    (0xC10C8F5ACDB64A29, -1004864938625349),
    (0xC089197A20E04EA6, -3449651731466),
    (0xBFC97DED345BE452, -855366249),
    (0x4122A2F8576651BC, 2622868541352158),
    (0x41277506085F4326, 3301296617529747),
    (0x3FAB584E883837CF, 229386052),
    (0xBF466CE9F156B716, -2939348),
    (0x3F472B2F7591665E, 3036767),
    (0xBF32AFA363069251, -1224611),
    (0x3F410E71F00225DD, 2235620),
    (0x3FC9D8D28849164E, 867280145),
    (0x3F4E7C67A79DE4DE, 3995855),
    (0xC127AAFCA392D423, -3330963258567185),
    (0xBF4F9CC29CA6144F, -4143493),
    (0xC0521C0457DD9FA8, -311117832054),
    (0xC08E784D68E3099D, -4187755453537),
    (0x41221767E46BF70E, 2546142281071495),
    (0xC110DA4D77FBFC78, -1185906471927582),
    (0xBFC1205D2F040EE1, -574667358),
    (0xC0847DA0C8D636AF, -2816225123015),
    (0xC10A6F18D73DF084, -930062734442000),
    (0xC1133BBA16D4DDBD, -1353423747626863),
    (0xBF41C78F3298EFA0, -2330398),
    (0xBFEBE28456F6A378, -3742638776),
    (0x3F0D0061DBC9BDE8, 237580),
    // juge : `x · 2^32` entier impair entre 2^52 et 2^53, où `(x · 2^32 + 0.5).floor()` en flottant arrondit faux
    (0x4130000000000001, 4503599627370497),
    (0xC130000000000001, -4503599627370497),
    (0x4130000000000003, 4503599627370499),
];

/// `(brut, bits du flottant)` : `Fixe(brut).vers_f64()` est exactement `brut / 2^32`, pour `|brut| <= 2^53` (V-FIXE-06).
const VERS_FLOTTANT: &[(i64, u64)] = &[
    (0, 0x0000000000000000),
    (1, 0x3DF0000000000000),
    (-1, 0xBDF0000000000000),
    (4294967296, 0x3FF0000000000000),
    (-4294967296, 0xBFF0000000000000),
    (2147483648, 0x3FE0000000000000),
    (30064771072, 0x401C000000000000),
    (-30064771072, 0xC01C000000000000),
    (6442450944, 0x3FF8000000000000),
    (9007199254740992, 0x4140000000000000),
    (-9007199254740992, 0xC140000000000000),
    (9007199254740991, 0x413FFFFFFFFFFFFF),
    (123456789012345, 0x40DC12218377DE40),
    (-987654321098765, 0xC10C122187C28068),
];

/// juste : les constantes et `entier` : `UN` a pour brut `2^32 = 4294967296`, `ZERO` 0, `Fixe::default()` est zéro,
/// `entier(n)` a pour brut `n · 2^32` y compris aux extrêmes de `i32` (V-FIXE-01).
///
/// Ne couvre pas : les opérations (autres tests).
#[test]
#[cfg_attr(not(feature = "etape-0"), ignore)]
fn fixe_constantes_et_entier() {
    assert_eq!(Fixe::UN, Fixe(4_294_967_296));
    assert_eq!(Fixe::ZERO, Fixe(0));
    assert_eq!(Fixe::default(), Fixe::ZERO);
    assert_eq!(Fixe::entier(0), Fixe::ZERO);
    assert_eq!(Fixe::entier(1), Fixe::UN);
    assert_eq!(Fixe::entier(3), Fixe(3 * 4_294_967_296));
    assert_eq!(Fixe::entier(-1), Fixe(-4_294_967_296));
    assert_eq!(Fixe::entier(i32::MAX), Fixe(2_147_483_647 * 4_294_967_296));
    assert_eq!(Fixe::entier(i32::MIN), Fixe(i64::MIN));
}

/// juste : `rapport` arrondit au plus proche, demi-cas vers +∞ : 60 cas (dont `1/3 → 1431655765`, `2/3 → 2863311531`,
/// `-1/3 → -1431655765`, les demi-cas `1/2^33 → 1`, `-1/2^33 → 0`, `3/2^33 → 2`, `-3/2^33 → -1`, les dénominateurs
/// négatifs, les extrêmes `(2^31 - 1)/1` et `-2^31/1`) contre `floor(num · 2^32 / den + 1/2)` en rationnels exacts (V-FIXE-02).
///
/// Ne couvre pas : les débordements (voir `fixe_debordements_et_zero`).
#[test]
#[cfg_attr(not(feature = "etape-0"), ignore)]
fn fixe_rapport() {
    for &(num, den, brut) in RAPPORTS {
        assert_eq!(Fixe::rapport(num, den), Fixe(brut), "rapport({num}, {den})");
    }
}

/// juste : `*` est le produit arrondi (au plus proche, demi-cas vers +∞), calculé sur 128 bits : 60 cas dont `UN * UN`,
/// `Fixe(2^31) * Fixe(1) = 1`, `Fixe(-2^31) * Fixe(1) = 0`, `Fixe(i64::MAX) * UN = i64::MAX` (dont le produit intermédiaire
/// tient sur 95 bits) ; commutatif ; contre `floor(a · b / 2^32 + 1/2)` en rationnels exacts (V-FIXE-03).
///
/// Ne couvre pas : le débordement du résultat (voir `fixe_debordements_et_zero`).
#[test]
#[cfg_attr(not(feature = "etape-0"), ignore)]
fn fixe_produit() {
    for &(a, b, brut) in PRODUITS {
        assert_eq!(Fixe(a) * Fixe(b), Fixe(brut), "{a} * {b}");
        assert_eq!(Fixe(b) * Fixe(a), Fixe(brut), "{b} * {a}");
    }
    assert_eq!(Fixe::UN * Fixe::UN, Fixe::UN);
    assert_eq!(Fixe::entier(-3) * Fixe::entier(5), Fixe::entier(-15));
}

/// juste : `div_entier` arrondit au plus proche, demi-cas vers +∞, et accepte un diviseur négatif : 60 cas (dont `5/2 → 3`,
/// `-5/2 → -2`, `-7/2 → -3`, `-8/3 → -3`, `8/-3 → -3`, `i64::MAX/2`, `i64::MIN/3`) contre `floor(brut / n + 1/2)` en
/// rationnels exacts ; la moyenne de six voisins `(somme).div_entier(6)` en est un cas (V-FIXE-04).
///
/// Ne couvre pas : la division par zéro et le débordement (voir `fixe_debordements_et_zero`).
#[test]
#[cfg_attr(not(feature = "etape-0"), ignore)]
fn fixe_division_entiere() {
    for &(brut, n, attendu) in QUOTIENTS {
        assert_eq!(Fixe(brut).div_entier(n), Fixe(attendu), "{brut} / {n}");
    }
    // moyenne de six voisins : (3 + 5 + 5 + 6 + 8 + 9) / 6 = 6 exactement ; 21 / 6 = 3,5 monte à 4 ; -21 / 6 = -3,5 monte à -3 ;
    // UN / 6 = 715827882,67 monte à 715827883
    let somme = [3, 5, 5, 6, 8, 9].map(Fixe::entier).into_iter().fold(Fixe::ZERO, |a, b| a + b);
    assert_eq!(somme.div_entier(6), Fixe::entier(6));
    assert_eq!([1, 2, 3, 4, 5, 6].map(Fixe).into_iter().fold(Fixe::ZERO, |a, b| a + b).div_entier(6), Fixe(4));
    assert_eq!([-1, -2, -3, -4, -5, -6].map(Fixe).into_iter().fold(Fixe::ZERO, |a, b| a + b).div_entier(6), Fixe(-3));
    assert_eq!(Fixe::UN.div_entier(6), Fixe(715_827_883));
}

/// juste : `+`, `-` et l'opposé sont exacts sur les champs bruts, et l'ordre est celui des valeurs (V-FIXE-07).
///
/// Ne couvre pas : les débordements (autre test).
#[test]
#[cfg_attr(not(feature = "etape-0"), ignore)]
fn fixe_somme_difference_opposee_ordre() {
    assert_eq!(Fixe(5) + Fixe(7), Fixe(12));
    assert_eq!(Fixe(5) - Fixe(7), Fixe(-2));
    assert_eq!(-Fixe(5), Fixe(-5));
    assert_eq!(-Fixe(-9), Fixe(9));
    assert_eq!(Fixe(i64::MAX - 1) + Fixe(1), Fixe(i64::MAX));
    assert_eq!(Fixe(i64::MIN + 1) - Fixe(1), Fixe(i64::MIN));
    assert_eq!(-Fixe(i64::MIN + 1), Fixe(i64::MAX));
    assert_eq!(Fixe::entier(1) + Fixe::entier(2), Fixe::entier(3));
    assert!(Fixe(-1) < Fixe(0) && Fixe(0) < Fixe(1) && Fixe::entier(2) > Fixe::UN);
    assert_eq!(Fixe(3).max(Fixe(-4)), Fixe(3));
    assert_eq!(Fixe(3).min(Fixe(-4)), Fixe(-4));
}

/// juste : conversion d'un flottant : arrondi au plus proche, demi-cas vers +∞ (`0.5 · 2^-32 → 1`, `-0.5 · 2^-32 → 0`,
/// `1.5 · 2^-32 → 2`, `-1.5 · 2^-32 → -1`), 48 valeurs contre `floor(x · 2^32 + 1/2)` en rationnels exacts (V-FIXE-05),
/// dont trois où `x · 2^32` est un entier impair au-delà de 2^52 (`x = ±(2^20 + 2^-32)`, `2^20 + 3 · 2^-32`) : là, ajouter
/// `0.5` en flottant arrondit (au pair) et un `(x · 2^32 + 0.5).floor()` se trompe d'une unité.
///
/// Ne couvre pas : les valeurs hors de `]-2^31, 2^31[` (voir `fixe_debordements_et_zero`).
#[test]
#[cfg_attr(not(feature = "etape-0"), ignore)]
fn fixe_depuis_f64() {
    for &(bits, brut) in CONVERSIONS {
        let x = f64::from_bits(bits);
        assert_eq!(Fixe::depuis_f64(x), Fixe(brut), "depuis_f64({x:e})");
    }
    assert_eq!(Fixe::depuis_f64(0.5), Fixe(1 << 31));
    assert_eq!(Fixe::depuis_f64(1.0), Fixe::UN);
    assert_eq!(Fixe::depuis_f64(0.0), Fixe::ZERO);
    assert_eq!(Fixe::depuis_f64(-0.0), Fixe::ZERO);
}

/// juste : `vers_f64` est exact, `brut / 2^32`, pour `|brut| <= 2^53` : 14 cas (V-FIXE-06) ; l'aller-retour
/// `depuis_f64(vers_f64(x)) = x` sur 2 000 champs bruts pseudo-aléatoires de moins de 2^53 ; au-delà de 2^53 l'arrondi est
/// celui de `brut as f64` : `Fixe(i64::MAX).vers_f64() = 2^31`.
///
/// Ne couvre pas : l'arrondi précis au-delà de 2^53 (seul `i64::MAX` est vérifié).
#[test]
#[cfg_attr(not(feature = "etape-0"), ignore)]
fn fixe_vers_f64() {
    for &(brut, bits) in VERS_FLOTTANT {
        assert_eq!(Fixe(brut).vers_f64().to_bits(), bits, "vers_f64({brut})");
    }
    let mut alea = Alea::new(31);
    for _ in 0..2000 {
        let brut = (alea.suivant_u64() >> 11) as i64 - (1 << 52);
        assert_eq!(Fixe::depuis_f64(Fixe(brut).vers_f64()), Fixe(brut), "aller-retour de {brut}");
    }
    assert_eq!(Fixe(i64::MAX).vers_f64(), 2_147_483_648.0);
}

/// juste : les débordements et la division par zéro font paniquer (la doc : « jamais de rebouclage silencieux ») :
/// `+`, `-`, l'opposé de `Fixe(i64::MIN)`, `*` dont le résultat dépasse 2^31 (`65536 · 65536`), `rapport(_, 0)`, `rapport`
/// dont le résultat dépasse l'intervalle, `div_entier(0)`, `Fixe(i64::MIN).div_entier(-1)`, `depuis_f64` de NaN, de l'infini,
/// de `±2^31` ou de `1e300` (V-FIXE-08). Les bornes incluses ne paniquent pas : `rapport(-2^31, 1)`, `Fixe(i64::MAX) + Fixe(0)`.
///
/// Ne couvre pas : le message des paniques.
#[test]
#[cfg_attr(not(feature = "etape-0"), ignore)]
fn fixe_debordements_et_zero() {
    use std::panic::catch_unwind;
    assert!(catch_unwind(|| Fixe(i64::MAX) + Fixe(1)).is_err(), "addition");
    assert!(catch_unwind(|| Fixe(i64::MIN) - Fixe(1)).is_err(), "soustraction");
    assert!(catch_unwind(|| -Fixe(i64::MIN)).is_err(), "opposé");
    assert!(catch_unwind(|| Fixe::entier(65536) * Fixe::entier(65536)).is_err(), "produit 2^32");
    assert!(catch_unwind(|| Fixe::entier(i32::MAX) * Fixe::entier(2)).is_err(), "produit hors intervalle");
    assert!(catch_unwind(|| Fixe::rapport(1, 0)).is_err(), "rapport par zéro");
    assert!(catch_unwind(|| Fixe::rapport(1 << 31, 1)).is_err(), "rapport 2^31");
    assert!(catch_unwind(|| Fixe::rapport(-(1 << 31) - 1, 1)).is_err(), "rapport -2^31 - 1");
    assert!(catch_unwind(|| Fixe(1).div_entier(0)).is_err(), "division par zéro");
    assert!(catch_unwind(|| Fixe(i64::MIN).div_entier(-1)).is_err(), "i64::MIN / -1");
    for x in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY, 2_147_483_648.0, -2_147_483_648.0, 1e300] {
        assert!(catch_unwind(|| Fixe::depuis_f64(x)).is_err(), "depuis_f64({x:e})");
    }
    assert_eq!(Fixe::rapport(-(1 << 31), 1), Fixe(i64::MIN));
    assert_eq!(Fixe(i64::MAX) + Fixe(0), Fixe(i64::MAX));
    assert_eq!(Fixe::depuis_f64(-2_147_483_647.5), Fixe(-9_223_372_034_707_292_160));
}

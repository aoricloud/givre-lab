//! Étape 0 : `math::EXP_NEG`, la table de `e^(-k/16)` en virgule fixe Q0.32 (la table d'Arrhenius de l'étape 4). Valeurs
//! attendues : `references/etape-0-valeurs.md`, identifiants `V-EXP-xx`.
//!
//! Source : `round(2^32 · e^(-k/16))` calculé deux fois, indépendamment du code Rust : avec le module `decimal` de Python
//! (80 chiffres, fonction `exp`), puis avec le module `fractions` (série de Taylor de `e^(-1/16)` à 60 termes en rationnels
//! exacts, puissances exactes, arrondi exact) ; les deux calculs donnent les 366 mêmes entiers. Aucune valeur n'est un demi
//! (la plus proche d'un demi en est à `3,75e-4` unités) : l'arrondi est sans ambiguïté.

use givre_lab::math::{EXP_NEG, exp_neg};

/// `ATTENDU[k] = round(2^32 · e^(-k/16))` pour `k` de 0 à 365 (V-EXP-01). `ATTENDU[366]` serait 0.
const ATTENDU: [u64; 366] = [
    4294967296, 4034748382, 3790295335, 3560652950, 3344923893, 3142265200, 2951884975, 2773039306, 2605029347,
    2447198598, 2298930330, 2159645183, 2028798896, 1905880184, 1790408741, 1681933360, 1580030169, 1484300980,
    1394371730, 1309891017, 1230528733, 1155974766, 1085937795, 1020144150, 958336741, 900274053, 845729205, 794489063,
    746353404, 701134137, 658654567, 618748704, 581260615, 546043815, 512960693, 481881975, 452686223, 425259351,
    399494189, 375290060, 352552385, 331192315, 311126387, 292276193, 274568073, 257932835, 242305474, 227624928,
    213833830, 200878294, 188707693, 177274472, 166533955, 156444172, 146965699, 138061498, 129696774, 121838844,
    114457002, 107522402, 101007949, 94888187, 89139202, 83738531, 78665070, 73898994, 69421681, 65215634, 61264418,
    57552595, 54065659, 50789987, 47712777, 44822006, 42106378, 39555281, 37158748, 34907413, 32792480, 30805684,
    28939262, 27185921, 25538809, 23991491, 22537920, 21172416, 19889645, 18684592, 17552550, 16489094, 15490071,
    14551575, 13669939, 12841720, 12063679, 11332778, 10646160, 10001141, 9395203, 8825976, 8291237, 7788897, 7316991,
    6873677, 6457222, 6065999, 5698479, 5353225, 5028890, 4724205, 4437980, 4169096, 3916503, 3679214, 3456302,
    3246895, 3050176, 2865375, 2691771, 2528685, 2375479, 2231556, 2096353, 1969341, 1850025, 1737938, 1632641,
    1533725, 1440801, 1353507, 1271502, 1194466, 1122097, 1054113, 990247, 930251, 873890, 820944, 771205, 724480,
    680586, 639352, 600615, 564226, 530041, 497928, 467760, 439419, 412796, 387786, 364292, 342220, 321486, 302008,
    283711, 266521, 250374, 235204, 220954, 207567, 194991, 183177, 172079, 161653, 151859, 142659, 134015, 125896,
    118268, 111103, 104371, 98048, 92107, 86527, 81284, 76360, 71733, 67387, 63304, 59469, 55866, 52481, 49301, 46314,
    43508, 40872, 38396, 36070, 33884, 31831, 29903, 28091, 26389, 24790, 23288, 21877, 20552, 19307, 18137, 17038,
    16006, 15036, 14125, 13269, 12465, 11710, 11001, 10334, 9708, 9120, 8567, 8048, 7561, 7103, 6672, 6268, 5888, 5531,
    5196, 4882, 4586, 4308, 4047, 3802, 3571, 3355, 3152, 2961, 2781, 2613, 2455, 2306, 2166, 2035, 1912, 1796, 1687,
    1585, 1489, 1399, 1314, 1234, 1159, 1089, 1023, 961, 903, 848, 797, 749, 703, 661, 621, 583, 548, 515, 483, 454,
    427, 401, 376, 354, 332, 312, 293, 275, 259, 243, 228, 214, 201, 189, 178, 167, 157, 147, 138, 130, 122, 115, 108,
    101, 95, 89, 84, 79, 74, 70, 65, 61, 58, 54, 51, 48, 45, 42, 40, 37, 35, 33, 31, 29, 27, 26, 24, 23, 21, 20, 19,
    18, 17, 16, 15, 14, 13, 12, 11, 11, 10, 9, 9, 8, 8, 7, 7, 6, 6, 6, 5, 5, 5, 4, 4, 4, 4, 3, 3, 3, 3, 3, 3, 2, 2, 2,
    2, 2, 2, 2, 2, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
];

/// juste : la table `EXP_NEG` est égale, entrée par entrée, aux 366 valeurs écrites de V-EXP-01 : `EXP_NEG[0] = 2^32 =
/// 4294967296`, `EXP_NEG[16] = round(2^32/e) = 1580030169`, `EXP_NEG[365] = 1` (la dernière non nulle).
///
/// Ne couvre pas : la méthode de calcul de la table (une `const fn` en entiers, ou des valeurs en dur) ; `e^(-k/16)` pour `k`
/// non entier.
#[test]
#[cfg_attr(not(feature = "etape-0"), ignore)]
fn table_egale_aux_valeurs_ecrites() {
    assert_eq!(EXP_NEG.len(), 366);
    assert_eq!(EXP_NEG[0], 4_294_967_296);
    assert_eq!(EXP_NEG[16], 1_580_030_169);
    assert_eq!(EXP_NEG[365], 1);
    for (k, attendu) in ATTENDU.into_iter().enumerate() {
        assert_eq!(EXP_NEG[k], attendu, "EXP_NEG[{k}]");
    }
}

/// juste : `exp_neg(k)` lit la table pour `k < 366` et rend 0 au-delà, sans paniquer : `exp_neg(365) = 1`, `exp_neg(366) = 0`,
/// `exp_neg(367) = 0`, `exp_neg(1_000_000) = 0`, `exp_neg(usize::MAX) = 0` (V-EXP-01, la doc : « 0 au-delà de la table »).
///
/// Ne couvre pas : les indices négatifs (le type est `usize`).
#[test]
#[cfg_attr(not(feature = "etape-0"), ignore)]
fn exp_neg_lit_la_table_puis_zero() {
    for (k, attendu) in ATTENDU.into_iter().enumerate() {
        assert_eq!(exp_neg(k), attendu, "exp_neg({k})");
    }
    assert_eq!(exp_neg(365), 1);
    for k in [366, 367, 400, 1_000_000, usize::MAX] {
        assert_eq!(exp_neg(k), 0, "exp_neg({k})");
    }
}

/// juste : la table est décroissante (`EXP_NEG[k + 1] <= EXP_NEG[k]`), strictement tant que la valeur dépasse 2^20, et
/// l'indice de seize en seize divise par `e` à l'arrondi près : `|EXP_NEG[k + 16] · e - EXP_NEG[k]| <= e/2 + 1/2 + 1e-5`
/// en unités du dernier chiffre, pour `k` de 0 à 349. Raison de la tolérance : les deux entrées sont chacune à une demi-unité
/// au plus de leur valeur exacte, qui vérifie `v(k+16) · e = v(k)` ; ce qui donne `e/2 + 1/2` ; les `1e-5` couvrent l'erreur de
/// `e` écrit en `f64` et du produit en flottant (au plus `1e-6` sur des valeurs de 4,3e9) (V-EXP-02, calcul).
///
/// Ne couvre pas : l'exactitude de chaque entrée (le test `table_egale_aux_valeurs_ecrites` s'en charge). Deux ancrages
/// (`EXP_NEG[0] = 2^32`, `EXP_NEG[365] = 1`) empêchent qu'une table de zéros passe ces propriétés.
#[test]
#[cfg_attr(not(feature = "etape-0"), ignore)]
fn table_decroissante_et_rapport_e() {
    // ancrage : une table de zéros est décroissante et vérifie le rapport à `e` ; elle ne passe pas ces deux valeurs
    assert_eq!(EXP_NEG[0], 4_294_967_296);
    assert_eq!(EXP_NEG[365], 1);
    for k in 0..365 {
        assert!(EXP_NEG[k + 1] <= EXP_NEG[k], "décroissance en {k}");
        if EXP_NEG[k] > 1 << 20 {
            assert!(EXP_NEG[k + 1] < EXP_NEG[k], "décroissance stricte en {k}");
        }
    }
    let e = std::f64::consts::E;
    for k in 0..=349 {
        let ecart = (EXP_NEG[k + 16] as f64 * e - EXP_NEG[k] as f64).abs();
        assert!(ecart <= e / 2.0 + 0.5 + 1e-5, "rapport e en {k} : écart {ecart}");
    }
}

//! Les fonctions écrites dans le dépôt (étape 0) : une racine entière, un logarithme, la table d'Arrhenius et la
//! virgule fixe. Rien ici n'appelle la bibliothèque mathématique du système (`ln`, `exp`, `powf`… sont interdits dans
//! `src/`, voir `AGENTS.md`) : le résultat est le même sous Linux et sous Windows, au bit près.
//!
//! - [`racine_entiere`] : la partie entière exacte de la racine carrée d'un `u64`.
//! - [`logarithme`] : le logarithme népérien, écrit avec `+ - * /`, `to_bits` et `from_bits`.
//! - [`EXP_NEG`] et [`exp_neg`] : la table de `e^(-k/16)` en virgule fixe Q0.32 (la table d'Arrhenius de l'étape 4 :
//!   une probabilité d'attachement `exp(-n·J/kT)` se lit à l'indice entier correspondant).
//! - [`Fixe`] : la virgule fixe Q31.32 sur `i64`, pour le modèle de Reiter de l'étape 3 (moyennes sur six voisins).

/// La partie entière exacte de la racine carrée de `n` : le plus grand `r` tel que `r * r <= n`. Exacte pour tout
/// `u64`, y compris près de `u64::MAX` (`racine_entiere(u64::MAX) = 4294967295`) et sur les carrés parfaits
/// (`racine_entiere(k * k) = k`, `racine_entiere(k * k - 1) = k - 1`). Aucun flottant n'est nécessaire.
pub fn racine_entiere(n: u64) -> u64 {
    // Recherche binaire sur [bas, haut[ avec `haut` = 2^32 : le carré se compare en `u128`, sans débordement.
    let mut bas = 0_u64;
    let mut haut = 1_u64 << 32;
    while bas + 1 < haut {
        let milieu = bas + (haut - bas) / 2;
        if u128::from(milieu) * u128::from(milieu) <= u128::from(n) {
            bas = milieu;
        } else {
            haut = milieu;
        }
    }
    bas
}

/// Le logarithme népérien de `x`, écrit avec `+ - * /`, `to_bits` et `from_bits` seulement.
///
/// **La méthode.** On écrit `x = m · 2^e` avec la mantisse `m` recentrée dans `[√2/2, √2]` (on prend d'abord `m` dans
/// `[1, 2[` avec les bits de `x`, puis on divise `m` par 2 et on ajoute 1 à `e` si `m > √2`). Alors
/// `ln x = e · ln 2 + ln m`, avec `ln 2` coupé en deux flottants et
/// `ln m = 2 · atanh(t) = 2 · (t + t³/3 + t⁵/5 + … + t²³/23)` pour `t = (m - 1) / (m + 1)` (`|t| <= 0,1716`).
///
/// **Les constantes** (calculées avec 80 chiffres décimaux ; le test n'en impose aucune, seulement la précision) :
/// `LN2_HI = f64::from_bits(0x3FE62E42FEFA4000)` (0,6931471805601177 ; 40 bits de mantisse seulement, donc `e · LN2_HI`
/// est exact pour tout exposant `e`) et `LN2_LO = f64::from_bits(0xBD48432A1B0E2634)` (-1,7239444525614835e-13), avec
/// `LN2_HI + LN2_LO = ln 2` à 3e-31 près en relatif ; le résultat se somme dans cet ordre : `(e · LN2_HI + ln m) + e · LN2_LO`.
/// `√2` s'écrit `std::f64::consts::SQRT_2` ; `2^54` vaut `18014398509481984.0`. Une série de 11 termes (jusqu'à `t²³/23`)
/// suffit : le suivant pèse moins de `2e-20` de `t`.
///
/// **Le domaine.**
/// - `x` strictement positif et fini : le logarithme, avec la précision annoncée plus bas ;
/// - `x = 1` : exactement 0 ;
/// - `x <= 0` (y compris `-0.0` et `-∞`) et `x` NaN : NaN (même `logarithme(0.0)`, que la mathématique voudrait
///   `-∞`) ;
/// - `x = +∞` : `+∞` ;
/// - les **sous-normaux sont traités** (non refusés) : on les multiplie par 2^54, ce qui est exact, et on retranche 54
///   à l'exposant ; `logarithme(5e-324)` est donc l'ordre de `-744,44`.
///
/// **La précision annoncée.** Pour tout `x` strictement positif et fini, `|logarithme(x) - ln(x)| <= 5e-16 · |ln(x)|`
/// (erreur relative d'au plus `5e-16`, soit un peu plus de deux unités du dernier chiffre : le résultat n'est pas toujours
/// l'arrondi correct). Le prototype mesure 3,0e-16 contre le logarithme exact (calculé en 50 chiffres décimaux) et 3,2e-16
/// contre le `f64::ln` du système (permis dans un test seulement), sur 13 402 points : des exposants et des mantisses au
/// hasard, des sous-normaux, toutes les puissances de 2, l'entourage de 1, de `√2` et de `√2/2`, une grille de 1/1000 à
/// 1000. Le pire cas est juste sous `√2/2` (là, `e = -1` et `e · ln 2 + ln m` perd un bit à la soustraction).
pub fn logarithme(x: f64) -> f64 {
    // Domaine : NaN pour tout ce qui n'est pas strictement positif (y compris -0,0 et -∞), +∞ pour +∞.
    if x.is_nan() || x <= 0.0 {
        return f64::NAN;
    }
    if x == f64::INFINITY {
        return f64::INFINITY;
    }
    if x == 1.0 {
        return 0.0;
    }

    const MANTISSE: u64 = 0x000F_FFFF_FFFF_FFFF;
    const BIT_CACHE: u64 = 0x3FF0_0000_0000_0000;
    const DEUX_54: f64 = 18_014_398_509_481_984.0;

    // x = m · 2^e avec m dans [1, 2[ : les sous-normaux sont d'abord multipliés par 2^54 (exact).
    let bits = x.to_bits();
    let exposant_brut = (bits >> 52) & 0x7FF;
    let (mut m, mut e) = if exposant_brut == 0 {
        let y = x * DEUX_54;
        let bits_y = y.to_bits();
        let e = ((bits_y >> 52) & 0x7FF) as i64 - 1023 - 54;
        (f64::from_bits((bits_y & MANTISSE) | BIT_CACHE), e)
    } else {
        let e = exposant_brut as i64 - 1023;
        (f64::from_bits((bits & MANTISSE) | BIT_CACHE), e)
    };

    // Recentrage de la mantisse dans [√2/2, √2].
    if m > std::f64::consts::SQRT_2 {
        m /= 2.0;
        e += 1;
    }

    // ln m = 2 · atanh(t) = 2 · (t + t³/3 + … + t²³/23), évalué par Horner en t² (ordre fixe).
    let t = (m - 1.0) / (m + 1.0);
    let t2 = t * t;
    let mut somme = 1.0 / 23.0;
    for denominateur in (1..23).step_by(2).rev() {
        somme = somme * t2 + 1.0 / denominateur as f64;
    }
    let ln_m = 2.0 * t * somme;

    const LN2_HI: f64 = f64::from_bits(0x3FE6_2E42_FEFA_4000);
    const LN2_LO: f64 = f64::from_bits(0xBD48_432A_1B0E_2634);
    let e_f = e as f64;
    (e_f * LN2_HI + ln_m) + e_f * LN2_LO
}

/// La table de `e^(-k/16)` en virgule fixe Q0.32 : `EXP_NEG[k] = round(2^32 · e^(-k/16))` pour `k = 0, 1, …, 365`,
/// arrondi au plus proche (aucune valeur n'est un demi : l'arrondi est sans ambiguïté), en `u64` (car `EXP_NEG[0]` vaut
/// `2^32 = 4294967296`, qui ne tient pas sur 32 bits). C'est une **fonction décroissante** de `k` ; la dernière
/// entrée, `EXP_NEG[365] = 1`, est la dernière non nulle : `round(2^32 · e^(-366/16)) = 0`, et il en va de même
/// au-delà. La table a donc exactement 366 entrées.
///
/// Une probabilité d'attachement `exp(-n·J/kT)` se lit à l'indice entier `k = 16 · n · J / kT` quand celui-ci est
/// entier : c'est la table d'Arrhenius de l'étape 4. Les valeurs peuvent s'écrire en dur ou se calculer en entiers (une
/// `const fn` en `u128`) ; elles ne peuvent pas passer par `exp` du système.
pub static EXP_NEG: [u64; 366] = [
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

/// `e^(-k/16)` en virgule fixe Q0.32 : `EXP_NEG[k]` si `k < 366`, et **0** au-delà de la table (aucun indice ne
/// panique, `usize::MAX` compris).
pub fn exp_neg(k: usize) -> u64 {
    if k < EXP_NEG.len() { EXP_NEG[k] } else { 0 }
}

/// Un nombre en virgule fixe **Q31.32** sur `i64` : la valeur est `brut / 2^32`, où `brut` est le champ public. Il
/// couvre `[-2^31, 2^31[` au pas de `2^-32`. Tous les arrondis ci-dessous sont **au plus proche, les demi-cas vers
/// +∞** (c'est-à-dire `floor(valeur exacte + 1/2)` en unités du dernier chiffre) ; toutes les opérations sont exactes
/// en entiers, donc identiques sur toutes les machines. Un résultat qui sort de l'intervalle fait **paniquer** (jamais
/// de rebouclage silencieux).
///
/// Les opérateurs `+`, `-` (binaire et unaire) et `*` sont définis : `+` et `-` additionnent et soustraient les
/// champs bruts (exacts), `*` est le produit arrondi décrit à l'implémentation de [`std::ops::Mul`]. L'ordre est celui
/// des champs bruts (donc celui des valeurs). `Fixe::default()` est zéro (champ brut 0) ; le type est `Copy`, `Debug`,
/// `Eq` et `Ord`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
#[allow(dead_code)]
pub struct Fixe(pub i64);

/// Division entière arrondie vers `-∞` (la division de Rust tronque vers zéro) : `floor(a / b)`, `b != 0`.
fn division_plancher(a: i128, b: i128) -> i128 {
    let quotient = a / b;
    let reste = a % b;
    if reste != 0 && ((reste < 0) != (b < 0)) { quotient - 1 } else { quotient }
}

impl Fixe {
    /// Zéro (`brut = 0`).
    pub const ZERO: Fixe = Fixe(0);

    /// Un (`brut = 2^32 = 4294967296`).
    pub const UN: Fixe = Fixe(1 << 32);

    /// L'entier `n` : `brut = n · 2^32` (exact, sans panique : tout `i32` tient).
    #[allow(unused_variables)]
    pub fn entier(n: i32) -> Fixe {
        Fixe(i64::from(n) * (1_i64 << 32))
    }

    /// Le rapport `num / den` arrondi : `brut = floor((num · 2^32) / den + 1/2)`, calculé en `i128` (demi-cas vers
    /// +∞ : `rapport(-1, 2^33)` vaut 0 et `rapport(1, 2^33)` vaut `brut = 1`). Le signe de `den` est sans importance :
    /// `rapport(a, -b) = rapport(-a, b)`. Exemples : `rapport(1, 3)` a pour `brut` 1431655765 ; `rapport(2, 3)` a pour
    /// `brut` 2863311531 ; `rapport(-1, 3)` a pour `brut` -1431655765.
    ///
    /// # Panique
    ///
    /// Si `den` vaut 0, ou si le résultat sort de `[-2^31, 2^31[`.
    #[allow(unused_variables)]
    pub fn rapport(num: i64, den: i64) -> Fixe {
        assert!(den != 0, "rapport par zéro");
        // floor(N/D + 1/2) = floor((2N + D) / (2D)) ; tout tient en i128.
        let numerateur = i128::from(num) * (1_i128 << 32);
        let denominateur = i128::from(den);
        let brut = division_plancher(2 * numerateur + denominateur, 2 * denominateur);
        Fixe(i64::try_from(brut).expect("rapport hors de l'intervalle"))
    }

    /// Le quotient par un entier : `brut = floor(self.brut / n + 1/2)` (au plus proche, demi-cas vers +∞), calculé en
    /// `i128`. Le signe de `n` est pris en compte : diviser `a` par `-n` donne le même résultat que diviser `-a` par `n`
    /// (c'est le même quotient). Demi-cas : `brut = -5` divisé par 2 donne `-2` (`-2,5` arrondi vers +∞), `brut = -7`
    /// divisé par 2 donne `-3`, `brut = 5` divisé par 2 donne `3`.
    /// Exemple : `entier(1).div_entier(3)` a pour `brut` 1431655765. C'est ce qui sert à la moyenne de six voisins.
    ///
    /// # Panique
    ///
    /// Si `n` vaut 0, ou si le résultat sort de l'intervalle (seul cas : `Fixe(i64::MIN)` divisé par `-1`).
    #[allow(unused_variables)]
    pub fn div_entier(self, n: i64) -> Fixe {
        assert!(n != 0, "division par zéro");
        // floor(b/n + 1/2) = floor((2b + n) / (2n)) ; tout tient en i128.
        let brut = division_plancher(2 * i128::from(self.0) + i128::from(n), 2 * i128::from(n));
        Fixe(i64::try_from(brut).expect("quotient hors de l'intervalle"))
    }

    /// Convertit un flottant : `brut = floor(x · 2^32 + 1/2)` (au plus proche, demi-cas vers +∞ ; le produit par
    /// `2^32` est exact en flottant, et le résultat est calculé sans erreur d'arrondi : `(x * 2^32 + 0.5).floor()` ne
    /// convient pas, car l'addition de `0.5` arrondit dès que `|x · 2^32| >= 2^52` ; on prend par exemple la partie
    /// entière `p = y.floor()` de `y = x · 2^32`, puis `p + 1` si `y - p >= 0.5`, ce qui est exact). `depuis_f64(0.5)` est
    /// `brut = 2^31` ; `depuis_f64(-0.5 / 2^32)` vaut 0 et `depuis_f64(0.5 / 2^32)` vaut `brut = 1`.
    ///
    /// # Panique
    ///
    /// Si `x` est NaN, infini ou de valeur absolue au moins `2^31`.
    #[allow(unused_variables)]
    pub fn depuis_f64(x: f64) -> Fixe {
        assert!(x.is_finite(), "flottant non fini");
        assert!(x.abs() < 2_147_483_648.0, "flottant hors de l'intervalle");
        // y = x · 2^32 est exact ; `y - p` l'est aussi, alors que `y + 0.5` arrondirait au-delà de 2^52.
        let y = x * 4_294_967_296.0;
        let p = y.floor();
        let p = if y - p >= 0.5 { p + 1.0 } else { p };
        Fixe(p as i64)
    }

    /// Convertit en flottant : `brut as f64 · 2^-32`, exact tant que `|brut| <= 2^53` (au-delà, `brut as f64` arrondit
    /// au plus proche, au pair en cas d'égalité).
    pub fn vers_f64(self) -> f64 {
        self.0 as f64 / 4_294_967_296.0
    }
}

impl std::ops::Add for Fixe {
    type Output = Fixe;

    /// La somme des champs bruts (exacte). Panique au débordement de l'`i64`.
    #[allow(unused_variables)]
    fn add(self, autre: Fixe) -> Fixe {
        Fixe(self.0.checked_add(autre.0).expect("débordement de l'addition"))
    }
}

impl std::ops::Sub for Fixe {
    type Output = Fixe;

    /// La différence des champs bruts (exacte). Panique au débordement de l'`i64`.
    #[allow(unused_variables)]
    fn sub(self, autre: Fixe) -> Fixe {
        Fixe(self.0.checked_sub(autre.0).expect("débordement de la soustraction"))
    }
}

impl std::ops::Neg for Fixe {
    type Output = Fixe;

    /// L'opposé (exact). Panique pour `Fixe(i64::MIN)`.
    fn neg(self) -> Fixe {
        Fixe(self.0.checked_neg().expect("débordement de l'opposé"))
    }
}

impl std::ops::Mul for Fixe {
    type Output = Fixe;

    /// Le produit arrondi : `brut = (a.brut · b.brut + 2^31) >> 32`, calculé en `i128` (le décalage est arithmétique :
    /// c'est `floor(a·b / 2^32 + 1/2)`, au plus proche, demi-cas vers +∞). Exemples : `UN * UN = UN` ; `Fixe(2^31) *
    /// Fixe(1)` vaut `brut = 1` (un demi arrondi vers le haut) et `Fixe(-2^31) * Fixe(1)` vaut 0.
    ///
    /// # Panique
    ///
    /// Si le résultat sort de `[-2^31, 2^31[`.
    #[allow(unused_variables)]
    fn mul(self, autre: Fixe) -> Fixe {
        let produit = i128::from(self.0) * i128::from(autre.0);
        let brut = (produit + (1_i128 << 31)) >> 32;
        Fixe(i64::try_from(brut).expect("produit hors de l'intervalle"))
    }
}

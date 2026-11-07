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
#[allow(unused_variables)]
pub fn racine_entiere(n: u64) -> u64 {
    todo!()
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
#[allow(unused_variables)]
pub fn logarithme(x: f64) -> f64 {
    todo!()
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
pub static EXP_NEG: [u64; 366] = [0; 366];

/// `e^(-k/16)` en virgule fixe Q0.32 : `EXP_NEG[k]` si `k < 366`, et **0** au-delà de la table (aucun indice ne
/// panique, `usize::MAX` compris).
#[allow(unused_variables)]
pub fn exp_neg(k: usize) -> u64 {
    todo!()
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

impl Fixe {
    /// Zéro (`brut = 0`).
    pub const ZERO: Fixe = Fixe(0);

    /// Un (`brut = 2^32 = 4294967296`).
    pub const UN: Fixe = Fixe(1 << 32);

    /// L'entier `n` : `brut = n · 2^32` (exact, sans panique : tout `i32` tient).
    #[allow(unused_variables)]
    pub fn entier(n: i32) -> Fixe {
        todo!()
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
        todo!()
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
        todo!()
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
        todo!()
    }

    /// Convertit en flottant : `brut as f64 · 2^-32`, exact tant que `|brut| <= 2^53` (au-delà, `brut as f64` arrondit
    /// au plus proche, au pair en cas d'égalité).
    pub fn vers_f64(self) -> f64 {
        todo!()
    }
}

impl std::ops::Add for Fixe {
    type Output = Fixe;

    /// La somme des champs bruts (exacte). Panique au débordement de l'`i64`.
    #[allow(unused_variables)]
    fn add(self, autre: Fixe) -> Fixe {
        todo!()
    }
}

impl std::ops::Sub for Fixe {
    type Output = Fixe;

    /// La différence des champs bruts (exacte). Panique au débordement de l'`i64`.
    #[allow(unused_variables)]
    fn sub(self, autre: Fixe) -> Fixe {
        todo!()
    }
}

impl std::ops::Neg for Fixe {
    type Output = Fixe;

    /// L'opposé (exact). Panique pour `Fixe(i64::MIN)`.
    fn neg(self) -> Fixe {
        todo!()
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
        todo!()
    }
}

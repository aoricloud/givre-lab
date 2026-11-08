//! La croissance de surface en bande : dépôt aléatoire, dépôt avec relaxation, dépôt balistique, Eden (étape 2, moitié « surfaces »).
//!
//! Quatre modèles font croître, sur une bande de `L` colonnes périodique en `x` (la colonne `L - 1` touche la colonne 0), un
//! front dont on mesure la rugosité `w(t)` ; chacun a son exposant de croissance `β`, défini par `w(t) ~ t^β` (Family et
//! Vicsek 1985 : `w(L, t) ~ L^α f(t / L^z)` avec `z = α / β`). Tout est en entiers, sauf la rugosité (un rapport d'entiers
//! puis `sqrt`) et l'exposant (une pente) : le résultat est le même sous Linux et sous Windows, au bit près.
//!
//! # Le temps, les hauteurs, la rugosité
//!
//! Une surface a `L` hauteurs entières `h[x]`, `x` dans `0..L`, toutes nulles au départ. Les voisins de `x` sont `x - 1` et
//! `x + 1` **modulo `L`** (`L = 1` : les deux voisins de la colonne 0 sont la colonne 0 ; `L = 2` : les deux voisins de `x`
//! sont la même autre colonne). Un **pas** dépose une particule ; le **temps** `t` compte les couches : `t = particules / L`
//! (division entière), de sorte qu'après `deposer_couche()` répété `t` fois il y a exactement `t · L` particules. La
//! rugosité est `w² = (1/L) Σ (h[x] - moyenne)²`, qu'on calcule **exactement** avec `S1 = Σ h[x]` et `S2 = Σ h[x]²` (en
//! `u128`) : `w² = (L · S2 - S1²) / L²`, le numérateur et le dénominateur étant des entiers ([`rugosite_exacte`] rend
//! `(L · S2 - S1², L²)`, sans simplifier). Puis `w² = numérateur as f64 / dénominateur as f64` (une conversion puis une
//! division, arrondies au plus proche) et `w = w².sqrt()` ([`rugosite_carre_de`], [`rugosite_de`]).
//!
//! # Les quatre modèles et le hasard, appel par appel
//!
//! Le générateur est `Alea::new(graine)` (flux 54, voir [`crate::alea`]). **Un pas** commence par un tirage, puis applique la
//! règle ; le tirage et les éventuels tirages de la règle sont, dans cet ordre, les seuls appels au générateur.
//!
//! 1. **Dépôt aléatoire** ([`Modele::Depot`]). Tirage : `i = borne(L)`. Règle : `h[i] += 1`. Aucune corrélation entre les
//!    colonnes : `β = 1/2`. La somme `S1` vaut le nombre de particules. Solution exacte (loi multinomiale de `N = t·L`
//!    particules sur `L` colonnes équiprobables) : `E[w²] = t (1 - 1/L)` ; voir [`variance_rugosite_carre_depot`] pour la
//!    variance (`Var(w²) = 2 n (n - 1) (L - 1) / L⁴`).
//! 2. **Dépôt avec relaxation de surface** ([`Modele::Relaxation`], modèle de Family 1986). Tirage : `i = borne(L)`. Soient
//!    `hi = h[i]`, `hg = h[i - 1]` et `hd = h[i + 1]` (voisins modulo `L`). La particule va dans la colonne `c` ainsi :
//!    - si `hi <= hg` **et** `hi <= hd` : `c = i` (une égalité avec un voisin laisse la particule en `i` ; aucun tirage) ;
//!    - sinon, si `hg < hd` : `c = i - 1` ; sinon, si `hd < hg` : `c = i + 1` (aucun tirage) ;
//!    - sinon (`hg == hd`, et ils sont alors strictement inférieurs à `hi`) : un tirage `pile = suivant() >> 31` (0 ou 1) ;
//!      `c = i - 1` si `pile == 0`, `c = i + 1` si `pile == 1`. Ce tirage n'a lieu **que** dans ce cas, **même si** `L = 2`
//!      (où `i - 1` et `i + 1` sont la même colonne).
//!
//!    puis `h[c] += 1`. Classe d'Edwards-Wilkinson : `β = 1/4`. **Convention** : la particule ne quitte `i` que si un voisin est
//!    *strictement* plus bas ; sur une surface plate elle reste donc en `i`. (Les sources secondaires lues divergent sur ce cas
//!    d'égalité : voir `references/etape-2s-valeurs.md`, V-REF-02. Cette convention est celle de ce dépôt ; l'autre lecture
//!    est une autre dynamique.)
//! 3. **Dépôt balistique** ([`Modele::Balistique`]). Tirage : `i = borne(L)`. Règle : `h[i] = max(h[i] + 1, h[i - 1],
//!    h[i + 1])` (voisins modulo `L`, les hauteurs lues **avant** la mise à jour, qui ne change que `h[i]`). Classe KPZ :
//!    `β = 1/3`. Les hauteurs ne décroissent jamais.
//! 4. **Eden** ([`Modele::Eden`]). Une grille d'occupation de `L` colonnes et de [`HAUTEUR_EDEN`] rangées `y = 0, 1, …`
//!    (`y` croît vers le haut), au-dessus d'un substrat plein (la rangée `-1`, jamais représentée). Un site est *occupé*, ou
//!    *dans le périmètre* (vide, mais touchant par un des quatre côtés un site occupé ou le substrat), ou *vide* (ni l'un ni
//!    l'autre). Le **périmètre** est un `Vec<(x, y)>` ordonné. Au départ : les `L` sites de la rangée 0, dans l'ordre
//!    `x = 0..L`. Tirage : `k = borne(len(périmètre))`. Règle : le site `s = (x, y) = périmètre[k]` s'occupe et sort du `Vec`
//!    par **`swap_remove(k)`** (le dernier élément prend la place `k`) ; puis, pour chaque voisin de `s` dans cet ordre :
//!    `(x + 1, y)`, `(x, y + 1)`, `(x - 1, y)`, `(x, y - 1)` (`x` modulo `L`), **ignoré** si `y' < 0` (le substrat), **panique**
//!    si `y' >= HAUTEUR_EDEN`, sinon s'il est vide (ni occupé ni déjà dans le périmètre) il est **ajouté en fin** du `Vec`.
//!    La hauteur de la colonne `x` est `0` si elle est vide, sinon `1 +` la plus haute rangée occupée de la colonne (le profil
//!    du dessus : des trous ou des surplombs en dessous ne comptent pas). Classe KPZ : `β = 1/3`. Chaque pas occupe exactement
//!    un site (le nombre de sites occupés vaut le nombre de particules) ; pour Eden `S1` est la somme des hauteurs du profil,
//!    donc au moins le nombre de particules (égale s'il n'y a ni trou ni surplomb).
//!
//! La panique d'Eden (`y' >= HAUTEUR_EDEN`) a lieu au pas qui occupe un site de la rangée `HAUTEUR_EDEN - 1`, quand il teste
//! son voisin du dessus : avec `L = 1` la colonne se remplit d'une particule par pas, et c'est le pas numéro
//! `HAUTEUR_EDEN` qui panique, le précédent non. Après une panique la surface ne doit plus servir.
//!
//! # L'écriture canonique de l'état ([`Surface::octets`]) et l'empreinte
//!
//! L'empreinte est le SHA-256 ([`crate::empreinte::sha256_hex`]) de cette suite d'octets, dans cet ordre :
//!
//! ```text
//! 8 octets    l'étiquette ASCII "GIVRE2S0"  (47 49 56 52 45 32 53 30)
//! 1 octet     le modèle : 0 dépôt, 1 relaxation, 2 balistique, 3 Eden
//! 4 octets    la largeur L, u32 petit-boutiste
//! 8 octets    le nombre de particules, u64 petit-boutiste
//! 8·L octets  les hauteurs h[0], h[1], …, h[L-1], chacune u64 petit-boutiste
//! 16 octets   Alea::octets() du générateur, dans son état courant (après tous les tirages faits)
//! ```
//!
//! et, pour Eden seulement, à la suite :
//!
//! ```text
//! 4 octets    n, le nombre de sites du périmètre, u32 petit-boutiste
//! 8·n octets  les sites du périmètre dans l'ordre du Vec : x puis y, chacun u32 petit-boutiste
//! 4 octets    r, la plus grande hauteur (max des h[x]), u32 petit-boutiste
//! r·L octets  l'occupation : pour y = 0..r, pour x = 0..L, l'octet 1 si (x, y) est occupé, 0 sinon
//! ```
//!
//! # Le scénario : la directive `surface` et ses sorties
//!
//! [`crate::lancer::lancer`] comprend la directive `surface <modele> <largeur> <couches>` : `modele` est l'un des noms
//! `depot`, `relaxation`, `balistique`, `eden` ([`Modele::nom`]) ; `largeur` (le nombre de colonnes) un entier de 1 à
//! [`LARGEUR_MAX`] ; `couches` un entier de 1 à [`COUCHES_MAX`] (entiers lus par [`crate::scenario::Directive::entier`] :
//! chiffres décimaux seulement). La forme (`boite` ou `hexagone`) que le format de fichier exige est **ignorée** : seule la
//! directive compte (l'exemple `scenarios/surface.txt` écrit la `boite` de même largeur pour qu'elle dise la même chose). Les
//! directives du scénario sont examinées dans l'ordre du fichier : la **première** doit être une `surface` bien formée, et il
//! ne doit y en avoir aucune autre ; sinon `ErreurLancer::DirectiveInconnue(ligne)`, avec la ligne de la première directive
//! fautive (la première directive si elle est mal formée ou n'est pas `surface`, sinon la seconde). Un nombre d'arguments
//! faux, un nom de modèle inconnu, un entier mal écrit ou hors bornes rendent aussi `DirectiveInconnue(ligne de la
//! directive)` : c'est la seule erreur de `lancer`.
//!
//! Avec la graine `g`, `lancer` fait `Surface::new(modele, largeur, g)` puis `couches` fois `deposer_couche()`. Ses sorties :
//!
//! - **`empreinte`** : [`Surface::empreinte`] de la surface finale (l'écriture canonique ci-dessus) ; la forme n'y entre pas.
//! - **`mesures`** : le CSV `couches,rugosite\n` puis une ligne par instant `t` : `t` en décimal sans zéro de tête, une
//!   virgule, la rugosité `w` de la surface après `t` couches complètes écrite par `format!("{:.6}", w)` (six décimales),
//!   puis `\n` (pas de `\r`). Les instants sont `1, 2, 4, 8, …` (les puissances de 2 au plus égales à `couches`), puis
//!   `couches` lui-même s'il n'est pas déjà une puissance de 2 ; chaque instant une fois, en ordre croissant.
//! - **`apercu`** : un PNG ([`crate::image::ecrire_png`]) de la surface finale vue de côté, de `largeur` pixels de large et
//!   `H = max(1, plus grande hauteur)` pixels de haut. Le pixel de colonne `x` et de ligne `y` (`y` compté depuis le haut de
//!   l'image) est celui de la rangée `r = H - 1 - y` de la colonne `x`. Si `r >= h[x]` (au-dessus du front) c'est le fond
//!   `(240, 240, 240)`. Sinon la rangée a été remplie pendant la couche `a`, de 1 à `couches` : la première couche complète
//!   après laquelle la hauteur de la colonne `x` dépasse `r` (`h_a[x] > r` et `h_(a-1)[x] <= r`, où `h_t` est le profil après
//!   `t` couches complètes et `h_0` est nul) ; sa couleur est `(v, 96, 255 - v)` avec
//!   `v = (a - 1) * 255 / max(couches - 1, 1)` (division entière sur des `u64`) : du bleu pour les premières couches à
//!   l'orange pour les dernières. Aucune rangée remplie n'a la couleur du fond. Pour Eden la rangée `r` est remplie si
//!   `r < h[x]` : le profil du dessus, les trous ne sont pas dessinés.
//!
//! # La mesure de l'exposant β
//!
//! [`exposant`] lit `β` comme la **pente des moindres carrés** de `ln w` contre `ln t`. Les instants `t` (des couches
//! complètes) sont donnés par l'appelant : c'est la **fenêtre**, qui doit finir avant la saturation (`t_sat ~ L^z`, soit
//! `L^2` pour la relaxation et `L^(3/2)` pour les modèles KPZ) et commencer après la transition depuis le dépôt aléatoire
//! (quelques couches). Pour chaque instant, `w²` est **moyenné sur les graines** (somme dans l'ordre des graines, divisée
//! par leur nombre), puis **une seule pente** est calculée sur la courbe moyenne : `ln` de la racine du `w²` moyen contre `ln t`
//! (`ln` est [`crate::math::logarithme`]). On ne moyenne pas les pentes de chaque graine : une `w` isolée est très bruitée, le
//! logarithme d'une moyenne est moins biaisé que la moyenne de logarithmes, et pour le dépôt aléatoire le `w²` moyen a une
//! solution exacte.
//!
//! La taille finie se sent bien avant `t_sat` : à `L = 1024`, la fenêtre `t = 16 … 512` des tests donne un exposant effectif
//! du balistique d'environ 0,28 (Farnudi et Vvedensky 2011 publient 0,280 à cette taille) et d'Eden d'environ 0,30, sous 1/3 ;
//! l'exposant effectif ne monte vers 1/3 que lentement avec `L`. Voir `references/etape-2s-valeurs.md`, V-EXP-03 et V-EXP-06.
//!
//! **Le coût.** Les tests de l'exposant font de l'ordre de 10⁸ pas (dont 8·10⁷ pas d'Eden à `L = 1024`) : un pas doit coûter
//! un temps constant. Pour Eden, l'état de chaque site (vide, périmètre, occupé) se lit dans une grille indexée par
//! `y · L + x` ; une recherche linéaire dans le périmètre, ou des `BTreeSet`/`BTreeMap` pour l'occupation et le périmètre, rendent
//! ces tests trop lents (plus d'une minute avec des `BTree`, bien davantage avec une recherche linéaire).
//!
//! # Ce que ce module ne fait pas
//!
//! Ni la dimension 2 + 1, ni les corrections de taille finie (l'exposant mesuré à `L` fini et sur une fenêtre finie n'est pas
//! l'exposant asymptotique : l'écart est mesuré dans `references/etape-2s-valeurs.md`), ni le calcul de `α`.

use crate::alea::Alea;

/// La plus grande largeur d'une surface (la même borne que `scenario::COTE_MAX`).
pub const LARGEUR_MAX: usize = 4096;

/// La capacité d'Eden : le nombre de rangées de la grille (`y` va de 0 à `HAUTEUR_EDEN - 1`). Un pas d'Eden qui tente
/// d'ajouter au périmètre un site de la rangée `HAUTEUR_EDEN` ou au-dessus panique (voir la documentation du module).
pub const HAUTEUR_EDEN: usize = 4096;

/// Le plus grand nombre de couches qu'une directive `surface` de `lancer` accepte. Avec `HAUTEUR_EDEN` ci-dessus, un Eden de
/// `COUCHES_MAX` couches tient dans la grille (le profil monte d'un peu plus d'une rangée par couche).
pub const COUCHES_MAX: u64 = 1024;

/// Les quatre modèles de croissance de surface (voir la documentation du module).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Modele {
    /// Le dépôt aléatoire : `h[i] += 1` (`β = 1/2`).
    Depot,
    /// Le dépôt avec relaxation de surface de Family (`β = 1/4`).
    Relaxation,
    /// Le dépôt balistique : `h[i] = max(h[i] + 1, h[i - 1], h[i + 1])` (`β = 1/3`).
    Balistique,
    /// Le modèle d'Eden sur le périmètre (`β = 1/3`).
    Eden,
}

impl Modele {
    /// Les quatre modèles, dans l'ordre `Depot`, `Relaxation`, `Balistique`, `Eden`.
    pub const TOUS: [Modele; 4] = [Modele::Depot, Modele::Relaxation, Modele::Balistique, Modele::Eden];

    /// Le nom écrit dans un scénario : `"depot"`, `"relaxation"`, `"balistique"`, `"eden"` (minuscules, sans accent).
    pub fn nom(self) -> &'static str {
        todo!()
    }

    /// Le modèle de ce nom (exactement l'un des quatre noms de [`Modele::nom`], même casse), sinon `None`.
    #[allow(unused_variables)]
    pub fn depuis_nom(nom: &str) -> Option<Modele> {
        todo!()
    }

    /// L'octet de l'écriture canonique : 0 dépôt, 1 relaxation, 2 balistique, 3 Eden.
    pub fn code(self) -> u8 {
        todo!()
    }
}

/// Une surface en croissance : un modèle, `L` hauteurs, un générateur (voir la documentation du module).
#[derive(Clone, Debug)]
#[allow(dead_code)]
pub struct Surface {
    modele: Modele,
    largeur: usize,
    alea: Alea,
    particules: u64,
    hauteurs: Vec<u64>,
    /// Eden : l'état de chaque site, rangée par rangée (`y * largeur + x`) : 0 vide, 1 périmètre, 2 occupé ; la grille
    /// grandit à la demande (les rangées absentes sont vides).
    cases: Vec<u8>,
    /// Eden : le périmètre, dans l'ordre du `Vec` de la documentation du module.
    perimetre: Vec<(usize, usize)>,
}

impl Surface {
    /// Une surface plate (toutes les hauteurs à 0, aucune particule) de `largeur` colonnes, pour ce modèle, avec le
    /// générateur `Alea::new(graine)`. Pour Eden, le périmètre est celui du départ (les `largeur` sites de la rangée 0, dans
    /// l'ordre `x = 0..largeur`).
    ///
    /// # Panique
    ///
    /// Si `largeur` vaut 0 ou dépasse [`LARGEUR_MAX`].
    #[allow(unused_variables)]
    pub fn new(modele: Modele, largeur: usize, graine: u64) -> Surface {
        todo!()
    }

    /// Une surface de départ **donnée**, pour tester une règle sur un profil écrit à la main : mêmes règles que [`Surface::new`]
    /// mais les hauteurs sont `hauteurs` et le nombre de particules est `Σ hauteurs`. Seuls `Depot`, `Relaxation` et
    /// `Balistique` sont permis.
    ///
    /// # Panique
    ///
    /// Si `modele` est `Eden`, si `hauteurs` est vide ou plus long que [`LARGEUR_MAX`].
    #[allow(unused_variables)]
    pub fn depuis_hauteurs(modele: Modele, hauteurs: &[u64], graine: u64) -> Surface {
        todo!()
    }

    /// Le modèle.
    pub fn modele(&self) -> Modele {
        todo!()
    }

    /// Le nombre de colonnes `L`.
    pub fn largeur(&self) -> usize {
        todo!()
    }

    /// Le nombre de particules déposées (une par pas).
    pub fn particules(&self) -> u64 {
        todo!()
    }

    /// Le temps : `particules / L` (division entière), le nombre de couches complètes.
    pub fn couches(&self) -> u64 {
        todo!()
    }

    /// Les `L` hauteurs `h[0..L]`. Pour Eden : le profil du dessus (voir la documentation du module).
    pub fn hauteurs(&self) -> Vec<u64> {
        todo!()
    }

    /// Un pas : tire le choix du modèle (`k = borne(len(périmètre))` pour Eden, sinon `i = borne(L)`) puis fait
    /// [`Surface::appliquer`]. Le premier tirage est donc toujours un appel à `Alea::borne`.
    pub fn deposer(&mut self) {
        todo!()
    }

    /// `L` pas : une couche de plus (le temps `couches()` croît de 1).
    pub fn deposer_couche(&mut self) {
        todo!()
    }

    /// La suite d'un pas, **après** le tirage du choix : `choix` est la colonne `i` (dans `0..L`) pour `Depot`, `Relaxation` et
    /// `Balistique`, l'indice `k` dans le périmètre (dans `0..len`) pour Eden. Applique la règle du modèle (voir la
    /// documentation du module), y compris l'éventuel tirage de la pièce de la relaxation, et compte une particule. `deposer`
    /// n'est que « tirer le choix, puis appliquer » : ce pas permet de tester une règle sur un choix imposé.
    ///
    /// # Panique
    ///
    /// Si `choix` est hors de sa plage ; pour Eden, au pas qui ajouterait au périmètre un site de la rangée
    /// `HAUTEUR_EDEN` ou au-dessus.
    #[allow(unused_variables)]
    pub fn appliquer(&mut self, choix: usize) {
        todo!()
    }

    /// Le périmètre d'Eden, dans l'ordre du `Vec` de la documentation du module (une copie). Vide pour les trois autres modèles.
    pub fn perimetre(&self) -> Vec<(usize, usize)> {
        todo!()
    }

    /// Le site `(x, y)` est-il occupé ? Pour Eden : l'occupation de la grille (faux au-delà de la rangée la plus haute atteinte).
    /// Pour les trois autres modèles : `y < h[x]` (la colonne est pleine jusqu'à sa hauteur).
    ///
    /// # Panique
    ///
    /// Si `x >= L`.
    #[allow(unused_variables)]
    pub fn occupe(&self, x: usize, y: usize) -> bool {
        todo!()
    }

    /// Le carré de la rugosité, `w²` (voir la documentation du module) : `rugosite_carre_de(&self.hauteurs())`.
    pub fn rugosite_carre(&self) -> f64 {
        todo!()
    }

    /// La rugosité `w` : `rugosite_de(&self.hauteurs())`.
    pub fn rugosite(&self) -> f64 {
        todo!()
    }

    /// L'écriture canonique de l'état (voir la documentation du module).
    pub fn octets(&self) -> Vec<u8> {
        todo!()
    }

    /// L'empreinte de l'état : `empreinte::sha256_hex(&self.octets())` (64 chiffres hexadécimaux minuscules).
    pub fn empreinte(&self) -> String {
        todo!()
    }
}

/// La rugosité au carré de `hauteurs`, en entiers : `(L · S2 - S1², L²)` avec `L = hauteurs.len()`, `S1 = Σ h` et
/// `S2 = Σ h²` en `u128` (le rapport n'est pas simplifié). Exemple : `[0, 2]` donne `(2 · 4 - 4, 4) = (4, 4)`, soit
/// `w² = 1`. Exact tant que chaque hauteur est inférieure à `2^40` et `L` à `2^20`.
///
/// # Panique
///
/// Si `hauteurs` est vide.
#[allow(unused_variables)]
pub fn rugosite_exacte(hauteurs: &[u64]) -> (u128, u128) {
    todo!()
}

/// `w²` de `hauteurs` : `numérateur as f64 / dénominateur as f64` pour `(numérateur, dénominateur) = rugosite_exacte(hauteurs)`.
///
/// # Panique
///
/// Si `hauteurs` est vide.
#[allow(unused_variables)]
pub fn rugosite_carre_de(hauteurs: &[u64]) -> f64 {
    todo!()
}

/// `w` de `hauteurs` : `rugosite_carre_de(hauteurs).sqrt()`.
///
/// # Panique
///
/// Si `hauteurs` est vide.
#[allow(unused_variables)]
pub fn rugosite_de(hauteurs: &[u64]) -> f64 {
    todo!()
}

/// La variance de `w²` pour le dépôt aléatoire après `n` particules sur `l` colonnes : `Var(w²) = 2 n (n - 1) (l - 1) / l⁴`,
/// calculée en `f64` par `2,0 · n · (n - 1) · (l - 1) / (l · l · l · l)` (les produits faits de gauche à droite).
///
/// Pourquoi : `w² = (S2 - n²/l) / l` avec `S2 = Σ h[x]²` ; `S2 = n + 2 · (le nombre de paires de particules tombées dans la
/// même colonne)` ; ces `n(n-1)/2` indicatrices de paire valent 1 avec la probabilité `1/l` et sont **deux à deux
/// indépendantes** (deux paires qui partagent une particule : `P(c_a = c_b et c_a = c_c) = 1/l² = P(c_a = c_b) · P(c_a = c_c)`),
/// donc `Var(S2) = 4 · (n(n-1)/2) · (1/l)(1 - 1/l) = 2 n (n-1) (l-1) / l²` et `Var(w²) = Var(S2) / l²`. De même
/// `E[S2] = n + n(n-1)/l`, d'où `E[w²] = n (1 - 1/l) / l`, soit `t (1 - 1/l)` pour `n = t · l`.
///
/// # Panique
///
/// Si `l` vaut 0.
#[allow(unused_variables)]
pub fn variance_rugosite_carre_depot(n: u64, l: u64) -> f64 {
    todo!()
}

/// La pente des moindres carrés de `ys` contre `xs` : `Σ (x - x̄)(y - ȳ) / Σ (x - x̄)²`, avec `x̄ = (Σ x) / n` et `ȳ = (Σ y) / n`
/// (sommes faites dans l'ordre des indices, en `f64`, `n as f64`). Le résultat est `NaN` si tous les `x` sont égaux
/// (`0 / 0`). Exemples : `xs = [0, 1, 2, 3]`, `ys = [1, 3, 5, 7]` donnent exactement 2 ; `xs = [1, 2, 3, 4, 5]`,
/// `ys = [2,2 ; 2,8 ; 4,4 ; 4,0 ; 5,6]` donnent `8 / 10 = 0,8`.
///
/// # Panique
///
/// Si `xs` et `ys` n'ont pas la même longueur ou moins de deux éléments.
#[allow(unused_variables)]
pub fn pente(xs: &[f64], ys: &[f64]) -> f64 {
    todo!()
}

/// Le `w²` moyen aux instants `instants`, sur les graines `graines` : pour chaque graine, une surface `Surface::new(modele,
/// largeur, graine)` fait `deposer_couche()` jusqu'au dernier instant, et `w²` (`rugosite_carre`) est relevé quand
/// `couches()` atteint chaque instant ; la moyenne d'un instant est la somme des `w²` **dans l'ordre des graines** (en
/// `f64`, à partir de 0,0) divisée par `graines.len() as f64`.
///
/// # Panique
///
/// Si `graines` ou `instants` est vide, si `instants` n'est pas strictement croissant, ou si son premier élément est 0.
#[allow(unused_variables)]
pub fn rugosites_moyennes(modele: Modele, largeur: usize, graines: &[u64], instants: &[u64]) -> Vec<f64> {
    todo!()
}

/// L'exposant de croissance `β` mesuré : la [`pente`] de `0,5 · ln(w²moyen)` contre `ln(t)` aux instants `instants` (voir
/// « La mesure de l'exposant β » dans la documentation du module), `w²moyen` venant de [`rugosites_moyennes`] et
/// `ln` de [`crate::math::logarithme`] : `xs[k] = logarithme(instants[k] as f64)`, `ys[k] = 0,5 · logarithme(w²moyen[k])`.
/// `NaN` si un `w²moyen` est nul (`largeur` 1).
///
/// # Panique
///
/// Comme [`rugosites_moyennes`], et si `instants` a moins de deux éléments.
#[allow(unused_variables)]
pub fn exposant(modele: Modele, largeur: usize, graines: &[u64], instants: &[u64]) -> f64 {
    todo!()
}

//! Attachement et détachement thermiques : le modèle « solide sur solide » (SOS, cristal de Kossel) en Monte-Carlo
//! (étape 4). Une face cristalline carrée et périodique, des hauteurs entières, des essais d'ajouter ou
//! d'ôter une particule, des probabilités d'Arrhenius **lues dans la table** [`crate::math::EXP_NEG`] : aucune
//! exponentielle dans le code.
//!
//! **Ce que « Monte-Carlo » veut dire ici.** C'est une dynamique de **Metropolis à une colonne** : une colonne tirée au
//! hasard, « ajouter » ou « ôter » à une chance sur deux, acceptation `min(1, e^(−ΔH/kT))`. Ce n'est **pas** la cinétique
//! de Gilmer et Bennema (1972), où une particule arrive à taux constant et part à un taux `e^(−n·φ/kT)` qui dépend de ses
//! seuls voisins : les deux dynamiques ont la même loi d'équilibre (Boltzmann, à `Δμ = 0`), **pas les mêmes vitesses**.
//! Le temps se compte en balayages de cet algorithme, sans lien avec un temps physique, et aucune vitesse mesurée ici
//! ne se compare en nombres à une vitesse publiée pour une autre dynamique.
//!
//! # Le modèle
//!
//! Une face de `L × L` colonnes (`L = taille`), périodique dans les deux directions : la colonne `(x, y)` a pour
//! hauteur un entier `h(x, y)` (de signe quelconque ; au départ tout est à 0). Ses quatre voisins sont, dans cet
//! ordre, `(x+1, y)`, `(x, y+1)`, `(x-1, y)`, `(x, y-1)`, les coordonnées étant prises modulo `L` (reste euclidien).
//! La colonne `(x, y)` porte le numéro `i = y * L + x` (de `0` à `L² - 1`). Avec `L >= 3`, les quatre voisins d'une
//! colonne sont quatre colonnes distinctes et distinctes d'elle, et chaque paire de colonnes voisines est une
//! **liaison**, comptée **une seule fois** (`2 L²` liaisons).
//!
//! L'énergie (le hamiltonien du modèle SOS à valeur absolue, ASOS) est
//!
//! ```text
//! H = J · Σ_liaisons |h_i − h_j|  −  Δμ · Σ_i h_i
//! ```
//!
//! `J` est l'énergie par unité de marche (une différence de hauteur de 1 entre deux voisins coûte `J`), et `Δμ` la
//! différence de potentiel chimique (la sursaturation : `Δμ > 0` fait croître la face, `Δμ < 0` la dissout, `Δμ = 0`
//! est l'équilibre). Les deux sont donnés **divisés par `kT`** (`k` la constante de Boltzmann, `T` la température
//! effective) et **en seizièmes entiers** : `j_seiziemes = 16 · J/kT` (un `u32`) et `mu_seiziemes = 16 · Δμ/kT` (un
//! `i32`). `j_seiziemes = 16` veut dire `J/kT = 1`. Dans toute la suite, `H` est compté en seizièmes de `kT` : c'est un
//! entier. Avec cette convention (chaque liaison une fois, `K = J/kT`), la face carrée de ce modèle (ASOS) est lisse
//! pour `K > K_R` et rugueuse pour `K < K_R`, `K_R = 0,80608(2)` (Hasenbusch et Pinn, J. Phys. A 30, 63 (1997)), soit
//! `kT_R ≈ 1,24 J` : entre `j_seiziemes = 12` (rugueuse) et `14` (lisse).
//!
//! # Un essai
//!
//! Un essai tire une colonne `i` au hasard, tire « ajouter » ou « ôter » (une chance sur deux), et propose de changer
//! `h_i` de `+1` (ajouter) ou de `-1` (ôter). Soit `k = H(après) − H(avant)` le changement d'énergie en seizièmes de
//! `kT`. Seules les liaisons de la colonne `i` et le terme `Δμ` changent. Si `n` est le nombre de voisins
//! de `i` (de 0 à 4) qui ont, **avant l'essai**,
//!
//! - pour **ajouter** : une hauteur **inférieure ou égale** à `h_i` (`h_j <= h_i` : la marche avec ce voisin grandit de 1) ;
//! - pour **ôter** : une hauteur **supérieure ou égale** à `h_i` (`h_j >= h_i` : la marche avec ce voisin grandit de 1) ;
//!
//! alors les autres voisins (`4 − n`) voient leur marche diminuer de 1, et
//!
//! ```text
//! ajouter :  k = j_seiziemes · (2n − 4) − mu_seiziemes
//! ôter    :  k = j_seiziemes · (2n − 4) + mu_seiziemes
//! ```
//!
//! (calculé en `i64`). C'est la règle de Metropolis : l'essai est **accepté avec la probabilité `min(1, e^(−k/16))`**.
//! Quand `k <= 0` l'essai est accepté à coup sûr (y compris `k = 0`), et sinon la probabilité est lue dans la table :
//! `p = EXP_NEG[k] / 2^32` (`k` entier de seizièmes ; `p = 0` pour `k >= 366`, au-delà de la table).
//!
//! # La consommation du hasard, appel par appel
//!
//! Le générateur est `Alea::new(graine)` (flux 54), propre à la face. **Chaque essai** consomme, dans cet ordre,
//!
//! 1. `i = alea.borne(L * L)` (un entier de `0` à `L² − 1`, avec ses éventuels rejets : voir `Alea::borne`) ;
//!    `x = i % L`, `y = i / L` ;
//! 2. `pile = alea.suivant() >> 31` (le bit de poids fort du tirage de 32 bits) : `pile = 1` veut dire **ajouter**,
//!    `pile = 0` veut dire **ôter** ;
//! 3. **seulement si `1 <= k <= 365`** (la probabilité est strictement entre 0 et 1) : `r = alea.suivant()` ; l'essai
//!    est accepté **si et seulement si `r < EXP_NEG[k]`** (comparaison stricte, `r` converti en `u64` ; un tirage égal
//!    à `EXP_NEG[k]` est refusé). Si `k <= 0` ou si `k >= 366`, **aucun tirage n'est consommé** pour la décision
//!    (accepté d'office dans le premier cas, refusé d'office dans le second).
//!
//! Si l'essai est accepté, `h_i` devient `h_i + 1` (ajouter) ou `h_i − 1` (ôter) ; les autres colonnes ne bougent pas.
//! Rien d'autre ne consomme le hasard. Toute implémentation fidèle donne donc, depuis la même graine, **exactement les
//! mêmes hauteurs** et le même état du générateur, essai après essai.
//!
//! Un **balayage** est une suite de `L²` essais, l'un après l'autre. Le temps se compte en balayages. Les hauteurs sont
//! des `i32` : un dépassement **panique** (jamais de rebouclage silencieux).
//!
//! # Les mesures
//!
//! Avec `N = L²`, `S = Σ h_i` ([`Face::somme_hauteurs`], un `i64`) et `Q = Σ h_i²` ([`Face::somme_carres`], un `u128`) :
//!
//! - la **hauteur moyenne** `<h> = S / N` ([`Face::hauteur_moyenne`] : `S as f64 / N as f64`, une seule division) ;
//! - la **rugosité** `w² = <h²> − <h>² = (N·Q − S²) / N²`, la variance des hauteurs sur la face
//!   ([`Face::rugosite_carree`] : `(N·Q − S²)` calculé exactement en entiers puis `as f64`, divisé par `(N²) as f64` ;
//!   c'est toujours `>= 0`) ;
//! - la **vitesse de croissance** : la variation de la hauteur moyenne par balayage ([`vitesse_croissance`]) ;
//! - la **rugosité moyenne** à l'équilibre ([`rugosite_moyenne`]).
//!
//! Sur le calcul de `f64` : seules des sommes **entières** sont faites ; chaque nombre `f64` rendu est une conversion
//! d'entier puis une division, donc le même sous Linux et sous Windows (aucune somme de flottants dont l'ordre ne serait
//! pas fixé). *Précondition : les hauteurs restent de valeur absolue inférieure à 2^20* (les calculs en `i128` ne
//! débordent alors pas, même à `4096 × 4096`) ; au-delà, les mesures ne sont pas définies.
//!
//! # L'écriture canonique et l'empreinte
//!
//! [`Face::ecrire_canonique`] ajoute à la suite de `sortie`, dans cet ordre (tous les entiers petit-boutistes) :
//!
//! ```text
//! 8 octets      l'étiquette ASCII "GIVRE004"  (47 49 56 52 45 30 30 34)
//! 4 octets      taille (u32)
//! 4 octets      j_seiziemes (u32)
//! 4 octets      mu_seiziemes (i32)
//! 4·L² octets   les hauteurs (i32), dans l'ordre des numéros i = y·L + x (de 0 à L² − 1)
//! 16 octets     l'état du générateur : alea.octets()  (voir Alea::octets), tel qu'il est maintenant
//! ```
//!
//! soit `36 + 4·L²` octets. L'**empreinte** est le SHA-256 de cette suite ([`crate::empreinte::sha256_hex`]) :
//! [`Face::empreinte`]. Elle dépend des hauteurs **et** du nombre exact de tirages consommés.
//!
//! # L'aperçu
//!
//! [`Face::apercu`] : une image RVB de `L × L` pixels (la ligne `y` de haut en bas, `x` de gauche à droite : le pixel
//! `(x, y)` est le numéro `i`). Avec `h_min` et `h_max` les hauteurs extrêmes de la face, le niveau de gris du pixel est
//! `v = 255 · (h − h_min) / (h_max − h_min)` en **division entière** (arrondie vers le bas, calculée en `i64`) si
//! `h_max > h_min`, et `v = 128` pour toute la face si `h_max = h_min` ; le pixel vaut `(v, v, v)`. L'aperçu est pour
//! l'œil, jamais une preuve.
//!
//! # La directive de scénario `cinetique`
//!
//! `cinetique <j> <mu> <balayages>` dans un fichier de scénario (voir [`crate::lancer`]) : `<j>` (un `u32`) et
//! `<balayages>` (de 1 à [`BALAYAGES_MAX`]) en chiffres décimaux ASCII ; `<mu>` un `i32` : un `-` facultatif puis des
//! chiffres (ni `+`, ni espace, ni point). La face est la `boite` du scénario, qui doit être **carrée** (`largeur =
//! hauteur = L`, `L` de [`TAILLE_MIN`] à [`TAILLE_MAX`]).
//!
//! # Ce que `lancer` fait de la directive
//!
//! Le code est dans `src/lancer.rs`, que l'étape 4 modifie ; son contrat est écrit ici et dans les tests de
//! `tests/etape_4_lancer.rs`. Un scénario porte **au plus une** directive `cinetique` (et toujours sa `boite`, ou son
//! `hexagone`, que `lire_scenario` exige). `lancer(scenario, graine)` lit cette directive, fait `Face::new(L, j, mu, graine)`
//! (`L` le côté de la boîte), puis [`simuler`]`(&mut face, balayages)`, et rend :
//!
//! - `empreinte` : `face.empreinte()` de la face **après** les `balayages` balayages (l'écriture canonique de ce module, qui
//!   remplace celle du scénario vide) ;
//! - `mesures` : le texte que rend `simuler` (`balayage,hauteur_moyenne,rugosite`), et non `temps,cases` ;
//! - `apercu` : le PNG `image::ecrire_png(L, L, &face.apercu())`.
//!
//! Une directive `cinetique` mal formée (mauvais nombre d'arguments, entier hors bornes ou mal écrit), une **seconde**
//! directive `cinetique`, une forme qui n'est pas une boîte carrée de côté de [`TAILLE_MIN`] à [`TAILLE_MAX`], ou, dans un
//! scénario qui porte une directive `cinetique`, toute autre directive (un tel scénario ne porte que sa forme et cette
//! directive) est refusée par `ErreurLancer::DirectiveInconnue(ligne)` : `ligne` est celle de la première directive
//! refusée, dans l'ordre du fichier, lignes ignorées comptées (pour une forme refusée : la ligne de la directive `cinetique`).
//! Sans directive `cinetique`, l'étape 4 ne change rien à `lancer` : il fait ce qu'il faisait sans elle (le scénario vide
//! de l'étape 0 ; les directives des autres étapes suivent leur propre doc).

use crate::alea::Alea;
use crate::math::EXP_NEG;

/// Le plus petit côté d'une face : 3 (en dessous, deux voisins d'une colonne seraient la même colonne).
pub const TAILLE_MIN: usize = 3;

/// Le plus grand côté d'une face : 4096.
pub const TAILLE_MAX: usize = 4096;

/// Le plus grand nombre de balayages d'une directive `cinetique` : 1 000 000.
pub const BALAYAGES_MAX: usize = 1_000_000;

/// Une face cristalline carrée et périodique (voir la documentation du module) : ses hauteurs, ses paramètres et son
/// générateur. Cloner une face clone aussi le générateur : le clone refait les mêmes essais.
#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct Face {
    taille: usize,
    j_seiziemes: u32,
    mu_seiziemes: i32,
    hauteurs: Vec<i32>,
    alea: Alea,
}

impl Face {
    /// Une face plane (toutes les hauteurs à 0) de `taille × taille` colonnes, avec `J/kT = j_seiziemes / 16`,
    /// `Δμ/kT = mu_seiziemes / 16` et le générateur `Alea::new(graine)`.
    ///
    /// # Panique
    ///
    /// Si `taille` n'est pas entre [`TAILLE_MIN`] et [`TAILLE_MAX`] (inclus).
    #[allow(unused_variables)]
    pub fn new(taille: usize, j_seiziemes: u32, mu_seiziemes: i32, graine: u64) -> Face {
        assert!((TAILLE_MIN..=TAILLE_MAX).contains(&taille), "taille {taille} hors de {TAILLE_MIN}..={TAILLE_MAX}");
        Face { taille, j_seiziemes, mu_seiziemes, hauteurs: vec![0; taille * taille], alea: Alea::new(graine) }
    }

    /// Une face de hauteurs données : `hauteurs[y * taille + x]` est la hauteur de la colonne `(x, y)`. Le reste est
    /// comme pour [`Face::new`] (générateur `Alea::new(graine)`, aucun tirage consommé).
    ///
    /// # Panique
    ///
    /// Si `taille` n'est pas entre [`TAILLE_MIN`] et [`TAILLE_MAX`], ou si `hauteurs.len() != taille * taille`.
    #[allow(unused_variables)]
    pub fn depuis_hauteurs(taille: usize, j_seiziemes: u32, mu_seiziemes: i32, graine: u64, hauteurs: &[i32]) -> Face {
        assert!((TAILLE_MIN..=TAILLE_MAX).contains(&taille), "taille {taille} hors de {TAILLE_MIN}..={TAILLE_MAX}");
        assert_eq!(hauteurs.len(), taille * taille, "nombre de hauteurs incohérent avec la taille");
        Face { taille, j_seiziemes, mu_seiziemes, hauteurs: hauteurs.to_vec(), alea: Alea::new(graine) }
    }

    /// Le côté `L` de la face.
    #[allow(unused_variables)]
    pub fn taille(&self) -> usize {
        self.taille
    }

    /// La hauteur de la colonne `(x, y)`.
    ///
    /// # Panique
    ///
    /// Si `x >= taille` ou `y >= taille`.
    #[allow(unused_variables)]
    pub fn hauteur(&self, x: usize, y: usize) -> i32 {
        assert!(x < self.taille, "x = {x} hors de la face de côté {}", self.taille);
        assert!(y < self.taille, "y = {y} hors de la face de côté {}", self.taille);
        self.hauteurs[y * self.taille + x]
    }

    /// Toutes les hauteurs, dans l'ordre des numéros `i = y * taille + x`.
    #[allow(unused_variables)]
    pub fn hauteurs(&self) -> &[i32] {
        &self.hauteurs
    }

    /// Le nombre de seizièmes de `kT` du changement d'énergie `k` d'un essai sur la colonne `(x, y)` **dans l'état
    /// actuel** (voir « Un essai » dans la documentation du module) : `ajout = true` pour ajouter une particule
    /// (`h + 1`), `false` pour en ôter une (`h − 1`). C'est l'indice lu dans la table quand il est entre 1 et 365. Il
    /// ne consomme aucun tirage et ne change pas la face.
    ///
    /// # Panique
    ///
    /// Si `x >= taille` ou `y >= taille`.
    #[allow(unused_variables)]
    pub fn indice(&self, x: usize, y: usize, ajout: bool) -> i64 {
        assert!(x < self.taille, "x = {x} hors de la face de côté {}", self.taille);
        assert!(y < self.taille, "y = {y} hors de la face de côté {}", self.taille);
        let l = self.taille;
        let h = self.hauteurs[y * l + x];
        let voisins = [
            self.hauteurs[y * l + (x + 1) % l],
            self.hauteurs[(y + 1) % l * l + x],
            self.hauteurs[y * l + (x + l - 1) % l],
            self.hauteurs[(y + l - 1) % l * l + x],
        ];
        let mut n = 0_i64;
        for v in voisins {
            let compte = if ajout { v <= h } else { v >= h };
            if compte {
                n += 1;
            }
        }
        let j = i64::from(self.j_seiziemes);
        let mu = i64::from(self.mu_seiziemes);
        let base = j * (2 * n - 4);
        if ajout { base - mu } else { base + mu }
    }

    /// Un essai (voir « Un essai » et « La consommation du hasard » dans la documentation du module). Rend `true` si
    /// l'essai est accepté (une hauteur a changé de 1), `false` sinon.
    #[allow(unused_variables)]
    pub fn essai(&mut self) -> bool {
        let l = self.taille;
        let i = self.alea.borne((l * l) as u32) as usize;
        let (x, y) = (i % l, i / l);
        let ajout = self.alea.suivant() >> 31 == 1;
        let k = self.indice(x, y, ajout);
        let accepte = if k <= 0 {
            true
        } else if k >= 366 {
            false
        } else {
            u64::from(self.alea.suivant()) < EXP_NEG[k as usize]
        };
        if accepte {
            let pas = if ajout { 1 } else { -1 };
            let case = y * l + x;
            self.hauteurs[case] =
                self.hauteurs[case].checked_add(pas).expect("dépassement d'un i32 : hauteur hors bornes");
        }
        accepte
    }

    /// Un balayage : `taille²` essais l'un après l'autre.
    #[allow(unused_variables)]
    pub fn balayage(&mut self) {
        for _ in 0..self.taille * self.taille {
            self.essai();
        }
    }

    /// `n` balayages l'un après l'autre.
    #[allow(unused_variables)]
    pub fn balayages(&mut self, n: usize) {
        for _ in 0..n {
            self.balayage();
        }
    }

    /// La somme `S` des hauteurs (exacte).
    #[allow(unused_variables)]
    pub fn somme_hauteurs(&self) -> i64 {
        self.hauteurs.iter().map(|&h| i64::from(h)).sum()
    }

    /// La somme `Q` des carrés des hauteurs (exacte).
    #[allow(unused_variables)]
    pub fn somme_carres(&self) -> u128 {
        self.hauteurs
            .iter()
            .map(|&h| {
                let h = i128::from(h);
                (h * h) as u128
            })
            .sum()
    }

    /// La hauteur moyenne `S / N` : `S as f64 / N as f64` (une seule division).
    #[allow(unused_variables)]
    pub fn hauteur_moyenne(&self) -> f64 {
        let n = self.taille * self.taille;
        self.somme_hauteurs() as f64 / n as f64
    }

    /// La rugosité `w² = (N·Q − S²) / N²` (voir la documentation du module) : le numérateur exact converti en `f64`,
    /// divisé par `(N²) as f64`.
    #[allow(unused_variables)]
    pub fn rugosite_carree(&self) -> f64 {
        let n = (self.taille * self.taille) as i128;
        let s = i128::from(self.somme_hauteurs());
        let q = self.somme_carres() as i128;
        let numerateur = n * q - s * s;
        numerateur as f64 / (n * n) as f64
    }

    /// L'écriture canonique de l'état, ajoutée à la suite de `sortie` (voir la documentation du module).
    #[allow(unused_variables, clippy::ptr_arg)]
    pub fn ecrire_canonique(&self, sortie: &mut Vec<u8>) {
        sortie.extend_from_slice(b"GIVRE004");
        sortie.extend_from_slice(&(self.taille as u32).to_le_bytes());
        sortie.extend_from_slice(&self.j_seiziemes.to_le_bytes());
        sortie.extend_from_slice(&self.mu_seiziemes.to_le_bytes());
        for &hauteur in &self.hauteurs {
            sortie.extend_from_slice(&hauteur.to_le_bytes());
        }
        sortie.extend_from_slice(&self.alea.octets());
    }

    /// L'empreinte de l'état : le SHA-256 en hexadécimal minuscule de son écriture canonique.
    #[allow(unused_variables)]
    pub fn empreinte(&self) -> String {
        let mut octets = Vec::new();
        self.ecrire_canonique(&mut octets);
        crate::empreinte::sha256_hex(&octets)
    }

    /// L'aperçu RVB `taille × taille` (voir « L'aperçu » dans la documentation du module) : `3 · taille²` octets.
    #[allow(unused_variables)]
    pub fn apercu(&self) -> Vec<u8> {
        let h_min = *self.hauteurs.iter().min().expect("une face a au moins une hauteur");
        let h_max = *self.hauteurs.iter().max().expect("une face a au moins une hauteur");
        let mut rvb = Vec::with_capacity(3 * self.hauteurs.len());
        for &h in &self.hauteurs {
            let v = if h_max > h_min {
                let numerateur = 255 * (i64::from(h) - i64::from(h_min));
                let denominateur = i64::from(h_max) - i64::from(h_min);
                (numerateur / denominateur) as u8
            } else {
                128
            };
            rvb.extend_from_slice(&[v, v, v]);
        }
        rvb
    }
}

/// La vitesse de croissance : fait `balayages` balayages de plus sur `face` et rend la variation de la hauteur moyenne
/// par balayage, `(S_après − S_avant) / (N · balayages)` (`S` la somme des hauteurs, `N = taille²`) : la différence
/// entière, convertie en `f64`, divisée par `(N · balayages) as f64`. Pour `J = 0` et `Δμ > 0`, la valeur attendue est
/// `(1 − p) / 2` avec `p = EXP_NEG[mu_seiziemes] / 2^32`.
///
/// # Panique
///
/// Si `balayages` vaut 0.
#[allow(unused_variables)]
pub fn vitesse_croissance(face: &mut Face, balayages: usize) -> f64 {
    assert!(balayages != 0, "balayages nul : la vitesse n'est pas définie");
    let n = (face.taille * face.taille) as i128;
    let avant = face.somme_hauteurs();
    face.balayages(balayages);
    let apres = face.somme_hauteurs();
    (apres - avant) as f64 / (n * balayages as i128) as f64
}

/// La rugosité moyenne : fait `balayages` balayages de plus sur `face` ; après **chacun**, ajoute le numérateur exact
/// `N·Q − S²` de la rugosité (un entier) à un total ; rend ce total converti en `f64` divisé par
/// `(N² · balayages) as f64` : la moyenne des `w²` mesurés après chaque balayage (l'état de départ n'est pas compté).
///
/// # Panique
///
/// Si `balayages` vaut 0.
#[allow(unused_variables)]
pub fn rugosite_moyenne(face: &mut Face, balayages: usize) -> f64 {
    assert!(balayages != 0, "balayages nul : la rugosité moyenne n'est pas définie");
    let n = (face.taille * face.taille) as i128;
    let mut total: i128 = 0;
    for _ in 0..balayages {
        face.balayage();
        let s = i128::from(face.somme_hauteurs());
        let q = face.somme_carres() as i128;
        total += n * q - s * s;
    }
    total as f64 / (n * n * balayages as i128) as f64
}

/// Fait `balayages` balayages sur `face` et rend le CSV des mesures. L'en-tête est `balayage,hauteur_moyenne,rugosite\n`
/// (la colonne `rugosite` est `w²`). Puis une ligne pour chaque `t` de `0` à `balayages` tel que `t` soit un multiple
/// de `pas = max(1, balayages / 50)` (division entière), plus une dernière ligne pour `t = balayages` si ce n'en est pas
/// un multiple ; la ligne de `t` décrit la face après `t` balayages (`t = 0` : avant tout balayage). Chaque ligne est
/// `<t>,<hauteur_moyenne>,<rugosite>\n`, sans `\r` ; les deux mesures sont écrites **en décimal avec exactement six
/// chiffres après la virgule** : la valeur exacte `S / N` (ou `(N·Q − S²) / N²`) arrondie au plus proche au millionième,
/// les demi-cas vers `+∞`, calculée en entiers (`floor((2·num·10^6 + den) / (2·den))`, `num` signé pour la hauteur
/// moyenne) ; un résultat négatif s'écrit avec un `-` devant la partie entière (`-0.250000`), zéro s'écrit `0.000000`. Pour
/// `balayages = 0` : l'en-tête et la ligne de `t = 0`.
#[allow(unused_variables)]
pub fn simuler(face: &mut Face, balayages: usize) -> String {
    let n = (face.taille * face.taille) as i128;
    let pas = std::cmp::max(1, balayages / 50);
    let ligne = |face: &Face, t: usize| {
        let s = i128::from(face.somme_hauteurs());
        let q = face.somme_carres() as i128;
        format!("{t},{},{}\n", millionieme(s, n), millionieme(n * q - s * s, n * n))
    };
    let mut sortie = String::from("balayage,hauteur_moyenne,rugosite\n");
    sortie.push_str(&ligne(face, 0));
    for t in 1..=balayages {
        face.balayage();
        if t % pas == 0 || t == balayages {
            sortie.push_str(&ligne(face, t));
        }
    }
    sortie
}

/// `num / den` arrondi au plus proche au millionième, demi-cas vers `+∞`, écrit avec exactement six chiffres après la
/// virgule (voir [`simuler`]) : `floor((2·num·10^6 + den) / (2·den))`, calculé en entiers (`den > 0`).
fn millionieme(num: i128, den: i128) -> String {
    let m = (2 * num * 1_000_000 + den).div_euclid(2 * den);
    let signe = if m < 0 { "-" } else { "" };
    format!("{signe}{}.{:06}", m.abs() / 1_000_000, m.abs() % 1_000_000)
}

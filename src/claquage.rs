//! Le claquage diélectrique de Niemeyer, Pietronero et Wiesmann sur la grille carrée (étape D1) : le champ de Laplace autour
//! d'un amas, et une croissance en `φ^η`.
//!
//! # Le modèle
//!
//! Un **germe** occupe le site `(0, 0)`. Autour de l'amas, le **potentiel** `φ` vérifie l'équation de Laplace : `φ = 0` sur
//! l'amas (l'électrode qui pousse), `φ = 1` sur une **frontière extérieure** circulaire (l'autre électrode), et chaque site
//! libre vaut la moyenne de ses quatre voisins. À chaque pas, les **candidates** sont les sites libres qui touchent l'amas par
//! un côté ; l'une d'elles rejoint l'amas, tirée avec une chance **proportionnelle à `φ^η`**, `η` entier (L. Niemeyer, L.
//! Pietronero et H. J. Wiesmann, Phys. Rev. Lett. 52, 1033, 1984). `η = 0` : toutes les candidates à égalité, un amas plein
//! (le modèle d'Eden) ; `η = 1` : la chance de coller est le potentiel (dans le continu, la probabilité qu'un marcheur arrive là
//! en premier ; sur la grille ce n'est PAS la DLA de [`crate::dla`], dont le marcheur colle dès qu'il touche l'amas : `MODELE.md`
//! le mesure) ; plus `η` monte, plus l'amas est fin. Un site est un couple d'entiers `(x, y)` (`i32`), `x`
//! vers la droite et `y` vers le bas ; le germe est le centre du réseau.
//!
//! **Les choix qui ne sont pas ceux de l'article** (sa méthode exacte, grille, bord et calcul du potentiel, n'a pas été lue) :
//! la frontière est un cercle, le voisinage est à quatre cases, le potentiel est calculé par une sur-relaxation rouge-noir
//! arrêtée à un seuil fixé, dont le départ est le potentiel du pas précédent. Ce sont des choix **à nous** : `MODELE.md` dit
//! ce qu'on a mesuré de leur effet. Aucune valeur publiée ne dépend de cette étape (la dimension de `η = 2` et de `η = 3` est
//! l'objet de l'étape D2).
//!
//! # La dynamique, à la lettre, et la consommation du hasard appel par appel
//!
//! **Cette section est la spécification** : une implémentation qui la suit donne EXACTEMENT les mêmes sites, dans le même
//! ordre, les mêmes potentiels (bit pour bit) et le même état du générateur, qu'une autre. Rien n'est laissé libre.
//!
//! Notations : `d` est le `demi_cote` donné à [`Claquage::new`]. Le **domaine** `Ω` est l'ensemble des sites entiers avec
//! `x² + y² <= d²` (comparaison en entiers, bord compris) ; tout site hors de `Ω`, même très loin, est sur la **frontière
//! extérieure**. Un site de `Ω` est soit dans l'**amas** `C`, soit **libre**. Le germe est dans `Ω` pour tout `d >= 0`.
//! Les quatre voisins d'un site sont, dans l'ordre de [`crate::reseau::VOISINS4`] : droite `(x + 1, y)`, bas `(x, y + 1)`,
//! gauche `(x - 1, y)`, haut `(x, y - 1)` (jamais les diagonales).
//!
//! ## Le potentiel
//!
//! `phi(z)` vaut `0.0` pour `z` dans `C` et `1.0` pour `z` hors de `Ω`. Pour un site libre il est rangé dans un tableau de
//! `f64` ; **au départ** (à la construction) il vaut `1.0` sur tous les sites libres, puis il est **relaxé** :
//!
//! ```text
//! relaxer :
//!   répéter
//!     écart = 0.0
//!     pour couleur dans [0, 1] :                         (d'abord les sites rouges, puis les noirs)
//!       pour chaque site libre z = (x, y) de Ω avec (x + y) mod 2 == couleur :     (mod 2 mathématique : -1 mod 2 = 1)
//!         m = (((phi(x+1,y) + phi(x,y+1)) + phi(x-1,y)) + phi(x,y-1)) / 4.0
//!         v = phi(z) + omega * (m - phi(z))
//!         v = min(max(v, 0.0), 1.0)                         (`clamp(0.0, 1.0)`)
//!         écart = max(écart, abs(v - phi(z)))
//!         phi(z) = v
//!   jusqu'à écart < SEUIL                                    (strictement ; au moins une passe)
//! ```
//!
//! - **Le schéma est une sur-relaxation rouge-noir** : les quatre voisins d'un site sont de l'autre couleur, donc **l'ordre
//!   des sites à l'intérieur d'une couleur ne change aucun bit** ; ce qui compte est que les sites de couleur 0 soient tous
//!   mis à jour avant ceux de couleur 1, chacun avec les valeurs courantes de ses voisins. Une passe est les deux couleurs.
//! - **`omega`** est [`surrelaxation`]`(d) = 2.0 - 6.0 / (d as f64 + 6.0)` (une addition, une division, une soustraction en
//!   `f64`) : entre 1 et 2, d'autant plus près de 2 que le domaine est grand. **Les opérations de `v` sont dans cet ordre** :
//!   `m - phi`, puis `omega * (…)`, puis `phi + (…)`, puis le `clamp` ; pas de `mul_add`, pas d'autre forme algébrique
//!   (`(1 - omega) * phi + omega * m` ne donne pas les mêmes bits). **La somme de `m` est dans l'ordre ci-dessus** ; `/ 4.0`
//!   est la division par quatre.
//! - **`SEUIL`** est [`SEUIL`] `= 2^-20`. Une passe de plus que nécessaire ne se fait pas : on s'arrête dès qu'une passe
//!   entière a un écart inférieur strictement au seuil. L'écart de la passe est le plus grand `|v - phi(z)|` des deux couleurs.
//! - **Quand on relaxe** : une fois à la construction (le champ du germe seul), puis **après chaque site ajouté** (le site
//!   ajouté passe d'abord à `0.0`), jamais ailleurs. Le départ de chaque relaxation est donc le champ relaxé du pas précédent :
//!   le champ ne dépend que de la suite ordonnée des sites, de `d` et de `η`. **À `η = 0` aucun champ n'est calculé** (les
//!   poids sont tous 1 : le résultat n'en dépend pas) et [`Claquage::potentiel`] rend `None`.
//!
//! - **Garde de terminaison** : une relaxation qui n'a pas convergé après [`PASSES_MAX`] passes **panique** (message contenant
//!   « converge »). Le schéma de la doc converge (`ω` entre 1 et 2, opérateur symétrique défini positif) : mesuré, une relaxation
//!   à froid prend de 354 passes (`d = 60`) à 1316 passes (`d = 255`), et au plus quelques centaines par site ajouté ensuite. La garde ne
//!   change aucun bit ; elle transforme en échec un code qui ne converge pas (un voisin diagonal dans la moyenne, par exemple, ne converge pas).
//!
//! **Ce que le seuil garantit** (utile aux tests, pas à l'implémentation) : après la dernière passe, `phi` est dans `[0, 1]`
//! (le `clamp`), et chaque site libre est la moyenne de ses quatre voisins à moins de `2 * SEUIL` près (le site mis à jour
//! en second reste à `|omega - 1| / omega * SEUIL` près de la moyenne ; le premier a vu ses voisins changer de moins de `SEUIL`).
//! Par le principe du maximum, l'écart de `phi` à la solution exacte du problème de Laplace discret est au plus
//! `2 * SEUIL * (d + 1)² `.
//!
//! ## Les candidates
//!
//! Les **candidates** sont les sites libres de `Ω` dont l'un des quatre voisins est dans `C`. Elles sont **énumérées dans
//! l'ordre canonique : `y` croissant, puis `x` croissant**. Le **poids** d'une candidate est `w = phi(z)^η`, calculé par
//! `η` multiplications répétées : `w = 1.0`, puis `w = w * phi(z)` répété `η` fois (jamais `powi`, jamais `powf` : `η = 0`
//! donne exactement `1.0`, `η = 1` donne `phi(z)` lui-même). À `η = 0` tous les poids valent `1.0`. [`Claquage::candidates`]
//! rend la liste dans cet ordre avec ces poids.
//!
//! ## Le tirage
//!
//! Le générateur est `Alea::new(graine)` (flux 54, voir [`crate::alea`]), amorcé par [`Claquage::new`] sans aucun tirage. Il
//! n'est tiré que d'**un seul** `Alea::suivant_f64()` par site ajouté (deux tirages de 32 bits), et d'aucun autre. Avec les
//! poids `w_0, w_1, …, w_{k-1}` des `k` candidates, dans l'ordre canonique :
//!
//! 1. `W = 0.0`, puis `W = W + w_i` pour `i = 0, 1, …` dans l'ordre (une somme de gauche à droite) ;
//! 2. `t = alea.suivant_f64()` puis `u = t * W` (une multiplication) ;
//! 3. le site choisi est la **première** candidate d'indice `i` dont la somme cumulée `S_i = (…((0.0 + w_0) + w_1) + …) + w_i`
//!    (les mêmes additions que `W`, dans le même ordre) **dépasse strictement** `u` : `S_i > u`. (Avec `η = 0`, c'est la
//!    candidate d'indice `⌊t · k⌋`.) Une candidate de poids `0.0` n'est donc jamais choisie. **Comme `t <= 1 - 2^-53`, `u < W`
//!    même après l'arrondi** : une candidate est toujours trouvée, et le dernier site n'a pas de règle à part.
//!
//! ## L'arrêt
//!
//! [`Claquage::essayer_ajouter`] rend `None` **sans consommer aucun tirage ni rien changer** dans ces cas, vérifiés dans cet
//! ordre :
//!
//! 1. **L'amas touche la frontière** : l'un de ses sites a un voisin hors de `Ω` (la décharge a rejoint l'autre électrode).
//!    Cela se décide quand un site est ajouté : le site qui touche est, lui, bien ajouté, et c'est le pas suivant qui rend
//!    `None`. Le germe touche la frontière si et seulement si `d = 0` (ses voisins sont à distance 1 : `1 > d²`) ;
//! 2. il n'y a **aucune candidate** (impossible tant que l'amas ne touche pas la frontière, par construction) ;
//! 3. `W == 0.0` (tous les poids nuls : un potentiel nul partout sur les candidates, ou un `φ^η` qui s'annule à l'arrondi ;
//!    cas qu'aucune graine connue n'atteint).
//!
//! # Le rayon de l'amas
//!
//! [`Claquage::rayon`] est le plus petit entier `r >= 0` tel que **tous** les sites vérifient `x² + y² <= r²` (le germe seul :
//! 0). C'est `ceil(sqrt(max (x² + y²)))`, en entiers. Il est au plus `d`.
//!
//! # L'écriture canonique et l'empreinte
//!
//! [`Claquage::ecriture_canonique`] est, dans cet ordre :
//!
//! ```text
//! 8 octets    l'étiquette ASCII "GIVREDB1"  (47 49 56 52 45 44 42 31)
//! 4 octets    demi_cote, u32 petit-boutiste
//! 4 octets    eta, u32 petit-boutiste
//! 4 octets    le nombre de sites n, u32 petit-boutiste
//! 8 n octets  pour chaque site, dans l'ordre d'arrivée : x puis y, chacun i32 petit-boutiste (4 octets chacun)
//! 16 octets   Alea.octets() : l'état du générateur après le dernier site (voir Alea::octets)
//! ```
//!
//! soit `36 + 8 n` octets. Le champ `phi` n'y figure pas : il est fonction de la suite des sites, de `d` et de `η`.
//! [`Claquage::empreinte`] est le SHA-256 en hexadécimal minuscule ([`crate::empreinte::sha256_hex`]) de cette écriture.
//!
//! # Un conseil de vitesse (pas une règle : seuls les bits ci-dessus sont imposés)
//!
//! Le coût est celui de la relaxation : environ 80 passes par site ajouté à `η = 1` (mesuré : `d = 45`, 300 sites), chacune sur les
//! `π d²` sites du domaine, soit un temps en `d³` par site. En profil `dev` (`opt-level = 3`), le prototype fait 300 sites à `η = 1`
//! dans un domaine de `d = 45` en 0,4 s, 1000 sites dans `d = 110` en 17 s (voir `MODELE.md`) ; à `η = 0` il n'y a pas de champ et
//! 2000 sites coûtent moins d'un dixième de seconde. Les tests jouent des amas de 100 à 300 sites (et 2000 à `η = 0`).
//! Ce qui compte : un tableau de `f64` de côté `2 d + 3` (une marge d'un site, qui vaut `1.0`, évite tout test de bord) ; le
//! demi-côté de chaque ligne, `isqrt(d² - y²)`, est entier ; les sites à `x + y` de même parité se parcourent de deux en deux ;
//! la liste des candidates se refait à chaque pas en un seul balayage.
//!
//! # La directive `claquage` et la sortie de `lancer`
//!
//! **Cette section est le contrat de [`crate::lancer::lancer`] à l'étape D1** (la doc de `src/lancer.rs` la reprend). Une
//! directive `claquage <eta> <particules>` (lue par [`parametres`]) sur une `boite largeur hauteur` lance un claquage :
//! `lancer` fait croître un [`Claquage`] de graine `graine`, de `demi_cote = dla::demi_cote_de_la_boite(largeur, hauteur)` et
//! de l'exposant `eta`, par des appels de [`Claquage::essayer_ajouter`] (jamais [`Claquage::ajouter`], qui paniquerait),
//! jusqu'à `particules` sites (germe compris), et rend, **à la place** des sorties du scénario vide de l'étape 0 :
//!
//! - **l'empreinte** : [`Claquage::empreinte`] de l'amas final (deux boîtes de même `demi_cote` ont donc la même empreinte) ;
//! - **les mesures** : le CSV `particules,rayon,rayon_giration`, **exactement celui de la directive `dla`** (voir
//!   [`crate::dla`]) : la ligne d'en-tête puis une ligne par nombre `k` de sites, `k` parcourant les puissances de 2
//!   `2, 4, 8, …` qui sont `<= particules`, puis `particules` s'il n'est pas déjà dans la liste ; la ligne `k` est
//!   `k,<rayon>,<giration>` avec `<rayon>` le [`Claquage::rayon`] de l'amas quand il avait `k` sites et `<giration>` le
//!   [`crate::mesures::rayon_de_giration`] de ses `k` premiers sites avec **six décimales** ; chaque ligne finit par un seul `\n` ;
//! - **l'aperçu** : le PNG ([`crate::image::ecrire_png`]) de [`crate::dla::amas_colore`]`(amas.sites(), COTE_APERCU)`.
//!
//! **La première directive choisit le lancer** : quand la première directive est `claquage` (et qu'aucune directive du
//! scénario n'est `dla`, `collage` ou `bruit`, auquel cas c'est le lancer de la DLA qui refuse le `claquage` à sa ligne, comme
//! toute directive qu'il ne comprend pas), `lancer` suit ces règles. **Les refus**, tous `ErreurLancer::DirectiveInconnue(ligne)`
//! (ni l'énumération ni `src/main.rs` ne changent), où `ligne` est la **plus petite** des lignes refusées : une directive dont
//! le nom n'est pas `claquage` (sa ligne : `surface`, `reiter`, `cinetique` ou une inconnue) ; une directive `claquage` que
//! [`parametres`] refuse (la ligne qu'elle rend, **une seconde directive `claquage` comprise**) ; une directive `claquage` sur un
//! `hexagone` (la ligne de la première) ; une boîte dont le `demi_cote` dépasse [`DEMI_COTE_MAX`] (la ligne de la directive).
//! Puis, si rien n'est refusé, un amas qui touche la frontière (ou ne peut plus croître) avant d'avoir `particules` sites
//! (`essayer_ajouter` rend `None`) : la ligne de la directive `claquage`. Rien n'est rendu d'autre dans ces cas. Un scénario
//! sans directive `claquage` se comporte exactement comme avant cette étape.
//!
//! **Aucune borne de temps** : le coût est celui de la relaxation (voir « Un conseil de vitesse »), en `d³` par site à `η >= 1`.
//! `claquage 1 3000` sur `boite 511 511` (`d = 255`) prend une dizaine de minutes (extrapolé : 0,2 s par site à `d = 255`, mesuré à `η = 3`) ; seules [`PARTICULES_MAX`] et [`DEMI_COTE_MAX`] bornent
//! la directive. Un appelant qui doit répondre vite (la page, étape D3) borne lui-même la taille.

use crate::alea::Alea;
use crate::reseau::VOISINS4;
use crate::scenario::{ErreurLecture, Scenario};

/// L'exposant `η` le plus grand accepté par [`Claquage::new`] et par la directive : 8.
pub const ETA_MAX: u32 = 8;

/// Le plus grand `demi_cote` d'un domaine : 255, soit une boîte de 511 sites de côté (le domaine a alors environ 205 000
/// sites ; le coût de la relaxation interdit bien plus grand).
pub const DEMI_COTE_MAX: u32 = 255;

/// Le nombre maximal de particules d'une directive `claquage` : 100 000, germe compris.
pub const PARTICULES_MAX: usize = 100_000;

/// Le nombre maximal de passes d'une relaxation : 100 000 (voir « Garde de terminaison » dans la documentation du module).
pub const PASSES_MAX: u32 = 100_000;

/// Le seuil d'arrêt de la relaxation : `2^-20` (voir « Le potentiel » dans la documentation du module).
pub const SEUIL: f64 = 1.0 / 1_048_576.0;

/// Le facteur de sur-relaxation du domaine de demi-côté `demi_cote` : `2.0 - 6.0 / (demi_cote as f64 + 6.0)`. Il vaut 1 pour
/// `demi_cote = 0`, environ 1,909 pour 60 et 1,977 pour 255. (Le facteur optimal d'une relaxation rouge-noir sur un disque
/// de rayon `R` est proche de `2 - 3,4 / R` ; on prend un peu plus, car un facteur trop bas coûte beaucoup plus cher qu'un
/// facteur trop haut.)
pub fn surrelaxation(demi_cote: u32) -> f64 {
    2.0 - 6.0 / (f64::from(demi_cote) + 6.0)
}

/// Un amas de claquage diélectrique en croissance (voir la documentation du module pour la dynamique, le potentiel, la
/// consommation du hasard, l'arrêt, l'écriture canonique et l'empreinte).
#[derive(Clone, Debug)]
#[allow(dead_code)]
pub struct Claquage {
    alea: Alea,
    demi_cote: u32,
    eta: u32,
    omega: f64,
    /// Le côté du tableau : `2 * demi_cote + 3` (une marge d'un site de chaque côté).
    cote: usize,
    sites: Vec<(i32, i32)>,
    occupe: Vec<bool>,
    /// Le potentiel : `1.0` hors du domaine (marge comprise), `0.0` sur l'amas, la valeur relaxée sur les sites libres.
    phi: Vec<f64>,
    plus_grand_carre: u64,
    touche: bool,
}

impl Claquage {
    /// Un amas réduit au germe `(0, 0)`, de graine `graine` (`Alea::new(graine)`, aucun tirage fait), de domaine de demi-côté
    /// `demi_cote` et d'exposant `eta`. Si `eta >= 1`, le champ du germe seul est relaxé (voir « Le potentiel »).
    ///
    /// # Panique
    ///
    /// Si `demi_cote > DEMI_COTE_MAX` ou `eta > ETA_MAX`.
    pub fn new(graine: u64, demi_cote: u32, eta: u32) -> Claquage {
        assert!(demi_cote <= DEMI_COTE_MAX, "le demi-côté doit être au plus {DEMI_COTE_MAX}");
        assert!(eta <= ETA_MAX, "l'exposant doit être au plus {ETA_MAX}");
        let cote = (2 * demi_cote + 3) as usize;
        let mut claquage = Claquage {
            alea: Alea::new(graine),
            demi_cote,
            eta,
            omega: surrelaxation(demi_cote),
            cote,
            sites: vec![(0, 0)],
            occupe: vec![false; cote * cote],
            phi: vec![1.0; cote * cote],
            plus_grand_carre: 0,
            touche: false,
        };
        let germe = claquage.indice(0, 0);
        claquage.occupe[germe] = true;
        claquage.phi[germe] = 0.0;
        claquage.touche = VOISINS4.iter().any(|&(dx, dy)| !claquage.dans_omega(dx as i32, dy as i32));
        if eta >= 1 {
            claquage.relaxer();
        }
        claquage
    }

    /// Le `demi_cote` du domaine, tel que donné à [`Claquage::new`] : le rayon de la frontière extérieure.
    pub fn demi_cote(&self) -> u32 {
        self.demi_cote
    }

    /// L'exposant `η`, tel que donné à [`Claquage::new`].
    pub fn eta(&self) -> u32 {
        self.eta
    }

    /// Les sites de l'amas, dans l'ordre d'arrivée : le germe `(0, 0)` d'abord. Il y en a au moins un.
    pub fn sites(&self) -> &[(i32, i32)] {
        &self.sites
    }

    /// Vrai si `(x, y)` est un site de l'amas (faux hors du domaine, sans panique, pour toute valeur de `i32`).
    pub fn contient(&self, x: i32, y: i32) -> bool {
        self.dans_omega(x, y) && self.occupe[self.indice(x, y)]
    }

    /// Le rayon de l'amas : le plus petit entier `r >= 0` tel que tous les sites vérifient `x² + y² <= r²` (voir « Le rayon de
    /// l'amas »).
    pub fn rayon(&self) -> u32 {
        let mut r = 0_u64;
        while r * r < self.plus_grand_carre {
            r += 1;
        }
        r as u32
    }

    /// Vrai si l'amas touche la frontière : l'un de ses sites a un voisin hors du domaine (voir « L'arrêt »). Alors
    /// [`Claquage::essayer_ajouter`] rend `None`.
    pub fn touche_la_frontiere(&self) -> bool {
        self.touche
    }

    /// Le potentiel courant au site `(x, y)`, pour toute valeur de `i32` : `Some(1.0)` hors du domaine, `Some(0.0)` sur
    /// l'amas, sinon `Some(phi)` la valeur relaxée du site libre (voir « Le potentiel »). `None` quand `eta == 0` (aucun champ
    /// n'est calculé), pour tout site. Entre deux appels de [`Claquage::essayer_ajouter`], le champ est celui de la dernière
    /// relaxation : il correspond à l'amas courant.
    pub fn potentiel(&self, x: i32, y: i32) -> Option<f64> {
        if self.eta == 0 {
            return None;
        }
        if !self.dans_omega(x, y) {
            return Some(1.0);
        }
        let indice = self.indice(x, y);
        if self.occupe[indice] { Some(0.0) } else { Some(self.phi[indice]) }
    }

    /// Les candidates (voir « Les candidates ») avec leur poids `φ^η`, dans l'ordre canonique (`y` croissant, puis `x`
    /// croissant). Vide si l'amas ne peut pas croître parce qu'aucun site libre ne touche l'amas ; jamais vide sinon.
    pub fn candidates(&self) -> Vec<((i32, i32), f64)> {
        let d = self.demi_cote as i32;
        let mut liste = Vec::new();
        for y in -d..=d {
            for x in -d..=d {
                if !self.dans_omega(x, y) {
                    continue;
                }
                let indice = self.indice(x, y);
                if self.occupe[indice] {
                    continue;
                }
                let touche = VOISINS4.iter().any(|&(dx, dy)| self.occupe[self.indice(x + dx as i32, y + dy as i32)]);
                if !touche {
                    continue;
                }
                let mut poids = 1.0;
                for _ in 0..self.eta {
                    poids *= self.phi[indice];
                }
                liste.push(((x, y), poids));
            }
        }
        liste
    }

    /// Ajoute un site (voir « La dynamique ») et le rend, aussi le dernier de [`Claquage::sites`].
    ///
    /// # Panique
    ///
    /// Si l'amas ne peut plus croître (voir « L'arrêt » ; message contenant « frontière »). Aucun site n'est alors ajouté.
    pub fn ajouter(&mut self) -> (i32, i32) {
        match self.essayer_ajouter() {
            Some(site) => site,
            None => panic!("l'amas ne peut plus croître : il touche la frontière"),
        }
    }

    /// Comme [`Claquage::ajouter`], mais rend `None` au lieu de paniquer quand l'amas ne peut plus croître (voir « L'arrêt »),
    /// sans consommer de tirage ni changer l'état.
    pub fn essayer_ajouter(&mut self) -> Option<(i32, i32)> {
        if self.touche {
            return None;
        }
        let candidates = self.candidates();
        if candidates.is_empty() {
            return None;
        }
        let mut total = 0.0;
        for &(_, poids) in &candidates {
            total += poids;
        }
        if total == 0.0 {
            return None;
        }
        let u = self.alea.suivant_f64() * total;
        let mut cumul = 0.0;
        let mut choisi = None;
        for &((x, y), poids) in &candidates {
            cumul += poids;
            if cumul > u {
                choisi = Some((x, y));
                break;
            }
        }
        let (x, y) = choisi.expect("u < W devrait toujours trouver une candidate");
        self.ajouter_site(x, y);
        Some((x, y))
    }

    /// Ajoute des sites jusqu'à ce que l'amas en ait `n` (germe compris) ; ne fait rien s'il en a déjà `n` ou plus.
    /// `croitre_jusqu_a(100)` puis `croitre_jusqu_a(200)` donne les mêmes sites, les mêmes potentiels et le même générateur
    /// que `croitre_jusqu_a(200)` seul.
    ///
    /// # Panique
    ///
    /// Si l'amas ne peut plus croître avant d'avoir `n` sites (voir [`Claquage::ajouter`]).
    pub fn croitre_jusqu_a(&mut self, n: usize) {
        while self.sites.len() < n {
            self.ajouter();
        }
    }

    /// L'écriture canonique de l'état, `36 + 8 n` octets (voir « L'écriture canonique et l'empreinte »).
    pub fn ecriture_canonique(&self) -> Vec<u8> {
        let mut sortie = Vec::with_capacity(36 + 8 * self.sites.len());
        sortie.extend_from_slice(b"GIVREDB1");
        sortie.extend_from_slice(&self.demi_cote.to_le_bytes());
        sortie.extend_from_slice(&self.eta.to_le_bytes());
        sortie.extend_from_slice(&(self.sites.len() as u32).to_le_bytes());
        for &(x, y) in &self.sites {
            sortie.extend_from_slice(&x.to_le_bytes());
            sortie.extend_from_slice(&y.to_le_bytes());
        }
        sortie.extend_from_slice(&self.alea.octets());
        sortie
    }

    /// L'empreinte de l'état : le SHA-256 (64 chiffres hexadécimaux minuscules) de [`Claquage::ecriture_canonique`].
    pub fn empreinte(&self) -> String {
        crate::empreinte::sha256_hex(&self.ecriture_canonique())
    }

    /// Vrai si `(x, y)` est dans le domaine `x² + y² <= demi_cote²` (comparaison entière, exacte pour tout `i32`).
    fn dans_omega(&self, x: i32, y: i32) -> bool {
        let x = i64::from(x);
        let y = i64::from(y);
        let demi_cote = i64::from(self.demi_cote);
        x * x + y * y <= demi_cote * demi_cote
    }

    /// L'indice de `(x, y)` dans les tableaux, avec une marge d'un site de chaque côté.
    fn indice(&self, x: i32, y: i32) -> usize {
        let demi_cote = self.demi_cote as i32;
        let cote = self.cote as i32;
        ((y + demi_cote + 1) * cote + (x + demi_cote + 1)) as usize
    }

    /// Ajoute le site `(x, y)` (voir « Le potentiel » : il est d'abord mis à `0.0`, puis le champ est relaxé).
    fn ajouter_site(&mut self, x: i32, y: i32) {
        self.sites.push((x, y));
        let indice = self.indice(x, y);
        self.occupe[indice] = true;
        self.phi[indice] = 0.0;
        let carre = {
            let x = i64::from(x);
            let y = i64::from(y);
            (x * x + y * y) as u64
        };
        if carre > self.plus_grand_carre {
            self.plus_grand_carre = carre;
        }
        if VOISINS4.iter().any(|&(dx, dy)| !self.dans_omega(x + dx as i32, y + dy as i32)) {
            self.touche = true;
        }
        if self.eta >= 1 {
            self.relaxer();
        }
    }

    /// La sur-relaxation rouge-noir de la documentation (voir « Le potentiel »).
    fn relaxer(&mut self) {
        let demi_cote = self.demi_cote as i32;
        for _ in 0..PASSES_MAX {
            let mut ecart = 0.0_f64;
            for couleur in 0..2_i32 {
                for y in -demi_cote..=demi_cote {
                    for x in -demi_cote..=demi_cote {
                        if (x + y).rem_euclid(2) != couleur || !self.dans_omega(x, y) {
                            continue;
                        }
                        let indice = self.indice(x, y);
                        if self.occupe[indice] {
                            continue;
                        }
                        let ancien = self.phi[indice];
                        let m = (((self.phi[self.indice(x + 1, y)] + self.phi[self.indice(x, y + 1)])
                            + self.phi[self.indice(x - 1, y)])
                            + self.phi[self.indice(x, y - 1)])
                            / 4.0;
                        let v = (ancien + self.omega * (m - ancien)).clamp(0.0, 1.0);
                        let changement = (v - ancien).abs();
                        if changement > ecart {
                            ecart = changement;
                        }
                        self.phi[indice] = v;
                    }
                }
            }
            if ecart < SEUIL {
                return;
            }
        }
        panic!("la relaxation ne converge pas après {PASSES_MAX} passes");
    }
}

/// La directive `claquage <eta> <particules>` d'un scénario : l'exposant et le nombre de particules, **germe compris**
/// (`claquage 1 1` : le germe seul).
///
/// - aucune directive nommée `claquage` dans `scenario.directives` : `Ok(None)` ;
/// - sinon la directive doit avoir **exactement deux arguments**, deux entiers lus comme
///   [`crate::scenario::Directive::entier`] (chiffres décimaux ASCII seulement, sans signe, zéros de tête permis :
///   `claquage 01 05` vaut `(1, 5)`), le premier de 0 à [`ETA_MAX`] inclus, le second de 1 à [`PARTICULES_MAX`] inclus :
///   `Ok(Some((eta, particules)))` ;
/// - toute autre chose est `Err(ErreurLecture::LigneInvalide(ligne))` avec le numéro de ligne de la **première directive
///   `claquage` fautive dans l'ordre du fichier** : un nombre d'arguments faux, un argument mal écrit ou hors bornes, ou une
///   **seconde** directive `claquage` (valide ou non : c'est la seconde qui est fautive).
///
/// Les autres directives sont ignorées ici (c'est `lancer` qui refuse celles qu'il ne comprend pas).
pub fn parametres(scenario: &Scenario) -> Result<Option<(u32, usize)>, ErreurLecture> {
    let mut trouve = None;
    for directive in &scenario.directives {
        if directive.nom != "claquage" {
            continue;
        }
        if trouve.is_some() {
            return Err(ErreurLecture::LigneInvalide(directive.ligne));
        }
        if directive.args.len() != 2 {
            return Err(ErreurLecture::LigneInvalide(directive.ligne));
        }
        let eta = directive.entier(0)?;
        let particules = directive.entier(1)?;
        if eta > u64::from(ETA_MAX) || !(1..=PARTICULES_MAX as u64).contains(&particules) {
            return Err(ErreurLecture::LigneInvalide(directive.ligne));
        }
        trouve = Some((eta as u32, particules as usize));
    }
    Ok(trouve)
}

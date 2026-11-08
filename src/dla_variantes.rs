//! Les variantes de la DLA : la probabilité de collage `p < 1` et la réduction du bruit (étape 2, moitié « variantes »).
//!
//! # Les deux variantes
//!
//! Chacune change **une seule règle** de [`crate::dla`] (l'étape 1) et se règle par **un seul paramètre** ; avec la valeur « neutre »
//! du paramètre, elle redonne EXACTEMENT les sites de [`crate::dla::Amas`] pour la même graine.
//!
//! - **Le collage** ([`Variante::Collage`]) : quand le marcheur touche l'amas, il ne colle qu'avec la probabilité `p` ; sinon il continue
//!   sa marche (T. A. Witten et L. M. Sander, Phys. Rev. B 27, 5686, 1983, section sur la condition aux limites : le marcheur ne colle
//!   que pendant une fraction `s` de ses rencontres ; `s = 1` est la DLA ; l'amas est épaissi aux échelles inférieures à `1/s` et garde
//!   la loi de puissance au-delà). Le paramètre neutre est `p = 1` ([`TOUJOURS`]). Le détail (un tirage à chaque contact, le pas vers
//!   l'amas refusé, voir « La dynamique ») est une **convention de ce dépôt** : Witten et Sander ne le fixent pas pour le réseau carré.
//! - **La réduction du bruit** ([`Variante::Bruit`]) : chaque site vide au contact de l'amas porte un compteur ; un marcheur qui le
//!   touche l'incrémente et disparaît ; le site ne rejoint l'amas que quand son compteur atteint `m` ; les compteurs des autres sites ne
//!   sont jamais remis à zéro (C. Tang, Phys. Rev. A 31, 1977, 1985 ; J. Nittmann et H. E. Stanley, Nature 321, 663, 1986 : textes non
//!   lus ; la règle est celle que décrit P. Meakin, « le score est incrémenté de 1 et la marche est terminée », un nouveau marcheur est
//!   lancé, les scores continuent de s'accumuler). Une autre variante publiée remet TOUS les compteurs à zéro après chaque croissance
//!   (attribuée à J. Kertész et T. Vicsek, J. Phys. A 19, L257, 1986, et à un modèle de Meakin) : ce n'est pas celle-ci. Le paramètre
//!   neutre est `m = 1`. Plus `m` est grand, moins le hasard des marcheurs isolés compte, et plus l'anisotropie du réseau carré se voit :
//!   l'amas pousse quatre bras sur les axes (S. G. Alves et S. C. Ferreira Jr, J. Phys. A 39, 2843, 2006).
//!
//! # La dynamique, à la lettre, et la consommation du hasard appel par appel
//!
//! **Cette section est la spécification.** Elle **renvoie** à la section « La dynamique, à la lettre » de [`crate::dla`] et dit
//! seulement ce qui change ; tout ce qu'elle ne dit pas est comme dans [`crate::dla`] : le générateur est `Alea::new(graine)` (flux 54),
//! amorcé sans tirage ; le lancement (`R = r + MARGE`, `r` le rayon de l'amas, `anneau(R)`, un `Alea::borne(len)` par lancement), les
//! quatre directions de [`crate::reseau::VOISINS4`] (0 droite, 1 bas, 2 gauche, 3 haut), un `Alea::borne(4)` par pas de marche,
//! l'abandon à `x² + y² > A²` avec `A = FACTEUR_ABANDON * R` (strictement), la capacité du réseau (`|x|, |y| <= demi_cote`), le
//! rayon de l'amas ; le germe `(0, 0)`. Le tirage supplémentaire de la variante **collage** (un seul genre de tirage nouveau) est un
//! `Alea::suivant()` : il consomme exactement **un** tirage de 32 bits, et la variante **bruit** n'en consomme aucun de plus.
//!
//! Une **particule** (un appel de [`AmasVariante::essayer_ajouter`]) se joue ainsi (les étapes 1 et 2.2 sont celles de `dla`) :
//!
//! 1. **Lancer** (comme `dla`, étape 1) : `k = alea.borne(len)` et le marcheur part de `anneau(R)[k]`.
//! 2. **Boucle**, avec la position courante `(x, y)` :
//!    1. **Contact** : le marcheur est *au contact* si l'un des quatre voisins `(x + dx, y + dy)` de sa position courante est un site
//!       de l'amas (les diagonales ne comptent pas, comme dans `dla`). Ce test précède tout pas, y compris le premier. S'il n'y a
//!       pas de contact, on passe à 2.2. S'il y a contact, la variante décide :
//!       - **`Collage(p)`** : si `p == TOUJOURS` (`2^32`), le marcheur **colle**, **sans aucun tirage**. Sinon on tire
//!         `u = alea.suivant()` (un `u32`, **un tirage par contact**) et le marcheur colle si et seulement si `u as u64 < p`
//!         (strictement inférieur : `u == p` ne colle pas). S'il ne colle pas, on passe à 2.2 **avec la même position**, sans refaire
//!         le test de contact ni le tirage pour cette position-ci : la boucle reprend à « abandonner », puis « marcher » ;
//!       - **`Bruit(m)`** : si `(x, y)` est **hors du réseau**, aucun compteur n'est touché : c'est un dépassement de la capacité, la
//!         particule finit là (voir « La capacité ») ; sinon le compteur de `(x, y)` (0 au départ, un par site du réseau) augmente de
//!         1 ; s'il vaut alors `m`, le marcheur **colle** ; sinon le marcheur est **abandonné** comme à l'étape 2.2 de `dla` : on
//!         revient à l'étape 1 avec le **même** `R` (l'amas n'a pas changé) et un **nouveau** tirage `borne(len)`, les pas déjà faits
//!         sont perdus, et le compteur garde sa nouvelle valeur ;
//!    2. **Abandonner** (comme `dla`, étape 2.2) : si `x² + y² > A²`, retour à l'étape 1 (même `R`, nouveau `borne(len)`) ;
//!    3. **Marcher** : on tire `d = alea.borne(4)` et le pas vise `(x + dx, y + dy)` pour la direction `d`. **Si le site visé est un
//!       site de l'amas, le pas est refusé : la position ne change pas** (le tirage est consommé quand même) ; sinon la position
//!       devient le site visé. Dans les deux cas on retourne à l'étape 2.1. (Un pas refusé ne peut arriver qu'avec `Collage(p)`,
//!       `p < TOUJOURS` : dans les autres cas un marcheur au contact colle ou est abandonné avant de marcher. Après un pas refusé, le
//!       marcheur est encore au contact : le test 2.1 a lieu de nouveau, avec un **nouveau** tirage `suivant()`.)
//!
//! Quand le marcheur colle en `(x, y)`, c'est le nouveau site (à la fin de [`AmasVariante::sites`]), sauf si `(x, y)` est hors du
//! réseau. Un site rejoint toujours l'amas à la position où le marcheur a collé, jamais à la précédente.
//!
//! **Les valeurs neutres redonnent `dla`** : avec `Collage(TOUJOURS)` ou `Bruit(1)`, aucun tirage n'est ajouté, aucun pas n'est refusé,
//! aucun marcheur n'est abandonné de plus : les sites, leur ordre, le nombre de tirages et l'état du générateur sont ceux de
//! [`crate::dla::Amas`] pour la même graine et le même `demi_cote`, appel par appel (y compris le moment où la capacité est dépassée).
//!
//! # La capacité du réseau
//!
//! Comme dans [`crate::dla`] : le réseau est le carré `|x|, |y| <= demi_cote` (au plus [`crate::dla::DEMI_COTE_MAX`]) ; une particule qui
//! colle hors du réseau n'est PAS ajoutée : [`AmasVariante::ajouter`] **panique** (message contenant « capacité ») et
//! [`AmasVariante::essayer_ajouter`] rend `None` (le générateur a avancé de tout ce que la particule a consommé, les sites sont inchangés,
//! et les compteurs que les marcheurs abandonnés de cet appel avaient déjà incrémentés le restent : un appel suivant repart de cet état). Avec `Bruit(m)`, **un contact hors du réseau est déjà un dépassement** : il rend `None` tout de
//! suite, quel que soit `m` (le compteur d'un site hors du réseau n'existe pas). La capacité ne change **rien** à la dynamique tant
//! qu'elle n'est pas dépassée : deux amas de même graine et de même variante, de `demi_cote` différents (assez grands), ont les mêmes sites.
//!
//! # Les mesures propres au mécanisme
//!
//! [`densite`] (`N / Rg²`) et [`anisotropie`] (`S4`) : voir ces fonctions. Elles ne modifient pas [`crate::mesures`].
//!
//! # L'écriture canonique et l'empreinte
//!
//! [`AmasVariante::ecriture_canonique`] est, dans cet ordre :
//!
//! ```text
//! 8 octets    l'étiquette ASCII "GIVRE2D0"  (47 49 56 52 45 32 44 30)
//! 1 octet     la variante : 0 pour Collage, 1 pour Bruit
//! 8 octets    le paramètre, u64 petit-boutiste : p (Q0.32, de 1 à 2^32) pour Collage, m pour Bruit
//! 4 octets    demi_cote, u32 petit-boutiste
//! 4 octets    le nombre de sites n, u32 petit-boutiste
//! 8 n octets  pour chaque site, dans l'ordre d'arrivée : x puis y, chacun i32 petit-boutiste (4 octets chacun)
//! (Bruit seulement)
//! 4 octets    le nombre c de sites du réseau qui ne sont PAS dans l'amas et dont le compteur n'est pas nul, u32 petit-boutiste
//! 12 c octets pour chacun d'eux, en balayant le réseau ligne par ligne (y croît de -demi_cote à demi_cote et, dans chaque ligne,
//!             x croît de -demi_cote à demi_cote) : x, y (i32 petit-boutiste) puis son compteur (u32 petit-boutiste)
//! 16 octets   Alea.octets() : l'état du générateur après la dernière particule
//! ```
//!
//! soit `41 + 8 n` octets pour `Collage` et `41 + 8 n + 4 + 12 c` pour `Bruit`. [`AmasVariante::empreinte`] est le SHA-256 en
//! hexadécimal minuscule ([`crate::empreinte::sha256_hex`]) de cette écriture.
//!
//! # Un conseil de vitesse (pas une règle : seule la dynamique ci-dessus est imposée)
//!
//! Comme pour `dla` : garder l'anneau en mémoire pour un `R` donné, tenir à jour le plus grand `x² + y²`, ranger l'amas dans un tableau
//! de booléens et, pour le bruit, les compteurs dans un tableau d'entiers de même indice
//! `(y + demi_cote) * (2 demi_cote + 1) + (x + demi_cote)`.
//!
//! # La directive `collage <p>` ou `bruit <m>` et la sortie de `lancer`
//!
//! **Le contrat de [`crate::lancer::lancer`] pour cette étape est ici**, avec [`variante_du_scenario`] (la lecture des directives),
//! [`collage_depuis_flottant`] (la conversion de `p`) et [`BRUIT_MAX`].
//!
//! Une directive `collage <p>` ou `bruit <m>` s'écrit **avec** une directive `dla <particules>` sur une `boite` (voir
//! [`crate::dla::particules`]). `lancer` fait alors croître un [`AmasVariante`] de graine `graine`, de `demi_cote =
//! dla::demi_cote_de_la_boite(largeur, hauteur)` et de la variante lue, jusqu'à `particules` sites (germe compris), et rend, **à la place** des
//! sorties de l'étape 1 :
//!
//! - **l'empreinte** : [`AmasVariante::empreinte`] de l'amas final ;
//! - **les mesures** : le CSV `particules,rayon,rayon_giration,anisotropie`, une ligne d'en-tête puis, pour les mêmes nombres `k` de
//!   particules que l'étape 1 (les puissances de 2 `2, 4, 8, …` qui sont `<= particules`, puis `particules` lui-même s'il n'y est pas
//!   déjà ; `dla 1` ne donne que la ligne `1`), la ligne `k,<rayon>,<giration>,<anisotropie>` : `<rayon>` est le rayon de l'amas arrêté
//!   à `k` sites (entier décimal), `<giration>` est `mesures::rayon_de_giration` de ses `k` premiers sites, et `<anisotropie>` est
//!   [`anisotropie`] de ces mêmes `k` sites ; les deux nombres à **six décimales** (`format!("{:.6}")`, donc `0.000000` pour le germe
//!   seul, et `-0.000000` pour un nombre négatif de valeur absolue inférieure à `5e-7`). Chaque ligne, la dernière comprise, se termine
//!   par un seul saut de ligne (pas de retour chariot) ;
//! - **l'aperçu** : le PNG (`image::ecrire_png`) de [`crate::dla::amas_colore`]`(amas.sites(), COTE_APERCU)`.
//!
//! Une `dla` sans `collage` ni `bruit` se comporte exactement comme à l'étape 1 (un `Amas`, le CSV à trois colonnes, l'étiquette
//! `GIVRE001`) : `collage 1` donne les mêmes sites mais une autre empreinte (étiquette et paramètre) et la colonne de plus.
//!
//! **Refus**, tous en `ErreurLancer::DirectiveInconnue(ligne)`, où `ligne` est la **plus petite** des lignes refusées (la première dans
//! l'ordre du fichier) ; rien d'autre n'est calculé alors. Sont refusées :
//! - une directive dont le nom n'est ni `dla`, ni `collage`, ni `bruit` ;
//! - une directive `dla` que `dla::particules` refuse (la ligne qu'elle donne) ;
//! - une directive `collage` ou `bruit` que [`variante_du_scenario`] refuse (la ligne qu'elle donne : un argument manquant ou en trop,
//!   une valeur mal écrite ou hors bornes, ou une **seconde** directive `collage` ou `bruit`, de l'une ou l'autre sorte, valide ou non) ;
//! - une directive `collage` ou `bruit` valide quand il n'y a **aucune** directive `dla` dans le scénario (la ligne de cette directive) ;
//! - sur un `hexagone`, la première directive (dans l'ordre du fichier) de nom `dla`, `collage` ou `bruit` ;
//! - un amas qui sort du réseau avant d'avoir `particules` sites (la ligne de la directive `dla`).
//!
//! **Quand ce contrat s'applique** : dès que le scénario contient au moins une directive nommée `dla`, `collage` ou `bruit` (c'est
//! l'extension de la règle de l'étape 1, où `lancer` passe à la DLA dès qu'il y a une `dla`). Une directive `surface` (étape 2s) n'est
//! alors qu'une directive au nom refusé, comme les autres (`boite 9 9`, `surface depot 8 3`, `bruit 2` : refus à la ligne 2). Un
//! scénario sans aucune directive `dla`, `collage` ni `bruit` se comporte exactement comme avant cette étape (le scénario vide de
//! l'étape 0, ou la directive `surface` de l'étape 2s).

use crate::scenario::ErreurLecture;
use crate::scenario::Scenario;

/// La probabilité de collage « toujours », en Q0.32 : `2^32` (`4294967296`). Avec `Collage(TOUJOURS)` le marcheur colle au premier
/// contact et **aucun tirage** n'a lieu : c'est la DLA de l'étape 1.
pub const TOUJOURS: u64 = 1 << 32;

/// Le plus grand `m` d'une directive `bruit` et d'un [`Variante::Bruit`] : un million (convention : un compteur tient toujours dans un
/// `u32`, et un `m` plus grand ne finirait jamais). Un `m` de 0 est refusé aussi.
pub const BRUIT_MAX: u32 = 1_000_000;

/// L'étiquette de l'écriture canonique : `"GIVRE2D0"`.
pub const ETIQUETTE: [u8; 8] = *b"GIVRE2D0";

/// La variante de la DLA et son paramètre (voir la documentation du module).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Variante {
    /// La probabilité de collage `p`, en Q0.32 : un `u64` de 1 à [`TOUJOURS`] inclus (la probabilité est `p / 2^32`). 0 est interdit.
    Collage(u64),
    /// La réduction du bruit `m` : un `u32` de 1 à [`BRUIT_MAX`] inclus. `Bruit(1)` est la DLA de l'étape 1.
    Bruit(u32),
}

/// Un amas de DLA avec une variante, en croissance (voir la documentation du module pour la dynamique, la consommation du hasard, la
/// capacité, l'écriture canonique et l'empreinte).
#[derive(Clone, Debug)]
#[allow(dead_code)]
pub struct AmasVariante {
    alea: crate::alea::Alea,
    demi_cote: u32,
    variante: Variante,
    sites: Vec<(i32, i32)>,
    occupe: Vec<bool>,
    compteurs: Vec<u32>,
    plus_grand_carre: u64,
    anneau_en_cache: (u32, Vec<(i32, i32)>),
}

impl AmasVariante {
    /// Un amas réduit au germe `(0, 0)`, de graine `graine` (`Alea::new(graine)`, aucun tirage fait), de capacité `demi_cote` et de
    /// variante `variante`.
    ///
    /// # Panique
    ///
    /// Si `demi_cote > DEMI_COTE_MAX`, si `Collage(p)` a `p == 0` ou `p > TOUJOURS`, ou si `Bruit(m)` a `m == 0` ou `m > BRUIT_MAX`
    /// (le message contient « variante » dans ces deux derniers cas).
    #[allow(unused_variables)]
    pub fn new(graine: u64, demi_cote: u32, variante: Variante) -> AmasVariante {
        assert!(
            demi_cote <= crate::dla::DEMI_COTE_MAX,
            "demi_cote {demi_cote} dépasse DEMI_COTE_MAX ({})",
            crate::dla::DEMI_COTE_MAX
        );
        match variante {
            Variante::Collage(p) => {
                assert!((1..=TOUJOURS).contains(&p), "la variante collage {p} est hors de 1..=TOUJOURS")
            }
            Variante::Bruit(m) => {
                assert!((1..=BRUIT_MAX).contains(&m), "la variante bruit {m} est hors de 1..=BRUIT_MAX")
            }
        }
        let demi_cote_usize = demi_cote as usize;
        let cote = 2 * demi_cote_usize + 1;
        let mut occupe = vec![false; cote * cote];
        occupe[demi_cote_usize * cote + demi_cote_usize] = true;
        AmasVariante {
            alea: crate::alea::Alea::new(graine),
            demi_cote,
            variante,
            sites: vec![(0, 0)],
            occupe,
            compteurs: vec![0; cote * cote],
            plus_grand_carre: 0,
            anneau_en_cache: (u32::MAX, Vec::new()),
        }
    }

    /// La variante, telle que donnée à [`AmasVariante::new`].
    pub fn variante(&self) -> Variante {
        self.variante
    }

    /// Le `demi_cote` du réseau, tel que donné à [`AmasVariante::new`].
    pub fn demi_cote(&self) -> u32 {
        self.demi_cote
    }

    /// Les sites de l'amas, dans l'ordre d'arrivée : le germe `(0, 0)` d'abord. Il y en a au moins un.
    pub fn sites(&self) -> &[(i32, i32)] {
        &self.sites
    }

    /// Vrai si `(x, y)` est un site de l'amas (faux hors du réseau, sans panique, pour toute valeur de `i32`).
    #[allow(unused_variables)]
    pub fn contient(&self, x: i32, y: i32) -> bool {
        self.indice_dans_reseau(x, y).is_some_and(|indice| self.occupe[indice])
    }

    /// Le compteur de la réduction du bruit au site `(x, y)` : le nombre de marcheurs qui ont touché ce site vide (voir « La dynamique »).
    /// Il vaut 0 pour un site de l'amas, pour un site hors du réseau, et toujours pour la variante `Collage`.
    #[allow(unused_variables)]
    pub fn compteur(&self, x: i32, y: i32) -> u32 {
        match self.indice_dans_reseau(x, y) {
            Some(indice) if !self.occupe[indice] => self.compteurs[indice],
            _ => 0,
        }
    }

    /// Le rayon de l'amas : le plus petit entier `r >= 0` tel que tous les sites vérifient `x² + y² <= r²` (comme
    /// [`crate::dla::Amas::rayon`]).
    pub fn rayon(&self) -> u32 {
        let racine = self.plus_grand_carre.isqrt();
        if racine * racine == self.plus_grand_carre { racine as u32 } else { (racine + 1) as u32 }
    }

    /// Ajoute une particule (voir « La dynamique ») et rend son site, aussi le dernier de [`AmasVariante::sites`].
    ///
    /// # Panique
    ///
    /// Si la particule colle hors du réseau (message contenant « capacité »). Aucun site n'est alors ajouté.
    pub fn ajouter(&mut self) -> (i32, i32) {
        self.essayer_ajouter().expect("la particule colle hors de la capacité du réseau")
    }

    /// Comme [`AmasVariante::ajouter`], mais rend `None` au lieu de paniquer quand la particule colle (ou touche, avec `Bruit`) hors du
    /// réseau (voir « La capacité du réseau »).
    pub fn essayer_ajouter(&mut self) -> Option<(i32, i32)> {
        let rayon_lancement = self.rayon() + crate::dla::MARGE;
        let rayon_abandon = crate::dla::FACTEUR_ABANDON * rayon_lancement;
        let carre_abandon = u64::from(rayon_abandon) * u64::from(rayon_abandon);
        if self.anneau_en_cache.0 != rayon_lancement {
            self.anneau_en_cache = (rayon_lancement, crate::dla::anneau(rayon_lancement));
        }
        let variante = self.variante;
        'relance: loop {
            let longueur = self.anneau_en_cache.1.len() as u32;
            let depart = self.alea.borne(longueur) as usize;
            let (mut x, mut y) = self.anneau_en_cache.1[depart];
            loop {
                // 2.1 Contact : l'un des quatre voisins est un site de l'amas.
                let contact =
                    crate::reseau::VOISINS4.iter().any(|&(dx, dy)| self.contient(x + dx as i32, y + dy as i32));
                if contact {
                    match variante {
                        Variante::Collage(p) => {
                            let colle = if p == TOUJOURS {
                                true
                            } else {
                                let u = self.alea.suivant();
                                u64::from(u) < p
                            };
                            if colle {
                                return self.ajouter_site(x, y);
                            }
                            // Sinon : on passe à 2.2 avec la même position, sans nouveau test de contact.
                        }
                        Variante::Bruit(m) => {
                            // Un contact hors du réseau est un dépassement : aucun compteur n'existe là.
                            let indice = self.indice_dans_reseau(x, y)?;
                            self.compteurs[indice] += 1;
                            if self.compteurs[indice] == m {
                                self.compteurs[indice] = 0;
                                return self.ajouter_site(x, y);
                            }
                            continue 'relance;
                        }
                    }
                }
                // 2.2 Abandonner.
                let carre = i64::from(x) * i64::from(x) + i64::from(y) * i64::from(y);
                if carre as u64 > carre_abandon {
                    continue 'relance;
                }
                // 2.3 Marcher ; un pas vers un site de l'amas est refusé mais son tirage est consommé.
                let direction = self.alea.borne(4) as usize;
                let (dx, dy) = crate::reseau::VOISINS4[direction];
                let (nx, ny) = (x + dx as i32, y + dy as i32);
                if !self.contient(nx, ny) {
                    x = nx;
                    y = ny;
                }
            }
        }
    }

    /// Ajoute des particules jusqu'à ce que l'amas ait `n` sites (germe compris) ; ne fait rien s'il en a déjà `n` ou plus.
    ///
    /// # Panique
    ///
    /// Si une particule colle hors du réseau (voir [`AmasVariante::ajouter`]).
    #[allow(unused_variables)]
    pub fn croitre_jusqu_a(&mut self, n: usize) {
        while self.sites.len() < n {
            self.ajouter();
        }
    }

    /// L'écriture canonique de l'état (voir « L'écriture canonique et l'empreinte »).
    pub fn ecriture_canonique(&self) -> Vec<u8> {
        let mut octets = Vec::new();
        octets.extend_from_slice(&ETIQUETTE);
        match self.variante {
            Variante::Collage(p) => {
                octets.push(0);
                octets.extend_from_slice(&p.to_le_bytes());
            }
            Variante::Bruit(m) => {
                octets.push(1);
                octets.extend_from_slice(&u64::from(m).to_le_bytes());
            }
        }
        octets.extend_from_slice(&self.demi_cote.to_le_bytes());
        octets.extend_from_slice(&(self.sites.len() as u32).to_le_bytes());
        for &(x, y) in &self.sites {
            octets.extend_from_slice(&x.to_le_bytes());
            octets.extend_from_slice(&y.to_le_bytes());
        }
        if matches!(self.variante, Variante::Bruit(_)) {
            let demi = self.demi_cote as i32;
            let cote = (2 * self.demi_cote + 1) as usize;
            let mut vides: Vec<(i32, i32, u32)> = Vec::new();
            for y in -demi..=demi {
                for x in -demi..=demi {
                    let indice = (y + demi) as usize * cote + (x + demi) as usize;
                    if !self.occupe[indice] && self.compteurs[indice] != 0 {
                        vides.push((x, y, self.compteurs[indice]));
                    }
                }
            }
            octets.extend_from_slice(&(vides.len() as u32).to_le_bytes());
            for (x, y, compteur) in vides {
                octets.extend_from_slice(&x.to_le_bytes());
                octets.extend_from_slice(&y.to_le_bytes());
                octets.extend_from_slice(&compteur.to_le_bytes());
            }
        }
        octets.extend_from_slice(&self.alea.octets());
        octets
    }

    /// L'empreinte de l'état : le SHA-256 (64 chiffres hexadécimaux minuscules) de [`AmasVariante::ecriture_canonique`].
    pub fn empreinte(&self) -> String {
        crate::empreinte::sha256_hex(&self.ecriture_canonique())
    }

    /// L'indice de `(x, y)` dans le tableau des sites, ou `None` hors du réseau.
    fn indice_dans_reseau(&self, x: i32, y: i32) -> Option<usize> {
        let demi_cote = i64::from(self.demi_cote);
        let x = i64::from(x);
        let y = i64::from(y);
        if x < -demi_cote || x > demi_cote || y < -demi_cote || y > demi_cote {
            return None;
        }
        let cote = (2 * demi_cote + 1) as usize;
        Some((y + demi_cote) as usize * cote + (x + demi_cote) as usize)
    }

    /// Ajoute le site `(x, y)` s'il tient dans le réseau ; rend `None` sinon (sans rien changer).
    fn ajouter_site(&mut self, x: i32, y: i32) -> Option<(i32, i32)> {
        let indice = self.indice_dans_reseau(x, y)?;
        self.sites.push((x, y));
        self.occupe[indice] = true;
        let carre = i64::from(x) * i64::from(x) + i64::from(y) * i64::from(y);
        self.plus_grand_carre = self.plus_grand_carre.max(carre as u64);
        Some((x, y))
    }
}

/// La conversion d'un `p` décimal en Q0.32, celle de la directive `collage` : `Some(q)` avec `q = round(p * 2^32)` (arrondi au plus
/// proche, **les demi vers le haut** : `f64::round`, la multiplication par `2^32` est exacte en `f64`), si `p` est fini et `0 < p <= 1` et
/// `q >= 1` ; `None` sinon (`p <= 0`, `-0.0`, `p > 1`, `NaN`, infini, ou `p` si petit que `q` serait 0, c'est-à-dire `p < 2^-33`).
/// Pour `p = 1` : `q = TOUJOURS`. `0.5` donne `2^31`, `0.25` donne `2^30`, `0.1` donne `429496730` (et non `429496729`), `2^-33` donne `1`
/// (un demi : arrondi vers le haut), `0.9999999999` donne `TOUJOURS` (`p * 2^32 = 4294967295,57` arrondi à `2^32`).
#[allow(unused_variables)]
pub fn collage_depuis_flottant(p: f64) -> Option<u64> {
    // `!(p > 0.0 && p <= 1.0)` refuse aussi NaN et les infinis, sans fonction mathématique.
    if !(p > 0.0 && p <= 1.0) {
        return None;
    }
    // La multiplication par 2^32 est exacte en f64 ; `round` est l'arrondi au plus proche, les demi vers le haut.
    let q = (p * 4_294_967_296.0).round();
    if q < 1.0 { None } else { Some(q as u64) }
}

/// La variante d'un scénario : la directive `collage <p>` ou `bruit <m>` (une seule des deux).
///
/// - aucune directive nommée `collage` ni `bruit` dans `scenario.directives` : `Ok(None)` ;
/// - `collage <p>` : **exactement un argument**, lu comme [`crate::scenario::Directive::flottant`] (notation décimale simple : un `-`
///   facultatif, des chiffres, facultativement un `.` et des chiffres ; pas de `+`, d'exposant, de `.5` ni de `5.`) puis converti par
///   [`collage_depuis_flottant`] : `Ok(Some(Variante::Collage(q)))` ;
/// - `bruit <m>` : **exactement un argument**, un entier lu comme [`crate::scenario::Directive::entier`] (chiffres décimaux ASCII, zéros
///   de tête permis : `bruit 05` vaut 5), compris entre 1 et [`BRUIT_MAX`] inclus : `Ok(Some(Variante::Bruit(m)))` ;
/// - toute autre chose est `Err(ErreurLecture::LigneInvalide(ligne))` avec le numéro de ligne de la **première directive fautive dans
///   l'ordre du fichier** : zéro ou plusieurs arguments, un argument mal écrit, hors bornes ou refusé par la conversion, ou une
///   **seconde** directive `collage` ou `bruit` (de l'une ou l'autre sorte, valide ou non : c'est la seconde qui est fautive).
///
/// Les autres directives (`dla` y compris) sont ignorées ici.
#[allow(unused_variables)]
pub fn variante_du_scenario(scenario: &Scenario) -> Result<Option<Variante>, ErreurLecture> {
    let mut trouvee: Option<Variante> = None;
    for directive in &scenario.directives {
        if directive.nom != "collage" && directive.nom != "bruit" {
            continue;
        }
        // Une seconde directive de variante, de l'une ou l'autre sorte, est fautive à sa ligne.
        if trouvee.is_some() {
            return Err(ErreurLecture::LigneInvalide(directive.ligne));
        }
        if directive.args.len() != 1 {
            return Err(ErreurLecture::LigneInvalide(directive.ligne));
        }
        match directive.nom.as_str() {
            "collage" => {
                let p = directive.flottant(0)?;
                let q = collage_depuis_flottant(p).ok_or(ErreurLecture::LigneInvalide(directive.ligne))?;
                trouvee = Some(Variante::Collage(q));
            }
            "bruit" => {
                let m = directive.entier(0)?;
                if !(1..=u64::from(BRUIT_MAX)).contains(&m) {
                    return Err(ErreurLecture::LigneInvalide(directive.ligne));
                }
                trouvee = Some(Variante::Bruit(m as u32));
            }
            _ => unreachable!("le nom a été filtré"),
        }
    }
    Ok(trouvee)
}

/// La densité d'un amas : `N / Rg²`, avec `N = sites.len()` et `Rg²` le carré du rayon de giration de [`crate::mesures::rayon_de_giration`].
/// Avec les mêmes sommes entières exactes que cette fonction (en `i128` : `Sx`, `Sy`, `Sxx`, `Syy`, `n = N`, et
/// `num = n (Sxx + Syy) - (Sx² + Sy²)`, de sorte que `Rg² = num / n²`), la densité est `n³ / num` : le résultat est
/// `(n³ as f64) / (num as f64)` (`n³` calculé en `i128`, chaque conversion arrondie au plus proche, au pair en cas d'égalité, puis une
/// seule division), le même flottant sur toutes les machines. Un disque plein de rayon `R` en a une d'environ `2 π` (`N = π R²`,
/// `Rg² = R² / 2`) ; un amas de DLA de 2000 sites, environ `1,3`.
///
/// # Panique
///
/// Si `sites` est vide ou si `num` vaut 0 (le rayon de giration est nul : un site seul, ou des sites tous confondus).
pub fn densite(sites: &[(i32, i32)]) -> f64 {
    assert!(!sites.is_empty(), "aucun site : la densité n'existe pas");
    let n = sites.len() as i128;
    let mut somme_x = 0_i128;
    let mut somme_y = 0_i128;
    let mut somme_xx = 0_i128;
    let mut somme_yy = 0_i128;
    for &(x, y) in sites {
        let (x, y) = (x as i128, y as i128);
        somme_x += x;
        somme_y += y;
        somme_xx += x * x;
        somme_yy += y * y;
    }
    let num = n * (somme_xx + somme_yy) - (somme_x * somme_x + somme_y * somme_y);
    assert!(num != 0, "le rayon de giration est nul : la densité n'existe pas");
    ((n * n * n) as f64) / (num as f64)
}

/// L'anisotropie d'ordre 4 autour de l'**origine** `(0, 0)` (le germe, pas le centre de masse) :
/// `S4 = Σ (x⁴ - 6 x² y² + y⁴) / Σ (x² + y²)²`, les deux sommes sur tous les sites. Autrement dit, la moyenne de `cos(4θ)` des sites
/// pondérée par `r⁴` (`x⁴ - 6 x² y² + y⁴ = r⁴ cos 4θ`). Le numérateur `num` et le dénominateur `den` sont **exacts** (en `i128`), puis
/// le résultat est `(num as f64) / (den as f64)` : une seule division, le même flottant sur toutes les machines ; si `den` vaut 0 (la
/// liste est vide ou ne contient que l'origine), le résultat est `0.0`. Les valeurs : `+1` quand tous les sites sont sur les axes
/// (`y = 0` ou `x = 0`), `-1` quand ils sont tous sur les diagonales (`|x| = |y|`), de petites valeurs pour un disque ; `S4` ne change
/// ni par `x ↔ y`, ni par `x → -x`, ni par `y → -y`, et vaut le même nombre pour des sites répétés.
///
/// Exact tant que `|x|` et `|y|` restent au plus `2^24` et qu'il y a moins de `2^28` sites ; au-delà le résultat n'est pas défini
/// (une panique par débordement est possible).
pub fn anisotropie(sites: &[(i32, i32)]) -> f64 {
    let mut num = 0_i128;
    let mut den = 0_i128;
    for &(x, y) in sites {
        let (x, y) = (x as i128, y as i128);
        let (x2, y2) = (x * x, y * y);
        let r2 = x2 + y2;
        num += x2 * x2 - 6 * x2 * y2 + y2 * y2;
        den += r2 * r2;
    }
    if den == 0 { 0.0 } else { (num as f64) / (den as f64) }
}

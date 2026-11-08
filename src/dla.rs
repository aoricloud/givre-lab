//! L'agrégation limitée par diffusion de Witten et Sander (DLA) sur la grille carrée (étape 1).
//!
//! # Le modèle
//!
//! Un **germe** occupe le site `(0, 0)`. Des marcheurs sont lancés un à un sur un cercle autour de l'amas, marchent au
//! hasard sur la grille, sont abandonnés s'ils s'éloignent trop, et **collent au premier contact** : dès qu'un des quatre
//! voisins du marcheur est dans l'amas, il s'arrête et son site est ajouté à l'amas (T. A. Witten et L. M. Sander, Phys. Rev.
//! Lett. 47, 1400, 1981). Un site est un couple d'entiers `(x, y)` (`i32`), `x` vers la droite et `y` vers le bas ; le germe
//! est le centre du réseau.
//!
//! # La dynamique, à la lettre, et la consommation du hasard appel par appel
//!
//! **Cette section est la spécification** : une implémentation qui la suit donne EXACTEMENT les mêmes sites, dans le même
//! ordre, et le même état du générateur, qu'une autre. Rien n'est laissé libre.
//!
//! Le générateur est `Alea::new(graine)` (flux 54, voir [`crate::alea`]), amorcé par [`Amas::new`] sans aucun tirage. Il
//! n'est tiré que de deux façons : **un `Alea::borne(len)` par lancement** (`len` est la longueur de l'anneau, voir
//! plus bas) et **un `Alea::borne(4)` par pas de marche**. Aucun autre tirage n'existe. (Comme 4 divise 2^32,
//! `borne(4)` ne rejette jamais un tirage : il consomme exactement un `suivant()` et rend ses deux bits de poids faible ;
//! `borne(len)` peut rejeter, avec la règle de [`crate::alea::Alea::borne`], et consomme alors plus d'un `suivant()`.)
//!
//! Une **particule** (un appel de [`Amas::ajouter`]) se joue ainsi. Notations : `r` est [`Amas::rayon`] (le rayon de l'amas
//! **avant** la particule), `R = r + MARGE` le rayon de lancement, `A = FACTEUR_ABANDON * R` le rayon d'abandon (tous
//! entiers), et les quatre directions sont celles de [`crate::reseau::VOISINS4`], dans l'ordre : 0 droite `(1, 0)`, 1 bas
//! `(0, 1)`, 2 gauche `(-1, 0)`, 3 haut `(0, -1)`.
//!
//! 1. **Lancer.** `anneau` est la liste [`anneau`]`(R)` (son énumération est décrite à cet endroit : `len` points).
//!    On tire `k = alea.borne(len as u32)` et le marcheur part de `anneau[k]`.
//! 2. **Boucle**, avec la position courante `(x, y)` :
//!    1. **Coller** : si l'un des quatre voisins `(x + dx, y + dy)` de la position courante est un site de l'amas, le
//!       marcheur s'arrête **à sa position courante** `(x, y)` (pas à sa position précédente) : c'est le nouveau site (voir
//!       plus bas pour la capacité), ajouté **à la fin** de la liste des sites ; la particule est finie. Seuls les quatre
//!       voisins comptent, jamais les diagonales. Ce test précède tout pas, y compris le premier (il ne peut pas réussir au
//!       départ, car l'anneau est à distance au moins `MARGE` de l'amas) ;
//!    2. **Abandonner** : sinon, si `x² + y² > A²` (strictement : sur le cercle d'abandon, le marcheur continue), le marcheur
//!       est abandonné : on revient à l'étape 1 avec le **même** `R` (l'amas n'a pas changé) et un **nouveau** tirage
//!       `borne(len)` ; les pas déjà faits sont perdus ;
//!    3. **Marcher** : sinon on tire `d = alea.borne(4)`, la position devient `(x + dx, y + dy)` pour la direction `d`, et
//!       on retourne à l'étape 2.1.
//!
//! Le marcheur peut sortir du réseau (il n'est limité que par `A`) ; seul un site **ajouté** doit tenir dans le réseau.
//! Un marcheur ne marche jamais sur un site de l'amas : il aurait collé avant (étape 2.1). Les sites sont donc distincts.
//!
//! # La capacité du réseau
//!
//! Le réseau est le carré des sites `(x, y)` avec `|x| <= demi_cote` et `|y| <= demi_cote` (`demi_cote` est donné à
//! [`Amas::new`], au plus [`DEMI_COTE_MAX`]). Si la particule collait hors de ce carré, elle n'est PAS ajoutée :
//! [`Amas::ajouter`] **panique** (message contenant « capacité »), et [`Amas::essayer_ajouter`] rend `None` (le générateur a
//! avancé de tout ce que la particule a consommé, les sites sont inchangés). La capacité ne change **rien** à la dynamique
//! tant qu'elle n'est pas dépassée : deux amas de même graine et de `demi_cote` différents (assez grands) ont les mêmes
//! sites.
//!
//! # Le rayon de l'amas
//!
//! [`Amas::rayon`] est le plus petit entier `r >= 0` tel que **tous** les sites vérifient `x² + y² <= r²` (le germe seul :
//! 0). C'est `ceil(sqrt(max (x² + y²)))`, en entiers.
//!
//! # L'écriture canonique et l'empreinte
//!
//! [`Amas::ecriture_canonique`] est, dans cet ordre :
//!
//! ```text
//! 8 octets    l'étiquette ASCII "GIVRE001"  (47 49 56 52 45 30 30 31)
//! 4 octets    demi_cote, u32 petit-boutiste
//! 4 octets    le nombre de sites n, u32 petit-boutiste
//! 8 n octets  pour chaque site, dans l'ordre d'arrivée : x puis y, chacun i32 petit-boutiste (4 octets chacun)
//! 16 octets   Alea.octets() : l'état du générateur après la dernière particule (voir Alea::octets)
//! ```
//!
//! soit `32 + 8 n` octets. [`Amas::empreinte`] est le SHA-256 en hexadécimal minuscule ([`crate::empreinte::sha256_hex`])
//! de cette écriture.
//!
//! # Un conseil de vitesse (pas une règle : seule la dynamique ci-dessus est imposée)
//!
//! Les tests jouent 40 amas de 4000 sites et un amas de 3000 sites ; le prototype fait 4000 sites en un tiers de seconde. Ce qui compte :
//! l'anneau ne change que quand le rayon de l'amas change (le garder en mémoire pour un `R` donné, au lieu de le recalculer à chaque
//! lancement : le recalculer coûte `(2R + 1)²` pas à chaque lancement) ; le plus grand `x² + y²` se tient à jour à chaque site ajouté ; les
//! sites se retrouvent par un tableau de booléens indexé par `(y + demi_cote) * (2 demi_cote + 1) + (x + demi_cote)` (un marcheur hors du
//! tableau n'a pas de voisin dans l'amas). Un anneau recalculé et un ensemble ordonné donnent exactement les mêmes sites, plus lentement.
//!
//! # La directive `dla` et la sortie de `lancer`
//!
//! **Cette section est le contrat de [`crate::lancer::lancer`] à l'étape 1** (la doc de `src/lancer.rs` la reprend). Une directive
//! `dla <particules>` (lue par [`particules`]) sur une `boite largeur hauteur` lance une DLA : `lancer` fait croître un [`Amas`]
//! de graine `graine` et de `demi_cote = demi_cote_de_la_boite(largeur, hauteur)`, par des appels de [`Amas::essayer_ajouter`]
//! (jamais [`Amas::ajouter`], qui paniquerait), jusqu'à `particules` sites (germe compris), et rend, **à la place** des sorties
//! du scénario vide de l'étape 0 :
//!
//! - **l'empreinte** : [`Amas::empreinte`] de l'amas final (deux boîtes de même `demi_cote` ont donc la même empreinte) ;
//! - **les mesures** : le CSV `particules,rayon,rayon_giration`, la ligne d'en-tête puis une ligne par nombre `k` de sites, dans
//!   cet ordre croissant : les puissances de 2 `2, 4, 8, …` qui sont `<= particules`, puis `particules` lui-même s'il n'est pas
//!   déjà dans la liste (jamais deux fois la même ligne ; `dla 1` ne donne que la ligne `1`). La ligne `k` est
//!   `k,<rayon>,<giration>` : `<rayon>` est [`Amas::rayon`] de l'amas quand il avait `k` sites (en décimal), `<giration>` est
//!   [`crate::mesures::rayon_de_giration`] de ses `k` premiers sites, écrit avec **six décimales** (`format!("{:.6}", g)` :
//!   `0.000000` pour le germe seul). Chaque ligne, la dernière comprise, finit par un seul `\n` (pas de `\r`) ;
//! - **l'aperçu** : le PNG ([`crate::image::ecrire_png`]) de [`amas_colore`]`(amas.sites(), COTE_APERCU)`.
//!
//! **Les refus**, tous `ErreurLancer::DirectiveInconnue(ligne)` (l'énumération de l'étape 0 n'a pas d'autre variante ; ni elle
//! ni `src/main.rs` ne changent), où `ligne` est la **plus petite** des lignes refusées : une directive dont le nom n'est pas
//! `dla` (sa ligne) ; une directive `dla` que [`particules`] refuse (la ligne que `particules` rend) ; une directive `dla` sur
//! un `hexagone` (la ligne de la première `dla`). Puis, si rien n'est refusé, un amas qui sort du réseau avant d'avoir
//! `particules` sites (`essayer_ajouter` rend `None`) : la ligne de la directive `dla`. Rien n'est rendu d'autre dans ces cas.
//! Un scénario sans directive `dla` (et sans autre directive) se comporte exactement comme à l'étape 0.
//!
//! L'aperçu est [`amas_colore`], avec la palette [`PALETTE`], le fond [`FOND`] et [`couleur_du_rang`].

use crate::scenario::ErreurLecture;
use crate::scenario::Scenario;

/// La marge de lancement : le marcheur part d'un anneau de rayon `R = rayon de l'amas + MARGE` (5).
pub const MARGE: u32 = 5;

/// Le facteur du rayon d'abandon : un marcheur est abandonné quand `x² + y² > (FACTEUR_ABANDON * R)²` (2).
///
/// `MARGE` et `FACTEUR_ABANDON` suivent la convention des premières simulations sur grille (lancement à « rayon de l'amas + 5 »,
/// abandon dès deux fois le rayon : Y. L. Loh, arXiv:1407.2586, 2014, section I, qui cite Witten et Sander 1981 et 1983). Un abandon
/// suivi d'un nouveau tirage uniforme sur l'anneau n'est pas le retour exact du marcheur : c'est une approximation connue du modèle.
pub const FACTEUR_ABANDON: u32 = 2;

/// Le nombre maximal de particules d'une directive `dla` (un million, germe compris).
pub const PARTICULES_MAX: usize = 1_000_000;

/// Le plus grand `demi_cote` d'un réseau : 2047, soit une boîte de 4095 sites de côté (`scenario::COTE_MAX` est 4096).
pub const DEMI_COTE_MAX: u32 = 2047;

/// Le côté visé, en pixels, de l'aperçu de `lancer` (512) : voir [`amas_colore`].
pub const COTE_APERCU: u32 = 512;

/// La couleur de fond de l'aperçu, `(R, V, B)` : `(13, 17, 23)`.
pub const FOND: [u8; 3] = [13, 17, 23];

/// Les cinq couleurs-repères de l'aperçu, `(R, V, B)`, du premier site arrivé au dernier : blanc `(255, 255, 255)`, glace
/// `(150, 230, 255)`, bleu `(60, 140, 255)`, violet `(140, 80, 240)`, rose `(255, 90, 160)`. Voir [`couleur_du_rang`].
pub const PALETTE: [[u8; 3]; 5] = [[255, 255, 255], [150, 230, 255], [60, 140, 255], [140, 80, 240], [255, 90, 160]];

/// L'anneau de rayon `rayon` : les points entiers `(x, y)` avec `rayon² <= x² + y² < (rayon + 1)²` (en entiers ; le cercle
/// de rayon `rayon` est dedans, celui de rayon `rayon + 1` est dehors), **énumérés ligne par ligne** : `y` croît de
/// `-rayon` à `rayon` et, dans chaque ligne, `x` croît de `-rayon` à `rayon` ; un point est gardé s'il vérifie la double
/// inégalité. (Tout point de l'anneau a `|x| <= rayon` et `|y| <= rayon`, car `x² < (rayon + 1)²`.) C'est cet ordre qui
/// fixe le flux du hasard : le marcheur part de `anneau(R)[k]` avec `k` tiré par `borne(len)`.
///
/// `anneau(0)` est `[(0, 0)]` ; `anneau(1)` est `[(-1, -1), (0, -1), (1, -1), (-1, 0), (1, 0), (-1, 1), (0, 1), (1, 1)]`.
/// Le nombre de points est d'ordre `2π · rayon`.
#[allow(unused_variables)]
pub fn anneau(rayon: u32) -> Vec<(i32, i32)> {
    let r = i64::from(rayon);
    let carre_interieur = r * r;
    let carre_exterieur = (r + 1) * (r + 1);
    let mut points = Vec::new();
    for y in -r..=r {
        for x in -r..=r {
            let carre = x * x + y * y;
            if carre >= carre_interieur && carre < carre_exterieur {
                points.push((x as i32, y as i32));
            }
        }
    }
    points
}

/// Un amas de DLA en croissance (voir la documentation du module pour la dynamique, la consommation du hasard, la capacité,
/// l'écriture canonique et l'empreinte).
#[derive(Clone, Debug)]
#[allow(dead_code)]
pub struct Amas {
    alea: crate::alea::Alea,
    demi_cote: u32,
    sites: Vec<(i32, i32)>,
    occupe: Vec<bool>,
    plus_grand_carre: u64,
    anneau_en_cache: (u32, Vec<(i32, i32)>),
}

impl Amas {
    /// Un amas réduit au germe `(0, 0)`, de graine `graine` (`Alea::new(graine)`, aucun tirage fait) et de capacité
    /// `demi_cote` (voir « La capacité du réseau »).
    ///
    /// # Panique
    ///
    /// Si `demi_cote > DEMI_COTE_MAX`.
    #[allow(unused_variables)]
    pub fn new(graine: u64, demi_cote: u32) -> Amas {
        assert!(demi_cote <= DEMI_COTE_MAX, "demi_cote {demi_cote} dépasse DEMI_COTE_MAX ({DEMI_COTE_MAX})");
        let demi_cote_usize = demi_cote as usize;
        let cote = 2 * demi_cote_usize + 1;
        let mut occupe = vec![false; cote * cote];
        occupe[demi_cote_usize * cote + demi_cote_usize] = true;
        Amas {
            alea: crate::alea::Alea::new(graine),
            demi_cote,
            sites: vec![(0, 0)],
            occupe,
            plus_grand_carre: 0,
            anneau_en_cache: (u32::MAX, Vec::new()),
        }
    }

    /// Le `demi_cote` du réseau, tel que donné à [`Amas::new`].
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

    /// Le rayon de l'amas : le plus petit entier `r >= 0` tel que tous les sites vérifient `x² + y² <= r²` (voir « Le rayon
    /// de l'amas »).
    pub fn rayon(&self) -> u32 {
        let racine = self.plus_grand_carre.isqrt();
        if racine * racine == self.plus_grand_carre { racine as u32 } else { (racine + 1) as u32 }
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

    /// Ajoute une particule (voir « La dynamique ») et rend son site, aussi le dernier de [`Amas::sites`].
    ///
    /// # Panique
    ///
    /// Si la particule colle hors du réseau (message contenant « capacité »). Aucun site n'est alors ajouté.
    pub fn ajouter(&mut self) -> (i32, i32) {
        self.essayer_ajouter().expect("la particule colle hors de la capacité du réseau")
    }

    /// Comme [`Amas::ajouter`], mais rend `None` au lieu de paniquer quand la particule colle hors du réseau (voir « La
    /// capacité du réseau »).
    pub fn essayer_ajouter(&mut self) -> Option<(i32, i32)> {
        let rayon_lancement = self.rayon() + MARGE;
        let rayon_abandon = FACTEUR_ABANDON * rayon_lancement;
        let carre_abandon = u64::from(rayon_abandon) * u64::from(rayon_abandon);
        if self.anneau_en_cache.0 != rayon_lancement {
            self.anneau_en_cache = (rayon_lancement, anneau(rayon_lancement));
        }
        'relance: loop {
            let longueur = self.anneau_en_cache.1.len() as u32;
            let depart = self.alea.borne(longueur) as usize;
            let (mut x, mut y) = self.anneau_en_cache.1[depart];
            loop {
                let colle = crate::reseau::VOISINS4.iter().any(|&(dx, dy)| self.contient(x + dx as i32, y + dy as i32));
                if colle {
                    let indice = self.indice_dans_reseau(x, y)?;
                    self.sites.push((x, y));
                    self.occupe[indice] = true;
                    let carre = i64::from(x) * i64::from(x) + i64::from(y) * i64::from(y);
                    self.plus_grand_carre = self.plus_grand_carre.max(carre as u64);
                    return Some((x, y));
                }
                let carre = i64::from(x) * i64::from(x) + i64::from(y) * i64::from(y);
                if carre as u64 > carre_abandon {
                    continue 'relance;
                }
                let direction = self.alea.borne(4) as usize;
                let (dx, dy) = crate::reseau::VOISINS4[direction];
                x += dx as i32;
                y += dy as i32;
            }
        }
    }

    /// Ajoute des particules jusqu'à ce que l'amas ait `n` sites (germe compris) ; ne fait rien s'il en a déjà `n` ou plus.
    /// `croitre_jusqu_a(100)` puis `croitre_jusqu_a(200)` donne les mêmes sites et le même générateur que
    /// `croitre_jusqu_a(200)` seul.
    ///
    /// # Panique
    ///
    /// Si une particule colle hors du réseau (voir [`Amas::ajouter`]).
    #[allow(unused_variables)]
    pub fn croitre_jusqu_a(&mut self, n: usize) {
        while self.sites.len() < n {
            self.ajouter();
        }
    }

    /// L'écriture canonique de l'état, `32 + 8 n` octets (voir « L'écriture canonique et l'empreinte »).
    pub fn ecriture_canonique(&self) -> Vec<u8> {
        let mut octets = Vec::with_capacity(32 + 8 * self.sites.len());
        octets.extend_from_slice(b"GIVRE001");
        octets.extend_from_slice(&self.demi_cote.to_le_bytes());
        octets.extend_from_slice(&(self.sites.len() as u32).to_le_bytes());
        for &(x, y) in &self.sites {
            octets.extend_from_slice(&x.to_le_bytes());
            octets.extend_from_slice(&y.to_le_bytes());
        }
        octets.extend_from_slice(&self.alea.octets());
        octets
    }

    /// L'empreinte de l'état : le SHA-256 (64 chiffres hexadécimaux minuscules) de [`Amas::ecriture_canonique`].
    pub fn empreinte(&self) -> String {
        crate::empreinte::sha256_hex(&self.ecriture_canonique())
    }
}

/// La directive `dla <particules>` d'un scénario : le nombre de particules, **germe compris** (`dla 1` : le germe seul).
///
/// - aucune directive nommée `dla` dans `scenario.directives` : `Ok(None)` ;
/// - sinon la directive doit avoir **exactement un argument**, un entier lu comme [`crate::scenario::Directive::entier`]
///   (chiffres décimaux ASCII seulement, sans signe, zéros de tête permis : `dla 05` vaut 5), compris entre 1 et
///   [`PARTICULES_MAX`] inclus : `Ok(Some(n))` ;
/// - toute autre chose est `Err(ErreurLecture::LigneInvalide(ligne))` avec le numéro de ligne de la **première directive
///   `dla` fautive dans l'ordre du fichier** : zéro ou plusieurs arguments, un argument mal écrit ou hors bornes, ou une
///   **seconde** directive `dla` (valide ou non : c'est la seconde qui est fautive).
///
/// Les autres directives sont ignorées ici (c'est `lancer` qui refuse celles qu'il ne comprend pas).
#[allow(unused_variables)]
pub fn particules(scenario: &Scenario) -> Result<Option<usize>, ErreurLecture> {
    let mut nombre = None;
    for directive in &scenario.directives {
        if directive.nom != "dla" {
            continue;
        }
        if nombre.is_some() {
            return Err(ErreurLecture::LigneInvalide(directive.ligne));
        }
        if directive.args.len() != 1 {
            return Err(ErreurLecture::LigneInvalide(directive.ligne));
        }
        let valeur = directive.entier(0)?;
        if !(1..=PARTICULES_MAX as u64).contains(&valeur) {
            return Err(ErreurLecture::LigneInvalide(directive.ligne));
        }
        nombre = Some(valeur as usize);
    }
    Ok(nombre)
}

/// Le `demi_cote` du réseau d'un amas lancé dans une `boite largeur hauteur` : `(min(largeur, hauteur) - 1) / 2` (division
/// entière). `boite 401 401` donne 200, `boite 401 300` donne 149, `boite 400 400` donne 199, `boite 1 1` donne 0. Pour les
/// bornes de `boite` (au plus 4096), le résultat est au plus [`DEMI_COTE_MAX`].
///
/// # Panique
///
/// Si `largeur` ou `hauteur` vaut 0.
#[allow(unused_variables)]
pub fn demi_cote_de_la_boite(largeur: u32, hauteur: u32) -> u32 {
    assert!(largeur > 0 && hauteur > 0, "une boîte a des côtés au moins 1");
    (largeur.min(hauteur) - 1) / 2
}

/// La couleur `(R, V, B)` du site de rang `rang` (0 pour le germe) parmi `nombre` sites, par interpolation linéaire entière
/// entre les cinq couleurs de [`PALETTE`]. Si `nombre == 1`, c'est `PALETTE[0]`. Sinon, avec `d = nombre - 1` et `s = 4 *
/// rang` (en `u64`) : le segment est `k = min(s / d, 3)` (division entière) et la fraction `f = s - k * d` (de 0 à `d`) ;
/// chaque canal vaut `(a * (d - f) + b * f + d / 2) / d` (division entière : arrondi au plus proche, demi vers le haut),
/// avec `a` et `b` le même canal de `PALETTE[k]` et `PALETTE[k + 1]`. Le rang 0 est donc `PALETTE[0]` et le rang `nombre - 1`
/// est `PALETTE[4]` ; avec 5 sites, les cinq couleurs de la palette ; avec 9 sites, le rang 1 est le milieu de `PALETTE[0]`
/// et de `PALETTE[1]`.
///
/// # Panique
///
/// Si `nombre == 0` ou `rang >= nombre`.
#[allow(unused_variables)]
pub fn couleur_du_rang(rang: usize, nombre: usize) -> [u8; 3] {
    todo!()
}

/// L'aperçu de l'amas : les sites colorés par ordre d'arrivée. Rend `(largeur, hauteur, rvb)` : l'image (carrée) et ses
/// `3 * largeur * hauteur` octets R, V, B, ligne par ligne de haut en bas (le format de `rendu` et d'`image::ecrire_png`).
///
/// - `m` est le plus grand `max(|x|, |y|)` des sites ; le côté en sites est `cote = 2m + 1` : la fenêtre est le carré
///   `[-m, m] × [-m, m]`, **centrée sur le germe** `(0, 0)` ;
/// - l'échelle est `echelle = max(1, cote_cible / cote)` (division entière) : chaque site est un bloc de
///   `echelle × echelle` pixels ; la largeur et la hauteur valent `cote * echelle` ;
/// - le pixel `(px, py)` (0 en haut à gauche) est dans le bloc du site `(px / echelle - m, py / echelle - m)` ; s'il est un
///   site de rang `i` dans `sites` (le germe : rang 0), il a la couleur [`couleur_du_rang`]`(i, sites.len())` ; sinon c'est
///   le fond [`FOND`].
///
/// Les sites sont supposés distincts. `lancer` appelle `amas_colore(amas.sites(), COTE_APERCU)`.
///
/// # Panique
///
/// Si `sites` est vide, ou si l'image serait trop grande pour un `usize`.
#[allow(unused_variables)]
pub fn amas_colore(sites: &[(i32, i32)], cote_cible: u32) -> (u32, u32, Vec<u8>) {
    todo!()
}

//! Les réseaux : la grille carrée, la grille hexagonale, la géométrie entière du disque et l'enroulement d'une bande
//! (étape 0). Tout est en entiers, sans aucun flottant : les symétries (ordre 4 sur la grille carrée, ordre 6 sur
//! l'hexagone) doivent être exactes.
//!
//! # La grille carrée
//!
//! [`Grille`] est un rectangle de `largeur × hauteur` cases. La case `(x, y)` a `0 <= x < largeur` et
//! `0 <= y < hauteur` ; `x` croît vers la droite et `y` vers le bas. Les cases sont rangées **par lignes** :
//! `index(x, y) = y * largeur + x`. Les voisinages à 4 et à 8 sont des tableaux de taille fixe, dans un ordre fixé
//! par [`VOISINS4`] et [`VOISINS8`] ; une case voisine hors de la grille donne `None` **à sa place** (le tableau ne
//! se compacte jamais) ; aucun voisinage ne s'enroule.
//!
//! # La grille hexagonale
//!
//! [`Hexagone`] est un grand hexagone de cases hexagonales, en **coordonnées axiales** `(q, r)` ; la troisième
//! coordonnée cubique est `s = -q - r`. Une case appartient à l'hexagone de rayon `R` si `max(|q|, |r|, |s|) <= R`.
//! Il y a donc `3R(R+1) + 1` cases (`R = 0` : une seule case). Les six voisins d'une case sont `(q + dq, r + dr)`
//! pour `(dq, dr)` dans [`DIRECTIONS_HEX`], dans cet ordre. La rotation d'un sixième de tour, [`tourner60`], est
//! exacte en entiers, et **tourner la direction `k` donne la direction `k + 1` (modulo 6)** : c'est ce qui fait
//! reposer sur des entiers la symétrie d'ordre 6 des flocons.
//!
//! # La géométrie du disque et de la bande
//!
//! [`dans_disque`] dit si un déplacement `(dx, dy)` est dans le disque de rayon `r` (`dx² + dy² <= r²`, calculé en
//! entiers : le bord est dans le disque). [`Grille::enrouler_x`] enroule une abscisse sur la largeur (bande
//! périodique, pour les surfaces).

/// Les quatre déplacements `(dx, dy)` des voisins à 4, dans l'ordre : droite, bas, gauche, haut (`y` croît vers le
/// bas). Chaque déplacement est le précédent tourné d'un quart de tour (dans le sens horaire de l'écran).
pub const VOISINS4: [(isize, isize); 4] = [(1, 0), (0, 1), (-1, 0), (0, -1)];

/// Les huit déplacements `(dx, dy)` des voisins à 8, dans l'ordre : droite, bas-droite, bas, bas-gauche, gauche,
/// haut-gauche, haut, haut-droite. Les quatre voisins à 4 sont aux rangs pairs (0, 2, 4, 6) de cette liste, dans
/// leur ordre.
pub const VOISINS8: [(isize, isize); 8] = [(1, 0), (1, 1), (0, 1), (-1, 1), (-1, 0), (-1, -1), (0, -1), (1, -1)];

/// Les six déplacements `(dq, dr)` des voisins d'une case hexagonale, dans l'ordre des directions 0 à 5 :
/// `(1, 0)`, `(0, 1)`, `(-1, 1)`, `(-1, 0)`, `(0, -1)`, `(1, -1)`. En coordonnées cubiques `(q, r, s)` :
/// `(1, 0, -1)`, `(0, 1, -1)`, `(-1, 1, 0)`, `(-1, 0, 1)`, `(0, -1, 1)`, `(1, -1, 0)`. La direction `k + 1` est la
/// direction `k` passée par [`tourner60`] (et la direction 0 est la direction 5 tournée).
pub const DIRECTIONS_HEX: [(i32, i32); 6] = [(1, 0), (0, 1), (-1, 1), (-1, 0), (0, -1), (1, -1)];

/// Une grille carrée de `largeur × hauteur` cases (voir la documentation du module). Les dimensions sont au moins 1.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[allow(dead_code)]
pub struct Grille {
    largeur: usize,
    hauteur: usize,
}

impl Grille {
    /// Une grille de `largeur` colonnes et `hauteur` lignes.
    ///
    /// # Panique
    ///
    /// Si `largeur` ou `hauteur` vaut 0, ou si `largeur * hauteur` ne tient pas dans un `usize`.
    #[allow(unused_variables)]
    pub fn new(largeur: usize, hauteur: usize) -> Grille {
        assert!(largeur > 0 && hauteur > 0, "les dimensions doivent être au moins 1");
        assert!(largeur.checked_mul(hauteur).is_some(), "largeur * hauteur déborde un usize");
        Grille { largeur, hauteur }
    }

    /// Le nombre de colonnes.
    pub fn largeur(&self) -> usize {
        self.largeur
    }

    /// Le nombre de lignes.
    pub fn hauteur(&self) -> usize {
        self.hauteur
    }

    /// Le nombre de cases : `largeur * hauteur`.
    pub fn cases(&self) -> usize {
        self.largeur * self.hauteur
    }

    /// Vrai si `(x, y)` est une case de la grille : `0 <= x < largeur` et `0 <= y < hauteur`. Les coordonnées
    /// signées permettent de demander une case voisine sans risque de débordement.
    #[allow(unused_variables)]
    pub fn contient(&self, x: isize, y: isize) -> bool {
        x >= 0 && y >= 0 && (x as usize) < self.largeur && (y as usize) < self.hauteur
    }

    /// L'indice de la case `(x, y)` dans le rangement par lignes : `y * largeur + x`.
    ///
    /// # Panique
    ///
    /// Si `x >= largeur` ou `y >= hauteur` (une abscisse trop grande ne glisse pas sur la ligne suivante).
    #[allow(unused_variables)]
    pub fn index(&self, x: usize, y: usize) -> usize {
        assert!(x < self.largeur && y < self.hauteur, "case hors de la grille");
        y * self.largeur + x
    }

    /// La case d'indice `i` : l'inverse de [`Grille::index`], `(i % largeur, i / largeur)`.
    ///
    /// # Panique
    ///
    /// Si `i >= cases()`.
    #[allow(unused_variables)]
    pub fn position(&self, i: usize) -> (usize, usize) {
        assert!(i < self.cases(), "indice hors de la grille");
        (i % self.largeur, i / self.largeur)
    }

    /// Les quatre voisins de `(x, y)`, dans l'ordre de [`VOISINS4`] : l'élément `k` est
    /// `Some((x + dx, y + dy))` pour `(dx, dy) = VOISINS4[k]` si cette case est dans la grille, `None` sinon. Sans
    /// enroulement, sans allocation.
    ///
    /// # Panique
    ///
    /// Si `(x, y)` n'est pas dans la grille.
    #[allow(unused_variables)]
    pub fn voisins4(&self, x: usize, y: usize) -> [Option<(usize, usize)>; 4] {
        assert!(x < self.largeur && y < self.hauteur, "case hors de la grille");
        let mut voisins = [None; 4];
        for (k, &(dx, dy)) in VOISINS4.iter().enumerate() {
            if let (Some(nx), Some(ny)) = (x.checked_add_signed(dx), y.checked_add_signed(dy))
                && nx < self.largeur
                && ny < self.hauteur
            {
                voisins[k] = Some((nx, ny));
            }
        }
        voisins
    }

    /// Les huit voisins de `(x, y)`, dans l'ordre de [`VOISINS8`], avec les mêmes règles que
    /// [`Grille::voisins4`].
    ///
    /// # Panique
    ///
    /// Si `(x, y)` n'est pas dans la grille.
    #[allow(unused_variables)]
    pub fn voisins8(&self, x: usize, y: usize) -> [Option<(usize, usize)>; 8] {
        assert!(x < self.largeur && y < self.hauteur, "case hors de la grille");
        let mut voisins = [None; 8];
        for (k, &(dx, dy)) in VOISINS8.iter().enumerate() {
            if let (Some(nx), Some(ny)) = (x.checked_add_signed(dx), y.checked_add_signed(dy))
                && nx < self.largeur
                && ny < self.hauteur
            {
                voisins[k] = Some((nx, ny));
            }
        }
        voisins
    }

    /// L'abscisse enroulée sur la largeur : le reste **euclidien** de `x` par `largeur`, toujours dans
    /// `0..largeur` (`-1` donne `largeur - 1`, `largeur` donne 0). C'est l'enroulement périodique d'une bande. Pour
    /// toute valeur de `x` raisonnable (`|x| < 2^31`).
    #[allow(unused_variables)]
    pub fn enrouler_x(&self, x: isize) -> usize {
        let reste = x.unsigned_abs() % self.largeur;
        if x < 0 && reste != 0 { self.largeur - reste } else { reste }
    }
}

/// Vrai si le déplacement `(dx, dy)` est dans le disque de rayon `rayon` : `dx² + dy² <= rayon²`, en entiers (le bord
/// est dans le disque : `dans_disque(3, 4, 5)` est vrai, `dans_disque(4, 4, 5)` est faux). Le rayon 0 ne contient
/// que `(0, 0)`. Exact pour toutes les valeurs de `i32` et de `u32`, sans débordement.
#[allow(unused_variables)]
pub fn dans_disque(dx: i32, dy: i32, rayon: u32) -> bool {
    let dx = i64::from(dx).unsigned_abs();
    let dy = i64::from(dy).unsigned_abs();
    let rayon = u64::from(rayon);
    dx * dx + dy * dy <= rayon * rayon
}

/// Le nombre de cases des rangées strictement avant la rangée `r`, pour un hexagone de rayon `rayon`.
fn cases_avant(rayon: i64, r: i64) -> i64 {
    if r >= 0 {
        rayon * (3 * rayon + 1) / 2 + r * (2 * rayon + 1) - r * (r - 1) / 2
    } else {
        (rayon + r) * (2 * rayon + 1) - (rayon - r + 1) * (rayon + r) / 2
    }
}

/// Le grand hexagone de rayon `R` en coordonnées axiales `(q, r)` (voir la documentation du module).
///
/// # L'ordre canonique des cases
///
/// Les cases sont numérotées de 0 à `cases() - 1` **rangée par rangée, `r` croissant de `-R` à `R`, et dans chaque
/// rangée `q` croissant**. La rangée `r` contient les `q` de `max(-R, -R - r)` à `min(R, R - r)` inclus, soit
/// `2R + 1 - |r|` cases. Pour `R = 1`, l'ordre est donc `(0, -1)`, `(1, -1)`, `(-1, 0)`, `(0, 0)`, `(1, 0)`,
/// `(-1, 1)`, `(0, 1)`. L'indice de la case `(q, r)` est le nombre de cases qui la précèdent dans cet ordre.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[allow(dead_code)]
pub struct Hexagone {
    rayon: u32,
}

impl Hexagone {
    /// L'hexagone de rayon `rayon` (0 permis : une seule case, l'origine). Le rayon est au plus 2^15 = 32768, pour que
    /// `cases()` tienne dans un `usize` de 32 bits (cible `wasm32`).
    ///
    /// # Panique
    ///
    /// Si `rayon > 32768`.
    #[allow(unused_variables)]
    pub fn new(rayon: u32) -> Hexagone {
        assert!(rayon <= 32768, "le rayon doit être au plus 32768");
        Hexagone { rayon }
    }

    /// Le rayon.
    pub fn rayon(&self) -> u32 {
        self.rayon
    }

    /// Le nombre de cases : `3 * R * (R + 1) + 1`.
    pub fn cases(&self) -> usize {
        let rayon = self.rayon as usize;
        3 * rayon * (rayon + 1) + 1
    }

    /// Vrai si `(q, r)` est une case : `max(|q|, |r|, |s|) <= R` avec `s = -q - r` (le bord de l'hexagone en fait
    /// partie). Exact pour toutes les valeurs de `i32`.
    #[allow(unused_variables)]
    pub fn contient(&self, q: i32, r: i32) -> bool {
        let q = i64::from(q);
        let r = i64::from(r);
        let s = -q - r;
        q.abs().max(r.abs()).max(s.abs()) <= i64::from(self.rayon)
    }

    /// L'indice canonique de la case `(q, r)` (voir « L'ordre canonique des cases »).
    ///
    /// # Panique
    ///
    /// Si `(q, r)` n'est pas une case de l'hexagone.
    #[allow(unused_variables)]
    pub fn index(&self, q: i32, r: i32) -> usize {
        assert!(self.contient(q, r), "case hors de l'hexagone");
        let rayon = i64::from(self.rayon);
        let q = i64::from(q);
        let r = i64::from(r);
        let debut = (-rayon).max(-rayon - r);
        (cases_avant(rayon, r) + (q - debut)) as usize
    }

    /// La case d'indice `i` : l'inverse de [`Hexagone::index`].
    ///
    /// # Panique
    ///
    /// Si `i >= cases()`.
    #[allow(unused_variables)]
    pub fn position(&self, i: usize) -> (i32, i32) {
        assert!(i < self.cases(), "indice hors de l'hexagone");
        let rayon = i64::from(self.rayon);
        let indice = i as i64;
        let mut bas = -rayon;
        let mut haut = rayon + 1;
        while bas + 1 < haut {
            let milieu = bas + (haut - bas) / 2;
            if cases_avant(rayon, milieu) <= indice {
                bas = milieu;
            } else {
                haut = milieu;
            }
        }
        let debut = (-rayon).max(-rayon - bas);
        let q = debut + (indice - cases_avant(rayon, bas));
        (q as i32, bas as i32)
    }

    /// Les six voisins de `(q, r)` dans l'ordre de [`DIRECTIONS_HEX`] : l'élément `k` est
    /// `Some((q + dq, r + dr))` pour `(dq, dr) = DIRECTIONS_HEX[k]` si cette case est dans l'hexagone, `None` sinon
    /// (à sa place : le tableau ne se compacte pas). Sans allocation.
    ///
    /// # Panique
    ///
    /// Si `(q, r)` n'est pas une case de l'hexagone.
    #[allow(unused_variables)]
    pub fn voisins(&self, q: i32, r: i32) -> [Option<(i32, i32)>; 6] {
        assert!(self.contient(q, r), "case hors de l'hexagone");
        let mut voisins = [None; 6];
        for (k, &(dq, dr)) in DIRECTIONS_HEX.iter().enumerate() {
            let (nq, nr) = (q + dq, r + dr);
            if self.contient(nq, nr) {
                voisins[k] = Some((nq, nr));
            }
        }
        voisins
    }
}

/// La rotation exacte d'un sixième de tour (60 degrés) autour de l'origine, en coordonnées axiales. En coordonnées
/// cubiques `(q, r, s)` elle envoie `(q, r, s)` sur `(-r, -s, -q)` ; en axial, `(q, r)` devient `(-r, q + r)` (car
/// `-s = q + r`, et la nouvelle troisième coordonnée est `-q`). Son sens est celui de [`DIRECTIONS_HEX`] : tourner
/// `DIRECTIONS_HEX[k]` donne `DIRECTIONS_HEX[(k + 1) % 6]`. Six rotations ramènent à la case de départ ; trois
/// donnent l'opposé `(-q, -r)` ; l'origine est fixe ; un hexagone centré à l'origine est envoyé sur lui-même.
/// Exacte tant que `q + r` tient dans un `i32` (sinon, panique au débordement).
#[allow(unused_variables)]
pub fn tourner60(q: i32, r: i32) -> (i32, i32) {
    let nq = r.checked_neg().expect("la rotation déborde un i32");
    let nr = q.checked_add(r).expect("q + r déborde un i32");
    (nq, nr)
}

/// La distance hexagonale entre deux cases `(q, r)` : le nombre minimal de pas de voisin à voisin, soit
/// `max(|dq|, |dr|, |ds|)` avec `dq = q1 - q2`, `dr = r1 - r2`, `ds = -dq - dr`. Elle vaut 0 entre une case et
/// elle-même, 1 entre deux voisins, et `distance_hex((0, 0), (q, r))` est le plus petit rayon d'hexagone qui
/// contient `(q, r)`. Exacte pour `|q|` et `|r|` d'au plus 2^29 (le résultat tient alors dans un `u32`).
#[allow(unused_variables)]
pub fn distance_hex(a: (i32, i32), b: (i32, i32)) -> u32 {
    let dq = i64::from(a.0) - i64::from(b.0);
    let dr = i64::from(a.1) - i64::from(b.1);
    let ds = -dq - dr;
    dq.abs().max(dr.abs()).max(ds.abs()) as u32
}

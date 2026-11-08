//! Les mesures de l'étape 1 : la pente des moindres carrés, la dimension de comptage de boîtes, la dimension de masse-rayon
//! et le rayon de giration. Elles se calibrent d'abord sur des images dont la réponse est connue (un segment, un carré plein,
//! la fractale de Vicsek, le tapis de Sierpinski : ces images se fabriquent dans les tests, pas ici), puis servent sur
//! l'amas de [`crate::dla`]. Tout est déterministe au bit près : les sommes de comptes sont des entiers, les seules
//! fonctions de `f64` sont `+ - * /`, `sqrt` et [`crate::math::logarithme`] (jamais le `ln` du système).
//!
//! # Les conventions communes
//!
//! - Un **site** est un couple d'entiers `(x, y)` (`i32`) ; `x` croît vers la droite et `y` vers le bas (comme
//!   [`crate::reseau`]). Un **rayon** est un entier `u32` ; un **côté de boîte** est un `usize` au moins égal à 1.
//! - Une mesure qui n'a pas de sens **panique** (message libre) : trop peu de points, un rayon ou un côté nul, un comptage
//!   nul (le logarithme de 0 n'existe pas), un jeu de sites vide pour [`rayon_de_giration`]. Chaque fonction dit quand.
//!
//! # La pente des moindres carrés
//!
//! Voir [`pente`] : la formule et l'ordre exact des sommes y sont écrits, de façon que deux implémentations fidèles
//! rendent le même flottant au bit près.
//!
//! # Le comptage de boîtes
//!
//! Voir [`boites_occupees`] et [`dimension_boites`]. Pour une image de côté `3^k` et des côtés de boîte `3^j`
//! (`j = 0..k-1`), le comptage d'une fractale de côté 3 est **exactement** une puissance : la pente de `ln N(c)` contre
//! `ln(1/c)` est alors exactement l'exposant (`ln 5 / ln 3` pour la fractale de Vicsek), à l'erreur d'arrondi près.
//!
//! # La masse-rayon et le rayon de giration
//!
//! Voir [`masse_dans_rayon`], [`points_masse_rayon`], [`dimension_masse_rayon`] et [`rayon_de_giration`]. Pour un amas
//! fractal de dimension `D`, la masse dans un disque de rayon `r` croît comme `r^D`, et le rayon de giration de `N` sites
//! comme `N^(1/D)` (Witten et Sander, Phys. Rev. Lett. 47, 1400, 1981 : « le rayon de giration a un comportement en loi de
//! puissance » ; Meakin, Phys. Rev. A 27, 604, 1983 : `D = 1/β` pour `R_g ∝ N^β`).

/// Une image binaire : `largeur × hauteur` pixels, chacun allumé ou éteint, rangés **par lignes** (le pixel `(x, y)` est
/// à l'indice `y * largeur + x`, `x` vers la droite, `y` vers le bas). Une image neuve est toute éteinte.
#[derive(Clone, Debug, PartialEq, Eq)]
#[allow(dead_code)]
pub struct Bitmap {
    largeur: usize,
    hauteur: usize,
    pixels: Vec<bool>,
}

impl Bitmap {
    /// Une image éteinte de `largeur` colonnes et `hauteur` lignes.
    ///
    /// # Panique
    ///
    /// Si `largeur` ou `hauteur` vaut 0, ou si `largeur * hauteur` ne tient pas dans un `usize`.
    #[allow(unused_variables)]
    pub fn new(largeur: usize, hauteur: usize) -> Bitmap {
        assert!(largeur != 0 && hauteur != 0, "une image de {largeur} × {hauteur} n'a pas de pixel");
        let nombre = largeur.checked_mul(hauteur).expect("l'image dépasse la capacité d'un usize");
        Bitmap { largeur, hauteur, pixels: vec![false; nombre] }
    }

    /// L'image des sites `sites`, avec la position de son centre. L'image est la **boîte englobante** des sites **et de
    /// l'origine `(0, 0)`** : `x_min = min(0, plus petit x)`, `x_max = max(0, plus grand x)`, de même pour `y` ; sa largeur
    /// est `x_max - x_min + 1` et sa hauteur `y_max - y_min + 1`. Le site `(x, y)` allume le pixel
    /// `(x - x_min, y - y_min)`. Le **centre** rendu est le pixel de l'origine, `(-x_min, -y_min)` : c'est le germe d'un
    /// amas de [`crate::dla`]. Une liste vide donne une image `1 × 1` éteinte et le centre `(0, 0)`. Des sites répétés
    /// allument le même pixel.
    ///
    /// # Panique
    ///
    /// Si l'image dépasse la capacité d'un `usize` (des coordonnées d'ordre `2^31` aux deux bouts).
    #[allow(unused_variables)]
    pub fn depuis_sites(sites: &[(i32, i32)]) -> (Bitmap, (usize, usize)) {
        let mut x_min = 0_i64;
        let mut x_max = 0_i64;
        let mut y_min = 0_i64;
        let mut y_max = 0_i64;
        for &(x, y) in sites {
            let (x, y) = (x as i64, y as i64);
            x_min = x_min.min(x);
            x_max = x_max.max(x);
            y_min = y_min.min(y);
            y_max = y_max.max(y);
        }
        let largeur = usize::try_from(x_max - x_min + 1).expect("l'image dépasse la capacité d'un usize");
        let hauteur = usize::try_from(y_max - y_min + 1).expect("l'image dépasse la capacité d'un usize");
        let mut image = Bitmap::new(largeur, hauteur);
        for &(x, y) in sites {
            let px = usize::try_from(x as i64 - x_min).expect("coordonnée hors de l'image");
            let py = usize::try_from(y as i64 - y_min).expect("coordonnée hors de l'image");
            image.allumer(px, py);
        }
        let centre = (
            usize::try_from(-x_min).expect("centre hors de l'image"),
            usize::try_from(-y_min).expect("centre hors de l'image"),
        );
        (image, centre)
    }

    /// Le nombre de colonnes.
    pub fn largeur(&self) -> usize {
        self.largeur
    }

    /// Le nombre de lignes.
    pub fn hauteur(&self) -> usize {
        self.hauteur
    }

    /// Allume le pixel `(x, y)` (sans effet s'il l'est déjà).
    ///
    /// # Panique
    ///
    /// Si `x >= largeur` ou `y >= hauteur`.
    #[allow(unused_variables)]
    pub fn allumer(&mut self, x: usize, y: usize) {
        assert!(x < self.largeur && y < self.hauteur, "pixel ({x}, {y}) hors de l'image");
        self.pixels[y * self.largeur + x] = true;
    }

    /// Vrai si le pixel `(x, y)` est allumé.
    ///
    /// # Panique
    ///
    /// Si `x >= largeur` ou `y >= hauteur`.
    #[allow(unused_variables)]
    pub fn est_allume(&self, x: usize, y: usize) -> bool {
        assert!(x < self.largeur && y < self.hauteur, "pixel ({x}, {y}) hors de l'image");
        self.pixels[y * self.largeur + x]
    }

    /// Le nombre de pixels allumés.
    pub fn nombre_allumes(&self) -> usize {
        self.pixels.iter().filter(|&&allume| allume).count()
    }
}

/// La pente de la droite des moindres carrés `y = a·x + b` à travers les points `(xs[i], ys[i])`, soit
/// `a = Σ (x_i - x̄)(y_i - ȳ) / Σ (x_i - x̄)²`.
///
/// **L'ordre exact des opérations** (tout en `f64`, avec `+ - * /` seulement ; `n = xs.len()` converti en `f64`) :
/// 1. `x̄ = (x_0 + x_1 + … + x_{n-1}) / n` : la somme de gauche à droite en partant de `0.0`, puis une division ; de même
///    `ȳ` ;
/// 2. `sxy = Σ (x_i - x̄) * (y_i - ȳ)` et `sxx = Σ (x_i - x̄) * (x_i - x̄)`, chaque somme de gauche à droite en partant de
///    `0.0` ;
/// 3. la pente est `sxy / sxx`.
///
/// Une pente à deux points seulement (`(y_n - y_0) / (x_n - x_0)`) n'est pas celle-ci dès qu'il y a trois points. La
/// pente de `y` contre `x` n'est pas celle de `x` contre `y`. L'ordre des points ne change la pente qu'à l'arrondi près.
///
/// # Panique
///
/// Si `xs` et `ys` n'ont pas la même longueur, si elles ont moins de deux points, ou si `sxx` vaut `0.0` (tous les `x`
/// égaux : la pente n'existe pas).
#[allow(unused_variables)]
pub fn pente(xs: &[f64], ys: &[f64]) -> f64 {
    assert_eq!(xs.len(), ys.len(), "xs et ys n'ont pas la même longueur");
    assert!(xs.len() >= 2, "une pente demande au moins deux points");
    let n = xs.len() as f64;
    let mut somme_x = 0.0;
    let mut somme_y = 0.0;
    for k in 0..xs.len() {
        somme_x += xs[k];
        somme_y += ys[k];
    }
    let moyenne_x = somme_x / n;
    let moyenne_y = somme_y / n;
    let mut numerateur = 0.0;
    let mut denominateur = 0.0;
    for k in 0..xs.len() {
        let dx = xs[k] - moyenne_x;
        let dy = ys[k] - moyenne_y;
        numerateur += dx * dy;
        denominateur += dx * dx;
    }
    assert!(denominateur != 0.0, "tous les x sont égaux : la pente n'existe pas");
    numerateur / denominateur
}

/// Le nombre de boîtes de côté `cote` qui contiennent au moins un pixel allumé : `N(cote)`. Les boîtes pavent le plan à
/// partir du **coin haut gauche de l'image** (le pixel `(0, 0)`) : la boîte `(i, j)` est l'ensemble des pixels
/// `(x, y)` avec `i·cote <= x < (i+1)·cote` et `j·cote <= y < (j+1)·cote` ; celles qui dépassent le bord droit ou le bord
/// bas de l'image sont **coupées** par l'image et comptent si elles contiennent un pixel allumé. Une boîte n'est jamais
/// décalée pour « coller » à l'objet : le pixel `(x, y)` est dans la boîte `(x / cote, y / cote)` (division entière).
/// `boites_occupees(image, 1)` est le nombre de pixels allumés.
///
/// # Panique
///
/// Si `cote` vaut 0.
#[allow(unused_variables)]
pub fn boites_occupees(image: &Bitmap, cote: usize) -> usize {
    assert!(cote != 0, "un côté de boîte nul");
    let colonnes = image.largeur.div_ceil(cote);
    let lignes = image.hauteur.div_ceil(cote);
    let mut occupees = vec![false; colonnes * lignes];
    let mut nombre = 0;
    for y in 0..image.hauteur {
        for x in 0..image.largeur {
            if image.est_allume(x, y) {
                let boite = (y / cote) * colonnes + x / cote;
                if !occupees[boite] {
                    occupees[boite] = true;
                    nombre += 1;
                }
            }
        }
    }
    nombre
}

/// La dimension de comptage de boîtes : la [`pente`] des points `(-ln c, ln N(c))`, un point par côté `c` de `cotes` (dans
/// l'ordre donné), avec `N(c) = boites_occupees(image, c)`. Les logarithmes sont ceux de [`crate::math::logarithme`] :
/// `x_i = -logarithme(c as f64)` et `y_i = logarithme(N(c) as f64)`. C'est bien la pente de `ln N(c)` contre `ln(1/c)` : un
/// ensemble de dimension `D` a `N(c) ∝ c^(-D)`, donc une pente `D`.
///
/// L'appelant choisit les côtés : pour une fractale de côté 3 (Vicsek, tapis de Sierpinski) dessinée sur une image de côté
/// `3^k`, les côtés `3^0, 3^1, …, 3^(k-1)` donnent la dimension exacte (voir la documentation du module).
///
/// # Panique
///
/// Si `cotes` a moins de deux éléments ou si tous sont égaux, si un côté vaut 0, ou si `N(c)` vaut 0 pour un côté (image
/// sans pixel allumé).
#[allow(unused_variables)]
pub fn dimension_boites(image: &Bitmap, cotes: &[usize]) -> f64 {
    let mut xs = Vec::with_capacity(cotes.len());
    let mut ys = Vec::with_capacity(cotes.len());
    for &cote in cotes {
        let nombre = boites_occupees(image, cote);
        assert!(nombre != 0, "l'image n'a aucun pixel allumé pour le côté {cote}");
        xs.push(-crate::math::logarithme(cote as f64));
        ys.push(crate::math::logarithme(nombre as f64));
    }
    pente(&xs, &ys)
}

/// La masse dans le disque de rayon `rayon` autour de `centre` : le nombre de sites `(x, y)` de `sites` (répétitions
/// comprises, chaque élément de la liste compte pour un) tels que `dx² + dy² <= rayon²`, avec `dx = x - centre.0` et
/// `dy = y - centre.1`, **comparaison entière** (le bord du disque est dedans : `dx² + dy² == rayon²` compte). Le rayon 0
/// ne compte que les sites égaux au centre. Exact pour toutes les valeurs de `i32` et de `u32` (calcul en `i128`).
#[allow(unused_variables)]
pub fn masse_dans_rayon(sites: &[(i32, i32)], centre: (i32, i32), rayon: u32) -> usize {
    let cx = centre.0 as i128;
    let cy = centre.1 as i128;
    let rayon_carre = (rayon as i128) * (rayon as i128);
    let mut masse = 0;
    for &(x, y) in sites {
        let dx = x as i128 - cx;
        let dy = y as i128 - cy;
        if dx * dx + dy * dy <= rayon_carre {
            masse += 1;
        }
    }
    masse
}

/// Les points de la courbe log-log de la masse-rayon : pour chaque `r` de `rayons` (dans l'ordre donné), le couple
/// `(logarithme(r as f64), logarithme(N(r) as f64))` avec `N(r) = masse_dans_rayon(sites, centre, r)`.
///
/// # Panique
///
/// Si un rayon vaut 0 ou si `N(r)` vaut 0 pour un rayon (aucun site dans le disque).
#[allow(unused_variables)]
pub fn points_masse_rayon(sites: &[(i32, i32)], centre: (i32, i32), rayons: &[u32]) -> Vec<(f64, f64)> {
    let mut points = Vec::with_capacity(rayons.len());
    for &rayon in rayons {
        assert!(rayon != 0, "un rayon nul");
        let masse = masse_dans_rayon(sites, centre, rayon);
        assert!(masse != 0, "aucun site dans le disque de rayon {rayon}");
        points.push((crate::math::logarithme(rayon as f64), crate::math::logarithme(masse as f64)));
    }
    points
}

/// La dimension de masse-rayon : la [`pente`] des points de [`points_masse_rayon`] (`ln r` en abscisse, `ln N(r)` en
/// ordonnée). Pour l'amas de [`crate::dla`], le centre est le germe `(0, 0)`.
///
/// # Panique
///
/// Dans les cas de [`points_masse_rayon`] et de [`pente`] (moins de deux rayons, ou tous égaux).
#[allow(unused_variables)]
pub fn dimension_masse_rayon(sites: &[(i32, i32)], centre: (i32, i32), rayons: &[u32]) -> f64 {
    let points = points_masse_rayon(sites, centre, rayons);
    let xs: Vec<f64> = points.iter().map(|point| point.0).collect();
    let ys: Vec<f64> = points.iter().map(|point| point.1).collect();
    pente(&xs, &ys)
}

/// Le rayon de giration de `sites` : la racine de la moyenne des carrés des distances au **centre de masse** (la moyenne des
/// sites, pas l'origine). Avec `n` sites, `Sx = Σ x`, `Sy = Σ y`, `Sxx = Σ x²`, `Syy = Σ y²` (tous **entiers exacts**, en
/// `i128`), la variance est `((Sxx + Syy) - (Sx² + Sy²) / n) / n`, soit `num / n²` avec l'entier exact
/// `num = n · (Sxx + Syy) - (Sx² + Sy²)` (toujours >= 0). Le résultat est
/// `sqrt((num as f64) / ((n * n) as f64))` : une seule division et une racine, en `f64` (`num as f64` est l'arrondi au plus
/// proche, au pair en cas d'égalité ; `sqrt` est l'arrondi correct), donc le même flottant sur toutes les machines. Le
/// rayon de giration d'un site seul est 0 ; celui de deux sites est la moitié de leur distance ; celui d'un carré plein de
/// côté `s` (les `s²` sites `(i, j)`, `0 <= i, j < s`) est `sqrt((s² - 1) / 6)`. Il ne change pas quand on translate tous
/// les sites.
///
/// # Panique
///
/// Si `sites` est vide.
#[allow(unused_variables)]
pub fn rayon_de_giration(sites: &[(i32, i32)]) -> f64 {
    assert!(!sites.is_empty(), "aucun site");
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
    let numerateur = n * (somme_xx + somme_yy) - (somme_x * somme_x + somme_y * somme_y);
    let denominateur = n * n;
    ((numerateur as f64) / (denominateur as f64)).sqrt()
}

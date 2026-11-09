//! La corrélation densité-densité `C(r)` d'un amas de sites (étape Z1), troisième mesure de sa dimension fractale après la
//! masse-rayon et le rayon de giration de [`crate::mesures`].
//!
//! # Ce que mesure `C(r)`
//!
//! Pour un amas de `n` sites, `C(r)` est la **densité moyenne autour d'un site**, vue depuis tous ses sites : le nombre de
//! paires ordonnées de sites à distance `<= r` l'un de l'autre (hors un site avec lui-même), divisé par `n` (autant de sites
//! de départ) et par le nombre de cases d'un disque plein de rayon `r`, centre exclu. C'est la **forme cumulée** : elle vaut
//! exactement 1 pour un plan plein (tous les sites occupés) et varie comme `r^(D-2)` pour un amas fractal de dimension `D` en
//! dimension 2 (le nombre de paires à distance `<= r` est `n · r^D`, le disque plein `π r²`). La pente de `ln C` contre `ln r`,
//! plus 2, donne `D`. Matsushita, Sano, Hayakawa, Honjo et Sawada (Phys. Rev. Lett. 53, 286, 1984) ont ainsi mesuré des feuilles
//! de zinc (leur résumé dit : « la corrélation densité-densité des motifs numérisés des photographies » ; la forme exacte de leur
//! `C(r)`, par couronne ou cumulée, n'a pas été lue) ; la forme cumulée a la même pente et est plus lisse sur une grille carrée.
//!
//! # Les conventions
//!
//! - Un **site** est un couple d'entiers `(x, y)` (`i32`), comme dans [`crate::mesures`]. Une **liste de sites** est une
//!   tranche `&[(i32, i32)]` ; son ordre ne compte pour rien dans les résultats ci-dessous (les comptes sont des entiers). Une
//!   **répétition** n'est pas écartée : chaque élément de la liste compte pour un site (donc pour un dans `n`).
//! - Un **rayon** est un entier `u32`. Les distances se comparent **au carré, en entiers** : jamais de racine ni de flottant
//!   pour décider si une paire est « à distance `<= r` » (`dx² + dy² <= r²`, l'égalité compte : le bord du disque est dedans).
//! - Un calcul qui n'a pas de sens **panique** (message libre) ; chaque fonction dit quand. Un rayon ne dépasse jamais
//!   [`RAYON_MAX`] (un tableau de `(rayon + 1)²` entrées est alloué), sauf pour [`sites_du_disque`] (voir [`RAYON_DISQUE_MAX`]).
//! - Tout est déterministe au bit près : les comptes sont des entiers (`u64`, `u128`), les seules fonctions de `f64` sont
//!   `+ - * /`, `sqrt` et [`crate::math::logarithme`] (jamais le `ln` du système).
//!
//! # La fenêtre d'ajustement
//!
//! La pente se mesure sur une **fenêtre de rayons fixée par une formule qui ne dépend que de l'amas** (jamais du résultat) : voir
//! [`etendue`], [`fenetre`] et [`mesure`]. Elle ne se règle pas sur la valeur de `D` que l'on voudrait trouver.
//!
//! # Le coût
//!
//! Compter les paires est en `O(n²)` (une double boucle) : de l'ordre de 4,5·10⁸ paires pour 30 000 sites. La page ne le fait
//! qu'à la demande, jamais à chaque image.

/// Le plus grand rayon accepté par [`paires_par_distance2`], [`correlation`], [`dimension_correlation`] et [`mesure`] : 4096
/// (un tableau de `4097² = 16 785 409` entiers de 64 bits au plus).
pub const RAYON_MAX: u32 = 4096;

/// Le plus grand rayon accepté par [`sites_du_disque`] : `2^20 = 1 048 576` (son coût est de l'ordre du rayon).
pub const RAYON_DISQUE_MAX: u32 = 1 << 20;

/// Le premier rayon de la fenêtre d'ajustement : 4 (voir [`fenetre`]).
pub const FENETRE_DEBUT: u32 = 4;

/// Le diviseur de l'étendue qui donne le dernier rayon de la fenêtre d'ajustement : 8 (voir [`fenetre`]).
pub const FENETRE_DIVISEUR: u32 = 8;

/// Le nombre minimal de rayons de la fenêtre : 6. Un amas dont la fenêtre en aurait moins n'est pas mesuré (voir [`fenetre`]).
pub const FENETRE_MIN_RAYONS: u32 = 6;

/// Le nombre de couples d'entiers `(dx, dy)` tels que `dx² + dy² <= rayon²`, **le centre `(0, 0)` compris** : le nombre de cases du
/// disque plein de rayon `rayon`, bord compris (le bord, `dx² + dy² == rayon²`, est dedans). Exact, en entiers, pour tout
/// `rayon <= RAYON_DISQUE_MAX` : `1, 5, 13, 29, 49, 81, 113, 149, 197, 253, 317, 377, 441` pour `rayon = 0..=12` (la suite A000328 de
/// l'OEIS), environ `π · rayon²` en général (le résultat dépasse `u32::MAX` dès `rayon = 36 975` : il se calcule et se rend en
/// `u64`).
///
/// # Panique
///
/// Si `rayon > RAYON_DISQUE_MAX`.
pub fn sites_du_disque(rayon: u32) -> u64 {
    assert!(rayon <= RAYON_DISQUE_MAX, "rayon {rayon} au-delà de RAYON_DISQUE_MAX");
    let rayon = u64::from(rayon);
    let rayon_carre = rayon * rayon;
    // Colonne `dx = 0` : les `dy` de `-rayon` à `rayon`.
    let mut nombre = 2 * crate::math::racine_entiere(rayon_carre) + 1;
    // Colonnes `dx > 0` : elles vont par paires `±dx`, de même hauteur.
    for dx in 1..=rayon {
        let reste = rayon_carre - dx * dx;
        nombre += 2 * (2 * crate::math::racine_entiere(reste) + 1);
    }
    nombre
}

/// Les nombres de paires de sites par distance au carré : à l'indice `k`, de `0` à `rayon_max²` inclus (le tableau a donc
/// `rayon_max² + 1` éléments), le nombre de **paires ordonnées** `(i, j)` de **positions** de `sites` avec `i != j` dont la distance au
/// carré `(x_i - x_j)² + (y_i - y_j)²` vaut **exactement** `k`. Une paire de deux sites distincts compte donc **deux fois** (`(i, j)` et
/// `(j, i)`), à la même distance ; un site n'est jamais apparié à lui-même (`i == j` est exclu), mais deux éléments de la liste égaux
/// (une répétition) forment deux paires ordonnées à la distance au carré 0, comptées à l'indice 0. Les paires à distance au carré
/// supérieure à `rayon_max²` ne sont comptées nulle part. Les comptes sont exacts pour toutes les valeurs de `i32` (les différences
/// se font sans débordement : des sites en `i32::MIN` et `i32::MAX` sont simplement trop éloignés pour compter).
///
/// Une liste vide ou d'un seul site donne un tableau de zéros ; `rayon_max = 0` donne un tableau d'un élément (les répétitions).
/// La somme de tous les éléments, avec `rayon_max` assez grand, est `n · (n - 1)`.
///
/// # Panique
///
/// Si `rayon_max > RAYON_MAX`.
pub fn paires_par_distance2(sites: &[(i32, i32)], rayon_max: u32) -> Vec<u64> {
    assert!(rayon_max <= RAYON_MAX, "rayon_max {rayon_max} au-delà de RAYON_MAX");
    let rayon_max = u64::from(rayon_max);
    let limite = rayon_max * rayon_max;
    let mut paires = vec![0_u64; limite as usize + 1];
    // Une paire de positions distinctes compte deux fois (deux ordres) : on parcourt `i < j` et on ajoute 2.
    for (i, &(xi, yi)) in sites.iter().enumerate() {
        for &(xj, yj) in &sites[i + 1..] {
            let dx = (i64::from(xi) - i64::from(xj)).unsigned_abs();
            if dx > rayon_max {
                continue;
            }
            let dy = (i64::from(yi) - i64::from(yj)).unsigned_abs();
            if dy > rayon_max {
                continue;
            }
            let distance2 = dx * dx + dy * dy;
            if distance2 <= limite {
                paires[distance2 as usize] += 2;
            }
        }
    }
    paires
}

/// La corrélation densité-densité cumulée `C(r)` pour `r = 1, 2, …, rayon_max` : le résultat a `rayon_max` éléments, et l'élément
/// d'indice `r - 1` est `C(r)`.
///
/// **La formule et l'ordre exact des opérations.** Avec `n = sites.len()`, `paires = paires_par_distance2(sites, rayon_max)` et
/// `cumul(r) = paires[1] + paires[2] + … + paires[r²]` (entier `u64`, **sans** `paires[0]` : les répétitions ne comptent pas dans le
/// numérateur) : `C(r) = cumul(r) / (n · (sites_du_disque(r) - 1))`. Le dénominateur est l'**entier exact** `n × (sites_du_disque(r) - 1)`
/// calculé en `u128` ; le résultat est `(cumul(r) as f64) / (denominateur as f64)`, **une seule division** en `f64` (chaque
/// conversion est l'arrondi au plus proche, au pair en cas d'égalité) : deux implémentations fidèles rendent le même flottant au bit
/// près. Le dénominateur dépasse `u32::MAX` dès 10 000 sites et `r = 370`. Un diviseur `n - 1` au lieu de `n`, ou `sites_du_disque(r)`
/// au lieu de `sites_du_disque(r) - 1`, donne une autre valeur.
///
/// Le cumul est croissant avec `r`. Un site seul (`n = 1`) a `C(r) = 0` pour tout `r` ; deux sites à distance `d` ont `C(r) = 0` pour
/// `r < d` puis `1 / (sites_du_disque(r) - 1)` ; un carré plein de côté `s` (les `s²` sites `(i, j)`, `0 <= i, j < s`) a, pour un
/// déplacement `(dx, dy)` non nul, `(s - |dx|)(s - |dy|)` paires ordonnées quand `|dx|` et `|dy|` sont `< s`, d'où
/// `cumul(r) = Σ (s - |dx|)(s - |dy|)` sur les déplacements non nuls de longueur `<= r`.
///
/// # Panique
///
/// Si `sites` est vide, ou si `rayon_max > RAYON_MAX`. `rayon_max = 0` rend un tableau vide (si `sites` n'est pas vide).
pub fn correlation(sites: &[(i32, i32)], rayon_max: u32) -> Vec<f64> {
    assert!(!sites.is_empty(), "aucun site");
    assert!(rayon_max <= RAYON_MAX, "rayon_max {rayon_max} au-delà de RAYON_MAX");
    if rayon_max == 0 {
        return Vec::new();
    }
    let paires = paires_par_distance2(sites, rayon_max);
    let n = sites.len() as u128;
    let mut valeurs = Vec::with_capacity(rayon_max as usize);
    let mut cumul = 0_u64;
    let mut rayon = 1_u32;
    let mut prochain_carre = 1_usize;
    for (distance2, &compte) in paires.iter().enumerate() {
        cumul += compte;
        if distance2 == prochain_carre {
            let numerateur = cumul - paires[0];
            let denominateur = n * u128::from(sites_du_disque(rayon) - 1);
            valeurs.push((numerateur as f64) / (denominateur as f64));
            rayon += 1;
            prochain_carre = (rayon as usize) * (rayon as usize);
        }
    }
    valeurs
}

/// La dimension fractale donnée par la corrélation : `2 + pente`, où `pente` est la [`crate::mesures::pente`] des points `(ln r, ln C(r))`, un point
/// par rayon `r` de `rayons` (dans l'ordre donné), avec `C(r)` de [`correlation`] et les logarithmes de [`crate::math::logarithme`] :
/// `x_i = logarithme(r_i as f64)`, `y_i = logarithme(C(r_i))`, puis `2.0 + pente(&xs, &ys)` (une addition en `f64`). Le résultat est
/// bit pour bit celui que donnerait cette formule avec les `C(r)` de [`correlation`]. Les rayons n'ont pas besoin d'être consécutifs ni
/// triés ; `rayon_max` est le plus grand rayon donné.
///
/// # Panique
///
/// Si `rayons` a moins de deux éléments ou si tous sont égaux (voir [`crate::mesures::pente`]), si un rayon vaut 0 ou dépasse [`RAYON_MAX`], si `sites`
/// est vide, ou si `C(r)` vaut 0 pour un des rayons (aucune paire à distance `<= r` : son logarithme n'existe pas).
pub fn dimension_correlation(sites: &[(i32, i32)], rayons: &[u32]) -> f64 {
    assert!(!sites.is_empty(), "aucun site");
    assert!(rayons.len() >= 2, "une dimension demande au moins deux rayons");
    for &rayon in rayons {
        assert!(rayon != 0, "un rayon nul");
        assert!(rayon <= RAYON_MAX, "rayon {rayon} au-delà de RAYON_MAX");
    }
    let rayon_max = *rayons.iter().max().expect("au moins un rayon");
    let courbe = correlation(sites, rayon_max);
    let mut xs = Vec::with_capacity(rayons.len());
    let mut ys = Vec::with_capacity(rayons.len());
    for &rayon in rayons {
        let valeur = courbe[(rayon - 1) as usize];
        assert!(valeur != 0.0, "C({rayon}) est nul : son logarithme n'existe pas");
        xs.push(crate::math::logarithme(rayon as f64));
        ys.push(crate::math::logarithme(valeur));
    }
    2.0 + crate::mesures::pente(&xs, &ys)
}

/// L'**étendue** d'une liste de sites : le plus grand des deux écarts `x_max - x_min` et `y_max - y_min` (le plus long côté de la boîte
/// qui contient les sites, en nombre de pas, pas en nombre de cases : un trait de 201 sites alignés a pour étendue 200). Une liste vide
/// ou d'un seul site : 0. Elle ne change pas quand on translate les sites, qu'on les tourne d'un quart de tour, qu'on les retourne en
/// miroir (axes) ou qu'on change leur ordre. Exacte pour toutes les valeurs de `i32` (l'écart maximal, `2³² - 1`, tient dans un `u32`).
pub fn etendue(sites: &[(i32, i32)]) -> u32 {
    let Some(&(x0, y0)) = sites.first() else {
        return 0;
    };
    let (mut x_min, mut x_max) = (i64::from(x0), i64::from(x0));
    let (mut y_min, mut y_max) = (i64::from(y0), i64::from(y0));
    for &(x, y) in sites {
        let (x, y) = (i64::from(x), i64::from(y));
        x_min = x_min.min(x);
        x_max = x_max.max(x);
        y_min = y_min.min(y);
        y_max = y_max.max(y);
    }
    // L'écart maximal, `2³² - 1`, tient dans un `u32`.
    let largeur = u32::try_from(x_max - x_min).expect("écart en abscisse hors d'un u32");
    let hauteur = u32::try_from(y_max - y_min).expect("écart en ordonnée hors d'un u32");
    largeur.max(hauteur)
}

/// La **fenêtre d'ajustement** : les rayons sur lesquels on mesure la pente, **tous les entiers** de [`FENETRE_DEBUT`] (4) à
/// `etendue(sites) / FENETRE_DIVISEUR` (division entière, 8) inclus, dans l'ordre croissant ; vide si cela fait moins de
/// [`FENETRE_MIN_RAYONS`] (6) rayons (un amas trop petit : l'étendue est alors inférieure à 72) ou si le dernier rayon, `étendue / 8`, dépasse
/// [`RAYON_MAX`] (une étendue d'au moins 32 776).
/// Elle ne dépend **que de l'étendue** de l'amas : ni de l'ordre des sites, ni de leur position, ni de ce que la mesure donne. Pour une
/// étendue `e`, elle va de 4 à `⌊e / 8⌋`.
pub fn fenetre(sites: &[(i32, i32)]) -> Vec<u32> {
    let dernier = etendue(sites) / FENETRE_DIVISEUR;
    let premier = FENETRE_DEBUT + FENETRE_MIN_RAYONS - 1;
    if !(premier..=RAYON_MAX).contains(&dernier) {
        return Vec::new();
    }
    (FENETRE_DEBUT..=dernier).collect()
}

/// Le résultat d'une [`mesure`] : la courbe `C(r)` sur la fenêtre et la dimension qu'elle donne.
#[derive(Clone, Debug, PartialEq)]
pub struct Mesure {
    /// Les rayons de la fenêtre, dans l'ordre croissant (ceux de [`fenetre`]).
    pub rayons: Vec<u32>,
    /// `C(r)` pour chaque rayon de `rayons` (même longueur, même ordre) : bit pour bit la valeur de [`correlation`].
    pub valeurs: Vec<f64>,
    /// La dimension : bit pour bit [`dimension_correlation`]`(sites, &rayons)`.
    pub dimension: f64,
}

/// La mesure complète d'un amas : la [`fenetre`], les `C(r)` sur cette fenêtre et la dimension. `None` si la fenêtre est vide (amas trop
/// petit) ou si `C(r)` est nul pour un rayon de la fenêtre (aucune paire assez proche : un amas en morceaux très éloignés, par exemple).
/// Ne panique jamais (c'est celle que la vitrine emploie). Les paires ne sont comptées qu'une fois.
pub fn mesure(sites: &[(i32, i32)]) -> Option<Mesure> {
    let rayons = fenetre(sites);
    if rayons.is_empty() {
        return None;
    }
    let rayon_max = *rayons.last().expect("une fenêtre non vide a un dernier rayon");
    let courbe = correlation(sites, rayon_max);
    let valeurs: Vec<f64> = rayons.iter().map(|&rayon| courbe[(rayon - 1) as usize]).collect();
    if valeurs.contains(&0.0) {
        return None;
    }
    let mut xs = Vec::with_capacity(rayons.len());
    let mut ys = Vec::with_capacity(rayons.len());
    for (&rayon, &valeur) in rayons.iter().zip(&valeurs) {
        xs.push(crate::math::logarithme(rayon as f64));
        ys.push(crate::math::logarithme(valeur));
    }
    let dimension = 2.0 + crate::mesures::pente(&xs, &ys);
    Some(Mesure { rayons, valeurs, dimension })
}

//! Ce que les fichiers de test de l'étape D2 partagent : un **calcul naïf** de la dimension d'un amas et de la moyenne / erreur-type d'un
//! point de la courbe, écrit d'après la seule documentation de `src/courbe.rs` (des boucles simples, rien d'optimisé, aucun appel à
//! `givre_lab::courbe` ni à `givre_lab::correlation`) ; des générateurs de formes à solution connue.
//!
//! Le naïf n'emploie que `givre_lab::claquage::Claquage` (étape D1, déjà éprouvée) pour faire croître les amas et
//! `givre_lab::math::logarithme` (étape 0) pour les logarithmes : la bibliothèque doit donner EXACTEMENT ses bits.

#![allow(dead_code)]

use givre_lab::claquage::Claquage;
use givre_lab::math::logarithme;

/// Les constantes de la documentation de `fenetre_petit_amas`, écrites ici en dur (jamais lues dans la bibliothèque).
pub const DEBUT_DOC: u32 = 2;
pub const DIVISEUR_DOC: u32 = 6;
pub const DERNIER_MIN_DOC: u32 = 4;
pub const RAYON_MAX_DOC: u32 = 4096;

/// L'étendue d'une liste de sites : le plus grand des deux écarts `x_max - x_min` et `y_max - y_min` (0 pour une liste vide).
pub fn etendue_doc(sites: &[(i32, i32)]) -> u32 {
    if sites.is_empty() {
        return 0;
    }
    let x_min = sites.iter().map(|s| i64::from(s.0)).min().unwrap();
    let x_max = sites.iter().map(|s| i64::from(s.0)).max().unwrap();
    let y_min = sites.iter().map(|s| i64::from(s.1)).min().unwrap();
    let y_max = sites.iter().map(|s| i64::from(s.1)).max().unwrap();
    (x_max - x_min).max(y_max - y_min) as u32
}

/// La fenêtre des petits amas de la documentation, pour une étendue `e` : `2..=max(4, e / 6)`, vide si ce dernier rayon dépasse `e / 2` ou 4096.
pub fn fenetre_doc(e: u32) -> Vec<u32> {
    let dernier = DERNIER_MIN_DOC.max(e / DIVISEUR_DOC);
    if dernier > e / 2 || dernier > RAYON_MAX_DOC {
        return Vec::new();
    }
    (DEBUT_DOC..=dernier).collect()
}

/// Le nombre de cases `(dx, dy)` avec `dx² + dy² <= r²`, centre compris, compté case par case.
pub fn disque_naif(r: u32) -> u64 {
    let r = i64::from(r);
    let mut nombre = 0;
    for dx in -r..=r {
        for dy in -r..=r {
            if dx * dx + dy * dy <= r * r {
                nombre += 1;
            }
        }
    }
    nombre
}

/// `C(r)` de la documentation de `correlation::correlation`, pour chaque rayon de `rayons` : les paires ordonnées `i != j` à distance au carré dans
/// `1..=r²`, divisées en une seule division de `f64` par `n * (disque(r) - 1)` (entier exact).
pub fn correlation_naive(sites: &[(i32, i32)], rayons: &[u32]) -> Vec<f64> {
    let n = sites.len() as u64;
    rayons
        .iter()
        .map(|&r| {
            let r2 = i64::from(r) * i64::from(r);
            let mut cumul = 0_u64;
            for (i, a) in sites.iter().enumerate() {
                for (j, b) in sites.iter().enumerate() {
                    if i == j {
                        continue;
                    }
                    let dx = i64::from(a.0) - i64::from(b.0);
                    let dy = i64::from(a.1) - i64::from(b.1);
                    let d2 = dx * dx + dy * dy;
                    if (1..=r2).contains(&d2) {
                        cumul += 1;
                    }
                }
            }
            let denominateur = u128::from(n) * u128::from(disque_naif(r) - 1);
            (cumul as f64) / (denominateur as f64)
        })
        .collect()
}

/// La pente des moindres carrés dans l'ordre de la documentation de `mesures::pente` : moyennes (sommes de gauche à droite puis division), puis
/// `sxy / sxx`.
pub fn pente_naive(xs: &[f64], ys: &[f64]) -> f64 {
    let n = xs.len() as f64;
    let mut sx = 0.0;
    let mut sy = 0.0;
    for k in 0..xs.len() {
        sx += xs[k];
        sy += ys[k];
    }
    let (mx, my) = (sx / n, sy / n);
    let mut sxy = 0.0;
    let mut sxx = 0.0;
    for k in 0..xs.len() {
        sxy += (xs[k] - mx) * (ys[k] - my);
        sxx += (xs[k] - mx) * (xs[k] - mx);
    }
    sxy / sxx
}

/// `D` de l'amas `sites` selon la documentation : la fenêtre de `fenetre_doc(etendue)`, `C(r)`, `2 + pente(ln r, ln C(r))` ; `None` si la fenêtre est
/// vide ou si une `C(r)` est nulle.
pub fn dimension_naive(sites: &[(i32, i32)]) -> Option<f64> {
    let rayons = fenetre_doc(etendue_doc(sites));
    if rayons.is_empty() {
        return None;
    }
    let c = correlation_naive(sites, &rayons);
    if c.contains(&0.0) {
        return None;
    }
    let xs: Vec<f64> = rayons.iter().map(|&r| logarithme(f64::from(r))).collect();
    let ys: Vec<f64> = c.iter().map(|&v| logarithme(v)).collect();
    Some(2.0 + pente_naive(&xs, &ys))
}

/// La moyenne et l'erreur-type de la documentation, en boucles simples : somme de gauche à droite, `/ n` ; variance corrigée ; `sqrt(variance / n)`.
pub fn statistiques_naives(valeurs: &[f64]) -> (f64, f64) {
    let n = valeurs.len() as f64;
    let mut somme = 0.0;
    for &v in valeurs {
        somme += v;
    }
    let moyenne = somme / n;
    let mut carres = 0.0;
    for &v in valeurs {
        carres += (v - moyenne) * (v - moyenne);
    }
    let variance = carres / (n - 1.0);
    (moyenne, (variance / n).sqrt())
}

/// Un amas de claquage de `taille` sites, ou `None` s'il touche la frontière avant.
pub fn amas_naif(graine: u64, demi_cote: u32, eta: u32, taille: usize) -> Option<Vec<(i32, i32)>> {
    let mut claquage = Claquage::new(graine, demi_cote, eta);
    while claquage.sites().len() < taille {
        claquage.essayer_ajouter()?;
    }
    Some(claquage.sites().to_vec())
}

/// La liste des `D` d'un point de la courbe, dans l'ordre des graines (chaque amas doit atteindre `taille` et avoir une dimension).
pub fn dimensions_naives(eta: u32, graines: &[u64], taille: usize, demi_cote: u32) -> Vec<f64> {
    graines
        .iter()
        .map(|&g| {
            let sites = amas_naif(g, demi_cote, eta, taille).unwrap_or_else(|| panic!("graine {g} : frontière"));
            dimension_naive(&sites).unwrap_or_else(|| panic!("graine {g} : pas de dimension"))
        })
        .collect()
}

/// Le point de la courbe selon la documentation : `(moyenne, erreur_type)` des `D` dans l'ordre des graines.
pub fn point_naif(eta: u32, graines: &[u64], taille: usize, demi_cote: u32) -> (f64, f64) {
    statistiques_naives(&dimensions_naives(eta, graines, taille, demi_cote))
}

/// Le demi-côté de `courbe` selon la documentation : `min(255, A + B * taille / 100)`, `A` et `B` écrits ici en dur.
pub fn demi_cote_doc(eta: u32, taille: usize) -> u32 {
    const A: [usize; 4] = [30, 32, 22, 12];
    const B: [usize; 4] = [10, 8, 18, 36];
    (A[eta as usize] + B[eta as usize] * taille / 100).min(255) as u32
}

// ---- les formes (les mêmes définitions que `d2_ref.py`, l'outil Python indépendant) -------------------------------------------

/// Un trait horizontal de `n` sites : `(0, 0)` à `(n - 1, 0)`.
pub fn trait_(n: i32) -> Vec<(i32, i32)> {
    (0..n).map(|x| (x, 0)).collect()
}

/// Une diagonale de `n` sites : `(0, 0)` à `(n - 1, n - 1)`.
pub fn diagonale(n: i32) -> Vec<(i32, i32)> {
    (0..n).map(|k| (k, k)).collect()
}

/// Un carré plein de `c × c` sites.
pub fn carre(c: i32) -> Vec<(i32, i32)> {
    (0..c).flat_map(|x| (0..c).map(move |y| (x, y))).collect()
}

/// Le disque plein `x² + y² <= r²`.
pub fn disque_plein(r: i32) -> Vec<(i32, i32)> {
    (-r..=r).flat_map(|x| (-r..=r).map(move |y| (x, y))).filter(|&(x, y)| x * x + y * y <= r * r).collect()
}

/// Une croix : `(k, 0)` et `(0, k)` pour `|k| <= b` (le centre une fois), `4 b + 1` sites.
pub fn croix(b: i32) -> Vec<(i32, i32)> {
    (-b..=b).map(|k| (k, 0)).chain((-b..=b).filter(|&k| k != 0).map(|k| (0, k))).collect()
}

/// La fractale de Vicsek en croix de niveau `k` : `5^k` sites, de dimension de similarité `ln 5 / ln 3`.
pub fn vicsek(k: u32) -> Vec<(i32, i32)> {
    let mut v = vec![(0, 0)];
    let mut cote = 1;
    for _ in 0..k {
        let mut suivant = Vec::new();
        for (px, py) in [(1, 1), (0, 1), (2, 1), (1, 0), (1, 2)] {
            for &(x, y) in &v {
                suivant.push((x + px * cote, y + py * cote));
            }
        }
        v = suivant;
        cote *= 3;
    }
    v
}

/// Le tapis de Sierpinski de niveau `k` : `8^k` sites, de dimension de similarité `ln 8 / ln 3`.
pub fn tapis(k: u32) -> Vec<(i32, i32)> {
    let mut v = vec![(0, 0)];
    let mut cote = 1;
    for _ in 0..k {
        let mut suivant = Vec::new();
        for px in 0..3 {
            for py in 0..3 {
                if px == 1 && py == 1 {
                    continue;
                }
                for &(x, y) in &v {
                    suivant.push((x + px * cote, y + py * cote));
                }
            }
        }
        v = suivant;
        cote *= 3;
    }
    v
}

//! Aides communes aux tests de l'étape Z1 (la corrélation densité-densité) : un compteur naïf de paires, des formes à solution connue
//! (trait, carré plein, fractale de Vicsek, tapis de Sierpinski), un petit générateur indépendant de `givre_lab::alea`, les
//! transformations du plan (translation, quart de tour, miroir, mélange) et la comparaison de flottants.
//!
//! Ce fichier n'est pas un test : il ne contient aucune fonction `#[test]`. Il est gelé avec les tests. Il n'emploie, parmi les
//! fonctions de `f64`, que `+ - * /`, `abs`, `to_bits` et `is_nan` : il ne dépend pas de `givre_lab::correlation`.
#![allow(dead_code)]

/// Le compteur NAÏF des paires de la doc de `correlation::paires_par_distance2` : une double boucle sur toutes les paires ORDONNÉES
/// de positions `(i, j)` avec `i != j`, la distance au carré en `i128`, rien d'optimisé.
pub fn paires_naives(sites: &[(i32, i32)], rayon_max: u32) -> Vec<u64> {
    let limite = i128::from(rayon_max) * i128::from(rayon_max);
    let mut paires = vec![0_u64; (limite + 1) as usize];
    for (i, &(xi, yi)) in sites.iter().enumerate() {
        for (j, &(xj, yj)) in sites.iter().enumerate() {
            if i == j {
                continue;
            }
            let dx = i128::from(xi) - i128::from(xj);
            let dy = i128::from(yi) - i128::from(yj);
            let distance2 = dx * dx + dy * dy;
            if distance2 <= limite {
                paires[distance2 as usize] += 1;
            }
        }
    }
    paires
}

/// Le nombre de cases du disque plein de rayon `r` (centre compris), compté case par case : la double boucle de la définition.
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

/// `C(r)` pour `r = 1..=rayon_max` d'après la formule de la doc de `correlation::correlation`, écrite sans rien d'astucieux à partir
/// des paires ordonnées déjà comptées : cumul entier, dénominateur entier exact en `u128`, une division.
pub fn correlation_attendue(n: usize, paires: &[u64], rayon_max: u32) -> Vec<f64> {
    (1..=rayon_max)
        .map(|r| {
            let cumul: u64 = paires[1..=(r * r) as usize].iter().sum();
            let denominateur = (n as u128) * u128::from(disque_naif(r) - 1);
            (cumul as f64) / (denominateur as f64)
        })
        .collect()
}

/// Un générateur xorshift64* (Marsaglia, Vigna) : pour des clusters au hasard, indépendant de la bibliothèque.
pub struct Hasard(pub u64);

impl Hasard {
    pub fn suivant(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.0 = x;
        x.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }

    /// Un entier de `0` à `borne - 1` (le léger biais du modulo est sans importance ici).
    pub fn borne(&mut self, borne: u64) -> u64 {
        self.suivant() % borne
    }
}

/// `n` sites au hasard dans le carré `[-demi, demi]²` (des répétitions sont possibles).
pub fn amas_au_hasard(graine: u64, n: usize, demi: i32) -> Vec<(i32, i32)> {
    let mut hasard = Hasard(graine.wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1);
    let largeur = 2 * demi as u64 + 1;
    (0..n).map(|_| (hasard.borne(largeur) as i32 - demi, hasard.borne(largeur) as i32 - demi)).collect()
}

// ------------------------------------------------------------------------------------------------ les formes

/// Les sites `(x, y)` de `[0, 3^niveau)²` dont chacun des `niveau` chiffres base 3 `(x mod 3, y mod 3)` (du plus faible au plus fort)
/// vérifie `ok`. Même construction que les images de `etape_1_mesures`, rangée par lignes (`y` d'abord).
pub fn fractale(niveau: u32, ok: impl Fn(usize, usize) -> bool) -> Vec<(i32, i32)> {
    let taille = 3_usize.pow(niveau);
    let mut sites = Vec::new();
    for y in 0..taille {
        for x in 0..taille {
            let (mut a, mut b, mut bon) = (x, y, true);
            for _ in 0..niveau {
                bon &= ok(a % 3, b % 3);
                a /= 3;
                b /= 3;
            }
            if bon {
                sites.push((x as i32, y as i32));
            }
        }
    }
    sites
}

/// La fractale de Vicsek en croix : des neuf cases d'un carré, on garde le centre et les quatre voisines par un côté (`5^niveau` sites).
pub fn vicsek_croix(niveau: u32) -> Vec<(i32, i32)> {
    fractale(niveau, |a, b| matches!((a, b), (1, 0) | (0, 1) | (1, 1) | (2, 1) | (1, 2)))
}

/// La fractale de Vicsek en X : on garde le centre et les quatre coins (`5^niveau` sites).
pub fn vicsek_x(niveau: u32) -> Vec<(i32, i32)> {
    fractale(niveau, |a, b| matches!((a, b), (0, 0) | (2, 0) | (0, 2) | (2, 2) | (1, 1)))
}

/// Le tapis de Sierpinski : on retire la case centrale (`8^niveau` sites).
pub fn tapis(niveau: u32) -> Vec<(i32, i32)> {
    fractale(niveau, |a, b| (a, b) != (1, 1))
}

/// Le carré plein de côté `s` : les `s²` sites `(i, j)`, `0 <= i, j < s`, rangés par colonnes `i` puis lignes `j`.
pub fn carre(s: i32) -> Vec<(i32, i32)> {
    (0..s).flat_map(|i| (0..s).map(move |j| (i, j))).collect()
}

/// Le trait horizontal de `longueur` sites `(i, 0)`.
pub fn trait_horizontal(longueur: i32) -> Vec<(i32, i32)> {
    (0..longueur).map(|i| (i, 0)).collect()
}

/// La diagonale de `longueur` sites `(i, i)`.
pub fn diagonale(longueur: i32) -> Vec<(i32, i32)> {
    (0..longueur).map(|i| (i, i)).collect()
}

// ------------------------------------------------------------------------------------------------ transformations

pub fn translater(sites: &[(i32, i32)], decalage: (i32, i32)) -> Vec<(i32, i32)> {
    sites.iter().map(|&(x, y)| (x + decalage.0, y + decalage.1)).collect()
}

/// Un quart de tour : `(x, y)` devient `(-y, x)`.
pub fn quart_de_tour(sites: &[(i32, i32)]) -> Vec<(i32, i32)> {
    sites.iter().map(|&(x, y)| (-y, x)).collect()
}

/// Le miroir d'axe vertical : `(x, y)` devient `(-x, y)`.
pub fn miroir(sites: &[(i32, i32)]) -> Vec<(i32, i32)> {
    sites.iter().map(|&(x, y)| (-x, y)).collect()
}

/// L'échange des axes : `(x, y)` devient `(y, x)` (un miroir d'axe diagonal).
pub fn transposer(sites: &[(i32, i32)]) -> Vec<(i32, i32)> {
    sites.iter().map(|&(x, y)| (y, x)).collect()
}

/// L'ordre des sites mélangé (Fisher-Yates avec le générateur ci-dessus, graine donnée).
pub fn melanger(sites: &[(i32, i32)], graine: u64) -> Vec<(i32, i32)> {
    let mut copie = sites.to_vec();
    let mut hasard = Hasard(graine.wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1);
    for i in (1..copie.len()).rev() {
        let j = hasard.borne(i as u64 + 1) as usize;
        copie.swap(i, j);
    }
    copie
}

// ------------------------------------------------------------------------------------------------ comparaison

/// Vérifie que `a` et `b` diffèrent d'au plus `tol` (le test échoue sinon, avec `ce_qui` dans le message).
pub fn proche(a: f64, b: f64, tol: f64, ce_qui: &str) {
    assert!((a - b).abs() <= tol, "{ce_qui} : obtenu {a}, attendu {b} (tolérance {tol})");
}

/// Les bits de chaque flottant : l'égalité « au bit près » de deux listes de `f64`.
pub fn bits(valeurs: &[f64]) -> Vec<u64> {
    valeurs.iter().map(|v| v.to_bits()).collect()
}

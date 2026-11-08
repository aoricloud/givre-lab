//! Aides communes aux tests de l'étape 3 : un simulateur NAÏF du modèle de Reiter écrit d'après la doc de `flocon` (autre
//! représentation : un dictionnaire ordonné de cases, des entiers `i128`, des arrondis écrits à la main, les voisins dans
//! un autre ordre), l'énumération des cases, les symétries de l'hexagone, quelques constructeurs de jeux de paramètres.
//!
//! Ce fichier n'est pas un test : il ne contient aucune fonction `#[test]`. Il est gelé avec les tests.
#![allow(dead_code)]

pub mod valeurs;

use givre_lab::flocon::Parametres;
use givre_lab::math::Fixe;
use std::collections::BTreeMap;

/// `Parametres::lire` sur trois décimaux (ils doivent être valides).
pub fn parametres(alpha: &str, beta: &str, gamma: &str) -> Parametres {
    Parametres::lire(alpha, beta, gamma).expect("paramètres valides")
}

/// Les cases de l'hexagone de rayon `rayon` dans l'ordre canonique de la doc de `reseau` (r croissant, puis q croissant),
/// réécrites ici : pour `r` de `-R` à `R`, `q` de `max(-R, -R - r)` à `min(R, R - r)`.
pub fn cases(rayon: i32) -> Vec<(i32, i32)> {
    let mut sortie = Vec::new();
    for r in -rayon..=rayon {
        for q in (-rayon).max(-rayon - r)..=rayon.min(rayon - r) {
            sortie.push((q, r));
        }
    }
    sortie
}

/// La distance hexagonale à l'origine : `max(|q|, |r|, |q + r|)`.
pub fn distance(q: i32, r: i32) -> i32 {
    q.abs().max(r.abs()).max((q + r).abs())
}

/// Une symétrie de l'hexagone : une fonction de `(q, r)` vers `(q, r)`.
pub type Symetrie = fn(i32, i32) -> (i32, i32);

/// Les douze symétries de l'hexagone (le groupe diédral engendré par la rotation de 60° et la réflexion `(q, r) -> (r, q)`),
/// écrites ici en toutes lettres : six rotations `(q, r) -> ...` puis ces six rotations après la réflexion.
pub fn symetries() -> Vec<(&'static str, Symetrie)> {
    vec![
        ("identité", |q, r| (q, r)),
        ("rotation 60°", |q, r| (-r, q + r)),
        ("rotation 120°", |q, r| (-q - r, q)),
        ("rotation 180°", |q, r| (-q, -r)),
        ("rotation 240°", |q, r| (r, -q - r)),
        ("rotation 300°", |q, r| (q + r, -q)),
        ("réflexion (q, r) -> (r, q)", |q, r| (r, q)),
        ("réflexion puis rotation 60°", |q, r| (-q, q + r)),
        ("réflexion puis rotation 120°", |q, r| (-q - r, r)),
        ("réflexion puis rotation 180°", |q, r| (-r, -q)),
        ("réflexion puis rotation 240°", |q, r| (q, -q - r)),
        ("réflexion puis rotation 300°", |q, r| (q + r, -r)),
    ]
}

/// Le simulateur naïf : des masses en `brut` (`i64`) dans un `BTreeMap` indexé par `(q, r)`, et le pas de la doc écrit avec
/// des entiers `i128` et des arrondis à la main (sans `Fixe`).
#[derive(Clone)]
pub struct Naif {
    pub rayon: i32,
    pub alpha: i128,
    pub beta: i128,
    pub gamma: i128,
    pub masses: BTreeMap<(i32, i32), i128>,
    pub temps: u64,
}

/// `floor(a / b + 1/2)` pour `b > 0`, en entiers : l'arrondi au plus proche, demi-cas vers `+∞`.
fn arrondi_div(a: i128, b: i128) -> i128 {
    assert!(b > 0);
    (2 * a + b).div_euclid(2 * b)
}

impl Naif {
    /// L'état initial : toutes les cases `β`, la case centrale 1 (`2^32`).
    pub fn initial(rayon: i32, p: Parametres) -> Naif {
        let mut masses = BTreeMap::new();
        for (q, r) in cases(rayon) {
            masses.insert((q, r), i128::from(p.beta.0));
        }
        masses.insert((0, 0), 1 << 32);
        Naif {
            rayon,
            alpha: i128::from(p.alpha.0),
            beta: i128::from(p.beta.0),
            gamma: i128::from(p.gamma.0),
            masses,
            temps: 0,
        }
    }

    /// Un état quelconque : `brut[i]` est la masse de la i-ième case de l'ordre canonique.
    pub fn depuis_brut(rayon: i32, p: Parametres, brut: &[i64]) -> Naif {
        let mut n = Naif::initial(rayon, p);
        let toutes = cases(rayon);
        assert_eq!(toutes.len(), brut.len());
        for (cellule, &b) in toutes.iter().zip(brut) {
            n.masses.insert(*cellule, i128::from(b));
        }
        n
    }

    fn gelee(&self, c: (i32, i32)) -> bool {
        self.masses[&c] >= 1 << 32
    }

    /// 'G', 'B' ou 'N' (gelée, de bord, non réceptive), avec les voisins qui existent seulement.
    pub fn classe(&self, c: (i32, i32)) -> char {
        if self.gelee(c) {
            return 'G';
        }
        // l'ordre des directions est volontairement différent de celui de la doc (ici : tourné de 3)
        for (dq, dr) in [(-1, 0), (0, -1), (1, -1), (1, 0), (0, 1), (-1, 1)] {
            let v = (c.0 + dq, c.1 + dr);
            if self.masses.contains_key(&v) && self.gelee(v) {
                return 'B';
            }
        }
        'N'
    }

    /// Un pas de la doc.
    pub fn pas(&mut self) {
        let classes: BTreeMap<(i32, i32), char> = self.masses.keys().map(|&c| (c, self.classe(c))).collect();
        let u = |c: (i32, i32)| -> i128 { if classes[&c] == 'N' { self.masses[&c] } else { 0 } };
        let mut nouvelles = BTreeMap::new();
        for (&(q, r), &s) in &self.masses {
            if distance(q, r) == self.rayon {
                nouvelles.insert((q, r), self.beta);
                continue;
            }
            let mut somme: i128 = 0;
            for (dq, dr) in [(-1, 0), (0, -1), (1, -1), (1, 0), (0, 1), (-1, 1)] {
                somme += u((q + dq, r + dr));
            }
            let moyenne = arrondi_div(somme, 6);
            let ecart = moyenne - u((q, r));
            // Fixe * Fixe : (a.b + 2^31) >> 32, décalage arithmétique
            let produit = (self.alpha * ecart + (1 << 31)) >> 32;
            let diffusion = arrondi_div(produit, 2);
            let nouvelle = if classes[&(q, r)] == 'N' { u((q, r)) + diffusion } else { s + self.gamma + diffusion };
            nouvelles.insert((q, r), nouvelle);
        }
        self.masses = nouvelles;
        self.temps += 1;
    }

    /// Les masses (brut) dans l'ordre canonique.
    pub fn brut(&self) -> Vec<i64> {
        cases(self.rayon).iter().map(|c| i64::try_from(self.masses[c]).expect("tient dans un i64")).collect()
    }
}

/// Les masses `brut` d'un `Fixe` en liste.
pub fn brut_de(masses: &[Fixe]) -> Vec<i64> {
    masses.iter().map(|m| m.0).collect()
}

/// La liste des masses d'une formule : `Fixe::rapport((a * i + b) % c, d)` pour la i-ème case, puis des surcharges.
pub fn masses_formule(rayon: i32, a: i64, b: i64, c: i64, d: i64, surcharges: &[((i32, i32), Fixe)]) -> Vec<Fixe> {
    let toutes = cases(rayon);
    let mut masses: Vec<Fixe> = (0..toutes.len() as i64).map(|i| Fixe::rapport((a * i + b) % c, d)).collect();
    for &(cellule, valeur) in surcharges {
        let i = toutes.iter().position(|&x| x == cellule).expect("la case existe");
        masses[i] = valeur;
    }
    masses
}

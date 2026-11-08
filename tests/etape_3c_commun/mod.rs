//! Aides communes aux tests de l'étape 3c (le bord circulaire de Reiter) : un simulateur NAÏF du modèle avec un bord au choix, écrit
//! d'après la doc de `flocon` (« Le bord », « Un pas, à la lettre »), et quelques fonctions d'énumération. Le simulateur naïf
//! calcule le bord par une AUTRE écriture que la doc : `(2q + r)² + 3r² >= 3R²` (c'est `4 · (q² + q r + r²) >= 3 R²`, car
//! `4 (q² + q r + r²) = (2q + r)² + 3 r²`), en entiers `i128`, avec un dictionnaire ordonné de cases et les voisins dans un autre
//! ordre que celui de la doc. Il réutilise `etape_3_commun` (les cases, les symétries, `parametres`, `masses_formule`) qui est
//! inchangé et gelé depuis l'étape 3.
//!
//! Ce fichier n'est pas un test : il ne contient aucune fonction `#[test]`. Il est gelé avec les tests.
#![allow(dead_code)]

pub mod valeurs;

use givre_lab::flocon::{Bord, Parametres};
use std::collections::BTreeMap;

/// `floor(a / b + 1/2)` pour `b > 0`, en entiers : l'arrondi au plus proche, demi-cas vers `+∞`.
fn arrondi_div(a: i128, b: i128) -> i128 {
    assert!(b > 0);
    (2 * a + b).div_euclid(2 * b)
}

/// La distance hexagonale à l'origine : `(|q| + |r| + |q + r|) / 2` (une autre écriture que `max(|q|, |r|, |q + r|)`).
pub fn distance_hex(q: i32, r: i32) -> i32 {
    (q.abs() + r.abs() + (q + r).abs()) / 2
}

/// Vrai si la case `(q, r)` de l'hexagone de rayon `rayon` est du bord `bord`, selon la doc (« Le bord »), écrit autrement que le code :
/// pour le bord circulaire, `(2q + r)² + 3 r² >= 3 R²` en `i128`.
pub fn du_bord(bord: Bord, rayon: i32, q: i32, r: i32) -> bool {
    match bord {
        Bord::Hexagonal => distance_hex(q, r) == rayon,
        Bord::Circulaire => {
            let (q, r, rayon) = (i128::from(q), i128::from(r), i128::from(rayon));
            (2 * q + r) * (2 * q + r) + 3 * r * r >= 3 * rayon * rayon
        }
    }
}

/// Le simulateur naïf avec un bord : des masses en `brut` dans un `BTreeMap` indexé par `(q, r)`, et le pas de la doc écrit avec des
/// entiers `i128` et des arrondis à la main (sans `Fixe`).
#[derive(Clone)]
pub struct NaifBord {
    pub rayon: i32,
    pub bord: Bord,
    pub alpha: i128,
    pub beta: i128,
    pub gamma: i128,
    pub masses: BTreeMap<(i32, i32), i128>,
    pub temps: u64,
}

impl NaifBord {
    /// L'état initial : toutes les cases `β`, la case centrale 1 (`2^32`).
    pub fn initial(rayon: i32, p: Parametres, bord: Bord) -> NaifBord {
        let mut masses = BTreeMap::new();
        for (q, r) in super::etape_3_commun::cases(rayon) {
            masses.insert((q, r), i128::from(p.beta.0));
        }
        masses.insert((0, 0), 1 << 32);
        NaifBord {
            rayon,
            bord,
            alpha: i128::from(p.alpha.0),
            beta: i128::from(p.beta.0),
            gamma: i128::from(p.gamma.0),
            masses,
            temps: 0,
        }
    }

    /// Un état quelconque : `brut[i]` est la masse de la i-ème case de l'ordre canonique.
    pub fn depuis_brut(rayon: i32, p: Parametres, bord: Bord, brut: &[i64]) -> NaifBord {
        let mut n = NaifBord::initial(rayon, p, bord);
        let toutes = super::etape_3_commun::cases(rayon);
        assert_eq!(toutes.len(), brut.len());
        for (cellule, &b) in toutes.iter().zip(brut) {
            n.masses.insert(*cellule, i128::from(b));
        }
        n
    }

    fn gelee(&self, c: (i32, i32)) -> bool {
        self.masses[&c] >= 1 << 32
    }

    /// 'G', 'B' ou 'N' (gelée, de bord au sens d'une classe, non réceptive), avec les voisins qui existent seulement.
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

    /// Un pas de la doc : les cases du bord reviennent à `β` ; les autres diffusent (leurs six voisins existent : la recherche dans le
    /// dictionnaire panique sinon).
    pub fn pas(&mut self) {
        let classes: BTreeMap<(i32, i32), char> = self.masses.keys().map(|&c| (c, self.classe(c))).collect();
        let u = |c: (i32, i32)| -> i128 { if classes[&c] == 'N' { self.masses[&c] } else { 0 } };
        let mut nouvelles = BTreeMap::new();
        for (&(q, r), &s) in &self.masses {
            if du_bord(self.bord, self.rayon, q, r) {
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
        super::etape_3_commun::cases(self.rayon)
            .iter()
            .map(|c| i64::try_from(self.masses[c]).expect("tient dans un i64"))
            .collect()
    }
}

//! Aides communes aux tests de l'étape G2 : un calcul NAÏF des cinq mesures (des boucles simples sur un ensemble de cases `(q, r)`, d'après la doc de `src/balayage.rs` seule), les
//! douze symétries de l'hexagone, des cristaux construits à la main (germe, disque, bras, anneau, étoile…) ou tirés par un congruentiel, la neige d'un ensemble de cases rattachées
//! et l'ensemble des cases rattachées d'une neige.
//!
//! Ce fichier n'est pas un test : il ne contient aucune fonction `#[test]`. Il est gelé avec les tests.
#![allow(dead_code)]

use givre_lab::balayage::Mesures;
use givre_lab::math::Fixe;
use givre_lab::neige::{Case, Neige, Parametres};
use std::collections::BTreeSet;

/// Un ensemble de cases rattachées : des coordonnées axiales `(q, r)`.
pub type Ensemble = BTreeSet<(i32, i32)>;

/// Les six directions de la doc de `reseau` (dans l'ordre de `DIRECTIONS_HEX`), réécrites ici.
pub const DIRECTIONS: [(i32, i32); 6] = [(1, 0), (0, 1), (-1, 1), (-1, 0), (0, -1), (1, -1)];

/// La distance hexagonale à l'origine : `max(|q|, |r|, |q + r|)`, réécrite ici.
pub fn distance(q: i32, r: i32) -> i32 {
    q.abs().max(r.abs()).max((q + r).abs())
}

/// Les cases de l'hexagone de rayon `rayon` dans l'ordre canonique de la doc de `reseau` (`r` croissant, puis `q` croissant), réécrites ici.
pub fn cases(rayon: i32) -> Vec<(i32, i32)> {
    let mut sortie = Vec::new();
    for r in -rayon..=rayon {
        for q in (-rayon).max(-rayon - r)..=rayon.min(rayon - r) {
            sortie.push((q, r));
        }
    }
    sortie
}

/// Une symétrie de l'hexagone : une fonction de `(q, r)` vers `(q, r)`.
pub type Symetrie = fn(i32, i32) -> (i32, i32);

/// Les douze symétries de l'hexagone (le groupe diédral engendré par la rotation de 60° `(q, r) -> (-r, q + r)` et la réflexion `(q, r) -> (r, q)`), écrites en toutes lettres.
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

/// L'image d'un ensemble par une symétrie.
pub fn image(ensemble: &Ensemble, g: Symetrie) -> Ensemble {
    ensemble.iter().map(|&(q, r)| g(q, r)).collect()
}

/// Un congruentiel linéaire de 64 bits (constantes de Knuth, MMIX), pour des cristaux sans symétrie : des graines fixes, écrites dans les tests.
pub struct Lcg(pub u64);

impl Lcg {
    pub fn suivant(&mut self) -> u64 {
        self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        self.0 >> 11
    }

    /// Un entier de `0` à `borne` exclu.
    pub fn jusqu_a(&mut self, borne: u64) -> u64 {
        self.suivant() % borne
    }
}

/// Les réglages écrits en décimaux, lus par `Parametres::lire` : dans l'ordre `rho, beta, alpha, theta, kappa, mu, gamma`.
pub fn parametres(rho: &str, beta: &str, alpha: &str, theta: &str, kappa: &str, mu: &str, gamma: &str) -> Parametres {
    Parametres::lire(rho, beta, alpha, theta, kappa, mu, gamma).expect("réglages valides")
}

/// La ligne « 9 » de l'annexe 8 de l'article (figure 9) avec `ρ = 0,5`.
pub fn fig9() -> Parametres {
    parametres("0.5", "1.3", "0.08", "0.025", "0.003", "0.07", "0.00005")
}

/// Un jeu d'essai à nous (pas l'article) où les cristaux grandissent vite et se remplissent.
pub fn jeu_dense() -> Parametres {
    parametres("0.9", "1.05", "0.5", "0.5", "0.1", "0.05", "0.001")
}

/// La neige de rayon `rayon` dont les cases rattachées sont exactement `ensemble` (`a = 1`, `c = UN`, `b = d = 0`) et dont toute autre case est vide (`b = c = d = 0`) : un état valide
/// pour `Neige::essayer_depuis_cases` (invariants I1, I2, I3) quel que soit l'ensemble.
pub fn neige_de(rayon: u32, ensemble: &Ensemble, parametres: Parametres) -> Neige {
    let liste: Vec<Case> = cases(rayon as i32)
        .iter()
        .map(|c| {
            if ensemble.contains(c) {
                Case { a: true, b: Fixe::ZERO, c: Fixe::UN, d: Fixe::ZERO }
            } else {
                Case { a: false, b: Fixe::ZERO, c: Fixe::ZERO, d: Fixe::ZERO }
            }
        })
        .collect();
    Neige::essayer_depuis_cases(rayon, parametres, &liste).expect("état valide")
}

/// L'ensemble des cases rattachées d'une neige, lu case par case dans l'ordre canonique ÉCRIT ICI (et vérifié contre `Neige::case`).
pub fn rattachees_de(neige: &Neige) -> Ensemble {
    let liste = neige.cases();
    let ordre = cases(neige.rayon() as i32);
    assert_eq!(liste.len(), ordre.len(), "nombre de cases");
    let mut ensemble = Ensemble::new();
    for (i, &(q, r)) in ordre.iter().enumerate() {
        assert_eq!(neige.case(q, r), Some(liste[i]), "cases() n'est pas dans l'ordre canonique à l'indice {i}");
        if liste[i].a {
            ensemble.insert((q, r));
        }
    }
    ensemble
}

/// Les six mesures calculées au plus bête depuis la définition de la doc : des boucles sur les cases de l'ensemble, les voisines parcourues dans l'ordre INVERSE de `DIRECTIONS` (l'ordre ne change
/// aucun entier), des divisions entières en `u64`. Une case voisine absente de l'ensemble (libre, ou hors de l'hexagone) compte comme libre : l'ensemble ne contient que des cases de l'hexagone.
pub fn mesures_naives(a: &Ensemble) -> Mesures {
    let n = a.len() as u64;
    let rayon = a.iter().map(|&(q, r)| distance(q, r)).max().unwrap_or(0) as u64;
    let disque = 3 * rayon * (rayon + 1) + 1;
    let mut pointes = 0_u32;
    let mut aiguilles = 0_u32;
    let mut anneau = 0_u64;
    for &(q, r) in a {
        let mut libres = 0;
        for &(dq, dr) in DIRECTIONS.iter().rev() {
            if !a.contains(&(q + dq, r + dr)) {
                libres += 1;
            }
        }
        if libres >= 3 {
            pointes += 1;
        }
        if libres >= 4 {
            aiguilles += 1;
        }
        if distance(q, r) as u64 == rayon {
            anneau += 1;
        }
    }
    Mesures {
        rattachees: n as u32,
        rayon: rayon as u32,
        compacite: (1000 * n / disque) as u32,
        pointes,
        aiguilles,
        hexagonalite: (1000 * anneau / (6 * rayon).max(1)) as u32,
    }
}

/// Le disque hexagonal plein de rayon `r` autour de `(0, 0)`.
pub fn disque(r: i32) -> Ensemble {
    cases(r).into_iter().collect()
}

/// Un bras : `(0, 0), (1, 0), …, (longueur, 0)`.
pub fn bras(longueur: i32) -> Ensemble {
    (0..=longueur).map(|q| (q, 0)).collect()
}

/// L'anneau : les cases à la distance exactement `r`.
pub fn anneau(r: i32) -> Ensemble {
    cases(r).into_iter().filter(|&(q, rr)| distance(q, rr) == r).collect()
}

/// L'étoile à six bras d'une case de large : les six demi-droites `k * d` pour `d` dans `DIRECTIONS` et `k = 0..=longueur`.
pub fn etoile(longueur: i32) -> Ensemble {
    let mut ensemble = Ensemble::new();
    for (dq, dr) in DIRECTIONS {
        for k in 0..=longueur {
            ensemble.insert((k * dq, k * dr));
        }
    }
    ensemble
}

/// Les cases à distance au plus `largeur` d'un des points de l'étoile de `longueur` (des bras épaissis : `largeur = 1` donne des bras de trois cases de large).
pub fn etoile_epaisse(longueur: i32, largeur: i32) -> Ensemble {
    let mut ensemble = Ensemble::new();
    for (q, r) in etoile(longueur) {
        for (dq, dr) in cases(largeur) {
            ensemble.insert((q + dq, r + dr));
        }
    }
    ensemble
}

/// Un ensemble tiré : chaque case de l'hexagone de rayon `rayon` est rattachée avec la probabilité `densite / 100` (pas de connexité, pas de symétrie).
pub fn ensemble_au_hasard(rayon: i32, densite: u64, graine: u64) -> Ensemble {
    let mut g = Lcg(graine);
    cases(rayon).into_iter().filter(|_| g.jusqu_a(100) < densite).collect()
}

/// Un cristal tiré, connexe, sans symétrie, qui grandit autour de l'origine : `tours` tours où chaque case voisine d'une case rattachée se rattache avec la probabilité `densite / 100`.
pub fn blob_au_hasard(rayon: i32, tours: u32, densite: u64, graine: u64) -> Ensemble {
    let mut g = Lcg(graine);
    let mut ensemble = Ensemble::new();
    ensemble.insert((0, 0));
    for _ in 0..tours {
        let voisines: Vec<(i32, i32)> = ensemble
            .iter()
            .flat_map(|&(q, r)| DIRECTIONS.iter().map(move |&(dq, dr)| (q + dq, r + dr)))
            .filter(|&(q, r)| distance(q, r) <= rayon && !ensemble.contains(&(q, r)))
            .collect();
        for c in voisines {
            if g.jusqu_a(100) < densite {
                ensemble.insert(c);
            }
        }
    }
    ensemble
}

//! Les aides communes des tests de l'étape P3 : la mesure NAÏVE de la pointe (le contour, la règle des lignes, la parabole, la vitesse
//! et `σ*` de la doc de `src/pointe.rs`, réécrits ici en boucles simples sur les champs, sans rien tirer de `givre_lab::pointe` sauf
//! ses types), ses fautes volontaires, les champs faits à la main (une ligne de `φ` linéaire en `x` autour d'un croisement voulu), la
//! dendrite de la table II de Kim et coll. selon `ε`, et un congruentiel pour les tirages. Les fonctions de la bibliothèque standard
//! sont permises ici : ce sont des références, pas du code de `src/`.
#![allow(dead_code)]

use givre_lab::dendrite::{Dendrite, Domaine, ModeU, Reglages};
use givre_lab::pointe::{ErreurPointe, Pointe};

/// Un congruentiel linéaire (Knuth, MMIX) : `x <- x * 6364136223846793005 + 1442695040888963407` ; rend un `f64` dans `[0, 1[` avec
/// les 53 bits du haut (le même que celui des tests de P2).
pub struct Lcg(pub u64);

impl Lcg {
    pub fn suivant(&mut self) -> f64 {
        self.0 = self.0.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1_442_695_040_888_963_407);
        (self.0 >> 11) as f64 / (1_u64 << 53) as f64
    }
    pub fn entre(&mut self, a: f64, b: f64) -> f64 {
        a + (b - a) * self.suivant()
    }
}

/// Les fautes volontaires de la mesure naïve (voir `etape_p3_fautes.rs`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Faute {
    /// `y` pris au bord de la case (`i dx`) au lieu de son centre (`(i + 0,5) dx`).
    YAuBord,
    /// la pointe lue sur la mauvaise ligne : `x0` est le croisement de la ligne 1 au lieu de la ligne 0.
    MauvaiseLigne,
    /// la profondeur comparée strictement (`<` au lieu de `<=`).
    ProfondeurStricte,
    /// la règle des lignes sans arrêt : une ligne qui manque à la règle est sautée, les suivantes sont encore regardées.
    SansArret,
    /// le DERNIER croisement de la ligne (le plus grand `j`) au lieu du premier.
    DernierCroisement,
    /// la parabole par les sommes brutes `b = (m Σ xY - ΣY Σx) / (m ΣY² - (ΣY)²)` au lieu des écarts aux moyennes.
    SommesBrutes,
    /// le signe de la parabole : `x = sommet + y² / (2 rayon)`, donc `rayon = 1 / (2 b)`.
    SigneDeLaParabole,
    /// `ρ` au lieu de `ρ²` dans `σ*`.
    RayonSimple,
    /// la vitesse mesurée sur le sommet de la parabole au lieu de la position de la pointe.
    VitesseDuSommet,
    /// l'interpolation écrite depuis la case de droite : `x_{j+1} - dx (φ[j+1] / (φ[j+1] - φ[j]))`.
    InterpolationParLaDroite,
}

pub const TOUTES_LES_FAUTES: [Faute; 10] = [
    Faute::YAuBord,
    Faute::MauvaiseLigne,
    Faute::ProfondeurStricte,
    Faute::SansArret,
    Faute::DernierCroisement,
    Faute::SommesBrutes,
    Faute::SigneDeLaParabole,
    Faute::RayonSimple,
    Faute::VitesseDuSommet,
    Faute::InterpolationParLaDroite,
];

/// Le croisement naïf d'une ligne (la doc : le plus petit `j` de `0` à `len - 2` avec `φ[j] > 0` et `φ[j+1] <= 0`, puis
/// `((j + 0,5) dx) + dx (φ[j] / (φ[j] - φ[j+1]))`), avec la faute éventuelle.
pub fn croisement_naif(ligne: &[f64], dx: f64, faute: Option<Faute>) -> Option<f64> {
    let mut trouve = None;
    for j in 0..ligne.len().saturating_sub(1) {
        if ligne[j] > 0.0 && ligne[j + 1] <= 0.0 {
            trouve = Some(j);
            if faute != Some(Faute::DernierCroisement) {
                break;
            }
        }
    }
    let j = trouve?;
    let (a, b) = (ligne[j], ligne[j + 1]);
    if faute == Some(Faute::InterpolationParLaDroite) {
        return Some(((j as f64 + 1.5) * dx) - dx * (b / (b - a)));
    }
    let gauche = (j as f64 + 0.5) * dx;
    Some(gauche + dx * (a / (a - b)))
}

/// Le contour naïf d'un champ `n × n` (ligne par ligne, `None` sans croisement).
pub fn contour_naif(phi: &[f64], n: usize, dx: f64, faute: Option<Faute>) -> Vec<Option<f64>> {
    let mut c = Vec::with_capacity(n);
    for i in 0..n {
        c.push(croisement_naif(&phi[i * n..(i + 1) * n], dx, faute));
    }
    c
}

/// La parabole naïve (la doc, une ligne = une opération), avec la faute éventuelle.
pub fn parabole_naive(points: &[(f64, f64)], faute: Option<Faute>) -> Result<(f64, f64), ErreurPointe> {
    for &(y, x) in points {
        if !y.is_finite() || !x.is_finite() {
            return Err(ErreurPointe::ParametreInvalide("points"));
        }
    }
    if points.len() < 3 {
        return Err(ErreurPointe::TropPeuDeLignes(points.len()));
    }
    let m = points.len() as f64;
    let b;
    let my;
    let mx;
    if faute == Some(Faute::SommesBrutes) {
        let (mut s1, mut s2, mut sx, mut sxy) = (0.0, 0.0, 0.0, 0.0);
        for &(y, x) in points {
            let yy = y * y;
            s1 += yy;
            s2 += yy * yy;
            sx += x;
            sxy += x * yy;
        }
        b = (m * sxy - s1 * sx) / (m * s2 - s1 * s1);
        my = s1 / m;
        mx = sx / m;
    } else {
        let mut sy = 0.0;
        let mut sx = 0.0;
        for &(y, x) in points {
            sy += y * y;
            sx += x;
        }
        my = sy / m;
        mx = sx / m;
        let mut sxy = 0.0;
        let mut syy = 0.0;
        for &(y, x) in points {
            let e = (y * y) - my;
            sxy += e * (x - mx);
            syy += e * e;
        }
        b = sxy / syy;
    }
    let sommet = mx - (b * my);
    let rayon = if faute == Some(Faute::SigneDeLaParabole) { 1.0 / (2.0 * b) } else { -1.0 / (2.0 * b) };
    if rayon > 0.0 && rayon.is_finite() && sommet.is_finite() {
        Ok((sommet, rayon))
    } else {
        Err(ErreurPointe::PasDeCourbure)
    }
}

/// La mesure naïve de la pointe d'un champ `n × n` de quart au pas `dx`, au temps `temps` (la doc de `mesurer_pointe`), avec la faute
/// éventuelle. `entier` : la dendrite n'est pas un quart.
pub fn mesure_naive(
    phi: &[f64],
    n: usize,
    dx: f64,
    temps: f64,
    entier: bool,
    profondeur: f64,
    faute: Option<Faute>,
) -> Result<Pointe, ErreurPointe> {
    if !(profondeur.is_finite() && profondeur > 0.0) {
        return Err(ErreurPointe::ParametreInvalide("profondeur"));
    }
    if entier {
        return Err(ErreurPointe::DomaineEntier);
    }
    let c = contour_naif(phi, n, dx, faute);
    let ligne_de_pointe = if faute == Some(Faute::MauvaiseLigne) { 1 } else { 0 };
    let Some(x0) = c[ligne_de_pointe] else {
        return Err(ErreurPointe::PasDePointe);
    };
    let mut points = Vec::new();
    for (i, xi) in c.iter().enumerate() {
        let retenue = match xi {
            Some(x) => {
                if faute == Some(Faute::ProfondeurStricte) {
                    x0 - x < profondeur
                } else {
                    x0 - x <= profondeur
                }
            }
            None => false,
        };
        if !retenue {
            if faute == Some(Faute::SansArret) {
                continue;
            }
            break;
        }
        let y = if faute == Some(Faute::YAuBord) { i as f64 * dx } else { (i as f64 + 0.5) * dx };
        points.push((y, xi.unwrap()));
    }
    let (sommet, rayon) = parabole_naive(&points, faute)?;
    Ok(Pointe { temps, position: x0, lignes: points.len(), sommet, rayon })
}

/// La mesure naïve d'une dendrite (ses champs et ses accesseurs).
pub fn mesure_naive_de(d: &Dendrite, profondeur: f64, faute: Option<Faute>) -> Result<Pointe, ErreurPointe> {
    mesure_naive(d.phi(), d.n(), d.reglages().dx, d.temps(), d.domaine() != Domaine::Quart, profondeur, faute)
}

/// La vitesse naïve (la doc de `vitesse_de_pointe`), avec la faute éventuelle.
pub fn vitesse_naive(avant: &Pointe, apres: &Pointe, faute: Option<Faute>) -> Result<f64, ErreurPointe> {
    if !(avant.temps.is_finite() && apres.temps.is_finite() && apres.temps > avant.temps) {
        return Err(ErreurPointe::ParametreInvalide("temps"));
    }
    if !(avant.position.is_finite() && apres.position.is_finite()) {
        return Err(ErreurPointe::ParametreInvalide("position"));
    }
    let (a, b) = if faute == Some(Faute::VitesseDuSommet) {
        (avant.sommet, apres.sommet)
    } else {
        (avant.position, apres.position)
    };
    let v = (b - a) / (apres.temps - avant.temps);
    if v.is_finite() { Ok(v) } else { Err(ErreurPointe::HorsDePortee) }
}

/// `σ*` naïve (la doc de `constante_de_selection`), avec la faute éventuelle.
pub fn selection_naive(d0: f64, d: f64, rayon: f64, vitesse: f64, faute: Option<Faute>) -> Result<f64, ErreurPointe> {
    for (nom, v) in [("d0", d0), ("d", d), ("rayon", rayon), ("vitesse", vitesse)] {
        if !(v.is_finite() && v > 0.0) {
            return Err(ErreurPointe::ParametreInvalide(nom));
        }
    }
    let r2 = if faute == Some(Faute::RayonSimple) { rayon } else { rayon * rayon };
    let s = ((2.0 * d0) * d) / (r2 * vitesse);
    if s.is_finite() && s > 0.0 { Ok(s) } else { Err(ErreurPointe::HorsDePortee) }
}

/// Les bits d'une mesure (pour comparer deux `Result<Pointe, _>` au bit près, zéros signés compris).
pub fn bits_mesure(m: &Result<Pointe, ErreurPointe>) -> Result<(u64, u64, usize, u64, u64), ErreurPointe> {
    m.clone().map(|p| (p.temps.to_bits(), p.position.to_bits(), p.lignes, p.sommet.to_bits(), p.rayon.to_bits()))
}

// ---------------------------------------------------------------------------------------------------------------------
// Les champs faits à la main
// ---------------------------------------------------------------------------------------------------------------------

/// Un champ `n × n` dont la ligne `i` vaut `φ[i][j] = clamp((xs[i] - x_j) / w, -1, 1)` avec `x_j = (j + 0,5) dx` : une rampe
/// linéaire en `x` qui croise zéro en `xs[i]` (l'interpolation de la doc la retrouve exactement quand `w > dx` et que les opérations
/// sont exactes, par exemple en rationnels dyadiques). Une ligne dont `xs[i]` est NaN vaut `-1` partout (pas de croisement).
pub fn champ_par_lignes(n: usize, dx: f64, xs: &[f64], w: f64) -> Vec<f64> {
    let mut phi = vec![-1.0; n * n];
    for i in 0..n {
        if xs[i].is_nan() {
            continue;
        }
        for j in 0..n {
            let x = (j as f64 + 0.5) * dx;
            phi[i * n + j] = ((xs[i] - x) / w).clamp(-1.0, 1.0);
        }
    }
    phi
}

/// Une dendrite qui porte ce champ, sans un pas fait (`u = 0` gelé ; `W0 = tau0 = lambda = D = 1`, `epsilon = 0,05`, `dt = 1e-5`,
/// sous la borne pour `dx >= 0,05`).
pub fn dendrite_de_champ(phi: &[f64], n: usize, dx: f64, domaine: Domaine) -> Dendrite {
    let r = Reglages { w0: 1.0, tau0: 1.0, lambda: 1.0, d: 1.0, epsilon: 0.05, dx, dt: 1e-5 };
    Dendrite::new(r, n, domaine, phi, &vec![0.0; n * n], ModeU::Gele).expect("un champ fait à la main valide")
}

/// La parabole exacte `x = 10 - y² / 8` (sommet 10, rayon 4), échantillonnée aux centres `y_i = (i + 0,5) · 0,5` des lignes d'un quart
/// au pas `dx = 0,5` : des rationnels dyadiques, exacts en `f64`. Les lignes dont `x` serait négatif n'ont pas de croisement.
pub fn xs_parabole_exacte(n: usize) -> Vec<f64> {
    (0..n)
        .map(|i| {
            let y = (i as f64 + 0.5) * 0.5;
            let x = 10.0 - (y * y) / 8.0;
            if x > 0.25 { x } else { f64::NAN }
        })
        .collect()
}

// ---------------------------------------------------------------------------------------------------------------------
// La dendrite de la table II de Kim et coll. selon epsilon
// ---------------------------------------------------------------------------------------------------------------------

/// La graine ronde de la table II de Kim et coll. (`W0 = 1`, `d0 = 0,5`, `D = 1`, `Delta = 0,65`, rayon 15, `u` joué) à l'anisotropie
/// `epsilon`, dans un quart de côté `l` au pas `dx` (`n = round(l / dx)` cases), avec `dt = periode / ceil(periode / dt_borne)` : un pas
/// sous la borne de la doc de P2 (`U = Delta`), et chaque multiple de `periode` est atteint en un nombre entier de pas.
pub fn graine_de_kim(epsilon: f64, dx: f64, l: f64, periode: f64) -> (Dendrite, u64) {
    let delta = 0.65;
    let r0 = Reglages::depuis_capillarite(1.0, 0.5, 1.0, epsilon, dx, 1.0);
    let borne = r0.dt_borne(delta, ModeU::Joue);
    assert!(borne > 0.0 && borne.is_finite(), "dt_borne = {borne}");
    let pas_par_periode = (periode / borne).ceil() as u64;
    let r = Reglages { dt: periode / pas_par_periode as f64, ..r0 };
    let n = (l / dx).round() as usize;
    (Dendrite::graine_ronde(r, n, Domaine::Quart, delta, 15.0, ModeU::Joue).expect("graine valide"), pas_par_periode)
}

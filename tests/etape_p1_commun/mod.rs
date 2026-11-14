//! Les aides communes des tests de l'étape P1 : les simulateurs NAÏFS (les deux équations en boucles simples, dans l'ordre
//! de la doc de `src/phase.rs`), leurs fautes volontaires, et les trois cas exacts. Aucune fonction n'est tirée de
//! `givre_lab::phase` sauf les types sous test : le naïf recalcule tout (constantes comprises). Les fonctions de la
//! bibliothèque standard (`tanh`, `sqrt`, `powi`) sont permises ici : ce sont des références, pas du code de `src/`.
#![allow(dead_code)]

use givre_lab::math::Fixe;
use givre_lab::phase::{Front1d, Front1dFixe, ModeU, Parametres};

/// a1 = 5 sqrt(2) / 8 (V-P1-A1 : recalculé à la main, voir la fiche).
pub const A1: f64 = 0.883_883_476_483_184_4;
/// a2 = 47/75 = 0,626 666… (V-P1-A2 : lu dans deux sources ouvertes et recalculé, voir la fiche) ; il ne sert qu'à CHOISIR la
/// cinétique nulle du cas 3.
pub const A2: f64 = 47.0 / 75.0;

/// Les fautes volontaires d'un naïf (voir `etape_p1_fautes.rs`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Faute {
    /// `+ lambda u (1 - φ²)²` au lieu de `- lambda u (1 - φ²)²`.
    SigneLambda,
    /// le `1/2` de `1/2 dφ` oublié.
    DemiOublie,
    /// `(1 - φ²)` au lieu de `(1 - φ²)²`.
    UnMoinsPhi2Simple,
    /// mise à jour en place de `φ` (le voisin de gauche est déjà le nouveau).
    EnPlace,
    /// bord qui fuit : la case voisine hors de la grille vaut 0 au lieu de la case du bord.
    BordFuit,
    /// `u` joué même quand il est gelé.
    UJoue,
    /// `W` au lieu de `W²` dans `c_phi = W / dx²`.
    WSansCarre,
    /// `dt` au lieu de `dt / tau`.
    TauOublie,
    /// `phi[i-1] - 2 phi[i] + phi[i+1]` au lieu de la somme par paires (même valeur au flottant près, mais plus la symétrie).
    SansPaires,
    /// `u` ne reçoit pas le dφ de ce pas mais celui du pas d'avant.
    DphiEnRetard,
}

pub const TOUTES_LES_FAUTES: [Faute; 10] = [
    Faute::SigneLambda,
    Faute::DemiOublie,
    Faute::UnMoinsPhi2Simple,
    Faute::EnPlace,
    Faute::BordFuit,
    Faute::UJoue,
    Faute::WSansCarre,
    Faute::TauOublie,
    Faute::SansPaires,
    Faute::DphiEnRetard,
];

/// L'état rendu par un simulateur : `φ` et `u` en `f64` (pour `Fixe`, `vers_f64` : exact).
#[derive(Clone, Debug, PartialEq)]
pub struct Etat {
    pub phi: Vec<f64>,
    pub u: Vec<f64>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Moteur {
    /// `Front1d` de la bibliothèque.
    Lib64,
    /// `Front1dFixe` de la bibliothèque.
    LibFixe,
    /// le naïf `f64`, avec une faute éventuelle.
    Naif64(Option<Faute>),
    /// le naïf `Fixe`.
    NaifFixe,
}

pub fn param(w: f64, tau: f64, lambda: f64, d: f64, dx: f64, dt: f64) -> Parametres {
    Parametres { w, tau, lambda, d, dx, dt }
}

/// Le naïf `f64` : les formules de la doc, une boucle par case, deux tableaux.
pub fn naif64(p: &Parametres, phi0: &[f64], u0: &[f64], mode: ModeU, pas: u64, faute: Option<Faute>) -> Etat {
    let n = phi0.len();
    let r = if faute == Some(Faute::TauOublie) { p.dt } else { p.dt / p.tau };
    let c_phi = if faute == Some(Faute::WSansCarre) { p.w / (p.dx * p.dx) } else { (p.w * p.w) / (p.dx * p.dx) };
    let c_u = (p.d * p.dt) / (p.dx * p.dx);
    let mut phi = phi0.to_vec();
    let mut u = u0.to_vec();
    let joue = mode == ModeU::Joue || faute == Some(Faute::UJoue);
    let mut dphi_avant = vec![0.0; n];
    for _ in 0..pas {
        let mut phi2 = phi.clone();
        let mut u2 = u.clone();
        let mut dphi_ce_pas = vec![0.0; n];
        for i in 0..n {
            let (gauche, droite) = match faute {
                Some(Faute::BordFuit) => {
                    (if i == 0 { 0.0 } else { phi[i - 1] }, if i + 1 == n { 0.0 } else { phi[i + 1] })
                }
                Some(Faute::EnPlace) => {
                    (if i == 0 { phi[0] } else { phi2[i - 1] }, if i + 1 == n { phi[n - 1] } else { phi[i + 1] })
                }
                _ => (if i == 0 { phi[0] } else { phi[i - 1] }, if i + 1 == n { phi[n - 1] } else { phi[i + 1] }),
            };
            let lap_phi = if faute == Some(Faute::SansPaires) {
                (gauche - 2.0 * phi[i]) + droite
            } else {
                (gauche + droite) - 2.0 * phi[i]
            };
            let f = phi[i] - (phi[i] * phi[i]) * phi[i];
            let q = 1.0 - phi[i] * phi[i];
            let q2 = if faute == Some(Faute::UnMoinsPhi2Simple) { q } else { q * q };
            let mut g = (p.lambda * u[i]) * q2;
            if faute == Some(Faute::SigneLambda) {
                g = -g;
            }
            let vitesse = c_phi * lap_phi + (f - g);
            let dphi = r * vitesse;
            phi2[i] = phi[i] + dphi;
            dphi_ce_pas[i] = dphi;
            if joue {
                let ug = if i == 0 { u[0] } else { u[i - 1] };
                let ud = if i + 1 == n { u[n - 1] } else { u[i + 1] };
                let lap_u = (ug + ud) - 2.0 * u[i];
                let moitie = if faute == Some(Faute::DemiOublie) { 1.0 } else { 0.5 };
                let dp = if faute == Some(Faute::DphiEnRetard) { dphi_avant[i] } else { dphi };
                u2[i] = u[i] + (c_u * lap_u + moitie * dp);
            }
        }
        phi = phi2;
        u = u2;
        dphi_avant = dphi_ce_pas;
    }
    Etat { phi, u }
}

/// Le naïf `Fixe` : les mêmes formules avec les opérateurs de `Fixe`, constantes converties par `Fixe::depuis_f64`.
pub fn naif_fixe(p: &Parametres, phi0: &[f64], u0: &[f64], mode: ModeU, pas: u64) -> Etat {
    let n = phi0.len();
    let r = Fixe::depuis_f64(p.dt / p.tau);
    let c_phi = Fixe::depuis_f64((p.w * p.w) / (p.dx * p.dx));
    let c_u = Fixe::depuis_f64((p.d * p.dt) / (p.dx * p.dx));
    let lambda = Fixe::depuis_f64(p.lambda);
    let (un, deux, moitie) = (Fixe::UN, Fixe::entier(2), Fixe::depuis_f64(0.5));
    let mut phi: Vec<Fixe> = phi0.iter().map(|&x| Fixe::depuis_f64(x)).collect();
    let mut u: Vec<Fixe> = u0.iter().map(|&x| Fixe::depuis_f64(x)).collect();
    for _ in 0..pas {
        let mut phi2 = phi.clone();
        let mut u2 = u.clone();
        for i in 0..n {
            let gauche = if i == 0 { phi[0] } else { phi[i - 1] };
            let droite = if i + 1 == n { phi[n - 1] } else { phi[i + 1] };
            let lap_phi = (gauche + droite) - deux * phi[i];
            let f = phi[i] - (phi[i] * phi[i]) * phi[i];
            let q = un - phi[i] * phi[i];
            let g = (lambda * u[i]) * (q * q);
            let vitesse = c_phi * lap_phi + (f - g);
            let dphi = r * vitesse;
            phi2[i] = phi[i] + dphi;
            if mode == ModeU::Joue {
                let ug = if i == 0 { u[0] } else { u[i - 1] };
                let ud = if i + 1 == n { u[n - 1] } else { u[i + 1] };
                let lap_u = (ug + ud) - deux * u[i];
                u2[i] = u[i] + (c_u * lap_u + moitie * dphi);
            }
        }
        phi = phi2;
        u = u2;
    }
    Etat { phi: phi.iter().map(|x| x.vers_f64()).collect(), u: u.iter().map(|x| x.vers_f64()).collect() }
}

/// Fait tourner un moteur : l'état après `pas` pas.
pub fn simuler(m: Moteur, p: &Parametres, phi: &[f64], u: &[f64], mode: ModeU, pas: u64) -> Etat {
    match m {
        Moteur::Lib64 => {
            let mut f = Front1d::new(*p, phi, u, mode).expect("réglages valides");
            f.avancer(pas).expect("pas");
            Etat { phi: f.phi().to_vec(), u: f.u().to_vec() }
        }
        Moteur::LibFixe => {
            let mut f = Front1dFixe::new(*p, phi, u, mode).expect("réglages valides");
            f.avancer(pas).expect("pas");
            Etat {
                phi: f.phi().iter().map(|x| x.vers_f64()).collect(),
                u: f.u().iter().map(|x| x.vers_f64()).collect(),
            }
        }
        Moteur::Naif64(faute) => naif64(p, phi, u, mode, pas, faute),
        Moteur::NaifFixe => naif_fixe(p, phi, u, mode, pas),
    }
}

/// La position du front, recodée ici d'après la doc : le plus petit `i` avec `φ[i] > 0` et `φ[i+1] <= 0`,
/// `(i + 1/2) dx + dx φ[i] / (φ[i] - φ[i+1])`.
pub fn position(phi: &[f64], dx: f64) -> Option<f64> {
    (0..phi.len() - 1)
        .find(|&i| phi[i] > 0.0 && phi[i + 1] <= 0.0)
        .map(|i| (i as f64 + 0.5) * dx + dx * (phi[i] / (phi[i] - phi[i + 1])))
}

// ---------------------------------------------------------------------------------------------------------------------
// Les trois cas exacts (V-P1-01 à V-P1-03)
// ---------------------------------------------------------------------------------------------------------------------

/// Le pas de temps des cas : `fraction · dt_max()`, après avoir vérifié que `dt_max()` est bien celui de la doc (recalculé ici, à
/// `1e-12` près en relatif) : un `dt_max` faux (nul, minuscule, NaN) fait ÉCHOUER le cas, au lieu de le faire tourner sans fin
/// (le nombre de pas est `durée / dt`).
pub fn pas_de_temps(p: &Parametres, fraction: f64) -> f64 {
    let attendu = ((p.dx * p.dx) / (2.0 * p.d)).min((p.tau * (p.dx * p.dx)) / (2.0 * (p.w * p.w) + p.dx * p.dx));
    let rendu = p.dt_max();
    assert!((rendu - attendu).abs() <= 1e-12 * attendu, "dt_max() rend {rendu:e}, la doc attend {attendu:e}");
    fraction * rendu
}

/// Cas 1 (front au repos) : `W = 1,25`, `tau = 1,6`, `lambda = 1`, `D = 1`, domaine de largeur 40 centré sur le front,
/// départ en marche `+1 | -1`, `u = 0` gelé, `dt = dt_max / 2`, temps 60. Rend `(écart max à -tanh(x/(W sqrt2)), état)`.
pub fn cas1(m: Moteur, dx: f64) -> (f64, Etat) {
    let (w, tau) = (1.25, 1.6);
    let mut p = param(w, tau, 1.0, 1.0, dx, 0.0);
    p.dt = pas_de_temps(&p, 0.5);
    let n = (40.0 / dx).round() as usize;
    let x = |i: usize| (i as f64 + 0.5 - n as f64 / 2.0) * dx;
    let depart: Vec<f64> = (0..n).map(|i| if x(i) < 0.0 { 1.0 } else { -1.0 }).collect();
    let pas = (60.0 * tau / p.dt).round() as u64;
    let e = simuler(m, &p, &depart, &vec![0.0; n], ModeU::Gele, pas);
    let ecart = (0..n).map(|i| (e.phi[i] - (-(x(i) / (w * 2f64.sqrt())).tanh())).abs()).fold(0.0, f64::max);
    (ecart, e)
}

/// Les vitesses EXACTES du front poussé (V-P1-ONDE) : `(a = lambda Delta, c = v tau / W)`, la vitesse de l'onde progressive de
/// `tau phi_t = W² phi_xx + phi - phi³ + a (1 - phi²)²`, calculée par `juge_onde.py` (un tir dans le plan de phase : ni grille
/// d'espace ni pas de temps), à tout ordre en `a`. Le premier ordre de l'article est `c1 = a / a1` ; `c / c1 - 1` vaut
/// -1,44e-5 ; -1,432e-3 ; -1,2367e-2 ; -9,790e-2 (soit `-0,144 a²` aux petits `a`).
pub const C_EXACT: [(f64, f64); 4] =
    [(0.01, 0.011_313_545_590), (0.1, 0.112_975_044_459), (0.3, 0.335_213_712_822), (1.0, 1.020_609_388_452)];

/// `c` exact pour un `lambda Delta` de la table `C_EXACT`.
pub fn c_exact(lambda_delta: f64) -> f64 {
    C_EXACT.iter().find(|t| t.0 == lambda_delta).expect("lambda Delta de la table C_EXACT").1
}

/// Cas 2 (front poussé) : `W = 1,25`, `tau = 2`, `lambda`, `D = 1`, `u = -Delta` gelé, `Delta = lambda_delta / lambda`, front à
/// 10 au départ sur le profil exact du repos, grille de largeur 60, `dt = dt_max / 2`. La vitesse est mesurée entre `t = 20` et
/// `t = 20 + T`, `T = max(20, 1,5 / v_exacte)` (le front parcourt au moins 1,5, soit trois cases au pas le plus gros : sur moins
/// d'une case, la mesure ne verrait que l'erreur de l'interpolation), par la position de la doc. Rend
/// `v mesurée / v_exacte - 1` (NaN si le front a disparu), avec `v_exacte = c_exact(lambda_delta) W / tau`.
pub fn cas2(m: Moteur, dx: f64, lambda: f64, lambda_delta: f64) -> f64 {
    let (w, tau) = (1.25, 2.0);
    let mut p = param(w, tau, lambda, 1.0, dx, 0.0);
    p.dt = pas_de_temps(&p, 0.5);
    let v_exacte = c_exact(lambda_delta) * w / tau;
    let duree = (1.5 / v_exacte).max(20.0);
    let n = (60.0 / dx).round() as usize;
    let phi: Vec<f64> = (0..n).map(|i| -(((i as f64 + 0.5) * dx - 10.0) / (w * 2f64.sqrt())).tanh()).collect();
    let u = vec![-lambda_delta / lambda; n];
    let pas_a = (20.0 / p.dt).round() as u64;
    let pas_b = ((20.0 + duree) / p.dt).round() as u64;
    let a = simuler(m, &p, &phi, &u, ModeU::Gele, pas_a);
    let b = simuler(m, &p, &phi, &u, ModeU::Gele, pas_b);
    let (Some(xa), Some(xb)) = (position(&a.phi, dx), position(&b.phi, dx)) else {
        return f64::NAN;
    };
    let v = (xb - xa) / ((pas_b - pas_a) as f64 * p.dt);
    v / v_exacte - 1.0
}

/// erf par la série de Taylor (assez précise pour `x <= 4` ; 1 au-delà) : l'outil de test pour le profil exact de `u`.
pub fn erf(x: f64) -> f64 {
    if x > 4.0 {
        return 1.0;
    }
    let (mut terme, mut somme) = (x, 0.0);
    for n in 0..200 {
        somme += terme / (2 * n + 1) as f64;
        terme = -terme * x * x / (n + 1) as f64;
    }
    2.0 / std::f64::consts::PI.sqrt() * somme
}

/// Cas 3 (front de Neumann) : `W`, `tau = 2 W²`, `D = 1,5`, `lambda = D tau / (a2 W²) = 2 D / a2 = 4,787…` (cinétique `β = 0` ;
/// `tau` suit `W²` pour que `lambda` ne change pas quand `W` diminue : à `tau` fixé, `lambda` croîtrait comme `1/W²` et l'écart
/// avec lui, voir la fiche), front de départ en `x0 = 10` avec le profil exact de `u` de la solution de Neumann à
/// `t0 = (x0 / (2k))² / D` (liquide `u = -Delta (1 - erfc(x / (2 sqrt(D t0))) / erfc(k))`, solide `u = 0`), solide contre le mur
/// sans flux à gauche ; largeur `8 sqrt(D t_fin) + 20` ; `dt = dt_max / 2` ; on avance de 400 en temps. Rend l'écart relatif
/// `X_mesuré / X_exact - 1` (NaN si le front a disparu), avec `X_exact = 2 k sqrt(D (t0 + 400))`.
pub fn cas3(m: Moteur, delta: f64, k: f64, w: f64, dx: f64) -> f64 {
    let d = 1.5;
    let tau = 2.0 * w * w;
    let lambda = d * tau / (A2 * w * w);
    let mut p = param(w, tau, lambda, d, dx, 0.0);
    p.dt = pas_de_temps(&p, 0.5);
    let x0 = 10.0;
    let t0 = (x0 / (2.0 * k)).powi(2) / d;
    let tfin = t0 + 400.0;
    let n = ((8.0 * (d * tfin).sqrt() + 20.0) / dx).round() as usize;
    let erfck = 1.0 - erf(k);
    let phi: Vec<f64> = (0..n).map(|i| -(((i as f64 + 0.5) * dx - x0) / (w * 2f64.sqrt())).tanh()).collect();
    let u: Vec<f64> = (0..n)
        .map(|i| {
            let x = (i as f64 + 0.5) * dx;
            if x < x0 { 0.0 } else { -delta * (1.0 - (1.0 - erf(x / (2.0 * (d * t0).sqrt()))) / erfck) }
        })
        .collect();
    let pas = (400.0 / p.dt).round() as u64;
    let e = simuler(m, &p, &phi, &u, ModeU::Joue, pas);
    let t = t0 + pas as f64 * p.dt;
    match position(&e.phi, dx) {
        Some(x) => x / (2.0 * k * (d * t).sqrt()) - 1.0,
        None => f64::NAN,
    }
}

/// Les trois `(Delta, k)` : `k` racine de `Delta = sqrt(pi) k e^(k²) erfc(k)`, calculé par `p1_k.py` (décimal 60
/// chiffres) et recalculé par `juge_k.py` (une autre écriture de la fonction) : voir la fiche, V-P1-K.
pub const K_NEUMANN: [(f64, f64); 3] = [(0.25, 0.168_999_042_462), (0.5, 0.432_751_599_366), (0.75, 0.971_850_282_583)];

// ---------------------------------------------------------------------------------------------------------------------
// Les tolérances MESURÉES (sur le prototype de l'écrivain et sur le second prototype du juge : les mêmes bits), et leurs points :
// 1,14 à 1,25 fois la valeur observée (la plus grande des deux arithmétiques), arrondie vers le haut. Chaque ligne dit : le
// réglage, la valeur observée, la tolérance. Aucune ne garde seule un cas : chaque cas porte aussi un critère d'ordre de
// convergence qui ne dépend pas de la mesure (voir `etape_p1_exact.rs`), et l'égalité bit à bit au naïf est le vrai garde-fou.
// ---------------------------------------------------------------------------------------------------------------------

/// Cas 1 : `(dx, tolérance sur l'écart max au profil exact)`. Observé (f64 / Fixe) : 1,668e-2 ; 4,375e-3 ; 1,1193e-3 ;
/// 2,815e-4 / 2,8153e-4. Le rapport entre deux pas successifs vaut 3,8 ; 3,9 ; 4,0 (ordre 2).
pub const TOL_CAS1: [(f64, f64); 4] = [(1.0, 1.95e-2), (0.5, 5.0e-3), (0.25, 1.3e-3), (0.125, 3.3e-4)];

/// Cas 2 : `(dx, lambda, lambda·Delta, tolérance sur |v / v_exacte - 1|)`. Observé (f64 ; Fixe identique à 3e-8 près) :
/// dx = 0,5 : -8,171e-3 ; -8,237e-3 ; -8,846e-3 ; -1,8561e-2 — dx = 0,25 : -2,0195e-3 ; -2,0435e-3 ; -2,2322e-3 ; -4,7174e-3 —
/// dx = 0,125 : -5,005e-4 ; -5,101e-4 ; -5,819e-4 ; -1,1587e-3 : une erreur de grille d'ordre 2 (rapports de 3,8 à 4,1 quand `dx` est
/// divisé par 2), que le second ordre en `lambda Delta` ne pollue plus puisque la référence est la vitesse exacte.
pub const TOL_CAS2: [(f64, f64, f64, f64); 12] = [
    (0.5, 1.0, 0.01, 9.9e-3),
    (0.5, 2.0, 0.1, 9.9e-3),
    (0.5, 1.0, 0.3, 1.07e-2),
    (0.5, 1.0, 1.0, 2.23e-2),
    (0.25, 1.0, 0.01, 2.5e-3),
    (0.25, 2.0, 0.1, 2.5e-3),
    (0.25, 1.0, 0.3, 2.7e-3),
    (0.25, 1.0, 1.0, 5.7e-3),
    (0.125, 1.0, 0.01, 6.1e-4),
    (0.125, 2.0, 0.1, 6.2e-4),
    (0.125, 1.0, 0.3, 7.0e-4),
    (0.125, 1.0, 1.0, 1.4e-3),
];

/// Cas 3 : `(indice dans K_NEUMANN, W, dx, tolérance sur |X / X_exact - 1|)`. Observé (f64 / Fixe) :
/// Delta = 0,25 : -3,802e-4 / -3,818e-4 (dx 0,5), -1,874e-4 / -1,932e-4 (dx 0,25) ; Delta = 0,5 : -4,389e-4 / -4,401e-4,
/// +2,941e-4 / +2,893e-4 ; W = 0,5, dx = 0,25 : -3,584e-4 / -3,622e-4 ; Delta = 0,75 : +8,917e-3 / +8,916e-3, +1,1903e-2 /
/// +1,1899e-2 ; W = 0,5, dx = 0,25 : +1,3640e-3 / +1,3611e-3. L'écart de `Delta = 0,75` est celui de l'interface mince (il ne
/// diminue pas avec `dx`) : il est 6,5 fois plus petit à `W = 0,5` qu'à `W = 1`.
pub const TOL_CAS3: [(usize, f64, f64, f64); 8] = [
    (0, 1.0, 0.5, 4.6e-4),
    (0, 1.0, 0.25, 2.4e-4),
    (1, 1.0, 0.5, 5.3e-4),
    (1, 1.0, 0.25, 3.6e-4),
    (1, 0.5, 0.25, 4.4e-4),
    (2, 1.0, 0.5, 1.07e-2),
    (2, 1.0, 0.25, 1.43e-2),
    (2, 0.5, 0.25, 1.65e-3),
];

// ---------------------------------------------------------------------------------------------------------------------
// Un congruentiel linéaire pour les états tirés (graine fixe écrite dans chaque test) et les aides de la diffusion 2D
// ---------------------------------------------------------------------------------------------------------------------

/// Un congruentiel linéaire (Knuth, MMIX) : `x <- x * 6364136223846793005 + 1442695040888963407` ; rend un `f64` dans
/// `[0, 1[` avec les 53 bits du haut.
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

/// La graine 2D à huit symétries : `c[i][j] = h(max(|a|,|b|), min(|a|,|b|))` avec `a = 2i - (n-1)`, `b = 2j - (n-1)` (les
/// coordonnées doublées du centre : `h` est une mixture entière de `(p, q)` rendue en `[0, 1[`). Deux cases images l'une de
/// l'autre par une des huit symétries du carré ont le même `(p, q)`, donc exactement le même flottant.
pub fn graine_2d(n: usize) -> Vec<f64> {
    graine_2d_sel(n, 0)
}

/// La même graine à huit symétries, par sel (`sel = 0` : `graine_2d`) : une graine différente par sel, pour éprouver la règle sur
/// plusieurs graines symétriques.
pub fn graine_2d_sel(n: usize, sel: u64) -> Vec<f64> {
    let mut c = vec![0.0; n * n];
    for i in 0..n {
        for j in 0..n {
            let a = (2 * i as i64 - (n as i64 - 1)).unsigned_abs();
            let b = (2 * j as i64 - (n as i64 - 1)).unsigned_abs();
            let (p, q) = (a.max(b), a.min(b));
            let mut h = p.wrapping_mul(0x9E37_79B9_7F4A_7C15)
                ^ q.wrapping_mul(0xC2B2_AE3D_27D4_EB4F)
                ^ 0x1234_5678
                ^ sel.wrapping_mul(0xD6E8_FEB8_6659_FD93);
            h ^= h >> 29;
            h = h.wrapping_mul(0xBF58_476D_1CE4_E5B9);
            h ^= h >> 32;
            c[i * n + j] = (h >> 11) as f64 / (1_u64 << 53) as f64;
        }
    }
    c
}

/// Les sept symétries non triviales du carré, écrites ici : `(nom, (n, i, j) -> image)`.
pub type Symetrie2d = (&'static str, fn(usize, usize, usize) -> (usize, usize));

pub const SYMETRIES_2D: [Symetrie2d; 7] = [
    ("miroir est-ouest", |n, i, j| (i, n - 1 - j)),
    ("miroir nord-sud", |n, i, j| (n - 1 - i, j)),
    ("rotation de 180 degrés", |n, i, j| (n - 1 - i, n - 1 - j)),
    ("rotation d'un quart de tour", |n, i, j| (j, n - 1 - i)),
    ("rotation de trois quarts de tour", |n, i, j| (n - 1 - j, i)),
    ("transposition", |_, i, j| (j, i)),
    ("anti-transposition", |n, i, j| (n - 1 - j, n - 1 - i)),
];

/// Pour chaque symétrie, le nombre de cases dont le flottant diffère (au bit) de celui de son image.
pub fn defauts_2d(n: usize, c: &[f64]) -> Vec<usize> {
    SYMETRIES_2D
        .iter()
        .map(|(_, g)| {
            let mut d = 0;
            for i in 0..n {
                for j in 0..n {
                    let (gi, gj) = g(n, i, j);
                    if c[i * n + j].to_bits() != c[gi * n + gj].to_bits() {
                        d += 1;
                    }
                }
            }
            d
        })
        .collect()
}

/// Un pas de diffusion à cinq points, bord sans flux, avec une des quatre écritures de la somme des voisines :
/// 0 : `(e + o) + (n + s)` (par paires : la règle) ; 1 : `((e + o) + n) + s` ; 2 : `(e + n) + (o + s)` (des paires qui ne
/// sont pas des opposées) ; 3 : `((e + n) + o) + s`. Le reste du pas est `c + k (somme - 4 c)`.
pub fn pas_2d(n: usize, k: f64, c: &[f64], ecriture: u8) -> Vec<f64> {
    let mut s = c.to_vec();
    for i in 0..n {
        for j in 0..n {
            let x = c[i * n + j];
            let e = if j + 1 < n { c[i * n + j + 1] } else { x };
            let o = if j > 0 { c[i * n + j - 1] } else { x };
            let no = if i > 0 { c[(i - 1) * n + j] } else { x };
            let su = if i + 1 < n { c[(i + 1) * n + j] } else { x };
            let somme = match ecriture {
                0 => (e + o) + (no + su),
                1 => ((e + o) + no) + su,
                2 => (e + no) + (o + su),
                _ => ((e + no) + o) + su,
            };
            s[i * n + j] = x + k * (somme - 4.0 * x);
        }
    }
    s
}

/// Le pas « sûr » de la doc de `src/phase.rs` (« La stabilité »), recalculé ici :
/// `min( dt_max, 2 tau / (4 w²/dx² + 2 + C lambda U), [u joué] 1 / (2 d/dx² + lambda / (4 tau)) )`, `C = 8 / (3 sqrt 3)`,
/// `U = max |u|`. (`dt_max` est recalculé à la main, sans la bibliothèque.)
pub fn dt_sur(p: &Parametres, u_max: f64, joue: bool) -> f64 {
    let c = 8.0 / (3.0 * 3f64.sqrt());
    let dt_max = (p.dx * p.dx / (2.0 * p.d)).min(p.tau * p.dx * p.dx / (2.0 * p.w * p.w + p.dx * p.dx));
    let a = 2.0 * p.tau / (4.0 * p.w * p.w / (p.dx * p.dx) + 2.0 + c * p.lambda * u_max);
    let b = if joue { 1.0 / (2.0 * p.d / (p.dx * p.dx) + p.lambda / (4.0 * p.tau)) } else { f64::INFINITY };
    a.min(b).min(dt_max)
}

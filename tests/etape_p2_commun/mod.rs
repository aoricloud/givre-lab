//! Les aides communes des tests de l'étape P2 : le simulateur NAÏF de la dendrite (le schéma de la doc de `src/dendrite.rs` en boucles
//! simples, une case à la fois), ses fautes volontaires, la graine ronde recalculée à la main, les graines à huit symétries, le front
//! droit d'axe et le front en diagonale. Aucune fonction n'est tirée de `givre_lab::dendrite` sauf les types sous test : le naïf recalcule
//! tout (constantes comprises). Les fonctions de la bibliothèque standard (`tanh`, `sqrt`, `powi`) sont permises ici : ce sont des
//! références, pas du code de `src/`.
#![allow(dead_code)]

use givre_lab::dendrite::{Dendrite, Domaine, ModeU, Reglages};

/// `a1 = 5 sqrt(2) / 8` (V-P2-A1 : la fiche de P1, V-P1-A1).
pub const A1: f64 = 0.883_883_476_483_184_4;
/// `a2 = 47/75` (V-P2-A2 : la fiche de P1, V-P1-A2).
pub const A2: f64 = 47.0 / 75.0;
/// Le seuil de la garde de la doc (`|∇φ|² < 1e-24`).
pub const GARDE: f64 = 1e-24;

/// L'état que les moteurs rendent.
#[derive(Clone, Debug)]
pub struct Etat {
    pub phi: Vec<f64>,
    pub u: Vec<f64>,
}

pub fn bits(v: &[f64]) -> Vec<u64> {
    v.iter().map(|x| x.to_bits()).collect()
}

/// Les fautes volontaires du naïf (voir `etape_p2_fautes.rs`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Faute {
    /// `ε` change de signe : `a = 1 - ε cos 4θ`, les bras sortent sur les diagonales.
    SigneAnisotropie,
    /// `ε` sur `W` mais pas sur `τ` : `dφ = r0 · rate` au lieu de `(r0 · rate) / (ac · ac)`.
    TauSansAnisotropie,
    /// les termes croisés oubliés : `H = a · a` (sans le `b`).
    TermesCroisesOublies,
    /// pas de garde de `|∇φ| = 0` : `0 / 0` loin du front.
    GardeAbsente,
    /// le seuil de la garde `1e-12` au lieu de `1e-24`.
    GardeSeuilFaux,
    /// le `1/2` de la chaleur latente oublié.
    DemiOublie,
    /// une constante calculée deux fois sous deux écritures : le flux en `y` emploie `w0 · (w0 · H)` au lieu de `(w0 · w0) · H`.
    ConstanteDeuxFois,
    /// mise à jour en place de `φ` (les cases déjà faites sont les nouvelles).
    EnPlace,
    /// bord qui fuit : la voisine hors de la grille vaut 0 au lieu de la case du bord (pour `φ` et `u`).
    BordFuit,
    /// la somme des quatre voisines de `u` en chaîne `((e + o) + n) + s` au lieu de `(e + o) + (n + s)`.
    SansPairesU,
    /// le gradient transverse en chaîne `((a + b) - c) - d` au lieu de `(a + b) - (c + d)`.
    SansPairesGt,
    /// `a1` au lieu de `a2` dans `tau0` (le rattachement de la doc : `Reglages::depuis_capillarite`).
    A1PourA2,
}

pub const TOUTES_LES_FAUTES: [Faute; 12] = [
    Faute::SigneAnisotropie,
    Faute::TauSansAnisotropie,
    Faute::TermesCroisesOublies,
    Faute::GardeAbsente,
    Faute::GardeSeuilFaux,
    Faute::DemiOublie,
    Faute::ConstanteDeuxFois,
    Faute::EnPlace,
    Faute::BordFuit,
    Faute::SansPairesU,
    Faute::SansPairesGt,
    Faute::A1PourA2,
];

/// Les réglages de cinétique nulle d'après la doc (`Reglages::depuis_capillarite`), recalculés à la main :
/// `lambda = (A1 w0) / d0`, `tau0 = (lambda (A2 (w0 w0))) / d`. La faute `A1PourA2` met `A1` à la place de `A2`.
pub fn reglages_naifs(w0: f64, d0: f64, d: f64, epsilon: f64, dx: f64, dt: f64, faute: Option<Faute>) -> Reglages {
    let lambda = (A1 * w0) / d0;
    let a = if faute == Some(Faute::A1PourA2) { A1 } else { A2 };
    let tau0 = (lambda * (a * (w0 * w0))) / d;
    Reglages { w0, tau0, lambda, d, epsilon, dx, dt }
}

/// La borne de pas de la doc (« La stabilité »), recalculée à la main : `0,5 min(dt_phi, dt_u)` (`dt_u` seulement si `u` est joué).
pub fn dt_borne_a_la_main(r: &Reglages, u_max: f64, mode: ModeU) -> f64 {
    let s_max = (r.w0 * r.w0) * ((1.0 - r.epsilon) * (1.0 + 15.0 * r.epsilon));
    let tau_min = r.tau0 * ((1.0 - r.epsilon) * (1.0 - r.epsilon));
    let dx2 = r.dx * r.dx;
    let dt_phi = (2.0 * tau_min) / ((8.0 * s_max) / dx2 + 2.0 + (1.54 * r.lambda) * u_max);
    let m =
        if mode == ModeU::Joue { dt_phi.min(1.0 / ((4.0 * r.d) / dx2 + r.lambda / (4.0 * tau_min))) } else { dt_phi };
    0.5 * m
}

/// Le pas de temps des cas : la borne de la doc, après avoir vérifié que `Reglages::dt_borne` rend bien celle-là (à `1e-12` près en
/// relatif) : une borne fausse (nulle, minuscule, NaN) fait ÉCHOUER le cas, au lieu de le faire tourner sans fin.
pub fn pas_de_temps(r: &Reglages, u_max: f64, mode: ModeU) -> f64 {
    let attendu = dt_borne_a_la_main(r, u_max, mode);
    let rendu = r.dt_borne(u_max, mode);
    assert!((rendu - attendu).abs() <= 1e-12 * attendu, "dt_borne() rend {rendu:e}, la doc attend {attendu:e}");
    attendu
}

/// Le naïf : le schéma de la doc, une boucle par case, un tableau par champ. `n` est le côté ; les voisines hors de la grille sont les
/// cases du bord par miroir (indices ramenés dans `[0, n-1]`, coins compris). Rend l'état après `pas` pas, avec la faute éventuelle.
pub fn naif(r: &Reglages, n: usize, phi0: &[f64], u0: &[f64], mode: ModeU, pas: u64, faute: Option<Faute>) -> Etat {
    let eps = if faute == Some(Faute::SigneAnisotropie) { -r.epsilon } else { r.epsilon };
    let garde = if faute == Some(Faute::GardeSeuilFaux) { 1e-12 } else { GARDE };
    let avec_garde = faute != Some(Faute::GardeAbsente);
    let ig = 1.0 / r.dx;
    let ig2 = 0.5 * ig;
    let ig4 = 0.25 * ig;
    let w2 = r.w0 * r.w0;
    let c1 = 1.0 - 3.0 * eps;
    let c2 = 4.0 * eps;
    let k16 = 16.0 * eps;
    let r0 = r.dt / r.tau0;
    let c_u = (r.d * r.dt) / (r.dx * r.dx);
    let joue = mode == ModeU::Joue;
    let ni = n as isize;
    // une lecture avec miroir (ou zéro si le bord fuit)
    let lire = |t: &[f64], i: isize, j: isize| -> f64 {
        if faute == Some(Faute::BordFuit) && (i < 0 || j < 0 || i >= ni || j >= ni) {
            return 0.0;
        }
        t[(i.clamp(0, ni - 1) * ni + j.clamp(0, ni - 1)) as usize]
    };
    let h = |s: f64, t: f64| -> f64 {
        let somme = s + t;
        if avec_garde && somme < garde {
            return 1.0;
        }
        let rr = somme * somme;
        let a = c1 + (c2 * ((s * s) + (t * t))) / rr;
        if faute == Some(Faute::TermesCroisesOublies) {
            return a * a;
        }
        let b = (k16 * (t * (s - t))) / rr;
        a * (a + b)
    };
    let mut phi = phi0.to_vec();
    let mut u = u0.to_vec();
    for _ in 0..pas {
        let mut nphi = phi.clone();
        let mut nu = u.clone();
        for i in 0..ni {
            for j in 0..ni {
                // la lecture de φ : l'ancien état, sauf en place (le tableau courant, déjà à moitié mis à jour)
                let lp = |a: isize, b: isize| lire(&phi, a, b);
                let fx = |a: isize, b: isize| -> f64 {
                    let gt = if faute == Some(Faute::SansPairesGt) {
                        (((lp(a + 1, b) + lp(a + 1, b + 1)) - lp(a - 1, b)) - lp(a - 1, b + 1)) * ig4
                    } else {
                        ((lp(a + 1, b) + lp(a + 1, b + 1)) - (lp(a - 1, b) + lp(a - 1, b + 1))) * ig4
                    };
                    let gn = (lp(a, b + 1) - lp(a, b)) * ig;
                    (w2 * h(gn * gn, gt * gt)) * gn
                };
                let fy = |a: isize, b: isize| -> f64 {
                    let gt = if faute == Some(Faute::SansPairesGt) {
                        (((lp(a, b + 1) + lp(a + 1, b + 1)) - lp(a, b - 1)) - lp(a + 1, b - 1)) * ig4
                    } else {
                        ((lp(a, b + 1) + lp(a + 1, b + 1)) - (lp(a, b - 1) + lp(a + 1, b - 1))) * ig4
                    };
                    let gn = (lp(a + 1, b) - lp(a, b)) * ig;
                    if faute == Some(Faute::ConstanteDeuxFois) {
                        (r.w0 * (r.w0 * h(gn * gn, gt * gt))) * gn
                    } else {
                        (w2 * h(gn * gn, gt * gt)) * gn
                    }
                };
                let ph = lp(i, j);
                let dxx = fx(i, j) - fx(i, j - 1);
                let dyy = fy(i, j) - fy(i - 1, j);
                let div = (dxx + dyy) * ig;
                let gxc = (lp(i, j + 1) - lp(i, j - 1)) * ig2;
                let gyc = (lp(i + 1, j) - lp(i - 1, j)) * ig2;
                let (p, q) = (gxc * gxc, gyc * gyc);
                let rs = p + q;
                let ac = if avec_garde && rs < garde { 1.0 } else { c1 + (c2 * ((p * p) + (q * q))) / (rs * rs) };
                let f = ph - (ph * ph) * ph;
                let qq = 1.0 - ph * ph;
                let k = (i * ni + j) as usize;
                let g = (r.lambda * u[k]) * (qq * qq);
                let rate = div + (f - g);
                let dphi = if faute == Some(Faute::TauSansAnisotropie) { r0 * rate } else { (r0 * rate) / (ac * ac) };
                let nv = ph + dphi;
                if faute == Some(Faute::EnPlace) {
                    phi[k] = nv;
                } else {
                    nphi[k] = nv;
                }
                if joue {
                    let (e, o) = (lire(&u, i, j + 1), lire(&u, i, j - 1));
                    let (no, su) = (lire(&u, i - 1, j), lire(&u, i + 1, j));
                    let lap = if faute == Some(Faute::SansPairesU) {
                        (((e + o) + no) + su) - 4.0 * u[k]
                    } else {
                        ((e + o) + (no + su)) - 4.0 * u[k]
                    };
                    let moitie = if faute == Some(Faute::DemiOublie) { 1.0 } else { 0.5 };
                    nu[k] = u[k] + (c_u * lap + moitie * dphi);
                }
            }
        }
        if faute != Some(Faute::EnPlace) {
            phi = nphi;
        }
        u = nu;
    }
    Etat { phi, u }
}

/// La graine ronde recalculée à la main d'après la doc (`Dendrite::graine_ronde`) : rend `(φ, u)`.
pub fn graine_ronde_naive(r: &Reglages, n: usize, domaine: Domaine, delta: f64, rayon: f64) -> (Vec<f64>, Vec<f64>) {
    let mut phi = vec![0.0; n * n];
    let mut u = vec![0.0; n * n];
    for i in 0..n {
        for j in 0..n {
            let coord = |k: usize| match domaine {
                Domaine::Quart => (k as f64 + 0.5) * r.dx,
                Domaine::Entier => (k as f64 - (n as f64 - 1.0) / 2.0) * r.dx,
            };
            let (x, y) = (coord(j), coord(i));
            let ray = (x * x + y * y).sqrt();
            let s = (ray - rayon) / (r.w0 * std::f64::consts::SQRT_2);
            let p = 0.0 - s.clamp(-1.0, 1.0);
            phi[i * n + j] = p;
            u[i * n + j] = 0.0 - (delta * ((1.0 - p) * 0.5));
        }
    }
    (phi, u)
}

/// Fait tourner la bibliothèque : l'état après `pas` pas.
pub fn simuler_lib(r: &Reglages, n: usize, domaine: Domaine, phi: &[f64], u: &[f64], mode: ModeU, pas: u64) -> Etat {
    let mut d = Dendrite::new(*r, n, domaine, phi, u, mode).expect("départ valide");
    d.avancer(pas).expect("pas");
    Etat { phi: d.phi().to_vec(), u: d.u().to_vec() }
}

// ---------------------------------------------------------------------------------------------------------------------
// Un congruentiel linéaire pour les états tirés (graine fixe écrite dans chaque test), les graines à huit symétries
// ---------------------------------------------------------------------------------------------------------------------

/// Un congruentiel linéaire (Knuth, MMIX) : `x <- x * 6364136223846793005 + 1442695040888963407` ; rend un `f64` dans `[0, 1[`
/// avec les 53 bits du haut.
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

/// Une graine à huit symétries dans `[0, 1[`, par sel : `c[i][j] = h(max(|a|,|b|), min(|a|,|b|), sel)` avec `a = 2i - (n-1)`,
/// `b = 2j - (n-1)` (les coordonnées doublées du centre : deux cases images l'une de l'autre par une des huit symétries du carré ont le
/// même `(p, q)`, donc exactement le même flottant, à mantisse pleine).
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

/// Un état de départ à huit symétries : `φ = 2 c - 1` dans `[-1, 1[` et `u = -c'` dans `]-1, 0]` (deux graines de sels différents).
pub fn depart_symetrique(n: usize, sel: u64) -> (Vec<f64>, Vec<f64>) {
    let phi = graine_2d_sel(n, 2 * sel).iter().map(|c| 2.0 * c - 1.0).collect();
    let u = graine_2d_sel(n, 2 * sel + 1).iter().map(|c| 0.0 - c).collect();
    (phi, u)
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

/// Pour une faute (ou `None`) : pour chacun des huit couples (`n` = 8 ou 9, graine 1 à 4), la perte de symétrie : vrai si le naïf, depuis
/// un départ tiré à huit symétries, a au moins une case asymétrique (au bit près, une des sept images, `φ` ou `u`) à l'un des points
/// 1, 2, 3, 10 et 100 pas. Réglages : `W0 = 1,1`, `tau0 = 1,3`, `lambda = 1,2`, `D = 0,8`, `epsilon = 0,05`, `dx = 0,6`, `u` joué, `dt` la borne
/// de la doc pour `U = 1`.
pub fn symetrie_perdue(faute: Option<Faute>) -> Vec<bool> {
    let mut r = Reglages { w0: 1.1, tau0: 1.3, lambda: 1.2, d: 0.8, epsilon: 0.05, dx: 0.6, dt: 1.0 };
    r.dt = pas_de_temps(&r, 1.0, ModeU::Joue);
    let mut v = Vec::new();
    for n in [8_usize, 9] {
        for sel in 1..=4_u64 {
            let (phi, u) = depart_symetrique(n, sel);
            v.push([1_u64, 2, 3, 10, 100].iter().any(|&pas| {
                let e = naif(&r, n, &phi, &u, ModeU::Joue, pas, faute);
                defauts_2d(n, &e.phi).iter().sum::<usize>() + defauts_2d(n, &e.u).iter().sum::<usize>() > 0
            }));
        }
    }
    v
}

/// Les trois petits cas de comparaison bit à bit (`bits_differents`) : `(réglages de la bibliothèque, réglages du naïf, n, domaine, φ, u,
/// mode, pas)`. Les réglages viennent de `depuis_capillarite` (côté bibliothèque) et de `reglages_naifs` (côté naïf, avec la faute).
type CasDeBits = (Reglages, Reglages, usize, Domaine, Vec<f64>, Vec<f64>, ModeU, u64);

fn cas_de_bits(faute: Option<Faute>) -> Vec<CasDeBits> {
    let mut cas = Vec::new();
    // alpha : un départ tiré, domaine entier, u joué, 25 pas
    let mut g = Lcg(7);
    let (phi, u): (Vec<f64>, Vec<f64>) =
        ((0..100).map(|_| g.entre(-1.0, 1.0)).collect(), (0..100).map(|_| g.entre(-1.0, 0.2)).collect());
    let umax = u.iter().fold(0.0_f64, |m, v| m.max(v.abs()));
    let base = Reglages::depuis_capillarite(1.2, 0.7, 0.9, 0.05, 0.5, 1.0);
    let dt = 0.7 * pas_de_temps(&base, umax, ModeU::Joue);
    cas.push((
        Reglages { dt, ..base },
        Reglages { dt, ..reglages_naifs(1.2, 0.7, 0.9, 0.05, 0.5, dt, faute) },
        10,
        Domaine::Entier,
        phi,
        u,
        ModeU::Joue,
        25,
    ));
    // beta : la graine ronde au coin d'un quart (des zones exactement plates), 150 pas
    let base = Reglages::depuis_capillarite(1.0, 0.5, 1.0, 0.05, 0.8, 1.0);
    let dt = pas_de_temps(&base, 0.65, ModeU::Joue);
    let (phi, u) = graine_ronde_naive(&base, 16, Domaine::Quart, 0.65, 5.0);
    cas.push((
        Reglages { dt, ..base },
        Reglages { dt, ..reglages_naifs(1.0, 0.5, 1.0, 0.05, 0.8, dt, faute) },
        16,
        Domaine::Quart,
        phi,
        u,
        ModeU::Joue,
        150,
    ));
    // gamma : un gradient minuscule (`|∇φ|²` entre 1e-24 et 1e-12 : sous le seuil faux, au-dessus du vrai), `u` gelé, 30 pas
    let (phi, u): (Vec<f64>, Vec<f64>) =
        ((0..144).map(|k| -1.0 + 1e-8 * (1 + (k / 12) * 3 + (k % 12) * 7) as f64).collect(), vec![-0.3; 144]);
    let base = Reglages::depuis_capillarite(1.0, 0.5, 1.0, 0.05, 0.8, 1.0);
    let dt = pas_de_temps(&base, 0.3, ModeU::Gele);
    cas.push((
        Reglages { dt, ..base },
        Reglages { dt, ..reglages_naifs(1.0, 0.5, 1.0, 0.05, 0.8, dt, faute) },
        12,
        Domaine::Entier,
        phi,
        u,
        ModeU::Gele,
        30,
    ));
    cas
}

/// Vrai si le naïf avec cette faute donne, sur au moins un des trois petits cas, des bits différents de ceux de la bibliothèque.
pub fn bits_differents(faute: Option<Faute>) -> bool {
    cas_de_bits(faute).iter().any(|(rl, rn, n, dom, phi, u, mode, pas)| {
        let a = simuler_lib(rl, *n, *dom, phi, u, *mode, *pas);
        let b = naif(rn, *n, phi, u, *mode, *pas, faute);
        bits(&a.phi) != bits(&b.phi) || bits(&a.u) != bits(&b.u)
    })
}

/// Vrai si le naïf avec cette faute sort de la tolérance du front droit d'axe (`dx = 0,5`, cas `(lambda, lambda Delta)` = `(2 ; 0,1)`,
/// boîte 24, `t_a = 10` : la tolérance 9,5e-3 de `etape_p2_exact.rs`), ou si le front disparaît (NaN).
pub fn axe_hors_tolerance(faute: Option<Faute>) -> bool {
    let (e, _) = front_droit(Moteur::Naif(faute), 0.5, 24.0, 10.0, 2.0, 0.1, 0);
    e.is_nan() || e.abs() > 9.5e-3
}

// ---------------------------------------------------------------------------------------------------------------------
// Le front droit d'axe, le front en diagonale (la vitesse exacte de P1) et la position de la pointe
// ---------------------------------------------------------------------------------------------------------------------

/// Les vitesses EXACTES du front poussé de P1 (V-P1-ONDE) : `(a = lambda Delta, c = v tau / W)`, calculées par le tir dans le plan de
/// phase de `juge_onde.py` : voir la fiche de P1 (`references/etape-p1-valeurs.md`) et `references/etape-p2-valeurs.md` (V-P2-02).
pub const C_EXACT: [(f64, f64); 4] =
    [(0.01, 0.011_313_545_590), (0.1, 0.112_975_044_459), (0.3, 0.335_213_712_822), (1.0, 1.020_609_388_452)];

pub fn c_exact(lambda_delta: f64) -> f64 {
    C_EXACT.iter().find(|t| t.0 == lambda_delta).expect("lambda Delta de la table C_EXACT").1
}

/// Le moteur d'un cas : la bibliothèque, ou le naïf avec une faute éventuelle.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Moteur {
    Lib,
    Naif(Option<Faute>),
}

#[allow(clippy::too_many_arguments)]
pub fn simuler(m: Moteur, r: &Reglages, n: usize, dom: Domaine, phi: &[f64], u: &[f64], mode: ModeU, pas: u64) -> Etat {
    match m {
        Moteur::Lib => simuler_lib(r, n, dom, phi, u, mode, pas),
        Moteur::Naif(f) => naif(r, n, phi, u, mode, pas, f),
    }
}

/// La position d'un zéro de `φ` le long d'une ligne de `n` cases au pas `dx` : le plus petit `j` avec `φ[j] > 0` et `φ[j+1] <= 0`,
/// `(j + 1/2) dx + dx φ[j] / (φ[j] - φ[j+1])` (la même interpolation que celle de la pointe).
pub fn position_ligne(ligne: &[f64], dx: f64) -> Option<f64> {
    (0..ligne.len() - 1)
        .find(|&j| ligne[j] > 0.0 && ligne[j + 1] <= 0.0)
        .map(|j| (j as f64 + 0.5) * dx + dx * (ligne[j] / (ligne[j] - ligne[j + 1])))
}

/// Cas du front droit d'axe (`direction = 0`) ou en diagonale (`direction = 1`). Les réglages : `W0 = 1,25`, `tau0 = 2`, `D = 1`,
/// `epsilon = 0,05`, `lambda`, `u = -Delta` gelé, `Delta = lambda_delta / lambda`, domaine ENTIER de côté `largeur / dx` cases ; le pas
/// de temps est la borne de la doc. Le front part sur le profil de repos `φ = -tanh(s / (sqrt 2 W))` avec `W = W0 (1 + epsilon)` (axe,
/// `a = 1 + epsilon`) ou `W0 (1 - epsilon)` (diagonale, `a = 1 - epsilon`), `s` la distance au front le long de la normale : l'axe
/// `x - x0` (front en `x0 = 6`), ou la diagonale `((x + y) / sqrt 2) - s0` avec `s0 = 0` (le front passe par le centre, en diagonale).
/// La vitesse est mesurée entre `t_a` et `t_a + T`, `T = max(t_a, 1,5 / v_exacte)`, sur la ligne `n/2` (axe) ou sur la diagonale
/// (cases `(k, k)`, abscisse `sqrt 2 · x`). Rend `(v mesurée / v_exacte - 1, toutes les lignes égales au bit près)` (la deuxième
/// valeur n'a de sens que pour l'axe), `v_exacte = c_exact(lambda_delta) W0 / (tau0 (1 ± epsilon))`.
pub fn front_droit(
    m: Moteur,
    dx: f64,
    largeur: f64,
    t_a: f64,
    lambda: f64,
    lambda_delta: f64,
    direction: u8,
) -> (f64, bool) {
    let (w0, tau0, d, eps) = (1.25, 2.0, 1.0, 0.05);
    let delta = lambda_delta / lambda;
    let n = (largeur / dx).round() as usize;
    let signe = if direction == 0 { 1.0 } else { -1.0 };
    let w = w0 * (1.0 + signe * eps);
    let v_exacte = c_exact(lambda_delta) * w0 / (tau0 * (1.0 + signe * eps));
    let mut r = Reglages { w0, tau0, lambda, d, epsilon: eps, dx, dt: 1.0 };
    r.dt = pas_de_temps(&r, delta, ModeU::Gele);
    let mut phi = vec![0.0; n * n];
    for i in 0..n {
        for j in 0..n {
            let s = if direction == 0 {
                (j as f64 + 0.5) * dx - 6.0
            } else {
                let x = (j as f64 - (n as f64 - 1.0) / 2.0) * dx;
                let y = (i as f64 - (n as f64 - 1.0) / 2.0) * dx;
                (x + y) / 2f64.sqrt()
            };
            phi[i * n + j] = -(s / (w * 2f64.sqrt())).tanh();
        }
    }
    let u = vec![-delta; n * n];
    let duree = (1.5 / v_exacte).max(t_a);
    let pas_a = (t_a / r.dt).round() as u64;
    let pas_b = ((t_a + duree) / r.dt).round() as u64;
    let a = simuler(m, &r, n, Domaine::Entier, &phi, &u, ModeU::Gele, pas_a);
    let b = simuler(m, &r, n, Domaine::Entier, &phi, &u, ModeU::Gele, pas_b);
    let pos = |e: &Etat| -> Option<f64> {
        if direction == 0 {
            position_ligne(&e.phi[(n / 2) * n..(n / 2) * n + n], dx)
        } else {
            let diag: Vec<f64> = (0..n).map(|k| e.phi[k * n + k]).collect();
            // la ligne des cases (k, k) est au pas `sqrt 2 · dx`, son origine est à `sqrt 2 · (-(n-1)/2 · dx)` du centre
            position_ligne(&diag, 2f64.sqrt() * dx).map(|p| p - 2f64.sqrt() * (n as f64 / 2.0) * dx)
        }
    };
    let egales = (1..n).all(|i| (0..n).all(|j| b.phi[i * n + j].to_bits() == b.phi[j].to_bits()));
    let (Some(xa), Some(xb)) = (pos(&a), pos(&b)) else {
        return (f64::NAN, egales);
    };
    let v = (xb - xa) / ((pas_b - pas_a) as f64 * r.dt);
    (v / v_exacte - 1.0, egales)
}

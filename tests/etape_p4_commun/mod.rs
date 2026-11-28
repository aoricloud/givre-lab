//! Ce que les fichiers de test de l'étape P4 (le métal sur la page) partagent : les constantes de la doc de `vitrine` (« Le métal »), **écrites ici à la main** d'après la fiche `references/etape-p4-valeurs.md` ; le métal **natif** en
//! face (une `Dendrite` construite directement par la bibliothèque, jouée par une boucle simple qui suit la règle du mur et des relevés de la doc, jamais par `vitrine`) ; l'image du métal de la doc de `docs/lecture.js` (`imageDuMetal`),
//! réécrite ici en Rust ; un congruentiel pour les tirages ; le lancement de Node.
//!
//! Ce fichier n'est pas un test : il ne contient aucune fonction `#[test]`. Il est gelé avec les tests.
#![allow(dead_code)]

use std::path::PathBuf;

use givre_lab::dendrite::{Dendrite, Domaine, ModeU, Reglages};
use givre_lab::empreinte::sha256_hex;
use givre_lab::pointe::{Pointe, constante_de_selection, mesurer_pointe, vitesse_de_pointe};

/// Les aides de l'étape V7 (qui reprennent celles de D3 et de l'étape 5 : `e5`, la lecture du texte de la page, les règles CSS, le lancement de Node).
#[path = "../etape_v7_commun/mod.rs"]
pub mod v7;

pub use v7::e5;

// ---------------------------------------------------------------------------------------------------------------
// Les constantes de la doc (V-P4-01), écrites à la main.
// ---------------------------------------------------------------------------------------------------------------

/// `ε` au plus, en millionièmes (0,06 : la plage du test du sens de P3).
pub const EPSILON_MAX_PPM: u32 = 60_000;
/// La surfusion, la longueur capillaire, la diffusivité, la largeur d'interface (Kim et coll., table II ; P2).
pub const DELTA: f64 = 0.65;
pub const D0: f64 = 0.5;
pub const DIFFUSIVITE: f64 = 1.0;
pub const W0: f64 = 1.0;
/// Le pas de grille et le rayon de la graine (P2 et P3).
pub const DX: f64 = 0.8;
pub const RAYON: f64 = 15.0;
/// Les côtés permis du quart.
pub const COTE_MIN: u32 = 128;
pub const COTE_MAX: u32 = 256;
/// La limite du mur, en pourcentage du côté (la limite est `côté · 55 / 100` cases, division entière, depuis le centre).
pub const MUR_POURCENT: u32 = 55;
/// La durée, en pas.
pub const PAS: u32 = 60_000;
/// L'intervalle des relevés, en pas.
pub const RELEVE: u64 = 500;
/// La profondeur de la mesure du rayon (P3).
pub const PROFONDEUR: f64 = 2.0;
/// « Pas de valeur » des lecteurs entiers du module.
pub const ABSENT: u32 = u32::MAX;

// ---------------------------------------------------------------------------------------------------------------
// Le métal natif en face (V-P4-02 à 06) : la bibliothèque, une boucle simple.
// ---------------------------------------------------------------------------------------------------------------

/// Les réglages de la doc pour `epsilon_ppm` : `epsilon = epsilon_ppm / 10⁶`, `depuis_capillarite(1, 0,5, 1, ε, 0,8, 1)`, puis `dt` = la borne de P2 pour `Δ = 0,65`, `u` joué.
pub fn reglages_de(epsilon_ppm: u32) -> Reglages {
    let epsilon = f64::from(epsilon_ppm) / 1_000_000.0;
    let mut r = Reglages::depuis_capillarite(W0, D0, DIFFUSIVITE, epsilon, DX, 1.0);
    r.dt = r.dt_borne(DELTA, ModeU::Joue);
    r
}

/// La limite du mur de la doc : `⌊cote · 55 / 100⌋ × 0,8` (la division est entière, puis un produit de flottants).
pub fn limite_de(cote: u32) -> f64 {
    f64::from(cote * MUR_POURCENT / 100) * DX
}

/// Le métal natif : la dendrite, la limite, les deux derniers relevés et la divergence, joués par une boucle d'un pas à la fois qui suit la doc mot pour mot.
#[derive(Clone)]
pub struct MetalNatif {
    pub dendrite: Dendrite,
    pub cote: u32,
    pub limite: f64,
    pub avant: Option<Pointe>,
    pub dernier: Option<Pointe>,
    pub diverge: bool,
}

impl MetalNatif {
    /// Le métal au pas 0 : la graine ronde (`Domaine::Quart`, `Δ`, rayon 15, `u` joué) et le relevé du pas 0.
    pub fn new(epsilon_ppm: u32, cote: u32) -> MetalNatif {
        let dendrite =
            Dendrite::graine_ronde(reglages_de(epsilon_ppm), cote as usize, Domaine::Quart, DELTA, RAYON, ModeU::Joue)
                .expect("une graine valide");
        let dernier = mesurer_pointe(&dendrite, PROFONDEUR).ok();
        MetalNatif { dendrite, cote, limite: limite_de(cote), avant: None, dernier, diverge: false }
    }

    /// Au mur : pas de pointe, ou la pointe à la limite ou au-delà.
    pub fn au_mur(&self) -> bool {
        match self.dendrite.position_pointe() {
            None => true,
            Some(x) => x >= self.limite,
        }
    }

    /// Joue au plus `n` pas (et jamais au-delà de la durée), en vérifiant avant chaque pas le mur et la divergence ; relève la pointe après chaque pas qui porte le compte à un multiple de 500. Rend les pas joués.
    pub fn avancer(&mut self, n: u32) -> u32 {
        let mut joues = 0;
        while joues < n && self.dendrite.pas_faits() < u64::from(PAS) {
            if self.diverge || self.au_mur() {
                break;
            }
            if self.dendrite.avancer(1).is_err() {
                self.diverge = true;
                break;
            }
            joues += 1;
            if self.dendrite.pas_faits().is_multiple_of(RELEVE) {
                self.avant = self.dernier.take();
                self.dernier = mesurer_pointe(&self.dendrite, PROFONDEUR).ok();
            }
        }
        joues
    }

    /// La vitesse entre les deux derniers relevés, ou `None`.
    pub fn vitesse(&self) -> Option<f64> {
        match (&self.avant, &self.dernier) {
            (Some(a), Some(b)) => vitesse_de_pointe(a, b).ok(),
            _ => None,
        }
    }

    /// Les onze mesures de la doc (« Les mesures du métal »), `NaN` quand elles n'existent pas.
    pub fn mesures(&self) -> [f64; 11] {
        let d = &self.dendrite;
        let releve = |f: &dyn Fn(&Pointe) -> f64| self.dernier.as_ref().map_or(f64::NAN, f);
        let vitesse = self.vitesse();
        let selection = match (vitesse, &self.dernier) {
            (Some(v), Some(p)) => constante_de_selection(D0, DIFFUSIVITE, p.rayon, v).unwrap_or(f64::NAN),
            _ => f64::NAN,
        };
        [
            d.position_pointe().unwrap_or(f64::NAN),
            d.temps(),
            if self.au_mur() { 1.0 } else { 0.0 },
            if self.diverge { 1.0 } else { 0.0 },
            releve(&|p| p.temps),
            releve(&|p| p.position),
            releve(&|p| p.lignes as f64),
            releve(&|p| p.sommet),
            releve(&|p| p.rayon),
            vitesse.unwrap_or(f64::NAN),
            selection,
        ]
    }

    /// Les quinze réglages de la doc (« Les réglages du métal »).
    pub fn reglages(&self) -> [f64; 15] {
        let r = self.dendrite.reglages();
        [
            r.epsilon,
            DELTA,
            D0,
            r.d,
            r.w0,
            r.lambda,
            r.tau0,
            r.dx,
            r.dt,
            RAYON,
            f64::from(self.cote),
            self.limite,
            f64::from(PAS),
            RELEVE as f64,
            PROFONDEUR,
        ]
    }

    /// Le nombre de cases du quart où `φ > 0`, et celui de la ligne 0.
    pub fn sites_et_rayon(&self) -> (u32, u32) {
        let n = self.cote as usize;
        let phi = self.dendrite.phi();
        (phi.iter().filter(|p| **p > 0.0).count() as u32, phi[..n].iter().filter(|p| **p > 0.0).count() as u32)
    }
}

/// Deux `f64` égaux **au bit près** (deux `NaN` sont égaux, quel que soit leur motif : le module ne promet que « `NaN` »).
pub fn memes_bits(a: f64, b: f64) -> bool {
    (a.is_nan() && b.is_nan()) || a.to_bits() == b.to_bits()
}

/// Un `f64` en hexadécimal (ses bits, gros-boutistes) ; `NaN` s'écrit `NaN`.
pub fn hexa(x: f64) -> String {
    if x.is_nan() { "NaN".to_string() } else { format!("{:016x}", x.to_bits()) }
}

/// Le SHA-256 des bits de `φ` puis de `u` (chaque valeur sur 8 octets petit-boutistes) : de quoi comparer deux champs entiers d'un mot.
pub fn hache_des_champs(phi: &[f64], u: &[f64]) -> String {
    let mut octets = Vec::with_capacity(16 * phi.len());
    for v in phi.iter().chain(u.iter()) {
        octets.extend_from_slice(&v.to_bits().to_le_bytes());
    }
    sha256_hex(&octets)
}

// ---------------------------------------------------------------------------------------------------------------
// L'image du métal (V-P4-30), la doc de `imageDuMetal` (`docs/lecture.js`) réécrite en Rust.
// ---------------------------------------------------------------------------------------------------------------

/// Les quatre couleurs du liquide, du plus froid au plus chaud, et celle du solide (la doc de `imageDuMetal`).
pub const LIQUIDE: [[f64; 3]; 4] = [[11.0, 17.0, 24.0], [24.0, 40.0, 66.0], [70.0, 92.0, 150.0], [255.0, 178.0, 122.0]];
pub const SOLIDE: [f64; 3] = [236.0, 242.0, 248.0];

/// L'image `taille × taille` du métal, octets R, V, B, 255 ligne par ligne, d'après la doc de `imageDuMetal` (le miroir du quart, l'interpolation entre les centres des cases, la part de solide, la chaleur, les couleurs).
pub fn image_du_metal(n: usize, phi: &[f64], u: &[f64], delta: f64, taille: usize) -> Vec<u8> {
    let mut rgba = vec![0_u8; 4 * taille * taille];
    let nf = n as f64;
    let t = taille as f64;
    let borne = |k: f64| -> usize {
        if k < 0.0 {
            0
        } else if k > nf - 1.0 {
            n - 1
        } else {
            k as usize
        }
    };
    for py in 0..taille {
        for px in 0..taille {
            let gx = ((2 * px + 1) * n) as f64 / t - nf;
            let gy = nf - ((2 * py + 1) * n) as f64 / t;
            let fj = gx.abs() - 0.5;
            let fi = gy.abs() - 0.5;
            let j0 = fj.floor();
            let i0 = fi.floor();
            let tj = fj - j0;
            let ti = fi - i0;
            let (ja, jb, ia, ib) = (borne(j0), borne(j0 + 1.0), borne(i0), borne(i0 + 1.0));
            let interpole = |f: &[f64]| {
                let r0 = f[ia * n + ja] + tj * (f[ia * n + jb] - f[ia * n + ja]);
                let r1 = f[ib * n + ja] + tj * (f[ib * n + jb] - f[ib * n + ja]);
                r0 + ti * (r1 - r0)
            };
            let p = interpole(phi);
            let v = interpole(u);
            let s = ((p + 1.0) * 0.5).clamp(0.0, 1.0);
            let h = ((v + delta) / delta).clamp(0.0, 1.0);
            let k = ((3.0 * h).floor() as usize).min(2);
            let a = 3.0 * h - k as f64;
            let o = 4 * (py * taille + px);
            for c in 0..3 {
                let liquide = LIQUIDE[k][c] + a * (LIQUIDE[k + 1][c] - LIQUIDE[k][c]);
                rgba[o + c] = (liquide + s * (SOLIDE[c] - liquide) + 0.5).floor() as u8;
            }
            rgba[o + 3] = 255;
        }
    }
    rgba
}

// ---------------------------------------------------------------------------------------------------------------
// Les tirages et Node.
// ---------------------------------------------------------------------------------------------------------------

/// Un congruentiel linéaire (Knuth, MMIX), celui des tests de P2 et P3 : un `u64` à chaque appel.
pub struct Lcg(pub u64);

impl Lcg {
    pub fn suivant(&mut self) -> u64 {
        self.0 = self.0.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1_442_695_040_888_963_407);
        self.0 >> 11
    }
    /// Un entier de `0` à `n − 1`.
    pub fn sous(&mut self, n: u64) -> u64 {
        self.suivant() % n
    }
}

pub fn racine() -> PathBuf {
    e5::racine().to_path_buf()
}

/// Lance un script de `tests/etape_p4/` avec ces arguments.
pub fn lancer_le_script(script: &str, arguments: &[&str]) -> std::process::Output {
    std::process::Command::new("node")
        .arg(racine().join("tests").join("etape_p4").join(script))
        .args(arguments)
        .output()
        .unwrap_or_else(|erreur| panic!("le lancement de Node échoue : {erreur}"))
}

pub fn chemin_du_module() -> String {
    e5::chemin_module().to_str().expect("chemin en UTF-8").to_string()
}

pub fn chemin_de_lecture() -> String {
    e5::chemin_de_lecture().to_str().expect("chemin en UTF-8").to_string()
}

// ---------------------------------------------------------------------------------------------------------------
// Les appels absurdes (V-P4-24) : la suite de `tests/etape_p4/metal-absurde.mjs`, rejouée sur le code natif.
// ---------------------------------------------------------------------------------------------------------------

/// Le xorshift de 32 bits du script (`x ^= x << 13 ; x ^= x >> 17 ; x ^= x << 5`, graine 2 463 534 242).
pub struct Xorshift(pub u32);

impl Xorshift {
    pub fn suivant(&mut self) -> u32 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 17;
        self.0 ^= self.0 << 5;
        self.0
    }
    pub fn tirer(&mut self, n: u32) -> u32 {
        self.suivant() % n
    }
    /// Une des quatorze valeurs absurdes (si `tirer(17) < 14`), sinon un `u32` tiré.
    pub fn argument(&mut self) -> u32 {
        const VALEURS: [u32; 14] = [0, 1, 2, 5, 127, 128, 129, 256, 257, 500, 50_000, 60_000, 60_001, u32::MAX];
        let r = self.tirer(17) as usize;
        if r < VALEURS.len() { VALEURS[r] } else { self.suivant() }
    }
}

/// Ce que `metal-absurde.mjs` doit écrire : la même suite de 400 tours rejouée sur les fonctions **natives** de la vitrine (dans l'ordre où le script tire ses nombres), chaque nombre rendu écrit comme le script l'écrit
/// (un `u32` en décimal, un `f64` par `hexa`), puis `appels`, `exceptions=0` et le SHA-256 de la suite. Ce n'est pas un test de justesse des valeurs (les autres tests le sont) : c'est le module contre le code natif, sur des appels
/// absurdes, sans panique. **Prend l'état global de la vitrine** : l'appelant tient le verrou de son fichier.
pub fn attendu_de_l_absurde() -> String {
    use givre_lab::vitrine::*;
    let mut x = Xorshift(2_463_534_242);
    let mut journal: Vec<(bool, u32, f64)> = Vec::new();
    for _ in 0..400 {
        match x.tirer(13) {
            0 => {
                let e = x.argument();
                let c = x.argument();
                let c = [128, 129, c][x.tirer(3) as usize];
                journal.push((true, givre_metal_demarrer(e, c), 0.0));
            }
            1 => journal.push((true, givre_avancer(x.tirer(6)), 0.0)),
            2 => journal.push((false, 0, givre_metal_reglage(x.argument()))),
            3 => journal.push((false, 0, givre_metal_mesure(x.argument()))),
            4 => {
                let champ = x.argument();
                let i = x.argument();
                journal.push((false, 0, givre_metal_case(champ, i)));
            }
            5 => {
                let modele = [0, 1, 2, 3, 8, 9, u32::MAX][x.tirer(7) as usize];
                let graine = x.argument();
                journal.push((true, givre_demarrer(modele, graine), 0.0));
            }
            6 => {
                if x.tirer(2) == 0 {
                    journal.push((true, givre_neige_demarrer(0, 400_000, 1_300_000, 3000, 70_000), 0.0));
                } else {
                    let a = [x.argument(), x.argument(), x.argument(), x.argument(), x.argument()];
                    journal.push((true, givre_neige_demarrer(a[0], a[1], a[2], a[3], a[4]), 0.0));
                }
            }
            7 => {
                journal.push((true, givre_temps(), 0.0));
                journal.push((true, givre_duree(), 0.0));
                journal.push((true, givre_sites(), 0.0));
                journal.push((true, givre_rayon(), 0.0));
            }
            8 => {
                journal.push((false, 0, givre_compacite()));
                let px = x.argument();
                let py = x.argument();
                journal.push((true, givre_pixel(px, py), 0.0));
                journal.push((true, givre_largeur(), 0.0));
                journal.push((true, givre_hauteur(), 0.0));
            }
            9 => journal.push((true, givre_empreinte_mot(x.tirer(10)), 0.0)),
            10 => {
                let a = x.argument();
                let cote = [2, 64, 63, a][x.tirer(4) as usize];
                let b = x.argument();
                let sur = [1, 2, 0, b][x.tirer(4) as usize];
                journal.push((true, givre_nette_preparer(cote, sur), 0.0));
            }
            11 => {
                journal.push((true, givre_mesurer(), 0.0));
                journal.push((true, givre_correlation_rayon(x.argument()), 0.0));
                journal.push((false, 0, givre_dimension()));
            }
            _ => {
                journal.push((true, givre_neige_masses_preparer(), 0.0));
                journal.push((true, givre_neige_masse(x.argument()), 0.0));
            }
        }
    }
    let rendus: Vec<String> =
        journal.iter().map(|(entier, v, f)| if *entier { v.to_string() } else { hexa(*f) }).collect();
    format!("appels={}\nexceptions=0\nhache={}\n", rendus.len(), sha256_hex(rendus.join(",").as_bytes()))
}

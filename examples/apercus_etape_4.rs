//! Les aperçus de l'étape 4 : cinq faces à l'équilibre, de la lisse à la rugueuse, et la rugosité d'équilibre contre
//! `J/kT` pour deux tailles. Des images pour l'œil, jamais une preuve : elles ne changent aucune référence.
//!
//! `apercus/etape-4-faces.png` : cinq faces de `64 × 64` colonnes côte à côte, à `Δμ = 0`, après `300` balayages depuis
//! une face plane (graine 1), pour `J/kT = 2 ; 1,5 ; 1 ; 0,75 ; 0,5` (`32 ; 24 ; 16 ; 12 ; 8` seizièmes), dans cet ordre
//! (de la face lisse à la face rugueuse). Chaque panneau est l'image de [`Face::apercu`] (l'étirement min-max de ses
//! propres hauteurs), et les panneaux sont séparés par un filet d'un pixel.
//!
//! `apercus/etape-4-rugosite.png` : la rugosité d'équilibre `w` contre `J/kT`, de `0,25` à `2` par pas de `1/16` (de `4`
//! à `32` seizièmes), pour `L = 16` et `L = 32`, une courbe et une couleur par taille, avec les deux axes, la grille et
//! leurs repères, sans texte. **Le protocole** : pour chaque point, les huit graines `1` à `8`, une face plane de `L × L`,
//! `300` balayages de transitoire jetés, puis la moyenne des `w²` sur `300` balayages ([`rugosite_moyenne`]) ; les huit
//! moyennes sont additionnées dans l'ordre des graines, divisées par huit, et le point tracé est `w = √<w²>`. À `Δμ = 0`
//! la face tend vers son équilibre de Boltzmann ; après le transitoire, on l'y suppose (c'est un aperçu, pas une mesure
//! d'équilibre).

use std::fs;
use std::path::PathBuf;

use givre_lab::cinetique::{Face, rugosite_moyenne};
use givre_lab::image::ecrire_png;
use givre_lab::math::logarithme;

/// Le côté de chaque face de la première image.
const COTE_FACE: usize = 64;
/// Le nombre de balayages de chaque face avant son aperçu.
const BALAYAGES_FACES: usize = 300;
/// La graine de chaque face (la même pour les cinq : elles ne diffèrent que par `J/kT`).
const GRAINE_FACES: u64 = 1;
/// Les `J/kT` des cinq faces, en seizièmes, de la plus lisse à la plus rugueuse.
const J_FACES: [u32; 5] = [32, 24, 16, 12, 8];
/// La couleur du filet entre les faces.
const FILET: [u8; 3] = [205, 205, 205];
/// L'épaisseur du filet, en pixels.
const FILET_EPAISSEUR: u32 = 1;

/// Les tailles `L` de la courbe de rugosité.
const TAILLES_RUGOSITE: [usize; 2] = [16, 32];
/// Les graines de la courbe de rugosité.
const GRAINES_RUGOSITE: [u64; 8] = [1, 2, 3, 4, 5, 6, 7, 8];
/// Les balayages de transitoire jetés avant chaque mesure.
const TRANSITOIRE_RUGOSITE: usize = 300;
/// Les balayages sur lesquels `rugosite_moyenne` moyenne chaque graine.
const FENETRE_RUGOSITE: usize = 300;
/// Le plus petit `J/kT` de la courbe, en seizièmes (`0,25`).
const J_MIN: u32 = 4;
/// Le plus grand `J/kT` de la courbe, en seizièmes (`2`).
const J_MAX: u32 = 32;

const LARGEUR_IMAGE: u32 = 640;
const HAUTEUR_IMAGE: u32 = 400;
const MARGE_GAUCHE: i64 = 56;
const MARGE_DROITE: i64 = 20;
const MARGE_HAUT: i64 = 20;
const MARGE_BAS: i64 = 44;
/// Les bornes de l'axe horizontal, un peu au-delà des données pour que les points des bords ne touchent pas l'axe.
const X_MIN: f64 = 0.2;
const X_MAX: f64 = 2.05;

const FOND: [u8; 3] = [250, 250, 248];
const BLANC: [u8; 3] = [255, 255, 255];
const GRILLE: [u8; 3] = [232, 232, 232];
const AXE: [u8; 3] = [70, 70, 70];
/// Une couleur par taille, dans l'ordre de `TAILLES_RUGOSITE`.
const COULEURS: [[u8; 3]; 2] = [[31, 119, 180], [255, 127, 14]];

fn main() {
    let dossier = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("apercus");
    fs::create_dir_all(&dossier).expect("création du dossier apercus/");

    let faces = apercu_faces();
    let chemin = dossier.join("etape-4-faces.png");
    fs::write(&chemin, &faces).expect("écriture de l'aperçu des faces");
    println!("{} : {} octets", chemin.display(), faces.len());

    let rugosite = apercu_rugosite();
    let chemin = dossier.join("etape-4-rugosite.png");
    fs::write(&chemin, &rugosite).expect("écriture de l'aperçu des rugosités");
    println!("{} : {} octets", chemin.display(), rugosite.len());
}

/// Les cinq faces côte à côte : chacune son aperçu de [`Face::apercu`], séparées par un filet de `FILET_EPAISSEUR`
/// pixels. La toile est d'abord remplie de la couleur du filet, puis chaque panneau est copié à sa place.
fn apercu_faces() -> Vec<u8> {
    let largeur = COTE_FACE as u32 * J_FACES.len() as u32 + FILET_EPAISSEUR * (J_FACES.len() as u32 - 1);
    let hauteur = COTE_FACE as u32;
    let mut rvb = Vec::with_capacity(3 * largeur as usize * hauteur as usize);
    for _ in 0..largeur as usize * hauteur as usize {
        rvb.extend_from_slice(&FILET);
    }
    for (indice, &j) in J_FACES.iter().enumerate() {
        let mut face = Face::new(COTE_FACE, j, 0, GRAINE_FACES);
        face.balayages(BALAYAGES_FACES);
        let panneau = face.apercu();
        let x0 = indice as u32 * (COTE_FACE as u32 + FILET_EPAISSEUR);
        for y in 0..COTE_FACE {
            let source = 3 * (y * COTE_FACE);
            let destination = 3 * ((y * largeur as usize) + x0 as usize);
            rvb[destination..destination + 3 * COTE_FACE].copy_from_slice(&panneau[source..source + 3 * COTE_FACE]);
        }
    }
    ecrire_png(largeur, hauteur, &rvb)
}

/// La rugosité d'équilibre de chaque taille, échantillonnée de `J_MIN` à `J_MAX` seizièmes inclus, dans cet ordre.
fn rugosites() -> Vec<Vec<f64>> {
    TAILLES_RUGOSITE
        .iter()
        .map(|&taille| {
            (J_MIN..=J_MAX)
                .map(|j| {
                    let mut total = 0.0;
                    for &graine in &GRAINES_RUGOSITE {
                        let mut face = Face::new(taille, j, 0, graine);
                        face.balayages(TRANSITOIRE_RUGOSITE);
                        total += rugosite_moyenne(&mut face, FENETRE_RUGOSITE);
                    }
                    (total / GRAINES_RUGOSITE.len() as f64).sqrt()
                })
                .collect()
        })
        .collect()
}

/// L'aperçu de la rugosité : une seule zone de tracé, l'axe horizontal linéaire en `J/kT`, l'axe vertical en échelle
/// logarithmique en `w`, une courbe et une couleur par taille, sans texte. La grille et les repères verticaux sont les
/// multiples de `1/4` de `J/kT` (et de petits traits à chaque seizième) ; les repères horizontaux sont les puissances de
/// deux de `w`.
fn apercu_rugosite() -> Vec<u8> {
    let courbes = rugosites();

    let mut log_w_min = f64::INFINITY;
    let mut log_w_max = f64::NEG_INFINITY;
    for courbe in &courbes {
        for &w in courbe {
            let log_w = logarithme(w);
            log_w_min = log_w_min.min(log_w);
            log_w_max = log_w_max.max(log_w);
        }
    }
    let marge = (log_w_max - log_w_min) / 20.0;
    let cadre = Cadre {
        gauche: MARGE_GAUCHE,
        droite: i64::from(LARGEUR_IMAGE) - MARGE_DROITE,
        haut: MARGE_HAUT,
        bas: i64::from(HAUTEUR_IMAGE) - MARGE_BAS,
        x_min: X_MIN,
        x_max: X_MAX,
        log_w_min: log_w_min - marge,
        log_w_max: log_w_max + marge,
    };

    let mut toile = Toile::nouvelle(LARGEUR_IMAGE, HAUTEUR_IMAGE, FOND);
    dessiner_panneau(&mut toile, &cadre, &courbes);
    ecrire_png(toile.largeur, toile.hauteur, &toile.pixels)
}

/// Les logarithmes des puissances de deux comprises dans `[bas, haut]`.
fn puissances_de_deux_entre(bas: f64, haut: f64) -> Vec<f64> {
    let mut valeur = 1.0_f64;
    while logarithme(valeur) >= bas {
        valeur /= 2.0;
    }
    let mut reperes = Vec::new();
    while logarithme(valeur) < haut {
        if logarithme(valeur) >= bas {
            reperes.push(logarithme(valeur));
        }
        valeur *= 2.0;
    }
    reperes
}

/// Le cadre du tracé : les bords de la zone de tracé en pixels, et les bornes des abscisses et des logarithmes.
struct Cadre {
    gauche: i64,
    droite: i64,
    haut: i64,
    bas: i64,
    x_min: f64,
    x_max: f64,
    log_w_min: f64,
    log_w_max: f64,
}

impl Cadre {
    /// La colonne de pixel de `J/kT = j`.
    fn x(&self, j: f64) -> i64 {
        let fraction = (j - self.x_min) / (self.x_max - self.x_min);
        (self.gauche as f64 + fraction * (self.droite - self.gauche) as f64).round() as i64
    }

    /// La ligne de pixel de `w` (l'axe vertical est logarithmique).
    fn y(&self, w: f64) -> i64 {
        self.y_log(logarithme(w))
    }

    /// La ligne de pixel du logarithme `log_w`.
    fn y_log(&self, log_w: f64) -> i64 {
        let fraction = (log_w - self.log_w_min) / (self.log_w_max - self.log_w_min);
        (self.bas as f64 - fraction * (self.bas - self.haut) as f64).round() as i64
    }
}

/// Le panneau : fond blanc, grille et repères, deux axes, puis les deux courbes et leurs points.
fn dessiner_panneau(toile: &mut Toile, cadre: &Cadre, courbes: &[Vec<f64>]) {
    toile.plein(cadre.gauche, cadre.haut, cadre.droite, cadre.bas, BLANC);

    // Les repères majeurs de l'axe horizontal : J/kT = 0,25 ; 0,5 ; … ; 2 (un tous les quatre seizièmes).
    for seiziemes in (J_MIN..=J_MAX).step_by(4) {
        let x = cadre.x(f64::from(seiziemes) / 16.0);
        toile.segment(x, cadre.haut, x, cadre.bas, GRILLE);
        toile.segment(x, cadre.bas + 1, x, cadre.bas + 5, AXE);
    }
    // Les petits repères : chaque seizième de J/kT.
    for seiziemes in J_MIN..=J_MAX {
        let x = cadre.x(f64::from(seiziemes) / 16.0);
        toile.segment(x, cadre.bas + 1, x, cadre.bas + 3, AXE);
    }

    // Les repères de l'axe vertical : les puissances de deux de w.
    let reperes_w = puissances_de_deux_entre(cadre.log_w_min, cadre.log_w_max);
    for &repere in &reperes_w {
        let y = cadre.y_log(repere);
        toile.segment(cadre.gauche, y, cadre.droite, y, GRILLE);
        toile.segment(cadre.gauche - 5, y, cadre.gauche - 1, y, AXE);
    }

    toile.segment(cadre.gauche, cadre.haut, cadre.gauche, cadre.bas, AXE);
    toile.segment(cadre.gauche, cadre.bas, cadre.droite, cadre.bas, AXE);

    for (indice, courbe) in courbes.iter().enumerate() {
        let points: Vec<(i64, i64)> = courbe
            .iter()
            .enumerate()
            .map(|(rang, &w)| {
                let seiziemes = J_MIN + rang as u32;
                (cadre.x(f64::from(seiziemes) / 16.0), cadre.y(w))
            })
            .collect();
        for fenetre in points.windows(2) {
            toile.segment(fenetre[0].0, fenetre[0].1, fenetre[1].0, fenetre[1].1, COULEURS[indice]);
        }
        for &(x, y) in &points {
            toile.carre(x, y, 1, COULEURS[indice]);
        }
    }
}

/// Une toile RVB de `largeur × hauteur` pixels, remplie d'une couleur au départ.
struct Toile {
    largeur: u32,
    hauteur: u32,
    pixels: Vec<u8>,
}

impl Toile {
    fn nouvelle(largeur: u32, hauteur: u32, fond: [u8; 3]) -> Toile {
        let mut pixels = Vec::with_capacity(3 * largeur as usize * hauteur as usize);
        for _ in 0..largeur as usize * hauteur as usize {
            pixels.extend_from_slice(&fond);
        }
        Toile { largeur, hauteur, pixels }
    }

    /// Peint le pixel `(x, y)`, en ignorant ce qui sort de la toile.
    fn poser(&mut self, x: i64, y: i64, couleur: [u8; 3]) {
        if x < 0 || y < 0 || x >= i64::from(self.largeur) || y >= i64::from(self.hauteur) {
            return;
        }
        let indice = 3 * (y as usize * self.largeur as usize + x as usize);
        self.pixels[indice..indice + 3].copy_from_slice(&couleur);
    }

    /// Remplit le rectangle `[x0, x1] × [y0, y1]`.
    fn plein(&mut self, x0: i64, y0: i64, x1: i64, y1: i64, couleur: [u8; 3]) {
        for y in y0.min(y1)..=y0.max(y1) {
            for x in x0.min(x1)..=x0.max(x1) {
                self.poser(x, y, couleur);
            }
        }
    }

    /// Trace un segment par l'algorithme de Bresenham.
    fn segment(&mut self, x0: i64, y0: i64, x1: i64, y1: i64, couleur: [u8; 3]) {
        let dx = (x1 - x0).abs();
        let sx = if x0 < x1 { 1 } else { -1 };
        let dy = -(y1 - y0).abs();
        let sy = if y0 < y1 { 1 } else { -1 };
        let mut erreur = dx + dy;
        let (mut x, mut y) = (x0, y0);
        loop {
            self.poser(x, y, couleur);
            if x == x1 && y == y1 {
                break;
            }
            let deux_erreurs = 2 * erreur;
            if deux_erreurs >= dy {
                erreur += dy;
                x += sx;
            }
            if deux_erreurs <= dx {
                erreur += dx;
                y += sy;
            }
        }
    }

    /// Peint le carré plein de côté `2 · rayon + 1` centré sur `(cx, cy)`.
    fn carre(&mut self, cx: i64, cy: i64, rayon: i64, couleur: [u8; 3]) {
        for y in cy - rayon..=cy + rayon {
            for x in cx - rayon..=cx + rayon {
                self.poser(x, y, couleur);
            }
        }
    }
}

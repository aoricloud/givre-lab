//! Les aperçus de l'étape 1 : l'amas de Witten et Sander coloré par ordre d'arrivée, et sa courbe log-log masse-rayon
//! avec la droite de pente 1,71 en repère. Des images pour l'œil, jamais une preuve : elles ne changent aucune référence.
//!
//! `apercus/etape-1-amas.png` : [`amas_colore`] de l'amas de graine 1, 4000 sites, réseau de demi-côté 200, chaque site
//! coloré par son rang d'arrivée (blanc pour le germe, rose pour le dernier site).
//!
//! `apercus/etape-1-loglog.png` : la masse dans le disque de rayon `r` autour du germe (`r = 2..=rayon` de l'amas), en
//! échelle log-log ([`points_masse_rayon`]), avec deux axes, leurs repères (les puissances de 2) et, en rouge pointillé,
//! une droite de pente 1,71 passant par le centre des points : un repère visuel, pas un ajustement. Les deux axes ont la
//! même échelle en pixels par unité de logarithme, pour que la pente se lise dans l'image.

use std::fs;
use std::path::PathBuf;

use givre_lab::dla::{Amas, COTE_APERCU, amas_colore};
use givre_lab::image::ecrire_png;
use givre_lab::math::logarithme;
use givre_lab::mesures::points_masse_rayon;

const GRAINE: u64 = 1;
const PARTICULES: usize = 4000;
const DEMI_COTE: u32 = 200;
const PENTE_REFERENCE: f64 = 1.71;

const FOND: [u8; 3] = [250, 250, 248];
const BLANC: [u8; 3] = [255, 255, 255];
const GRILLE: [u8; 3] = [232, 232, 232];
const AXE: [u8; 3] = [70, 70, 70];
const COURBE: [u8; 3] = [31, 119, 180];
const REFERENCE: [u8; 3] = [214, 39, 40];

/// Les pixels par unité de logarithme, communs aux deux axes : la pente se lit directement dans l'image.
const PIXELS_PAR_LOGARITHME: f64 = 90.0;
const MARGE_GAUCHE: i64 = 44;
const MARGE_DROITE: i64 = 16;
const MARGE_HAUT: i64 = 16;
const MARGE_BAS: i64 = 44;
/// La fraction de l'étendue des données ajoutée de chaque côté dans le cadre.
const MARGE_DONNEES: f64 = 0.06;
/// Les longueurs d'un tiret et d'un vide de la droite de repère, en pixels.
const TIRET: i64 = 7;
const VIDE: i64 = 5;

fn main() {
    let dossier = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("apercus");
    fs::create_dir_all(&dossier).expect("création du dossier apercus/");

    let mut amas = Amas::new(GRAINE, DEMI_COTE);
    amas.croitre_jusqu_a(PARTICULES);
    let sites = amas.sites();

    let (largeur, hauteur, rvb) = amas_colore(sites, COTE_APERCU);
    let png = ecrire_png(largeur, hauteur, &rvb);
    let chemin = dossier.join("etape-1-amas.png");
    fs::write(&chemin, &png).expect("écriture de l'aperçu de l'amas");
    println!("{} : {largeur} × {hauteur}, {} octets", chemin.display(), png.len());

    let png = apercu_loglog(sites, amas.rayon());
    let chemin = dossier.join("etape-1-loglog.png");
    fs::write(&chemin, &png).expect("écriture de l'aperçu log-log");
    println!("{} : {} octets", chemin.display(), png.len());
}

/// L'aperçu log-log : la masse dans le rayon contre le rayon, de 2 au rayon de l'amas.
fn apercu_loglog(sites: &[(i32, i32)], rayon: u32) -> Vec<u8> {
    let rayons: Vec<u32> = (2..=rayon).collect();
    let points = points_masse_rayon(sites, (0, 0), &rayons);
    let cadre = Cadre::autour(&points);
    let mut toile = Toile::nouvelle(cadre.largeur(), cadre.hauteur(), FOND);
    dessiner_panneau(&mut toile, &cadre, &points);
    ecrire_png(toile.largeur, toile.hauteur, &toile.pixels)
}

/// Les logarithmes des puissances de 2 comprises dans `[bas, haut]`.
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

/// Le cadre du tracé : les bords de la zone de tracé en pixels, et les bornes des logarithmes en `x` et en `y`.
struct Cadre {
    gauche: i64,
    droite: i64,
    haut: i64,
    bas: i64,
    x_min: f64,
    x_max: f64,
    y_min: f64,
    y_max: f64,
}

impl Cadre {
    /// Le cadre qui contient `points` (des couples de logarithmes) avec une marge de `MARGE_DONNEES` de chaque côté, aux
    /// mêmes pixels par unité de logarithme dans les deux directions.
    fn autour(points: &[(f64, f64)]) -> Cadre {
        let mut x_min = f64::INFINITY;
        let mut x_max = f64::NEG_INFINITY;
        let mut y_min = f64::INFINITY;
        let mut y_max = f64::NEG_INFINITY;
        for &(x, y) in points {
            x_min = x_min.min(x);
            x_max = x_max.max(x);
            y_min = y_min.min(y);
            y_max = y_max.max(y);
        }
        let marge_x = MARGE_DONNEES * (x_max - x_min);
        let marge_y = MARGE_DONNEES * (y_max - y_min);
        let x_min = x_min - marge_x;
        let x_max = x_max + marge_x;
        let y_min = y_min - marge_y;
        let y_max = y_max + marge_y;
        let largeur_zone = ((x_max - x_min) * PIXELS_PAR_LOGARITHME).round() as i64;
        let hauteur_zone = ((y_max - y_min) * PIXELS_PAR_LOGARITHME).round() as i64;
        Cadre {
            gauche: MARGE_GAUCHE,
            droite: MARGE_GAUCHE + largeur_zone,
            haut: MARGE_HAUT,
            bas: MARGE_HAUT + hauteur_zone,
            x_min,
            x_max,
            y_min,
            y_max,
        }
    }

    /// La largeur de l'image, marges comprises.
    fn largeur(&self) -> u32 {
        (self.droite + MARGE_DROITE) as u32
    }

    /// La hauteur de l'image, marges comprises.
    fn hauteur(&self) -> u32 {
        (self.bas + MARGE_BAS) as u32
    }

    /// La colonne de pixel du logarithme `log_x`.
    fn x(&self, log_x: f64) -> i64 {
        let fraction = (log_x - self.x_min) / (self.x_max - self.x_min);
        (self.gauche as f64 + fraction * (self.droite - self.gauche) as f64).round() as i64
    }

    /// La ligne de pixel du logarithme `log_y` (`y` croît vers le bas dans l'image).
    fn y(&self, log_y: f64) -> i64 {
        let fraction = (log_y - self.y_min) / (self.y_max - self.y_min);
        (self.bas as f64 - fraction * (self.bas - self.haut) as f64).round() as i64
    }
}

/// Le panneau : fond blanc, grille et repères, deux axes, la droite de pente 1,71 en repère, puis la courbe et ses points.
fn dessiner_panneau(toile: &mut Toile, cadre: &Cadre, points: &[(f64, f64)]) {
    toile.plein(cadre.gauche, cadre.haut, cadre.droite, cadre.bas, BLANC);
    let reperes_x = puissances_de_deux_entre(cadre.x_min, cadre.x_max);
    let reperes_y = puissances_de_deux_entre(cadre.y_min, cadre.y_max);

    // La grille, puis les deux axes et leurs repères (des traits à l'extérieur).
    for &repere in &reperes_x {
        let x = cadre.x(repere);
        toile.segment(x, cadre.haut, x, cadre.bas, GRILLE);
    }
    for &repere in &reperes_y {
        let y = cadre.y(repere);
        toile.segment(cadre.gauche, y, cadre.droite, y, GRILLE);
    }
    toile.segment(cadre.gauche, cadre.haut, cadre.gauche, cadre.bas, AXE);
    toile.segment(cadre.gauche, cadre.bas, cadre.droite, cadre.bas, AXE);
    for &repere in &reperes_x {
        let x = cadre.x(repere);
        toile.segment(x, cadre.bas + 1, x, cadre.bas + 5, AXE);
    }
    for &repere in &reperes_y {
        let y = cadre.y(repere);
        toile.segment(cadre.gauche - 5, y, cadre.gauche - 1, y, AXE);
    }

    dessiner_reference(toile, cadre, points);

    let pixels: Vec<(i64, i64)> = points.iter().map(|&(x, y)| (cadre.x(x), cadre.y(y))).collect();
    for fenetre in pixels.windows(2) {
        toile.segment(fenetre[0].0, fenetre[0].1, fenetre[1].0, fenetre[1].1, COURBE);
    }
    for &(x, y) in &pixels {
        toile.carre(x, y, 2, COURBE);
    }
}

/// La droite de pente `PENTE_REFERENCE` passant par le centre des points, en pointillé : un repère visuel, pas un
/// ajustement (la courbe mesurée de l'amas de 4000 sites est un peu moins pentue).
fn dessiner_reference(toile: &mut Toile, cadre: &Cadre, points: &[(f64, f64)]) {
    let nombre = points.len() as f64;
    let x_centre = points.iter().map(|point| point.0).sum::<f64>() / nombre;
    let y_centre = points.iter().map(|point| point.1).sum::<f64>() / nombre;

    let pixel_debut = cadre.x(cadre.x_min);
    let pixel_fin = cadre.x(cadre.x_max);
    let periode = TIRET + VIDE;
    for pixel in pixel_debut..=pixel_fin {
        if (pixel - pixel_debut).rem_euclid(periode) >= TIRET {
            continue;
        }
        let fraction = (pixel - pixel_debut) as f64 / (pixel_fin - pixel_debut) as f64;
        let log_x = cadre.x_min + fraction * (cadre.x_max - cadre.x_min);
        let log_y = y_centre + PENTE_REFERENCE * (log_x - x_centre);
        toile.poser(pixel, cadre.y(log_y), REFERENCE);
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

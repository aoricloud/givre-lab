//! Les aperçus de l'étape 2s : les quatre fronts de surface côte à côte, et leurs courbes de rugosité en échelle log-log.
//! Des images pour l'œil, jamais une preuve : elles ne changent aucune référence.
//!
//! `apercus/etape-2s-fronts.png` montre les quatre modèles (dépôt aléatoire, relaxation, balistique, Eden) sur `L = 128`
//! colonnes et `96` couches, graine 1, chaque panneau suivant le principe de l'aperçu de `lancer` : une rangée remplie est
//! colorée selon la couche où le front l'a atteinte. `apercus/etape-2s-rugosite.png` montre `w(t)` en échelle log-log pour
//! `L = 256`, aux instants `1, 2, 4, …, 256`, `w²` moyenné sur huit graines, un panneau, un axe et une couleur par modèle.

use std::fs;
use std::path::PathBuf;

use givre_lab::image::ecrire_png;
use givre_lab::math::logarithme;
use givre_lab::surface::{Modele, Surface, rugosites_moyennes};

const LARGEUR_FRONTS: usize = 128;
const COUCHES_FRONTS: u64 = 96;
const GRAINE_FRONTS: u64 = 1;
const FOND_FRONTS: [u8; 3] = [240, 240, 240];
const SEPARATEUR_FRONTS: [u8; 3] = [205, 205, 205];
const LARGEUR_SEPARATEUR: u32 = 1;

const LARGEUR_RUGOSITE: usize = 256;
const GRAINES_RUGOSITE: [u64; 8] = [1, 2, 3, 4, 5, 6, 7, 8];
const INSTANTS_RUGOSITE: [u64; 9] = [1, 2, 4, 8, 16, 32, 64, 128, 256];

const PANNEAU_LARGEUR: u32 = 256;
const PANNEAU_HAUTEUR: u32 = 256;
const COLONNES_PANNEAUX: u32 = 2;
const RANGEES_PANNEAUX: u32 = 2;
const MARGE_GAUCHE: i64 = 34;
const MARGE_DROITE: i64 = 14;
const MARGE_HAUT: i64 = 14;
const MARGE_BAS: i64 = 30;

const FOND: [u8; 3] = [250, 250, 248];
const BLANC: [u8; 3] = [255, 255, 255];
const GRILLE: [u8; 3] = [232, 232, 232];
const AXE: [u8; 3] = [70, 70, 70];
/// Les couleurs des quatre modèles, dans l'ordre de `Modele::TOUS`.
const COULEURS: [[u8; 3]; 4] = [[31, 119, 180], [255, 127, 14], [44, 160, 44], [148, 103, 189]];

fn main() {
    let dossier = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("apercus");
    fs::create_dir_all(&dossier).expect("création du dossier apercus/");

    let fronts = apercu_fronts();
    let chemin = dossier.join("etape-2s-fronts.png");
    fs::write(&chemin, &fronts).expect("écriture de l'aperçu des fronts");
    println!("{} : {} octets", chemin.display(), fronts.len());

    let rugosite = apercu_rugosite();
    let chemin = dossier.join("etape-2s-rugosite.png");
    fs::write(&chemin, &rugosite).expect("écriture de l'aperçu des rugosités");
    println!("{} : {} octets", chemin.display(), rugosite.len());
}

/// Un front simulé : les hauteurs finales et, pour chaque colonne, la couche d'arrivée de chaque rangée.
struct Front {
    hauteurs: Vec<u64>,
    couche_de: Vec<Vec<u32>>,
}

/// Le front d'un modèle après `COUCHES_FRONTS` couches complètes, graine 1. `couche_de[x][r]` est la première couche
/// complète après laquelle `h[x] > r`, comme dans l'aperçu de `lancer` (la hauteur ne décroît jamais).
fn simuler_front(modele: Modele) -> Front {
    let mut surface = Surface::new(modele, LARGEUR_FRONTS, GRAINE_FRONTS);
    let mut couche_de = vec![Vec::new(); LARGEUR_FRONTS];
    let mut precedent = vec![0_u64; LARGEUR_FRONTS];
    for t in 1..=COUCHES_FRONTS {
        surface.deposer_couche();
        let hauteurs = surface.hauteurs();
        for (x, &hauteur) in hauteurs.iter().enumerate() {
            while precedent[x] < hauteur {
                couche_de[x].push(t as u32);
                precedent[x] += 1;
            }
        }
    }
    Front { hauteurs: surface.hauteurs(), couche_de }
}

/// L'aperçu des quatre fronts : les quatre aperçus de `lancer` (chacun de la hauteur de son propre front) posés côte à
/// côte et alignés sur le bas, séparés par un trait clair. Dans un panneau, au-dessus du front, le fond `(240, 240, 240)` ;
/// en dessous, la couleur `(v, 96, 255 - v)` de la couche d'arrivée, `v = (a - 1) · 255 / (COUCHES_FRONTS - 1)`. Hors
/// d'un panneau (au-dessus d'un front plus bas que le plus haut), la toile est blanche.
fn apercu_fronts() -> Vec<u8> {
    let fronts: Vec<Front> = Modele::TOUS.iter().map(|&modele| simuler_front(modele)).collect();
    let hauteurs_panneaux: Vec<u64> = fronts
        .iter()
        .map(|front| front.hauteurs.iter().copied().max().expect("une surface a au moins une colonne").max(1))
        .collect();
    let hauteur_image = hauteurs_panneaux.iter().copied().max().expect("quatre panneaux") as u32;
    let largeur_image = LARGEUR_FRONTS as u32 * fronts.len() as u32 + LARGEUR_SEPARATEUR * (fronts.len() as u32 - 1);
    let mut rvb = Vec::with_capacity(3 * largeur_image as usize * hauteur_image as usize);
    for y in 0..hauteur_image {
        let rangee = u64::from(hauteur_image - 1 - y);
        for (p, front) in fronts.iter().enumerate() {
            for (&hauteur, couche_de) in front.hauteurs.iter().zip(&front.couche_de) {
                if rangee >= hauteurs_panneaux[p] {
                    rvb.extend_from_slice(&BLANC);
                } else if rangee >= hauteur {
                    rvb.extend_from_slice(&FOND_FRONTS);
                } else {
                    let arrivee = u64::from(couche_de[rangee as usize]);
                    let v = (arrivee - 1) * 255 / COUCHES_FRONTS.saturating_sub(1).max(1);
                    rvb.extend_from_slice(&[v as u8, 96, (255 - v) as u8]);
                }
            }
            if p + 1 < fronts.len() {
                for _ in 0..LARGEUR_SEPARATEUR {
                    rvb.extend_from_slice(&SEPARATEUR_FRONTS);
                }
            }
        }
    }
    ecrire_png(largeur_image, hauteur_image, &rvb)
}

/// Les courbes `w(t)` des quatre modèles, dans l'ordre de `Modele::TOUS` : `w²` moyenné sur les huit graines, puis la
/// racine.
fn courbes_de_rugosite() -> Vec<Vec<f64>> {
    Modele::TOUS
        .iter()
        .map(|&modele| {
            rugosites_moyennes(modele, LARGEUR_RUGOSITE, &GRAINES_RUGOSITE, &INSTANTS_RUGOSITE)
                .into_iter()
                .map(|carre| carre.sqrt())
                .collect()
        })
        .collect()
}

/// L'aperçu des rugosités : quatre panneaux, un par modèle, chacun avec ses axes, en échelle log-log, sans texte. Les
/// repères verticaux sont les instants (les puissances de 2), les repères horizontaux les puissances de 2 de `w`.
fn apercu_rugosite() -> Vec<u8> {
    let courbes = courbes_de_rugosite();

    // Les bornes verticales communes : les logarithmes de tous les `w`, avec une petite marge.
    let mut bas = f64::INFINITY;
    let mut haut = f64::NEG_INFINITY;
    for courbe in &courbes {
        for &w in courbe {
            let logarithme_w = logarithme(w);
            bas = bas.min(logarithme_w);
            haut = haut.max(logarithme_w);
        }
    }
    let marge = (haut - bas) / 20.0;
    let bas = bas - marge;
    let haut = haut + marge;

    let log_t_min = logarithme(INSTANTS_RUGOSITE[0] as f64);
    let log_t_max = logarithme(INSTANTS_RUGOSITE[INSTANTS_RUGOSITE.len() - 1] as f64);
    let marge_t = (log_t_max - log_t_min) / 25.0;
    let log_t_min = log_t_min - marge_t;
    let log_t_max = log_t_max + marge_t;
    let reperes_t: Vec<f64> = INSTANTS_RUGOSITE.iter().map(|&t| logarithme(t as f64)).collect();
    let reperes_w = puissances_de_deux_entre(bas, haut);

    let largeur_image = PANNEAU_LARGEUR * COLONNES_PANNEAUX;
    let hauteur_image = PANNEAU_HAUTEUR * RANGEES_PANNEAUX;
    let mut toile = Toile::nouvelle(largeur_image, hauteur_image, FOND);
    for (indice, courbe) in courbes.iter().enumerate() {
        let cadre = Cadre::panneau(indice, log_t_min, log_t_max, bas, haut);
        dessiner_panneau(&mut toile, &cadre, courbe, &reperes_t, &reperes_w, COULEURS[indice]);
    }
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

/// Le cadre d'un panneau : les bords de la zone de tracé en pixels, et les bornes des logarithmes en `x` et en `y`.
struct Cadre {
    gauche: i64,
    droite: i64,
    haut: i64,
    bas: i64,
    log_t_min: f64,
    log_t_max: f64,
    log_w_min: f64,
    log_w_max: f64,
}

impl Cadre {
    /// Le cadre du panneau `indice`, dans la grille `COLONNES_PANNEAUX × RANGEES_PANNEAUX`.
    fn panneau(indice: usize, log_t_min: f64, log_t_max: f64, log_w_min: f64, log_w_max: f64) -> Cadre {
        let colonne = i64::from(indice as u32 % COLONNES_PANNEAUX);
        let rangee = i64::from(indice as u32 / COLONNES_PANNEAUX);
        let origine_x = colonne * i64::from(PANNEAU_LARGEUR);
        let origine_y = rangee * i64::from(PANNEAU_HAUTEUR);
        Cadre {
            gauche: origine_x + MARGE_GAUCHE,
            droite: origine_x + i64::from(PANNEAU_LARGEUR) - MARGE_DROITE,
            haut: origine_y + MARGE_HAUT,
            bas: origine_y + i64::from(PANNEAU_HAUTEUR) - MARGE_BAS,
            log_t_min,
            log_t_max,
            log_w_min,
            log_w_max,
        }
    }

    /// La colonne de pixel du logarithme du temps `log_t`.
    fn x(&self, log_t: f64) -> i64 {
        let fraction = (log_t - self.log_t_min) / (self.log_t_max - self.log_t_min);
        (self.gauche as f64 + fraction * (self.droite - self.gauche) as f64).round() as i64
    }

    /// La ligne de pixel du logarithme de la rugosité `log_w`.
    fn y(&self, log_w: f64) -> i64 {
        let fraction = (log_w - self.log_w_min) / (self.log_w_max - self.log_w_min);
        (self.bas as f64 - fraction * (self.bas - self.haut) as f64).round() as i64
    }
}

/// Un panneau : le fond blanc, la grille, les deux axes et leurs repères, puis la courbe et ses points.
fn dessiner_panneau(
    toile: &mut Toile,
    cadre: &Cadre,
    courbe: &[f64],
    reperes_t: &[f64],
    reperes_w: &[f64],
    couleur: [u8; 3],
) {
    toile.plein(cadre.gauche, cadre.haut, cadre.droite, cadre.bas, BLANC);
    for &repere in reperes_t {
        let x = cadre.x(repere);
        toile.segment(x, cadre.haut, x, cadre.bas, GRILLE);
    }
    for &repere in reperes_w {
        let y = cadre.y(repere);
        toile.segment(cadre.gauche, y, cadre.droite, y, GRILLE);
    }
    toile.segment(cadre.gauche, cadre.haut, cadre.gauche, cadre.bas, AXE);
    toile.segment(cadre.gauche, cadre.bas, cadre.droite, cadre.bas, AXE);
    for &repere in reperes_t {
        let x = cadre.x(repere);
        toile.segment(x, cadre.bas + 1, x, cadre.bas + 4, AXE);
    }
    for &repere in reperes_w {
        let y = cadre.y(repere);
        toile.segment(cadre.gauche - 4, y, cadre.gauche - 1, y, AXE);
    }

    let points: Vec<(i64, i64)> = INSTANTS_RUGOSITE
        .iter()
        .zip(courbe)
        .map(|(&t, &w)| (cadre.x(logarithme(t as f64)), cadre.y(logarithme(w))))
        .collect();
    for fenetre in points.windows(2) {
        toile.segment(fenetre[0].0, fenetre[0].1, fenetre[1].0, fenetre[1].1, couleur);
    }
    for &(x, y) in &points {
        toile.carre(x, y, 2, couleur);
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

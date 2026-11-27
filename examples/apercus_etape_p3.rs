//! L'aperçu de l'étape P3 : la même dendrite à quatre anisotropies, côte à côte, au même temps. Une image pour l'œil, jamais une
//! preuve : elle ne change aucune référence (les valeurs sont dans `references/etape-p3-valeurs.md`).
//!
//! `apercus/etape-p3-anisotropie.png` : la graine ronde de la table II de Kim et coll. (`Delta = 0,65`, `d0 = 0,5`, `D = 1`, `W0 = 1`,
//! rayon 15, `u` joué), quart de 150 × 150 cases au pas `dx = 0,8` (`dt = 25 / ceil(25 / dt_borne)`), au temps `t = 800`, pour
//! `ε` = 0 ; 0,02 ; 0,04 ; 0,06, de gauche à droite. Chaque panneau est le domaine entier recomposé par les deux miroirs du quart
//! (300 × 300 pixels, un pixel par case, `y` vers le haut), `φ` en dégradé du bleu foncé (solide, `φ = 1`) au blanc (liquide,
//! `φ = -1`) ; en rouge, la parabole ajustée à la pointe (`mesurer_pointe`, profondeur 2), tracée sur les quatre bras jusqu'à la
//! profondeur 12 ; un filet blanc de 4 pixels entre les panneaux (1212 × 300 pixels, environ 1,1 Mo : `image.rs` ne compresse pas).
//! Ce qu'on y voit : la même graine, plus ou moins effilée ; à `ε = 0`, pas de dendrite (le régime établi n'existe pas sans
//! anisotropie). Le programme écrit aussi, pour chaque `ε`, la position, le rayon, la vitesse (de `t = 775` à 800) et `σ*`.
//!
//! Le calcul dure de 20 à 40 secondes (quatre dendrites l'une après l'autre) : à lancer **au premier plan, avec un délai**. Usage :
//! `cargo run --release --example apercus_etape_p3 [dossier]`.

use std::fs;
use std::path::PathBuf;
use std::time::Instant;

use givre_lab::dendrite::{Dendrite, Domaine, ModeU, Reglages};
use givre_lab::image::ecrire_png;
use givre_lab::pointe::{constante_de_selection, mesurer_pointe, vitesse_de_pointe};

const EPSILONS: [f64; 4] = [0.0, 0.02, 0.04, 0.06];
const N: usize = 150;
const DX: f64 = 0.8;
const TEMPS: f64 = 800.0;
const PERIODE: f64 = 25.0;
const FILET: usize = 4;

/// La couleur d'une case : du bleu foncé `(20, 45, 120)` pour `φ = 1` au blanc pour `φ = -1`, en entiers.
fn couleur(phi: f64) -> [u8; 3] {
    let c = ((1.0 - phi.clamp(-1.0, 1.0)) * 0.5 * 255.0).round() as u32; // 0 : solide, 255 : liquide
    let melange = |fonce: u32| ((fonce * (255 - c) + 255 * c) / 255) as u8;
    [melange(20), melange(45), melange(120)]
}

/// L'indice de la case du quart pour l'indice `k` du domaine entier de côté `2 N` (les miroirs des axes).
fn quart(k: usize) -> usize {
    if k >= N { k - N } else { N - 1 - k }
}

fn main() {
    let arguments: Vec<String> = std::env::args().collect();
    let dossier =
        arguments.get(1).map_or_else(|| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("apercus"), PathBuf::from);
    fs::create_dir_all(&dossier).expect("création du dossier");
    let cote = 2 * N;
    let largeur = EPSILONS.len() * cote + (EPSILONS.len() - 1) * FILET;
    let mut rvb = vec![255_u8; 3 * largeur * cote];
    let debut = Instant::now();
    for (k, &epsilon) in EPSILONS.iter().enumerate() {
        let r0 = Reglages::depuis_capillarite(1.0, 0.5, 1.0, epsilon, DX, 1.0);
        let pas_par_periode = (PERIODE / r0.dt_borne(0.65, ModeU::Joue)).ceil() as u64;
        let r = Reglages { dt: PERIODE / pas_par_periode as f64, ..r0 };
        let mut d = Dendrite::graine_ronde(r, N, Domaine::Quart, 0.65, 15.0, ModeU::Joue).expect("graine valide");
        let periodes = (TEMPS / PERIODE).round() as u64;
        d.avancer((periodes - 1) * pas_par_periode).expect("pas");
        let avant = mesurer_pointe(&d, 2.0).expect("la pointe avant");
        d.avancer(pas_par_periode).expect("pas");
        let p = mesurer_pointe(&d, 2.0).expect("la pointe");
        let v = vitesse_de_pointe(&avant, &p).expect("la vitesse");
        let sigma = constante_de_selection(0.5, 1.0, p.rayon, v).unwrap_or(f64::NAN);
        println!(
            "epsilon = {epsilon} : t = {:.1}, position {:.4}, {} lignes, rayon {:.4}, vitesse {:.5}, sigma {:.5}",
            p.temps, p.position, p.lignes, p.rayon, v, sigma
        );
        let x0 = k * (cote + FILET);
        let phi = d.phi();
        for py in 0..cote {
            let i = quart(cote - 1 - py);
            for px in 0..cote {
                let j = quart(px);
                let o = 3 * (py * largeur + x0 + px);
                rvb[o..o + 3].copy_from_slice(&couleur(phi[i * N + j]));
            }
        }
        // la parabole ajustée, sur les quatre bras : x = sommet - y² / (2 rayon), jusqu'à la profondeur 12
        let mut point = |x: f64, y: f64| {
            let (a, b) = ((x / DX + N as f64).floor(), (y / DX + N as f64).floor());
            if a >= 0.0 && b >= 0.0 && (a as usize) < cote && (b as usize) < cote {
                let o = 3 * ((cote - 1 - b as usize) * largeur + x0 + a as usize);
                rvb[o..o + 3].copy_from_slice(&[220, 30, 30]);
            }
        };
        let pas_y = DX / 8.0;
        let mut y = 0.0;
        while (y * y) / (2.0 * p.rayon) <= 12.0 {
            let x = p.sommet - (y * y) / (2.0 * p.rayon);
            for (sx, sy) in [(1.0, 1.0), (1.0, -1.0), (-1.0, 1.0), (-1.0, -1.0)] {
                point(sx * x, sy * y);
                point(sy * y, sx * x);
            }
            y += pas_y;
        }
    }
    let png = ecrire_png(largeur as u32, cote as u32, &rvb);
    let chemin = dossier.join("etape-p3-anisotropie.png");
    fs::write(&chemin, &png).expect("écriture de l'aperçu");
    println!(
        "{} : {} octets, calculé et dessiné en {:.1} s",
        chemin.display(),
        png.len(),
        debut.elapsed().as_secs_f64()
    );
}

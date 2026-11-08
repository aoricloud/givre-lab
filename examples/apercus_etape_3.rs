//! Les aperçus de l'étape 3 : une planche de neuf flocons de Reiter selon β et γ, et la plaque du scénario
//! `scenarios/reiter-plaque.txt`. Des images pour l'œil, jamais une preuve : elles ne changent aucune référence.
//!
//! `apercus/etape-3-planche.png` : neuf flocons (`α = 1`, hexagone de rayon 60, 250 pas) en grille 3 × 3, β dans
//! `0.3, 0.45, 0.6` en colonnes et γ dans `0.001, 0.01, 0.1` en lignes, chacun l'image RVB de [`Flocon::apercu`],
//! séparés par un filet d'un pixel.
//! `apercus/etape-3-plaque.png` : l'aperçu, écrit tel quel, du scénario `scenarios/reiter-plaque.txt` (hexagone de rayon
//! 30, `α = 1`, `β = 0`, `γ = 0.05`, 400 pas).

use std::fs;
use std::path::PathBuf;

use givre_lab::flocon::{Flocon, Parametres};
use givre_lab::image::ecrire_png;
use givre_lab::lancer::lancer;
use givre_lab::scenario::lire_scenario;

/// Le rayon de l'hexagone de chaque flocon de la planche.
const RAYON: u32 = 60;
/// Le nombre de pas de chaque flocon de la planche.
const PAS: u64 = 250;
/// Les β de la planche, en colonnes.
const BETAS: [&str; 3] = ["0.3", "0.45", "0.6"];
/// Les γ de la planche, en lignes.
const GAMMAS: [&str; 3] = ["0.001", "0.01", "0.1"];
/// La diffusion, celle de l'article.
const ALPHA: &str = "1";
/// La graine du lancer : sans effet sur un flocon de Reiter (le modèle n'a aucun hasard).
const GRAINE: u64 = 1;
/// La couleur du filet entre les panneaux de la planche.
const FILET: [u8; 3] = [205, 205, 205];
/// L'épaisseur du filet, en pixels.
const FILET_EPAISSEUR: u32 = 1;

fn main() {
    let dossier = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("apercus");
    fs::create_dir_all(&dossier).expect("création du dossier apercus/");

    let planche = apercu_planche();
    let chemin = dossier.join("etape-3-planche.png");
    fs::write(&chemin, &planche).expect("écriture de la planche");
    println!("{} : {} octets", chemin.display(), planche.len());

    let plaque = apercu_plaque();
    let chemin = dossier.join("etape-3-plaque.png");
    fs::write(&chemin, &plaque).expect("écriture de l'aperçu de la plaque");
    println!("{} : {} octets", chemin.display(), plaque.len());
}

/// La planche 3 × 3 : `BETAS` en colonnes, `GAMMAS` en lignes, chaque panneau étant l'aperçu de [`Flocon::apercu`] de
/// l'état final après `PAS` pas, posé à sa place et séparé de ses voisins par un filet de `FILET_EPAISSEUR` pixels.
fn apercu_planche() -> Vec<u8> {
    let cote = 2 * RAYON + 1;
    let largeur = 3 * cote + 2 * FILET_EPAISSEUR;
    let hauteur = largeur;
    let mut rvb = Vec::with_capacity(3 * largeur as usize * hauteur as usize);
    for _ in 0..largeur as usize * hauteur as usize {
        rvb.extend_from_slice(&FILET);
    }
    for (rangee, &gamma) in GAMMAS.iter().enumerate() {
        for (colonne, &beta) in BETAS.iter().enumerate() {
            let parametres = Parametres::lire(ALPHA, beta, gamma).expect("paramètres valides");
            let mut flocon = Flocon::new(RAYON, parametres);
            for _ in 0..PAS {
                flocon.pas();
            }
            let panneau = flocon.apercu();
            let x0 = colonne as u32 * (cote + FILET_EPAISSEUR);
            let y0 = rangee as u32 * (cote + FILET_EPAISSEUR);
            for y in 0..cote {
                let source = 3 * (y * cote) as usize;
                let destination = 3 * (((y0 + y) * largeur + x0) as usize);
                rvb[destination..destination + 3 * cote as usize]
                    .copy_from_slice(&panneau[source..source + 3 * cote as usize]);
            }
        }
    }
    ecrire_png(largeur, hauteur, &rvb)
}

/// L'aperçu du scénario de la plaque, écrit tel quel : `lire_scenario` puis `lancer`, dont on garde le PNG.
fn apercu_plaque() -> Vec<u8> {
    let chemin = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("scenarios").join("reiter-plaque.txt");
    let texte = fs::read_to_string(&chemin).expect("lecture du scénario de la plaque");
    let scenario = lire_scenario(&texte).expect("scénario de la plaque bien formé");
    lancer(&scenario, GRAINE).expect("lancer de la plaque").apercu
}

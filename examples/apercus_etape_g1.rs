//! Les aperçus nets de l'étape G1, calculés hors ligne : un cristal (`apercus/etape-g1-cristal.png`) et une planche qui fait varier `ρ`
//! (`apercus/etape-g1-planche.png`), dessinés par `rendu_hex` (des hexagones à leur vraie géométrie, un dégradé de bleus sur fond blanc,
//! du suréchantillonnage en entiers). Des images pour l'œil, jamais une preuve : elles ne changent aucune référence.
//!
//! Les réglages sont ceux de la figure 9 de Gravner et Griffeath (annexe 8, ligne « 9 » : `β = 1,3`, `α = 0,08`, `θ = 0,025`, `κ = 0,003`,
//! `μ = 0,07`, `γ = 0,00005`), avec `ρ` de 0,40 à 0,50 par pas de 0,01 (la figure 9 montre 0,40 0,41 0,42 0,44 0,46 0,50). Chaque cristal grandit
//! dans un hexagone de rayon `RAYON_DOMAINE` jusqu'à ce que son rayon atteigne `RAYON_CIBLE` (comme l'article : « comparables en taille »), ou
//! jusqu'à `PAS_MAX` pas.
//!
//! Usage : `cargo run --release --example apercus_etape_g1 [dossier [planche|cristal]]`

use std::fs;
use std::path::PathBuf;
use std::time::Instant;

use givre_lab::image::ecrire_png;
use givre_lab::math::Fixe;
use givre_lab::neige::{Neige, Parametres};
use givre_lab::rendu_hex::{ReglagesImage, rendre};

/// Le rayon de l'hexagone où grandit chaque cristal (1,5 fois le rayon cible : la règle de l'article pour une dendrite).
const RAYON_DOMAINE: u32 = 150;
/// Le rayon du cristal à atteindre.
const RAYON_CIBLE: u32 = 100;
/// Le nombre de pas au plus.
const PAS_MAX: u64 = 40_000;
/// Les `ρ` de la planche : 0,40 à 0,50 de deux centièmes en deux centièmes (six panneaux, trois colonnes sur deux rangées).
const RHOS: [&str; 6] = ["0.40", "0.42", "0.44", "0.46", "0.48", "0.50"];
/// Le côté d'un panneau de la planche, en pixels, et son suréchantillonnage.
const COTE_PANNEAU: u32 = 400;
const SUR_PANNEAU: u32 = 4;
/// Le côté du cristal seul, et son suréchantillonnage.
const COTE_CRISTAL: u32 = 1000;
const SUR_CRISTAL: u32 = 4;
/// La valeur de `c` qui donne le bleu le plus foncé : 1,5 (en `Fixe`). Les cases rattachées de ces réglages ont `c` entre 1,0 et 1,45 (une case de plaque
/// remplie après coup par la lame de couteau a `c` près de 0,1) : un blanc pour 0, un bleu clair pour les plaques minces, un bleu moyen pour 1,0 et un bleu plus
/// foncé pour 1,45 (le bleu le plus foncé du dégradé est atteint dès `c >= 1,5` : les cases les plus épaisses, pour un contraste net).
fn plein() -> i64 {
    Fixe::entier(3).0 / 2
}

fn reglages(rho: &str) -> Parametres {
    Parametres::lire(rho, "1.3", "0.08", "0.025", "0.003", "0.07", "0.00005").expect("réglages de la figure 9")
}

/// Fait grandir un cristal jusqu'à `RAYON_CIBLE` (ou `PAS_MAX` pas) ; rend la neige.
fn grandir(rho: &str) -> (Neige, u64) {
    let mut neige = Neige::essayer_new(RAYON_DOMAINE, reglages(rho)).expect("neige valide");
    while neige.rayon_cristal() < RAYON_CIBLE && neige.temps() < PAS_MAX {
        neige.avancer(50);
    }
    let pas = neige.temps();
    (neige, pas)
}

fn image(neige: &Neige, cote: u32, sur: u32) -> Vec<u8> {
    let reglages = ReglagesImage { cote, rayon_vue: RAYON_CIBLE + 8, sur, plein: plein() };
    rendre(neige.rayon(), &neige.valeurs_du_cristal(), &reglages).expect("rendu valide")
}

fn main() {
    let arguments: Vec<String> = std::env::args().collect();
    let dossier =
        arguments.get(1).map_or_else(|| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("apercus"), PathBuf::from);
    let quoi = arguments.get(2).map_or("tout", String::as_str);
    fs::create_dir_all(&dossier).expect("création du dossier");

    if quoi == "tout" || quoi == "cristal" {
        let debut = Instant::now();
        let (neige, pas) = grandir("0.46");
        let simule = debut.elapsed();
        let debut = Instant::now();
        let rvb = image(&neige, COTE_CRISTAL, SUR_CRISTAL);
        let png = ecrire_png(COTE_CRISTAL, COTE_CRISTAL, &rvb);
        let chemin = dossier.join("etape-g1-cristal.png");
        fs::write(&chemin, &png).expect("écriture du cristal");
        println!(
            "{} : ρ = 0,46, {pas} pas, rayon {}, {} cases rattachées ; simulé en {:.1} s, dessiné ({COTE_CRISTAL} px, {SUR_CRISTAL}x{SUR_CRISTAL}) en {:.2} s, {} octets",
            chemin.display(),
            neige.rayon_cristal(),
            neige.cases_rattachees(),
            simule.as_secs_f64(),
            debut.elapsed().as_secs_f64(),
            png.len()
        );
    }

    if quoi == "tout" || quoi == "planche" {
        let colonnes = 3_u32;
        let rangees = (RHOS.len() as u32).div_ceil(colonnes);
        let largeur = colonnes * COTE_PANNEAU;
        let hauteur = rangees * COTE_PANNEAU;
        let mut rvb = vec![255_u8; 3 * largeur as usize * hauteur as usize];
        let (mut total_simule, mut total_dessine) = (0.0, 0.0);
        for (k, rho) in RHOS.iter().enumerate() {
            let debut = Instant::now();
            let (neige, pas) = grandir(rho);
            total_simule += debut.elapsed().as_secs_f64();
            let debut = Instant::now();
            let panneau = image(&neige, COTE_PANNEAU, SUR_PANNEAU);
            total_dessine += debut.elapsed().as_secs_f64();
            println!(
                "ρ = {rho} : {pas} pas, rayon {}, {} cases rattachées",
                neige.rayon_cristal(),
                neige.cases_rattachees()
            );
            let (x0, y0) = ((k as u32 % colonnes) * COTE_PANNEAU, (k as u32 / colonnes) * COTE_PANNEAU);
            for y in 0..COTE_PANNEAU {
                let source = 3 * (y * COTE_PANNEAU) as usize;
                let destination = 3 * (((y0 + y) * largeur + x0) as usize);
                rvb[destination..destination + 3 * COTE_PANNEAU as usize]
                    .copy_from_slice(&panneau[source..source + 3 * COTE_PANNEAU as usize]);
            }
        }
        let png = ecrire_png(largeur, hauteur, &rvb);
        let chemin = dossier.join("etape-g1-planche.png");
        fs::write(&chemin, &png).expect("écriture de la planche");
        println!(
            "{} : {} cristaux, simulés en {total_simule:.1} s, dessinés en {total_dessine:.2} s, {largeur}x{hauteur} px, {} octets",
            chemin.display(),
            RHOS.len(),
            png.len()
        );
    }
}

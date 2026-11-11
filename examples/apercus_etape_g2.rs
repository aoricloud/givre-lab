//! Les aperçus de l'étape G2 : les trois balayages de la neige (`ρ`, `β`, `μ` : figures 9, 10 et 12 de Gravner et Griffeath), chacun une planche nette de six cristaux, et un
//! tableau CSV de leurs mesures. Des images et un tableau pour l'œil, jamais une preuve : ils ne changent aucune référence. L'étiquette de chaque cristal (plaque, secteurs,
//! dendrite, fougère) est **notre modèle** (`MODELE.md`, « L'étiquette ») : elle est écrite dans `apercus/LISEZ-MOI.md` et dans le CSV, pas dans l'image (`image.rs` ne dessine pas de texte).
//!
//! `apercus/etape-g2-rho.png`, `etape-g2-beta.png`, `etape-g2-mu.png` : le balayage de [`jeu_rho`], [`jeu_beta`], [`jeu_mu`] (domaine de rayon 100 ; 2000, 1300 et 2000 pas), un cristal par valeur,
//! dessiné par `rendu_hex::rendre` (des hexagones à leur vraie géométrie, un dégradé de bleus d'après la masse cristalline `c`, suréchantillonnage 4 × 4 en entiers), six panneaux de 300 × 300 pixels,
//! trois colonnes sur deux rangées (900 × 600 pixels, 1,6 Mo : `image.rs` n'écrit pas de compression), **même échelle pour les six panneaux** d'une planche (la vue montre un rayon de
//! `RAYON_VUE` cases : les cristaux lents sont donc plus petits que les rapides), les valeurs dans l'ordre de la lecture (de gauche à droite, puis de haut en bas), qui est celui de l'article.
//!
//! `apercus/etape-g2-mesures.csv` : [`EN_TETE_CSV`] puis une ligne par cristal, les trois balayages à la suite (18 lignes).
//!
//! Le calcul dure de 30 à 60 secondes en tout (les trois balayages) : à lancer **au premier plan, avec un délai**. Usage : `cargo run --release --example apercus_etape_g2 [dossier]`.

use std::fs;
use std::path::PathBuf;
use std::time::Instant;

use givre_lab::balayage::{
    EN_TETE_CSV, JeuDeBalayage, Resultat, classer, jeu_beta, jeu_mu, jeu_rho, ligne_csv, mesurer,
};
use givre_lab::image::ecrire_png;
use givre_lab::math::Fixe;
use givre_lab::neige::Neige;
use givre_lab::rendu_hex::{ReglagesImage, rendre};

/// Le côté d'un panneau, en pixels, et son suréchantillonnage.
const COTE_PANNEAU: u32 = 300;
const SUR_PANNEAU: u32 = 4;
/// Le rayon de la vue (en cases) de chaque planche : le plus grand rayon de cristal du balayage, plus 4 cases de marge.
const RAYON_VUE_RHO: u32 = 59;
const RAYON_VUE_BETA: u32 = 66;
const RAYON_VUE_MU: u32 = 63;

/// La valeur de `c` qui donne le bleu le plus foncé : 1,5 (comme à G1 : les cases rattachées ont `c` entre 1,0 et 1,5, une plaque mince a moins).
fn plein() -> i64 {
    Fixe::entier(3).0 / 2
}

/// La valeur en six décimales exactes (comme le CSV), pour l'affichage : aucun flottant.
fn ppm(valeur: Fixe) -> String {
    let ppm = (2_000_000 * i128::from(valeur.0) + (1_i128 << 32)) / (1_i128 << 33);
    format!("{}.{:06}", ppm / 1_000_000, ppm % 1_000_000)
}

/// Une planche : les six cristaux du jeu, trois colonnes sur deux rangées ; ajoute aussi leurs lignes au CSV. Chaque cristal est calculé **une seule fois** (`Neige::essayer_new`, `avancer`) et
/// rangé comme le fait [`givre_lab::balayage::balayage`] (`mesurer`, `classer`, `empreinte`) : les tests de l'étape vérifient que `balayage` rend exactement cela.
fn planche(jeu: &JeuDeBalayage, rayon_vue: u32, csv: &mut String) -> Vec<u8> {
    let colonnes = 3_u32;
    let rangees = (jeu.valeurs.len() as u32).div_ceil(colonnes);
    let (largeur, hauteur) = (colonnes * COTE_PANNEAU, rangees * COTE_PANNEAU);
    let mut rvb = vec![255_u8; 3 * largeur as usize * hauteur as usize];
    let debut = Instant::now();
    for (k, &valeur) in jeu.valeurs.iter().enumerate() {
        let mut neige = Neige::essayer_new(jeu.rayon, jeu.reglage.fixer(jeu.base, valeur)).expect("réglages valides");
        neige.avancer(jeu.pas);
        let mesures = mesurer(&neige);
        let resultat = Resultat { valeur, mesures, forme: classer(&mesures), empreinte: neige.empreinte() };
        csv.push_str(&ligne_csv(jeu.nom, jeu.rayon, jeu.pas, &resultat));
        let reglages = ReglagesImage { cote: COTE_PANNEAU, rayon_vue, sur: SUR_PANNEAU, plein: plein() };
        let panneau = rendre(neige.rayon(), &neige.valeurs_du_cristal(), &reglages).expect("rendu valide");
        let (x0, y0) = ((k as u32 % colonnes) * COTE_PANNEAU, (k as u32 / colonnes) * COTE_PANNEAU);
        for y in 0..COTE_PANNEAU {
            let source = 3 * (y * COTE_PANNEAU) as usize;
            let destination = 3 * (((y0 + y) * largeur + x0) as usize);
            rvb[destination..destination + 3 * COTE_PANNEAU as usize]
                .copy_from_slice(&panneau[source..source + 3 * COTE_PANNEAU as usize]);
        }
        println!(
            "{} = {} : rayon {}, {} cases rattachées, compacité {}, pointes {}, aiguilles {}, hexagonalité {}, forme {}",
            jeu.nom,
            ppm(valeur),
            mesures.rayon,
            mesures.rattachees,
            mesures.compacite,
            mesures.pointes,
            mesures.aiguilles,
            mesures.hexagonalite,
            resultat.forme.nom()
        );
    }
    println!("{} : calculé et dessiné en {:.1} s", jeu.nom, debut.elapsed().as_secs_f64());
    ecrire_png(largeur, hauteur, &rvb)
}

fn main() {
    let arguments: Vec<String> = std::env::args().collect();
    let dossier =
        arguments.get(1).map_or_else(|| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("apercus"), PathBuf::from);
    fs::create_dir_all(&dossier).expect("création du dossier");
    let mut csv = String::from(EN_TETE_CSV);
    for (jeu, rayon_vue) in [(jeu_rho(), RAYON_VUE_RHO), (jeu_beta(), RAYON_VUE_BETA), (jeu_mu(), RAYON_VUE_MU)] {
        let png = planche(&jeu, rayon_vue, &mut csv);
        let chemin = dossier.join(format!("etape-g2-{}.png", jeu.nom));
        fs::write(&chemin, &png).expect("écriture de la planche");
        println!("{} : {} octets", chemin.display(), png.len());
    }
    let chemin = dossier.join("etape-g2-mesures.csv");
    fs::write(&chemin, &csv).expect("écriture du CSV");
    println!("{} : {} lignes", chemin.display(), csv.lines().count());
}

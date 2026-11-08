//! Les aperçus de l'étape 2d : les dendrites du collage partiel et de la réduction du bruit. Des images pour l'œil,
//! jamais une preuve : elles ne changent aucune référence.
//!
//! `apercus/etape-2d-bruit.png` montre les amas de réduction du bruit `m = 1, 4, 16` côte à côte, de `m = 1` à
//! `m = 16` : les bras se rangent sur les axes quand `m` croît. `apercus/etape-2d-collage.png` montre le collage
//! partiel `p = 1, 0,5, 0,1` côte à côte : les branches s'épaississent quand `p` baisse.
//!
//! Chaque amas est grandi par `AmasVariante` (graine 1, `demi_cote` 250, 3000 sites) puis rendu par
//! [`amas_colore`] avec le côté visé [`COTE_APERCU`]. Les panneaux, centrés sur le germe, sont alignés par leur centre
//! vertical et séparés par un filet clair. Le panneau `p = 1` est l'amas `m = 1` : le collage « toujours » et
//! `Bruit(1)` ont la même dynamique.

use std::fs;
use std::path::PathBuf;

use givre_lab::dla::{COTE_APERCU, amas_colore};
use givre_lab::dla_variantes::{AmasVariante, Variante, collage_depuis_flottant};
use givre_lab::image::ecrire_png;

const GRAINE: u64 = 1;
const DEMI_COTE: u32 = 250;
const SITES: usize = 3000;

/// La couleur du filet entre deux panneaux.
const FILET: [u8; 3] = [205, 205, 205];
/// La largeur du filet, en pixels.
const LARGEUR_FILET: u32 = 2;
/// La couleur de la toile hors des panneaux.
const TOILE: [u8; 3] = [250, 250, 248];

fn main() {
    let dossier = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("apercus");
    fs::create_dir_all(&dossier).expect("création du dossier apercus/");

    // Les cinq amas : `m = 1` sert aussi de panneau `p = 1` (même dynamique).
    let bruit_1 = amas(Variante::Bruit(1), SITES);
    let bruit_4 = amas(Variante::Bruit(4), SITES);
    let bruit_16 = amas(Variante::Bruit(16), SITES);
    let collage_05 = amas(Variante::Collage(collage(0.5)), SITES);
    let collage_01 = amas(Variante::Collage(collage(0.1)), SITES);

    let bruit = composer(&[panneau(&bruit_1), panneau(&bruit_4), panneau(&bruit_16)]);
    let chemin = dossier.join("etape-2d-bruit.png");
    fs::write(&chemin, &bruit).expect("écriture de l'aperçu du bruit");
    println!("{} : {} octets", chemin.display(), bruit.len());

    let collage = composer(&[panneau(&bruit_1), panneau(&collage_05), panneau(&collage_01)]);
    let chemin = dossier.join("etape-2d-collage.png");
    fs::write(&chemin, &collage).expect("écriture de l'aperçu du collage");
    println!("{} : {} octets", chemin.display(), collage.len());
}

/// Le `q` Q0.32 du collage `p`, par la conversion du module : la même que la directive `collage`.
fn collage(p: f64) -> u64 {
    collage_depuis_flottant(p).expect("une probabilité de collage strictement positive et au plus 1")
}

/// Les sites de l'amas de variante `variante`, graine 1, après `sites` sites (germe compris).
fn amas(variante: Variante, sites: usize) -> Vec<(i32, i32)> {
    let mut amas = AmasVariante::new(GRAINE, DEMI_COTE, variante);
    amas.croitre_jusqu_a(sites);
    amas.sites().to_vec()
}

/// L'aperçu d'un amas : `amas_colore` au côté visé `COTE_APERCU`.
fn panneau(sites: &[(i32, i32)]) -> Panneau {
    let (largeur, hauteur, rvb) = amas_colore(sites, COTE_APERCU);
    Panneau { largeur, hauteur, rvb }
}

/// Un panneau RVB carré, ligne par ligne de haut en bas.
struct Panneau {
    largeur: u32,
    hauteur: u32,
    rvb: Vec<u8>,
}

/// Les panneaux côte à côte, alignés par leur centre vertical, séparés par un filet `FILET` de `LARGEUR_FILET`
/// pixels. La toile est `TOILE`.
fn composer(panneaux: &[Panneau]) -> Vec<u8> {
    assert!(!panneaux.is_empty(), "il faut au moins un panneau");
    let hauteur = panneaux.iter().map(|p| p.hauteur).max().expect("un panneau");
    let largeur: u32 = panneaux.iter().map(|p| p.largeur).sum::<u32>() + LARGEUR_FILET * (panneaux.len() as u32 - 1);
    let mut rvb = Vec::with_capacity(3 * largeur as usize * hauteur as usize);
    for _ in 0..largeur as usize * hauteur as usize {
        rvb.extend_from_slice(&TOILE);
    }

    let mut x0 = 0_u32;
    for (indice, panneau) in panneaux.iter().enumerate() {
        let y0 = (hauteur - panneau.hauteur) / 2;
        for y in 0..panneau.hauteur {
            let source = 3 * y as usize * panneau.largeur as usize;
            let cible = 3 * ((y0 + y) as usize * largeur as usize + x0 as usize);
            let ligne = 3 * panneau.largeur as usize;
            rvb[cible..cible + ligne].copy_from_slice(&panneau.rvb[source..source + ligne]);
        }
        x0 += panneau.largeur;
        if indice + 1 < panneaux.len() {
            for y in 0..hauteur {
                for x in x0..x0 + LARGEUR_FILET {
                    let indice = 3 * (y as usize * largeur as usize + x as usize);
                    rvb[indice..indice + 3].copy_from_slice(&FILET);
                }
            }
            x0 += LARGEUR_FILET;
        }
    }
    ecrire_png(largeur, hauteur, &rvb)
}

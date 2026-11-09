//! L'aperçu de l'étape D1 : le claquage diélectrique à `η = 0`, `1` et `2`, côte à côte. Une image pour l'œil, jamais une preuve : elle ne change aucune
//! référence.
//!
//! `apercus/etape-d1-planche.png` montre trois amas de 400 sites (graine 1, domaine de demi-côté 75) : `η = 0` est un disque plein (le modèle d'Eden),
//! `η = 1` un buisson (le claquage qui ressemble à la DLA), `η = 2` un éclair plus fin et plus étendu. Les trois panneaux ont la MÊME échelle (la même
//! taille de site en pixels, celle qui fait tenir l'amas le plus étendu dans le côté visé) et sont centrés sur le germe ; chaque site est coloré par son
//! ordre d'arrivée ([`couleur_du_rang`]), du blanc du germe au rose du dernier site. Un filet clair sépare les panneaux.

use std::fs;
use std::path::PathBuf;

use givre_lab::claquage::Claquage;
use givre_lab::dla::{COTE_APERCU, FOND, couleur_du_rang};
use givre_lab::image::ecrire_png;

const GRAINE: u64 = 1;
const DEMI_COTE: u32 = 75;
const SITES: usize = 400;

/// La couleur du filet entre deux panneaux, sa largeur en pixels et la couleur de la toile.
const FILET: [u8; 3] = [205, 205, 205];
const LARGEUR_FILET: usize = 2;

fn main() {
    let dossier = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("apercus");
    fs::create_dir_all(&dossier).expect("création du dossier apercus/");

    let amas: Vec<Vec<(i32, i32)>> = (0..=2)
        .map(|eta| {
            let mut c = Claquage::new(GRAINE, DEMI_COTE, eta);
            c.croitre_jusqu_a(SITES);
            println!("eta {eta} : {} sites, rayon {}", c.sites().len(), c.rayon());
            c.sites().to_vec()
        })
        .collect();

    // La fenêtre commune : le carré `[-m, m]²` qui contient les trois amas ; l'échelle fait tenir ce carré dans `COTE_APERCU` pixels.
    let m = amas.iter().flatten().map(|&(x, y)| x.abs().max(y.abs())).max().expect("des sites") as usize;
    let cote = 2 * m + 1;
    let echelle = (COTE_APERCU as usize / cote).max(1);
    let cote_pixels = cote * echelle;
    let largeur = 3 * cote_pixels + 2 * LARGEUR_FILET;

    let mut rvb = Vec::with_capacity(3 * largeur * cote_pixels);
    for _ in 0..largeur * cote_pixels {
        rvb.extend_from_slice(&FOND);
    }
    for (panneau, sites) in amas.iter().enumerate() {
        let x0 = panneau * (cote_pixels + LARGEUR_FILET);
        for (rang, &(x, y)) in sites.iter().enumerate() {
            let couleur = couleur_du_rang(rang, sites.len());
            let px = x0 + (x + m as i32) as usize * echelle;
            let py = (y + m as i32) as usize * echelle;
            for j in py..py + echelle {
                for i in px..px + echelle {
                    let indice = 3 * (j * largeur + i);
                    rvb[indice..indice + 3].copy_from_slice(&couleur);
                }
            }
        }
        if panneau < 2 {
            for j in 0..cote_pixels {
                for i in x0 + cote_pixels..x0 + cote_pixels + LARGEUR_FILET {
                    let indice = 3 * (j * largeur + i);
                    rvb[indice..indice + 3].copy_from_slice(&FILET);
                }
            }
        }
    }
    let png = ecrire_png(largeur as u32, cote_pixels as u32, &rvb);
    let chemin = dossier.join("etape-d1-planche.png");
    fs::write(&chemin, &png).expect("écriture de la planche");
    println!("{} : {largeur} × {cote_pixels}, {} octets", chemin.display(), png.len());
}

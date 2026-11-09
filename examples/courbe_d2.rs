//! L'aperçu de l'étape D2 : la courbe `D(η)` du claquage diélectrique à 100 sites, avec ses barres d'erreur, à côté des points publiés. Une image et un
//! tableau pour l'œil, jamais une preuve : ils ne changent aucune référence.
//!
//! `apercus/etape-d2-courbe.png` : [`png_de_la_courbe`] des quatre points `η = 0, 1, 2, 3` (100 sites chacun, graines 1 à 50, demi-côtés de
//! [`demi_cote_de_la_courbe`]) : les carrés bleus et leurs barres d'erreur sont la dimension moyenne du banc ; les carrés rouges, décalés vers la droite,
//! sont les repères publiés (la DLA à 1,71 pour `η = 1`, Hastings 2001 pour `η = 2` et `3`). **Le banc n'est pas « en accord » avec le publié** : ses
//! amas sont petits (100 sites) et sa méthode n'est pas celle de l'article ; l'écart a un signe qui dépend de `η`.
//!
//! `apercus/etape-d2-courbe.csv` : [`csv_de_la_courbe`] des mêmes points (`eta,taille,graines,D,erreur,publie,ecart`).
//!
//! Usage : `cargo run --example courbe_d2 [dossier [taille [graines]]]` (par défaut : `apercus/`, 100 sites, 50 graines). Les graines sont `1..=graines`.

use std::fs;
use std::path::PathBuf;

use givre_lab::courbe::{courbe, csv_de_la_courbe, png_de_la_courbe};

fn main() {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    let dossier =
        arguments.first().map_or_else(|| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("apercus"), PathBuf::from);
    let taille: usize = arguments.get(1).map_or(100, |t| t.parse().expect("la taille est un entier"));
    let nombre: u64 = arguments.get(2).map_or(50, |g| g.parse().expect("le nombre de graines est un entier"));
    fs::create_dir_all(&dossier).expect("création du dossier");

    let graines: Vec<u64> = (1..=nombre).collect();
    let points = courbe(&[0, 1, 2, 3], &graines, taille);

    let csv = csv_de_la_courbe(&points);
    let chemin = dossier.join("etape-d2-courbe.csv");
    fs::write(&chemin, &csv).expect("écriture du CSV");
    print!("{} :\n{csv}", chemin.display());

    let png = png_de_la_courbe(&points);
    let chemin = dossier.join("etape-d2-courbe.png");
    fs::write(&chemin, &png).expect("écriture de l'image");
    println!("{} : {} octets", chemin.display(), png.len());
}

//! Les aperçus de l'étape 0 : deux graines, deux images de « la graine seule », pour l'œil et jamais une preuve.
//!
//! Écrit `apercus/etape-0-graine-1.png` et `apercus/etape-0-graine-2.png`, deux PNG de 128 × 96 pixels dont chaque
//! pixel est `(v, v, v)`, `v` étant les 8 bits de poids fort d'un tirage de PCG32 (voir `rendu::apercu_graine`).

use std::fs;
use std::path::PathBuf;

use givre_lab::image::ecrire_png;
use givre_lab::rendu::apercu_graine;

const LARGEUR: u32 = 128;
const HAUTEUR: u32 = 96;
const GRAINES: [u64; 2] = [1, 2];

fn main() {
    let dossier = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("apercus");
    fs::create_dir_all(&dossier).expect("création du dossier apercus/");
    for graine in GRAINES {
        let rvb = apercu_graine(LARGEUR, HAUTEUR, graine);
        let png = ecrire_png(LARGEUR, HAUTEUR, &rvb);
        let chemin = dossier.join(format!("etape-0-graine-{graine}.png"));
        fs::write(&chemin, &png).expect("écriture de l'aperçu");
        println!("{} : {} octets", chemin.display(), png.len());
    }
}

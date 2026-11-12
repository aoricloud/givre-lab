//! Ce que les fichiers de test de l'étape V8 (la masse de chaque case de la neige) partagent : les constantes de la doc de `vitrine` (« La masse de chaque case »), **écrites ici à la main** ; l'ordre canonique des cases écrit **comme la
//! doc le dit** (deux boucles imbriquées : jamais lu dans `reseau::Hexagone`) ; les masses attendues d'une neige (`Neige::valeurs_du_cristal` décalé de 21 bits) ; et **le rastériseur de référence** : un second rendu de l'image nette, écrit
//! dans ce fichier **d'après la seule doc** (la géométrie, les échantillons, la case d'un point, le dégradé, le pixel : en entiers), qui n'emploie que les masses et jamais `rendu_hex`. Les tests en comparent les octets à ceux de `givre_nette_pixel`.
//!
//! Ce fichier n'est pas un test : il ne contient aucune fonction `#[test]`. Il est gelé avec les tests.
#![allow(dead_code)]

use std::path::PathBuf;

use givre_lab::empreinte::sha256_hex;
use givre_lab::neige::Neige;

/// Les aides de l'étape G3 (les dix-neuf jeux écrits à la main, la neige native en face, l'image nette attendue, Node).
#[path = "../etape_g3_commun/mod.rs"]
pub mod g3;

#[allow(unused_imports)]
pub use g3::{e5, v7};

// ---------------------------------------------------------------------------------------------------------------
// Les constantes de la doc, écrites à la main (V-V8-01).
// ---------------------------------------------------------------------------------------------------------------

/// `u32::MAX` : « pas de valeur » (hors du tampon, pas d'instantané).
pub const ABSENT: u32 = u32::MAX;
/// Le nombre de bits jetés du `brut` : l'unité est `2^-11`.
pub const DECALAGE: u32 = 21;
/// La masse qui donne le bleu le plus foncé : `NEIGE_PLEIN >> 21 = 3 × 2^31 / 2^21`.
pub const PLEIN_MASSE: u32 = 3072;
/// Le rayon du domaine de toute neige de la vitrine.
pub const RAYON: u32 = 100;
/// Le nombre de cases : `3 R (R + 1) + 1` pour `R = 100`.
pub const CASES: usize = 30_301;
/// L'indice du germe `(0, 0)` : `3 R (R + 1) / 2`.
pub const INDICE_DU_GERME: usize = 15_150;
/// La vue de l'image nette de la neige : `R · 65 / 100 + 4`.
pub const RAYON_VUE: u32 = 69;
/// L'inverse rationnel de `√3` de la doc : `A / B`.
pub const A: i64 = 1_239_850_262;
pub const B: i64 = 2_147_483_647;
/// Les cinq arrêts du dégradé, `t = 0, 256, 512, 768, 1024`.
pub const ARRETS: [[i64; 3]; 5] = [[255, 255, 255], [198, 219, 239], [107, 174, 214], [33, 113, 181], [8, 48, 107]];

// ---------------------------------------------------------------------------------------------------------------
// L'ordre canonique, comme la doc l'écrit.
// ---------------------------------------------------------------------------------------------------------------

/// Les cases de l'hexagone de rayon `rayon` dans l'ordre de la doc : `r` de `−R` à `R`, et dans chaque rangée `q` de `max(−R, −R − r)` à `min(R, R − r)`.
pub fn cases_dans_l_ordre(rayon: i64) -> Vec<(i64, i64)> {
    let mut cases = Vec::new();
    for r in -rayon..=rayon {
        for q in (-rayon).max(-rayon - r)..=rayon.min(rayon - r) {
            cases.push((q, r));
        }
    }
    cases
}

/// L'indice de la case `(q, r)` par la formule fermée de la doc : `avant(r) + (q − max(−R, −R − r))`.
pub fn indice_de(rayon: i64, q: i64, r: i64) -> usize {
    let avant = if r < 0 {
        (rayon + r) * (2 * rayon + 1) - (rayon - r + 1) * (rayon + r) / 2
    } else {
        rayon * (3 * rayon + 1) / 2 + r * (2 * rayon + 1) - r * (r - 1) / 2
    };
    usize::try_from(avant + q - (-rayon).max(-rayon - r)).expect("un indice positif")
}

// ---------------------------------------------------------------------------------------------------------------
// Les masses attendues.
// ---------------------------------------------------------------------------------------------------------------

/// Les masses de la doc d'une neige native : pour chaque case dans l'ordre canonique, le `brut` de `c` si la case est rattachée (0 sinon), décalé de 21 bits vers la droite.
pub fn masses_de(neige: &Neige) -> Vec<u32> {
    neige.valeurs_du_cristal().iter().map(|&brut| u32::try_from(brut >> DECALAGE).expect("une masse de u32")).collect()
}

/// Les cases **rattachées** d'une neige native, dans l'ordre canonique.
pub fn rattachees_de(neige: &Neige) -> Vec<bool> {
    neige.cases().iter().map(|case| case.a).collect()
}

/// Les octets d'une suite de masses (quatre octets chacune, petit-boutiste), pour la sommer.
pub fn octets_des_masses(masses: &[u32]) -> Vec<u8> {
    masses.iter().flat_map(|m| m.to_le_bytes()).collect()
}

/// Le SHA-256 des masses (voir `octets_des_masses`) : ce que `masses-wasm.mjs` écrit pour un tampon lu par `givre_neige_masse`.
pub fn somme_des_masses(masses: &[u32]) -> String {
    sha256_hex(&octets_des_masses(masses))
}

// ---------------------------------------------------------------------------------------------------------------
// Le rastériseur de référence : l'image nette, à partir des seules masses (la doc : « De la masse à l'image nette »).
// ---------------------------------------------------------------------------------------------------------------

/// `t` de la doc : `min(masse, 3072) / 3` (division entière), de 0 à 1024.
pub fn t_de_la_masse(masse: u32) -> i64 {
    i64::from(masse.min(PLEIN_MASSE) / 3)
}

/// La couleur du dégradé pour `t` (0 à 1024) : l'interpolation entière entre les cinq arrêts, écrite d'après la doc.
pub fn couleur_de_t(t: i64) -> [u8; 3] {
    assert!((0..=1024).contains(&t));
    if t == 1024 {
        return ARRETS[4].map(|c| c as u8);
    }
    let (s, f) = ((t / 256) as usize, t % 256);
    let mut couleur = [0_u8; 3];
    for c in 0..3 {
        couleur[c] = ((ARRETS[s][c] * (256 - f) + ARRETS[s + 1][c] * f + 128) >> 8) as u8;
    }
    couleur
}

/// L'image nette (octets R, V, B, `3 × cote²`) de la neige de rayon `rayon` dont les masses sont `masses` (dans l'ordre canonique), pour la vue de rayon `rayon_vue`, le côté `cote` (pair) et `sur × sur` échantillons par pixel : **le rendu de la doc**, en `i64`.
/// N'emploie que `masses` : ni `rendu_hex`, ni `Neige`, ni `Hexagone`.
pub fn rasteriser(rayon: u32, masses: &[u32], cote: u32, rayon_vue: u32, sur: u32) -> Vec<u8> {
    let r = i64::from(rayon);
    assert_eq!(masses.len(), 3 * rayon as usize * (rayon as usize + 1) + 1, "le nombre de masses");
    let n = i64::from(cote) * i64::from(sur);
    let large = 2 * i64::from(rayon_vue) + 1;
    let m = 2 * n * B;
    let arrondi = |z: i64| (2 * z + m).div_euclid(2 * m);
    let carre = i64::from(sur) * i64::from(sur);
    let mut image = Vec::with_capacity(3 * (cote as usize).pow(2));
    for py in 0..i64::from(cote) {
        for px in 0..i64::from(cote) {
            let mut somme = [0_i64; 3];
            for j in 0..i64::from(sur) {
                let ny = (2 * (py * i64::from(sur) + j) + 1 - n) * large;
                for i in 0..i64::from(sur) {
                    let nx = (2 * (px * i64::from(sur) + i) + 1 - n) * large;
                    let (qn, rn, sn) = (-ny * B + nx * A, -2 * nx * A, ny * B + nx * A);
                    let (mut rq, mut rr, rs) = (arrondi(qn), arrondi(rn), arrondi(sn));
                    let (dq, dr, ds) = ((rq * m - qn).abs(), (rr * m - rn).abs(), (rs * m - sn).abs());
                    if dq > dr && dq > ds {
                        rq = -rr - rs;
                    } else if dr > ds {
                        rr = -rq - rs;
                    }
                    let couleur = if rq.abs().max(rr.abs()).max((rq + rr).abs()) > r {
                        [255, 255, 255]
                    } else {
                        couleur_de_t(t_de_la_masse(masses[indice_de(r, rq, rr)]))
                    };
                    for c in 0..3 {
                        somme[c] += i64::from(couleur[c]);
                    }
                }
            }
            for total in somme {
                image.push(((total + carre / 2) / carre) as u8);
            }
        }
    }
    image
}

// ---------------------------------------------------------------------------------------------------------------
// Node.
// ---------------------------------------------------------------------------------------------------------------

pub fn racine() -> PathBuf {
    e5::racine().to_path_buf()
}

/// Lance un script de `tests/etape_v8/` avec ces arguments.
pub fn lancer_le_script(script: &str, arguments: &[&str]) -> std::process::Output {
    std::process::Command::new("node")
        .arg(racine().join("tests").join("etape_v8").join(script))
        .args(arguments)
        .output()
        .unwrap_or_else(|erreur| panic!("le lancement de Node échoue : {erreur}"))
}

pub fn chemin_du_module() -> String {
    e5::chemin_module().to_str().expect("chemin en UTF-8").to_string()
}

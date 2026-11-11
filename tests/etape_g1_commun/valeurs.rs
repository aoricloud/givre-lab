//! Les valeurs de référence de l'étape G1 calculées par le code Python INDÉPENDANT (`g1_ref.py`, écrit d'après la doc de `src/neige.rs`
//! seule, SHA-256 par `hashlib`, sans lire le Rust). Identifiants `V-G1-xx` : `references/etape-g1-valeurs.md`.
//!
//! Ce fichier n'est pas un test : aucune fonction `#[test]`. Il est gelé avec les tests.
#![allow(dead_code)]

/// `(jeu, rayon, pas, réglages, empreinte, rattachées, rayon_cristal, masse_totale, masse_du_cristal)`.
pub type Trajectoire = (&'static str, u32, u64, [&'static str; 7], &'static str, usize, u32, i128, i128);

/// Les trajectoires depuis l'état de départ, calculées par g1_ref.py.
pub const TRAJECTOIRES: &[Trajectoire] = &[
    (
        "FIG9_RHO_050",
        6,
        40,
        ["0.5", "1.3", "0.08", "0.025", "0.003", "0.07", "0.00005"],
        "5d636c81316aef1200a438ba645b4d5dbd347ed4f75bd3d8a1f06dffaa900ff1",
        37,
        3,
        274877907004,
        189310351450,
    ),
    (
        "JEU_B",
        10,
        120,
        ["0.38", "1.06", "0.35", "0.112", "0.001", "0.14", "0.0006"],
        "f27aa7c2ef19bcbb8b7155e4f6f21002fc40c2f0d517007c1d0f5647ef3aa1dc",
        37,
        5,
        542883866332,
        167602223932,
    ),
    (
        "JEU_C",
        8,
        60,
        ["0.9", "1.05", "0.5", "0.5", "0.1", "0.05", "0.001"],
        "a0d97b3eb6857a5313f1347eca36449b83ef614f9c2f7d62c61a193cde2ca057",
        169,
        7,
        839236609618,
        773539696480,
    ),
    (
        "JEU_D",
        3,
        25,
        ["0.65", "1.75", "0.2", "0.026", "0.15", "0.015", "0.00001"],
        "7f29e72b56d037743b95e62138db31cf7316ff494049f17a7b0c5b6a6db298d0",
        7,
        1,
        104797202014,
        59611868656,
    ),
    (
        "FIG9_RHO_050",
        12,
        150,
        ["0.5", "1.3", "0.08", "0.025", "0.003", "0.07", "0.00005"],
        "d6e660720f031994903107db9bbb3b348b6dbe7c86136f0cf86ef2daf767900f",
        127,
        7,
        1009317314554,
        613204809172,
    ),
    (
        "JEU_C",
        5,
        80,
        ["0.9", "1.05", "0.5", "0.5", "0.1", "0.05", "0.001"],
        "63d56cdf2a73b96608c9361c382ef0fdaca3ba4f74695c19365e7a197b7a7d07",
        61,
        4,
        352187318284,
        304702658680,
    ),
    (
        "FIG13_MILIEU",
        12,
        300,
        ["0.36", "1.09", "0.01", "0.0745", "0.0001", "0.14", "0.00001"],
        "c701bc67951d40cbc6a3fe4863e261f2446be89fe139b57b85012321d811a841",
        91,
        5,
        727911057712,
        409576432276,
    ),
    (
        "FIG9_RHO_040",
        12,
        300,
        ["0.4", "1.3", "0.08", "0.025", "0.003", "0.07", "0.00005"],
        "2e7dfc3277697b08c6ec091c59e914763c114a43ebf1b0a89f186715fdf44ab7",
        127,
        6,
        808312844800,
        605871605242,
    ),
];

/// Le `sha256` des octets de l'image de référence du rendu (V-G1-RENDU-07), calculé par `g1_ref.py image` (simulation et rendu de la doc en entiers) : `R = 20`,
/// `FIG9_RHO_050`, 400 pas, `cote = 96`, `rayon_vue = 20`, `sur = 4`, `plein = 3 * 2^31`.
pub const EMPREINTE_IMAGE_REFERENCE: &str = "f535822dd6cdac22ee1cf78c62e0d02e1acc86be9fd648343426e2075230a88a";
/// Le nombre de cases rattachées de ce cristal (Python), et son rayon est 14.
pub const IMAGE_REFERENCE_RATTACHEES: usize = 439;

/// Le cristal de référence des régimes (V-G1-30) : `R = 30`, `FIG9_RHO_050`, de 100 en 100 pas jusqu'à 500 : `(pas, rattachées, rayon_cristal, masse_du_cristal)`, puis à
/// 500 pas l'empreinte et la masse totale. Calculés par `g1_ref.py` (la doc en entiers).
pub const CRISTAL_R30: [(u64, usize, u32, i128); 5] = [
    (100, 109, 6, 524108048548),
    (200, 235, 10, 1141037672896),
    (300, 397, 13, 1925210857942),
    (400, 523, 16, 2526250583398),
    (500, 709, 19, 3450866754550),
];
pub const CRISTAL_R30_EMPREINTE: &str = "4d40f3c887b95b236b9e64afc13e84c8dbe55ca10dcc1e1a38c8e6bda469e830";
pub const CRISTAL_R30_MASSE: i128 = 5995774344202;

/// Le scénario `hexagone 6` + `neige 45 0.5 1.3 0.08 0.025 0.003 0.07 0.00005` (V-G1-31) : l'empreinte de l'état final et les mesures, calculées par `g1_ref.py` (`csv_scenario`).
pub const SCENARIO_R6_EMPREINTE: &str = "f2cf72537c8d763b58c881a35abc01f1d3560d8631c699202deed30ea3c57767";
pub const SCENARIO_R6_MESURES: &str = "temps,rattachees,rayon,masse_cristal\n0,1,0,1.000000\n2,1,0,1.000000\n4,1,0,1.000000\n6,1,0,1.000000\n8,7,1,9.201066\n10,7,1,9.201066\n12,7,1,9.201066\n14,7,1,9.201066\n16,7,1,9.201066\n18,7,1,9.201066\n20,13,2,17.199796\n22,19,2,23.672251\n24,19,2,23.672251\n26,19,2,23.672251\n28,19,2,23.672251\n30,19,2,23.672251\n32,19,2,23.672251\n34,19,2,23.672251\n36,19,2,23.672251\n38,25,3,31.673391\n40,37,3,44.077251\n42,37,3,44.077251\n44,37,3,44.077251\n45,37,3,44.077251\n";
/// Le scénario `hexagone 6` + `neige 7 ...` (mêmes réglages) : les huit lignes de 0 à 7 (V-G1-31).
pub const SCENARIO_R6_PAS7_MESURES: &str = "temps,rattachees,rayon,masse_cristal\n0,1,0,1.000000\n1,1,0,1.000000\n2,1,0,1.000000\n3,1,0,1.000000\n4,1,0,1.000000\n5,1,0,1.000000\n6,1,0,1.000000\n7,1,0,1.000000\n";

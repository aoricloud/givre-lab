//! Étape V8 : **la doc et les masses suffisent à redessiner l'image nette**. Valeurs attendues : `references/etape-v8-valeurs.md`, identifiants `V-V8-xx`.
//!
//! Le rastériseur de référence (`etape_v8_commun::rasteriser`) est écrit **dans ce dossier de tests, d'après la seule doc** de `vitrine` (« De la masse à l'image nette », « La géométrie », « L'unité », « Le rapport au dégradé ») : il n'emploie que les masses lues
//! par `givre_neige_masse`, l'ordre canonique, le centre des cases en `X = −(√3/2) r`, `Y = −(q + r/2)` (`√3` par l'inverse rationnel `A/B`), la masse → `t = min(masse, 3072) / 3` → les cinq arrêts du dégradé, la moyenne des échantillons du pixel, tout en entiers. Il ne
//! lit ni `rendu_hex`, ni `Neige`, ni `Hexagone`. Les tests en comparent les octets à ceux de `givre_nette_pixel` (le rendu de V7 et G3) : **une faute de l'unité, de l'ordre, de la géométrie ou du dégradé dans la doc ou dans l'export le fait échouer**.
//!
//! L'état de la vitrine est global au processus : chaque test prend le verrou. Tant qu'une des deux fonctions des masses a son corps `todo!()`, le binaire s'arrête d'un coup : c'est un échec.

mod etape_v8_commun;

use std::sync::{Mutex, MutexGuard};

use etape_v8_commun::g3::{neige_jouee, quatre_du_jeu};
use etape_v8_commun::{CASES, PLEIN_MASSE, RAYON, RAYON_VUE, couleur_de_t, masses_de, rasteriser, t_de_la_masse};
use givre_lab::rendu_hex::{ReglagesImage, couleur_du_degrade, rendre};
use givre_lab::vitrine::{
    givre_avancer, givre_neige_demarrer, givre_neige_masse, givre_neige_masses_preparer, givre_nette_hauteur,
    givre_nette_largeur, givre_nette_pixel, givre_nette_preparer,
};

static VERROU: Mutex<()> = Mutex::new(());

fn verrou() -> MutexGuard<'static, ()> {
    VERROU.lock().unwrap_or_else(|empoisonne| empoisonne.into_inner())
}

/// Les octets R, V, B du tampon de l'image nette, lus un par un par `givre_nette_pixel`.
fn tampon(cote: u32) -> Vec<u8> {
    assert_eq!((givre_nette_largeur(), givre_nette_hauteur()), (cote, cote));
    let mut rvb = Vec::with_capacity(3 * (cote as usize).pow(2));
    for y in 0..cote {
        for x in 0..cote {
            let c = givre_nette_pixel(x, y);
            assert!(c <= 0x00FF_FFFF, "le pixel ({x}, {y}) n'est pas empaqueté en 0x00RRGGBB : {c:#x}");
            rvb.extend_from_slice(&[(c >> 16) as u8, (c >> 8) as u8, c as u8]);
        }
    }
    rvb
}

/// Démarre le jeu `jeu` avec `quatre`, joue `pas` pas, prépare les masses et rend `(masses lues par givre_neige_masse, pas joués)`.
fn masses_de_la_vitrine(jeu: usize, quatre: [u32; 4], pas: u32) -> (Vec<u32>, u32) {
    assert_eq!(givre_neige_demarrer(jeu as u32, quatre[0], quatre[1], quatre[2], quatre[3]), 0);
    let joues = givre_avancer(pas);
    assert_eq!(givre_neige_masses_preparer() as usize, CASES);
    ((0..CASES as u32).map(|i| givre_neige_masse(i)).collect(), joues)
}

/// juste : le rastériseur de référence (écrit d'après la doc, en entiers, à partir des seules masses lues par `givre_neige_masse`) redonne **octet pour octet** les pixels de `givre_nette_pixel` (V-V8-11). Huit états : le germe seul du jeu 6 (0 pas),
/// ce jeu à 60 et à 412 pas (la fougère de la figure 10), le jeu 5 à 300 pas (une dendrite), le jeu 11 à 150 pas (une plaque), le milieu de la figure 13 (jeu 18) à 200 pas, des réglages libres (le jeu 3 avec `ρ = 0,7`, `β = 2`, `κ = 0,005`, `μ = 0,03`) à 100 pas
/// et une neige rapide arrêtée **au mur** (le jeu 1 avec `ρ = 0,956862`) ; quatre images chacun : `(cote, sur)` = `(62, 2)`, `(96, 3)`, `(128, 1)` et `(2, 1)` (la plus petite : quatre échantillons). Chaque pixel de chaque image est comparé (l'image entière, pas
/// une somme). Les couleurs viennent de la masse (`t = min(masse, 3072) / 3`), la case d'un point de l'ordre canonique et de la géométrie de la doc. Une unité fausse (`>> 20`, `>> 22`, des millionièmes), une case décalée ou échangée, une géométrie fausse (`A/B`, le
/// signe de `Y`, le sens d'un bras), un `t` arrondi autrement ou un dégradé faux se voient.
///
/// Ne couvre pas : une inversion de l'ordre qui est une symétrie de l'hexagone (le cristal les a toutes : le germe, la rotation de 180°, une réflexion : mêmes nombres) ; un rendu autre que celui de `rendre` (l'image n'a pas d'autre étalon).
#[test]
#[cfg_attr(not(feature = "etape-v8"), ignore)]
fn le_rasteriseur_de_la_doc_redonne_l_image_nette() {
    let _verrou = verrou();
    let rapide = [956_862, 1_122_299, 48_396, 36_667];
    let libres = [700_000, 2_000_000, 5_000, 30_000];
    let etats: [(usize, [u32; 4], u32); 8] = [
        (6, quatre_du_jeu(6), 0),
        (6, quatre_du_jeu(6), 60),
        (6, quatre_du_jeu(6), 412),
        (5, quatre_du_jeu(5), 300),
        (11, quatre_du_jeu(11), 150),
        (18, quatre_du_jeu(18), 200),
        (3, libres, 100),
        (1, rapide, 1000),
    ];
    for (jeu, quatre, pas) in etats {
        let (masses, joues) = masses_de_la_vitrine(jeu, quatre, pas);
        assert_eq!(joues, neige_jouee(jeu, quatre, pas).1);
        for (cote, sur) in [(62_u32, 2_u32), (96, 3), (128, 1), (2, 1)] {
            assert_eq!(givre_nette_preparer(cote, sur), 0);
            let attendue = tampon(cote);
            let lue = rasteriser(RAYON, &masses, cote, RAYON_VUE, sur);
            assert!(
                lue == attendue,
                "jeu {jeu}, {pas} pas, ({cote}, {sur}) : le rastériseur de la doc ne redonne pas l'image nette"
            );
        }
    }
}

/// juste : la doc suffit pour **toute** caméra, pas seulement la vue de la vitrine (V-V8-12) : pour l'état du jeu 6 à 412 pas, le rastériseur de la doc avec une autre vue (rayon 30, 100) ou un autre côté redonne `rendu_hex::rendre` appliqué à
/// `valeurs_du_cristal` avec le `plein` 1,5 : `(80, 30, 2)`, `(50, 100, 1)` et `(64, 69, 3)`. C'est ce que la page fera (changer la taille de l'image et la vue sans repasser par le module). Et la **sensibilité** du test : l'image de référence de `(62, 2)` à vue 69
/// change si les masses sont doublées, divisées par deux, décalées d'une case, ou si la vue vaut 68 : le test ne passerait donc pas avec une unité, un ordre décalé ou une géométrie fausses.
///
/// Ne couvre pas : les vues qui dépassent `RAYON_MAX` ; la lecture du module (voir `le_rasteriseur_de_la_doc_redonne_l_image_nette`).
#[test]
#[cfg_attr(not(feature = "etape-v8"), ignore)]
fn la_doc_suffit_pour_une_autre_camera_et_le_test_est_sensible() {
    let _verrou = verrou();
    let quatre = quatre_du_jeu(6);
    let (masses, _) = masses_de_la_vitrine(6, quatre, 412);
    let (neige, _) = neige_jouee(6, quatre, 412);
    let valeurs = neige.valeurs_du_cristal();
    assert_eq!(masses, masses_de(&neige));
    for (cote, vue, sur) in [(80_u32, 30_u32, 2_u32), (50, 100, 1), (64, 69, 3)] {
        let natif = rendre(RAYON, &valeurs, &ReglagesImage { cote, rayon_vue: vue, sur, plein: 6_442_450_944 })
            .expect("un rendu");
        let doc = rasteriser(RAYON, &masses, cote, vue, sur);
        assert!(doc == natif, "(côté {cote}, vue {vue}, sur {sur}) : la doc ne redonne pas `rendre`");
    }
    // La sensibilité.
    assert_eq!(givre_nette_preparer(62, 2), 0);
    let etalon = tampon(62);
    assert!(rasteriser(RAYON, &masses, 62, RAYON_VUE, 2) == etalon);
    let doubles: Vec<u32> = masses.iter().map(|m| m * 2).collect();
    let moities: Vec<u32> = masses.iter().map(|m| m / 2).collect();
    let mut decalees = masses.clone();
    decalees.rotate_left(1);
    for (nom, autres) in [("doublées", &doubles), ("divisées par deux", &moities), ("décalées d'une case", &decalees)]
    {
        assert!(
            rasteriser(RAYON, autres, 62, RAYON_VUE, 2) != etalon,
            "les masses {nom} donnent la même image : le test est aveugle"
        );
    }
    assert!(
        rasteriser(RAYON, &masses, 62, RAYON_VUE - 1, 2) != etalon,
        "une vue de 68 donne la même image : le test est aveugle à la géométrie"
    );
}

/// juste : **la couleur de l'image nette est une fonction de la masse exportée** (V-V8-13). Pour chaque masse `a` de 0 à 3200 et quelques-unes au-delà (3 071, 3 072, 3 073, 6 300, 100 000, `2^27`), les deux extrémités de l'intervalle de `brut` que `a` représente
/// (`a << 21` et `(a << 21) + 2^21 − 1`) ont, par `rendu_hex::couleur_du_degrade(brut, 6 442 450 944)`, **la même couleur**, et c'est celle de la doc : l'interpolation des cinq arrêts pour `t = min(a, 3072) / 3`. La couleur du premier `brut` qui n'est plus dans
/// l'intervalle (`(a + 1) << 21`) est celle de `a + 1`. Une unité plus grossière (`>> 22`, des millionièmes) ferait qu'une masse représente deux couleurs ; `>> 21` est la plus grossière qui rende le dégradé exact (une plus fine, `>> 20`, le rendrait exact aussi, avec des nombres deux fois plus grands : le test d'égalité avec `valeurs_du_cristal >> 21` de `etape_v8_vitrine.rs` la refuse). La masse `0` est le
/// blanc, `3 072` et au-delà le bleu foncé `(8, 48, 107)`, et la couleur est monotone décroissante dans chaque canal.
///
/// Ne couvre pas : la bibliothèque est l'étalon du dégradé (`rendu_hex`, étape G1) ; le calcul de `t` à la main est celui de la doc.
#[test]
#[cfg_attr(not(feature = "etape-v8"), ignore)]
fn la_couleur_est_une_fonction_de_la_masse_exportee() {
    let plein = i64::from(PLEIN_MASSE) << 21;
    assert_eq!(plein, 6_442_450_944);
    let masses: Vec<u32> = (0..=3200_u32).chain([3_071, 3_072, 3_073, 6_300, 100_000, 1 << 27]).collect();
    let mut precedente = [255_u8, 255, 255];
    for &a in &masses {
        let attendue = couleur_de_t(t_de_la_masse(a));
        let bas = i64::from(a) << 21;
        let haut = bas + (1 << 21) - 1;
        assert_eq!(couleur_du_degrade(bas, plein), attendue, "masse {a} : le bas de l'intervalle de brut");
        assert_eq!(couleur_du_degrade(haut, plein), attendue, "masse {a} : le haut de l'intervalle de brut");
        assert_eq!(
            couleur_du_degrade(haut + 1, plein),
            couleur_de_t(t_de_la_masse(a + 1)),
            "masse {a} : le brut suivant est celui de {}",
            a + 1
        );
        if a <= 3200 {
            assert!((0..3).all(|c| attendue[c] <= precedente[c]), "masse {a} : le dégradé ne s'inverse jamais");
            precedente = attendue;
        }
    }
    assert_eq!(couleur_de_t(t_de_la_masse(0)), [255, 255, 255]);
    assert_eq!(couleur_de_t(t_de_la_masse(3072)), [8, 48, 107]);
    assert_eq!(couleur_de_t(t_de_la_masse(1 << 27)), [8, 48, 107]);
    assert_eq!(t_de_la_masse(3071), 1023);
    assert_eq!(t_de_la_masse(3), 1);
    assert_eq!(t_de_la_masse(2), 0);
}

//! Étape G3 : l'image nette de la neige, un calcul **exact à la main** et des empreintes de référence. Valeurs attendues : `references/etape-g3-valeurs.md`, identifiants `V-G3-14` et `V-G3-15`.
//!
//! Comme `etape_v7_rendu.rs` : un cas où l'image entière se calcule à la main (le germe seul), et des résultats figés de notre code. L'état de la vitrine est global au processus et les tests d'un fichier tournent en parallèle :
//! chaque test prend le verrou `VERROU`.
//!
//! Tant qu'une des cinq fonctions `givre_neige_*` de `src/vitrine.rs` a encore son corps `todo!()`, ce binaire de test s'arrête d'un coup : c'est un échec.

mod etape_g3_commun;

use std::sync::{Mutex, MutexGuard};

use etape_g3_commun::{ABSENT, image_nette_attendue, neige_jouee, quatre_du_jeu};
use givre_lab::empreinte::sha256_hex;
use givre_lab::vitrine::{
    givre_avancer, givre_neige_demarrer, givre_nette_hauteur, givre_nette_largeur, givre_nette_pixel,
    givre_nette_preparer,
};

static VERROU: Mutex<()> = Mutex::new(());

fn verrou() -> MutexGuard<'static, ()> {
    VERROU.lock().unwrap_or_else(|empoisonne| empoisonne.into_inner())
}

/// Les octets R, V, B du tampon de l'image nette, lus pixel par pixel.
fn lire_le_tampon() -> (u32, u32, Vec<u8>) {
    let (largeur, hauteur) = (givre_nette_largeur(), givre_nette_hauteur());
    let mut rvb = Vec::with_capacity(3 * largeur as usize * hauteur as usize);
    for y in 0..hauteur {
        for x in 0..largeur {
            let couleur = givre_nette_pixel(x, y);
            assert_ne!(couleur, ABSENT);
            rvb.extend_from_slice(&[(couleur >> 16) as u8, (couleur >> 8) as u8, couleur as u8]);
        }
    }
    (largeur, hauteur, rvb)
}

/// juste : le germe seul, à la main (V-G3-14). Au temps 0, la neige n'est que le germe : une seule case rattachée, de masse cristalline exactement 1 (le champ dessiné de la doc : `valeurs_du_cristal`, la masse `c` de la case rattachée, 0 ailleurs). Avec
/// `(cote, sur) = (556, 1)` et la vue de rayon 69 (139 cases de large : **4 pixels par case**), le point échantillonné du pixel `(px, py)` est `X = (2 px − 555) / 8`, `Y = (2 py − 555) / 8` (en unités de case) et l'hexagone de la case centrale
/// (à plat en haut, côtés à la distance 1/2 : `|Y| <= 1/2` et `|√3 X / 2| + |Y / 2| <= 1/2`) contient **exactement douze pixels** : les lignes 276 et 279 (colonnes 277 et 278) et les lignes 277 et 278 (colonnes 276 à 279) ; tous les autres pixels sont blancs (le fond,
/// la valeur 0). La couleur du germe est celle du dégradé à `t = floor(1 × 1024 / 1,5) = 682` : `s = 2`, `f = 170`, soit `((107 × 86 + 33 × 170 + 128) >> 8, (174 × 86 + 113 × 170 + 128) >> 8, (214 × 86 + 181 × 170 + 128) >> 8) = (58, 133, 192)`
/// (calculée à la main d'après les arrêts du dégradé de la doc de `rendu_hex`). L'image entière, `556 × 556` pixels, est donc **construite ici** et comparée octet pour octet à ce que la vitrine rend pour la neige de trois jeux (0, 6 et 18 : le germe est le même
/// quel que soit le jeu). Une vue autre que 69 (le nombre de pixels de l'hexagone change), un `plein` autre (la couleur change), un champ qui dessine la vapeur (le fond n'est plus blanc) ou la masse de
/// bord, ou l'image d'un pixel par case se voient. Le calcul est recoupé par `essai-g3-outils/germe_ref.py` (Python, fractions exactes, d'après la seule doc de `rendu_hex` : douze pixels, la couleur (58, 133, 192), l'image de SHA-256 `84149be2…a766`, vérifié ci-dessous).
///
/// Ne couvre pas : une neige qui a grandi (voir `etape_g3_vitrine.rs`, `l_image_nette_de_la_neige`, comparée à `rendu_hex::rendre`) ; la couleur d'une case dont la masse n'est pas 1.
#[test]
#[cfg_attr(not(feature = "etape-g3"), ignore)]
fn le_germe_seul_a_la_main() {
    let _verrou = verrou();
    let mut attendue = vec![255_u8; 3 * 556 * 556];
    let germe: [(u32, std::ops::RangeInclusive<u32>); 4] =
        [(276, 277..=278), (277, 276..=279), (278, 276..=279), (279, 277..=278)];
    for (ligne, colonnes) in germe {
        for colonne in colonnes {
            let debut = 3 * (ligne as usize * 556 + colonne as usize);
            attendue[debut..debut + 3].copy_from_slice(&[58, 133, 192]);
        }
    }
    let sombres = attendue.chunks(3).filter(|pixel| *pixel != [255, 255, 255]).count();
    assert_eq!(sombres, 12, "douze pixels dans l'hexagone du germe");
    assert_eq!(
        sha256_hex(&attendue),
        "84149be2448ab917af0a6b7bd6ad8923bdafb092d04d8c8244f187da59eaa766",
        "l'image construite ici est celle de l'outil indépendant (germe_ref.py)"
    );
    for jeu in [0_usize, 6, 18] {
        let quatre = quatre_du_jeu(jeu);
        assert_eq!(givre_neige_demarrer(jeu as u32, quatre[0], quatre[1], quatre[2], quatre[3]), 0);
        assert_eq!(givre_nette_preparer(556, 1), 0, "jeu {jeu}");
        let (largeur, hauteur, rvb) = lire_le_tampon();
        assert_eq!((largeur, hauteur), (556, 556));
        assert!(rvb == attendue, "jeu {jeu} : l'image du germe n'est pas celle du calcul à la main");
        // Le germe du module est aussi celui de la bibliothèque et de `rendre`.
        let (neige, _) = neige_jouee(jeu, quatre, 0);
        assert!(rvb == image_nette_attendue(&neige, 556, 1), "jeu {jeu} : l'image du germe n'est pas celle de rendre");
        assert_eq!(givre_nette_preparer(557, 1), 3);
        assert_eq!(givre_nette_largeur(), 0);
    }
}

/// régression : les empreintes de l'image nette de trois neiges (V-G3-15). Le jeu 3 (figure 9) à 150 pas en `(96, 3)`, le jeu 6 (figure 10) à 300 pas en `(128, 2)` et le jeu 18 (figure 13) à 200 pas en `(62, 2)` : le SHA-256 des octets R, V, B de ce que
/// `givre_nette_pixel` lit est la valeur écrite ici (un résultat **de notre code**, figé : il ne dit pas que l'image est belle, seulement qu'elle ne change pas, ni d'un système à l'autre : tout est en entiers). **À côté**, `le_germe_seul_a_la_main` et
/// `l_image_nette_de_la_neige` (etape_g3_vitrine.rs) calculent l'image sans lire de référence.
///
/// Ne couvre pas : que ces images soient justes (voir ces deux tests) ; la page.
#[test]
#[cfg_attr(not(feature = "etape-g3"), ignore)]
fn l_empreinte_des_images_de_reference() {
    let _verrou = verrou();
    let cas: [(usize, u32, u32, u32, &str); 3] = [
        (3, 150, 96, 3, "1679adb800b839f29cacd5d7697895585c32a35f7e791820a343a68be6cd3dfe"),
        (6, 300, 128, 2, "1cd608ffabd2a85687d5bbe04ad69d2c0e99c37ad364b8a341c64e2d8eaed6fe"),
        (18, 200, 62, 2, "a86f24d70408f92ff2f7062adbdd7a864a968082eb9970017fc39379eef77783"),
    ];
    for (jeu, pas, cote, sur, sha) in cas {
        let quatre = quatre_du_jeu(jeu);
        assert_eq!(givre_neige_demarrer(jeu as u32, quatre[0], quatre[1], quatre[2], quatre[3]), 0);
        assert_eq!(givre_avancer(pas), pas);
        assert_eq!(givre_nette_preparer(cote, sur), 0);
        let (_, _, rvb) = lire_le_tampon();
        assert_eq!(sha256_hex(&rvb), sha, "jeu {jeu}, {pas} pas, ({cote}, {sur})");
    }
}

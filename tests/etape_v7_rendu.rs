//! Étape V7 : ce que l'image nette des flocons montre, par des calculs **faits à la main** et une empreinte de référence. Valeurs attendues : `references/etape-v7-valeurs.md`, identifiants `V-V7-xx`.
//!
//! Le germe d'un flocon (la case centrale, gelée au temps 0) gagne `γ` à chaque pas et ne reçoit rien d'autre (ses voisines sont toutes réceptives : leur masse qui diffuse est 0) : sa masse est `1 + γ t` **exacte**,
//! donc son champ (`3 (brut − UN) + E`, `plein = 4 E`) vaut `E + 3 γ.brut t` et son `t` du dégradé `256 + 768 γ.brut t / E` : 256 au temps 0, 448 après le quart de la durée, 640 à la moitié, 1024 à la fin. Les couleurs
//! se calculent à la main sur les cinq arrêts du dégradé de `rendu_hex` ; l'outil indépendant `v7_ref.py` (un second code Python écrit d'après la doc seule : le modèle de Reiter, le champ et le rendu) les redonne.
//!
//! Étiquettes : « juste » = un calcul fait à la main (confirmé par l'outil indépendant) ou comparé au code natif ; « régression » = l'empreinte d'une image de notre code (recoupée par l'outil indépendant : les deux
//! donnent les mêmes octets, mais la preuve de la page n'en dépend pas).

mod etape_v7_commun;

use std::sync::{Mutex, MutexGuard};

use etape_v7_commun::{HORS, MODELES_NETS, UN, champ_du_modele, image_attendue, pixel_de, rgba_de};
use givre_lab::empreinte::sha256_hex;
use givre_lab::rendu_hex::couleur_du_degrade;
use givre_lab::reseau::Hexagone;
use givre_lab::vitrine::{
    givre_avancer, givre_demarrer, givre_nette_hauteur, givre_nette_largeur, givre_nette_pixel, givre_nette_preparer,
};

static VERROU: Mutex<()> = Mutex::new(());

/// Prend le verrou de la vitrine ; un test qui a paniqué en le tenant ne bloque pas les suivants.
fn verrou() -> MutexGuard<'static, ()> {
    VERROU.lock().unwrap_or_else(|empoisonne| empoisonne.into_inner())
}

/// Les octets R, V, B du tampon, lus par `givre_nette_pixel`.
fn tampon() -> (u32, Vec<u8>) {
    let (largeur, hauteur) = (givre_nette_largeur(), givre_nette_hauteur());
    assert_eq!(largeur, hauteur, "l'image est carrée");
    let mut rvb = Vec::with_capacity(3 * (largeur * hauteur) as usize);
    for y in 0..hauteur {
        for x in 0..largeur {
            let couleur = givre_nette_pixel(x, y);
            assert_ne!(couleur, HORS);
            rvb.extend_from_slice(&[(couleur >> 16) as u8, (couleur >> 8) as u8, couleur as u8]);
        }
    }
    (largeur, rvb)
}

/// Le `brut` de `γ = 0,05` (la plaque) : `round(0,05 × 2^32) = 214 748 365` (V-V7-01).
const GAMMA_PLAQUE: i64 = 214_748_365;

/// `0x00RRGGBB` d'une couleur.
fn empaquete(couleur: [u8; 3]) -> u32 {
    (u32::from(couleur[0]) << 16) | (u32::from(couleur[1]) << 8) | u32::from(couleur[2])
}

/// juste : le germe seul, à la main (V-V7-10). Aux trois modèles au temps 0, la seule case qui a de la glace est la case `(0, 0)` : sa masse est 1, donc son champ est `E` et `plein = 4 E` (le facteur `E` s'en va : la même chose pour les trois
/// modèles), soit `t = 256`, **le premier arrêt du dégradé**, `(198, 219, 239)`. Les autres cases (la vapeur `β` de 0 à 0,45, les voisines) ne sont pas gelées : blanches. Avec `côté = 62` et `sur = 2` (`N = 124` échantillons par côté), les échantillons sont aux
/// `X = (2 a + 1 − 124) (2 R + 1) / 248`, `Y` de même : les plus proches du centre sont à `± (2 R + 1) / 248` en `X` et en `Y` (les suivants, à `± 3 (2 R + 1) / 248`, sont dehors : plus de 0,74 pour `R = 30`), et la case centrale est un hexagone à plat de
/// demi-hauteur 1/2 et de sommets à `± 0,577` en `X` (`|Y| <= 1/2` et `|Y| <= √3 (0,577 − |X|)`). **Plaque (`R = 30`, 61/248 = 0,246) et étoile (`R = 40`, 81/248 = 0,327) :** chacun des quatre pixels centraux `(30, 30)`, `(31, 30)`, `(30, 31)`, `(31, 31)`
/// a **un** échantillon sur quatre dans la case (en `(± 0,246, ± 0,246)` : `√3 (0,577 − 0,246) = 0,573 >= 0,246`, dedans ; en `(± 0,327, ± 0,327)` : `0,434 >= 0,327`, dedans) : `(3 × 255 + 198 + 2) / 4 = 241`, `(3 × 255 + 219 + 2) / 4 = 246`,
/// `(3 × 255 + 239 + 2) / 4 = 251` (la moyenne entière de la doc, `(somme + 2) / 4`), et les 3 840 autres pixels sont `(255, 255, 255)`. **Dendrite (`R = 60`, 121/248 = 0,488) :** l'échantillon le plus proche est à `(± 0,488, ± 0,488)` : `√3 (0,577 − 0,488) = 0,154 < 0,488`,
/// dehors : aucun échantillon n'est dans la case et l'image entière est blanche (3 844 pixels). Un fond qui n'est pas blanc (pour `β = 0,45` et `0,4` aussi : la vapeur ne se dessine pas), un champ qui compte la vapeur, une échelle autre que `E`, un germe qui n'est pas au
/// premier arrêt, un centre décalé ou une vue d'un autre rayon que `R` se voient.
///
/// Ne couvre pas : les instants suivants (voir `le_germe_a_quatre_instants`) ; les cases autres que le germe.
#[test]
#[cfg_attr(not(feature = "etape-v7"), ignore)]
fn le_germe_seul_quatre_pixels_a_la_main() {
    let _verrou = verrou();
    for modele in &MODELES_NETS {
        assert_eq!(givre_demarrer(modele.numero, 0), 1);
        assert_eq!(givre_nette_preparer(62, 2), 0, "modèle {}", modele.numero);
        let (cote, rvb) = tampon();
        assert_eq!(cote, 62);
        let mut non_blancs = Vec::new();
        for y in 0..62_u32 {
            for x in 0..62_u32 {
                if pixel_de(&rvb, 62, x, y) != 0x00FF_FFFF {
                    non_blancs.push((x, y, pixel_de(&rvb, 62, x, y)));
                }
            }
        }
        let attendu = empaquete([241, 246, 251]);
        let pixels_du_germe: &[(u32, u32, u32)] = if modele.rayon == 60 {
            &[]
        } else {
            &[(30, 30, 0x00F1_F6FB), (31, 30, 0x00F1_F6FB), (30, 31, 0x00F1_F6FB), (31, 31, 0x00F1_F6FB)]
        };
        assert_eq!(attendu, 0x00F1_F6FB, "(241, 246, 251) empaqueté");
        assert_eq!(non_blancs, pixels_du_germe, "modèle {} : les pixels qui ne sont pas blancs", modele.numero);
    }
}

/// juste : la couleur du germe à quatre instants, à la main (V-V7-11). Le centre du pixel `(305, 305)` d'une image de `610` pixels à `sur = 1` (la vue de rayon 30 : 10 pixels par case) est à `0,05` du centre de la case `(0, 0)` : il est dedans, et
/// sa couleur est celle du germe sans mélange. Plaque (`E = 85 899 346 000`, `γ.brut = 214 748 365`) : au temps 0 le champ est `E`, `t = 256` : **(198, 219, 239)** (le premier arrêt) ; après 100 pas il est `E + 3 E / 4 = 1,75 E`, `t = 448`,
/// `s = 1`, `f = 192` : `(198 × 64 + 107 × 192 + 128) >> 8 = 130`, `(219 × 64 + 174 × 192 + 128) >> 8 = 185`, `(239 × 64 + 214 × 192 + 128) >> 8 = 220` : **(130, 185, 220)** ; après 200 pas `2,5 E`, `t = 640`, `s = 2`, `f = 128` :
/// `(107 × 128 + 33 × 128 + 128) >> 8 = 70`, `(174 × 128 + 113 × 128 + 128) >> 8 = 144`, `(214 × 128 + 181 × 128 + 128) >> 8 = 198` : **(70, 144, 198)** ; après 400 pas `4 E`, `t = 1024` : **(8, 48, 107)**, le bleu foncé du dernier arrêt. Pour
/// l'étoile (`E = 51 539 607 600`) : (198, 219, 239) au temps 0, (70, 144, 198) à 60 pas (la moitié de 120), (8, 48, 107) à 120 ; pour la dendrite (`E = 3 435 973 600`) : (8, 48, 107) à 800 (la masse du germe est `1 + 800 γ` : exactement 1 + γ.brut × 800).
/// Un champ qui n'est pas affine en `brut`, un `plein` qui n'est pas `4 E`, une échelle `E` qui n'est pas `γ.brut × durée`, un plancher de 256 oublié ou un dégradé interpolé à la main se voient.
///
/// Ne couvre pas : les cases autres que le germe (voir `la_couleur_suit_la_masse_le_long_d_un_bras`).
#[test]
#[cfg_attr(not(feature = "etape-v7"), ignore)]
fn le_germe_a_quatre_instants() {
    let _verrou = verrou();
    let cas: [(u32, u32, [u8; 3]); 8] = [
        (1, 0, [198, 219, 239]),
        (1, 100, [130, 185, 220]),
        (1, 200, [70, 144, 198]),
        (1, 400, [8, 48, 107]),
        (2, 0, [198, 219, 239]),
        (2, 60, [70, 144, 198]),
        (2, 120, [8, 48, 107]),
        (3, 800, [8, 48, 107]),
    ];
    for (modele, pas, couleur) in cas {
        assert_eq!(givre_demarrer(modele, 0), 1);
        assert_eq!(givre_avancer(pas), pas, "modèle {modele}");
        assert_eq!(givre_nette_preparer(610, 1), 0);
        assert_eq!(givre_nette_pixel(305, 305), empaquete(couleur), "modèle {modele}, {pas} pas : le germe");
        assert_eq!(givre_nette_pixel(0, 0), 0x00FF_FFFF, "le coin de l'image est blanc");
        assert_eq!(givre_nette_pixel(609, 609), 0x00FF_FFFF, "le coin de l'image est blanc");
    }
}

/// juste : la couleur suit la masse, case par case, le long d'un bras (V-V7-12). Pour la plaque après 400 pas, dans l'image de `610` pixels à `sur = 1`, le pixel `(305, 305 − 10 q)` est à `0,05` du centre de la case `(q, 0)` (le bras `(1, 0)` va
/// **vers le haut** : `Y = −q`) et `(305, 305 + 10 q)` à `0,05` du centre de la case `(−q, 0)` : pour `q` de 0 à 30 (la vue de rayon 30 : l'image va de la case −30 à la case 30), sa couleur est celle du dégradé pour **le champ de la doc de cette case**,
/// `couleur_du_degrade(champ, 4 E)` : 0 (blanc) pour une case qui n'est pas gelée (au-delà du cristal, de rayon 20, la masse est 0), `3 (brut − UN) + E` pour une case gelée. Le champ est recalculé à la main d'après les masses d'un `Flocon` construit en face, à l'indice
/// canonique de `Hexagone::index`. Le cristal a 21 cases gelées sur le bras (de la 0 à la 20), puis du blanc. Un bras dessiné vers le bas ou le côté, une échelle
/// fausse, des cases de bord (gelables, pas gelées) dessinées ou la mauvaise case se voient.
///
/// **Le champ de ce bras se calcule aussi à la main** (ajout du juge), et le test l'exige : la plaque a `β = 0` (aucune vapeur : rien ne diffuse, une case ne gagne que `γ` par pas, et seulement une fois réceptive, c'est-à-dire gelée ou voisine d'une case gelée). L'anneau 1
/// est réceptif dès le temps 0 : il atteint la masse 1 en 20 pas (`20 × 214 748 365 = 4 294 967 300 >= 2^32 = 4 294 967 296` ; à 19 pas `4 080 218 935 < 2^32`) ; l'anneau `q` devient réceptif au pas `20 (q − 1)` et gèle au pas `20 q` : à 400 pas le cristal a le rayon 20
/// (21 cases sur le bras), et la case `q` (de 1 à 20) a gagné `γ` pendant `400 − 20 (q − 1)` pas : son `brut` est `(420 − 20 q) × 214 748 365` et son champ `3 (brut − 2^32) + E` ; le germe (`2^32 + 400 γ.brut`) vaut `4 E`, la case 21 (réceptive au pas 400 seulement) 0.
///
/// Ne couvre pas : les deux autres modèles ; les directions autres que verticales (voir `etape_v7_vitrine.rs`, l'image entière).
#[test]
#[cfg_attr(not(feature = "etape-v7"), ignore)]
fn la_couleur_suit_la_masse_le_long_d_un_bras() {
    let _verrou = verrou();
    let modele = &MODELES_NETS[0];
    assert_eq!(givre_demarrer(1, 0), 1);
    givre_avancer(400);
    assert_eq!(givre_nette_preparer(610, 1), 0);
    let champ = champ_du_modele(0, 400);
    let hexagone = Hexagone::new(modele.rayon);
    let plein = 4 * modele.e;
    let mut gelees = 0;
    for q in 0..=30_i32 {
        for (signe, nom) in [(1, "vers le haut"), (-1, "vers le bas")] {
            let case = hexagone.index(signe * q, 0);
            let attendue = couleur_du_degrade(champ[case], plein);
            let ligne = (305 - signe * 10 * q) as u32;
            assert_eq!(givre_nette_pixel(305, ligne), empaquete(attendue), "case ({}, 0), {nom}", signe * q);
        }
        let valeur = champ[hexagone.index(q, 0)];
        // Le champ de la case, **à la main** (voir la doc du test) : le germe vaut `4 E` ; la case `q` de 1 à 20 a la masse `(420 − 20 q) γ`.
        let a_la_main = match q {
            0 => 4 * modele.e,
            1..=20 => 3 * (i64::from(420 - 20 * q) * GAMMA_PLAQUE - UN) + modele.e,
            _ => 0,
        };
        assert_eq!(valeur, a_la_main, "le champ de la case ({q}, 0) de la plaque à 400 pas, calculé à la main");
        if valeur > 0 {
            gelees += 1;
            assert!(valeur >= modele.e, "une case gelée vaut au moins E (la masse 1) : case ({q}, 0)");
        }
    }
    // Le cristal de la plaque à 400 pas a un rayon de 20 : 21 cases gelées sur le bras (de 0 à 20), puis du blanc.
    assert_eq!(gelees, 21, "cases gelées sur le bras (0 à 20)");
    assert_eq!(champ[hexagone.index(21, 0)], 0);
    assert_eq!(givre_nette_pixel(305, 305 - 210), 0x00FF_FFFF, "la case (21, 0) n'est pas gelée : blanche");
}

/// régression (recoupée par l'outil indépendant `v7_ref.py`) : le SHA-256 des octets R, V, B de l'image nette de référence (V-V7-13). Le modèle 2 (l'étoile) après 60 pas, `côté = 96`, `sur = 3` : `218e8fec…b427` ; et trois autres : le modèle 1
/// après 400 pas (`64`, `sur = 2`) : `87ad3c3a…f673` ; le modèle 3 après 800 pas (`64`, `sur = 2`) : `bb77ac2f…9ae4` ; le modèle 2 au temps 0 (`96`, `sur = 3`) : `030775df…0bb9`. Pour chacune, trois choses sont égales : le SHA-256 de ce que
/// `givre_nette_pixel` lit, celui de ce que `rendu_hex::rendre` donne pour le champ de la doc (`image_attendue`), et la valeur écrite ici, qui est celle du code Python (un second code, écrit d'après la doc seule : le modèle de Reiter en entiers
/// Python, le champ de la doc, le rendu de la doc). C'est une régression : une valeur de l'outil et non d'un article ; les trois étant égales, un seul code ne la fait pas plier.
///
/// Ne couvre pas : d'autres états ni d'autres tailles (voir `etape_v7_vitrine.rs`, où chaque octet est comparé à `rendre`).
#[test]
#[cfg_attr(not(feature = "etape-v7"), ignore)]
fn l_empreinte_des_images_de_reference() {
    let _verrou = verrou();
    let cas: [(u32, u32, u32, u32, &str); 4] = [
        (2, 60, 96, 3, "218e8fec69be27a8bb7e9a1179ac65909d55d8470b3aecdf2f424febf9f6b427"),
        (1, 400, 64, 2, "87ad3c3a4cbe267243acf65c6cf3b4ff3666dafd5ef0c18c170a47b95263f673"),
        (3, 800, 64, 2, "bb77ac2f0423598f3bfcb810c79da05f879861d1d18c2ca94db23a2436359ae4"),
        (2, 0, 96, 3, "030775df7e12d5d0376ab30678f565e12adb2826ca3880b5c66c6a24e3780bb9"),
    ];
    for (modele, pas, cote, sur, empreinte) in cas {
        assert_eq!(givre_demarrer(modele, 1), 1);
        givre_avancer(pas);
        assert_eq!(givre_nette_preparer(cote, sur), 0);
        let (largeur, rvb) = tampon();
        assert_eq!(largeur, cote);
        assert_eq!(
            sha256_hex(&rvb),
            empreinte,
            "modèle {modele}, {pas} pas, côté {cote}, sur {sur} : ce que la vitrine rend"
        );
        let attendue = image_attendue(modele as usize - 1, pas, cote, sur);
        assert_eq!(
            sha256_hex(&attendue),
            empreinte,
            "modèle {modele}, {pas} pas : ce que rendu_hex::rendre rend pour le champ de la doc"
        );
        assert_eq!(rgba_de(&attendue).len(), 4 * (cote * cote) as usize);
    }
}

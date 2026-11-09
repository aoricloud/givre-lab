//! Étape D1 : la directive `claquage <eta> <particules>`, `lancer` et le binaire `givre`. Valeurs attendues : `references/etape-d1-valeurs.md`,
//! identifiants `V-D1-20` à `V-D1-28`.
//!
//! Le contrat est dans la documentation de `src/claquage.rs`, section « La directive `claquage` et la sortie de `lancer` ». Les empreintes et
//! les mesures attendues viennent du code Python indépendant (`d1_ref.py scenario …`, d'après la doc seule, SHA-256 par `hashlib`) ; seuls les
//! tests dits `régression` comparent à un fichier produit par NOTRE code. Ces tests sont ignorés tant que `etape-d1` n'est pas dans
//! `default`, et ils ÉCHOUENT tant que `src/lancer.rs` ne consomme pas la directive (c'est la tâche de la vague du code).

mod common;

use givre_lab::claquage::{Claquage, DEMI_COTE_MAX, PARTICULES_MAX, parametres};
use givre_lab::dla::{COTE_APERCU, amas_colore};
use givre_lab::lancer::{ErreurLancer, Sortie, lancer};
use givre_lab::scenario::{ErreurLecture, lire_scenario};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn lancer_texte(texte: &str, graine: u64) -> Result<Sortie, ErreurLancer> {
    lancer(&lire_scenario(texte).expect("scénario valide"), graine)
}

fn racine() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// L'amas qu'un `lancer` de `n` particules doit construire, grandi sans le moindre appel à `lancer`.
fn amas_attendu(graine: u64, demi_cote: u32, eta: u32, n: usize) -> Claquage {
    let mut amas = Claquage::new(graine, demi_cote, eta);
    amas.croitre_jusqu_a(n);
    amas
}

/// Le rayon entier : le plus petit `r` tel que tous les sites vérifient `x² + y² <= r²`.
fn rayon_entier(sites: &[(i32, i32)]) -> i64 {
    let plus_grand =
        sites.iter().map(|&(x, y)| i64::from(x) * i64::from(x) + i64::from(y) * i64::from(y)).max().unwrap();
    let mut r = 0;
    while r * r < plus_grand {
        r += 1;
    }
    r
}

/// Le rayon de giration par les formules de la doc de `mesures::rayon_de_giration` : sommes entières exactes, une division, une racine.
fn giration(sites: &[(i32, i32)]) -> f64 {
    let n = sites.len() as i128;
    let (mut sx, mut sy, mut sxx, mut syy) = (0_i128, 0_i128, 0_i128, 0_i128);
    for &(x, y) in sites {
        let (x, y) = (i128::from(x), i128::from(y));
        sx += x;
        sy += y;
        sxx += x * x;
        syy += y * y;
    }
    let num = n * (sxx + syy) - (sx * sx + sy * sy);
    (num as f64 / (n * n) as f64).sqrt()
}

/// Les mesures attendues d'un amas de `n` sites, d'après la doc : `particules,rayon,rayon_giration`, une ligne par puissance de 2 de 2 à `n`, puis
/// la ligne `n` si elle n'est pas déjà là.
fn mesures_attendues(sites: &[(i32, i32)]) -> String {
    let n = sites.len();
    let mut ks: Vec<usize> = std::iter::successors(Some(2_usize), |k| Some(2 * k)).take_while(|&k| k <= n).collect();
    if ks.last() != Some(&n) {
        ks.push(n);
    }
    let mut texte = String::from("particules,rayon,rayon_giration\n");
    for k in ks {
        texte.push_str(&format!("{k},{},{:.6}\n", rayon_entier(&sites[..k]), giration(&sites[..k])));
    }
    texte
}

/// juste : `parametres` lit la directive `claquage <eta> <particules>` (V-D1-20) : pas de directive `claquage` donne `Ok(None)` ; `claquage 1 3000`
/// donne `Ok(Some((1, 3000)))` (avant ou après la forme) ; `claquage 0 1`, `claquage 8 100000` (les deux bornes hautes), `claquage 01 05` (zéros de
/// tête permis : `(1, 5)`) sont acceptés ; les autres directives (`dla 5`, `inconnue`) sont ignorées. Refusés, avec le numéro de la ligne fautive
/// (lignes ignorées et commentaires comptés) : `claquage`, `claquage 1`, `claquage 1 2 3`, `claquage 9 5` (`η` au-delà de 8), `claquage 1 0`,
/// `claquage 1 100001`, `claquage -1 5`, `claquage +1 5`, `claquage 1.0 5`, `claquage x 5`, `claquage 1 y`, `claquage 99999999999999999999 5` ; une
/// SECONDE directive `claquage` (valide ou non) est la fautive, et la première fautive l'emporte.
///
/// Ne couvre pas : l'interprétation par `lancer` (voir plus bas).
#[test]
#[cfg_attr(not(feature = "etape-d1"), ignore)]
fn parametres_lit_la_directive() {
    let lire = |texte: &str| parametres(&lire_scenario(texte).expect("scénario valide"));
    assert_eq!(lire("boite 5 5\n"), Ok(None));
    assert_eq!(lire("boite 5 5\ninconnue 1\ndla 5\nautre\n"), Ok(None));
    assert_eq!(lire("boite 5 5\nclaquage 1 3000\n"), Ok(Some((1, 3000))));
    assert_eq!(lire("claquage 2 7\nboite 5 5\n"), Ok(Some((2, 7))));
    assert_eq!(lire("boite 5 5\nclaquage 0 1\n"), Ok(Some((0, 1))));
    assert_eq!(lire("boite 5 5\nclaquage 8 100000\n"), Ok(Some((8, 100_000))));
    assert_eq!(lire("boite 5 5\nclaquage 01 05\n"), Ok(Some((1, 5))));
    assert_eq!(PARTICULES_MAX, 100_000);
    assert_eq!(lire("# un commentaire\nboite 5 5\n\ninconnue\nclaquage 3 12\n"), Ok(Some((3, 12))));
    for mauvais in [
        "claquage",
        "claquage 1",
        "claquage 1 2 3",
        "claquage 9 5",
        "claquage 1 0",
        "claquage 1 100001",
        "claquage -1 5",
        "claquage +1 5",
        "claquage 1.0 5",
        "claquage x 5",
        "claquage 1 y",
        "claquage 99999999999999999999 5",
    ] {
        assert_eq!(lire(&format!("# c\nboite 5 5\n{mauvais}\n")), Err(ErreurLecture::LigneInvalide(3)), "{mauvais}");
    }
    assert_eq!(
        lire("boite 5 5\nclaquage 1 3\nclaquage 0 4\n"),
        Err(ErreurLecture::LigneInvalide(3)),
        "seconde directive valide"
    );
    assert_eq!(
        lire("boite 5 5\nclaquage 1 3\n\nclaquage 9 4\n"),
        Err(ErreurLecture::LigneInvalide(4)),
        "seconde directive fautive"
    );
    assert_eq!(
        lire("boite 5 5\nclaquage 9 3\nclaquage 1 4\n"),
        Err(ErreurLecture::LigneInvalide(2)),
        "la première fautive l'emporte"
    );
}

/// juste : trois petits scénarios dont les sorties sont celles du code Python indépendant (V-D1-21). Chacun donne l'empreinte (SHA-256 de l'écriture
/// canonique, `demi_cote = (min(largeur, hauteur) - 1) / 2`), le CSV `particules,rayon,rayon_giration` et un aperçu qui est celui de `amas_colore`
/// du même amas. `boite 21 21` + `claquage 1 12`, graine 3 : empreinte `8511b9cf…f88b`, lignes `2,1,0.500000`, `4,1,0.829156`, `8,3,1.672386`,
/// `12,5,2.409472` ; `boite 21 21` + `claquage 2 20`, graine 4 : `20c44f18…8673`, lignes `2,1,0.500000`, `4,2,0.935414`, `8,3,1.293010`, `16,5,2.217649`,
/// `20,6,2.456624` ; `boite 21 21` + `claquage 0 33`, graine 5 : `2c81f3df…23fc`, lignes `2,1,0.500000`, `4,2,0.707107`, `8,4,1.391941`, `16,5,2.228193`, `32,7,2.947953`, `33,7,3.061056` ; `boite 3 3` + `claquage 0 2`,
/// graine 2 : `60ce2ca7…0070` (le deuxième site touche la frontière : il est bien ajouté) ; `boite 1 1` + `claquage 0 1` : le germe seul, `1,0,0.000000`,
/// aperçu blanc de 512 × 512.
///
/// Ne couvre pas : de grands amas (voir `exemples_du_depot`).
#[test]
#[cfg_attr(not(feature = "etape-d1"), ignore)]
fn lancer_a_la_main() {
    let cas: [(&str, u64, u32, u32, usize, &str, &str); 5] = [
        (
            "boite 21 21\nclaquage 1 12\n",
            3,
            10,
            1,
            12,
            "8511b9cffeb0184d6ae4b9c0fcd92a8ee37da81fa40f88f81500ddd6e588f88b",
            "particules,rayon,rayon_giration\n2,1,0.500000\n4,1,0.829156\n8,3,1.672386\n12,5,2.409472\n",
        ),
        (
            "boite 21 21\nclaquage 2 20\n",
            4,
            10,
            2,
            20,
            "20c44f18f36b88b02eb6c15c710992b88434cd8a7d6567bddd1b470ff6428673",
            "particules,rayon,rayon_giration\n2,1,0.500000\n4,2,0.935414\n8,3,1.293010\n16,5,2.217649\n20,6,2.456624\n",
        ),
        (
            "boite 21 21\nclaquage 0 33\n",
            5,
            10,
            0,
            33,
            "2c81f3df7d69e418e5fdb53666c3d7f5a1b2a61b9c063e14ccf8b9bf110823fc",
            "particules,rayon,rayon_giration\n2,1,0.500000\n4,2,0.707107\n8,4,1.391941\n16,5,2.228193\n32,7,2.947953\n33,7,3.061056\n",
        ),
        (
            "boite 3 3\nclaquage 0 2\n",
            2,
            1,
            0,
            2,
            "60ce2ca762b457f8b349e4003409ec58b5f1a783ad5e61de533d557d351b0070",
            "particules,rayon,rayon_giration\n2,1,0.500000\n",
        ),
        (
            "boite 1 1\nclaquage 0 1\n",
            7,
            0,
            0,
            1,
            "3dd1715619219906b0417d25cc7d35ebfbf999758757960482e6f3b507105cb6",
            "particules,rayon,rayon_giration\n1,0,0.000000\n",
        ),
    ];
    for (texte, graine, demi_cote, eta, n, empreinte, mesures) in cas {
        let sortie = lancer_texte(texte, graine).expect("lancer");
        assert_eq!(sortie.empreinte, empreinte, "{texte:?} : empreinte");
        let amas = amas_attendu(graine, demi_cote, eta, n);
        assert_eq!(amas.empreinte(), empreinte, "{texte:?} : l'amas de la bibliothèque a la même empreinte");
        assert_eq!(sortie.mesures, mesures, "{texte:?} : mesures");
        let (l, h, rvb) = amas_colore(amas.sites(), COTE_APERCU);
        let png = common::relire_png(&sortie.apercu).expect("PNG valide");
        assert_eq!((png.largeur, png.hauteur), (l, h), "{texte:?} : dimensions de l'aperçu");
        assert_eq!(png.rvb, rvb, "{texte:?} : pixels de l'aperçu");
        if n == 1 {
            assert_eq!((png.largeur, png.hauteur), (512, 512));
            assert!(png.rvb.chunks(3).all(|p| p == [255, 255, 255]), "le germe seul est blanc");
        }
    }
}

/// juste : les trois scénarios d'exemple du dépôt, `scenarios/claquage-0.txt` (Eden : `boite 61 61`, `claquage 0 600`), `claquage-1.txt` (`boite 61 61`,
/// `claquage 1 150`) et `claquage-2.txt` (`boite 81 81`, `claquage 2 120`) (V-D1-22) : chaque fichier se lit comme cela (une `boite` et UNE directive
/// `claquage`, les commentaires comptent pour rien), et `lancer` avec la graine 1 rend l'empreinte et le CSV du code Python indépendant : `939571b6…d903`
/// (rayon final 18, giration `10.257649` à 600 sites), `6c912e3e…8c11` (rayon 13, giration `7.283137`) et `f4f572d5…bd68` (rayon 19, giration `9.082427`) ; l'aperçu
/// est celui de `amas_colore` du même amas.
///
/// Ne couvre pas : les autres graines ; que les images soient belles (la planche `apercus/etape-d1-planche.png` est pour l'œil).
#[test]
#[cfg_attr(not(feature = "etape-d1"), ignore)]
fn exemples_du_depot() {
    let cas: [(&str, u32, u32, u32, usize, &str, &str); 3] = [
        (
            "claquage-0.txt",
            61,
            30,
            0,
            600,
            "939571b62e06d0f7d11d328926f56026cb3b9b25e35e5f4e45a84d35b3fad903",
            "particules,rayon,rayon_giration\n2,1,0.500000\n4,2,0.866025\n8,2,1.192424\n16,4,1.836054\n32,5,2.512275\n64,7,3.673038\n128,10,4.850569\n256,14,6.878486\n512,17,9.527341\n600,18,10.257649\n",
        ),
        (
            "claquage-1.txt",
            61,
            30,
            1,
            150,
            "6c912e3efba4b16bf4d5e102307c9bede4e5cb499301e4105e698a8f9fc38c11",
            "particules,rayon,rayon_giration\n2,1,0.500000\n4,2,0.935414\n8,3,1.639360\n16,4,2.460627\n32,6,3.491060\n64,8,4.501383\n128,13,6.697746\n150,13,7.283137\n",
        ),
        (
            "claquage-2.txt",
            81,
            40,
            2,
            120,
            "f4f572d5f6cc8ca72734e1eea6725e574a7aa2c2d1fd62bf8947b8ee9a42bd68",
            "particules,rayon,rayon_giration\n2,1,0.500000\n4,2,0.935414\n8,3,1.639360\n16,6,2.743601\n32,7,3.814933\n64,12,5.941754\n120,19,9.082427\n",
        ),
    ];
    for (fichier, cote, demi_cote, eta, n, empreinte, mesures) in cas {
        let texte = fs::read_to_string(racine().join("scenarios").join(fichier))
            .unwrap_or_else(|_| panic!("scenarios/{fichier}"));
        let scenario = lire_scenario(&texte).expect("scénario valide");
        assert_eq!(
            scenario.forme,
            givre_lab::scenario::Forme::Boite { largeur: cote, hauteur: cote },
            "{fichier} : la boîte"
        );
        assert_eq!(scenario.directives.len(), 1, "{fichier} : une seule directive");
        assert_eq!(scenario.directives[0].nom, "claquage");
        assert_eq!(scenario.directives[0].args, [eta.to_string(), n.to_string()], "{fichier} : eta et particules");
        let sortie = lancer_texte(&texte, 1).expect("lancer");
        assert_eq!(sortie.empreinte, empreinte, "{fichier} : empreinte");
        assert_eq!(sortie.mesures, mesures, "{fichier} : mesures");
        let amas = amas_attendu(1, demi_cote, eta, n);
        assert_eq!(amas.empreinte(), empreinte);
        let (l, h, rvb) = amas_colore(amas.sites(), COTE_APERCU);
        let png = common::relire_png(&sortie.apercu).expect("PNG valide");
        assert_eq!((png.largeur, png.hauteur), (l, h), "{fichier} : dimensions de l'aperçu");
        assert_eq!(png.rvb, rvb, "{fichier} : pixels de l'aperçu");
    }
}

/// juste : pour plusieurs `η` (0, 1, 2), plusieurs `n` (1, 2, 3, 5, 8, 9, 16, 17, 40) et deux graines, les mesures sont celles que la doc décrit, reconstruites
/// ici depuis les sites d'un `Claquage` (V-D1-23) : l'en-tête, une ligne par puissance de 2 `<= n` à partir de 2, puis la ligne `n` si elle n'y est pas
/// déjà (jamais deux lignes identiques : `claquage 1 8` finit par `8`, une seule fois ; `claquage 1 1` n'a que la ligne `1,0,0.000000`), le rayon étant celui
/// des `k` premiers sites et la giration celle de ces `k` sites avec six décimales ; l'empreinte est celle de `Claquage::empreinte()` d'un amas de même graine,
/// de même `η` et de `demi_cote = (min(largeur, hauteur) - 1) / 2` (ici `boite 61 71` : 30 ; la même empreinte pour `boite 71 61`) ; l'aperçu est le PNG de
/// `amas_colore(sites, COTE_APERCU)` ; chaque ligne finit par un seul `\n`, sans `\r`. Une directive `claquage 0 …` ne tire aucun potentiel : mêmes sites
/// que `Claquage::new(graine, 30, 0)`.
///
/// Ne couvre pas : les amas qui touchent la frontière (voir `refus_et_arret`).
#[test]
#[cfg_attr(not(feature = "etape-d1"), ignore)]
fn mesures_empreinte_et_apercu_suivent_la_doc() {
    for eta in 0..=2_u32 {
        for graine in [1_u64, 2] {
            for n in [1_usize, 2, 3, 5, 8, 9, 16, 17, 40] {
                let amas = amas_attendu(graine, 30, eta, n);
                let sortie = lancer_texte(&format!("boite 61 71\nclaquage {eta} {n}\n"), graine).expect("lancer");
                assert_eq!(
                    sortie.mesures,
                    mesures_attendues(amas.sites()),
                    "eta {eta}, graine {graine}, {n} particules"
                );
                assert!(sortie.mesures.ends_with('\n') && !sortie.mesures.contains('\r'));
                assert_eq!(
                    sortie.empreinte,
                    amas.empreinte(),
                    "eta {eta}, graine {graine}, {n} particules : empreinte"
                );
                let (l, h, rvb) = amas_colore(amas.sites(), COTE_APERCU);
                let png = common::relire_png(&sortie.apercu).expect("PNG valide");
                assert_eq!((png.largeur, png.hauteur), (l, h), "dimensions de l'aperçu, {n} particules");
                assert_eq!(png.rvb, rvb, "pixels de l'aperçu, {n} particules");
                let autre = lancer_texte(&format!("boite 71 61\nclaquage {eta} {n}\n"), graine).expect("lancer");
                assert_eq!(autre.empreinte, sortie.empreinte, "le demi_cote ne dépend que du plus petit côté");
            }
        }
    }
    assert_eq!(
        lancer_texte("boite 5 5\nclaquage 1 1\n", 3).unwrap().mesures,
        "particules,rayon,rayon_giration\n1,0,0.000000\n"
    );
}

/// juste : mêmes entrées, mêmes sorties, octet pour octet (le PNG compris) ; deux graines donnent deux empreintes, deux CSV et deux aperçus différents ;
/// `claquage 1 40` et `claquage 1 41` donnent deux empreintes différentes ; deux exposants donnent deux empreintes (l'écriture canonique contient `η`) ;
/// une directive `claquage` rend une empreinte qui n'est pas celle du scénario vide de la même boîte, ni celle d'une `dla` de même taille (V-D1-24).
///
/// Ne couvre pas : l'égalité d'empreintes entre Linux et Windows (la CI la joue : tout est en entiers et en `+ - * /`).
#[test]
#[cfg_attr(not(feature = "etape-d1"), ignore)]
fn determinisme_et_graines() {
    let a = lancer_texte("boite 41 41\nclaquage 1 40\n", 5).unwrap();
    let b = lancer_texte("boite 41 41\nclaquage 1 40\n", 5).unwrap();
    assert_eq!(a, b);
    let c = lancer_texte("boite 41 41\nclaquage 1 40\n", 6).unwrap();
    assert_ne!(a.empreinte, c.empreinte);
    assert_ne!(a.mesures, c.mesures);
    assert_ne!(a.apercu, c.apercu);
    let d = lancer_texte("boite 41 41\nclaquage 1 41\n", 5).unwrap();
    assert_ne!(a.empreinte, d.empreinte);
    let autre_eta = lancer_texte("boite 41 41\nclaquage 2 40\n", 5).unwrap();
    assert_ne!(a.empreinte, autre_eta.empreinte);
    let vide = lancer_texte("boite 41 41\n", 5).unwrap();
    assert_ne!(a.empreinte, vide.empreinte);
    assert!(vide.mesures.starts_with("temps,cases"), "sans directive, les mesures sont celles de l'étape 0");
    let dla = lancer_texte("boite 41 41\ndla 40\n", 5).unwrap();
    assert_ne!(a.empreinte, dla.empreinte);
}

/// juste : les refus (V-D1-25), tous `ErreurLancer::DirectiveInconnue(ligne)` avec la PLUS PETITE ligne refusée (lignes ignorées comptées) : une première
/// directive inconnue (`inconnue` ligne 2 devant un `claquage` ligne 3 : 2) ; une autre directive après un `claquage` valide (`inconnue 1`, `surface depot 8 3`,
/// `reiter 1 0.5 0.01 10`, `cinetique 1 0 5` : 3) ; une SECONDE `claquage` (3) ; un `claquage` fautif (`claquage 9 5`, `claquage 1 0`, `claquage`, `claquage 1
/// 100001` : sa ligne) ; un `claquage` sur un `hexagone` (la ligne du premier, même après une ligne ignorée ; si une inconnue le précède, c'est elle) ; une
/// boîte au-delà de `DEMI_COTE_MAX` (`boite 513 513` : `demi_cote` 256 ; `boite 600 600`) ; la DLA l'emporte quand une directive `dla`, `collage` ou `bruit` est dans le
/// scénario, où qu'elle soit, et refuse le `claquage` à sa ligne (`claquage` ligne 2 devant `dla 5` : 2 ; `dla 5` ligne 2 devant `claquage` ligne 3 : 3 ; avec
/// `collage 0.5` : 2) ; et un amas qui touche la frontière avant d'avoir `particules` sites : la ligne de la directive `claquage`. Ce qui est permis :
/// `boite 1 1` + `claquage 0 1` (le germe seul), `boite 3 3` + `claquage 0 2` (le deuxième site, qui touche la frontière, est ajouté) mais pas `claquage 0 3`
/// (ligne 2), `boite 511 511` + `claquage 0 5` et `boite 512 512` + `claquage 0 5` (`demi_cote` 255, la borne, bord compris).
///
/// Ne couvre pas : le message du binaire (voir `binaire_*`).
#[test]
#[cfg_attr(not(feature = "etape-d1"), ignore)]
fn refus_et_arret() {
    assert_eq!(DEMI_COTE_MAX, 255);
    for (texte, ligne) in [
        ("boite 9 9\ninconnue\nclaquage 1 5\n", 2),
        ("boite 9 9\nclaquage 1 5\ninconnue 1\n", 3),
        ("boite 9 9\nclaquage 1 5\nsurface depot 8 3\n", 3),
        ("boite 9 9\nclaquage 1 5\nreiter 1 0.5 0.01 10\n", 3),
        ("boite 9 9\nclaquage 1 5\ncinetique 1 0 5\n", 3),
        ("boite 9 9\nclaquage 0 5\nclaquage 1 5\n", 3),
        ("boite 9 9\nclaquage 9 5\n", 2),
        ("boite 9 9\nclaquage 1 0\n", 2),
        ("boite 9 9\nclaquage\n", 2),
        ("boite 9 9\n# c\nclaquage 1 100001\n", 3),
        ("boite 9 9\nclaquage 9 5\nclaquage 1 5\n", 2),
        ("hexagone 5\nclaquage 1 4\n", 2),
        ("# c\nhexagone 5\n\nclaquage 1 4\nclaquage 2 5\n", 4),
        ("hexagone 5\ninconnue\nclaquage 1 4\n", 2),
        ("hexagone 5\nclaquage 1 4\ninconnue\n", 2),
        ("boite 513 513\nclaquage 0 5\n", 2),
        ("boite 600 600\nclaquage 0 5\n", 2),
        ("boite 513 600\n# c\nclaquage 0 5\n", 3),
        ("boite 9 9\nclaquage 1 5\ndla 5\n", 2),
        ("boite 9 9\ndla 5\nclaquage 1 5\n", 3),
        ("boite 9 9\nclaquage 1 5\ncollage 0.5\ndla 5\n", 2),
        ("boite 5 5\n\nclaquage 1 100\n", 3),
        ("boite 5 5\n# c\n\n\nclaquage 0 26\n", 5),
        ("boite 3 3\nclaquage 0 3\n", 2),
        ("boite 1 1\nclaquage 0 2\n", 2),
    ] {
        assert_eq!(lancer_texte(texte, 1).map(|_| ()), Err(ErreurLancer::DirectiveInconnue(ligne)), "{texte:?}");
    }
    let germe = lancer_texte("boite 1 1\nclaquage 0 1\n", 4).expect("le germe seul tient dans une boîte de 1 × 1");
    assert_eq!(germe.mesures, "particules,rayon,rayon_giration\n1,0,0.000000\n");
    let png = common::relire_png(&germe.apercu).expect("PNG valide");
    assert_eq!((png.largeur, png.hauteur), (512, 512));
    assert!(png.rvb.chunks(3).all(|p| p == [255, 255, 255]));
    assert!(lancer_texte("boite 3 3\nclaquage 0 2\n", 9).is_ok());
    assert!(lancer_texte("boite 511 511\nclaquage 0 5\n", 9).is_ok(), "demi_cote 255, la borne, est permis");
    assert!(lancer_texte("boite 512 512\nclaquage 0 5\n", 9).is_ok(), "demi_cote (512 - 1) / 2 = 255 est permis");
}

// ------------------------------------------------------------------------------------------------------------- le binaire

fn givre(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_givre")).args(args).output().expect("lancement de givre")
}

fn s(chemin: &Path) -> &str {
    chemin.to_str().expect("chemin en UTF-8")
}

/// juste : `givre lancer scenarios/claquage-1.txt --graine 1 --sortie <dossier>` réussit (code 0) et écrit exactement `empreinte.txt` (l'empreinte de
/// `exemples_du_depot` puis `\n`), `mesures.csv` et `apercu.png`, et rien d'autre ; ces fichiers sont ceux de `lancer` octet pour octet. Le binaire n'a pas
/// changé (V-D1-26).
///
/// Ne couvre pas : les options du binaire (étape 0).
#[test]
#[cfg_attr(not(feature = "etape-d1"), ignore)]
fn binaire_claquage() {
    let dossier = common::dossier_temporaire("claquage-binaire");
    let scenario = racine().join("scenarios").join("claquage-1.txt");
    let resultat = givre(&["lancer", s(&scenario), "--graine", "1", "--sortie", s(&dossier)]);
    assert_eq!(resultat.status.code(), Some(0), "stderr : {}", String::from_utf8_lossy(&resultat.stderr));
    let mut noms: Vec<String> =
        fs::read_dir(&dossier).unwrap().map(|e| e.unwrap().file_name().into_string().unwrap()).collect();
    noms.sort();
    assert_eq!(noms, ["apercu.png", "empreinte.txt", "mesures.csv"]);
    let attendu = lancer_texte(&fs::read_to_string(&scenario).unwrap(), 1).unwrap();
    assert_eq!(
        fs::read_to_string(dossier.join("empreinte.txt")).unwrap(),
        "6c912e3efba4b16bf4d5e102307c9bede4e5cb499301e4105e698a8f9fc38c11\n"
    );
    assert_eq!(fs::read_to_string(dossier.join("empreinte.txt")).unwrap(), format!("{}\n", attendu.empreinte));
    assert_eq!(fs::read_to_string(dossier.join("mesures.csv")).unwrap(), attendu.mesures);
    assert_eq!(fs::read(dossier.join("apercu.png")).unwrap(), attendu.apercu);
    let _ = fs::remove_dir_all(&dossier);
}

/// juste : un scénario dont la directive `claquage` est refusée (`claquage 9 5`) ou dont l'amas touche la frontière (`boite 5 5` + `claquage 1 100`) fait
/// échouer le binaire : code de sortie 1, un message qui contient `ligne 2` et pas `panicked`, et RIEN d'écrit (le dossier de sortie n'est pas créé) ; un scénario
/// voisin qui doit être ACCEPTÉ (`boite 21 21` + `claquage 1 15`) réussit (code 0, trois fichiers) : un test qui ne vérifie que des refus ne distinguerait pas
/// le code de l'étape D1 de celui d'avant (V-D1-27).
///
/// Ne couvre pas : les autres erreurs d'arguments (étape 0).
#[test]
#[cfg_attr(not(feature = "etape-d1"), ignore)]
fn binaire_refus_et_acceptation() {
    for (nom, texte, code) in [
        ("refus-eta", "boite 9 9\nclaquage 9 5\n", 1),
        ("refus-frontiere", "boite 5 5\nclaquage 1 100\n", 1),
        ("accepte", "boite 21 21\nclaquage 1 15\n", 0),
    ] {
        let espace = common::dossier_temporaire(nom);
        fs::create_dir_all(&espace).unwrap();
        let scenario = espace.join("scenario.txt");
        fs::write(&scenario, texte).unwrap();
        let sortie = espace.join("sortie");
        let resultat = givre(&["lancer", s(&scenario), "--graine", "1", "--sortie", s(&sortie)]);
        assert_eq!(resultat.status.code(), Some(code), "{nom} : {}", String::from_utf8_lossy(&resultat.stderr));
        let message = String::from_utf8_lossy(&resultat.stderr);
        assert!(!message.contains("panicked"), "{nom} : {message}");
        if code == 1 {
            assert!(message.contains("ligne 2"), "{nom} : {message}");
            assert!(!sortie.exists(), "{nom} : rien ne doit être écrit");
        } else {
            let mut noms: Vec<String> =
                fs::read_dir(&sortie).unwrap().map(|e| e.unwrap().file_name().into_string().unwrap()).collect();
            noms.sort();
            assert_eq!(noms, ["apercu.png", "empreinte.txt", "mesures.csv"]);
        }
        let _ = fs::remove_dir_all(&espace);
    }
}

/// régression : `references/claquage-0-graine-1.empreinte`, `claquage-1-graine-1.empreinte` et `claquage-2-graine-1.empreinte` contiennent l'empreinte que NOTRE
/// code donne pour `scenarios/claquage-0.txt`, `-1.txt` et `-2.txt` avec la graine 1 (64 chiffres hexadécimaux puis `\n`). Elles sont figées après que
/// `exemples_du_depot` a vérifié les mêmes valeurs contre le code Python indépendant : ce test ne prouve que la stabilité (V-D1-28).
///
/// Ne couvre pas : que la valeur soit juste (voir `exemples_du_depot`).
#[test]
#[cfg_attr(not(feature = "etape-d1"), ignore)]
fn regression_references_claquage_graine_1() {
    for eta in 0..=2 {
        let reference =
            fs::read_to_string(racine().join("references").join(format!("claquage-{eta}-graine-1.empreinte")))
                .unwrap_or_else(|_| panic!("references/claquage-{eta}-graine-1.empreinte"));
        let texte = fs::read_to_string(racine().join("scenarios").join(format!("claquage-{eta}.txt"))).unwrap();
        assert_eq!(reference, format!("{}\n", lancer_texte(&texte, 1).unwrap().empreinte), "eta {eta}");
    }
}

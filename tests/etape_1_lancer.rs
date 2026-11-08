//! Étape 1 : `lancer` et le binaire `givre` sur un scénario `dla`. Valeurs attendues : `references/etape-1-valeurs.md`, identifiants
//! `V-LDLA-xx`.
//!
//! Le contrat est dans la documentation de `src/dla.rs`, section « La directive `dla` et la sortie de `lancer` ». Les empreintes, les
//! mesures et les aperçus attendus des petits cas viennent du code Python indépendant (`ref_dla.py`, d'après la doc seule, SHA-256 par
//! `hashlib`) ; seul `regression_reference_dla_graine_1` compare à un fichier produit par NOTRE code.

mod common;

use givre_lab::dla::{Amas, COTE_APERCU, amas_colore};
use givre_lab::empreinte::sha256_hex;
use givre_lab::lancer::{ErreurLancer, Sortie, lancer};
use givre_lab::scenario::lire_scenario;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn lancer_texte(texte: &str, graine: u64) -> Result<Sortie, ErreurLancer> {
    lancer(&lire_scenario(texte).expect("scénario valide"), graine)
}

fn racine() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// L'amas qu'un `lancer` de `n` particules doit construire, grandi en entiers sans le moindre test de `lancer`.
fn amas_attendu(graine: u64, demi_cote: u32, n: usize) -> Amas {
    let mut amas = Amas::new(graine, demi_cote);
    amas.croitre_jusqu_a(n);
    amas
}

/// Le plus petit entier `r` tel que tous les sites vérifient `x² + y² <= r²`.
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

/// Les mesures attendues d'un amas de `n` sites, d'après la doc de `lancer` : `particules,rayon,rayon_giration`, une ligne par puissance
/// de 2 de 2 à `n`, puis la ligne `n` si elle n'est pas déjà là.
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

/// juste : sur `boite 41 41` + `dla 40`, graine 3 (V-LDLA-01) : l'empreinte, les mesures et l'aperçu sont ceux du code Python indépendant : le SHA-256 de
/// l'écriture canonique (`demi_cote` 20) `ae05f58a…43ee` ; le CSV `particules,rayon,rayon_giration` avec les lignes `2,1,0.500000`,
/// `4,3,1.118034`, `8,5,1.727534`, `16,5,2.492959`, `32,8,3.737218`, `40,9,4.295856` (les puissances de 2 jusqu'à 32, puis 40) ; un PNG de 510 × 510
/// pixels (la fenêtre est `[-8, 8]²`, côté 17, échelle `512 / 17 = 30`) dont les octets RVB ont le SHA-256 `b66c262c…3ff5`.
///
/// Ne couvre pas : de grands amas (voir `exemple_dla_graine_1`).
#[test]
#[cfg_attr(not(feature = "etape-1"), ignore)]
fn lancer_dla_a_la_main() {
    let sortie = lancer_texte("boite 41 41\ndla 40\n", 3).expect("lancer");
    assert_eq!(sortie.empreinte, "ae05f58a2dd9cd0183bd7e3d2bc6db329dd62e5ba4e9135799cc10d39fc343ee");
    assert_eq!(
        sortie.mesures,
        "particules,rayon,rayon_giration\n2,1,0.500000\n4,3,1.118034\n8,5,1.727534\n16,5,2.492959\n32,8,3.737218\n40,9,4.295856\n"
    );
    let png = common::relire_png(&sortie.apercu).expect("PNG valide");
    assert_eq!((png.largeur, png.hauteur), (510, 510));
    assert_eq!(sha256_hex(&png.rvb), "b66c262cdc61b6c26505acf485f5a4b3022292227c6700625da8645ec4dc3ff5");
}

/// juste : le scénario d'exemple `scenarios/dla.txt` (`boite 401 401`, `dla 3000`), graine 1 (V-LDLA-02) : l'empreinte est
/// `c1d90557…0ee6`, calculée par le code Python indépendant (le même amas de 3000 sites, l'écriture canonique de `demi_cote` 200) ; le CSV est
/// celui de Python (12 lignes : 2, 4, …, 2048, puis 3000 : `3000,110,50.238930`) ; l'aperçu est un PNG de 434 × 434 (côté 217, échelle 2) dont les
/// octets RVB ont le SHA-256 `2fc30fdb…5711`. Le scénario est lu depuis le fichier du dépôt.
///
/// Ne couvre pas : les autres graines (leurs mesures sont aux tests statistiques).
#[test]
#[cfg_attr(not(feature = "etape-1"), ignore)]
fn exemple_dla_graine_1() {
    let texte = fs::read_to_string(racine().join("scenarios").join("dla.txt")).expect("scenarios/dla.txt");
    let sortie = lancer_texte(&texte, 1).expect("lancer");
    assert_eq!(sortie.empreinte, "c1d905579070373fc8f718e8f75fb8ff1f7824e89331f292517900ceb3e00ee6");
    let attendu = "particules,rayon,rayon_giration\n2,1,0.500000\n4,2,0.935414\n8,3,1.414214\n16,5,2.260393\n32,6,3.139964\n\
                   64,11,4.762730\n128,17,7.423647\n256,21,10.591807\n512,34,16.155341\n1024,55,25.166440\n2048,82,39.235352\n\
                   3000,110,50.238930\n";
    assert_eq!(sortie.mesures, attendu);
    let png = common::relire_png(&sortie.apercu).expect("PNG valide");
    assert_eq!((png.largeur, png.hauteur), (434, 434));
    assert_eq!(sha256_hex(&png.rvb), "2fc30fdb575706e63a9b2a3281e4ab412fe6d208159ff004ffb5929833fc5711");
}

/// juste : pour plusieurs `n` (1, 2, 3, 4, 5, 7, 8, 9, 16, 17, 64, 100) et trois graines, les mesures sont celles que la doc décrit,
/// reconstruites ici depuis les sites d'un `Amas` (V-LDLA-03) : l'en-tête, une ligne par puissance de 2 `<= n` à partir de 2, puis la ligne `n` si
/// elle n'y est pas déjà (jamais deux lignes identiques : `dla 8` finit par `8`, une seule fois ; `dla 1` n'a que la ligne `1,0,0.000000`),
/// le rayon étant celui des `k` premiers sites et la giration celle de ces `k` sites avec six décimales ; l'empreinte est celle de
/// `Amas::empreinte()` d'un amas de même graine et de `demi_cote = (min(largeur, hauteur) - 1) / 2` (ici `boite 61 41` : 20 ; la même
/// empreinte pour `boite 41 99`) ; l'aperçu est le PNG de `amas_colore(sites, COTE_APERCU)` ; chaque ligne finit par un seul `\n`, sans `\r`.
///
/// Ne couvre pas : les amas qui débordent la boîte (voir `refus_et_debordement`).
#[test]
#[cfg_attr(not(feature = "etape-1"), ignore)]
fn mesures_empreinte_et_apercu_suivent_la_doc() {
    for graine in [1_u64, 2, 3] {
        for n in [1_usize, 2, 3, 4, 5, 7, 8, 9, 16, 17, 64, 100] {
            let amas = amas_attendu(graine, 20, n);
            let sortie = lancer_texte(&format!("boite 61 41\ndla {n}\n"), graine).expect("lancer");
            assert_eq!(sortie.mesures, mesures_attendues(amas.sites()), "graine {graine}, {n} particules");
            assert!(sortie.mesures.ends_with('\n') && !sortie.mesures.contains('\r'));
            assert_eq!(sortie.empreinte, amas.empreinte(), "graine {graine}, {n} particules : empreinte");
            let (l, h, rvb) = amas_colore(amas.sites(), COTE_APERCU);
            let png = common::relire_png(&sortie.apercu).expect("PNG valide");
            assert_eq!((png.largeur, png.hauteur), (l, h), "dimensions de l'aperçu, {n} particules");
            assert_eq!(png.rvb, rvb, "pixels de l'aperçu, {n} particules");
            let autre = lancer_texte(&format!("boite 41 99\ndla {n}\n"), graine).expect("lancer");
            assert_eq!(autre.empreinte, sortie.empreinte, "le demi_cote ne dépend que du plus petit côté");
        }
    }
    assert_eq!(
        lancer_texte("boite 5 5\ndla 1\n", 3).unwrap().mesures,
        "particules,rayon,rayon_giration\n1,0,0.000000\n"
    );
    assert_eq!(COTE_APERCU, 512);
}

/// juste : mêmes entrées, mêmes sorties, octet pour octet (le PNG compris) ; deux graines donnent deux empreintes, deux CSV et deux aperçus
/// différents ; `dla 40` et `dla 41` donnent deux empreintes différentes ; une directive `dla` rend une empreinte qui n'est pas celle du
/// scénario vide de la même boîte (V-LDLA-04).
///
/// Ne couvre pas : l'égalité d'empreintes entre Linux et Windows (la CI la joue : tout est en entiers).
#[test]
#[cfg_attr(not(feature = "etape-1"), ignore)]
fn determinisme_et_graines() {
    let a = lancer_texte("boite 41 41\ndla 60\n", 5).unwrap();
    let b = lancer_texte("boite 41 41\ndla 60\n", 5).unwrap();
    assert_eq!(a, b);
    let c = lancer_texte("boite 41 41\ndla 60\n", 6).unwrap();
    assert_ne!(a.empreinte, c.empreinte);
    assert_ne!(a.mesures, c.mesures);
    assert_ne!(a.apercu, c.apercu);
    let d = lancer_texte("boite 41 41\ndla 61\n", 5).unwrap();
    assert_ne!(a.empreinte, d.empreinte);
    let vide = lancer_texte("boite 41 41\n", 5).unwrap();
    assert_ne!(a.empreinte, vide.empreinte);
    assert!(vide.mesures.starts_with("temps,cases"), "sans `dla`, les mesures sont celles de l'étape 0");
}

/// juste : les refus (V-LDLA-05), tous `ErreurLancer::DirectiveInconnue(ligne)` avec la PLUS PETITE ligne refusée (lignes ignorées comptées) :
/// une directive inconnue (`boite 9 9`, `inconnue`, `dla 5` : ligne 2) ; une directive `dla` fautive devant une inconnue (`dla 0`, ligne 2, avant
/// `inconnue` ligne 3) ; une `dla` valide devant une inconnue (ligne 3) ; une seconde `dla` (ligne 3) ; une `dla` sur un `hexagone`
/// (la ligne de la première `dla`, même après une ligne ignorée ; si une inconnue la précède, c'est elle) ; un amas qui sort de la boîte
/// (`boite 5 5` + `dla 100` : le réseau n'a que 25 cases, donc c'est sûr pour toute graine) : la ligne de la directive `dla`. Et ce qui est
/// permis : `boite 1 1` + `dla 1` (le germe seul, 512 × 512 blanc) ; `boite 5 5` + `dla 3` (trois sites tiennent toujours dans `demi_cote` 2).
///
/// Ne couvre pas : le message du binaire (voir `etape_1_lancer::binaire_*`).
#[test]
#[cfg_attr(not(feature = "etape-1"), ignore)]
fn refus_et_debordement() {
    for (texte, ligne) in [
        ("boite 9 9\ninconnue\ndla 5\n", 2),
        ("boite 9 9\ndla 0\ninconnue\n", 2),
        ("boite 9 9\ndla 5\ninconnue 1\n", 3),
        ("boite 9 9\ndla 3\ndla 4\n", 3),
        ("boite 9 9\ndla\n", 2),
        ("boite 9 9\n# c\ndla 1000001\n", 3),
        ("hexagone 5\ndla 4\n", 2),
        ("# c\nhexagone 5\n\ndla 4\ndla 5\n", 4),
        ("hexagone 5\ninconnue\ndla 4\n", 2),
        ("boite 5 5\n\ndla 100\n", 3),
        ("boite 5 5\n# c\n\n\ndla 26\n", 5),
    ] {
        assert_eq!(lancer_texte(texte, 1).map(|_| ()), Err(ErreurLancer::DirectiveInconnue(ligne)), "{texte:?}");
    }
    let germe = lancer_texte("boite 1 1\ndla 1\n", 4).expect("le germe seul tient dans une boîte de 1 × 1");
    assert_eq!(germe.mesures, "particules,rayon,rayon_giration\n1,0,0.000000\n");
    let png = common::relire_png(&germe.apercu).expect("PNG valide");
    assert_eq!((png.largeur, png.hauteur), (512, 512));
    assert!(png.rvb.chunks(3).all(|p| p == [255, 255, 255]));
    assert_eq!(lancer_texte("boite 1 1\ndla 2\n", 4).map(|_| ()), Err(ErreurLancer::DirectiveInconnue(2)));
    assert!(lancer_texte("boite 5 5\ndla 3\n", 9).is_ok());
}

// ------------------------------------------------------------------------------------------------------------- le binaire

fn givre(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_givre")).args(args).output().expect("lancement de givre")
}

fn s(chemin: &Path) -> &str {
    chemin.to_str().expect("chemin en UTF-8")
}

/// juste : `givre lancer scenarios/dla.txt --graine 1 --sortie <dossier>` réussit (code 0) et écrit exactement `empreinte.txt` (l'empreinte
/// de `exemple_dla_graine_1` puis `\n`), `mesures.csv` et `apercu.png` (434 × 434), et rien d'autre ; ces fichiers sont ceux de `lancer` octet pour
/// octet. Le binaire n'a pas changé (V-LDLA-06).
///
/// Ne couvre pas : les options du binaire (étape 0).
#[test]
#[cfg_attr(not(feature = "etape-1"), ignore)]
fn binaire_dla() {
    let dossier = common::dossier_temporaire("dla-binaire");
    let scenario = racine().join("scenarios").join("dla.txt");
    let resultat = givre(&["lancer", s(&scenario), "--graine", "1", "--sortie", s(&dossier)]);
    assert_eq!(resultat.status.code(), Some(0), "stderr : {}", String::from_utf8_lossy(&resultat.stderr));
    let mut noms: Vec<String> =
        fs::read_dir(&dossier).unwrap().map(|e| e.unwrap().file_name().into_string().unwrap()).collect();
    noms.sort();
    assert_eq!(noms, ["apercu.png", "empreinte.txt", "mesures.csv"]);
    let attendu = lancer_texte(&fs::read_to_string(&scenario).unwrap(), 1).unwrap();
    assert_eq!(fs::read_to_string(dossier.join("empreinte.txt")).unwrap(), format!("{}\n", attendu.empreinte));
    assert_eq!(
        fs::read_to_string(dossier.join("empreinte.txt")).unwrap(),
        "c1d905579070373fc8f718e8f75fb8ff1f7824e89331f292517900ceb3e00ee6\n"
    );
    assert_eq!(fs::read_to_string(dossier.join("mesures.csv")).unwrap(), attendu.mesures);
    assert_eq!(fs::read(dossier.join("apercu.png")).unwrap(), attendu.apercu);
    let _ = fs::remove_dir_all(&dossier);
}

/// juste : un scénario dont la directive `dla` est refusée (`dla 0`) fait échouer le binaire : code de sortie 1, un message d'une ligne au
/// moins qui contient `ligne 2` et pas `panicked`, et RIEN d'écrit (le dossier de sortie n'est pas créé) ; de même un amas qui déborde la boîte
/// (`boite 5 5` + `dla 100` : `ligne 2`) ; et un scénario voisin qui doit être ACCEPTÉ (`boite 21 21` + `dla 20`) réussit (code 0, trois fichiers) :
/// le binaire de l'étape 0 refusait toute directive, un test qui ne vérifie que des refus ne distinguerait pas le code de l'étape 1 de celui de
/// l'étape 0 (V-LDLA-07).
///
/// Ne couvre pas : les autres erreurs d'arguments (étape 0).
#[test]
#[cfg_attr(not(feature = "etape-1"), ignore)]
fn binaire_refus_et_acceptation() {
    for (nom, texte, code) in [
        (
            "refus-zero",
            "boite 9 9
dla 0
",
            1,
        ),
        (
            "refus-debordement",
            "boite 5 5
dla 100
",
            1,
        ),
        (
            "accepte",
            "boite 21 21
dla 20
",
            0,
        ),
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

/// régression : `references/dla-graine-1.empreinte` contient l'empreinte que NOTRE code donne pour `scenarios/dla.txt` avec la graine 1
/// (64 chiffres hexadécimaux puis `\n`). Elle a été figée après que `exemple_dla_graine_1` a vérifié la même valeur contre le code Python
/// indépendant : ce test ne prouve que la stabilité (V-LDLA-08).
///
/// Ne couvre pas : que la valeur soit juste (voir `exemple_dla_graine_1`).
#[test]
#[cfg_attr(not(feature = "etape-1"), ignore)]
fn regression_reference_dla_graine_1() {
    let reference = fs::read_to_string(racine().join("references").join("dla-graine-1.empreinte"))
        .expect("references/dla-graine-1.empreinte");
    let texte = fs::read_to_string(racine().join("scenarios").join("dla.txt")).unwrap();
    assert_eq!(reference, format!("{}\n", lancer_texte(&texte, 1).unwrap().empreinte));
}

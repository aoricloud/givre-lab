//! Étape 4 : la directive `cinetique` de `lancer`, le scénario d'exemple et le binaire `givre`. Valeurs attendues :
//! `references/etape-4-valeurs.md`, identifiants `V-DIR-xx`.
//!
//! Les valeurs écrites viennent du simulateur naïf Python de la fiche (`simulateur_naif.py`, `valeurs_lancer.py`) : l'empreinte
//! finale, le SHA-256 du CSV et celui des octets RVB de l'aperçu de quatre scénarios. Seul le dernier test,
//! `regression_reference_cinetique_graine_1`, compare à un fichier produit par notre propre code.

mod common;

use givre_lab::cinetique::{Face, simuler};
use givre_lab::empreinte::sha256_hex;
use givre_lab::image::ecrire_png;
use givre_lab::lancer::{ErreurLancer, Sortie, lancer};
use givre_lab::scenario::lire_scenario;
use std::fs;
use std::process::Command;

fn lancer_texte(texte: &str, graine: u64) -> Result<Sortie, ErreurLancer> {
    lancer(&lire_scenario(texte).expect("scénario valide"), graine)
}

fn nombre_de_lignes(texte: &str) -> usize {
    texte.lines().count()
}

/// juste : valeurs de Python, d'après la doc de `cinetique` et de `lancer` : `lancer` sur un scénario `boite L L` et `cinetique J Δμ B`
/// rend l'**empreinte finale**, le **CSV** et l'**aperçu** de la face menée de `B` balayages (V-DIR-01). Quatre scénarios :
/// `boite 8 8 / cinetique 16 -6 100`, graine 11 (empreinte `204bbc9d…81ef`, 52 lignes de CSV de SHA-256 `698e19b9…9881`, octets RVB de
/// l'aperçu de SHA-256 `0d0949da…44b6`) ; `boite 16 16 / cinetique 12 8 120`, graine 4 (`b7c50c8d…bc62`, 62 lignes `5645c453…d5fe`,
/// aperçu `e44c0052…3716`) ; `boite 5 5 / cinetique 12 4 20`, graine 3 (`891ed75a…6abc`, 22 lignes `9711424d…2aa9`, aperçu `2b263dd5…004a`) ;
/// `boite 3 3 / cinetique 20 0 7`, graine 5, une face qui reste plane (`7571cf3d…611d`, 9 lignes `481cd12a…29e4`, aperçu gris 128,
/// `b5939d4c…3b36`). L'aperçu est un PNG que le décodeur du test relit, de `L × L` pixels. L'empreinte est celle de l'écriture canonique de
/// `Face` (`GIVRE004`…), non celle du scénario vide.
///
/// Ne couvre pas : les mesures elles-mêmes (`etape_4_mesures`), les erreurs (`directives_refusees`).
#[test]
#[cfg_attr(not(feature = "etape-4"), ignore)]
#[allow(clippy::type_complexity)]
fn lancer_donne_les_valeurs_de_python() {
    // (L, J, Δμ, balayages, graine, empreinte, lignes du CSV, SHA-256 du CSV, SHA-256 des octets RVB)
    let cas: [(u32, u32, i32, usize, u64, &str, usize, &str, &str); 4] = [
        (
            8,
            16,
            -6,
            100,
            11,
            "204bbc9dc66e287c1f0e8282d3420d5867e83cb69f57951632c509ad502981ef",
            52,
            "698e19b932373b14d593c23a6513e6474ce939e5f37f1ca75b359ae314969881",
            "0d0949dac0db56a49fb0b37ebdf5ce47fadec21f5e070ea644de6e59a12344b6",
        ),
        (
            16,
            12,
            8,
            120,
            4,
            "b7c50c8da8f39ccf5e62948031392b6b051495261d216c70bf6c33e83258bc62",
            62,
            "5645c4530c9addd3e0d0879e0e7f7f643564f40d32aef93fb0da9a161310d5fe",
            "e44c00523bccdb9135d8aedfe60eb33712178c2d1f3689947b69da30131a3716",
        ),
        (
            5,
            12,
            4,
            20,
            3,
            "891ed75a31aa5a3eb4da29823b6c6acf1e32b8b17720649bae7fc57313816abc",
            22,
            "9711424d6c9be8395dc2022142dbf79559b4437c3d7a46c1709c850cbea32aa9",
            "2b263dd5b6a76c37dec9ff5894b5aced79a7803cb388c914e2c9337c4d98004a",
        ),
        (
            3,
            20,
            0,
            7,
            5,
            "7571cf3ddab0440c7e384e78bac0f28213a7eb384e8a2c66e71717171eb3611d",
            9,
            "481cd12af55764cfe5aad22c8f789aedd1590d381813078e23178aac384629e4",
            "b5939d4cd02dc0ec2c7b162ad69c7157a59a807f5ffc4a6b0533ddbbe88d3b36",
        ),
    ];
    for (l, j, mu, balayages, graine, empreinte, lignes, sha_csv, sha_rvb) in cas {
        let texte = format!("boite {l} {l}\ncinetique {j} {mu} {balayages}\n");
        let sortie = lancer_texte(&texte, graine).expect("lancer");
        assert_eq!(sortie.empreinte, empreinte, "{texte:?} graine {graine} : empreinte");
        assert_eq!(nombre_de_lignes(&sortie.mesures), lignes, "{texte:?} : lignes du CSV");
        assert_eq!(sha256_hex(sortie.mesures.as_bytes()), sha_csv, "{texte:?} : CSV");
        assert!(sortie.mesures.starts_with("balayage,hauteur_moyenne,rugosite\n0,0.000000,0.000000\n"));
        let png = common::relire_png(&sortie.apercu).expect("PNG valide");
        assert_eq!((png.largeur, png.hauteur), (l, l));
        assert_eq!(common::hex(&givre_lab::empreinte::sha256(&png.rvb)), sha_rvb, "{texte:?} : octets RVB de l'aperçu");
    }
}

/// juste : la composition est écrite dans la doc de `lancer` : `lancer` fait `Face::new(L, J, Δμ, graine)`, puis `simuler(&mut face, B)` : sa
/// sortie est **exactement** `(face.empreinte(), le CSV de simuler, ecrire_png(L, L, &face.apercu()))` (V-DIR-02), pour deux
/// scénarios, dont un où les directives sont dans l'autre ordre (`cinetique` avant `boite`), séparées de lignes vides et de
/// commentaires, avec des zéros de tête (`016 -004 0120`, qui valent `16 -4 120`) et `-0` pour `Δμ` (qui vaut 0).
///
/// Ne couvre pas : les valeurs elles-mêmes (le test précédent).
#[test]
#[cfg_attr(not(feature = "etape-4"), ignore)]
fn lancer_compose_face_simuler_png() {
    for (texte, l, j, mu, balayages, graine) in [
        ("boite 16 16\ncinetique 12 8 120\n", 16_usize, 12_u32, 8_i32, 120_usize, 4_u64),
        ("# une face\n\ncinetique 016 -004 0120\n\n  boite 12 12\n", 12, 16, -4, 120, 9),
        ("boite 9 9\ncinetique 8 -0 33", 9, 8, 0, 33, 2),
    ] {
        let sortie = lancer_texte(texte, graine).expect("lancer");
        let mut face = Face::new(l, j, mu, graine);
        let mesures = simuler(&mut face, balayages);
        assert_eq!(sortie.mesures, mesures, "{texte:?}");
        assert_eq!(sortie.empreinte, face.empreinte(), "{texte:?}");
        assert_eq!(sortie.apercu, ecrire_png(l as u32, l as u32, &face.apercu()), "{texte:?}");
    }
}

/// juste : propriétés exactes : **même scénario et même graine, même sortie** ; vingt graines donnent vingt empreintes et vingt
/// CSV distincts ; deux paramètres différents (`J`, `Δμ`, durée) donnent des empreintes différentes ; l'empreinte a 64 chiffres
/// hexadécimaux minuscules (V-DIR-03).
///
/// Ne couvre pas : l'égalité entre Linux et Windows (la CI la rejoue contre `references/cinetique-graine-1.empreinte`).
#[test]
#[cfg_attr(not(feature = "etape-4"), ignore)]
fn determinisme_et_graines_distinctes() {
    let texte = "boite 8 8\ncinetique 16 12 60\n";
    let a = lancer_texte(texte, 5).unwrap();
    assert_eq!(a, lancer_texte(texte, 5).unwrap());
    let (mut empreintes, mut csv) = (std::collections::BTreeSet::new(), std::collections::BTreeSet::new());
    for graine in 0..20_u64 {
        let sortie = lancer_texte(texte, graine).unwrap();
        assert_eq!(sortie.empreinte.len(), 64);
        assert!(sortie.empreinte.bytes().all(|c| c.is_ascii_digit() || (b'a'..=b'f').contains(&c)));
        assert!(empreintes.insert(sortie.empreinte), "deux graines ont la même empreinte (graine {graine})");
        assert!(csv.insert(sortie.mesures), "deux graines ont le même CSV (graine {graine})");
    }
    let autres = [
        "boite 8 8\ncinetique 17 12 60\n",
        "boite 8 8\ncinetique 16 13 60\n",
        "boite 8 8\ncinetique 16 12 61\n",
        "boite 9 9\ncinetique 16 12 60\n",
    ];
    for autre in autres {
        assert_ne!(lancer_texte(autre, 5).unwrap().empreinte, a.empreinte, "{autre:?}");
    }
}

/// juste : la doc de `lancer` dit « refusée par `DirectiveInconnue(ligne)` » : **les directives refusées** (V-DIR-04). La ligne
/// rendue est celle de la première directive refusée, dans l'ordre du fichier, lignes ignorées comptées. Refusés : un nombre
/// d'arguments faux (2 ou 4) ; `J` négatif, `J` ou `Δμ` écrits `+4`, `4.0`, `0x4`, `1e2`, `-` seul, `--4` ; des entiers hors bornes (`J = 4294967296`,
/// `Δμ = 2147483648` et `-2147483649`, 0 balayage, `1000001` balayages) ; une deuxième directive `cinetique` ; une autre directive,
/// avant ou après ; une forme `hexagone` ; une boîte non carrée ; une boîte de côté 2 ; une directive mal formée qui précède
/// une autre. Acceptés : `J = 0`, `J = 4294967295`, `Δμ = 2147483647` et `-2147483648`, 1 balayage, `1 000 000` balayages sur une boîte de 3 × 3
/// (51 lignes de CSV), `Δμ = -0`, les zéros de tête.
///
/// Ne couvre pas : une boîte de 4096 × 4096 (la mémoire), les messages du binaire (test suivant).
#[test]
#[cfg_attr(not(feature = "etape-4"), ignore)]
fn directives_refusees() {
    let refus = |texte: &str, ligne: usize| {
        assert_eq!(lancer_texte(texte, 1).map(|_| ()), Err(ErreurLancer::DirectiveInconnue(ligne)), "{texte:?}");
    };
    refus("boite 8 8\ncinetique 16 4\n", 2);
    refus("boite 8 8\ncinetique 16 4 100 5\n", 2);
    refus("boite 8 8\ncinetique\n", 2);
    refus("boite 8 8\ncinetique -16 4 100\n", 2);
    for mauvais in ["+4", "4.0", "0x4", "1e2", "-", "--4", "٤"] {
        refus(&format!("boite 8 8\ncinetique 16 {mauvais} 100\n"), 2);
        refus(&format!("boite 8 8\ncinetique {mauvais} 4 100\n"), 2);
        refus(&format!("boite 8 8\ncinetique 16 4 {mauvais}\n"), 2);
    }
    refus("boite 8 8\ncinetique 4294967296 4 100\n", 2);
    refus("boite 8 8\ncinetique 16 2147483648 100\n", 2);
    refus("boite 8 8\ncinetique 16 -2147483649 100\n", 2);
    refus("boite 8 8\ncinetique 16 4 0\n", 2);
    refus("boite 8 8\ncinetique 16 4 1000001\n", 2);
    refus("boite 8 8\ncinetique 16 4 100\ncinetique 16 4 100\n", 3);
    refus("boite 8 8\ncinetique 16 4 100\nautre 1\n", 3);
    refus("autre 1\nboite 8 8\ncinetique 16 4 100\n", 1);
    refus("boite 8 8\n\n# c\ncinetique 16 x 100\n", 4);
    refus("cinetique 16 4\ncinetique 16 4 100\nboite 8 8\n", 1);
    refus("hexagone 4\ncinetique 16 4 100\n", 2);
    refus("boite 8 6\ncinetique 16 4 100\n", 2);
    refus("boite 6 8\n\ncinetique 16 4 100\n", 3);
    refus("boite 2 2\ncinetique 16 4 100\n", 2);
    refus("boite 1 1\ncinetique 16 4 100\n", 2);

    for texte in [
        "boite 4 4\ncinetique 0 4 5\n",
        "boite 4 4\ncinetique 4294967295 4 5\n",
        "boite 4 4\ncinetique 16 2147483647 5\n",
        "boite 4 4\ncinetique 16 -2147483648 5\n",
        "boite 4 4\ncinetique 16 4 1\n",
        "boite 3 3\ncinetique 16 4 5\n",
    ] {
        assert!(lancer_texte(texte, 1).is_ok(), "{texte:?} devait être accepté");
    }
    assert_eq!(lancer_texte("boite 4 4\ncinetique 16 -0 5\n", 1), lancer_texte("boite 4 4\ncinetique 16 0 5\n", 1));
    assert_eq!(lancer_texte("boite 4 4\ncinetique 016 004 005\n", 1), lancer_texte("boite 4 4\ncinetique 16 4 5\n", 1));
    let longue = lancer_texte("boite 3 3\ncinetique 16 0 1000000\n", 1).unwrap();
    assert_eq!(nombre_de_lignes(&longue.mesures), 52);
}

/// juste : la doc de `lancer` ; l'étape 0 reste vraie : sans directive `cinetique`, `lancer` se comporte comme à l'étape 0 : la sortie
/// d'un scénario `boite 64 48` est celle du scénario vide (CSV `temps,cases\n0,3072\n`, empreinte `5a21e2ae…f613` pour la graine 1) et
/// `boite 4 4` sans directive n'est pas une face cinétique (V-DIR-05).
///
/// Ne couvre pas : les autres étapes.
#[test]
#[cfg_attr(not(feature = "etape-4"), ignore)]
fn sans_directive_cinetique_rien_ne_change() {
    let sortie = lancer_texte("boite 64 48\n", 1).unwrap();
    assert_eq!(sortie.mesures, "temps,cases\n0,3072\n");
    assert_eq!(sortie.empreinte, "5a21e2aef0d732996015dcca8b79365294ccd49900bec7a833e498f92455f613");
}

/// juste : le contenu du fichier est écrit dans la doc de l'étape (une boîte 32 × 32 et `cinetique 16 24 400`) : le scénario d'exemple
/// `scenarios/cinetique.txt` se lit, a pour forme `boite 32 32` et pour unique autre directive `cinetique 16 24 400`, et `lancer` le
/// mène à 400 balayages : le CSV a 51 lignes de données (de 0 à 400 par pas de 8), la hauteur moyenne passe de 0 à plus de 50 et la
/// rugosité de 0 à plus de 0,5 (la face lisse devient rugueuse sous `Δμ/kT = 1,5` à `J/kT = 1` : la croissance cinétique) (V-DIR-06).
///
/// Ne couvre pas : la valeur exacte de ces mesures (l'empreinte de référence est `regression_reference_cinetique_graine_1`).
#[test]
#[cfg_attr(not(feature = "etape-4"), ignore)]
fn scenario_d_exemple() {
    let racine = env!("CARGO_MANIFEST_DIR");
    let texte = fs::read_to_string(format!("{racine}/scenarios/cinetique.txt")).expect("scenarios/cinetique.txt");
    let scenario = lire_scenario(&texte).expect("scénario valide");
    assert_eq!(scenario.forme, givre_lab::scenario::Forme::Boite { largeur: 32, hauteur: 32 });
    assert_eq!(scenario.directives.len(), 1);
    assert_eq!(scenario.directives[0].nom, "cinetique");
    assert_eq!(scenario.directives[0].args, ["16", "24", "400"]);
    let sortie = lancer(&scenario, 1).unwrap();
    let lignes: Vec<&str> = sortie.mesures.lines().collect();
    assert_eq!(lignes.len(), 1 + 51);
    let derniere: Vec<&str> = lignes[51].split(',').collect();
    assert_eq!(derniere[0], "400");
    let (hauteur, rugosite): (f64, f64) = (derniere[1].parse().unwrap(), derniere[2].parse().unwrap());
    assert!(hauteur > 50.0 && rugosite > 0.5, "la face n'a pas grandi : <h> = {hauteur}, w² = {rugosite}");
}

/// juste : le binaire est une coque autour de `lancer` : `givre lancer scenarios/cinetique.txt --graine 1 --sortie <dossier>` réussit
/// (code 0) et écrit exactement `empreinte.txt` (l'empreinte de `lancer`, puis `\n`), `mesures.csv` (le CSV de `lancer`) et `apercu.png`
/// (le PNG de `lancer`, relu : 32 × 32), et aucun autre fichier ; un scénario non carré avec `cinetique` est refusé : code 1, message avec
/// `ligne 2`, rien d'écrit (V-DIR-07).
///
/// Ne couvre pas : les options du binaire (étape 0).
#[test]
#[cfg_attr(not(feature = "etape-4"), ignore)]
fn binaire_ecrit_les_sorties_de_lancer() {
    let racine = env!("CARGO_MANIFEST_DIR");
    let dossier = common::dossier_temporaire("cinetique-binaire");
    fs::create_dir_all(&dossier).unwrap();
    let sortie = dossier.join("sortie");
    let scenario = format!("{racine}/scenarios/cinetique.txt");
    let resultat = Command::new(env!("CARGO_BIN_EXE_givre"))
        .args(["lancer", &scenario, "--graine", "1", "--sortie", sortie.to_str().unwrap()])
        .output()
        .expect("lancement de givre");
    assert_eq!(resultat.status.code(), Some(0), "stderr : {}", String::from_utf8_lossy(&resultat.stderr));
    let attendu = lancer(&lire_scenario(&fs::read_to_string(&scenario).unwrap()).unwrap(), 1).unwrap();
    let mut noms: Vec<String> =
        fs::read_dir(&sortie).unwrap().map(|e| e.unwrap().file_name().into_string().unwrap()).collect();
    noms.sort();
    assert_eq!(noms, ["apercu.png", "empreinte.txt", "mesures.csv"]);
    assert_eq!(fs::read_to_string(sortie.join("empreinte.txt")).unwrap(), format!("{}\n", attendu.empreinte));
    assert_eq!(fs::read_to_string(sortie.join("mesures.csv")).unwrap(), attendu.mesures);
    assert_eq!(fs::read(sortie.join("apercu.png")).unwrap(), attendu.apercu);
    let png = common::relire_png(&attendu.apercu).expect("PNG valide");
    assert_eq!((png.largeur, png.hauteur), (32, 32));

    // un scénario dont les valeurs viennent de Python : `boite 8 8`, `cinetique 16 -6 100`, graine 11 (V-DIR-01)
    let connu = dossier.join("connu.txt");
    fs::write(&connu, "boite 8 8\ncinetique 16 -6 100\n").unwrap();
    let sortie = dossier.join("sortie-connue");
    let resultat = Command::new(env!("CARGO_BIN_EXE_givre"))
        .args(["lancer", connu.to_str().unwrap(), "--graine", "11", "--sortie", sortie.to_str().unwrap()])
        .output()
        .expect("lancement de givre");
    assert_eq!(resultat.status.code(), Some(0), "stderr : {}", String::from_utf8_lossy(&resultat.stderr));
    assert_eq!(
        fs::read_to_string(sortie.join("empreinte.txt")).unwrap(),
        "204bbc9dc66e287c1f0e8282d3420d5867e83cb69f57951632c509ad502981ef\n"
    );
    let csv = fs::read(sortie.join("mesures.csv")).unwrap();
    assert_eq!(sha256_hex(&csv), "698e19b932373b14d593c23a6513e6474ce939e5f37f1ca75b359ae314969881");
    let png = common::relire_png(&fs::read(sortie.join("apercu.png")).unwrap()).expect("PNG valide");
    assert_eq!((png.largeur, png.hauteur), (8, 8));
    assert_eq!(
        common::hex(&givre_lab::empreinte::sha256(&png.rvb)),
        "0d0949dac0db56a49fb0b37ebdf5ce47fadec21f5e070ea644de6e59a12344b6"
    );

    // un scénario refusé : code 1, la ligne dans le message, rien d'écrit
    let mauvais = dossier.join("mauvais.txt");
    fs::write(&mauvais, "boite 8 6\ncinetique 16 4 100\n").unwrap();
    let refus = dossier.join("refus");
    let resultat = Command::new(env!("CARGO_BIN_EXE_givre"))
        .args(["lancer", mauvais.to_str().unwrap(), "--graine", "1", "--sortie", refus.to_str().unwrap()])
        .output()
        .expect("lancement de givre");
    assert_eq!(resultat.status.code(), Some(1));
    let message = String::from_utf8_lossy(&resultat.stderr);
    assert!(message.contains("ligne 2") && !message.contains("panicked"), "message : {message}");
    assert!(!refus.exists(), "le dossier de sortie a été créé pour un scénario refusé");
    let _ = fs::remove_dir_all(&dossier);
}

/// régression : l'empreinte du scénario `scenarios/cinetique.txt` avec la graine 1 est celle du fichier
/// `references/cinetique-graine-1.empreinte` (l'empreinte puis un saut de ligne), produite par le binaire sur ce scénario ; c'est ce que la
/// CI rejoue sous Linux et sous Windows. Ce fichier n'existe pas dans le jeu de tests : c'est la vague du code qui le produit (`givre
/// lancer scenarios/cinetique.txt --graine 1 --sortie <dossier>`, puis copie de `empreinte.txt`) (V-DIR-08).
///
/// Ne couvre pas : que l'empreinte soit juste (`lancer_donne_les_valeurs_de_python` le prouve sur d'autres scénarios, par le même code).
/// Ce test dit seulement que rien n'a bougé.
#[test]
#[cfg_attr(not(feature = "etape-4"), ignore)]
fn regression_reference_cinetique_graine_1() {
    let racine = env!("CARGO_MANIFEST_DIR");
    let reference = fs::read_to_string(format!("{racine}/references/cinetique-graine-1.empreinte"))
        .expect("references/cinetique-graine-1.empreinte est absente : la vague du code la produit avec le binaire");
    let texte = fs::read_to_string(format!("{racine}/scenarios/cinetique.txt")).expect("scenarios/cinetique.txt");
    let sortie = lancer(&lire_scenario(&texte).unwrap(), 1).unwrap();
    assert_eq!(reference, format!("{}\n", sortie.empreinte));
}

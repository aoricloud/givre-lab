//! Étape 0 : `lancer`, l'empreinte, les mesures et l'aperçu du scénario vide. Valeurs attendues :
//! `references/etape-0-valeurs.md`, identifiants `V-LANC-xx`.
//!
//! L'empreinte est vérifiée d'abord **à la main** : l'écriture canonique de petits cas (33 octets pour une boîte, 29 pour un
//! hexagone, écrits un à un d'après la doc de `lancer`) et son SHA-256 calculé par Python `hashlib`, un outil indépendant ;
//! seul le test final, `regression_reference_vide_graine_1`, compare à un fichier de référence produit par notre propre code.

mod common;

use givre_lab::lancer::{ErreurLancer, lancer};
use givre_lab::scenario::lire_scenario;

fn lancer_texte(texte: &str, graine: u64) -> givre_lab::lancer::Sortie {
    let scenario = lire_scenario(texte).expect("scénario valide");
    lancer(&scenario, graine).expect("lancer")
}

/// juste : l'empreinte est le SHA-256 de l'écriture canonique, vérifiée sur neuf cas dont les octets sont écrits en toutes
/// lettres (V-LANC-01) : `GIVRE000`, l'octet de forme, les dimensions en `u32` petit-boutiste (largeur puis hauteur ; ou le
/// rayon), puis l'état de `Alea::new(graine)` (état puis incrément `6D 00…`). Exemple, `boite 3 2` graine 7 :
/// `4749565245303030 00 03000000 02000000 d1a0bdb386a42205 6d00000000000000`. Les SHA-256 viennent de Python `hashlib`. Une
/// largeur et une hauteur échangées, un octet de forme faux, un `u32` grand-boutiste ou un état déjà tiré échouent ici.
///
/// Ne couvre pas : le contenu de `mesures` et de `apercu` (autres tests).
#[test]
#[cfg_attr(not(feature = "etape-0"), ignore)]
fn empreinte_canonique_a_la_main() {
    // (scénario, graine, écriture canonique en hexadécimal, SHA-256 attendu)
    let cas: [(&str, u64, &str, &str); 9] = [
        (
            "boite 3 2",
            7,
            "4749565245303030000300000002000000d1a0bdb386a422056d00000000000000",
            "f637e3609ca6fe6a4c2554cefe8506e4ce67a8c696d0c6629f2a7ccc12b95530",
        ),
        (
            "boite 2 3",
            7,
            "4749565245303030000200000003000000d1a0bdb386a422056d00000000000000",
            "968a78c8dc36ae226297e7d488791d920d6c160c9746c21b1cf1a2315dd00d56",
        ),
        (
            "boite 64 48",
            1,
            "4749565245303030004000000030000000c3a53ce876eb36f36d00000000000000",
            "5a21e2aef0d732996015dcca8b79365294ccd49900bec7a833e498f92455f613",
        ),
        (
            "boite 64 48",
            2,
            "4749565245303030004000000030000000f024d234a4df884b6d00000000000000",
            "584cfbd290d38b53e89f19d3e8e500dc8c6b349c2e788bd47e28380df31c778a",
        ),
        (
            "boite 1 1",
            0,
            "47495652453030300001000000010000009626a79b49f7e49a6d00000000000000",
            "c242f3b686330c895ba6cd8557fe60f227502b66263bb115d21f613014e3b909",
        ),
        (
            "boite 4096 1",
            u64::MAX,
            "474956524530303000001000000100000069a7114f1c0393426d00000000000000",
            "421fddb2840900f39d68d4aad555cec5fea4cd8745968f8676150661b7ad3421",
        ),
        (
            "hexagone 1",
            7,
            "47495652453030300101000000d1a0bdb386a422056d00000000000000",
            "ef2dffc85155b1f49bc062ee8c6b5b19ff5215b245fecc98bbac3a1d6eaa316a",
        ),
        (
            "hexagone 3",
            7,
            "47495652453030300103000000d1a0bdb386a422056d00000000000000",
            "a282a3db378fc918d513dac1433866c804a6c1b76ae1a07dfd66687ede3f9788",
        ),
        (
            "hexagone 2047",
            42,
            "474956524530303001ff070000f8032e2cb80657186d00000000000000",
            "4d443e2aaf2162d839dd5f7bebfa9c30ca8e5b6e3addc55496d176b3d1e944fc",
        ),
    ];
    for (texte, graine, octets_hex, sha) in cas {
        let attendu_longueur = if texte.starts_with("boite") { 33 } else { 29 };
        assert_eq!(octets_hex.len(), 2 * attendu_longueur, "{texte} : longueur de l'écriture canonique");
        assert_eq!(lancer_texte(texte, graine).empreinte, sha, "{texte}, graine {graine}");
    }
}

/// juste : les mesures sont le CSV `temps,cases\n0,<cases>\n` : `largeur · hauteur` pour une boîte (6, 3072, 4096), `3R(R+1) + 1`
/// pour un hexagone (7, 37, 12 576 769 pour `R = 2047`), chaque ligne finie par `\n`, sans `\r` (V-LANC-02, la doc ; le nombre
/// de cases de l'hexagone est la suite A003215 de l'OEIS).
///
/// Ne couvre pas : l'empreinte et l'aperçu.
#[test]
#[cfg_attr(not(feature = "etape-0"), ignore)]
fn mesures_csv() {
    for (texte, cases) in [
        ("boite 3 2", 6_u64),
        ("boite 64 48", 3072),
        ("boite 4096 1", 4096),
        ("boite 1 1", 1),
        ("hexagone 1", 7),
        ("hexagone 3", 37),
        ("hexagone 2047", 12_576_769),
    ] {
        let sortie = lancer_texte(texte, 3);
        assert_eq!(sortie.mesures, format!("temps,cases\n0,{cases}\n"), "{texte}");
    }
}

/// juste : l'aperçu est un PNG que le décodeur du test relit : pour une boîte, aux dimensions de la boîte ; pour un hexagone de
/// rayon `R`, un carré de côté `2R + 1` ; les pixels sont `(v, v, v)` avec `v` les 8 bits de poids fort des tirages de
/// `Alea::new(graine)` (modèle Python : graine 7 donne `164, 108, 25, 183, 185, 125, 208, 195, …`) (V-LANC-03).
///
/// Ne couvre pas : la comparaison d'un aperçu complet pour de grandes dimensions (voir `etape_0_rendu`).
#[test]
#[cfg_attr(not(feature = "etape-0"), ignore)]
fn apercu_est_un_png_de_la_graine() {
    let v7 = [
        164_u8, 108, 25, 183, 185, 125, 208, 195, 140, 168, 18, 10, 203, 225, 110, 227, 61, 98, 86, 119, 224, 68, 49,
        70, 57,
    ];
    let png = lancer_texte("boite 3 2", 7).apercu;
    let lu = common::relire_png(&png).expect("PNG valide");
    assert_eq!((lu.largeur, lu.hauteur), (3, 2));
    assert_eq!(lu.rvb, v7[..6].iter().flat_map(|&v| [v, v, v]).collect::<Vec<u8>>());
    let png = lancer_texte("hexagone 2", 7).apercu;
    let lu = common::relire_png(&png).expect("PNG valide");
    assert_eq!((lu.largeur, lu.hauteur), (5, 5));
    assert_eq!(lu.rvb, v7.iter().flat_map(|&v| [v, v, v]).collect::<Vec<u8>>());
    let png = lancer_texte("boite 64 48", 1).apercu;
    let lu = common::relire_png(&png).expect("PNG valide");
    assert_eq!((lu.largeur, lu.hauteur), (64, 48));
    assert_eq!(lu.rvb.len(), 3 * 64 * 48);
    assert_eq!(lu.rvb[..6], [155, 155, 155, 12, 12, 12]);
    // SHA-256 des 9 216 octets de pixels de 64 × 48 (Python hashlib sur les 3 072 tirages de la graine 1)
    assert_eq!(
        common::hex(&givre_lab::empreinte::sha256(&lu.rvb)),
        "ad8e00dcb2adb40c9ca20ecffe40a48e3312f8b8d29b96e5834594acd9fc6a74"
    );
}

/// juste : même scénario et même graine donnent exactement la même sortie (empreinte, mesures, aperçu) ; vingt graines
/// donnent vingt empreintes distinctes ; une boîte de 3 × 2 et une de 2 × 3, une boîte et un hexagone, deux rayons, donnent des
/// empreintes distinctes ; l'empreinte a 64 chiffres hexadécimaux minuscules (V-LANC-04). La preuve de l'empreinte elle-même
/// est dans `empreinte_canonique_a_la_main`.
///
/// Ne couvre pas : l'égalité entre Linux et Windows (c'est la CI qui la rejoue, contre `references/*.empreinte`).
#[test]
#[cfg_attr(not(feature = "etape-0"), ignore)]
fn determinisme_et_graines_distinctes() {
    let a = lancer_texte("boite 8 6", 5);
    assert_eq!(a, lancer_texte("boite 8 6", 5));
    let mut vues = std::collections::BTreeSet::new();
    for graine in 0..20_u64 {
        let empreinte = lancer_texte("boite 8 6", graine).empreinte;
        assert_eq!(empreinte.len(), 64);
        assert!(empreinte.bytes().all(|c| c.is_ascii_digit() || (b'a'..=b'f').contains(&c)), "{empreinte}");
        assert!(vues.insert(empreinte), "deux graines ont la même empreinte (graine {graine})");
    }
    let formes = ["boite 3 2", "boite 2 3", "hexagone 3", "hexagone 2", "boite 7 1", "boite 1 7"];
    let empreintes: std::collections::BTreeSet<String> = formes.iter().map(|f| lancer_texte(f, 4).empreinte).collect();
    assert_eq!(empreintes.len(), formes.len());
}

/// juste : l'étape 0 ne comprend aucune directive en plus de la forme : la première directive restante, dans l'ordre du
/// fichier, fait rendre `ErreurLancer::DirectiveInconnue(numéro de sa ligne)`, lignes ignorées comptées, que la directive
/// vienne avant ou après la forme (V-LANC-05, la doc de `lancer`).
///
/// Ne couvre pas : ce que feront les étapes suivantes de leurs directives.
#[test]
#[cfg_attr(not(feature = "etape-0"), ignore)]
fn directive_inconnue_refusee() {
    let essai = |texte: &str| lancer(&lire_scenario(texte).unwrap(), 1).map(|_| ());
    assert_eq!(essai("boite 4 3\n# c\ninconnue 1 2\nautre 3\n"), Err(ErreurLancer::DirectiveInconnue(3)));
    assert_eq!(essai("autre 1\nboite 2 2\ninconnue 2\n"), Err(ErreurLancer::DirectiveInconnue(1)));
    assert_eq!(essai("\n\nhexagone 2\n\ninconnue\n"), Err(ErreurLancer::DirectiveInconnue(5)));
    assert_eq!(essai("boite 2 2\n"), Ok(()));
}

/// régression : l'empreinte du scénario `scenarios/vide.txt` avec la graine 1 est celle du fichier
/// `references/vide-graine-1.empreinte` (l'empreinte et un saut de ligne), produite par le binaire sur ce scénario ; ce
/// fichier est ce que la CI rejoue sous Linux et sous Windows. Il n'existe pas dans le jeu de tests : c'est la vague du code
/// qui le produit (`givre lancer scenarios/vide.txt --graine 1 --sortie <dossier>` puis copie de `empreinte.txt`).
///
/// Ne couvre pas : que l'empreinte soit juste (`empreinte_canonique_a_la_main` le prouve : même scénario et même graine, le
/// SHA-256 de l'écriture canonique à la main est `5a21e2ae…f613`). Ce test dit seulement que rien n'a bougé.
#[test]
#[cfg_attr(not(feature = "etape-0"), ignore)]
fn regression_reference_vide_graine_1() {
    let racine = env!("CARGO_MANIFEST_DIR");
    let reference = std::fs::read_to_string(format!("{racine}/references/vide-graine-1.empreinte"))
        .expect("references/vide-graine-1.empreinte est absente : la vague du code la produit avec le binaire");
    let texte = std::fs::read_to_string(format!("{racine}/scenarios/vide.txt")).expect("scenarios/vide.txt");
    let sortie = lancer(&lire_scenario(&texte).unwrap(), 1).unwrap();
    assert_eq!(reference, format!("{}\n", sortie.empreinte));
}

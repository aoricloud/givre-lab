//! Étape 2s : l'écriture canonique de l'état, l'empreinte, et la directive `surface` de `lancer` (mesures CSV, aperçu PNG,
//! refus), avec le binaire. Valeurs attendues : `references/etape-2s-valeurs.md`, identifiants `V-ECR-xx`, `V-LAN-xx`.

mod common;
mod etape_2s_commun;

use etape_2s_commun::{NOMS, Naif, hex, rugosite_naive};
use givre_lab::alea::Alea;
use givre_lab::empreinte::sha256_hex;
use givre_lab::lancer::{ErreurLancer, Sortie, lancer};
use givre_lab::scenario::lire_scenario;
use givre_lab::surface::{Modele, Surface};

fn modele(nom: &str) -> Modele {
    Modele::depuis_nom(nom).expect("nom de modèle")
}

fn lancer_texte(texte: &str, graine: u64) -> Result<Sortie, ErreurLancer> {
    lancer(&lire_scenario(texte).expect("scénario valide"), graine)
}

/// juste : l'écriture canonique de l'état, écrite OCTET PAR OCTET à la main (V-ECR-01, la doc de `surface`) : `L = 1`, graine 7,
/// trois particules, pour les quatre modèles. L'étiquette `GIVRE2S0` (`47 49 56 52 45 32 53 30`), l'octet du modèle (0, 1, 2, 3),
/// la largeur `01 00 00 00`, les particules `03` sur 8 octets, la seule hauteur `03` sur 8 octets (les quatre modèles donnent 3 sur
/// une colonne), l'état du générateur après trois tirages : `d0c2a4bb078b789b` puis l'incrément `6d00000000000000` (valeurs de
/// `ref_alea.py`, la transcription Python de PCG32) ; Eden ajoute `01000000` (un site au périmètre), son site `(0, 3)` en
/// `00000000 03000000`, `03000000` (trois rangées) et l'occupation `01 01 01`. Le générateur du test, un `Alea::new(7)` tiré trois
/// fois, redonne les 16 octets du générateur. L'écriture complète vient du second prototype Python.
///
/// Ne couvre pas : les hauteurs différentes d'un modèle à l'autre (le test suivant les compare au naïf).
#[test]
#[cfg_attr(not(feature = "etape-2s"), ignore)]
fn octets_a_la_main_une_colonne() {
    let mut alea = Alea::new(7);
    for _ in 0..3 {
        alea.suivant();
    }
    let generateur = "d0c2a4bb078b789b6d00000000000000";
    assert_eq!(hex(&alea.octets()), generateur);
    for (nom, code) in [("depot", "00"), ("relaxation", "01"), ("balistique", "02"), ("eden", "03")] {
        let mut s = Surface::new(modele(nom), 1, 7);
        for _ in 0..3 {
            s.deposer();
        }
        let mut attendu = String::new();
        attendu += "4749565245325330"; // GIVRE2S0
        attendu += code; // le modèle
        attendu += "01000000"; // la largeur
        attendu += "0300000000000000"; // trois particules
        attendu += "0300000000000000"; // h[0] = 3
        attendu += generateur; // Alea::octets()
        if nom == "eden" {
            attendu += "01000000"; // un site au périmètre
            attendu += "0000000003000000"; // (0, 3)
            attendu += "03000000"; // trois rangées
            attendu += "010101"; // occupation de (0, 0), (0, 1), (0, 2)
        }
        assert_eq!(hex(&s.octets()), attendu, "{nom}");
    }
}

/// juste : `octets()` est l'écriture canonique de la doc pour des états quelconques (V-ECR-02) : l'état initial et les états après
/// 1, 50, 200 particules, pour les quatre modèles, L = 3 et 8, graines 2 et 9, égaux à ce qu'écrit l'encodeur du test (`Naif::octets`,
/// écrit octet par octet d'après la doc : `u32` et `u64` petits-boutistes, périmètre dans l'ordre du `Vec`, occupation rangée par rangée),
/// y compris pour Eden (périmètre et occupation) et pour l'état initial (zéro particule, périmètre de départ, zéro rangée).
///
/// Ne couvre pas : la largeur 4096 (la largeur est un `u32` petit-boutiste dans tous les cas).
#[test]
#[cfg_attr(not(feature = "etape-2s"), ignore)]
fn octets_contre_encodeur_du_test() {
    for nom in NOMS {
        for l in [3_usize, 8] {
            for graine in [2_u64, 9] {
                let mut s = Surface::new(modele(nom), l, graine);
                let mut naif = Naif::new(nom, l, graine);
                assert_eq!(s.octets(), naif.octets(), "{nom} L={l} graine {graine} : état initial");
                for fait in [1_usize, 50, 200] {
                    let deja = s.particules() as usize;
                    for _ in deja..fait {
                        s.deposer();
                        naif.pas();
                    }
                    assert_eq!(s.octets(), naif.octets(), "{nom} L={l} graine {graine} après {fait} particules");
                }
            }
        }
    }
}

/// juste : l'empreinte est le SHA-256 de l'écriture canonique, en 64 chiffres hexadécimaux minuscules (V-ECR-03) : égale à
/// `sha256_hex(&octets())` ; et pour 8 états les SHA-256 calculés par Python `hashlib` sur l'écriture canonique du second prototype :
/// (L, graine, particules) = (8, 1, 200) et (5, 4, 120) pour chacun des quatre modèles ; et pour l'état initial de `L = 3`, graine
/// 2, des quatre modèles (zéro particule : Eden y écrit son périmètre de départ et zéro rangée).
///
/// Ne couvre pas : l'égalité entre Linux et Windows (la CI rejoue `references/surface-graine-1.empreinte`).
#[test]
#[cfg_attr(not(feature = "etape-2s"), ignore)]
fn empreinte_sha256_de_l_ecriture() {
    let cas: [(&str, usize, u64, u64, &str); 8] = [
        ("depot", 8, 1, 200, "155dfd914346577853337d6f192c1bd00c0632562b881a9b4c87892b25243491"),
        ("depot", 5, 4, 120, "10338b285f1a8d29b325ced56f448a7935b2fe1128e4a451836ef86f0a0d0b8f"),
        ("relaxation", 8, 1, 200, "e94f60a2e2cc7fc702c4e7ac4003bc8d7af51514cf9ce05c965c2dd79bffbfa6"),
        ("relaxation", 5, 4, 120, "15555fce49ea408383da08dfec4c6190b9a0ad12aa90bfd9f97db48eb47fdd90"),
        ("balistique", 8, 1, 200, "27624bae0aaddffc10e50f97b1e007060a6b4ff56ce2d6f885fceec23454b0c9"),
        ("balistique", 5, 4, 120, "c7a7c5cb629a2ac5efd1aa7e5e029d64eb9280fae728a220ccf631fe6befa6d0"),
        ("eden", 8, 1, 200, "ca66ba6e78a96bc841950510f51db635fb704390b904172ada7e340f7dd2080e"),
        ("eden", 5, 4, 120, "0b1e3472ba4184a2bb57b8107cff9bb9e08abec685f3ee6b4fa8a3b51c94f163"),
    ];
    for (nom, l, graine, n, sha) in cas {
        let mut s = Surface::new(modele(nom), l, graine);
        for _ in 0..n {
            s.deposer();
        }
        assert_eq!(s.empreinte(), sha, "{nom} L={l} graine {graine} N={n}");
        assert_eq!(s.empreinte(), sha256_hex(&s.octets()));
        assert_eq!(s.empreinte().len(), 64);
    }
    // l'état initial de L = 3, graine 2 : Python hashlib
    for (nom, sha) in [
        ("depot", "59f2891cfed23fb7dd9084a5c587964ffdc83d431e0e8c1ad5121b57f941ccec"),
        ("relaxation", "0181515216b88c3afd18c62d6325b4f53bbb585375846d479240b94d83b6b0b7"),
        ("balistique", "b08e9844104d6278211b6ae8e68609c3bb554bbe1daba888d3dba166cea9c926"),
        ("eden", "c42433a8d1dbb7f4c40910d62c1fd85c1558c752e9ba8f6866e955d18d5614d3"),
    ] {
        assert_eq!(Surface::new(modele(nom), 3, 2).empreinte(), sha, "{nom} : état initial");
    }
}

/// Les instants du CSV, écrits ici autrement que dans la doc : les puissances de 2 jusqu'à `couches`, puis `couches`.
fn instants(couches: u64) -> Vec<u64> {
    let mut v = Vec::new();
    let mut p = 1;
    while p <= couches {
        v.push(p);
        p *= 2;
    }
    if *v.last().unwrap() != couches {
        v.push(couches);
    }
    v
}

/// Le CSV attendu : refait avec le simulateur naïf et les entiers exacts, `w` par une division et une racine, six décimales.
fn csv_attendu(nom: &'static str, l: usize, couches: u64, graine: u64) -> String {
    let mut naif = Naif::new(nom, l, graine);
    let mut texte = String::from("couches,rugosite\n");
    let mut fait = 0;
    for t in instants(couches) {
        while fait < t * l as u64 {
            naif.pas();
            fait += 1;
        }
        let (num, den) = rugosite_naive(&naif.hauteurs());
        let w = (num as f64 / den as f64).sqrt();
        texte += &format!("{t},{w:.6}\n");
    }
    texte
}

/// juste : les mesures de `lancer` pour `surface <modele> <largeur> <couches>` sont le CSV `couches,rugosite` : un en-tête puis une
/// ligne aux couches 1, 2, 4, 8, … (puissances de 2 au plus égales à `couches`) puis `couches` s'il n'en est pas une, la rugosité
/// `w` écrite avec six décimales, chaque ligne finie par `\n` (V-LAN-01, la doc de `surface`). Refait ici avec le simulateur naïf
/// (entiers exacts, `w = sqrt(num / den)`) pour les quatre modèles, L = 8 et 5, graine 3, `couches` = 1, 2, 3, 16, 17, 20 (aucun
/// instant en double : `couches = 16` donne 1, 2, 4, 8, 16 ; `couches = 1` une seule ligne ; `couches = 2` : 1 et 2) ; et L = 4 avec
/// 100 couches : 1, 2, 4, 8, 16, 32, 64, 100.
///
/// Ne couvre pas : la valeur de `w` à plus de six décimales (le test compare des chaînes).
#[test]
#[cfg_attr(not(feature = "etape-2s"), ignore)]
fn mesures_csv() {
    for nom in NOMS {
        for (l, couches) in [(8_usize, 1_u64), (8, 2), (8, 3), (8, 16), (8, 17), (5, 20), (4, 100)] {
            let texte = format!("boite 8 4\nsurface {nom} {l} {couches}\n");
            let sortie = lancer_texte(&texte, 3).expect("lancer");
            assert_eq!(sortie.mesures, csv_attendu(nom, l, couches, 3), "{nom} L={l} couches={couches}");
        }
    }
    let sortie = lancer_texte("boite 8 4\nsurface depot 4 100\n", 3).expect("lancer");
    let temps: Vec<&str> = sortie.mesures.lines().skip(1).map(|l| l.split(',').next().unwrap()).collect();
    assert_eq!(temps, ["1", "2", "4", "8", "16", "32", "64", "100"]);
    assert!(sortie.mesures.ends_with('\n') && !sortie.mesures.contains('\r'));
}

/// juste : l'empreinte de `lancer` est celle de la surface finale, le SHA-256 de l'écriture canonique que le simulateur naïf du test
/// écrit octet par octet (V-LAN-02) : pour les quatre modèles, L = 8, 12 couches, graine 5 ; elle ne dépend pas de la forme du
/// fichier (une `boite` de n'importe quelle taille, ou un `hexagone`, que `surface` ignore), ni des directives de forme, et deux
/// graines donnent deux empreintes.
///
/// Ne couvre pas : l'égalité entre Linux et Windows (la CI).
#[test]
#[cfg_attr(not(feature = "etape-2s"), ignore)]
fn empreinte_et_forme_ignoree() {
    for nom in NOMS {
        let mut naif = Naif::new(nom, 8, 5);
        for _ in 0..8 * 12 {
            naif.pas();
        }
        let attendu = sha256_hex(&naif.octets());
        for forme in ["boite 8 12", "boite 3 3", "boite 4096 4096", "hexagone 5", "hexagone 1"] {
            let sortie = lancer_texte(&format!("{forme}\nsurface {nom} 8 12\n"), 5).expect("lancer");
            assert_eq!(sortie.empreinte, attendu, "{nom} avec {forme}");
        }
        let avant = lancer_texte(&format!("surface {nom} 8 12\nboite 3 3\n"), 5).expect("lancer");
        assert_eq!(avant.empreinte, attendu, "{nom} : la forme après la directive");
        let autre = lancer_texte(&format!("boite 8 12\nsurface {nom} 8 12\n"), 6).expect("lancer");
        assert_ne!(autre.empreinte, attendu, "{nom} : une autre graine");
        let en_tete = lancer_texte(&format!("# a\n\nboite 8 12\n\n  surface   {nom}\t8   012  \n"), 5).expect("lancer");
        assert_eq!(en_tete.empreinte, attendu, "{nom} : espaces, tabulation et zéros de tête permis");
    }
}

/// L'aperçu attendu d'après la doc, refait avec le simulateur naïf : `(largeur, hauteur, rvb)`.
fn apercu_attendu(nom: &'static str, l: usize, couches: u64, graine: u64) -> (u32, u32, Vec<u8>) {
    let mut naif = Naif::new(nom, l, graine);
    let mut profils: Vec<Vec<u64>> = vec![vec![0; l]];
    for _ in 1..=couches {
        for _ in 0..l {
            naif.pas();
        }
        profils.push(naif.hauteurs());
    }
    let finale = profils.last().unwrap().clone();
    let haut = (*finale.iter().max().unwrap()).max(1) as usize;
    let diviseur = (couches - 1).max(1);
    let mut rvb = Vec::new();
    for y in 0..haut {
        let r = (haut - 1 - y) as u64;
        for x in 0..l {
            if r >= finale[x] {
                rvb.extend_from_slice(&[240, 240, 240]);
            } else {
                // la première couche a après laquelle la hauteur dépasse r
                let a = (1..=couches).find(|&a| profils[a as usize][x] > r).unwrap();
                let v = (a - 1) * 255 / diviseur;
                rvb.extend_from_slice(&[v as u8, 96, (255 - v) as u8]);
            }
        }
    }
    (l as u32, haut as u32, rvb)
}

/// juste : l'aperçu de `lancer` est le PNG que décrit la doc de `surface` (V-LAN-03) : `largeur` pixels de large,
/// `max(1, h_max)` de haut ; le pixel de la rangée `r` (comptée depuis le bas) de la colonne `x` est le fond `(240, 240, 240)` si
/// `r >= h[x]`, sinon `(v, 96, 255 - v)` avec `v = (a - 1) · 255 / max(couches - 1, 1)` où `a` est la première couche complète
/// après laquelle la hauteur de la colonne dépasse `r`. Le décodeur du test (`common::relire_png`, indépendant du crate) relit le
/// PNG ; ses dimensions et TOUS ses pixels sont comparés à ceux que refait le simulateur naïf couche par couche : quatre modèles,
/// L = 8 / 5 / 1, 12 / 7 / 1 / 30 couches (dont une seule couche : `v = 0`, tout en `(0, 96, 255)`).
///
/// Ne couvre pas : l'esthétique (c'est pour l'œil).
#[test]
#[cfg_attr(not(feature = "etape-2s"), ignore)]
fn apercu_png() {
    for nom in NOMS {
        for (l, couches) in [(8_usize, 12_u64), (5, 7), (1, 6), (8, 1), (6, 30)] {
            let sortie = lancer_texte(&format!("boite 8 4\nsurface {nom} {l} {couches}\n"), 4).expect("lancer");
            let lu = common::relire_png(&sortie.apercu).expect("PNG valide");
            let (largeur, hauteur, rvb) = apercu_attendu(nom, l, couches, 4);
            assert_eq!((lu.largeur, lu.hauteur), (largeur, hauteur), "{nom} L={l} couches={couches}");
            assert_eq!(lu.rvb, rvb, "{nom} L={l} couches={couches} : pixels");
        }
    }
    // une seule couche : aucune rangée n'a la couleur du fond, la couleur est (0, 96, 255)
    let sortie = lancer_texte("boite 4 4\nsurface depot 4 1\n", 4).expect("lancer");
    let lu = common::relire_png(&sortie.apercu).unwrap();
    assert!(lu.rvb.chunks(3).all(|p| p == [240, 240, 240] || p == [0, 96, 255]));
    assert!(lu.rvb.chunks(3).any(|p| p == [0, 96, 255]));
}

/// juste : les refus de la directive `surface` (V-LAN-04, la doc) : `ErreurLancer::DirectiveInconnue(numéro de ligne de la première
/// directive fautive)`, lignes ignorées comptées. Fautes : deux ou quatre arguments ; un modèle inconnu ou d'une autre casse ;
/// `largeur` 0, 4097, négative, décimale, avec un signe `+`, hexadécimale ; `couches` 0, 1025, écrite en lettres ; la directive
/// n'est pas la première des directives (une inconnue avant) ; une seconde directive `surface` ou une directive inconnue après
/// une `surface` valide (c'est la ligne de la seconde) ; une `surface` fautive suivie d'une inconnue (la ligne de la fautive).
/// Acceptés : `largeur` 4096 avec 1 couche, `largeur` 1 avec 1024 couches (les bornes incluses), les zéros de tête.
///
/// Ne couvre pas : les erreurs de lecture du scénario (c'est `etape_0_scenario`).
#[test]
#[cfg_attr(not(feature = "etape-2s"), ignore)]
fn refus_de_la_directive_surface() {
    let refus = |texte: &str| lancer_texte(texte, 1).map(|_| ());
    // (directive fautive, ligne attendue) : la directive est écrite après `boite 4 4` (ligne 1) et un commentaire (ligne 2)
    for mauvaise in [
        "surface depot 8",
        "surface depot 8 5 3",
        "surface depot",
        "surface",
        "surface inconnu 8 5",
        "surface Depot 8 5",
        "surface eden. 8 5",
        "surface depot 0 5",
        "surface depot 4097 5",
        "surface depot -8 5",
        "surface depot 8.0 5",
        "surface depot +8 5",
        "surface depot 0x8 5",
        "surface depot 8 0",
        "surface depot 8 1025",
        "surface depot 8 cinq",
        "surface depot 8 -5",
        "surface depot 99999999999999999999 5",
    ] {
        assert_eq!(
            refus(&format!("boite 4 4\n# note\n{mauvaise}\n")),
            Err(ErreurLancer::DirectiveInconnue(3)),
            "{mauvaise}"
        );
    }
    assert_eq!(refus("inconnue 1\nsurface depot 8 5\nboite 4 4\n"), Err(ErreurLancer::DirectiveInconnue(1)));
    assert_eq!(refus("boite 4 4\nsurface depot 8 5\nsurface eden 8 5\n"), Err(ErreurLancer::DirectiveInconnue(3)));
    assert_eq!(refus("boite 4 4\nsurface depot 8 5\n\n# c\nautre 2\n"), Err(ErreurLancer::DirectiveInconnue(5)));
    assert_eq!(refus("boite 4 4\nsurface depot 8\nautre 2\n"), Err(ErreurLancer::DirectiveInconnue(2)));
    assert_eq!(refus("boite 4 4\nsurface depot 8 5\nsurface depot 8 5 5\n"), Err(ErreurLancer::DirectiveInconnue(3)));
    assert_eq!(refus("boite 4 4\nsurface depot 4096 1\n"), Ok(()));
    assert_eq!(refus("boite 4 4\nsurface depot 1 1024\n"), Ok(()));
    assert_eq!(refus("boite 4 4\nsurface eden 1 1024\n"), Ok(()));
    assert_eq!(refus("boite 4 4\nsurface relaxation 008 005\n"), Ok(()));
}

/// juste : le scénario d'exemple `scenarios/surface.txt` se lit et se lance (V-LAN-05) : la relaxation sur 64 colonnes et 200 couches
/// (le fichier écrit `boite 64 200` et `surface relaxation 64 200`) ; ses mesures sont celles du simulateur naïf (graine 1) et son
/// aperçu a 64 pixels de large.
///
/// Ne couvre pas : les autres scénarios.
#[test]
#[cfg_attr(not(feature = "etape-2s"), ignore)]
fn scenario_d_exemple() {
    let racine = env!("CARGO_MANIFEST_DIR");
    let texte = std::fs::read_to_string(format!("{racine}/scenarios/surface.txt")).expect("scenarios/surface.txt");
    let scenario = lire_scenario(&texte).expect("le scénario d'exemple se lit");
    assert_eq!(scenario.directives.len(), 1);
    assert_eq!(scenario.directives[0].nom, "surface");
    assert_eq!(scenario.directives[0].args, ["relaxation", "64", "200"]);
    let sortie = lancer(&scenario, 1).expect("lancer");
    assert_eq!(sortie.mesures, csv_attendu("relaxation", 64, 200, 1));
    let lu = common::relire_png(&sortie.apercu).expect("PNG valide");
    assert_eq!(lu.largeur, 64);
    let mut naif = Naif::new("relaxation", 64, 1);
    for _ in 0..64 * 200 {
        naif.pas();
    }
    assert_eq!(sortie.empreinte, sha256_hex(&naif.octets()));
}

/// régression : l'empreinte du scénario `scenarios/surface.txt` avec la graine 1 est celle du fichier
/// `references/surface-graine-1.empreinte` (l'empreinte et un saut de ligne), produite par le binaire sur ce scénario ; ce fichier est
/// ce que la CI rejoue sous Linux et sous Windows. Il n'existe pas dans le jeu de tests : c'est la vague du code qui le produit
/// (`givre lancer scenarios/surface.txt --graine 1 --sortie <dossier>` puis copie de `empreinte.txt`).
///
/// Ne couvre pas : que l'empreinte soit juste (`scenario_d_exemple` la recalcule avec le simulateur naïf : même scénario, même
/// graine). Ce test dit seulement que rien n'a bougé.
#[test]
#[cfg_attr(not(feature = "etape-2s"), ignore)]
fn regression_reference_surface_graine_1() {
    let racine = env!("CARGO_MANIFEST_DIR");
    let reference = std::fs::read_to_string(format!("{racine}/references/surface-graine-1.empreinte"))
        .expect("references/surface-graine-1.empreinte est absente : la vague du code la produit avec le binaire");
    let texte = std::fs::read_to_string(format!("{racine}/scenarios/surface.txt")).expect("scenarios/surface.txt");
    let sortie = lancer(&lire_scenario(&texte).unwrap(), 1).unwrap();
    assert_eq!(reference, format!("{}\n", sortie.empreinte));
}

/// juste : le binaire `givre lancer` sur une directive `surface` (V-LAN-06) : `givre lancer scenarios/surface.txt --graine 1 --sortie
/// <dossier>` réussit (code 0) et écrit exactement `apercu.png`, `empreinte.txt`, `mesures.csv` ; leur contenu est celui de `lancer`
/// (l'empreinte, un `\n`) ; un scénario dont la directive `surface` est refusée (`surface eden 8 0`, ligne 2) donne le code 1,
/// un message qui contient `ligne 2` sans `panicked`, et aucun dossier de sortie créé.
///
/// Ne couvre pas : les autres erreurs du binaire (étape 0).
#[test]
#[cfg_attr(not(feature = "etape-2s"), ignore)]
fn binaire_surface() {
    use std::process::Command;
    let racine = env!("CARGO_MANIFEST_DIR");
    let dossier = common::dossier_temporaire("surface-bin");
    let exemple = format!("{racine}/scenarios/surface.txt");
    let sortie = dossier.join("sortie");
    let resultat = Command::new(env!("CARGO_BIN_EXE_givre"))
        .args(["lancer", &exemple, "--graine", "1", "--sortie", sortie.to_str().unwrap()])
        .output()
        .expect("lancement de givre");
    assert_eq!(resultat.status.code(), Some(0), "{}", String::from_utf8_lossy(&resultat.stderr));
    let mut noms: Vec<String> =
        std::fs::read_dir(&sortie).unwrap().map(|e| e.unwrap().file_name().into_string().unwrap()).collect();
    noms.sort();
    assert_eq!(noms, ["apercu.png", "empreinte.txt", "mesures.csv"]);
    let attendu = lancer(&lire_scenario(&std::fs::read_to_string(&exemple).unwrap()).unwrap(), 1).unwrap();
    assert_eq!(std::fs::read_to_string(sortie.join("empreinte.txt")).unwrap(), format!("{}\n", attendu.empreinte));
    assert_eq!(std::fs::read_to_string(sortie.join("mesures.csv")).unwrap(), attendu.mesures);
    assert_eq!(std::fs::read(sortie.join("apercu.png")).unwrap(), attendu.apercu);

    let mauvais = dossier.join("mauvais.txt");
    std::fs::write(&mauvais, "boite 4 4\nsurface eden 8 0\n").unwrap();
    let refus = dossier.join("refus");
    let resultat = Command::new(env!("CARGO_BIN_EXE_givre"))
        .args(["lancer", mauvais.to_str().unwrap(), "--graine", "1", "--sortie", refus.to_str().unwrap()])
        .output()
        .expect("lancement de givre");
    assert_eq!(resultat.status.code(), Some(1));
    let message = String::from_utf8_lossy(&resultat.stderr).into_owned();
    assert!(message.contains("ligne 2") && !message.contains("panicked"), "{message}");
    assert!(!refus.exists(), "le dossier de sortie ne doit pas être créé");
    let _ = std::fs::remove_dir_all(&dossier);
}

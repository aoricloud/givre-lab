//! Étape 0 : `scenario`, la lecture d'un fichier de scénario. Valeurs attendues : `references/etape-0-valeurs.md`,
//! identifiants `V-SCEN-xx`.
//!
//! Ici la source est la définition du format écrite dans la documentation de `scenario` (une convention de l'étape 0, pas
//! une valeur publiée) ; les cas sont des textes écrits à la main dont le résultat se lit sans calcul. Pour « directive
//! inconnue », ces tests n'emploient que `inconnue 1 2` et d'autres noms qu'aucune étape suivante ne réclamera (`x_1`,
//! `autre`, `sans_argument`) : jamais `dla`, `collage`, `bruit`, `surface`, `flocon`, `cinetique`…

use givre_lab::scenario::{COTE_MAX, Directive, ErreurLecture, Forme, RAYON_MAX, Scenario, lire_scenario};

fn lire(texte: &str) -> Result<Scenario, ErreurLecture> {
    lire_scenario(texte)
}

fn invalide(texte: &str) -> ErreurLecture {
    lire(texte).expect_err(&format!("{texte:?} devrait être refusé"))
}

fn directive(ligne: usize, nom: &str, args: &[&str]) -> Directive {
    Directive { ligne, nom: nom.to_string(), args: args.iter().map(|a| a.to_string()).collect() }
}

/// juste : les deux formes comprises : `boite 64 48` (largeur puis hauteur) et `hexagone 3`, sans autre directive
/// (V-SCEN-01) ; les bornes publiques valent 4096 et 2047.
///
/// Ne couvre pas : les erreurs (autres tests).
#[test]
#[cfg_attr(not(feature = "etape-0"), ignore)]
fn les_deux_formes() {
    let s = lire("boite 64 48\n").unwrap();
    assert_eq!(s.forme, Forme::Boite { largeur: 64, hauteur: 48 });
    assert!(s.directives.is_empty());
    let s = lire("hexagone 3\n").unwrap();
    assert_eq!(s.forme, Forme::Hexagone { rayon: 3 });
    assert!(s.directives.is_empty());
    assert_eq!(lire("boite 5 9").unwrap().forme, Forme::Boite { largeur: 5, hauteur: 9 }, "sans saut de ligne final");
    assert_eq!((COTE_MAX, RAYON_MAX), (4096, 2047));
}

/// juste : sont ignorées les lignes vides, celles qui ne contiennent que des espaces ou des tabulations, celles dont le
/// premier caractère non blanc est `#` (même indenté, même `#` seul) ; les champs peuvent être séparés par plusieurs
/// espaces ou des tabulations ; les espaces de bord et un `\r` final (fin de ligne Windows) sont ignorés (V-SCEN-02).
///
/// Ne couvre pas : les autres caractères blancs que l'espace, la tabulation et `\r`.
#[test]
#[cfg_attr(not(feature = "etape-0"), ignore)]
fn commentaires_blancs_et_fins_de_ligne() {
    let texte = "# un commentaire\n\n   \n\t\n  # indenté\n#\n   boite \t 8   6  \n# fin";
    let s = lire(texte).unwrap();
    assert_eq!(s.forme, Forme::Boite { largeur: 8, hauteur: 6 });
    assert!(s.directives.is_empty());
    let s = lire("# c\r\nboite 3 4\r\n\r\n").unwrap();
    assert_eq!(s.forme, Forme::Boite { largeur: 3, hauteur: 4 });
    assert!(s.directives.is_empty());
}

/// juste : les bornes : acceptés `boite 1 1`, `boite 4096 4096`, `boite 4096 1`, `boite 1 4096`, `hexagone 1`,
/// `hexagone 2047` ; refusés (ligne 1) `boite 0 5`, `boite 5 0`, `boite 4097 1`, `boite 1 4097`, `hexagone 0`,
/// `hexagone 2048`, et un entier qui ne tient pas dans un `u32` ; les zéros de tête sont permis (`064` vaut 64)
/// (V-SCEN-03).
///
/// Ne couvre pas : les autres entiers hors bornes que ceux-là.
#[test]
#[cfg_attr(not(feature = "etape-0"), ignore)]
fn bornes_des_formes() {
    for (texte, largeur, hauteur) in
        [("boite 1 1", 1, 1), ("boite 4096 4096", 4096, 4096), ("boite 4096 1", 4096, 1), ("boite 1 4096", 1, 4096)]
    {
        assert_eq!(lire(texte).unwrap().forme, Forme::Boite { largeur, hauteur }, "{texte}");
    }
    assert_eq!(lire("hexagone 1").unwrap().forme, Forme::Hexagone { rayon: 1 });
    assert_eq!(lire("hexagone 2047").unwrap().forme, Forme::Hexagone { rayon: 2047 });
    for texte in [
        "boite 0 5",
        "boite 5 0",
        "boite 0 0",
        "boite 4097 1",
        "boite 1 4097",
        "hexagone 0",
        "hexagone 2048",
        "boite 4294967297 1",
        "boite 1 99999999999999999999",
        "hexagone 4294967297",
    ] {
        assert_eq!(invalide(texte), ErreurLecture::LigneInvalide(1), "{texte}");
    }
    assert_eq!(lire("boite 064 048").unwrap().forme, Forme::Boite { largeur: 64, hauteur: 48 });
}

/// juste : un entier est écrit en chiffres décimaux ASCII seulement : sont refusés (ligne 1) `+5`, `-5`, `5.0`, `0x10`,
/// `1e2`, `5_0`, `5a`, des chiffres non ASCII (`٣`), et l'absence d'argument ou un argument en trop (V-SCEN-04) ; les mêmes
/// formes bien écrites (`boite 5 4`, `boite 3 4`, `hexagone 1`) se lisent (un code qui refuse tout échoue).
///
/// Ne couvre pas : les formes de `hexagone` autres que celles essayées.
#[test]
#[cfg_attr(not(feature = "etape-0"), ignore)]
fn entiers_et_nombre_d_arguments() {
    for texte in [
        "boite +5 4",
        "boite 5 +4",
        "boite -5 4",
        "boite 5.0 4",
        "boite 0x10 4",
        "boite 1e2 4",
        "boite 5_0 4",
        "boite 5a 4",
        "boite ٣ 4",
        "hexagone +1",
        "hexagone 1.0",
        "hexagone x",
        "boite",
        "boite 3",
        "boite 3 4 5",
        "hexagone",
        "hexagone 1 2",
        "boite 3 3 # une note",
    ] {
        assert_eq!(invalide(texte), ErreurLecture::LigneInvalide(1), "{texte}");
    }
    assert_eq!(lire("boite 5 4").unwrap().forme, Forme::Boite { largeur: 5, hauteur: 4 });
    assert_eq!(lire("boite 3 4").unwrap().forme, Forme::Boite { largeur: 3, hauteur: 4 });
    assert_eq!(lire("hexagone 1").unwrap().forme, Forme::Hexagone { rayon: 1 });
}

/// juste : exactement une forme : une seconde directive de forme, quelle qu'elle soit (`boite` après `boite`, `hexagone`
/// après `hexagone`, `boite` après `hexagone` et l'inverse), est la ligne fautive, même si ses arguments sont bons ; le
/// numéro est celui de la seconde, lignes ignorées comprises (V-SCEN-05).
///
/// Ne couvre pas : trois formes ou plus (la deuxième suffit à échouer).
#[test]
#[cfg_attr(not(feature = "etape-0"), ignore)]
fn forme_en_double() {
    assert_eq!(invalide("boite 3 3\nboite 4 4\n"), ErreurLecture::LigneInvalide(2));
    assert_eq!(invalide("boite 3 3\nboite 3 3\n"), ErreurLecture::LigneInvalide(2));
    assert_eq!(invalide("hexagone 2\nhexagone 2\n"), ErreurLecture::LigneInvalide(2));
    assert_eq!(invalide("boite 3 3\nhexagone 2\n"), ErreurLecture::LigneInvalide(2));
    assert_eq!(invalide("hexagone 2\n# c\n\nboite 3 3\n"), ErreurLecture::LigneInvalide(4));
}

/// juste : le nom d'une directive est fait de minuscules ASCII, de chiffres et de `_`, et n'est pas vide : sont mal formés
/// (ligne fautive 2) `Boite`, `BOITE`, `boite-2`, `a.b`, `é`, `x:1` ; sont bien formés `9x`, `_`, `a_1b2` (conservés comme
/// directives) (V-SCEN-06).
///
/// Ne couvre pas : les noms très longs.
#[test]
#[cfg_attr(not(feature = "etape-0"), ignore)]
fn noms_de_directive() {
    for nom in ["Boite", "BOITE", "boite-2", "a.b", "é", "x:1", "inconnue!"] {
        let texte = format!("boite 2 2\n{nom} 3 3\n");
        assert_eq!(invalide(&texte), ErreurLecture::LigneInvalide(2), "{nom}");
    }
    let s = lire("boite 2 2\n9x 1\n_\na_1b2 z\n").unwrap();
    assert_eq!(s.directives, [directive(2, "9x", &["1"]), directive(3, "_", &[]), directive(4, "a_1b2", &["z"])]);
}

/// juste : le numéro de ligne d'une erreur compte les lignes ignorées (vides, blanches, commentaires), à partir de 1 ; la
/// première ligne fautive l'emporte sur les suivantes et sur `Manquante` (V-SCEN-07).
///
/// Ne couvre pas : un fichier de plus de 100 lignes.
#[test]
#[cfg_attr(not(feature = "etape-0"), ignore)]
fn numero_de_ligne() {
    assert_eq!(invalide("# a\n\n   \nboite 3\n"), ErreurLecture::LigneInvalide(4));
    assert_eq!(invalide("boite 2 2\n\n\nBad 1\n"), ErreurLecture::LigneInvalide(4));
    assert_eq!(invalide("Bad\nboite 0 0\n"), ErreurLecture::LigneInvalide(1));
    assert_eq!(invalide("inconnue 1 2\nBad\n"), ErreurLecture::LigneInvalide(2));
    assert_eq!(invalide("boite 2 2\nhexagone 0\nboite 9 9\n"), ErreurLecture::LigneInvalide(2));
    assert_eq!(invalide("\n\n\n\n\nhexagone 2048\n"), ErreurLecture::LigneInvalide(6));
    assert_eq!(invalide("\r\n# x\r\nboite 3\r\n"), ErreurLecture::LigneInvalide(3));
}

/// juste : sans forme, l'erreur est `Manquante("forme")` : texte vide, que des blancs, que des commentaires, que d'autres
/// directives (V-SCEN-08) ; les mêmes textes suivis d'une forme se lisent (un code qui rend toujours `Manquante` échoue).
///
/// Ne couvre pas : l'ordre entre `Manquante` et une ligne fautive (voir `numero_de_ligne`).
#[test]
#[cfg_attr(not(feature = "etape-0"), ignore)]
fn forme_manquante() {
    for texte in ["", "\n\n", "   \n\t\n", "# rien\n\n", "inconnue 1 2\n", "# c\nx_1\nautre a b\n"] {
        assert_eq!(invalide(texte), ErreurLecture::Manquante("forme"), "{texte:?}");
        let avec_forme = format!("{texte}\nhexagone 1\n");
        assert_eq!(lire(&avec_forme).map(|s| s.forme), Ok(Forme::Hexagone { rayon: 1 }), "{avec_forme:?}");
    }
}

/// juste : toute autre directive bien formée est gardée sans être interprétée, dans l'ordre du fichier, avec son numéro de
/// ligne (lignes ignorées comptées), ses arguments tels qu'écrits (séparés par des blancs, zéro permis), les doublons
/// conservés ; la forme peut venir après des directives ; `directive(nom)` rend la première de ce nom ou `None`, et jamais
/// `boite` ni `hexagone` (V-SCEN-09).
///
/// Ne couvre pas : l'interprétation d'une directive (c'est le rôle de `lancer`).
#[test]
#[cfg_attr(not(feature = "etape-0"), ignore)]
fn directives_gardees_dans_l_ordre() {
    let texte = "# scénario\ninconnue 1 2\nboite 8 6\n\nx_1\n  autre   a  b\tc\ninconnue 3\nsans_argument\n";
    let s = lire(texte).unwrap();
    assert_eq!(s.forme, Forme::Boite { largeur: 8, hauteur: 6 });
    assert_eq!(
        s.directives,
        [
            directive(2, "inconnue", &["1", "2"]),
            directive(5, "x_1", &[]),
            directive(6, "autre", &["a", "b", "c"]),
            directive(7, "inconnue", &["3"]),
            directive(8, "sans_argument", &[]),
        ]
    );
    assert_eq!(s.directive("inconnue"), Some(&directive(2, "inconnue", &["1", "2"])));
    assert_eq!(s.directive("autre").unwrap().ligne, 6);
    assert!(s.directive("x_1").unwrap().args.is_empty());
    assert_eq!(s.directive("absent"), None);
    assert_eq!(s.directive("boite"), None);
    assert_eq!(s.directive("hexagone"), None);
    let s = lire("x_1 a\nhexagone 4\nx_1 b\n").unwrap();
    assert_eq!(s.forme, Forme::Hexagone { rayon: 4 });
    assert_eq!(s.directives, [directive(1, "x_1", &["a"]), directive(3, "x_1", &["b"])]);
    assert_eq!(s.directive("x_1").unwrap().args, ["a"]);
}

/// juste : un `#` en fin de ligne n'est pas un commentaire : `x 1 # note` est une directive à trois arguments `1`, `#`,
/// `note` (V-SCEN-10).
///
/// Ne couvre pas : les commentaires en début de ligne (autre test).
#[test]
#[cfg_attr(not(feature = "etape-0"), ignore)]
fn diese_en_fin_de_ligne() {
    let s = lire("boite 2 2\nx 1 # note\n").unwrap();
    assert_eq!(s.directives, [directive(2, "x", &["1", "#", "note"])]);
}

/// juste : `Directive::entier(i)` lit l'argument `i` en chiffres décimaux ASCII seulement : `12`, `0`, `u64::MAX`, `007` (7) ;
/// l'erreur est `LigneInvalide(ligne de la directive)` pour `u64::MAX + 1`, `+3`, `-1`, `abc`, `1.5`, et pour un argument
/// absent (V-SCEN-11).
///
/// Ne couvre pas : les arguments non UTF-8 (un texte Rust est UTF-8).
#[test]
#[cfg_attr(not(feature = "etape-0"), ignore)]
fn directive_entier() {
    let s = lire("boite 1 1\n\nx 12 0 18446744073709551615 18446744073709551616 +3 -1 abc 1.5 007\n").unwrap();
    let d = s.directive("x").unwrap();
    assert_eq!(d.ligne, 3);
    assert_eq!(d.entier(0), Ok(12));
    assert_eq!(d.entier(1), Ok(0));
    assert_eq!(d.entier(2), Ok(u64::MAX));
    assert_eq!(d.entier(8), Ok(7));
    for i in [3, 4, 5, 6, 7, 9, 10, 1000] {
        assert_eq!(d.entier(i), Err(ErreurLecture::LigneInvalide(3)), "argument {i}");
    }
}

/// juste : `Directive::flottant(i)` lit `-?chiffres(.chiffres)?` : `0.5`, `-2`, `0.1` (bits `0x3FB999999999999A`), `-0.0` (zéro
/// négatif), `007.25`, `123456789.123456789` (bits `0x419D6F34547E6B75`), `-3.14159265358979323846` (bits `0xC00921FB54442D18`) ;
/// l'erreur est `LigneInvalide(ligne)` pour `1e3`, `+1`, `.5`, `5.`, `inf`, `nan`, `1.2.3`, `-`, `--1`, `12abc`, `1,5`, un nombre
/// de quatre cents chiffres (infini à l'arrondi), et un argument absent (V-SCEN-12 ; arrondis au plus proche, valeurs de
/// Python `float`).
///
/// Ne couvre pas : la notation scientifique, volontairement refusée.
#[test]
#[cfg_attr(not(feature = "etape-0"), ignore)]
fn directive_flottant() {
    let enormes = "9".repeat(400);
    let texte = format!(
        "boite 1 1\nf 0.5 -2 0.1 -0.0 007.25 123456789.123456789 -3.14159265358979323846 1e3 +1 .5 5. inf nan 1.2.3 - --1 12abc 1,5 {enormes}\n"
    );
    let s = lire(&texte).unwrap();
    let d = s.directive("f").unwrap();
    assert_eq!(d.ligne, 2);
    assert_eq!(d.flottant(0).unwrap().to_bits(), 0.5_f64.to_bits());
    assert_eq!(d.flottant(1).unwrap().to_bits(), (-2.0_f64).to_bits());
    assert_eq!(d.flottant(2).unwrap().to_bits(), 0x3FB9_9999_9999_999A);
    assert_eq!(d.flottant(3).unwrap().to_bits(), 0x8000_0000_0000_0000);
    assert_eq!(d.flottant(4).unwrap().to_bits(), 0x401D_0000_0000_0000);
    assert_eq!(d.flottant(5).unwrap().to_bits(), 0x419D_6F34_547E_6B75);
    assert_eq!(d.flottant(6).unwrap().to_bits(), 0xC009_21FB_5444_2D18);
    for i in 7..=18 {
        assert_eq!(d.flottant(i), Err(ErreurLecture::LigneInvalide(2)), "argument {i} : {:?}", d.args[i]);
    }
    assert_eq!(d.flottant(19), Err(ErreurLecture::LigneInvalide(2)), "argument absent");
}

/// juste : le fichier d'exemple `scenarios/vide.txt` se lit : une boîte de 64 sur 48, sans autre directive (V-SCEN-13, la
/// description du fichier dans le brief de l'étape).
///
/// Ne couvre pas : les autres fichiers de `scenarios/`.
#[test]
#[cfg_attr(not(feature = "etape-0"), ignore)]
fn scenario_d_exemple_du_depot() {
    let chemin = concat!(env!("CARGO_MANIFEST_DIR"), "/scenarios/vide.txt");
    let texte = std::fs::read_to_string(chemin).expect("scenarios/vide.txt");
    let s = lire(&texte).unwrap();
    assert_eq!(s.forme, Forme::Boite { largeur: 64, hauteur: 48 });
    assert!(s.directives.is_empty());
}

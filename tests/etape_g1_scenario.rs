//! Étape G1 : la directive `neige <pas> <rho> <beta> <alpha> <theta> <kappa> <mu> <gamma>`, `lancer` et le binaire `givre`. Valeurs attendues :
//! `references/etape-g1-valeurs.md`, identifiants `V-G1-SC-xx`.
//!
//! Le contrat est dans la documentation de `src/neige.rs`, section « La directive `neige` et la sortie de `lancer` ». Les empreintes et les mesures attendues viennent du code Python
//! indépendant (`g1_ref.py`, d'après la doc seule). Ces tests sont ignorés tant que `etape-g1` n'est pas dans `default`, et ils ÉCHOUENT tant que `src/lancer.rs` ne consomme pas la
//! directive (c'est la tâche de la vague du code) : ils appellent `lancer`, et seulement lui.

mod common;
mod etape_g1_commun;

use etape_g1_commun::valeurs::{SCENARIO_R6_EMPREINTE, SCENARIO_R6_MESURES, SCENARIO_R6_PAS7_MESURES};
use etape_g1_commun::*;
use givre_lab::image::ecrire_png;
use givre_lab::lancer::{ErreurLancer, Sortie, lancer};
use givre_lab::neige::Neige;
use givre_lab::scenario::lire_scenario;
use std::fs;
use std::path::Path;
use std::process::{Command, Output};

const REGLAGES: &str = "0.5 1.3 0.08 0.025 0.003 0.07 0.00005";

fn lancer_texte(texte: &str, graine: u64) -> Result<Sortie, ErreurLancer> {
    lancer(&lire_scenario(texte).expect("scénario valide"), graine)
}

fn refus(texte: &str) -> Option<usize> {
    match lancer_texte(texte, 1) {
        Err(ErreurLancer::DirectiveInconnue(ligne)) => Some(ligne),
        Ok(_) => None,
    }
}

/// juste : le scénario `hexagone 6` + `neige 45 0.5 1.3 0.08 0.025 0.003 0.07 0.00005` (V-G1-SC-01) : l'empreinte est celle de l'état final calculée par le code Python indépendant ; les mesures sont le
/// CSV de Python (`temps,rattachees,rayon,masse_cristal`, les temps 0, 2, 4, … 44 puis 45 : `intervalle = max(1, 45 / 20) = 2` et 45 n'est pas un multiple) ; l'empreinte est celle de
/// `Neige::empreinte()` après 45 pas ; l'aperçu est le PNG de `Neige::apercu()` (13 sur 13 pixels : `2 R + 1`), relu et identique octet pour octet ; la graine n'est pas utilisée (trois graines, mêmes sorties).
///
/// Ne couvre pas : les autres tailles (voir `mesures_et_temps`).
#[test]
#[cfg_attr(not(feature = "etape-g1"), ignore)]
fn sortie_d_un_scenario() {
    let texte = format!("# un petit cristal\nhexagone 6\nneige 45 {REGLAGES}\n");
    let sortie = lancer_texte(&texte, 1).expect("scénario accepté");
    assert_eq!(sortie.empreinte, SCENARIO_R6_EMPREINTE);
    assert_eq!(sortie.mesures, SCENARIO_R6_MESURES);
    let mut neige = Neige::essayer_new(6, p(FIG9_RHO_050)).expect("neige valide");
    neige.avancer(45);
    assert_eq!(sortie.empreinte, neige.empreinte());
    assert_eq!(sortie.apercu, ecrire_png(13, 13, &neige.apercu()));
    let png = common::relire_png(&sortie.apercu).expect("PNG valide");
    assert_eq!((png.largeur, png.hauteur), (13, 13));
    assert_eq!(png.rvb, neige.apercu());
    for graine in [0, 7, u64::MAX] {
        assert_eq!(lancer_texte(&texte, graine).expect("accepté"), sortie, "graine {graine}");
    }
}

/// La colonne `masse_cristal` écrite comme la doc : `ppm = (2_000_000 * brut + 2^32) / 2^33`, `{ppm / 10^6}.{ppm % 10^6 sur 6 chiffres}`.
fn six_decimales(brut: i128) -> String {
    let ppm = (2_000_000 * brut + (1_i128 << 32)) / (1_i128 << 33);
    format!("{}.{:06}", ppm / 1_000_000, ppm % 1_000_000)
}

/// juste : les instants et les lignes des mesures (V-G1-SC-02), pour `pas` = 1, 2, 7, 19, 20, 21, 39, 40, 41, 100 sur `hexagone 4` : l'en-tête `temps,rattachees,rayon,masse_cristal`, puis le
/// temps 0, chaque `t` multiple de `intervalle = max(1, pas / 20)` (division entière) de 1 à `pas`, puis `pas` s'il n'est pas un multiple (jamais deux fois la même ligne) ; chaque ligne est
/// `t,rattachees,rayon,masse` avec les valeurs d'une `Neige` avancée pas à pas, la masse écrite en six décimales exactes, et un seul `\n` à la fin de chaque ligne (la dernière comprise, pas de
/// `\r`). Et les valeurs de Python pour `pas = 7` (huit lignes).
///
/// Ne couvre pas : le contenu du cristal au-delà des trois colonnes.
#[test]
#[cfg_attr(not(feature = "etape-g1"), ignore)]
fn mesures_et_temps() {
    for pas in [1_u64, 2, 7, 19, 20, 21, 39, 40, 41, 100] {
        let sortie = lancer_texte(&format!("hexagone 4\nneige {pas} {REGLAGES}\n"), 3).expect("accepté");
        let intervalle = (pas / 20).max(1);
        let mut instants = vec![0];
        instants.extend((1..=pas).filter(|t| t % intervalle == 0));
        if pas % intervalle != 0 {
            instants.push(pas);
        }
        let mut neige = Neige::essayer_new(4, p(FIG9_RHO_050)).expect("neige valide");
        let mut attendu = String::from("temps,rattachees,rayon,masse_cristal\n");
        for &t in &instants {
            neige.avancer(t - neige.temps());
            attendu += &format!(
                "{t},{},{},{}\n",
                neige.cases_rattachees(),
                neige.rayon_cristal(),
                six_decimales(neige.masse_du_cristal())
            );
        }
        assert_eq!(sortie.mesures, attendu, "pas = {pas}");
        assert!(!sortie.mesures.contains('\r'));
        assert!(sortie.mesures.ends_with('\n') && !sortie.mesures.ends_with("\n\n"));
    }
    let sortie = lancer_texte(&format!("hexagone 6\nneige 7 {REGLAGES}\n"), 1).expect("accepté");
    assert_eq!(sortie.mesures, SCENARIO_R6_PAS7_MESURES);
}

/// juste : l'aperçu écrit à la main (V-G1-SC-03) : pour `hexagone 1` et `neige 1 0.5 2 0.25 0.125 0.25 0.5 0.5`, après un pas le centre est rattaché et chacune des six voisines a `d = 2^30` : l'image
/// de 3 × 3 pixels, avec `x = q + R + floor(r / 2)` et `y = r + R`, est : le centre `(235, 245, 255)` en `(1, 1)` ; les six voisines `(0, 31, 63)` (`niveau = (2^30 * 255) >> 32 = 63`, `/ 2 = 31`) en `(0, 0)`, `(1, 0)`,
/// `(0, 1)`, `(2, 1)`, `(0, 2)`, `(1, 2)` ; les deux coins qui ne sont l'image d'aucune case, `(2, 0)` et `(2, 2)`, `(0, 0, 0)`. À l'état de départ (`ρ = 1/2`) les voisines valent `(0, 63, 127)`. Le PNG est relu.
///
/// Ne couvre pas : un rayon plus grand (voir `sortie_d_un_scenario`).
#[test]
#[cfg_attr(not(feature = "etape-g1"), ignore)]
fn apercu_a_la_main() {
    let sortie = lancer_texte("hexagone 1\nneige 1 0.5 2 0.25 0.125 0.25 0.5 0.5\n", 1).expect("accepté");
    let png = common::relire_png(&sortie.apercu).expect("PNG valide");
    assert_eq!((png.largeur, png.hauteur), (3, 3));
    let (voisine, centre, noir) = ([0, 31, 63], [235, 245, 255], [0, 0, 0]);
    let attendu: Vec<u8> = [voisine, voisine, noir, voisine, centre, voisine, voisine, voisine, noir].concat();
    assert_eq!(png.rvb, attendu);
    // l'état de départ, par la bibliothèque
    let depart =
        Neige::essayer_new(1, parametres("0.5", "2", "0.25", "0.125", "0.25", "0.5", "0.5")).expect("neige valide");
    let (voisine, centre) = ([0, 63, 127], [235, 245, 255]);
    assert_eq!(depart.apercu(), [voisine, voisine, noir, voisine, centre, voisine, voisine, voisine, noir].concat());
}

/// juste : l'aperçu plafonne la vapeur (V-G1-SC-07), à la main : `R = 1`, aucune case rattachée, des vapeurs `0`, `1 - 2^-32`, `1`, `1 + 2^-32`, `2`, `1/2`, `3/4` posées dans l'ordre canonique
/// (`(0,-1) (1,-1) (-1,0) (0,0) (1,0) (-1,1) (0,1)`). `niveau = min(255, (d.brut * 255) >> 32)` vaut 0, 254, 255, 255, 255, 127, 191 : une vapeur de 1 ou plus donne `(0, 127, 255)`, jamais un
/// niveau qui reboucle (sans le `min`, `1 + 2^-32` donnerait encore 255 mais `2` donnerait `510 mod 256 = 254`) ; la couleur est `(0, niveau / 2, niveau)`. Une case de bord ou lointaine peut
/// dépasser 1 (la fonte rend `μ b` à la vapeur, et `b` va jusqu'à `β <= 8`).
///
/// Ne couvre pas : la couleur d'une case rattachée (voir `apercu_a_la_main`).
#[test]
#[cfg_attr(not(feature = "etape-g1"), ignore)]
fn apercu_plafonne_la_vapeur() {
    let vapeurs: [i128; 7] = [0, UN - 1, UN, UN + 1, 2 * UN, UN / 2, 3 * (UN / 4)];
    let liste: Vec<_> = vapeurs.iter().map(|&d| case_de([0, 0, 0, d])).collect();
    let neige = Neige::essayer_depuis_cases(1, parametres("0.5", "2", "0.25", "0.125", "0.25", "0.5", "0.5"), &liste)
        .expect("état valide");
    let noir = [0_u8, 0, 0];
    let attendu: Vec<u8> =
        [[0, 0, 0], [0, 127, 254], noir, [0, 127, 255], [0, 127, 255], [0, 127, 255], [0, 63, 127], [0, 95, 191], noir]
            .concat();
    assert_eq!(neige.apercu(), attendu);
}

/// juste : les refus (V-G1-SC-04), tous `ErreurLancer::DirectiveInconnue(ligne)` avec la ligne de la PREMIÈRE directive fautive dans l'ordre du fichier (lignes ignorées comptées) : un nombre d'arguments
/// de 7 ou de 9 ; un `pas` de 0, de 1000001, avec un signe, un point, un préfixe `0x` ; un réglage mal écrit (`1e-1`, `.5`, `0,5`) ou hors plage (`ρ = 1.5`, `β = 0.9`, `α = 1.2`, `θ = 9`, `κ`
/// ou `γ` négatif ou au-dessus de 1, `μ = 2`) ; une forme qui n'est pas un `hexagone` (la ligne de `neige`) ; une seconde directive (valide ou non, de n'importe quel nom : la ligne de la seconde) ;
/// une première directive qui n'est pas `neige` (sa ligne) ; deux fautes : la première. Les commentaires et lignes vides comptent dans les numéros. Le `pas` de 1000000, les bornes des
/// réglages (`ρ = 1`, `β = 1` et `8`, `θ = 8`) et un hexagone de rayon 1 sont acceptés (un refus qui ne distinguerait pas ces cas ne prouverait rien).
///
/// Ne couvre pas : les refus des autres lancers (étapes précédentes).
#[test]
#[cfg_attr(not(feature = "etape-g1"), ignore)]
fn refus_de_la_directive() {
    let bon = format!("neige 40 {REGLAGES}");
    let hexa = |reste: &str| format!("hexagone 6\n{reste}\n");
    // arguments en trop ou en moins
    assert_eq!(refus(&hexa("neige 40 0.5 1.3 0.08 0.025 0.003 0.07")), Some(2));
    assert_eq!(refus(&hexa("neige 40 0.5 1.3 0.08 0.025 0.003 0.07 0.00005 0")), Some(2));
    assert_eq!(refus(&hexa("neige")), Some(2));
    // le pas
    for pas in ["0", "1000001", "+5", "4.0", "0x10", "-3", "", "99999999999999999999999"] {
        let ligne = format!("neige {pas} {REGLAGES}");
        assert_eq!(refus(&hexa(&ligne)), Some(2), "pas = {pas:?}");
    }
    // les réglages : mal écrits, hors plage
    let base = ["0.5", "1.3", "0.08", "0.025", "0.003", "0.07", "0.00005"];
    let fautes: [(usize, &str); 18] = [
        (0, "1e-1"),
        (0, ".5"),
        (0, "0,5"),
        (0, "1.5"),
        (1, "0.9"),
        (1, "8.5"),
        (2, "1.2"),
        (3, "9"),
        (4, "-0.1"),
        (4, "1.0000001"),
        (5, "2"),
        (6, "-1"),
        (6, "1.5"),
        (0, "+0.5"),
        (3, "0.5."),
        (1, ""),
        (2, "9999999999999999999"),
        (5, "0x1"),
    ];
    for (i, valeur) in fautes {
        let mut r = base;
        r[i] = valeur;
        let ligne = format!("neige 40 {}", r.join(" "));
        assert_eq!(refus(&hexa(&ligne)), Some(2), "réglage {i} = {valeur:?}");
    }
    // la forme
    assert_eq!(refus(&format!("boite 10 10\n{bon}\n")), Some(2));
    // les lignes ignorées comptent
    assert_eq!(refus("# un commentaire\n\nhexagone 6\n# autre\nneige 40 0.5 1.3 0.08 0.025 0.003 0.07\n"), Some(5));
    // une seconde directive, quelle qu'elle soit
    assert_eq!(refus(&format!("hexagone 6\n{bon}\n{bon}\n")), Some(3));
    assert_eq!(refus(&format!("hexagone 6\n{bon}\nfoo 1\n")), Some(3));
    assert_eq!(refus(&format!("hexagone 6\n{bon}\nreiter 1 0.4 0.1 5\n")), Some(3));
    assert_eq!(refus(&format!("hexagone 6\n{bon}\n\n# c\nneige 1 0.5 1.3 0.08 0.025 0.003 0.07 0.00005\n")), Some(5));
    // deux fautes : la première
    assert_eq!(refus(&format!("hexagone 6\nneige 0 {REGLAGES}\nfoo 1\n")), Some(2));
    // une première directive qui n'est pas neige
    assert_eq!(refus(&format!("hexagone 6\nfoo 1\n{bon}\n")), Some(2));
    // les cas acceptés voisins
    for texte in [
        format!("hexagone 1\nneige 1000000 {REGLAGES}\n"),
        "hexagone 2\nneige 3 1 1 0 8 1 1 1\n".to_string(),
        "hexagone 2\nneige 3 0 8 1 0 0 0 0\n".to_string(),
        "hexagone 1\nneige 5 1 1 0.5 0.5 0.5 0.5 0.5\n".to_string(),
        format!("hexagone 6\n{bon}\n"),
    ] {
        assert!(refus(&texte).is_none(), "{texte:?} doit être accepté");
    }
}

/// juste : les autres lancers sont inchangés (V-G1-SC-05) : un scénario sans directive, une `reiter` et un nom inconnu se comportent comme avant (le scénario vide est accepté, `reiter 1 0.4 0.1 5` est accepté
/// sur un hexagone, `foo` en première directive est refusé à sa ligne), et une directive `dla` quelque part garde la DLA (`hexagone 5` + `neige` + `dla 10` : refus à la ligne de `neige`, qui n'est pas une `dla`).
///
/// Ne couvre pas : le détail de ces lancers (leurs étapes).
#[test]
#[cfg_attr(not(feature = "etape-g1"), ignore)]
fn les_autres_lancers_restent_les_memes() {
    assert!(refus("hexagone 4\n").is_none());
    assert!(refus("hexagone 5\nreiter 1 0.4 0.1 5\n").is_none());
    assert_eq!(refus("hexagone 5\nfoo 1\n"), Some(2));
    assert_eq!(refus(&format!("boite 12 12\nneige 5 {REGLAGES}\ndla 10\n")), Some(2));
    assert_eq!(refus(&format!("boite 12 12\ndla 10\nneige 5 {REGLAGES}\n")), Some(3));
}

// ------------------------------------------------------------------------------------------------------------- le binaire

fn givre(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_givre")).args(args).output().expect("lancement de givre")
}

fn s(chemin: &Path) -> &str {
    chemin.to_str().expect("chemin en UTF-8")
}

/// juste : `givre lancer <scénario> --graine 5 --sortie <dossier>` pour un scénario `neige` (V-G1-SC-06) réussit (code 0) et écrit exactement `empreinte.txt` (l'empreinte de Python puis `\n`),
/// `mesures.csv` (celles de Python) et `apercu.png`, et rien d'autre ; ces fichiers sont ceux de `lancer` octet pour octet. Un scénario refusé (`neige 0 …` à la ligne 2) fait échouer le
/// binaire : code 1, un message qui contient `ligne 2` et pas `panicked`, et RIEN d'écrit (le dossier de sortie n'est pas créé).
///
/// Ne couvre pas : les options du binaire (étape 0).
#[test]
#[cfg_attr(not(feature = "etape-g1"), ignore)]
fn binaire_neige() {
    let espace = common::dossier_temporaire("neige-binaire");
    fs::create_dir_all(&espace).unwrap();
    let scenario = espace.join("scenario.txt");
    fs::write(&scenario, format!("hexagone 6\nneige 45 {REGLAGES}\n")).unwrap();
    let sortie = espace.join("sortie");
    let resultat = givre(&["lancer", s(&scenario), "--graine", "5", "--sortie", s(&sortie)]);
    assert_eq!(resultat.status.code(), Some(0), "stderr : {}", String::from_utf8_lossy(&resultat.stderr));
    let mut noms: Vec<String> =
        fs::read_dir(&sortie).unwrap().map(|e| e.unwrap().file_name().into_string().unwrap()).collect();
    noms.sort();
    assert_eq!(noms, ["apercu.png", "empreinte.txt", "mesures.csv"]);
    assert_eq!(fs::read_to_string(sortie.join("empreinte.txt")).unwrap(), format!("{SCENARIO_R6_EMPREINTE}\n"));
    assert_eq!(fs::read_to_string(sortie.join("mesures.csv")).unwrap(), SCENARIO_R6_MESURES);
    let attendu = lancer_texte(&fs::read_to_string(&scenario).unwrap(), 5).unwrap();
    assert_eq!(fs::read(sortie.join("apercu.png")).unwrap(), attendu.apercu);

    let refuse = espace.join("refuse.txt");
    fs::write(&refuse, format!("hexagone 6\nneige 0 {REGLAGES}\n")).unwrap();
    let dossier = espace.join("rien");
    let resultat = givre(&["lancer", s(&refuse), "--graine", "5", "--sortie", s(&dossier)]);
    let message = String::from_utf8_lossy(&resultat.stderr);
    assert_eq!(resultat.status.code(), Some(1), "{message}");
    assert!(message.contains("ligne 2") && !message.contains("panicked"), "{message}");
    assert!(!dossier.exists(), "rien ne doit être écrit");
    let _ = fs::remove_dir_all(&espace);
}

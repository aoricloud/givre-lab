//! Étape 3 : la directive `reiter <alpha> <beta> <gamma> <pas>` lancée par `lancer` et par le binaire `givre` : l'empreinte, le CSV des
//! mesures, l'aperçu PNG, les refus, les scénarios d'exemple. Valeurs attendues : `references/etape-3-valeurs.md`, identifiants
//! `V-LANC3-xx`. Les CSV, empreintes et hachages d'aperçu viennent de Python (`modele_independant.py`) : aucun ne vient de notre code, sauf le
//! fichier de référence `references/reiter-plaque.empreinte` (test `regression_...`).

mod common;
mod etape_3_commun;

use etape_3_commun::valeurs::*;
use givre_lab::empreinte::sha256_hex;
use givre_lab::lancer::{ErreurLancer, Sortie, lancer};
use givre_lab::scenario::lire_scenario;
use std::fs;
use std::process::Command;

fn lancer_texte(texte: &str, graine: u64) -> Result<Sortie, ErreurLancer> {
    lancer(&lire_scenario(texte).expect("scénario valide"), graine)
}

fn lire_exemple(nom: &str) -> String {
    fs::read_to_string(format!("{}/scenarios/{nom}", env!("CARGO_MANIFEST_DIR")))
        .unwrap_or_else(|e| panic!("scenarios/{nom} : {e}"))
}

/// juste : quatre scénarios `hexagone R` + `reiter α β γ pas`, dont les sorties sont celles de Python d'après la doc (V-LANC3-01) : le CSV
/// `temps,gelees,rayon,compacite` (une ligne au temps 0, puis tous les `pas / 20` pas, puis le temps final : `pas = 7` donne une ligne par
/// pas, `pas = 1` deux lignes, `pas = 45` (intervalle 2) donne 0, 2, ..., 44 puis 45, `pas = 100` (intervalle 5) donne 21 lignes, la dernière
/// n'étant pas répétée), l'empreinte de l'état final, et l'aperçu PNG relu par le décodeur du test : côté `2R + 1`, octets RVB de SHA-256 celui
/// de Python. La compacité est écrite en six décimales exactes (`0.684211`, `1.000000`).
///
/// Ne couvre pas : les refus (voir `refus`).
#[test]
#[cfg_attr(not(feature = "etape-3"), ignore)]
fn sorties_des_scenarios_petits() {
    let cas = [
        ("hexagone 6\nreiter 1 0.8 0.05 7\n", 6, S_PETIT_CSV, S_PETIT_EMPREINTE, S_PETIT_APERCU_SHA256),
        ("hexagone 5\nreiter 1 0.9 0 1\n", 5, S_UN_PAS_CSV, S_UN_PAS_EMPREINTE, S_UN_PAS_APERCU_SHA256),
        ("hexagone 30\nreiter 1 0.8 0.002 45\n", 30, S_SECTEUR_CSV, S_SECTEUR_EMPREINTE, S_SECTEUR_APERCU_SHA256),
        (
            "hexagone 30\nreiter 1 0 0.25 100\n",
            30,
            S_PLAQUE_COURTE_CSV,
            S_PLAQUE_COURTE_EMPREINTE,
            S_PLAQUE_COURTE_APERCU_SHA256,
        ),
    ];
    for (texte, rayon, csv, empreinte, apercu_sha) in cas {
        let sortie = lancer_texte(texte, 1).expect("lancer");
        assert_eq!(sortie.mesures, csv, "{texte}");
        assert!(!sortie.mesures.contains('\r') && sortie.mesures.ends_with('\n') && !sortie.mesures.ends_with("\n\n"));
        assert_eq!(sortie.empreinte, empreinte, "{texte}");
        let png = common::relire_png(&sortie.apercu).expect("PNG valide");
        assert_eq!((png.largeur, png.hauteur), (2 * rayon + 1, 2 * rayon + 1), "{texte}");
        assert_eq!(sha256_hex(&png.rvb), apercu_sha, "{texte}");
    }
}

/// juste : le scénario n'a aucun hasard (V-LANC3-02) : quatre graines (0, 1, un grand nombre, `u64::MAX`) donnent la même `Sortie`, octet pour
/// octet (empreinte, CSV et PNG) ; et la même empreinte que celle de Python. Un code qui mélange la graine à l'état échoue ici.
///
/// Ne couvre pas : la reproductibilité entre machines.
#[test]
#[cfg_attr(not(feature = "etape-3"), ignore)]
fn graine_sans_effet() {
    let texte = "hexagone 6\nreiter 1 0.8 0.05 7\n";
    let reference = lancer_texte(texte, 0).unwrap();
    assert_eq!(reference.empreinte, S_PETIT_EMPREINTE);
    for graine in [1, 123_456_789, u64::MAX] {
        assert_eq!(lancer_texte(texte, graine).unwrap(), reference, "graine {graine}");
    }
}

/// juste : les refus (V-LANC3-03), `ErreurLancer::DirectiveInconnue(ligne)` avec la ligne de la **première directive fautive dans l'ordre du
/// fichier**, lignes ignorées (commentaires, vides) comptées : une seconde directive `reiter` (3), trois ou cinq arguments (2), `α > 2`, `β = 1`,
/// `γ > 1`, un `pas` de 0, de 1000001, signé (`+5`, `-5`), à exposant (`1e2`), à point (`5.0`), trop grand pour un `u64`, un décimal mal écrit
/// (`.5`, `0,4`, `-0.4`), un commentaire de fin de ligne (un septième argument), une forme `boite`, une autre directive après la bonne (3), une
/// autre directive avant (2), la faute à la ligne 4 après un commentaire et une ligne vide. Et acceptés : `reiter` avant `hexagone`, un `pas` de
/// `010` (zéros de tête), `α = 1.00` (même état que `1`), `2.0` et `02` (même état que `2`).
///
/// Ne couvre pas : l'erreur de lecture du scénario lui-même (étape 0).
#[test]
#[cfg_attr(not(feature = "etape-3"), ignore)]
fn refus() {
    let refuses: [(&str, usize); 26] = [
        ("hexagone 5\nreiter 1 0.4 0.001 10\nreiter 1 0.4 0.001 10\n", 3),
        ("hexagone 5\nreiter 1 0.4 0.001\n", 2),
        ("hexagone 5\nreiter 1 0.4 0.001 10 7\n", 2),
        ("hexagone 5\nreiter 3 0.4 0.001 10\n", 2),
        ("hexagone 5\nreiter 1 1 0.001 10\n", 2),
        ("hexagone 5\nreiter 1 0.4 1.5 10\n", 2),
        ("hexagone 5\nreiter 1 0.4 0.001 0\n", 2),
        ("hexagone 5\nreiter 1 0.4 0.001 1000001\n", 2),
        ("hexagone 5\nreiter 1 0.4 0.001 +5\n", 2),
        ("hexagone 5\nreiter 1 0.4 0.001 -5\n", 2),
        ("hexagone 5\nreiter 1 0.4 0.001 1e2\n", 2),
        ("hexagone 5\nreiter 1 0.4 0.001 5.0\n", 2),
        ("hexagone 5\nreiter 1 0.4 0.001 99999999999999999999\n", 2),
        ("hexagone 5\nreiter .5 0.4 0.001 10\n", 2),
        ("hexagone 5\nreiter 1 0,4 0.001 10\n", 2),
        ("hexagone 5\nreiter 1 -0.4 0.001 10\n", 2),
        ("hexagone 5\nreiter 1 0.4 0.001 10 # note\n", 2),
        ("boite 5 5\nreiter 1 0.4 0.001 10\n", 2),
        ("hexagone 5\nreiter 1 0.4 0.001 10\nautre 1\n", 3),
        ("hexagone 5\nautre 1\nreiter 1 0.4 0.001 10\n", 2),
        ("# commentaire\nhexagone 5\n\nreiter 1 0.4 0.001 x\n", 4),
        ("hexagone 5\nreiter\n", 2),
        ("hexagone 5\nreiter 1 0.4 0.001 10\n\n# fin\nreiter 1 0.4 0.001 10\n", 5),
        ("hexagone 5\nreiter 1 0.4 0.001 10 7 8\n", 2),
        ("hexagone 5\nreiter1 1 0.4 0.001 10\n", 2),
        ("hexagone 5\nreiter 1 0.4 0.001 10\nreiter 3 0.4 0.001 10\n", 3),
    ];
    for (texte, ligne) in refuses {
        assert_eq!(lancer_texte(texte, 1).map(|_| ()), Err(ErreurLancer::DirectiveInconnue(ligne)), "{texte:?}");
    }
    let attendu = lancer_texte("hexagone 5\nreiter 1 0.4 0.001 10\n", 1).unwrap();
    for texte in [
        "reiter 1 0.4 0.001 10\nhexagone 5\n",
        "hexagone 5\nreiter 1 0.4 0.001 010\n",
        "hexagone 5\nreiter 1.00 0.4 0.001 10\n",
    ] {
        let sortie = lancer_texte(texte, 1).unwrap_or_else(|e| panic!("{texte:?} : {e:?}"));
        assert_eq!(sortie.mesures, attendu.mesures, "{texte:?}");
        assert_eq!(sortie.empreinte.len(), 64);
    }
    let deux = lancer_texte("hexagone 5\nreiter 2 0.4 0.001 10\n", 1).unwrap();
    for texte in ["hexagone 5\nreiter 2.0 0.4 0.001 10\n", "hexagone 5\nreiter 02 0.4 0.001 10\n"] {
        assert_eq!(lancer_texte(texte, 1).unwrap(), deux, "{texte:?}");
    }
    assert_ne!(deux.empreinte, attendu.empreinte);
    // les autres directives de l'étape 0 restent inconnues
    assert_eq!(lancer_texte("hexagone 3\ndla 3\n", 1).map(|_| ()), Err(ErreurLancer::DirectiveInconnue(2)));
}

/// juste : le nombre de pas maximal 1 000 000 est accepté et 1 000 001 refusé (V-LANC3-04). Sur `hexagone 1` (7 cases : seul le centre bouge,
/// la couronne est remise à `β` à chaque pas), `reiter 1 0 1 1000000` : le CSV a 21 lignes de mesure (intervalle 50 000), chacune au temps
/// `50 000 k` ; la case centrale gelée reste la seule (1 case gelée, rayon 0, compacité `1.000000`) ; l'empreinte a 64 chiffres.
///
/// Ne couvre pas : les durées intermédiaires.
#[test]
#[cfg_attr(not(feature = "etape-3"), ignore)]
fn pas_maximal() {
    let sortie = lancer_texte("hexagone 1\nreiter 1 0 1 1000000\n", 1).expect("pas maximal accepté");
    let mut attendu = String::from("temps,gelees,rayon,compacite\n");
    for k in 0..=20 {
        attendu.push_str(&format!("{},1,0,1.000000\n", 50_000 * k));
    }
    assert_eq!(sortie.mesures, attendu);
    assert_eq!(sortie.empreinte.len(), 64);
    assert_eq!(
        lancer_texte("hexagone 1\nreiter 1 0 1 1000001\n", 1).map(|_| ()),
        Err(ErreurLancer::DirectiveInconnue(2))
    );
}

/// juste : les trois scénarios d'exemple du dépôt (`scenarios/reiter-plaque.txt`, `reiter-etoile.txt`, `reiter-dendrite.txt`) donnent le CSV et
/// l'empreinte de Python (V-LANC3-05) : une plaque (`hexagone 30`, `reiter 1 0 0.05 400` : un anneau tous les 20 pas, compacité 1 à chaque ligne),
/// une étoile (`hexagone 40`, `reiter 1 0.45 0.1 120`) et une dendrite (`hexagone 60`, `reiter 1 0.4 0.001 800`).
///
/// Ne couvre pas : la reproductibilité entre machines (c'est la CI qui rejoue la référence).
#[test]
#[cfg_attr(not(feature = "etape-3"), ignore)]
fn scenarios_d_exemple() {
    for (nom, csv, empreinte) in [
        ("reiter-plaque.txt", EX_PLAQUE_CSV, EX_PLAQUE_EMPREINTE),
        ("reiter-etoile.txt", EX_ETOILE_CSV, EX_ETOILE_EMPREINTE),
        ("reiter-dendrite.txt", EX_DENDRITE_CSV, EX_DENDRITE_EMPREINTE),
    ] {
        let sortie = lancer_texte(&lire_exemple(nom), 1).expect("lancer");
        assert_eq!(sortie.mesures, csv, "{nom}");
        assert_eq!(sortie.empreinte, empreinte, "{nom}");
    }
}

/// régression : l'empreinte du scénario `scenarios/reiter-plaque.txt` est celle du fichier `references/reiter-plaque.empreinte` (l'empreinte et un
/// saut de ligne), produite par le binaire sur ce scénario ; ce fichier est ce que la CI rejoue sous Linux et sous Windows. Il n'existe pas dans le
/// jeu de tests : c'est la vague du code qui le produit (`givre lancer scenarios/reiter-plaque.txt --graine 1 --sortie <dossier>`, copie de
/// `empreinte.txt`). La graine est sans effet.
///
/// Ne couvre pas : que l'empreinte soit juste (`scenarios_d_exemple` le prouve : le SHA-256 est celui de Python). Ce test dit seulement que le
/// fichier de référence n'a pas bougé.
#[test]
#[cfg_attr(not(feature = "etape-3"), ignore)]
fn regression_reference_reiter_plaque() {
    let racine = env!("CARGO_MANIFEST_DIR");
    let reference = fs::read_to_string(format!("{racine}/references/reiter-plaque.empreinte"))
        .expect("references/reiter-plaque.empreinte est absente : la vague du code la produit avec le binaire");
    let sortie = lancer_texte(&lire_exemple("reiter-plaque.txt"), 1).unwrap();
    assert_eq!(reference, format!("{}\n", sortie.empreinte));
    assert_eq!(reference, format!("{EX_PLAQUE_EMPREINTE}\n"));
}

/// juste : le binaire `givre lancer scenarios/reiter-plaque.txt --graine 3 --sortie <dossier>` écrit `empreinte.txt` (l'empreinte de Python puis
/// `\n`), `mesures.csv` (le CSV de Python) et `apercu.png` (un PNG de 61 × 61 pixels relu par le décodeur du test), code de sortie 0 (V-LANC3-06) ; un
/// scénario `reiter` refusé donne le code 1, un message avec `ligne 2` sur la sortie d'erreur, et aucun dossier créé.
///
/// Ne couvre pas : les options de la ligne de commande (étape 0).
#[test]
#[cfg_attr(not(feature = "etape-3"), ignore)]
fn binaire() {
    let dossier = common::dossier_temporaire("reiter-binaire");
    fs::create_dir_all(&dossier).unwrap();
    let scenario = format!("{}/scenarios/reiter-plaque.txt", env!("CARGO_MANIFEST_DIR"));
    let sortie = dossier.join("sortie");
    let resultat = Command::new(env!("CARGO_BIN_EXE_givre"))
        .args(["lancer", &scenario, "--graine", "3", "--sortie", sortie.to_str().unwrap()])
        .output()
        .expect("lancement de givre");
    assert_eq!(resultat.status.code(), Some(0), "stderr : {}", String::from_utf8_lossy(&resultat.stderr));
    assert_eq!(fs::read_to_string(sortie.join("empreinte.txt")).unwrap(), format!("{EX_PLAQUE_EMPREINTE}\n"));
    assert_eq!(fs::read_to_string(sortie.join("mesures.csv")).unwrap(), EX_PLAQUE_CSV);
    let png = common::relire_png(&fs::read(sortie.join("apercu.png")).unwrap()).expect("PNG valide");
    assert_eq!((png.largeur, png.hauteur), (61, 61));

    let mauvais = dossier.join("mauvais.txt");
    fs::write(&mauvais, "hexagone 5\nreiter 1 1 0.001 10\n").unwrap();
    let refus_dossier = dossier.join("refus");
    let refus = Command::new(env!("CARGO_BIN_EXE_givre"))
        .args(["lancer", mauvais.to_str().unwrap(), "--graine", "3", "--sortie", refus_dossier.to_str().unwrap()])
        .output()
        .expect("lancement de givre");
    assert_eq!(refus.status.code(), Some(1));
    let message = String::from_utf8_lossy(&refus.stderr);
    assert!(message.contains("ligne 2") && !message.contains("panicked"), "message : {message}");
    assert!(!refus_dossier.exists(), "aucun dossier créé");
    let _ = fs::remove_dir_all(&dossier);
}

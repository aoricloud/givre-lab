//! Le binaire `givre` : `givre lancer <scénario> --graine <n> --sortie <dossier>` (étape 0).
//!
//! - `<scénario>` : le chemin d'un fichier de scénario (voir `givre_lab::scenario`). `--graine <n>` : un entier non signé
//!   de 64 bits écrit en chiffres décimaux seulement (sans signe). `--sortie <dossier>` : le dossier des sorties, créé
//!   (avec ses parents) s'il manque. Les deux options figurent **chacune une fois**, dans n'importe quel ordre, et le
//!   chemin du scénario est le seul argument qui ne soit pas une option ni la valeur d'une option.
//! - Il lit le fichier, appelle `givre_lab::lancer::lancer` et écrit dans le dossier trois fichiers, et aucun autre :
//!   `empreinte.txt` (l'empreinte, 64 chiffres hexadécimaux, puis un `\n`), `mesures.csv` (les mesures, telles que
//!   rendues) et `apercu.png` (l'aperçu, tel que rendu). Un dossier qui contient déjà ces fichiers les voit remplacés.
//! - Il ne rend rien d'autre que ces fichiers (rien d'imposé sur la sortie standard).
//! - **Toute erreur** (aucune commande ou une commande autre que `lancer` ; une option inconnue, répétée, sans valeur ou
//!   absente ; une graine mal écrite ou qui dépasse 64 bits ; un scénario en trop ; un fichier illisible ; un scénario
//!   invalide ou refusé par `lancer` ; un dossier impossible à créer) : un message d'une ligne au moins sur la sortie
//!   d'erreur (sans `panicked`, car ce n'est pas un plantage), le **code de sortie 1**, et **rien d'écrit ni de créé**
//!   (le dossier de sortie n'est pas créé quand le scénario est refusé). Le message d'une erreur de scénario contient
//!   `ligne <n>` (le numéro décimal de la ligne fautive, par exemple `ligne 7`, ou, pour une directive que `lancer` refuse,
//!   le numéro de la ligne de cette directive) ; celui d'une forme manquante contient le mot `forme`.
//! - Code de sortie 0 quand tout est écrit.

use std::ffi::{OsStr, OsString};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use givre_lab::lancer::{ErreurLancer, Sortie, lancer};
use givre_lab::scenario::{ErreurLecture, lire_scenario};

fn main() -> std::process::ExitCode {
    match executer(std::env::args_os().skip(1).collect()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("givre : {message}");
            ExitCode::from(1)
        }
    }
}

/// Lit les arguments (sans le nom du programme), lance le scénario et écrit ses trois sorties.
fn executer(args: Vec<OsString>) -> Result<(), String> {
    let Some(commande) = args.first() else {
        return Err("aucune commande : attendu `lancer`".to_string());
    };
    if commande != OsStr::new("lancer") {
        return Err(format!("commande inconnue : {}", commande.to_string_lossy()));
    }

    let mut graine: Option<u64> = None;
    let mut dossier: Option<PathBuf> = None;
    let mut chemin: Option<PathBuf> = None;
    let mut i = 1;
    while i < args.len() {
        let arg = &args[i];
        if arg == OsStr::new("--graine") {
            if graine.is_some() {
                return Err("option --graine répétée".to_string());
            }
            let valeur = args.get(i + 1).ok_or_else(|| "option --graine sans valeur".to_string())?;
            graine = Some(lire_graine(valeur)?);
            i += 2;
        } else if arg == OsStr::new("--sortie") {
            if dossier.is_some() {
                return Err("option --sortie répétée".to_string());
            }
            let valeur = args.get(i + 1).ok_or_else(|| "option --sortie sans valeur".to_string())?;
            dossier = Some(PathBuf::from(valeur.clone()));
            i += 2;
        } else if arg.to_str().is_some_and(|texte| texte.starts_with("--")) {
            return Err(format!("option inconnue : {}", arg.to_string_lossy()));
        } else {
            if chemin.is_some() {
                return Err("un second scénario a été donné".to_string());
            }
            chemin = Some(PathBuf::from(arg.clone()));
            i += 1;
        }
    }

    let graine = graine.ok_or_else(|| "option --graine absente".to_string())?;
    let dossier = dossier.ok_or_else(|| "option --sortie absente".to_string())?;
    let chemin = chemin.ok_or_else(|| "scénario absent".to_string())?;

    let texte = fs::read_to_string(&chemin)
        .map_err(|erreur| format!("fichier de scénario illisible ({}) : {erreur}", chemin.display()))?;
    let scenario = lire_scenario(&texte).map_err(message_lecture)?;
    let sortie = lancer(&scenario, graine).map_err(message_lancer)?;
    ecrire_sorties(&dossier, &sortie)
}

/// La graine : des chiffres décimaux ASCII seulement, et une valeur qui tient dans un `u64`.
fn lire_graine(arg: &OsStr) -> Result<u64, String> {
    let texte = arg.to_str().ok_or_else(|| "graine invalide : chiffres décimaux seulement".to_string())?;
    if texte.is_empty() || !texte.bytes().all(|octet| octet.is_ascii_digit()) {
        return Err(format!("graine invalide : {texte:?} (chiffres décimaux seulement)"));
    }
    texte.parse::<u64>().map_err(|_| format!("graine hors bornes pour un u64 : {texte}"))
}

/// Le message d'une erreur de lecture de scénario : `ligne <n>` pour une ligne fautive, `forme` pour une forme manquante.
fn message_lecture(erreur: ErreurLecture) -> String {
    match erreur {
        ErreurLecture::LigneInvalide(ligne) => format!("scénario invalide : ligne {ligne}"),
        ErreurLecture::Manquante(nom) => format!("directive obligatoire manquante : {nom}"),
    }
}

/// Le message d'une erreur de `lancer` : `ligne <n>` pour la directive refusée.
fn message_lancer(erreur: ErreurLancer) -> String {
    match erreur {
        ErreurLancer::DirectiveInconnue(ligne) => format!("directive inconnue : ligne {ligne}"),
    }
}

/// Crée le dossier (et ses parents) puis y écrit exactement les trois sorties ; en cas d'échec, retire ce qu'il a écrit et
/// créé.
fn ecrire_sorties(dossier: &Path, sortie: &Sortie) -> Result<(), String> {
    let empreinte = format!("{}\n", sortie.empreinte);

    // Les dossiers qui n'existaient pas avant la création, du plus profond au plus haut : à retirer si l'écriture échoue.
    let mut crees: Vec<PathBuf> = Vec::new();
    let mut courant = Some(dossier);
    while let Some(chemin) = courant {
        if chemin.exists() {
            break;
        }
        crees.push(chemin.to_path_buf());
        courant = chemin.parent();
    }

    if let Err(erreur) = fs::create_dir_all(dossier) {
        for chemin in &crees {
            let _ = fs::remove_dir(chemin);
        }
        return Err(format!("dossier de sortie impossible à créer ({}) : {erreur}", dossier.display()));
    }

    let fichiers: [(&str, &[u8]); 3] = [
        ("empreinte.txt", empreinte.as_bytes()),
        ("mesures.csv", sortie.mesures.as_bytes()),
        ("apercu.png", &sortie.apercu),
    ];
    let mut ecrits: Vec<&str> = Vec::new();
    for (nom, contenu) in fichiers {
        if let Err(erreur) = fs::write(dossier.join(nom), contenu) {
            for ecrit in &ecrits {
                let _ = fs::remove_file(dossier.join(ecrit));
            }
            for chemin in &crees {
                let _ = fs::remove_dir(chemin);
            }
            return Err(format!("écriture de `{nom}` impossible ({}) : {erreur}", dossier.display()));
        }
        ecrits.push(nom);
    }
    Ok(())
}

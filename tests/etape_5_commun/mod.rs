//! Aides communes aux tests de l'étape 5 (la vitrine) : la table des quatre modèles de la doc de `vitrine`, l'état **natif** en
//! face (un `Amas` ou un `Flocon` construit directement par la bibliothèque, jamais par `vitrine`), le décodage des mots de
//! l'empreinte, un petit lecteur du format binaire WebAssembly, et le lancement de Node.
//!
//! Ce fichier n'est pas un test : il ne contient aucune fonction `#[test]`. Il est gelé avec les tests.
#![allow(dead_code)]

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use givre_lab::dla::{Amas, COTE_APERCU, amas_colore};
use givre_lab::flocon::{Flocon, Parametres};

// ---------------------------------------------------------------------------------------------------------------
// La table de la doc de `vitrine`.
// ---------------------------------------------------------------------------------------------------------------

/// Un modèle de la table « Les quatre modèles » de la doc de `vitrine`.
pub struct Modele {
    /// Le numéro passé à `givre_demarrer`.
    pub numero: u32,
    /// Le nom du fichier de scénario d'exemple (sous `scenarios/`).
    pub scenario: &'static str,
    /// La durée du scénario : `givre_duree()`.
    pub duree: u32,
    /// Le rayon de l'hexagone d'un flocon (0 pour la DLA).
    pub rayon: u32,
    /// Les trois paramètres d'un flocon, écrits comme dans le scénario (vides pour la DLA).
    pub alpha: &'static str,
    pub beta: &'static str,
    pub gamma: &'static str,
}

/// Les quatre modèles : la DLA, la plaque, l'étoile, la dendrite. Ce sont **les valeurs de la doc**, écrites ici à la main (et
/// comparées aux fichiers de scénario par `etape_5_vitrine.rs`).
pub const TABLE: [Modele; 4] = [
    Modele { numero: 0, scenario: "dla.txt", duree: 2999, rayon: 0, alpha: "", beta: "", gamma: "" },
    Modele { numero: 1, scenario: "reiter-plaque.txt", duree: 400, rayon: 30, alpha: "1", beta: "0", gamma: "0.05" },
    Modele { numero: 2, scenario: "reiter-etoile.txt", duree: 120, rayon: 40, alpha: "1", beta: "0.45", gamma: "0.1" },
    Modele {
        numero: 3,
        scenario: "reiter-dendrite.txt",
        duree: 800,
        rayon: 60,
        alpha: "1",
        beta: "0.4",
        gamma: "0.001",
    },
];

/// Le `demi_cote` du réseau de la DLA de la vitrine (`boite 401 401`).
pub const DEMI_COTE_DLA: u32 = 200;

pub fn racine() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
}

/// Le contenu d'un fichier de `references/`, sans l'espace final.
pub fn reference(nom: &str) -> String {
    let chemin = racine().join("references").join(nom);
    std::fs::read_to_string(&chemin)
        .unwrap_or_else(|erreur| panic!("{} se lit : {erreur}", chemin.display()))
        .trim()
        .to_string()
}

// ---------------------------------------------------------------------------------------------------------------
// L'état natif en face.
// ---------------------------------------------------------------------------------------------------------------

/// Tout ce que la vitrine expose, tel que le code natif le donne pour le même modèle, la même graine et le même nombre
/// d'unités jouées.
pub struct Natif {
    pub temps: u32,
    pub duree: u32,
    pub largeur: u32,
    pub hauteur: u32,
    /// Les octets R, V, B de l'image (l'aperçu de la bibliothèque).
    pub rvb: Vec<u8>,
    pub sites: u32,
    pub rayon: u32,
    /// `None` pour la DLA (la vitrine rend `NaN`).
    pub compacite: Option<f64>,
    /// Les 64 caractères hexadécimaux de l'empreinte.
    pub empreinte: String,
}

/// L'état natif du modèle `modele` (0 à 3) après `min(pas, durée)` unités : la DLA par `Amas::new(graine, 200)` puis
/// `croitre_jusqu_a(1 + joué)`, un flocon par `Flocon::new(rayon, Parametres::lire(..))` puis `avancer(joué)`, avec les
/// réglages de la table de la doc.
pub fn natif(modele: u32, graine: u32, pas: u32) -> Natif {
    let reglage = &TABLE[modele as usize];
    let joues = pas.min(reglage.duree);
    if modele == 0 {
        let mut amas = Amas::new(u64::from(graine), DEMI_COTE_DLA);
        amas.croitre_jusqu_a(1 + joues as usize);
        let (largeur, hauteur, rvb) = amas_colore(amas.sites(), COTE_APERCU);
        Natif {
            temps: joues,
            duree: reglage.duree,
            largeur,
            hauteur,
            rvb,
            sites: amas.sites().len() as u32,
            rayon: amas.rayon(),
            compacite: None,
            empreinte: amas.empreinte(),
        }
    } else {
        let parametres = Parametres::lire(reglage.alpha, reglage.beta, reglage.gamma).expect("réglages valides");
        let mut flocon = Flocon::new(reglage.rayon, parametres);
        flocon.avancer(u64::from(joues));
        let cote = 2 * reglage.rayon + 1;
        Natif {
            temps: joues,
            duree: reglage.duree,
            largeur: cote,
            hauteur: cote,
            rvb: flocon.apercu(),
            sites: flocon.gelees() as u32,
            rayon: flocon.rayon_cristal(),
            compacite: Some(flocon.compacite().vers_f64()),
            empreinte: flocon.empreinte(),
        }
    }
}

impl Natif {
    /// Le pixel `(x, y)` attendu, empaqueté en `0x00RRGGBB`, ou `u32::MAX` hors de l'image.
    pub fn pixel(&self, x: u32, y: u32) -> u32 {
        if x >= self.largeur || y >= self.hauteur {
            return u32::MAX;
        }
        let debut = 3 * (y as usize * self.largeur as usize + x as usize);
        (u32::from(self.rvb[debut]) << 16) | (u32::from(self.rvb[debut + 1]) << 8) | u32::from(self.rvb[debut + 2])
    }

    /// Les huit mots de 32 bits de l'empreinte, gros-boutistes : les huit tranches de 8 chiffres hexadécimaux.
    pub fn mots(&self) -> [u32; 8] {
        let mut mots = [0_u32; 8];
        for (i, mot) in mots.iter_mut().enumerate() {
            *mot = u32::from_str_radix(&self.empreinte[8 * i..8 * i + 8], 16).expect("hexadécimal");
        }
        mots
    }
}

// ---------------------------------------------------------------------------------------------------------------
// Les fonctions du module : leurs signatures, et un petit lecteur du format binaire WebAssembly.
// ---------------------------------------------------------------------------------------------------------------

/// Les types de valeurs WebAssembly qui servent ici, tels que le format binaire les code.
pub const I32: u8 = 0x7F;
pub const F64: u8 = 0x7C;

/// Les fonctions que le module doit exporter, avec leur signature : types des paramètres, types des résultats (`u32` est un
/// `i32` en WebAssembly). Ce sont les onze fonctions de `src/vitrine.rs`.
pub const FONCTIONS: [(&str, &[u8], &[u8]); 11] = [
    ("givre_demarrer", &[I32, I32], &[I32]),
    ("givre_avancer", &[I32], &[I32]),
    ("givre_temps", &[], &[I32]),
    ("givre_duree", &[], &[I32]),
    ("givre_largeur", &[], &[I32]),
    ("givre_hauteur", &[], &[I32]),
    ("givre_pixel", &[I32, I32], &[I32]),
    ("givre_sites", &[], &[I32]),
    ("givre_rayon", &[], &[I32]),
    ("givre_compacite", &[], &[F64]),
    ("givre_empreinte_mot", &[I32], &[I32]),
];

/// Les quatre fonctions de mesure de l'étape Z1 (voir `src/vitrine.rs`, « La mesure de la corrélation »), avec leur signature. Le module et la
/// page de l'étape 5 ne les ont pas : tant que la fonctionnalité `etape-z1` n'est pas active, les tests acceptent le module et la page tels
/// qu'ils sont (les onze fonctions de `FONCTIONS`) ou étendus de ces quatre ; avec `etape-z1`, ils n'acceptent que l'étendu.
pub const FONCTIONS_Z1: [(&str, &[u8], &[u8]); 4] = [
    ("givre_mesurer", &[], &[I32]),
    ("givre_correlation_rayon", &[I32], &[I32]),
    ("givre_correlation_valeur", &[I32], &[F64]),
    ("givre_dimension", &[], &[F64]),
];

/// Vrai quand la fonctionnalité `etape-z1` est active : le module et la page doivent alors avoir les quinze fonctions.
pub const Z1_ACTIVE: bool = cfg!(feature = "etape-z1");

/// Les huit fonctions de la courbe `D(η)` de l'étape D3 (voir `src/vitrine.rs`, « La courbe D(η) à la demande »), avec leur signature. Le module et la
/// page d'avant D3 ne les ont pas : tant que la fonctionnalité `etape-d3` n'est pas active, les tests acceptent le module et la page tels qu'ils sont
/// (les onze de `FONCTIONS`, ou les quinze avec `FONCTIONS_Z1`) ou étendus de ces huit (vingt-trois) ; avec `etape-d3`, ils n'acceptent que l'étendu.
pub const FONCTIONS_D3: [(&str, &[u8], &[u8]); 8] = [
    ("givre_courbe_demarrer", &[I32, I32], &[I32]),
    ("givre_courbe_pas", &[], &[I32]),
    ("givre_courbe_fait", &[], &[I32]),
    ("givre_courbe_total", &[], &[I32]),
    ("givre_courbe_code", &[], &[I32]),
    ("givre_courbe_moyenne", &[I32], &[F64]),
    ("givre_courbe_erreur_type", &[I32], &[F64]),
    ("givre_courbe_repere", &[I32], &[F64]),
];

/// Vrai quand la fonctionnalité `etape-d3` est active : le module et la page doivent alors avoir les vingt-trois fonctions.
pub const D3_ACTIVE: bool = cfg!(feature = "etape-d3");

/// Les quatre fonctions de l'image nette de l'étape V7 (voir `src/vitrine.rs`, « L'image nette »), avec leur signature. Le module et la page d'avant V7 ne les ont pas : tant que la fonctionnalité `etape-v7` n'est pas active, les tests acceptent le module et
/// la page tels qu'ils sont (onze, quinze ou vingt-trois fonctions) ou étendus de ces quatre (vingt-sept) ; avec `etape-v7`, ils n'acceptent que les vingt-sept.
pub const FONCTIONS_V7: [(&str, &[u8], &[u8]); 4] = [
    ("givre_nette_preparer", &[I32, I32], &[I32]),
    ("givre_nette_largeur", &[], &[I32]),
    ("givre_nette_hauteur", &[], &[I32]),
    ("givre_nette_pixel", &[I32, I32], &[I32]),
];

/// Vrai quand la fonctionnalité `etape-v7` est active : le module et la page doivent alors avoir les vingt-sept fonctions.
pub const V7_ACTIVE: bool = cfg!(feature = "etape-v7");

/// Les cinq fonctions de la neige de l'étape G3 (voir `src/vitrine.rs`, « La neige »), avec leur signature. Le module et la page d'avant G3 ne les ont pas : tant que la fonctionnalité `etape-g3` n'est pas active, les tests acceptent le module et
/// la page tels qu'ils sont (onze, quinze, vingt-trois ou vingt-sept fonctions) ou étendus de ces cinq (trente-deux) ; avec `etape-g3`, ils n'acceptent que les trente-deux.
pub const FONCTIONS_G3: [(&str, &[u8], &[u8]); 5] = [
    ("givre_neige_demarrer", &[I32, I32, I32, I32, I32], &[I32]),
    ("givre_neige_jeux", &[], &[I32]),
    ("givre_neige_jeu", &[I32, I32], &[I32]),
    ("givre_neige_reglage", &[I32], &[I32]),
    ("givre_neige_mesure", &[I32], &[I32]),
];

/// Vrai quand la fonctionnalité `etape-g3` est active : le module et la page doivent alors avoir les trente-deux fonctions.
pub const G3_ACTIVE: bool = cfg!(feature = "etape-g3");

/// Un lecteur d'octets avec une position.
struct Lecteur<'a> {
    octets: &'a [u8],
    position: usize,
}

impl<'a> Lecteur<'a> {
    fn new(octets: &'a [u8]) -> Lecteur<'a> {
        Lecteur { octets, position: 0 }
    }

    fn fini(&self) -> bool {
        self.position >= self.octets.len()
    }

    fn octet(&mut self) -> u8 {
        let octet = *self.octets.get(self.position).expect("fin inattendue du module");
        self.position += 1;
        octet
    }

    /// Un entier non signé de 32 bits en LEB128 : 7 bits par octet, le bit de poids fort dit qu'il y en a un autre.
    fn entier(&mut self) -> u32 {
        let mut valeur: u64 = 0;
        for rang in 0..5 {
            let octet = self.octet();
            valeur |= u64::from(octet & 0x7F) << (7 * rang);
            if octet & 0x80 == 0 {
                return u32::try_from(valeur).expect("entier LEB128 trop grand pour 32 bits");
            }
        }
        panic!("entier LEB128 de plus de 5 octets");
    }

    fn tranche(&mut self, longueur: usize) -> &'a [u8] {
        let fin = self.position.checked_add(longueur).filter(|fin| *fin <= self.octets.len());
        let fin = fin.expect("section plus longue que le module");
        let tranche = &self.octets[self.position..fin];
        self.position = fin;
        tranche
    }

    fn nom(&mut self) -> String {
        let longueur = self.entier() as usize;
        String::from_utf8(self.tranche(longueur).to_vec()).expect("nom en UTF-8")
    }
}

/// Une exportation : son nom, son genre (0 fonction, 1 table, 2 mémoire, 3 globale) et son indice.
pub struct Exportation {
    pub nom: String,
    pub genre: u8,
    pub indice: u32,
}

/// Ce que le test lit d'un module.
pub struct Module {
    /// Le nombre d'importations (0 si la section est absente).
    pub importations: u32,
    /// La section des types : pour chacun, les types des paramètres et ceux des résultats.
    pub types: Vec<(Vec<u8>, Vec<u8>)>,
    /// La section des fonctions : l'indice de type de chaque fonction définie dans le module.
    pub fonctions: Vec<u32>,
    pub exportations: Vec<Exportation>,
}

/// Analyse un module : l'en-tête (`\0asm` puis la version 1), puis les sections repérées par leur numéro. Les sections qui ne
/// servent pas ici (personnalisées, mémoire, code, données…) sont sautées sans être lues.
pub fn analyser(octets: &[u8]) -> Module {
    assert!(octets.len() >= 8, "module trop court : {} octets", octets.len());
    assert_eq!(&octets[0..4], b"\0asm", "le module ne commence pas par \\0asm");
    assert_eq!(octets[4..8], [1, 0, 0, 0], "le module n'est pas en version 1");

    let mut module = Module { importations: 0, types: Vec::new(), fonctions: Vec::new(), exportations: Vec::new() };
    let mut lecteur = Lecteur::new(&octets[8..]);
    while !lecteur.fini() {
        let numero = lecteur.octet();
        let taille = lecteur.entier() as usize;
        let mut section = Lecteur::new(lecteur.tranche(taille));
        match numero {
            1 => {
                for _ in 0..section.entier() {
                    assert_eq!(section.octet(), 0x60, "un type de fonction commence par 0x60");
                    let parametres = (0..section.entier()).map(|_| section.octet()).collect();
                    let resultats = (0..section.entier()).map(|_| section.octet()).collect();
                    module.types.push((parametres, resultats));
                }
            }
            2 => module.importations = section.entier(),
            3 => {
                for _ in 0..section.entier() {
                    module.fonctions.push(section.entier());
                }
            }
            7 => {
                for _ in 0..section.entier() {
                    let nom = section.nom();
                    let genre = section.octet();
                    let indice = section.entier();
                    module.exportations.push(Exportation { nom, genre, indice });
                }
            }
            _ => {}
        }
    }
    module
}

// ---------------------------------------------------------------------------------------------------------------
// Le module commité, la page et Node.
// ---------------------------------------------------------------------------------------------------------------

pub fn chemin_module() -> PathBuf {
    racine().join("docs").join("givre.wasm")
}

pub fn lire_module() -> Vec<u8> {
    std::fs::read(chemin_module()).unwrap_or_else(|erreur| panic!("docs/givre.wasm se lit : {erreur}"))
}

/// Le contenu d'un fichier du dossier de la page (`docs/` ou `GIVRE_DOCS` : voir `dossier_des_pages`). Un fichier absent ou vide est un échec (un test qui n'a rien lu ne prouve rien). Un fichier HTML est rendu **sans ses commentaires** `<!-- … -->` (ce qui n'est écrit que dans un commentaire n'est pas dans la page) ;
/// les recherches de ce qui est interdit jusque dans les commentaires lisent les fichiers bruts (`textes_des_fichiers`).
pub fn lire_page(nom: &str) -> String {
    let chemin = dossier_des_pages().join(nom);
    let texte = std::fs::read_to_string(&chemin)
        .unwrap_or_else(|erreur| panic!("{} existe et se lit : {erreur}", chemin.display()));
    assert!(!texte.trim().is_empty(), "{} est vide", chemin.display());
    if !matches!(extension_de(nom).as_str(), "html" | "htm") {
        return texte;
    }
    // Un HTML se lit **sans ses commentaires** : un identifiant, une phrase ou un script écrits seulement dans `<!-- … -->` ne sont pas dans la page.
    let sans = sans_commentaires_html(&texte);
    assert!(!sans.trim().is_empty(), "{} ne contient que des commentaires", chemin.display());
    sans
}

pub fn chemin_script(nom: &str) -> PathBuf {
    racine().join("tests").join("etape_5").join(nom)
}

/// Vrai si Node est là. Sinon : le test échoue si la variable d'environnement `CI` est définie (les machines de la CI ont
/// Node, et un test qui ne vérifie rien n'y est pas acceptable) ; autrement une note sur la sortie d'erreur et le test passe
/// (un poste sans Node ne bloque pas le développement).
pub fn node_est_la() -> bool {
    let present = Command::new("node").arg("--version").output().map(|sortie| sortie.status.success()).unwrap_or(false);
    if !present {
        assert!(std::env::var_os("CI").is_none(), "Node est introuvable alors que la variable CI est définie");
        eprintln!("note : Node est absent, ce test ne vérifie rien sur ce poste");
    }
    present
}

/// Lance ce script (de `tests/etape_5/`) avec ces arguments.
pub fn lancer_le_script(script: &str, arguments: &[&str]) -> Output {
    Command::new("node")
        .arg(chemin_script(script))
        .args(arguments)
        .output()
        .unwrap_or_else(|erreur| panic!("le lancement de Node échoue : {erreur}"))
}

// ---------------------------------------------------------------------------------------------------------------
// Étape V8 : la page est **tous les fichiers de son dossier**, pas trois fichiers à la virgule près.
// ---------------------------------------------------------------------------------------------------------------

/// Les deux fonctions des masses de l'étape V8 (voir `src/vitrine.rs`, « La masse de chaque case »), avec leur signature. Le module d'avant V8 ne les a pas : tant que la fonctionnalité `etape-v8` n'est pas active, les tests acceptent le module tel qu'il est (les trente-deux fonctions) ou
/// étendu de ces deux (trente-quatre) ; avec `etape-v8`, ils n'acceptent que les trente-quatre. **La page peut employer ces deux fonctions ou non, dans les deux régimes** (la session de site les emploie dans un worker quand le module les a).
pub const FONCTIONS_V8: [(&str, &[u8], &[u8]); 2] =
    [("givre_neige_masses_preparer", &[], &[I32]), ("givre_neige_masse", &[I32], &[I32])];

/// Vrai quand la fonctionnalité `etape-v8` est active : le module doit alors avoir les trente-quatre fonctions.
pub const V8_ACTIVE: bool = cfg!(feature = "etape-v8");

/// Les identifiants que les tests de la page exigent (étapes 5, Z1, D3, V7 et G3), **chacun une seule fois, dans `index.html`** : à n'importe quelle place (un tiroir, un `<aside>`, un `<dialog>`, un `<div hidden>`), dans n'importe quel élément. Quarante-huit.
pub const IDENTIFIANTS_DE_LA_PAGE: [&str; 48] = [
    "vitrine",
    "modele",
    "graine",
    "relancer",
    "vitesse",
    "temps",
    "empreinte",
    "mesurer",
    "courbe",
    "resultat",
    "barre-zinc",
    "marque-d",
    "repere-zinc",
    "compatible",
    "hors-barre",
    "taille-zinc",
    "eta",
    "valeur-eta",
    "commande-eta",
    "bloc-zinc",
    "bloc-repere-eta",
    "repere-eta",
    "ecart-eta",
    "pas-un-accord",
    "calculer-courbe",
    "avancement-courbe",
    "courbe-eta",
    "legende-courbe",
    "limite-courbe",
    "pas-un-accord-courbe",
    "vitrine-nette",
    "legende-nette",
    "commandes-neige",
    "jeu-neige",
    "neige-rho",
    "neige-beta",
    "neige-kappa",
    "neige-mu",
    "valeur-neige-rho",
    "valeur-neige-beta",
    "valeur-neige-kappa",
    "valeur-neige-mu",
    "source-neige",
    "legende-neige",
    "mesures-neige-bloc",
    "reglages-neige",
    "mesures-neige",
    "forme-neige",
];

/// Le dossier de la page : `docs/`, **sauf** si la variable d'environnement `GIVRE_DOCS` (non vide) en désigne un autre (un chemin absolu, ou relatif à la racine du dépôt). Elle ne sert qu'à jouer les tests de la page sur les pages d'essai de `tests/etape_v8/pages/` :
/// la CI ne la définit pas, et les tests, eux, ne changent pas (le dossier seul change). Le module `docs/givre.wasm` reste toujours celui de `docs/` (une page d'essai n'en a pas).
pub fn dossier_des_pages() -> PathBuf {
    match std::env::var_os("GIVRE_DOCS").filter(|valeur| !valeur.is_empty()) {
        Some(valeur) => {
            let dossier = PathBuf::from(valeur);
            if dossier.is_absolute() { dossier } else { racine().join(dossier) }
        }
        None => racine().join("docs"),
    }
}

/// Le chemin de `lecture.js` dans le dossier de la page (celui que les scripts de Node chargent).
pub fn chemin_de_lecture() -> PathBuf {
    dossier_des_pages().join("lecture.js")
}

/// Un fichier de la page : son nom (le chemin relatif au dossier de la page, avec des `/`), son chemin et sa taille.
pub struct FichierDePage {
    pub nom: String,
    pub chemin: PathBuf,
    pub taille: u64,
}

/// Tous les fichiers du dossier de la page, sous-dossiers compris, triés par nom, **sauf le module** `givre.wasm` (à la racine du dossier : son poids est dans `etape_5_artefact.rs`). Un dossier absent ou sans aucun fichier est un échec (un compte vide n'est pas un compte vert), **un fichier de 0 octet aussi**.
pub fn fichiers_de_la_page() -> Vec<FichierDePage> {
    fn parcourir(dossier: &Path, prefixe: &str, sortie: &mut Vec<FichierDePage>) {
        let entrees =
            std::fs::read_dir(dossier).unwrap_or_else(|erreur| panic!("{} se lit : {erreur}", dossier.display()));
        for entree in entrees {
            let chemin = entree.expect("entrée de dossier").path();
            let nom =
                format!("{prefixe}{}", chemin.file_name().and_then(|n| n.to_str()).expect("nom de fichier en UTF-8"));
            if chemin.is_dir() {
                parcourir(&chemin, &format!("{nom}/"), sortie);
            } else if nom != "givre.wasm" {
                let taille = std::fs::metadata(&chemin).expect("métadonnées").len();
                sortie.push(FichierDePage { nom, chemin, taille });
            }
        }
    }
    let mut fichiers = Vec::new();
    parcourir(&dossier_des_pages(), "", &mut fichiers);
    fichiers.sort_by(|a, b| a.nom.cmp(&b.nom));
    assert!(!fichiers.is_empty(), "le dossier de la page ({}) est vide", dossier_des_pages().display());
    for fichier in &fichiers {
        assert!(fichier.taille > 0, "le fichier {} de la page est vide (0 octet)", fichier.nom);
    }
    fichiers
}

/// L'extension d'un nom de fichier, en minuscules (`""` s'il n'en a pas).
pub fn extension_de(nom: &str) -> String {
    nom.rsplit_once('.').map(|(_, extension)| extension.to_lowercase()).unwrap_or_default()
}

/// Les extensions comptées comme **images** (budget à part : voir `la_page_ne_depasse_pas_son_budget`) ; tout autre fichier de la page, hors le module, est du code (HTML, CSS, JS, SVG, polices, données).
pub const IMAGES: [&str; 7] = ["png", "jpg", "jpeg", "webp", "avif", "gif", "ico"];

pub fn est_une_image(nom: &str) -> bool {
    IMAGES.contains(&extension_de(nom).as_str())
}

/// Les fichiers de la page d'une de ces extensions, avec leur texte (les fichiers qui ne sont pas de l'UTF-8 sont sautés : ce ne sont pas des textes).
pub fn textes_des_fichiers(extensions: &[&str]) -> Vec<(String, String)> {
    fichiers_de_la_page()
        .into_iter()
        .filter(|f| extensions.contains(&extension_de(&f.nom).as_str()))
        .filter_map(|f| std::fs::read_to_string(&f.chemin).ok().map(|texte| (f.nom, texte)))
        .collect()
}

/// Un morceau d'un script : du code, une chaîne (son guillemet et son texte brut, échappements gardés) ou un littéral d'expression régulière. Les commentaires ne sont pas des morceaux.
enum MorceauJs {
    Code(String),
    Chaine(char, String),
    Regex(String),
}

/// Vrai si un `/` rencontré après ce code commence un littéral d'expression régulière (et non une division) : au début du script, après un signe qui attend une valeur (`(`, `,`, `=`, `:`, `[`, `!`, `&`, `|`, `?`, `{`, `}`, `;`, `+`, `-`, `*`, `%`, `<`, `>`, `~`, `^`)
/// ou après un mot-clé qui en attend une (`return`, `typeof`, `case`…). Après un nom, un nombre, `)` ou `]`, c'est une division.
fn une_regex_peut_commencer(code: &str, apres_un_litteral: bool) -> bool {
    let avant = code.trim_end();
    let Some(dernier) = avant.chars().next_back() else { return !apres_un_litteral };
    if "(,=:[!&|?{};+-*%<>~^".contains(dernier) {
        return true;
    }
    let mot: String = avant.chars().rev().take_while(|c| c.is_alphanumeric() || *c == '_' || *c == '$').collect();
    let mot: String = mot.chars().rev().collect();
    ["return", "typeof", "case", "in", "of", "void", "delete", "throw", "do", "else", "yield", "await"]
        .contains(&mot.as_str())
}

/// L'indice du `/` qui ferme le littéral d'expression régulière ouvert en `debut` (un `/` dans une classe `[…]` ou après `\` ne ferme pas) ; `None` si la ligne finit avant (ce n'était pas une expression régulière).
fn fin_de_regex(c: &[char], debut: usize) -> Option<usize> {
    let (mut i, mut classe) = (debut + 1, false);
    while i < c.len() {
        match c[i] {
            '\n' => return None,
            '\\' => i += 1,
            '[' => classe = true,
            ']' => classe = false,
            '/' if !classe => return Some(i),
            _ => {}
        }
        i += 1;
    }
    None
}

/// Découpe un script en morceaux (voir `MorceauJs`), commentaires `//` et `/* */` ôtés (un commentaire `/* */` laisse une espace). Un littéral d'expression régulière est reconnu (`/"/g`, `/\/\//`) : ses guillemets n'ouvrent pas de chaîne et ses `//` ne sont pas un commentaire.
/// **Limite** : `${…}` d'un gabarit n'est pas analysé (le gabarit va jusqu'à l'accent grave suivant).
fn decouper_le_js(script: &str) -> Vec<MorceauJs> {
    let c: Vec<char> = script.chars().collect();
    let mut morceaux: Vec<MorceauJs> = Vec::new();
    let mut code = String::new();
    let mut i = 0;
    while i < c.len() {
        match c[i] {
            guillemet @ ('"' | '\'' | '`') => {
                let mut brut = String::new();
                i += 1;
                while i < c.len() && c[i] != guillemet {
                    if c[i] == '\\' && i + 1 < c.len() {
                        brut.push(c[i]);
                        i += 1;
                    }
                    brut.push(c[i]);
                    i += 1;
                }
                i += 1;
                if !code.is_empty() {
                    morceaux.push(MorceauJs::Code(std::mem::take(&mut code)));
                }
                morceaux.push(MorceauJs::Chaine(guillemet, brut));
            }
            '/' if c.get(i + 1) == Some(&'/') => {
                while i < c.len() && c[i] != '\n' {
                    i += 1;
                }
            }
            '/' if c.get(i + 1) == Some(&'*') => {
                i += 2;
                while i + 1 < c.len() && !(c[i] == '*' && c[i + 1] == '/') {
                    i += 1;
                }
                i += 2;
                code.push(' ');
            }
            '/' if une_regex_peut_commencer(&code, !morceaux.is_empty()) && fin_de_regex(&c, i).is_some() => {
                let fin = fin_de_regex(&c, i).expect("la fin du littéral");
                if !code.is_empty() {
                    morceaux.push(MorceauJs::Code(std::mem::take(&mut code)));
                }
                morceaux.push(MorceauJs::Regex(c[i..=fin].iter().collect()));
                i = fin + 1;
            }
            autre => {
                code.push(autre);
                i += 1;
            }
        }
    }
    if !code.is_empty() {
        morceaux.push(MorceauJs::Code(code));
    }
    morceaux
}

/// Le code d'un script sans ses commentaires `//` et `/* */` (hors des chaînes `"…"`, `'…'` et `` `…` `` et des littéraux d'expression régulière, gardés tels quels : voir `decouper_le_js`).
pub fn sans_commentaires(script: &str) -> String {
    let mut sortie = String::new();
    for morceau in decouper_le_js(script) {
        match morceau {
            MorceauJs::Code(code) | MorceauJs::Regex(code) => sortie.push_str(&code),
            MorceauJs::Chaine(guillemet, brut) => {
                sortie.push(guillemet);
                sortie.push_str(&brut);
                sortie.push(guillemet);
            }
        }
    }
    sortie
}

/// Le texte d'une chaîne JavaScript sans ses échappements : `\'`, `\"`, `` \` `` et `\\` rendent le caractère, `\n`, `\r` et `\t` une espace, `\uXXXX`, `\u{…}` et `\xXX` le caractère du code ; tout autre `\c` rend `c`.
fn sans_echappements(brut: &str) -> String {
    let c: Vec<char> = brut.chars().collect();
    let mut sortie = String::new();
    let mut i = 0;
    while i < c.len() {
        if c[i] != '\\' || i + 1 >= c.len() {
            sortie.push(c[i]);
            i += 1;
            continue;
        }
        let hexa = |debut: usize, fin: usize| -> Option<char> {
            let chiffres: String = c.get(debut..fin)?.iter().collect();
            char::from_u32(u32::from_str_radix(&chiffres, 16).ok()?)
        };
        match c[i + 1] {
            'n' | 'r' | 't' => {
                sortie.push(' ');
                i += 2;
            }
            'u' if c.get(i + 2) == Some(&'{') => match c[i + 3..].iter().position(|x| *x == '}') {
                Some(longueur) if hexa(i + 3, i + 3 + longueur).is_some() => {
                    sortie.push(hexa(i + 3, i + 3 + longueur).expect("un caractère"));
                    i += 4 + longueur;
                }
                _ => {
                    sortie.push('u');
                    i += 2;
                }
            },
            'u' if hexa(i + 2, i + 6).is_some() => {
                sortie.push(hexa(i + 2, i + 6).expect("un caractère"));
                i += 6;
            }
            'x' if hexa(i + 2, i + 4).is_some() => {
                sortie.push(hexa(i + 2, i + 4).expect("un caractère"));
                i += 4;
            }
            autre => {
                sortie.push(autre);
                i += 2;
            }
        }
    }
    sortie
}

/// Un texte **tel qu'on le lit à l'écran**, pour y chercher une phrase : l'apostrophe typographique `’` (et `ʼ`, `‘`) devient `'`, les espaces insécables (U+00A0, U+202F, U+2009) deviennent une espace. Une phrase d'honnêteté ou un mot interdit se trouvent quelle que soit la typographie de la page.
pub fn normaliser_le_texte(texte: &str) -> String {
    texte
        .chars()
        .map(|c| match c {
            '\u{2019}' | '\u{2bc}' | '\u{2018}' => '\'',
            '\u{a0}' | '\u{202f}' | '\u{2009}' => ' ',
            autre => autre,
        })
        .collect()
}

/// Un texte HTML dont les entités les plus courantes sont rendues en caractères (`&nbsp;`, `&eacute;`, `&egrave;`, `&agrave;`, `&rsquo;`, `&#233;`, `&#xE9;`…) : « r&eacute;aliste » se lit « réaliste ».
pub fn sans_entites(texte: &str) -> String {
    const NOMMEES: [(&str, char); 22] = [
        ("nbsp", ' '),
        ("amp", '&'),
        ("lt", '<'),
        ("gt", '>'),
        ("quot", '"'),
        ("apos", '\''),
        ("rsquo", '\''),
        ("lsquo", '\''),
        ("eacute", 'é'),
        ("egrave", 'è'),
        ("ecirc", 'ê'),
        ("euml", 'ë'),
        ("agrave", 'à'),
        ("acirc", 'â'),
        ("ccedil", 'ç'),
        ("ocirc", 'ô'),
        ("ucirc", 'û'),
        ("ugrave", 'ù'),
        ("icirc", 'î'),
        ("iuml", 'ï'),
        ("laquo", '«'),
        ("raquo", '»'),
    ];
    let mut sortie = String::new();
    let mut reste = texte;
    while let Some(debut) = reste.find('&') {
        sortie.push_str(&reste[..debut]);
        let apres = &reste[debut + 1..];
        let caractere = apres.find(';').filter(|fin| *fin <= 10).and_then(|fin| {
            let entite = &apres[..fin];
            let lu = if let Some(hexa) = entite.strip_prefix("#x").or_else(|| entite.strip_prefix("#X")) {
                u32::from_str_radix(hexa, 16).ok().and_then(char::from_u32)
            } else if let Some(decimal) = entite.strip_prefix('#') {
                decimal.parse::<u32>().ok().and_then(char::from_u32)
            } else {
                NOMMEES.iter().find(|(nom, _)| *nom == entite).map(|(_, c)| *c)
            };
            lu.map(|c| (c, fin))
        });
        match caractere {
            Some((c, fin)) => {
                sortie.push(c);
                reste = &apres[fin + 1..];
            }
            None => {
                sortie.push('&');
                reste = apres;
            }
        }
    }
    sortie.push_str(reste);
    sortie
}

/// Un HTML (ou un SVG) sans ses commentaires `<!-- … -->` (un commentaire jamais fermé va jusqu'à la fin).
pub fn sans_commentaires_html(html: &str) -> String {
    let mut sortie = String::new();
    let mut reste = html;
    while let Some(debut) = reste.find("<!--") {
        sortie.push_str(&reste[..debut]);
        reste = match reste[debut + 4..].find("-->") {
            Some(fin) => &reste[debut + 4 + fin + 3..],
            None => "",
        };
    }
    sortie.push_str(reste);
    sortie
}

/// Le texte où chaque contenu d'une adresse `data:…` (une police ou une image en base64, un SVG en ligne : voir `est_un_contenu_de_donnees`) est remplacé par `data:` : un tel contenu n'est pas du code ni du texte de la page, et ses lettres ne doivent faire échouer aucune recherche de mot.
pub fn sans_donnees(texte: &str) -> String {
    let mut sortie = String::new();
    let mut reste = texte;
    while let Some(debut) = reste.find("data:") {
        sortie.push_str(&reste[..debut + 5]);
        let apres = &reste[debut + 5..];
        if !est_un_contenu_de_donnees(apres) {
            reste = apres;
            continue;
        }
        let fin = apres
            .find(|c: char| c == '"' || c == '\'' || c == '`' || c == ')' || c == '<' || c == '>' || c.is_whitespace())
            .unwrap_or(apres.len());
        reste = &apres[fin..];
    }
    sortie.push_str(reste);
    sortie
}

/// Vrai si ce qui suit `data:` est le contenu d'une adresse `data:` : un type (`font/woff2`, `image/svg+xml`, `text/javascript`…) suivi de `;` ou de `,`, ou une virgule seule (`data:,…`). Une clé d'objet `data:` suivie de code (`{data:fetch("https://…")}`, `{data:lire(m)}`) n'en est pas un : ce code reste lu.
fn est_un_contenu_de_donnees(apres: &str) -> bool {
    let Some(fin) = apres.find([';', ',']) else { return false };
    let sorte = &apres[..fin];
    fin == 0 && apres.starts_with(',')
        || (!sorte.is_empty()
            && sorte.len() <= 40
            && sorte.contains('/')
            && sorte.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, '/' | '+' | '-' | '.')))
}

/// Les balises du HTML, chacune réduite à son contenu entre `<` et `>` (le nom, puis les attributs).
pub fn balises(html: &str) -> Vec<&str> {
    html.split('<').skip(1).filter_map(|morceau| morceau.split('>').next()).collect()
}

/// La valeur de l'attribut `nom` d'une balise (précédé d'une espace ; le nom en majuscules ou en minuscules, des espaces permises autour du `=`, la valeur entre guillemets, entre apostrophes **ou sans guillemets** : `<img src=http://…>` charge autant que `<img src="http://…">`).
pub fn attribut(balise: &str, nom: &str) -> Option<String> {
    let minuscules = balise.to_ascii_lowercase();
    let nom = nom.to_ascii_lowercase();
    for (position, _) in minuscules.match_indices(&nom) {
        if !minuscules[..position].ends_with(char::is_whitespace) {
            continue;
        }
        let Some(valeur) = balise[position + nom.len()..].trim_start().strip_prefix('=') else { continue };
        let valeur = valeur.trim_start();
        return match valeur.chars().next() {
            Some(guillemet @ ('"' | '\'')) => valeur[1..].find(guillemet).map(|fin| valeur[1..1 + fin].to_string()),
            _ => Some(valeur.split(char::is_whitespace).next().unwrap_or("").trim_end_matches('/').to_string()),
        };
    }
    None
}

/// Le nom d'une balise, en minuscules.
pub fn nom_de_balise(balise: &str) -> String {
    balise.trim_start_matches('/').split(|c: char| c.is_whitespace() || c == '/').next().unwrap_or("").to_lowercase()
}

/// Les scripts d'un HTML : pour chaque `<script …>…</script>`, `(la balise ouvrante sans < ni >, le corps)`.
pub fn scripts_en_ligne(html: &str) -> Vec<(String, String)> {
    let minuscules = html.to_ascii_lowercase();
    let mut scripts = Vec::new();
    let mut depuis = 0;
    while let Some(debut) = minuscules[depuis..].find("<script") {
        let debut = depuis + debut;
        let Some(fin_balise) = minuscules[debut..].find('>') else { break };
        let ouvrante = &html[debut + 1..debut + fin_balise];
        let corps_debut = debut + fin_balise + 1;
        let Some(fermeture) = minuscules[corps_debut..].find("</script") else { break };
        scripts.push((ouvrante.to_string(), html[corps_debut..corps_debut + fermeture].to_string()));
        depuis = corps_debut + fermeture + 1;
    }
    scripts
}

/// **Tout le code JavaScript de la page** : chaque fichier `.js` ou `.mjs` du dossier (`lecture.js`, `app.js`, un worker, n'importe quel autre) et chaque script en ligne des fichiers HTML (`index.html#script1`, …), **sans leurs commentaires** : `(nom, code)`.
/// Un script de données (`application/json`, `importmap`, `ld+json`) n'est pas du code.
pub fn code_js_de_la_page() -> Vec<(String, String)> {
    let mut code: Vec<(String, String)> = textes_des_fichiers(&["js", "mjs"])
        .into_iter()
        .map(|(nom, texte)| (nom, sans_commentaires(&sans_donnees(&texte))))
        .collect();
    for (nom, html) in textes_des_fichiers(&["html", "htm"]) {
        for (rang, (ouvrante, corps)) in scripts_en_ligne(&html).into_iter().enumerate() {
            let sorte = attribut(&ouvrante, "type").unwrap_or_default().to_lowercase();
            if attribut(&ouvrante, "src").is_some() || sorte.contains("json") || sorte.contains("importmap") {
                continue;
            }
            code.push((format!("{nom}#script{}", rang + 1), sans_commentaires(&sans_donnees(&corps))));
        }
    }
    code
}

/// Tout le code JavaScript de la page, bout à bout (voir `code_js_de_la_page`) : la lecture des tests de forme qui ne dépendent d'aucun fichier en particulier.
pub fn tout_le_code_js() -> String {
    code_js_de_la_page().into_iter().map(|(_, code)| code).collect::<Vec<_>>().join("\n")
}

/// Le code JavaScript de la page **hors `lecture.js`** (tous les autres fichiers et les scripts en ligne).
pub fn code_js_hors_lecture() -> String {
    code_js_de_la_page()
        .into_iter()
        .filter(|(nom, _)| nom != "lecture.js" && !nom.ends_with("/lecture.js"))
        .map(|(_, code)| code)
        .collect::<Vec<_>>()
        .join("\n")
}

/// Les chaînes de caractères d'un code (le contenu entre guillemets `"…"`, `'…'` ou `` `…` ``) : ce que le code écrit à l'écran, sans le code lui-même. Elles sont rendues **comme on les lit** : sans leurs échappements (`'l\'article'` se lit `l'article`, `\u00e9` se lit `é`) et avec la typographie
/// normalisée (`normaliser_le_texte`) ; les guillemets d'un littéral d'expression régulière n'ouvrent pas de chaîne.
pub fn chaines_du_code(code: &str) -> Vec<String> {
    decouper_le_js(code)
        .into_iter()
        .filter_map(|morceau| match morceau {
            MorceauJs::Chaine(_, brut) => Some(normaliser_le_texte(&sans_echappements(&brut))),
            _ => None,
        })
        .collect()
}

/// Le texte visible d'un HTML : tout ce qui n'est pas dans une balise, ni dans `<style>` ni dans `<script>`.
pub fn texte_visible_du_html(html: &str) -> String {
    let html = sans_commentaires_html(html);
    let mut sortie = String::new();
    let mut ignore = false;
    for morceau in html.split('<').skip(1) {
        let (balise, texte) = morceau.split_once('>').unwrap_or((morceau, ""));
        let ferme = balise.starts_with('/');
        if matches!(nom_de_balise(balise).as_str(), "style" | "script") {
            ignore = !ferme;
        }
        if !ignore {
            sortie.push_str(texte);
            sortie.push(' ');
        }
    }
    normaliser_le_texte(&sans_entites(&sortie))
}

/// Les éléments HTML sans contenu (jamais de balise fermante).
const ELEMENTS_VIDES: [&str; 13] =
    ["area", "base", "br", "col", "embed", "hr", "img", "input", "link", "meta", "source", "track", "wbr"];

/// **Le texte de l'élément d'`id` donné** : tout le texte entre sa balise ouvrante et sa balise fermante, balises intérieures ôtées (`Le <em>mur</em> est à nous` se lit « Le mur est à nous » ; une légende peut avoir un mot en romain, un lien), sans les commentaires ni le contenu d'un `<script>` ou d'un `<style>`,
/// entités rendues, typographie normalisée, espaces réduites à une seule, sans espaces de début et de fin. `None` si aucun élément n'a cet `id` ; `""` pour un élément vide ou sans contenu (`<input>`, `<output></output>`). Si la balise fermante ne se trouve pas (HTML mal fermé), c'est le texte jusqu'à la balise suivante.
pub fn texte_de_l_element(html: &str, id: &str) -> Option<String> {
    let html = sans_commentaires_html(html);
    let propre =
        |texte: &str| normaliser_le_texte(&sans_entites(texte)).split_whitespace().collect::<Vec<_>>().join(" ");
    let (mut sorte, mut profondeur, mut ignore) = (String::new(), 0_i32, false);
    let (mut texte, mut premier) = (String::new(), None);
    for morceau in html.split('<').skip(1) {
        let (balise, suite) = morceau.split_once('>').unwrap_or((morceau, ""));
        let ferme = balise.starts_with('/');
        let nom = nom_de_balise(balise);
        if profondeur == 0 {
            if !ferme && attribut(balise, "id").as_deref() == Some(id) {
                if ELEMENTS_VIDES.contains(&nom.as_str()) || balise.trim_end().ends_with('/') {
                    return Some(String::new());
                }
                sorte = nom;
                profondeur = 1;
                texte.push_str(suite);
                premier = Some(propre(suite));
            }
            continue;
        }
        if nom == sorte {
            profondeur += if ferme { -1 } else { 1 };
            if profondeur == 0 {
                return Some(propre(&texte));
            }
        }
        if matches!(nom.as_str(), "script" | "style") {
            ignore = !ferme;
        }
        if !ignore {
            texte.push_str(suite);
        }
    }
    premier
}

/// **Tout ce que la page écrit à l'écran, en texte** : le texte visible de chaque fichier HTML (sans ses commentaires, entités rendues), les chaînes de caractères de tout le code JavaScript (une légende peut être écrite par un script), le texte des fichiers SVG, **les attributs que le navigateur montre ou donne à lire**
/// (`content` d'un `<meta>` : la carte de partage ; `alt`, `title`, `aria-label`, `placeholder`, `label`) et **les chaînes des feuilles de style** (`content: "…"` s'affiche), typographie normalisée (`normaliser_le_texte`). Les mots interdits et les valeurs publiées se cherchent ici.
pub fn textes_ecrits_par_la_page() -> String {
    textes_ecrits(false)
}

/// Comme `textes_ecrits_par_la_page`, mais **en prose** : parmi les chaînes des scripts, seules celles qui contiennent une espace (une phrase ; pas un identifiant comme `"pas-un-accord"`, une classe ou un sélecteur).
pub fn textes_en_prose_de_la_page() -> String {
    textes_ecrits(true)
}

fn textes_ecrits(prose: bool) -> String {
    let garde = |texte: &String| !prose || texte.contains(char::is_whitespace);
    let mut textes: Vec<String> = Vec::new();
    for (_, brut) in textes_des_fichiers(&["html", "htm", "svg"]) {
        let html = sans_commentaires_html(&sans_donnees(&brut));
        // Les attributs que le navigateur montre ou donne à lire (une infobulle, un texte de remplacement, la carte de partage).
        for balise in balises(&html) {
            for nom in ["content", "alt", "title", "aria-label", "placeholder", "label"] {
                textes.extend(attribut(balise, nom).map(|v| normaliser_le_texte(&sans_entites(&v))).filter(garde));
            }
        }
        // Les chaînes des feuilles de style en ligne (`content: "…"` s'affiche).
        let mut reste = html.as_str();
        while let Some(debut) = reste.to_ascii_lowercase().find("<style") {
            let Some(fin_balise) = reste[debut..].find('>') else { break };
            let corps = &reste[debut + fin_balise + 1..];
            let fin = corps.to_ascii_lowercase().find("</style").unwrap_or(corps.len());
            textes.extend(chaines_du_code(&sans_donnees_css(&corps[..fin])).into_iter().filter(garde));
            reste = &corps[fin..];
        }
    }
    for (nom, html) in textes_des_fichiers(&["html", "htm", "svg"]) {
        if matches!(extension_de(&nom).as_str(), "html" | "htm") {
            textes.push(texte_visible_du_html(&sans_donnees(&html)));
        } else {
            textes.push(normaliser_le_texte(&sans_entites(&sans_commentaires_html(&sans_donnees(&html)))));
        }
    }
    for (_, code) in code_js_de_la_page() {
        textes.extend(chaines_du_code(&code).into_iter().filter(garde));
    }
    for (_, css) in textes_des_fichiers(&["css"]) {
        textes.extend(chaines_du_code(&sans_donnees_css(&css)).into_iter().filter(garde));
    }
    textes.join("\n")
}

/// Vrai si l'adresse charge quelque chose **hors de la page** : commence par `//`, ou a pour protocole `http`, `https`, `ws`, `wss`, `ftp` ou `file`. Une adresse relative, `data:` et `blob:` ne sont pas externes.
pub fn adresse_externe(adresse: &str) -> bool {
    let a = adresse.trim().to_lowercase();
    a.starts_with("//")
        || ["http:", "https:", "ws:", "wss:", "ftp:", "file:"].iter().any(|protocole| a.starts_with(protocole))
}

/// Les adresses des `url(…)` d'un texte CSS (ou d'un attribut `style`) et celles de ses `@import`.
fn adresses_du_css(css: &str) -> Vec<String> {
    let mut adresses = Vec::new();
    let mut reste = css;
    while let Some(debut) = reste.find("url(") {
        let apres = reste[debut + 4..].trim_start();
        let (adresse, suite) = match apres.chars().next() {
            Some(g @ ('"' | '\'')) => match apres[1..].find(g) {
                Some(fin) => (apres[1..1 + fin].to_string(), &apres[1 + fin..]),
                None => break,
            },
            _ => match apres.find(')') {
                Some(fin) => (apres[..fin].trim().to_string(), &apres[fin..]),
                None => break,
            },
        };
        adresses.push(adresse);
        reste = suite;
    }
    let mut reste = css;
    while let Some(debut) = reste.find("@import") {
        let apres = reste[debut + 7..].trim_start();
        if let Some(g @ ('"' | '\'')) = apres.chars().next()
            && let Some(fin) = apres[1..].find(g)
        {
            adresses.push(apres[1..1 + fin].to_string());
        }
        reste = &reste[debut + 7..];
    }
    adresses
}

/// Le littéral (entre `"`, `'` ou `` ` ``) qui commence le texte (espaces de tête sautés) ; un `new URL(` de tête est sauté (`new Worker(new URL("./calcul.js", import.meta.url))`).
fn litteral_au_debut(texte: &str) -> Option<String> {
    let mut t = texte.trim_start();
    if let Some(suite) = t.strip_prefix("new URL(") {
        t = suite.trim_start();
    }
    let guillemet = t.chars().next().filter(|g| matches!(g, '"' | '\'' | '`'))?;
    let fin = t[1..].find(guillemet)?;
    Some(t[1..1 + fin].to_string())
}

/// Les adresses que ce code JavaScript charge : l'argument littéral de `fetch(`, `importScripts(`, `import(`, `Worker(`, `SharedWorker(`, `EventSource(`, `WebSocket(` et `sendBeacon(`, le second argument de `.open(` (XMLHttpRequest), le texte d'une affectation `.src = "…"`, et celui de
/// `import … from "…"` ou `import "…"`. Un `new Worker(new URL("./calcul.js", import.meta.url))` est une adresse relative. **Limite** : seule une adresse écrite en littéral se voit (une adresse composée ne se lit pas).
fn adresses_chargees_par_le_code(code: &str) -> Vec<String> {
    let mut adresses = Vec::new();
    for appel in
        ["fetch(", "importScripts(", "import(", "Worker(", "SharedWorker(", "EventSource(", "WebSocket(", "sendBeacon("]
    {
        for (position, _) in code.match_indices(appel) {
            let avant = code[..position].chars().next_back();
            // `monFetch(` est un autre nom ; `Worker(` est précédé de `new ` ou de `.`.
            if !appel.ends_with("Worker(") && avant.is_some_and(|x| x.is_alphanumeric() || x == '_' || x == '$') {
                continue;
            }
            adresses.extend(litteral_au_debut(&code[position + appel.len()..]));
        }
    }
    for (position, _) in code.match_indices(".open(") {
        let reste = &code[position + 6..];
        if let Some((_, second)) = reste[..reste.len().min(200)].split_once(',') {
            adresses.extend(litteral_au_debut(second));
        }
    }
    for (position, _) in code.match_indices(".src") {
        if let Some(valeur) = code[position + 4..].trim_start().strip_prefix('=').filter(|v| !v.starts_with('=')) {
            adresses.extend(litteral_au_debut(valeur));
        }
    }
    for (position, _) in code.match_indices(" from ") {
        adresses.extend(litteral_au_debut(&code[position + 6..]));
    }
    for (position, _) in code.match_indices("import ") {
        adresses.extend(litteral_au_debut(&code[position + 7..]));
    }
    adresses
}

/// **Ce que la page charge hors d'elle** (V8) : la liste, vide si la page ne charge rien du dehors. Sont comptés : un `src` (de n'importe quel élément) ou un `href` d'un `<link>` (feuille de style, police, icône, préchargement, manifeste…) ou d'un élément SVG autre que le lien `<a>`, **le `href` d'un `<base>`** (il enverrait toutes les adresses relatives de la page au-dehors), un `url(…)` ou un
/// `@import` d'un CSS (une feuille ou un attribut `style`), et les adresses chargées par le JavaScript (voir `adresses_chargees_par_le_code`) dans tous les fichiers ; une adresse est externe si `adresse_externe`. **Ne sont pas comptés** : un lien `<a href="https://…">`, l'attribut `xmlns="http://www.w3.org/2000/svg"` d'un SVG,
/// une balise `<meta>` (`og:image`, `canonical`…), un lien `<link rel="canonical">` ou `alternate`, une adresse relative, `data:` et `blob:`.
pub fn ressources_externes() -> Vec<String> {
    let mut externes = Vec::new();
    let mut noter = |source: &str, genre: &str, adresse: &str| {
        if adresse_externe(adresse) {
            externes.push(format!("{source} : {genre} « {adresse} »"));
        }
    };
    for (nom, texte) in textes_des_fichiers(&["html", "htm", "svg"]) {
        let texte = sans_commentaires_html(&texte);
        for balise in balises(&sans_donnees(&texte)) {
            let element = nom_de_balise(balise);
            let rel = attribut(balise, "rel").unwrap_or_default().to_lowercase();
            let est_lien_de_donnee = element == "link"
                && !["canonical", "alternate", "author", "license", "help", "next", "prev"].contains(&rel.trim());
            for attr in ["src", "srcset", "poster", "data", "xlink:href"] {
                if let Some(valeur) = attribut(balise, attr) {
                    noter(&nom, &format!("<{element} {attr}>"), valeur.split_whitespace().next().unwrap_or(""));
                }
            }
            if let Some(valeur) = attribut(balise, "href")
                && ((element == "link" && est_lien_de_donnee) || !["a", "area", "link"].contains(&element.as_str()))
            {
                noter(&nom, &format!("<{element} href>"), &valeur);
            }
            if let Some(style) = attribut(balise, "style") {
                for adresse in adresses_du_css(&style) {
                    noter(&nom, "style url()", &adresse);
                }
            }
        }
        let mut reste = texte.as_str();
        while let Some(debut) = reste.to_ascii_lowercase().find("<style") {
            let Some(fin_balise) = reste[debut..].find('>') else { break };
            let corps = &reste[debut + fin_balise + 1..];
            let Some(fin) = corps.to_ascii_lowercase().find("</style") else { break };
            for adresse in adresses_du_css(&sans_donnees_css(&corps[..fin])) {
                noter(&nom, "<style> url() ou @import", &adresse);
            }
            reste = &corps[fin..];
        }
    }
    for (nom, texte) in textes_des_fichiers(&["css"]) {
        for adresse in adresses_du_css(&sans_donnees_css(&texte)) {
            noter(&nom, "url() ou @import", &adresse);
        }
    }
    for (nom, code) in code_js_de_la_page() {
        for adresse in adresses_chargees_par_le_code(&code) {
            noter(&nom, "adresse chargée par le script", &adresse);
        }
    }
    externes
}

/// Un CSS sans ses commentaires `/* … */` (et ses `data:` rendus vides) : ce qu'on lit pour y chercher des `url(` ou des règles.
pub fn sans_donnees_css(css: &str) -> String {
    let mut sortie = String::new();
    let mut morceaux = css.split("/*");
    sortie.push_str(morceaux.next().unwrap_or(""));
    for morceau in morceaux {
        if let Some((_, suite)) = morceau.split_once("*/") {
            sortie.push_str(suite);
        }
    }
    sans_donnees(&sortie)
}

/// Les budgets de la page (V8) : **le code** (HTML, CSS, JS, SVG, polices, données : tout fichier du dossier qui n'est ni une image ni le module) au plus `CODE_MAX` octets ; **une image** (`IMAGES`) au plus `IMAGE_MAX` octets, et toutes les images ensemble au plus `IMAGES_MAX` ; le module est borné dans `etape_5_artefact.rs`.
pub const CODE_MAX: u64 = 384 * 1024;
pub const IMAGE_MAX: u64 = 300_000;
pub const IMAGES_MAX: u64 = 600_000;

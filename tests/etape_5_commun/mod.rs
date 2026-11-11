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

pub fn lire_page(nom: &str) -> String {
    std::fs::read_to_string(racine().join("docs").join(nom))
        .unwrap_or_else(|erreur| panic!("docs/{nom} existe et se lit : {erreur}"))
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

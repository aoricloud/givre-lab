//! Ce que les fichiers de test de l'étape D3 partagent : la table des quatre modèles de claquage de la doc de `vitrine` (écrite ici à la main, jamais lue dans la
//! bibliothèque), l'état **natif** en face (un `Claquage` construit directement par la bibliothèque, jamais par `vitrine`), les chaînes que les scripts de Node doivent
//! écrire, le lancement de Node, et les aides de l'étape D2 (le calcul naïf de la dimension et du point de la courbe, qui n'appelle ni `courbe` ni `correlation`).
//!
//! Ce fichier n'est pas un test : il ne contient aucune fonction `#[test]`. Il est gelé avec les tests.
#![allow(dead_code)]

use std::path::PathBuf;
use std::process::{Command, Output};

use givre_lab::claquage::Claquage;
use givre_lab::dla::{COTE_APERCU, amas_colore};

/// Les aides de l'étape D2 (`dimension_naive`, `point_naif`, `fenetre_doc`, `correlation_naive`, `etendue_doc`, `demi_cote_doc`, les formes à solution connue…).
#[path = "../etape_d2_commun/mod.rs"]
pub mod d2;

/// Les aides de l'étape 5 (`node_est_la`, `chemin_module`, `racine`, `reference`, `lire_page`).
#[path = "../etape_5_commun/mod.rs"]
pub mod e5;

// ---------------------------------------------------------------------------------------------------------------
// La table de la doc de `vitrine` : « Les huit modèles », lignes 4 à 7.
// ---------------------------------------------------------------------------------------------------------------

/// Un modèle de claquage de la table « Les huit modèles » de la doc de `vitrine` (V-D3-10).
pub struct ModeleClaquage {
    /// Le numéro passé à `givre_demarrer` : `4 + η`.
    pub numero: u32,
    /// L'exposant `η`.
    pub eta: u32,
    /// Les sites (germe compris) que le modèle fait croître.
    pub sites: u32,
    /// Le demi-côté du domaine : `demi_cote_de_la_courbe(η, sites)`.
    pub demi_cote: u32,
    /// La durée : `sites - 1`.
    pub duree: u32,
}

/// Les quatre modèles de claquage, **valeurs de la doc** écrites à la main (le demi-côté est la règle de D2 : `30 + 10 * 400 / 100 = 70`, `32 + 8 * 250 / 100 = 52`,
/// `22 + 18 * 150 / 100 = 49`, `12 + 36 * 100 / 100 = 48`).
pub const TABLE_CLAQUAGE: [ModeleClaquage; 4] = [
    ModeleClaquage { numero: 4, eta: 0, sites: 400, demi_cote: 70, duree: 399 },
    ModeleClaquage { numero: 5, eta: 1, sites: 250, demi_cote: 52, duree: 249 },
    ModeleClaquage { numero: 6, eta: 2, sites: 150, demi_cote: 49, duree: 149 },
    ModeleClaquage { numero: 7, eta: 3, sites: 100, demi_cote: 48, duree: 99 },
];

// ---------------------------------------------------------------------------------------------------------------
// L'état natif en face.
// ---------------------------------------------------------------------------------------------------------------

/// Tout ce que la vitrine expose d'un modèle de claquage, tel que la bibliothèque le donne pour le même `η`, la même graine et le même nombre d'unités jouées.
pub struct NatifClaquage {
    pub temps: u32,
    pub duree: u32,
    pub largeur: u32,
    pub hauteur: u32,
    /// Les octets R, V, B de l'image (`amas_colore(sites, COTE_APERCU)`).
    pub rvb: Vec<u8>,
    pub sites: u32,
    pub rayon: u32,
    /// Les 64 caractères hexadécimaux de l'empreinte.
    pub empreinte: String,
    /// Les sites de l'amas (pour mesurer).
    pub amas: Vec<(i32, i32)>,
}

/// L'état natif du modèle de claquage d'exposant `eta` après `min(pas, durée)` unités : `Claquage::new(graine, demi-côté de la table, eta)` puis
/// `croitre_jusqu_a(1 + joué)`.
pub fn natif_claquage(eta: u32, graine: u32, pas: u32) -> NatifClaquage {
    let reglage = &TABLE_CLAQUAGE[eta as usize];
    let joues = pas.min(reglage.duree);
    let mut claquage = Claquage::new(u64::from(graine), reglage.demi_cote, eta);
    claquage.croitre_jusqu_a(1 + joues as usize);
    natif_depuis(&claquage, eta)
}

/// L'état natif d'un claquage déjà grandi (pour jouer un amas par morceaux sans le refaire à chaque morceau) : le temps est ses sites moins un.
pub fn natif_depuis(claquage: &Claquage, eta: u32) -> NatifClaquage {
    let reglage = &TABLE_CLAQUAGE[eta as usize];
    let (largeur, hauteur, rvb) = amas_colore(claquage.sites(), COTE_APERCU);
    NatifClaquage {
        temps: claquage.sites().len() as u32 - 1,
        duree: reglage.duree,
        largeur,
        hauteur,
        rvb,
        sites: claquage.sites().len() as u32,
        rayon: claquage.rayon(),
        empreinte: claquage.empreinte(),
        amas: claquage.sites().to_vec(),
    }
}

impl NatifClaquage {
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

/// Les 8 octets d'un `f64`, petit-boutiste, en hexadécimal, ou `NaN`.
pub fn hexa(x: f64) -> String {
    if x.is_nan() { "NaN".to_string() } else { x.to_le_bytes().iter().map(|o| format!("{o:02x}")).collect() }
}

/// Le demi-côté de la règle de D2 (écrite en dur dans `d2::demi_cote_doc`) pour `eta` et `taille`.
pub fn demi_cote_regle(eta: u32, taille: usize) -> u32 {
    d2::demi_cote_doc(eta, taille)
}

/// Le repère publié de `eta` : `1,71`, `1,433`, `1,263` pour `η = 1, 2, 3`, **écrits ici à la main** (V-D3-22) ; `None` sinon.
pub fn repere_ecrit_a_la_main(eta: u32) -> Option<f64> {
    match eta {
        1 => Some(1.71),
        2 => Some(1.433),
        3 => Some(1.263),
        _ => None,
    }
}

// ---------------------------------------------------------------------------------------------------------------
// Node.
// ---------------------------------------------------------------------------------------------------------------

pub fn racine() -> PathBuf {
    e5::racine().to_path_buf()
}

/// Lance un script de `tests/etape_d3/` avec ces arguments.
pub fn lancer_le_script(script: &str, arguments: &[&str]) -> Output {
    Command::new("node")
        .arg(racine().join("tests").join("etape_d3").join(script))
        .args(arguments)
        .output()
        .unwrap_or_else(|erreur| panic!("le lancement de Node échoue : {erreur}"))
}

/// Le chemin du module commité et de `lecture.js`, en texte.
pub fn chemin_du_module() -> String {
    e5::chemin_module().to_str().expect("chemin en UTF-8").to_string()
}

pub fn chemin_de_lecture() -> String {
    racine().join("docs").join("lecture.js").to_str().expect("chemin en UTF-8").to_string()
}

// ---------------------------------------------------------------------------------------------------------------
// Ce que les scripts de Node doivent écrire (leur en-tête dit chaque ligne).
// ---------------------------------------------------------------------------------------------------------------

/// Les octets RGBA d'une image RVB (le canal alpha à 255), comme `lireImage` de `lecture.js` les range.
pub fn rgba_de(rvb: &[u8]) -> Vec<u8> {
    let mut rgba = Vec::with_capacity(rvb.len() / 3 * 4);
    for pixel in rvb.chunks(3) {
        rgba.extend_from_slice(pixel);
        rgba.push(255);
    }
    rgba
}

/// La mesure du calcul naïf de D2 d'un amas : `(rayons, valeurs, dimension)`, ou `None` s'il n'y a pas de dimension.
pub fn mesure_naive(amas: &[(i32, i32)]) -> Option<(Vec<u32>, Vec<f64>, f64)> {
    let rayons = d2::fenetre_doc(d2::etendue_doc(amas));
    let dimension = d2::dimension_naive(amas)?;
    let valeurs = d2::correlation_naive(amas, &rayons);
    Some((rayons, valeurs, dimension))
}

/// Ce que `modeles-wasm.mjs` doit écrire pour le modèle de claquage `modele` (4 à 7), la graine et le nombre de pas : l'état et la mesure du code natif, lus comme le script les lit.
pub fn attendu_du_script_des_modeles(modele: u32, graine: u32, pas: u32) -> String {
    let eta = modele - 4;
    let natif = natif_claquage(eta, graine, pas);
    let mesure = mesure_naive(&natif.amas);
    let (points, rayons, valeurs, dimension) = match &mesure {
        Some((rayons, valeurs, dimension)) => (
            rayons.len(),
            rayons.iter().map(u32::to_string).collect::<Vec<_>>().join(","),
            valeurs.iter().map(|v| hexa(*v)).collect::<Vec<_>>().join(","),
            hexa(*dimension),
        ),
        None => (0, String::new(), String::new(), "NaN".to_string()),
    };
    // Un `givre_avancer(1)` de plus : il joue une unité (et efface la mesure) sauf si le scénario est fini (il ne joue rien, la mesure reste).
    let apres = if natif.temps < natif.duree {
        "1,0,NaN".to_string()
    } else {
        match &mesure {
            Some((rayons, _, dimension)) => format!("0,{},{}", rayons[0], hexa(*dimension)),
            None => "0,0,NaN".to_string(),
        }
    };
    format!(
        "avant=1\ndemarre=1\njoue={}\ntemps={}\nduree={}\nlargeur={}\nhauteur={}\nsites={}\nrayon={}\ncompacite=NaN\nempreinte={}\nrgba={}\n\
         points={points}\nrayons={rayons}\nvaleurs={valeurs}\ndimension={dimension}\nfidele=1\napres={apres}\n",
        natif.temps,
        natif.temps,
        natif.duree,
        natif.largeur,
        natif.hauteur,
        natif.sites,
        natif.rayon,
        natif.empreinte,
        givre_lab::empreinte::sha256_hex(&rgba_de(&natif.rvb)),
    )
}

/// Ce que `courbe-wasm.mjs` doit écrire pour un travail de `taille` sites et `graines` graines (acceptés : 1 à 100 sites, 2 à 8 graines) : les amas de la bibliothèque faits dans l'ordre
/// (`η` de 0 à 3, graines 1 à `graines`), le travail s'arrêtant au premier amas qui échoue (code 3 pour une frontière touchée, 4 pour une fenêtre vide).
pub fn attendu_du_script_de_la_courbe(taille: u32, graines: u32) -> String {
    use givre_lab::courbe::{ErreurCourbe, essayer_dimension_d_une_graine, essayer_point_de_la_courbe, publie};
    let total = 4 * graines;
    let regle = |eta: u32| demi_cote_regle(eta, taille as usize);
    let mut suivants = Vec::new();
    let mut codes = Vec::new();
    let mut faits = Vec::new();
    let mut fait = 0_u32;
    let mut code_final = 0_u32;
    for _ in 0..40 {
        if fait == total {
            suivants.push("fin".to_string());
            break;
        }
        let (eta, graine) = (fait / graines, fait % graines + 1);
        suivants.push(format!("{eta}:{graine}"));
        match essayer_dimension_d_une_graine(eta, u64::from(graine), taille as usize, regle(eta)) {
            Ok(_) => {
                fait += 1;
                codes.push("0".to_string());
                faits.push(fait.to_string());
            }
            Err(erreur) => {
                code_final = match erreur {
                    ErreurCourbe::FrontiereTouchee { .. } => 3,
                    ErreurCourbe::FenetreVide { .. } => 4,
                    _ => 6,
                };
                codes.push(code_final.to_string());
                faits.push(fait.to_string());
                break;
            }
        }
    }
    let liste: Vec<u64> = (1..=u64::from(graines)).collect();
    let points: Vec<String> = (0..4_u32)
        .map(|eta| {
            let repere = publie(eta).map_or("NaN".to_string(), hexa);
            if fait >= (eta + 1) * graines {
                let p = essayer_point_de_la_courbe(eta, &liste, taille as usize, regle(eta)).expect("un point");
                format!("{}:{}:{repere}", hexa(p.moyenne), hexa(p.erreur_type))
            } else {
                format!("NaN:NaN:{repere}")
            }
        })
        .collect();
    let encore = if code_final == 0 { format!("0,{fait}") } else { format!("{code_final},{fait}") };
    let hors = ["NaN", "NaN", "NaN", "NaN", "NaN", "NaN"].join(",");
    format!(
        "avant=1\nrefus=1,1,1,2,2,2,2,1\nsans=0,0,1;5\ndemarre=0\ntotal={total}\nsuivants={}\ncodes={}\nfaits={}\nfin={fait},{total},{code_final}\n\
         points={}\nfidele=1\nhors={hors}\nencore={encore}\nefface=0,0,{total},NaN\n",
        suivants.join(","),
        codes.join(","),
        faits.join(","),
        points.join(","),
    )
}

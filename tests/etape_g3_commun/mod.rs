//! Ce que les fichiers de test de l'étape G3 (la neige sur la page) partagent : la table des dix-neuf jeux de réglages nommés de la doc de `vitrine` (« La neige »), **écrite ici à la main en millionièmes
//! d'après la source** (la fiche `references/etape-g3-valeurs.md`, qui renvoie à celles de G1 et de G2 : jamais lue dans la bibliothèque ni dans `vitrine`), la neige native en face (une `Neige` construite directement par la
//! bibliothèque, ses mesures, son étiquette, son image d'un pixel par case et son image nette de la doc), la règle du mur écrite en boucle simple, les chaînes que les scripts de Node doivent écrire, et les aides de lecture
//! du texte de la page (celles de l'étape V7, reprises).
//!
//! Ce fichier n'est pas un test : il ne contient aucune fonction `#[test]`. Il est gelé avec les tests.
#![allow(dead_code)]

use std::path::PathBuf;

use givre_lab::balayage::{Forme, Mesures, classer, mesurer};
use givre_lab::empreinte::sha256_hex;
use givre_lab::flocon::lire_decimal;
use givre_lab::math::Fixe;
use givre_lab::neige::{Neige, Parametres};
use givre_lab::rendu_hex::{ReglagesImage, rendre};

/// Les aides de l'étape V7 (qui reprennent celles de D3 et de l'étape 5 : `e5`, `d3`, la lecture du texte de la page, les règles CSS, le lancement de Node).
#[path = "../etape_v7_commun/mod.rs"]
pub mod v7;

pub use v7::e5;

/// Les aides de l'étape G1 (`FIG13_MILIEU` : le jeu du milieu de la figure 13, écrit en décimaux).
#[path = "../etape_g1_commun/mod.rs"]
pub mod g1;

/// Les aides de l'étape G2 (le calcul naïf des six mesures d'un ensemble de cases, l'ensemble des cases rattachées d'une neige).
#[path = "../etape_g2_commun/mod.rs"]
pub mod g2;

// ---------------------------------------------------------------------------------------------------------------
// La table de la doc de `vitrine` : « La neige », les jeux nommés (V-G3-01).
// ---------------------------------------------------------------------------------------------------------------

/// Un jeu de réglages nommé : **écrit à la main en millionièmes** d'après la fiche (V-G3-01 : G2, V-G2-01 à 03 ; G1, V-G1-02).
pub struct JeuAttendu {
    /// Le numéro de la figure de l'article.
    pub figure: u32,
    /// Le rang du cristal dans sa figure (à partir de 1).
    pub rang: u32,
    /// Le nombre de cristaux de la figure.
    pub cristaux: u32,
    /// Le numéro du réglage qui varie (0 `ρ`, 1 `β`, 5 `μ`), 7 s'il n'y en a pas.
    pub balaye: u32,
    /// `ρ β α θ κ μ γ` en millionièmes.
    pub reglages: [u32; 7],
    /// Le rayon du domaine.
    pub rayon: u32,
    /// Le nombre de pas.
    pub pas: u32,
    /// 1 si les réglages sont déduits de l'annexe (la colonne du réglage qui varie y manque), 0 s'ils sont lus.
    pub deduit: u32,
}

const fn jeu(
    figure: u32,
    rang: u32,
    cristaux: u32,
    balaye: u32,
    reglages: [u32; 7],
    pas: u32,
    deduit: u32,
) -> JeuAttendu {
    JeuAttendu { figure, rang, cristaux, balaye, reglages, rayon: 100, pas, deduit }
}

/// Les dix-neuf jeux dans l'ordre de la doc : 0 à 5 la figure 9 (`ρ = 0,40 ; 0,41 ; 0,42 ; 0,44 ; 0,46 ; 0,50`, annexe 8 ligne « 9 » : `β α θ κ μ γ = 1,3 ; 0,08 ; 0,025 ; 0,003 ; 0,07 ; 0,00005`, 2000 pas),
/// 6 à 11 la figure 10 (`β = 1,9 ; 2,2 ; 2,4 ; 2,6 ; 2,7 ; 2,8`, ligne « 10 » : `ρ α θ κ μ γ = 0,8 ; 0,004 ; 0,001 ; 0,05 ; 0,015 ; 0,0001`, 1300 pas, **déduit**), 12 à 17 la figure 12
/// (`μ = 0,04 ; … ; 0,09`, ligne « 12 » : `ρ β α θ κ γ = 0,5 ; 1,4 ; 0,1 ; 0,005 ; 0,001 ; 0,001`, 2000 pas, **déduit**), 18 le milieu de la figure 13 (`ρ β α θ κ μ γ = 0,36 ; 1,09 ; 0,01 ; 0,0745 ; 0,0001 ;
/// 0,14 ; 0,00001`, 3000 pas : un choix à nous). Chaque nombre est `décimal × 10^6`.
pub const JEUX: [JeuAttendu; 19] = [
    jeu(9, 1, 6, 0, [400_000, 1_300_000, 80_000, 25_000, 3_000, 70_000, 50], 2000, 0),
    jeu(9, 2, 6, 0, [410_000, 1_300_000, 80_000, 25_000, 3_000, 70_000, 50], 2000, 0),
    jeu(9, 3, 6, 0, [420_000, 1_300_000, 80_000, 25_000, 3_000, 70_000, 50], 2000, 0),
    jeu(9, 4, 6, 0, [440_000, 1_300_000, 80_000, 25_000, 3_000, 70_000, 50], 2000, 0),
    jeu(9, 5, 6, 0, [460_000, 1_300_000, 80_000, 25_000, 3_000, 70_000, 50], 2000, 0),
    jeu(9, 6, 6, 0, [500_000, 1_300_000, 80_000, 25_000, 3_000, 70_000, 50], 2000, 0),
    jeu(10, 1, 6, 1, [800_000, 1_900_000, 4_000, 1_000, 50_000, 15_000, 100], 1300, 1),
    jeu(10, 2, 6, 1, [800_000, 2_200_000, 4_000, 1_000, 50_000, 15_000, 100], 1300, 1),
    jeu(10, 3, 6, 1, [800_000, 2_400_000, 4_000, 1_000, 50_000, 15_000, 100], 1300, 1),
    jeu(10, 4, 6, 1, [800_000, 2_600_000, 4_000, 1_000, 50_000, 15_000, 100], 1300, 1),
    jeu(10, 5, 6, 1, [800_000, 2_700_000, 4_000, 1_000, 50_000, 15_000, 100], 1300, 1),
    jeu(10, 6, 6, 1, [800_000, 2_800_000, 4_000, 1_000, 50_000, 15_000, 100], 1300, 1),
    jeu(12, 1, 6, 5, [500_000, 1_400_000, 100_000, 5_000, 1_000, 40_000, 1_000], 2000, 1),
    jeu(12, 2, 6, 5, [500_000, 1_400_000, 100_000, 5_000, 1_000, 50_000, 1_000], 2000, 1),
    jeu(12, 3, 6, 5, [500_000, 1_400_000, 100_000, 5_000, 1_000, 60_000, 1_000], 2000, 1),
    jeu(12, 4, 6, 5, [500_000, 1_400_000, 100_000, 5_000, 1_000, 70_000, 1_000], 2000, 1),
    jeu(12, 5, 6, 5, [500_000, 1_400_000, 100_000, 5_000, 1_000, 80_000, 1_000], 2000, 1),
    jeu(12, 6, 6, 5, [500_000, 1_400_000, 100_000, 5_000, 1_000, 90_000, 1_000], 2000, 1),
    jeu(13, 1, 1, 7, [360_000, 1_090_000, 10_000, 74_500, 100, 140_000, 10], 3000, 0),
];

/// Les constantes de la doc, **écrites ici à la main** (V-G3-02) : 19 jeux, la limite du mur `R · 65 / 100` (65 pour `R = 100`), la vue `65 + 4 = 69`, le `plein` 1,5 (`6 442 450 944`).
pub const NEIGE_JEUX: u32 = 19;
pub const LIMITE_MUR: u32 = 65;
pub const RAYON_VUE: u32 = 69;
pub const PLEIN: i64 = 6_442_450_944;
/// `u32::MAX` : « pas de valeur » des lecteurs de la neige.
pub const ABSENT: u32 = u32::MAX;

/// Les numéros des étiquettes : 0 aucune, 1 plaque, 2 secteurs, 3 dendrite, 4 fougère (l'ordre de `balayage::Forme`, écrit ici à la main).
pub fn code_de_la_forme(forme: Forme) -> u32 {
    match forme {
        Forme::Aucune => 0,
        Forme::Plaque => 1,
        Forme::Secteurs => 2,
        Forme::Dendrite => 3,
        Forme::Fougere => 4,
    }
}

/// Le code d'une étiquette écrite dans le CSV de G2 (`"plaque"`, `"secteurs"`, `"dendrite"`, `"fougere"`, `"aucune"`).
pub fn code_du_nom_de_forme(nom: &str) -> u32 {
    ["aucune", "plaque", "secteurs", "dendrite", "fougere"]
        .iter()
        .position(|n| *n == nom)
        .expect("une étiquette connue") as u32
}

// ---------------------------------------------------------------------------------------------------------------
// La neige native.
// ---------------------------------------------------------------------------------------------------------------

/// Un réglage en millionièmes lu comme le fait `Parametres::lire` d'un décimal : par `flocon::lire_decimal` du texte `E.FFFFFF` (**une autre voie que `Fixe::rapport`** que `vitrine` emploie).
pub fn fixe_de(millionniemes: u32) -> Fixe {
    let texte = format!("{}.{:06}", millionniemes / 1_000_000, millionniemes % 1_000_000);
    lire_decimal(&texte).expect("un décimal valide")
}

/// Les sept réglages du jeu `jeu` avec `ρ`, `β`, `κ`, `μ` remplacés (en millionièmes, dans cet ordre).
pub fn parametres_du_jeu(jeu: usize, rho: u32, beta: u32, kappa: u32, mu: u32) -> Parametres {
    let r = &JEUX[jeu].reglages;
    Parametres {
        rho: fixe_de(rho),
        beta: fixe_de(beta),
        alpha: fixe_de(r[2]),
        theta: fixe_de(r[3]),
        kappa: fixe_de(kappa),
        mu: fixe_de(mu),
        gamma: fixe_de(r[6]),
    }
}

/// Les quatre réglages `(ρ, β, κ, μ)` d'un jeu, en millionièmes.
pub fn quatre_du_jeu(jeu: usize) -> [u32; 4] {
    let r = &JEUX[jeu].reglages;
    [r[0], r[1], r[4], r[5]]
}

/// Joue `neige` jusqu'au temps `jusqu_a` (au plus la durée du jeu, `duree`), **en vérifiant avant chaque pas la règle du mur écrite en boucle simple** (V-G3-02 : `rayon_cristal >= 65` : on ne joue pas ce pas) ; rend le nombre de pas joués.
pub fn avancer_la_neige(neige: &mut Neige, jusqu_a: u32, duree: u32) -> u32 {
    let mut joues = 0;
    while neige.temps() < u64::from(jusqu_a.min(duree)) && neige.rayon_cristal() < LIMITE_MUR {
        neige.pas();
        joues += 1;
    }
    joues
}

/// La neige de la bibliothèque seule : le jeu `jeu`, ses quatre réglages remplacés par `quatre` (`ρ, β, κ, μ`), jouée `pas_demandes` pas au plus (au plus la durée du jeu et jusqu'au mur). Rend la neige et le nombre de pas joués.
pub fn neige_jouee(jeu: usize, quatre: [u32; 4], pas_demandes: u32) -> (Neige, u32) {
    let [rho, beta, kappa, mu] = quatre;
    let mut neige =
        Neige::essayer_new(JEUX[jeu].rayon, parametres_du_jeu(jeu, rho, beta, kappa, mu)).expect("une neige valide");
    let joues = avancer_la_neige(&mut neige, pas_demandes, JEUX[jeu].pas);
    (neige, joues)
}

/// Ce que la vitrine doit rendre pour une neige : l'état de la bibliothèque, lu comme `vitrine` le donne.
pub struct EtatNatif {
    pub temps: u32,
    pub duree: u32,
    pub sites: u32,
    pub rayon: u32,
    pub empreinte: String,
    pub largeur: u32,
    pub hauteur: u32,
    /// Les octets R, V, B de l'image d'un pixel par case (`Neige::apercu`).
    pub rvb: Vec<u8>,
    /// Les six mesures (`balayage::mesurer`, **recoupées par le calcul naïf de G2** : `g2::mesures_naives`).
    pub mesures: Mesures,
    /// Le numéro de l'étiquette (`balayage::classer`).
    pub forme: u32,
    /// 1 si le rayon du cristal a atteint 65.
    pub au_mur: u32,
}

/// L'état natif d'une neige déjà jouée (`neige_jouee`), de durée `duree`.
pub fn etat_natif(neige: &Neige, duree: u32) -> EtatNatif {
    let mesures = mesurer(neige);
    assert_eq!(
        mesures,
        g2::mesures_naives(&g2::rattachees_de(neige)),
        "balayage::mesurer et le calcul naïf de G2 ne s'accordent pas"
    );
    let cote = 2 * neige.rayon() + 1;
    EtatNatif {
        temps: u32::try_from(neige.temps()).expect("un temps de u32"),
        duree,
        sites: u32::try_from(neige.cases_rattachees()).expect("des sites de u32"),
        rayon: neige.rayon_cristal(),
        empreinte: neige.empreinte(),
        largeur: cote,
        hauteur: cote,
        rvb: neige.apercu(),
        forme: code_de_la_forme(classer(&mesures)),
        mesures,
        au_mur: u32::from(neige.rayon_cristal() >= LIMITE_MUR),
    }
}

impl EtatNatif {
    /// Les huit mots de l'empreinte (gros-boutistes) : les 64 chiffres hexadécimaux lus par huit.
    pub fn mots(&self) -> [u32; 8] {
        let mut mots = [0_u32; 8];
        for (i, mot) in mots.iter_mut().enumerate() {
            *mot = u32::from_str_radix(&self.empreinte[8 * i..8 * i + 8], 16).expect("des chiffres hexadécimaux");
        }
        mots
    }
}

/// L'image nette attendue de la neige (les octets R, V, B) : `rendu_hex::rendre` appliqué au champ de la doc (`valeurs_du_cristal`), avec la vue de rayon 69 et `plein` 1,5 **écrits à la main** (V-G3-03).
pub fn image_nette_attendue(neige: &Neige, cote: u32, sur: u32) -> Vec<u8> {
    let reglages = ReglagesImage { cote, rayon_vue: RAYON_VUE, sur, plein: PLEIN };
    rendre(neige.rayon(), &neige.valeurs_du_cristal(), &reglages).expect("un rendu valide")
}

/// Les octets RGBA d'une image RVB (le canal alpha à 255).
pub fn rgba_de(rvb: &[u8]) -> Vec<u8> {
    v7::rgba_de(rvb)
}

/// Les mesures dans l'ordre de `givre_neige_mesure` (champs 0 à 7).
pub fn mesures_en_champs(etat: &EtatNatif) -> [u32; 8] {
    let m = &etat.mesures;
    [m.rattachees, m.rayon, m.compacite, m.pointes, m.aiguilles, m.hexagonalite, etat.forme, etat.au_mur]
}

// ---------------------------------------------------------------------------------------------------------------
// Les résultats de G2 : le CSV commité (régression de notre code, calculé par les aperçus de G2 sous Linux et sous Windows).
// ---------------------------------------------------------------------------------------------------------------

/// Une ligne du CSV de G2 (`apercus/etape-g2-mesures.csv`) : le cristal du jeu `jeu` (0 à 17) à la fin de ses pas. `(rattachees, rayon, compacite, pointes, aiguilles, hexagonalite, forme, empreinte)`.
pub struct LigneG2 {
    pub champs: [u32; 6],
    pub forme: u32,
    pub empreinte: String,
}

pub fn ligne_g2(jeu: usize) -> LigneG2 {
    let texte = std::fs::read_to_string(e5::racine().join("apercus").join("etape-g2-mesures.csv"))
        .expect("le CSV de G2 se lit");
    let ligne = texte.lines().nth(1 + jeu).unwrap_or_else(|| panic!("le CSV de G2 n'a pas de ligne pour le jeu {jeu}"));
    let champs: Vec<&str> = ligne.split(',').collect();
    assert_eq!(champs.len(), 12, "une ligne du CSV de G2 a douze champs : {ligne}");
    let nombre = |i: usize| champs[i].parse::<u32>().expect("un entier");
    LigneG2 {
        champs: [nombre(4), nombre(5), nombre(6), nombre(7), nombre(8), nombre(9)],
        forme: code_du_nom_de_forme(champs[10]),
        empreinte: champs[11].to_string(),
    }
}

// ---------------------------------------------------------------------------------------------------------------
// Node.
// ---------------------------------------------------------------------------------------------------------------

pub fn racine() -> PathBuf {
    e5::racine().to_path_buf()
}

/// Lance un script de `tests/etape_g3/` avec ces arguments.
pub fn lancer_le_script(script: &str, arguments: &[&str]) -> std::process::Output {
    std::process::Command::new("node")
        .arg(racine().join("tests").join("etape_g3").join(script))
        .args(arguments)
        .output()
        .unwrap_or_else(|erreur| panic!("le lancement de Node échoue : {erreur}"))
}

pub fn chemin_du_module() -> String {
    e5::chemin_module().to_str().expect("chemin en UTF-8").to_string()
}

pub fn chemin_de_lecture() -> String {
    e5::chemin_de_lecture().to_str().expect("chemin en UTF-8").to_string()
}

/// Un nombre non signé écrit en décimal.
fn t(n: u32) -> String {
    n.to_string()
}

/// La ligne `jeux` de `neige-wasm.mjs` : les dix-neuf jeux lus par `lireJeuxNeige` de `lecture.js`, chacun écrit `figure.rang.cristaux.balaye:ρ,β,α,θ,κ,μ,γ:rayon:pas:déduit`, séparés par des barres verticales.
pub fn attendu_des_jeux() -> String {
    let jeux: Vec<String> = JEUX
        .iter()
        .map(|j| {
            let r: Vec<String> = j.reglages.iter().map(|v| t(*v)).collect();
            format!(
                "{}.{}.{}.{}:{}:{}:{}:{}",
                j.figure,
                j.rang,
                j.cristaux,
                j.balaye,
                r.join(","),
                j.rayon,
                j.pas,
                j.deduit
            )
        })
        .collect();
    jeux.join("|")
}

/// L'état lu par le script : `temps,durée,sites,rayon,empreinte,LxH,sha256(rgba de l'image d'un pixel par case)`.
pub fn ligne_d_etat(etat: &EtatNatif) -> String {
    format!(
        "{},{},{},{},{},{}x{},{}",
        etat.temps,
        etat.duree,
        etat.sites,
        etat.rayon,
        etat.empreinte,
        etat.largeur,
        etat.hauteur,
        sha256_hex(&rgba_de(&etat.rvb))
    )
}

/// Ce que `neige-wasm.mjs` doit écrire pour le jeu `jeu`, les quatre réglages `quatre`, `pas` pas demandés d'un seul appel et l'image nette de `cote` pixels à `sur × sur` échantillons : l'état et l'image du code natif,
/// lus comme le script les lit (son en-tête dit chaque ligne).
pub fn attendu_du_script_neige(jeu: usize, quatre: [u32; 4], pas: u32, cote: u32, sur: u32) -> String {
    let (neige, joues) = neige_jouee(jeu, quatre, pas);
    let etat = etat_natif(&neige, JEUX[jeu].pas);
    let image = image_nette_attendue(&neige, cote, sur);
    let max = ABSENT;
    let [rho, beta, kappa, mu] = quatre;
    let r = &JEUX[jeu].reglages;
    let reglage = [rho, beta, r[2], r[3], kappa, mu, r[6], jeu as u32, JEUX[jeu].rayon, JEUX[jeu].pas];
    let reglage: Vec<String> = reglage.iter().map(|v| t(*v)).collect();
    let mesure: Vec<String> = mesures_en_champs(&etat).iter().map(|v| t(*v)).collect();
    // `givre_avancer(1)` joue une unité si le scénario n'est pas fini (ni la durée ni le mur) : le tampon est alors effacé, sinon il reste.
    let jouee = u32::from(etat.temps < etat.duree && etat.au_mur == 0);
    let efface = if jouee == 1 {
        format!("1,0,0,{max}")
    } else {
        format!("0,{cote},{cote},{}", v7::pixel_de(&image, cote, 0, 0))
    };
    format!(
        "avant=19,9,{max},{max},2\njeux={jeux}\nrefus=1,1,2,3,3,4,5,2\napres_refus={max},{max},2\ndemarre=0\nreglage={reglage}\njoue={joues}\netat={etat}\nmesure={mesure}\nvide=0,0,{max}\ncode=0\n\
         dimensions={cote}x{cote}\nrgba={rgba}\npixels={pixels}\nhors={max},{max},{max},{max},{max}\netat_apres={etat}\nrefus_nette=3,4,4,5\napres_refus_nette=0,0,{max}\nefface={efface}\nredemarre=0,0,0,{max}\nautre=1,{max},{max},9\nhuit=0\n",
        jeux = attendu_des_jeux(),
        reglage = reglage.join(","),
        etat = ligne_d_etat(&etat),
        mesure = mesure.join(","),
        rgba = sha256_hex(&rgba_de(&image)),
        pixels = sha256_hex(&image),
    )
}

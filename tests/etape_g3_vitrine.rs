//! Étape G3 : la vitrine côté natif, la neige de Gravner et Griffeath par les cinq fonctions `givre_neige_*` et par les fonctions qui existent déjà. Valeurs attendues : `references/etape-g3-valeurs.md`, identifiants `V-G3-xx`.
//!
//! Comme `etape_v7_vitrine.rs`, on appelle les fonctions `givre_*` comme des fonctions Rust et on les compare à ce que la bibliothèque donne **seule** : une `Neige` construite en face (`etape_g3_commun`), ses mesures (recoupées par le
//! calcul naïf de G2), son image d'un pixel par case et son image nette de la doc (`rendu_hex::rendre` appliqué à `Neige::valeurs_du_cristal`, la vue 69 et le `plein` 1,5 **écrits à la main**). Les dix-neuf jeux nommés sont
//! **écrits à la main en millionièmes** d'après la source (`etape_g3_commun::JEUX`) et comparés à `balayage` (une seule source). Le module WebAssembly est éprouvé dans `etape_g3_module.rs` (sous Node), la page dans
//! `etape_g3_page.rs`, l'absence de tout démarrage dans `etape_g3_neuf.rs`.
//!
//! L'état de la vitrine est global au processus et les tests d'un fichier tournent en parallèle : chaque test prend le verrou `VERROU` et le garde jusqu'à la fin. Aucun test de ce fichier n'a de tolérance : tout est une égalité exacte.
//!
//! Tant qu'une des cinq fonctions `givre_neige_*` de `src/vitrine.rs` a encore son corps `todo!()`, ce binaire de test **s'arrête d'un coup** au lieu de marquer un test en échec : une panique ne peut pas traverser une `extern "C" fn`
//! (« panic in a function that cannot unwind », `SIGABRT` sous Linux, `0xc0000409` sous Windows). C'est un échec, et la ligne citée dit quelle fonction reste à écrire ; les autres fichiers de test continuent avec `--no-fail-fast`.

mod etape_g3_commun;

use std::sync::{Mutex, MutexGuard};

use etape_g3_commun::{
    ABSENT, EtatNatif, JEUX, LIMITE_MUR, NEIGE_JEUX, avancer_la_neige, code_de_la_forme, e5, etat_natif, fixe_de, g1,
    g2, image_nette_attendue, ligne_g2, mesures_en_champs, neige_jouee, parametres_du_jeu, quatre_du_jeu, v7,
};
use givre_lab::balayage::{classer, jeu_beta, jeu_mu, jeu_rho, mesurer};
use givre_lab::flocon::lire_decimal;
use givre_lab::math::Fixe;
use givre_lab::neige::Neige;
use givre_lab::vitrine::{
    NEIGE_JEUX as NEIGE_JEUX_DU_MODULE, givre_avancer, givre_compacite, givre_correlation_rayon,
    givre_correlation_valeur, givre_demarrer, givre_dimension, givre_duree, givre_empreinte_mot, givre_hauteur,
    givre_largeur, givre_mesurer, givre_neige_demarrer, givre_neige_jeu, givre_neige_jeux, givre_neige_mesure,
    givre_neige_reglage, givre_nette_hauteur, givre_nette_largeur, givre_nette_pixel, givre_nette_preparer,
    givre_pixel, givre_rayon, givre_sites, givre_temps,
};

static VERROU: Mutex<()> = Mutex::new(());

/// Prend le verrou de la vitrine ; un test qui a paniqué en le tenant ne bloque pas les suivants.
fn verrou() -> MutexGuard<'static, ()> {
    VERROU.lock().unwrap_or_else(|empoisonne| empoisonne.into_inner())
}

/// Démarre le jeu `jeu` avec ses quatre réglages `quatre` (`ρ, β, κ, μ`) : le code que rend le module.
fn demarrer(jeu: usize, quatre: [u32; 4]) -> u32 {
    givre_neige_demarrer(jeu as u32, quatre[0], quatre[1], quatre[2], quatre[3])
}

/// Les huit mots de l'empreinte de la vitrine, chacun sur 8 chiffres hexadécimaux, dans l'ordre.
fn empreinte_de_la_vitrine() -> String {
    (0..8).map(|i| format!("{:08x}", givre_empreinte_mot(i))).collect()
}

/// Les octets R, V, B de l'image d'un pixel par case (`givre_pixel`), lus un par un.
fn image_d_un_pixel_par_case() -> (u32, u32, Vec<u8>) {
    let (largeur, hauteur) = (givre_largeur(), givre_hauteur());
    let mut rvb = Vec::with_capacity(3 * largeur as usize * hauteur as usize);
    for y in 0..hauteur {
        for x in 0..largeur {
            let couleur = givre_pixel(x, y);
            rvb.extend_from_slice(&[(couleur >> 16) as u8, (couleur >> 8) as u8, couleur as u8]);
        }
    }
    (largeur, hauteur, rvb)
}

/// Les octets R, V, B du tampon de l'image nette, lus par `givre_nette_pixel` : `(largeur, hauteur, octets)`. Aucun pixel du tampon n'est `u32::MAX`.
fn lire_le_tampon(contexte: &str) -> (u32, u32, Vec<u8>) {
    let (largeur, hauteur) = (givre_nette_largeur(), givre_nette_hauteur());
    let mut rvb = Vec::with_capacity(3 * largeur as usize * hauteur as usize);
    for y in 0..hauteur {
        for x in 0..largeur {
            let couleur = givre_nette_pixel(x, y);
            assert_ne!(couleur, ABSENT, "{contexte} : le pixel ({x}, {y}) du tampon est hors image");
            assert!(
                couleur <= 0x00FF_FFFF,
                "{contexte} : le pixel ({x}, {y}) n'est pas empaqueté en 0x00RRGGBB : {couleur:#x}"
            );
            rvb.extend_from_slice(&[(couleur >> 16) as u8, (couleur >> 8) as u8, couleur as u8]);
        }
    }
    (largeur, hauteur, rvb)
}

/// Le tampon est vide : largeur et hauteur 0, et tout pixel vaut `u32::MAX`.
fn exiger_le_tampon_vide(contexte: &str) {
    assert_eq!(givre_nette_largeur(), 0, "{contexte} : largeur du tampon");
    assert_eq!(givre_nette_hauteur(), 0, "{contexte} : hauteur du tampon");
    for (x, y) in [(0, 0), (1, 0), (0, 1), (10, 10), (61, 61), (u32::MAX, u32::MAX)] {
        assert_eq!(givre_nette_pixel(x, y), ABSENT, "{contexte} : le pixel ({x}, {y}) d'un tampon vide");
    }
}

/// Exige que le tampon soit l'image nette de `cote` pixels : `attendu` (les octets de `rendre`), dimensions comprises.
fn exiger_le_tampon(attendu: &[u8], cote: u32, contexte: &str) {
    let (largeur, hauteur, rvb) = lire_le_tampon(contexte);
    assert_eq!((largeur, hauteur), (cote, cote), "{contexte} : dimensions");
    assert!(rvb == attendu, "{contexte} : l'image n'est pas celle de rendu_hex::rendre appliqué au champ de la doc");
    assert_eq!(givre_nette_pixel(cote, 0), ABSENT, "{contexte} : x = largeur est hors image");
    assert_eq!(givre_nette_pixel(0, cote), ABSENT, "{contexte} : y = hauteur est hors image");
}

/// Exige que la vitrine soit dans l'état natif `etat` d'une neige démarrée par le jeu `jeu` avec `quatre` : temps, durée, sites, rayon, compacité `NaN`, les huit mots de l'empreinte et l'empreinte entière, **chaque pixel de
/// l'image d'un pixel par case**, les dix champs des réglages, les huit champs des mesures, et `givre_mesurer` (0 : pas de mesure de corrélation). Rien de ce que cette fonction lit ne change l'état ni le tampon.
fn exiger_la_neige(etat: &EtatNatif, jeu: usize, quatre: [u32; 4], contexte: &str) {
    assert_eq!(givre_temps(), etat.temps, "{contexte} : temps");
    assert_eq!(givre_duree(), etat.duree, "{contexte} : durée");
    assert_eq!(givre_sites(), etat.sites, "{contexte} : sites");
    assert_eq!(givre_rayon(), etat.rayon, "{contexte} : rayon");
    assert!(givre_compacite().is_nan(), "{contexte} : la compacité d'une neige est NaN");
    // Les huit mots de l'empreinte (huit SHA-256 de tout l'état : lus une seule fois), puis l'empreinte entière qu'ils écrivent.
    let mots: Vec<u32> = (0..8).map(|i| givre_empreinte_mot(i)).collect();
    assert_eq!(mots, etat.mots(), "{contexte} : les huit mots de l'empreinte");
    assert_eq!(
        mots.iter().map(|mot| format!("{mot:08x}")).collect::<String>(),
        etat.empreinte,
        "{contexte} : empreinte"
    );
    let (largeur, hauteur, rvb) = image_d_un_pixel_par_case();
    assert_eq!(
        (largeur, hauteur),
        (etat.largeur, etat.hauteur),
        "{contexte} : dimensions de l'image d'un pixel par case"
    );
    assert!(rvb == etat.rvb, "{contexte} : l'image d'un pixel par case n'est pas l'aperçu de la bibliothèque");
    let r = &JEUX[jeu].reglages;
    let reglage =
        [quatre[0], quatre[1], r[2], r[3], quatre[2], quatre[3], r[6], jeu as u32, JEUX[jeu].rayon, JEUX[jeu].pas];
    for (champ, attendu) in reglage.iter().enumerate() {
        assert_eq!(givre_neige_reglage(champ as u32), *attendu, "{contexte} : réglage {champ}");
    }
    for (champ, attendu) in mesures_en_champs(etat).iter().enumerate() {
        assert_eq!(givre_neige_mesure(champ as u32), *attendu, "{contexte} : mesure {champ}");
    }
    assert_eq!(givre_mesurer(), 0, "{contexte} : pas de mesure de corrélation pour une neige");
}

/// Un générateur congruentiel (des graines **fixes** écrites dans les tests : aucun tirage n'est choisi parce qu'il passe).
struct Hasard(u64);

impl Hasard {
    fn suivant(&mut self) -> u64 {
        self.0 = self.0.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1_442_695_040_888_963_407);
        self.0 >> 33
    }

    fn entre(&mut self, bas: u32, haut: u32) -> u32 {
        bas + (self.suivant() % u64::from(haut - bas + 1)) as u32
    }
}

/// juste : les dix-neuf jeux nommés sont ceux de la source (V-G3-01). `NEIGE_JEUX` et `givre_neige_jeux()` valent 19 ; pour chaque jeu de 0 à 18, les quatorze champs de `givre_neige_jeu(jeu, champ)` (la figure, le rang, le nombre
/// de cristaux de la figure, le réglage qui varie, les sept réglages en millionièmes, le rayon du domaine, le nombre de pas, « déduit ») sont ceux de la table **écrite à la main** d'après la source (figures 9, 10 et 12 de Gravner et Griffeath, annexe 8, et
/// le milieu de la figure 13 : la fiche) ; hors de portée (`jeu >= 19`, `champ >= 14`, `u32::MAX` compris) : `u32::MAX`. Lire un jeu ne change rien : un flocon démarré, son tampon prêt, et rien n'est effacé. `ρ` lu pour `β`, deux jeux
/// échangés, un champ décalé, un réglage balayé faux ou un nombre de pas faux se voient.
///
/// Ne couvre pas : que la table soit celle de l'article à la lettre (les valeurs balayées sont lues sur les figures, les autres réglages sur l'annexe dont la colonne balayée est **déduite** pour les figures 10 et 12 : la fiche le dit) ; que
/// ces jeux soient ceux de `balayage` (voir `les_jeux_nommes_ont_une_seule_source`).
#[test]
#[cfg_attr(not(feature = "etape-g3"), ignore)]
fn les_jeux_nommes_sont_ceux_de_la_source() {
    let _verrou = verrou();
    assert_eq!(NEIGE_JEUX_DU_MODULE, NEIGE_JEUX);
    assert_eq!(NEIGE_JEUX, 19);
    assert_eq!(givre_neige_jeux(), 19);
    assert_eq!(givre_demarrer(2, 0), 1);
    assert_eq!(givre_nette_preparer(62, 2), 0);
    for (jeu, attendu) in JEUX.iter().enumerate() {
        let champs = [
            attendu.figure,
            attendu.rang,
            attendu.cristaux,
            attendu.balaye,
            attendu.reglages[0],
            attendu.reglages[1],
            attendu.reglages[2],
            attendu.reglages[3],
            attendu.reglages[4],
            attendu.reglages[5],
            attendu.reglages[6],
            attendu.rayon,
            attendu.pas,
            attendu.deduit,
        ];
        for (champ, valeur) in champs.iter().enumerate() {
            assert_eq!(givre_neige_jeu(jeu as u32, champ as u32), *valeur, "jeu {jeu}, champ {champ}");
        }
    }
    for jeu in [19_u32, 20, 100, 4096, u32::MAX] {
        for champ in 0..16_u32 {
            assert_eq!(givre_neige_jeu(jeu, champ), ABSENT, "jeu {jeu} (hors de portée), champ {champ}");
        }
    }
    for jeu in 0..19_u32 {
        for champ in [14_u32, 15, 100, u32::MAX] {
            assert_eq!(givre_neige_jeu(jeu, champ), ABSENT, "jeu {jeu}, champ {champ} (hors de portée)");
        }
    }
    // Lire un jeu ne change rien : le tampon de l'image du flocon est encore là.
    assert_eq!(givre_nette_largeur(), 62);
    assert_eq!(givre_temps(), 0);
    assert_eq!(givre_neige_reglage(7), ABSENT, "un flocon n'est pas une neige");
}

/// juste : les jeux nommés n'ont qu'une source, `balayage` (V-G3-02). Pour chaque cristal des trois jeux de `balayage` (`jeu_rho`, `jeu_beta`, `jeu_mu`, dans cet ordre : les jeux 0 à 17), les sept réglages de `reglage.fixer(base, valeurs[rang])` convertis
/// en millionièmes par `ppm = floor((2 000 000 · brut + 2^32) / 2^33)`, le rayon du domaine et le nombre de pas sont ceux de `givre_neige_jeu` ; le jeu 18 est celui de `g1::FIG13_MILIEU` (le jeu du milieu de la figure 13 de G1) avec le rayon 100
/// et 3000 pas. Et la conversion est un aller-retour exact : `fixe_de(ppm)` (la lecture de G1 du décimal) redonne le `brut` de chaque réglage. Un jeu recopié avec une autre valeur, ou lu dans le désordre, se voit.
///
/// Ne couvre pas : la valeur de ces jeux par rapport à l'article (voir `les_jeux_nommes_sont_ceux_de_la_source` et la fiche).
#[test]
#[cfg_attr(not(feature = "etape-g3"), ignore)]
fn les_jeux_nommes_ont_une_seule_source() {
    let _verrou = verrou();
    let ppm = |valeur: Fixe| {
        u32::try_from((2_000_000 * i128::from(valeur.0) + (1_i128 << 32)) / (1_i128 << 33)).expect("un u32")
    };
    for (figure, balayage) in [(0_usize, jeu_rho()), (1, jeu_beta()), (2, jeu_mu())] {
        for (rang, valeur) in balayage.valeurs.iter().enumerate() {
            let jeu = 6 * figure + rang;
            let p = balayage.reglage.fixer(balayage.base, *valeur);
            let sept = [p.rho, p.beta, p.alpha, p.theta, p.kappa, p.mu, p.gamma];
            for (numero, reglage) in sept.iter().enumerate() {
                assert_eq!(
                    givre_neige_jeu(jeu as u32, 4 + numero as u32),
                    ppm(*reglage),
                    "jeu {jeu}, réglage {numero}"
                );
                assert_eq!(
                    fixe_de(ppm(*reglage)),
                    *reglage,
                    "jeu {jeu}, réglage {numero} : l'aller-retour des millionièmes"
                );
            }
            assert_eq!(givre_neige_jeu(jeu as u32, 11), balayage.rayon, "jeu {jeu} : rayon");
            assert_eq!(u64::from(givre_neige_jeu(jeu as u32, 12)), balayage.pas, "jeu {jeu} : pas");
        }
    }
    let milieu = g1::parametres(
        g1::FIG13_MILIEU[0],
        g1::FIG13_MILIEU[1],
        g1::FIG13_MILIEU[2],
        g1::FIG13_MILIEU[3],
        g1::FIG13_MILIEU[4],
        g1::FIG13_MILIEU[5],
        g1::FIG13_MILIEU[6],
    );
    let sept = [milieu.rho, milieu.beta, milieu.alpha, milieu.theta, milieu.kappa, milieu.mu, milieu.gamma];
    for (numero, reglage) in sept.iter().enumerate() {
        assert_eq!(givre_neige_jeu(18, 4 + numero as u32), ppm(*reglage), "jeu 18, réglage {numero}");
    }
    assert_eq!((givre_neige_jeu(18, 11), givre_neige_jeu(18, 12)), (100, 3000));
}

/// juste : la conversion des réglages en `Fixe` (V-G3-03). `ρ`, `β`, `κ` et `μ` passent en millionièmes ; le module les lit comme `Fixe::rapport(ppm, 1_000_000)`, **au plus proche** : l'état d'une neige démarrée avec ces valeurs est, au temps 0
/// (l'écriture canonique de G1 porte les sept `brut`), celui d'une `Neige` de la bibliothèque dont les réglages sont lus par `flocon::lire_decimal` du même décimal (`ppm` écrit `E.FFFFFF`) ; les `brut` attendus de quelques valeurs sont **calculés à la main**
/// (`ppm × 2^32 / 10^6` arrondi : 1 donne 4295 et non 4294, 3 donne 12885, 50 donne 214748, 400 000 donne 1 717 986 918, 500 000 donne 2 147 483 648, 999 999 donne 4 294 963 001, 1 000 000 donne 4 294 967 296) ; et la lecture inverse
/// (`givre_neige_reglage`) rend les millionièmes passés. Les valeurs : toutes les bornes et des valeurs que l'arrondi sépare du plancher. Une conversion qui tronque, ou qui arrondit autrement, se voit.
///
/// Ne couvre pas : les réglages hors plage (voir `les_codes_du_demarrage_et_leur_ordre`) ; la dynamique (voir `une_neige_demarree_est_celle_de_la_bibliotheque`).
#[test]
#[cfg_attr(not(feature = "etape-g3"), ignore)]
fn la_conversion_des_reglages_en_fixe() {
    let _verrou = verrou();
    for (ppm, brut) in [
        (1_u32, 4295_i64),
        (3, 12_885),
        (50, 214_748),
        (400_000, 1_717_986_918),
        (500_000, 2_147_483_648),
        (999_999, 4_294_963_001),
        (1_000_000, 4_294_967_296),
    ] {
        assert_eq!(fixe_de(ppm).0, brut, "le brut de {ppm} millionièmes, calculé à la main");
    }
    let valeurs_de_rho_kappa_mu = [0_u32, 1, 3, 50, 123_457, 400_000, 999_999, 1_000_000];
    let valeurs_de_beta = [1_000_000_u32, 1_000_001, 1_090_000, 2_999_999, 7_999_999, 8_000_000];
    let valide = [500_000_u32, 1_500_000, 5_000, 50_000];
    let mut cas: Vec<[u32; 4]> = Vec::new();
    for &v in &valeurs_de_rho_kappa_mu {
        cas.push([v, valide[1], valide[2], valide[3]]);
        cas.push([valide[0], valide[1], v, valide[3]]);
        cas.push([valide[0], valide[1], valide[2], v]);
    }
    for &v in &valeurs_de_beta {
        cas.push([valide[0], v, valide[2], valide[3]]);
    }
    for quatre in cas {
        let jeu = 4;
        assert_eq!(demarrer(jeu, quatre), 0, "{quatre:?}");
        let (neige, _) = neige_jouee(jeu, quatre, 0);
        assert_eq!(empreinte_de_la_vitrine(), neige.empreinte(), "{quatre:?} : l'état au temps 0");
        assert_eq!(givre_neige_reglage(0), quatre[0], "{quatre:?} : ρ relu");
        assert_eq!(givre_neige_reglage(1), quatre[1], "{quatre:?} : β relu");
        assert_eq!(givre_neige_reglage(4), quatre[2], "{quatre:?} : κ relu");
        assert_eq!(givre_neige_reglage(5), quatre[3], "{quatre:?} : μ relu");
        // Les trois autres réglages sont ceux du jeu.
        assert_eq!(givre_neige_reglage(2), JEUX[jeu].reglages[2]);
        assert_eq!(givre_neige_reglage(3), JEUX[jeu].reglages[3]);
        assert_eq!(givre_neige_reglage(6), JEUX[jeu].reglages[6]);
    }
    // Les décimaux que G1 lit exactement : le milieu de la figure 13 par `lire_decimal` redonne les millionièmes de la table.
    assert_eq!(lire_decimal("0.0745").map(|v| v.0), Some(fixe_de(74_500).0));
}

/// juste : les codes de `givre_neige_demarrer`, leurs bornes et leur ordre (V-G3-04). Le code 0 pour toute valeur de la plage, **bornes incluses** (`ρ`, `κ`, `μ` de 0 à 1 000 000 ; `β` de 1 000 000 à 8 000 000) ; 1 pour un jeu hors de 0 à 18 (vérifié
/// **avant** les réglages, même fautifs) ; puis le premier réglage fautif dans l'ordre `ρ β κ μ` : 2 pour `ρ` (1 000 001, `u32::MAX`), 3 pour `β` (999 999, 0, 8 000 001, `u32::MAX`), 4 pour `κ`, 5 pour `μ` ; deux réglages fautifs donnent le code
/// du premier (`ρ` et `β` : 2 ; `β` et `κ` : 3 ; `κ` et `μ` : 4 ; les quatre : 2). **Un refus ne change rien** : ni l'état ni le tampon de l'image nette, ni quand le modèle en cours est une neige (jeu 7, 20 pas), ni quand c'est un flocon
/// dont le tampon est prêt ; et la neige est démarrée quand le code est 0 (réglages relus, temps 0). Un curseur hors plage accepté, un code échangé ou un ordre inversé se voient.
///
/// Ne couvre pas : l'absence de tout démarrage (voir `etape_g3_neuf.rs`) ; que les valeurs refusées soient celles de `Parametres::verifier` pour `α`, `θ` et `γ` (le module n'en passe aucun).
#[test]
#[cfg_attr(not(feature = "etape-g3"), ignore)]
fn les_codes_du_demarrage_et_leur_ordre() {
    let _verrou = verrou();
    let v = quatre_du_jeu(0);
    let (rho, beta, kappa, mu) = (v[0], v[1], v[2], v[3]);
    let refus: [(u32, [u32; 4], u32); 24] = [
        (19, v, 1),
        (u32::MAX, v, 1),
        (100, [u32::MAX; 4], 1),
        (19, [1_000_001, 999_999, 1_000_001, 1_000_001], 1),
        (0, [1_000_001, beta, kappa, mu], 2),
        (0, [u32::MAX, beta, kappa, mu], 2),
        (0, [1_000_001, 999_999, 1_000_001, 1_000_001], 2),
        (0, [u32::MAX; 4], 2),
        (0, [rho, 999_999, kappa, mu], 3),
        (0, [rho, 0, kappa, mu], 3),
        (0, [rho, 8_000_001, kappa, mu], 3),
        (0, [rho, u32::MAX, kappa, mu], 3),
        (0, [rho, 999_999, 1_000_001, 1_000_001], 3),
        (0, [rho, beta, 1_000_001, mu], 4),
        (0, [rho, beta, u32::MAX, mu], 4),
        (0, [rho, beta, 1_000_001, 1_000_001], 4),
        (0, [rho, beta, kappa, 1_000_001], 5),
        (0, [rho, beta, kappa, u32::MAX], 5),
        (18, [rho, 8_000_001, kappa, mu], 3),
        (18, [rho, beta, kappa, 4_294_967_295], 5),
        (7, [1_000_001, beta, kappa, mu], 2),
        (7, [rho, beta, 1_000_001, mu], 4),
        (12, [rho, 0, kappa, mu], 3),
        (6, [rho, beta, kappa, 1_000_001], 5),
    ];
    // Un refus ne change rien : d'abord quand une neige est en cours (jeu 7, 20 pas, tampon prêt), puis quand c'est un flocon dont le tampon est prêt.
    let quatre_en_cours = quatre_du_jeu(7);
    assert_eq!(demarrer(7, quatre_en_cours), 0);
    assert_eq!(givre_avancer(20), 20);
    assert_eq!(givre_nette_preparer(62, 2), 0);
    let (neige, _) = neige_jouee(7, quatre_en_cours, 20);
    let etat = etat_natif(&neige, JEUX[7].pas);
    let attendue = image_nette_attendue(&neige, 62, 2);
    for (jeu, quatre, code) in refus {
        assert_eq!(demarrer_hors_plage(jeu, quatre), code, "jeu {jeu}, réglages {quatre:?}");
        assert_eq!(givre_temps(), 20, "jeu {jeu}, {quatre:?} : le temps après un refus");
        assert_eq!(givre_neige_reglage(7), 7, "jeu {jeu}, {quatre:?} : la neige en cours après un refus");
        assert_eq!(givre_sites(), etat.sites);
        assert_eq!(givre_nette_largeur(), 62, "jeu {jeu}, {quatre:?} : le tampon après un refus");
    }
    exiger_la_neige(&etat, 7, quatre_en_cours, "après les refus");
    exiger_le_tampon(&attendue, 62, "après les refus");

    assert_eq!(givre_demarrer(2, 0), 1);
    assert_eq!(givre_avancer(10), 10);
    assert_eq!(givre_nette_preparer(62, 2), 0);
    for (jeu, quatre, code) in refus {
        assert_eq!(demarrer_hors_plage(jeu, quatre), code, "flocon en cours, jeu {jeu}, réglages {quatre:?}");
        assert_eq!(givre_neige_reglage(7), ABSENT, "un refus ne démarre pas de neige");
        assert_eq!(givre_temps(), 10);
        assert_eq!(givre_nette_largeur(), 62, "flocon en cours : le tampon après un refus");
    }
    exiger_le_tampon(&e5_image_du_flocon_2_au_pas_10(), 62, "le flocon après les refus");

    // Les bornes incluses démarrent.
    for (jeu, quatre) in [
        (0, [0, 1_000_000, 0, 0]),
        (0, [1_000_000, 8_000_000, 1_000_000, 1_000_000]),
        (18, [1_000_000, 1_000_000, 1_000_000, 0]),
        (12, [0, 8_000_000, 0, 1_000_000]),
    ] {
        assert_eq!(demarrer(jeu, quatre), 0, "jeu {jeu}, réglages {quatre:?} : les bornes sont incluses");
        assert_eq!(givre_temps(), 0);
        assert_eq!(givre_neige_reglage(7), jeu as u32);
        assert_eq!((givre_neige_reglage(0), givre_neige_reglage(1)), (quatre[0], quatre[1]));
        assert_eq!((givre_neige_reglage(4), givre_neige_reglage(5)), (quatre[2], quatre[3]));
    }
}

/// Le code de `givre_neige_demarrer` pour un jeu ou des réglages qu'on veut hors de portée (un nom qui dit le but de l'appel).
fn demarrer_hors_plage(jeu: u32, quatre: [u32; 4]) -> u32 {
    givre_neige_demarrer(jeu, quatre[0], quatre[1], quatre[2], quatre[3])
}

/// L'image nette `(62, 2)` du flocon 2 (l'étoile) au pas 10, de `rendu_hex::rendre` appliqué au champ de la doc de V7 recalculé à la main.
fn e5_image_du_flocon_2_au_pas_10() -> Vec<u8> {
    g3_v7_image_attendue(1, 10, 62, 2)
}

fn g3_v7_image_attendue(indice: usize, pas: u32, cote: u32, sur: u32) -> Vec<u8> {
    etape_g3_commun::v7::image_attendue(indice, pas, cote, sur)
}

/// juste : une neige démarrée est celle de la bibliothèque, pour chacun des dix-neuf jeux nommés (V-G3-05). Pour chaque jeu, démarré avec **ses propres quatre réglages** (champs 4, 5, 8 et 9), aux temps 0, 1 et 45 (joués par appels de 1 puis 44),
/// l'état de la vitrine est celui d'une `Neige` construite par la bibliothèque seule : temps, durée (le nombre de pas du jeu), sites (les cases rattachées), rayon, compacité `NaN`, les huit mots de l'empreinte et l'empreinte (celle de G1), **chaque
/// pixel de l'image d'un pixel par case** (celle de `Neige::apercu`, `201 × 201`), les dix champs des réglages, les huit champs des mesures (`balayage::mesurer`, recoupé par le calcul naïf de G2, et `balayage::classer`), `givre_mesurer` à 0.
/// Un jeu pour un autre, `ρ` lu pour `β`, `κ` pour `μ`, une mesure ou une étiquette lues dans le mauvais ordre, l'image du flocon, un temps décalé ou une durée fausse se voient.
///
/// Ne couvre pas : des réglages tirés (voir `des_reglages_tires_sont_ceux_de_la_bibliotheque`) ; les grands nombres de pas (voir `jusqu_au_bout_comme_g2_et_la_bibliotheque`) ; l'image nette (voir `l_image_nette_de_la_neige`).
#[test]
#[cfg_attr(not(feature = "etape-g3"), ignore)]
fn une_neige_demarree_est_celle_de_la_bibliotheque() {
    let _verrou = verrou();
    for (jeu, attendu) in JEUX.iter().enumerate() {
        let quatre = quatre_du_jeu(jeu);
        assert_eq!(demarrer(jeu, quatre), 0, "jeu {jeu}");
        let (mut neige, _) = neige_jouee(jeu, quatre, 0);
        exiger_la_neige(&etat_natif(&neige, attendu.pas), jeu, quatre, &format!("jeu {jeu}, temps 0"));
        assert_eq!(givre_avancer(1), 1);
        avancer_la_neige(&mut neige, 1, attendu.pas);
        exiger_la_neige(&etat_natif(&neige, attendu.pas), jeu, quatre, &format!("jeu {jeu}, temps 1"));
        assert_eq!(givre_avancer(44), 44);
        avancer_la_neige(&mut neige, 45, attendu.pas);
        exiger_la_neige(&etat_natif(&neige, attendu.pas), jeu, quatre, &format!("jeu {jeu}, temps 45"));
    }
}

/// juste : des réglages tirés donnent la neige de la bibliothèque (V-G3-05). Douze tirages (un générateur congruentiel de graine fixe écrite dans le test : aucun n'est choisi parce qu'il passe) du jeu et des quatre réglages sur la
/// plage des curseurs de la page (`ρ` de 0 à 1 000 000, `β` de 1 000 000 à 4 000 000, `κ` de 0 à 100 000, `μ` de 0 à 200 000, en millionièmes), plus quatre cas aux bornes ; chacun est joué 25 puis 55 pas : l'état de la vitrine est celui de la
/// bibliothèque à chaque temps (temps, durée, sites, rayon, empreinte, image d'un pixel par case, réglages, mesures, étiquette). Les réglages **ne sont pas ceux du jeu** : `ρ`, `β`, `κ`, `μ` sont ceux qu'on a passés, `α`, `θ`, `γ` ceux du jeu.
/// Les quatre valeurs permutées, un réglage du jeu gardé à la place du réglage passé, ou `α`, `θ`, `γ` pris d'un autre jeu se voient.
///
/// Ne couvre pas : la neige jusqu'au bout (voir `jusqu_au_bout_comme_g2_et_la_bibliotheque`) ; un tirage qui atteindrait le mur (voir `le_mur_arrete_la_neige`).
#[test]
#[cfg_attr(not(feature = "etape-g3"), ignore)]
fn des_reglages_tires_sont_ceux_de_la_bibliotheque() {
    let _verrou = verrou();
    let mut hasard = Hasard(20_251_111);
    let mut cas: Vec<(usize, [u32; 4])> = Vec::new();
    for _ in 0..12 {
        let jeu = hasard.entre(0, 18) as usize;
        let quatre = [
            hasard.entre(0, 1_000_000),
            hasard.entre(1_000_000, 4_000_000),
            hasard.entre(0, 100_000),
            hasard.entre(0, 200_000),
        ];
        cas.push((jeu, quatre));
    }
    cas.extend([
        (3, [0, 1_000_000, 0, 0]),
        (9, [1_000_000, 4_000_000, 100_000, 200_000]),
        (14, [500_000, 1_000_000, 100_000, 0]),
        (18, [0, 4_000_000, 0, 200_000]),
    ]);
    for (jeu, quatre) in cas {
        assert_eq!(demarrer(jeu, quatre), 0, "jeu {jeu}, {quatre:?}");
        let (mut neige, _) = neige_jouee(jeu, quatre, 0);
        for jusqu_a in [25_u32, 55] {
            let a_jouer = jusqu_a - u32::try_from(neige.temps()).expect("un temps de u32");
            assert_eq!(
                givre_avancer(a_jouer),
                avancer_la_neige(&mut neige, jusqu_a, JEUX[jeu].pas),
                "jeu {jeu}, {quatre:?}"
            );
            exiger_la_neige(
                &etat_natif(&neige, JEUX[jeu].pas),
                jeu,
                quatre,
                &format!("jeu {jeu}, {quatre:?}, temps {jusqu_a}"),
            );
        }
    }
}

/// régression + juste : quatre jeux jusqu'au bout, en plusieurs appels (V-G3-06). Les jeux 5 (figure 9, `ρ = 0,50`, 2000 pas), 8 (figure 10, `β = 2,4`, 1300 pas) et 17 (figure 12, `μ = 0,09`, 2000 pas) joués par morceaux de 1, 99, 400, 500 puis 1500 : à
/// la fin, `givre_avancer(1)` rend 0, le temps est la durée du jeu, la neige n'est pas au mur, et les six mesures, l'étiquette et **l'empreinte** sont celles de la ligne du jeu dans `apercus/etape-g2-mesures.csv` (le tableau que G2 a calculé sous Linux
/// et sous Windows : un résultat figé de **notre** code, d'où « régression » : la même neige que le tableau de G2, à l'octet de l'empreinte). Le jeu 18 (le milieu de la figure 13, 3000 pas) est comparé à la bibliothèque seule (le calcul est refait dans le test : état, mesures
/// recoupées par le calcul naïf de G2, image d'un pixel par case). Un découpage qui change la dynamique, une durée fausse, un jeu pour un autre ou un arrêt avant la fin se voient.
///
/// Ne couvre pas : les quatorze autres jeux jusqu'au bout (le temps : 1,5 à 3,7 s chacun en natif ; leurs débuts sont dans `une_neige_demarree_est_celle_de_la_bibliotheque`) ; le mur (voir `le_mur_arrete_la_neige`).
#[test]
#[cfg_attr(not(feature = "etape-g3"), ignore)]
fn jusqu_au_bout_comme_g2_et_la_bibliotheque() {
    let _verrou = verrou();
    for jeu in [5_usize, 8, 17] {
        let quatre = quatre_du_jeu(jeu);
        assert_eq!(demarrer(jeu, quatre), 0);
        let duree = JEUX[jeu].pas;
        let mut joue = 0;
        for morceau in [1_u32, 99, 400, 500, 1500] {
            joue += givre_avancer(morceau);
        }
        assert_eq!(joue, duree, "jeu {jeu} : les pas joués");
        assert_eq!(givre_avancer(1), 0, "jeu {jeu} : le scénario est fini");
        assert_eq!(givre_avancer(1000), 0);
        assert_eq!((givre_temps(), givre_duree()), (duree, duree));
        assert_eq!(givre_neige_mesure(7), 0, "jeu {jeu} : ces cristaux ne touchent pas le mur");
        let g2 = ligne_g2(jeu);
        for (champ, attendu) in g2.champs.iter().enumerate() {
            assert_eq!(givre_neige_mesure(champ as u32), *attendu, "jeu {jeu}, mesure {champ} : le tableau de G2");
        }
        assert_eq!(givre_neige_mesure(6), g2.forme, "jeu {jeu} : l'étiquette du tableau de G2");
        assert_eq!(empreinte_de_la_vitrine(), g2.empreinte, "jeu {jeu} : l'empreinte du tableau de G2");
        assert_eq!(givre_rayon(), g2.champs[1]);
        assert_eq!(givre_sites(), g2.champs[0]);
    }
    let quatre = quatre_du_jeu(18);
    assert_eq!(demarrer(18, quatre), 0);
    assert_eq!(givre_avancer(1), 1);
    assert_eq!(givre_avancer(2999), 2999);
    assert_eq!(givre_avancer(1), 0);
    let (neige, joues) = neige_jouee(18, quatre, 100_000);
    assert_eq!(joues, 3000, "le milieu de la figure 13 ne touche pas le mur en 3000 pas");
    exiger_la_neige(&etat_natif(&neige, 3000), 18, quatre, "jeu 18, jusqu'au bout");
}

/// juste : le mur arrête la neige (V-G3-07). La règle de la doc écrite en boucle simple dans le test (`etape_g3_commun::avancer_la_neige` : **avant chaque pas**, si `rayon_cristal >= 65` on ne joue pas ce pas) donne, pour trois réglages
/// libres de neige rapide (le jeu 1 avec `ρ = 0,956862`…, le jeu 5 avec `ρ = 0,700218`…, le jeu 0 avec `ρ` et `β` au plus fort de la vapeur), un nombre de pas `n` inférieur à la durée et un rayon de cristal **exactement 65** (le rayon croît d'au plus 1 par pas).
/// Pour chacun, `givre_avancer(100000)` rend `n`, l'état est celui de la bibliothèque à `n` pas (empreinte, image d'un pixel par case, mesures), le rayon vaut 65, `givre_neige_mesure(7)` vaut 1, `givre_avancer(1)` et `givre_avancer(1000)` rendent 0 et ne
/// changent rien (le temps, l'empreinte) ; **le découpage ne change rien** : joué par appels de 7 pas jusqu'à ce que `givre_avancer` rende 0, le total est `n` et l'état est le même ; et l'image nette `(96, 2)` du cristal au mur est celle de la doc (la vue de
/// rayon 69 le contient entier). Une limite fausse (`>` pour `>=`, 64, 66, la limite sur le diamètre), un arrêt avant ou après le pas, une vérification qui saute un lot, ou un mur qui n'arrête pas la neige se voient.
///
/// Ne couvre pas : que la neige au mur soit encore fidèle à l'article (le mur est à nous : `MODELE.md`) ; la neige qui finit par sa durée (voir `jusqu_au_bout_comme_g2_et_la_bibliotheque`).
#[test]
#[cfg_attr(not(feature = "etape-g3"), ignore)]
fn le_mur_arrete_la_neige() {
    let _verrou = verrou();
    let cas: [(usize, [u32; 4]); 3] = [
        (1, [956_862, 1_122_299, 48_396, 36_667]),
        (5, [700_218, 1_124_914, 73_360, 31_067]),
        (0, [1_000_000, 1_000_000, 0, 0]),
    ];
    for (jeu, quatre) in cas {
        let (neige, n) = neige_jouee(jeu, quatre, 100_000);
        assert!(n < JEUX[jeu].pas, "jeu {jeu}, {quatre:?} : la neige atteint le mur avant la durée ({n} pas)");
        assert_eq!(
            neige.rayon_cristal(),
            LIMITE_MUR,
            "jeu {jeu}, {quatre:?} : le rayon au mur (la doc : exactement 65)"
        );
        let etat = etat_natif(&neige, JEUX[jeu].pas);
        assert_eq!(etat.au_mur, 1);

        assert_eq!(demarrer(jeu, quatre), 0);
        assert_eq!(givre_avancer(100_000), n, "jeu {jeu}, {quatre:?} : les pas joués jusqu'au mur");
        exiger_la_neige(&etat, jeu, quatre, &format!("jeu {jeu}, {quatre:?}, au mur"));
        assert_eq!(givre_rayon(), 65);
        assert_eq!(givre_neige_mesure(7), 1);
        assert_eq!(givre_avancer(1), 0, "au mur, plus rien ne se joue");
        assert_eq!(givre_avancer(1000), 0);
        assert_eq!(givre_temps(), n);
        assert_eq!(empreinte_de_la_vitrine(), etat.empreinte, "au mur, l'état ne bouge plus");
        assert_eq!(givre_nette_preparer(96, 2), 0);
        exiger_le_tampon(&image_nette_attendue(&neige, 96, 2), 96, &format!("jeu {jeu} : l'image nette au mur"));
        assert_eq!(givre_avancer(1), 0, "un appel qui ne joue rien laisse le tampon");
        exiger_le_tampon(&image_nette_attendue(&neige, 96, 2), 96, "le tampon après un givre_avancer qui ne joue rien");

        // Le même scénario par appels de 7 pas : le même état et le même nombre de pas.
        assert_eq!(demarrer(jeu, quatre), 0);
        let mut total = 0;
        loop {
            let joue = givre_avancer(7);
            if joue == 0 {
                break;
            }
            assert!(joue <= 7);
            total += joue;
        }
        assert_eq!(total, n, "jeu {jeu}, {quatre:?} : par appels de 7 pas");
        exiger_la_neige(&etat, jeu, quatre, &format!("jeu {jeu}, {quatre:?}, au mur, par appels de 7"));
    }
}

/// juste : l'image nette de la neige est celle de la doc (V-G3-08). Pour trois jeux (3 : figure 9, 6 : figure 10, 18 : figure 13) à 150 pas, `(62, 2)`, `(96, 3)`, `(128, 1)` et `(2, 1)`, puis les réglages de la page, `(384, 2)` et
/// (jeu 6 à 300 pas) `(1024, 3)` : `givre_nette_preparer` rend 0 et **chaque octet** du tampon est celui de `rendre(100, valeurs_du_cristal, { cote, rayon_vue: 69, sur, plein: 6 442 450 944 })` (la vue, le `plein` et le champ de la doc écrits à la main :
/// la vue de rayon 100 au lieu de 69, un `plein` autre, la masse du bord ou de la vapeur dessinée, un champ lu dans un autre ordre, ou l'image d'un pixel par case rendue à la place se voient). Le tampon est relu : une lecture ne recalcule rien. Et
/// l'image nette n'est jamais refusée pour le code 1 (la neige est sur la grille hexagonale).
///
/// Ne couvre pas : un champ lu dans un autre ordre de cases qui serait une symétrie de l'hexagone (un cristal symétrique d'ordre 6 donne la même image : cette faute-là ne change rien) ; le temps que cela prend (voir le rapport).
#[test]
#[cfg_attr(not(feature = "etape-g3"), ignore)]
fn l_image_nette_de_la_neige() {
    let _verrou = verrou();
    for jeu in [3_usize, 6, 18] {
        let quatre = quatre_du_jeu(jeu);
        assert_eq!(demarrer(jeu, quatre), 0);
        assert_eq!(givre_avancer(150), 150);
        let (neige, _) = neige_jouee(jeu, quatre, 150);
        for (cote, sur) in [(62_u32, 2_u32), (96, 3), (128, 1), (2, 1), (384, 2)] {
            let contexte = format!("jeu {jeu}, 150 pas, côté {cote}, sur {sur}");
            assert_eq!(givre_nette_preparer(cote, sur), 0, "{contexte}");
            let attendue = image_nette_attendue(&neige, cote, sur);
            exiger_le_tampon(&attendue, cote, &contexte);
            exiger_le_tampon(&attendue, cote, &format!("{contexte}, relu"));
        }
    }
    let quatre = quatre_du_jeu(6);
    assert_eq!(demarrer(6, quatre), 0);
    assert_eq!(givre_avancer(300), 300);
    let (neige, _) = neige_jouee(6, quatre, 300);
    assert_eq!(givre_nette_preparer(1024, 3), 0);
    exiger_le_tampon(&image_nette_attendue(&neige, 1024, 3), 1024, "jeu 6, la fin de la page (1024, 3)");
    // Les refus de l'image nette sont ceux de V7 (jamais le code 1 pour une neige).
    for (cote, sur, code) in
        [(63_u32, 2_u32, 3_u32), (62, 0, 4), (62, 5, 4), (2048, 3, 5), (0, 2, 3), (u32::MAX, u32::MAX, 3)]
    {
        assert_eq!(givre_nette_preparer(cote, sur), code, "({cote}, {sur})");
        exiger_le_tampon_vide(&format!("après le refus ({cote}, {sur})"));
    }
}

/// juste : le tampon et l'état d'une neige (V-G3-09). **Lire n'efface rien** : après `givre_neige_jeu`, `givre_neige_reglage`, `givre_neige_mesure` (et `givre_mesurer`, qui rend 0) et toutes les lectures de l'état, le tampon de l'image nette est encore l'image
/// préparée ; **démarrer efface le tampon** (`givre_neige_demarrer` qui réussit, `givre_demarrer` qui réussit : le tampon d'un flocon préparé n'est jamais celui de la neige démarrée après lui, ni l'inverse) ; `givre_avancer` l'efface dès qu'il joue au
/// moins un pas (pas quand il n'en joue aucun) ; une préparation ne change **rien** à l'état (l'état natif avant et après, la suite du scénario jouée après elle donne l'état natif de la durée) ; deux préparations de suite donnent le même tampon ;
/// et **l'ancienne image n'est jamais rendue** (le tampon d'avant `givre_avancer(1)` n'est pas lisible après). Un tampon jamais effacé, effacé à tort ou rendu d'un autre état, ou un état modifié par la préparation se voient. **La mesure gardée de corrélation**
/// (celle d'un claquage mesuré juste avant) **ne survit pas au démarrage d'une neige**, et un démarrage refusé la laisse (ajout du juge : une neige qui gardait la mesure du modèle d'avant passait tous les tests).
///
/// Ne couvre pas : le contenu de l'image (voir `l_image_nette_de_la_neige`).
#[test]
#[cfg_attr(not(feature = "etape-g3"), ignore)]
fn le_tampon_et_l_etat_d_une_neige() {
    let _verrou = verrou();
    // Un flocon préparé, puis une neige démarrée : le tampon n'est pas celui du flocon.
    assert_eq!(givre_demarrer(2, 0), 1);
    assert_eq!(givre_nette_preparer(62, 2), 0);
    let quatre = quatre_du_jeu(5);
    assert_eq!(demarrer(5, quatre), 0);
    exiger_le_tampon_vide("la neige démarrée après un flocon préparé");
    assert_eq!(givre_avancer(30), 30);
    exiger_le_tampon_vide("après des pas, avant toute préparation");
    assert_eq!(givre_nette_preparer(62, 2), 0);
    let (neige, _) = neige_jouee(5, quatre, 30);
    exiger_le_tampon(&image_nette_attendue(&neige, 62, 2), 62, "(62, 2) au pas 30");

    // Lire n'efface rien.
    let etat = etat_natif(&neige, JEUX[5].pas);
    exiger_la_neige(&etat, 5, quatre, "après la préparation");
    assert_eq!(givre_neige_jeu(0, 0), 9);
    assert_eq!(givre_neige_jeux(), 19);
    exiger_le_tampon(&image_nette_attendue(&neige, 62, 2), 62, "après toutes les lectures de l'état et des mesures");
    assert_eq!(givre_avancer(0), 0);
    exiger_le_tampon(&image_nette_attendue(&neige, 62, 2), 62, "après givre_avancer(0)");

    // Deux préparations de suite : le même tampon ; une préparation de plus petit côté remplace la précédente.
    assert_eq!(givre_nette_preparer(62, 2), 0);
    exiger_le_tampon(&image_nette_attendue(&neige, 62, 2), 62, "deux préparations de suite");
    assert_eq!(givre_nette_preparer(64, 1), 0);
    exiger_le_tampon(&image_nette_attendue(&neige, 64, 1), 64, "(64, 1) remplace (62, 2)");
    assert_eq!(givre_nette_preparer(64, 3), 0);
    exiger_le_tampon(
        &image_nette_attendue(&neige, 64, 3),
        64,
        "(64, 3) remplace (64, 1) : le même côté, un autre suréchantillonnage",
    );

    // Jouer efface, l'ancienne image n'est jamais rendue.
    assert_eq!(givre_avancer(1), 1);
    exiger_le_tampon_vide("après givre_avancer(1), qui joue un pas");
    assert_eq!(givre_nette_preparer(62, 2), 0);
    let (neige, _) = neige_jouee(5, quatre, 31);
    exiger_le_tampon(&image_nette_attendue(&neige, 62, 2), 62, "le tampon du pas 31 est celui du pas 31");
    let (plus_tard, _) = neige_jouee(5, quatre, 131);
    assert!(
        image_nette_attendue(&plus_tard, 62, 2) != image_nette_attendue(&neige, 62, 2),
        "les images des pas 31 et 131 diffèrent : sans effacement, l'ancienne se verrait"
    );

    // Une préparation ne change rien à l'état : l'état natif avant et après, et la suite du scénario.
    exiger_la_neige(&etat_natif(&neige, JEUX[5].pas), 5, quatre, "au pas 31, après la préparation");
    assert_eq!(givre_nette_preparer(63, 2), 3);
    assert_eq!(givre_nette_preparer(62, 9), 4);
    assert_eq!(givre_nette_preparer(64, 2), 0);
    exiger_la_neige(
        &etat_natif(&neige, JEUX[5].pas),
        5,
        quatre,
        "au pas 31, après des préparations refusées et réussies",
    );
    assert_eq!(givre_avancer(100), 100);
    let (apres, _) = neige_jouee(5, quatre, 131);
    exiger_la_neige(&etat_natif(&apres, JEUX[5].pas), 5, quatre, "au pas 131, après des préparations au pas 31");

    // Démarrer efface le tampon : la neige, puis le flocon, puis un démarrage refusé qui ne l'efface pas.
    assert_eq!(givre_nette_preparer(62, 2), 0);
    assert_eq!(demarrer(5, quatre), 0);
    exiger_le_tampon_vide("après un givre_neige_demarrer qui réussit (la même neige)");
    assert_eq!(givre_nette_preparer(62, 2), 0);
    assert_eq!(givre_neige_demarrer(19, 0, 1_000_000, 0, 0), 1);
    assert_eq!(givre_nette_largeur(), 62, "un démarrage refusé ne change rien, tampon compris");
    assert_eq!(givre_demarrer(3, 0), 1);
    exiger_le_tampon_vide("le flocon démarré après une neige préparée");
    assert_eq!(givre_neige_reglage(7), ABSENT, "le flocon n'est pas une neige");

    // La mesure gardée de corrélation ne survit pas au démarrage d'une neige (ajout du juge) : un claquage `η = 0` entier, mesuré, puis une neige ; les lectures de la mesure sont vides **avant** tout `givre_mesurer`
    // (qui, lui, l'effacerait de toute façon : c'est pourquoi `exiger_la_neige` ne voyait pas cette faute), et un refus, lui, la laisse.
    assert_eq!(givre_demarrer(4, 1), 1);
    assert_eq!(givre_avancer(399), 399);
    let points = givre_mesurer();
    assert!(points > 0, "le claquage entier se mesure");
    let (rayon, dimension) = (givre_correlation_rayon(0), givre_dimension());
    assert!(rayon > 0 && !dimension.is_nan(), "une mesure est gardée");
    assert_eq!(givre_neige_demarrer(19, 0, 1_000_000, 0, 0), 1);
    assert_eq!(givre_neige_demarrer(5, 1_000_001, 1_000_000, 0, 0), 2);
    assert_eq!(givre_correlation_rayon(0), rayon, "un démarrage refusé laisse la mesure gardée");
    assert_eq!(givre_dimension().to_bits(), dimension.to_bits(), "un démarrage refusé laisse la mesure gardée");
    assert_eq!(demarrer(5, quatre), 0);
    assert_eq!(givre_correlation_rayon(0), 0, "la neige démarrée ne garde pas la mesure du claquage d'avant");
    assert!(givre_correlation_valeur(0).is_nan(), "la neige démarrée ne garde pas la mesure du claquage d'avant");
    assert!(givre_dimension().is_nan(), "la neige démarrée ne garde pas la mesure du claquage d'avant");
}

/// régression + juste : les cinq étiquettes passent par le module (V-G3-10). Quatre neiges jouées jusqu'à des instants où la bibliothèque donne **chacune des cinq étiquettes** (le jeu 6 à 200 pas : plaque, à 300 : plaque à secteurs, à 700 : fougère ; le jeu 5 à 400 :
/// dendrite ; le jeu 0 à 100 : aucune), `givre_neige_mesure(6)` est le numéro de `balayage::classer` des mesures de `balayage::mesurer` (0 aucune, 1 plaque, 2 secteurs, 3 dendrite, 4 fougère, **numéros écrits à la main**), et les cinq numéros se voient tous
/// (une table des numéros qui en confond deux, ou qui renvoie toujours la même, échoue). Les instants viennent de nos mesures (régression : la bibliothèque donne ces étiquettes à ces instants, `MODELE.md`) ; **l'étiquette est notre classement, pas celui de l'article**.
///
/// Ne couvre pas : que l'étiquette soit juste pour l'article (elle ne l'est ni ne le prétend : `MODELE.md`, étape G2) ; les seuils (voir G2).
#[test]
#[cfg_attr(not(feature = "etape-g3"), ignore)]
fn les_cinq_etiquettes_passent_par_le_module() {
    let _verrou = verrou();
    let mut vues = std::collections::BTreeSet::new();
    for (jeu, instants) in [(6_usize, vec![200_u32, 300, 700]), (5, vec![400]), (0, vec![100])] {
        let quatre = quatre_du_jeu(jeu);
        assert_eq!(demarrer(jeu, quatre), 0);
        let (mut neige, _) = neige_jouee(jeu, quatre, 0);
        let mut temps = 0;
        for instant in instants {
            assert_eq!(givre_avancer(instant - temps), instant - temps);
            avancer_la_neige(&mut neige, instant, JEUX[jeu].pas);
            temps = instant;
            let mesures = mesurer(&neige);
            let forme = code_de_la_forme(classer(&mesures));
            assert_eq!(givre_neige_mesure(6), forme, "jeu {jeu}, {instant} pas : l'étiquette");
            assert_eq!(mesures, g2::mesures_naives(&g2::rattachees_de(&neige)));
            for (champ, valeur) in mesures_en_champs(&etat_natif(&neige, JEUX[jeu].pas)).iter().enumerate().take(6) {
                assert_eq!(givre_neige_mesure(champ as u32), *valeur, "jeu {jeu}, {instant} pas : mesure {champ}");
            }
            vues.insert(forme);
        }
    }
    assert_eq!(
        vues.into_iter().collect::<Vec<_>>(),
        [0, 1, 2, 3, 4],
        "les cinq étiquettes se voient : aucune, plaque, secteurs, dendrite, fougère"
    );
}

/// juste : les modèles qui existent ne changent pas, et la neige s'en distingue (V-G3-11). Après une neige jouée, `givre_demarrer` des modèles 0 à 3 puis 4 à 7 redonne l'état natif (temps, durée, sites, rayon, empreinte, image d'un pixel par
/// case : `e5::natif` et `d3::natif_claquage`) et `givre_neige_reglage` et `givre_neige_mesure` rendent alors `u32::MAX` (le modèle en cours n'est pas une neige) ; **`givre_demarrer(8)`** et au-delà (9, 100, `u32::MAX`) rendent 0 et ne changent rien, y compris
/// une neige en cours (même empreinte, mêmes réglages) : la neige n'est pas un numéro de `givre_demarrer` ; une neige démarrée après chaque modèle (0 à 7) n'en garde rien (temps 0, réglages de la neige, image de la neige).
///
/// Ne couvre pas : les modèles 0 à 7 en détail (voir les tests des étapes 5, D3 et V7, qui jouent sur `givre_demarrer`).
#[test]
#[cfg_attr(not(feature = "etape-g3"), ignore)]
fn les_modeles_qui_existent_ne_changent_pas() {
    let _verrou = verrou();
    let quatre = quatre_du_jeu(2);
    for modele in 0..=7_u32 {
        assert_eq!(demarrer(2, quatre), 0);
        assert_eq!(givre_avancer(20), 20);
        assert_eq!(givre_demarrer(modele, 7), 1, "modèle {modele}");
        let pas = 20;
        let (temps, duree, sites, rayon, empreinte, largeur, hauteur, rvb) = if modele >= 4 {
            let natif = v7::d3::natif_claquage(modele - 4, 7, pas);
            (
                natif.temps,
                natif.duree,
                natif.sites,
                natif.rayon,
                natif.empreinte,
                natif.largeur,
                natif.hauteur,
                natif.rvb,
            )
        } else {
            let natif = e5::natif(modele, 7, pas);
            (
                natif.temps,
                natif.duree,
                natif.sites,
                natif.rayon,
                natif.empreinte,
                natif.largeur,
                natif.hauteur,
                natif.rvb,
            )
        };
        assert_eq!(givre_avancer(pas), pas);
        assert_eq!(
            (givre_temps(), givre_duree(), givre_sites(), givre_rayon()),
            (temps, duree, sites, rayon),
            "modèle {modele}"
        );
        assert_eq!(empreinte_de_la_vitrine(), empreinte, "modèle {modele} : empreinte");
        let (l, h, octets) = image_d_un_pixel_par_case();
        assert_eq!((l, h), (largeur, hauteur));
        assert!(octets == rvb, "modèle {modele} : l'image d'un pixel par case");
        for champ in 0..10_u32 {
            assert_eq!(givre_neige_reglage(champ), ABSENT, "modèle {modele} : réglage {champ}");
        }
        for champ in 0..8_u32 {
            assert_eq!(givre_neige_mesure(champ), ABSENT, "modèle {modele} : mesure {champ}");
        }
        // Une neige démarrée après ce modèle n'en garde rien.
        assert_eq!(demarrer(2, quatre), 0);
        assert_eq!(givre_temps(), 0);
        assert_eq!(givre_neige_reglage(7), 2);
        assert_eq!((givre_largeur(), givre_hauteur()), (201, 201));
    }
    assert_eq!(demarrer(2, quatre), 0);
    assert_eq!(givre_avancer(10), 10);
    let empreinte = empreinte_de_la_vitrine();
    for modele in [8_u32, 9, 100, u32::MAX] {
        assert_eq!(givre_demarrer(modele, 3), 0, "modèle {modele}");
        assert_eq!(givre_neige_reglage(7), 2, "modèle {modele} : la neige est encore là");
        assert_eq!(givre_temps(), 10);
        assert_eq!(empreinte_de_la_vitrine(), empreinte, "modèle {modele} : l'état n'a pas changé");
    }
}

/// juste : jamais de panique, pour aucun `u32` (V-G3-12). `givre_neige_demarrer` pour chaque combinaison de jeux (18, 19, `u32::MAX`) et de valeurs de `ρ`, `β`, `κ`, `μ` choisies aux bornes et au-delà (0, 1, 999 999, 1 000 000, 1 000 001, 8 000 000, 8 000 001,
/// `u32::MAX`) rend le code que la règle de la doc donne (écrite ici : 1 pour le jeu, puis le premier réglage hors plage dans l'ordre `ρ β κ μ`) ; `givre_neige_jeu`, `givre_neige_reglage` et `givre_neige_mesure` pour tous les champs de 0 à 20, 100, 4096 et
/// `u32::MAX`, avec une neige en cours, avec un flocon en cours, et après un démarrage refusé, rendent un entier sans panique (`u32::MAX` hors de portée ou sans neige). Une panique dans une `extern "C" fn` abat tout le binaire de test, ce qui est un échec.
///
/// Ne couvre pas : l'absence de tout démarrage (voir `etape_g3_neuf.rs`) ; les valeurs des champs (voir les autres tests).
#[test]
#[cfg_attr(not(feature = "etape-g3"), ignore)]
fn jamais_de_panique_pour_une_neige_absurde() {
    let _verrou = verrou();
    let valeurs = [0_u32, 1, 999_999, 1_000_000, 1_000_001, 8_000_000, 8_000_001, u32::MAX];
    let code_de_la_doc = |jeu: u32, q: [u32; 4]| -> u32 {
        if jeu >= 19 {
            1
        } else if q[0] > 1_000_000 {
            2
        } else if !(1_000_000..=8_000_000).contains(&q[1]) {
            3
        } else if q[2] > 1_000_000 {
            4
        } else if q[3] > 1_000_000 {
            5
        } else {
            0
        }
    };
    let mut reussis = 0;
    for jeu in [18_u32, 19, u32::MAX] {
        for &rho in &valeurs {
            for &beta in &valeurs {
                for &kappa in &valeurs {
                    for &mu in &valeurs {
                        let q = [rho, beta, kappa, mu];
                        let code = givre_neige_demarrer(jeu, rho, beta, kappa, mu);
                        assert_eq!(code, code_de_la_doc(jeu, q), "jeu {jeu}, {q:?}");
                        reussis += u32::from(code == 0);
                    }
                }
            }
        }
    }
    // Les cas qui réussissent (jeu 18 seulement) : `ρ` parmi 4 valeurs (0, 1, 999 999, 1 000 000), `β` parmi 3 (1 000 000, 1 000 001, 8 000 000), `κ` parmi 4, `μ` parmi 4.
    assert_eq!(reussis, 4 * 3 * 4 * 4);
    let champs = [0_u32, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 20, 100, 4096, u32::MAX];
    assert_eq!(demarrer(18, quatre_du_jeu(18)), 0);
    for etat in ["une neige", "un flocon", "un démarrage refusé"] {
        match etat {
            "un flocon" => assert_eq!(givre_demarrer(1, 0), 1),
            "un démarrage refusé" => {
                assert_eq!(demarrer(18, quatre_du_jeu(18)), 0);
                assert_eq!(givre_neige_demarrer(u32::MAX, 0, 0, 0, 0), 1);
            }
            _ => {}
        }
        for jeu in [0_u32, 18, 19, u32::MAX] {
            for &champ in &champs {
                let _ = givre_neige_jeu(jeu, champ);
            }
        }
        for &champ in &champs {
            let reglage = givre_neige_reglage(champ);
            let mesure = givre_neige_mesure(champ);
            if etat == "un flocon" {
                assert_eq!((reglage, mesure), (ABSENT, ABSENT), "{etat}, champ {champ}");
            } else {
                // Une neige est en cours (jeu 18, ou celle d'un démarrage refusé) : les réglages vont de 0 à 9, les mesures de 0 à 7 ; au-delà, `u32::MAX` ; en deçà, jamais `u32::MAX`.
                assert_eq!(reglage == ABSENT, champ >= 10, "{etat}, réglage {champ}");
                assert_eq!(mesure == ABSENT, champ >= 8, "{etat}, mesure {champ}");
            }
        }
    }
    let _ = Neige::essayer_new(1, parametres_du_jeu(0, 0, 1_000_000, 0, 0)).expect("la bibliothèque seule");
}

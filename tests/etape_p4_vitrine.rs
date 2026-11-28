//! Étape P4 : le métal dans la vitrine (`src/vitrine.rs`, « Le métal »), **en natif** : le module conduit la bibliothèque au bit près (la dendrite de P2, la pointe de P3), la règle du mur, les relevés, les codes, et ce que les
//! fonctions d'avant rendent pour le métal. Valeurs attendues : `references/etape-p4-valeurs.md`, identifiants `V-P4-xx`. En face, toujours le métal **natif** (`etape_p4_commun::MetalNatif` : la bibliothèque, une boucle d'un pas à
//! la fois écrite d'après la doc), jamais la vitrine elle-même. L'état de la vitrine est global : chaque test prend le verrou de ce fichier.

mod etape_p4_commun;

use std::sync::{Mutex, MutexGuard};

use etape_p4_commun::{
    ABSENT, COTE_MAX, COTE_MIN, D0, DELTA, DIFFUSIVITE, DX, EPSILON_MAX_PPM, MUR_POURCENT, MetalNatif, PAS, PROFONDEUR,
    RAYON, RELEVE, W0, hache_des_champs, limite_de, memes_bits, reglages_de,
};
use givre_lab::dendrite::{Dendrite, Domaine, ModeU};
use givre_lab::empreinte::sha256_hex;
use givre_lab::pointe::{constante_de_selection, mesurer_pointe, vitesse_de_pointe};
use givre_lab::vitrine::*;

static VERROU: Mutex<()> = Mutex::new(());

fn verrou() -> MutexGuard<'static, ()> {
    VERROU.lock().unwrap_or_else(|e| e.into_inner())
}

/// L'empreinte de la vitrine : ses huit mots en hexadécimal, bout à bout.
fn empreinte_de_la_vitrine() -> String {
    (0..8).map(|i| format!("{:08x}", givre_empreinte_mot(i))).collect()
}

/// Les quinze réglages et les onze mesures de la vitrine.
fn reglages_de_la_vitrine() -> [f64; 15] {
    std::array::from_fn(|k| givre_metal_reglage(k as u32))
}
fn mesures_de_la_vitrine() -> [f64; 11] {
    std::array::from_fn(|k| givre_metal_mesure(k as u32))
}

/// Les champs de la vitrine lus case par case : leur empreinte (`hache_des_champs`).
fn champs_de_la_vitrine(cote: u32) -> String {
    let n = cote * cote;
    let phi: Vec<f64> = (0..n).map(|i| givre_metal_case(0, i)).collect();
    let u: Vec<f64> = (0..n).map(|i| givre_metal_case(1, i)).collect();
    hache_des_champs(&phi, &u)
}

/// Exige que la vitrine soit, au bit près, le métal natif : temps, durée, empreinte, réglages, mesures, champs, sites et rayon.
fn exiger_le_natif(metal: &MetalNatif, contexte: &str) {
    assert_eq!(u64::from(givre_temps()), metal.dendrite.pas_faits(), "{contexte} : le temps");
    assert_eq!(givre_duree(), PAS, "{contexte} : la durée");
    assert_eq!(empreinte_de_la_vitrine(), metal.dendrite.empreinte(), "{contexte} : l'empreinte");
    let (r, rn) = (reglages_de_la_vitrine(), metal.reglages());
    for k in 0..15 {
        assert!(memes_bits(r[k], rn[k]), "{contexte} : le réglage {k} ({} au lieu de {})", r[k], rn[k]);
    }
    let (m, mn) = (mesures_de_la_vitrine(), metal.mesures());
    for k in 0..11 {
        assert!(memes_bits(m[k], mn[k]), "{contexte} : la mesure {k} ({} au lieu de {})", m[k], mn[k]);
    }
    assert_eq!(
        champs_de_la_vitrine(metal.cote),
        hache_des_champs(metal.dendrite.phi(), metal.dendrite.u()),
        "{contexte} : les champs φ et u"
    );
    let (sites, rayon) = metal.sites_et_rayon();
    assert_eq!((givre_sites(), givre_rayon()), (sites, rayon), "{contexte} : les sites et le rayon");
}

/// forme : les constantes de la doc (V-P4-01), **écrites à la main** dans `etape_p4_commun` d'après la fiche : `ε` de 0 à 60 000 millionièmes, `Δ = 0,65`, `d0 = 0,5`, `D = 1`, `W0 = 1`, `dx = 0,8`, la graine de rayon 15, les côtés de 128
/// à 256, la limite du mur à 55 % du côté, 60 000 pas, un relevé tous les 500 pas, la profondeur 2 ; et la limite du mur de trois côtés, calculée à la main (`⌊128 · 0,55⌋ = 70` cases, `⌊200 · 0,55⌋ = 110`, `⌊255 · 0,55⌋ = 140`, fois 0,8).
///
/// Ne couvre pas : que ces nombres soient les bons pour la page (ils sont mesurés : la fiche et `MODELE.md`) ; ce que le module en fait (les autres tests) : les constantes sont dans le contrat, ce test passe pour tout code.
#[test]
#[cfg_attr(not(feature = "etape-p4"), ignore)]
fn les_constantes_de_la_doc() {
    assert_eq!(METAL_EPSILON_MAX_PPM, EPSILON_MAX_PPM);
    for (constante, attendue) in [
        (METAL_DELTA, DELTA),
        (METAL_D0, D0),
        (METAL_D, DIFFUSIVITE),
        (METAL_W0, W0),
        (METAL_DX, DX),
        (METAL_RAYON, RAYON),
        (METAL_PROFONDEUR, PROFONDEUR),
    ] {
        assert_eq!(constante.to_bits(), attendue.to_bits(), "{constante} au lieu de {attendue}");
    }
    assert_eq!((METAL_COTE_MIN, METAL_COTE_MAX), (COTE_MIN, COTE_MAX));
    assert_eq!((METAL_MUR_POURCENT, METAL_PAS, u64::from(METAL_RELEVE)), (MUR_POURCENT, PAS, RELEVE));
    assert_eq!((DELTA, D0, DIFFUSIVITE, W0, DX, RAYON, PROFONDEUR), (0.65, 0.5, 1.0, 1.0, 0.8, 15.0, 2.0));
    assert_eq!((COTE_MIN, COTE_MAX, MUR_POURCENT, PAS, RELEVE, EPSILON_MAX_PPM), (128, 256, 55, 60_000, 500, 60_000));
    for (cote, cases) in [(128_u32, 70.0_f64), (200, 110.0), (255, 140.0)] {
        assert_eq!(limite_de(cote).to_bits(), (cases * 0.8).to_bits(), "la limite du côté {cote}");
    }
}

/// juste : les codes de `givre_metal_demarrer` et leur ordre (V-P4-02) : 1 pour `ε` au-delà de 60 000 (même avec un côté faux : `ε` est vérifié d'abord), 2 pour un côté hors de 128 à 256 (0, 127, 257, `u32::MAX`), 0 aux bornes
/// (`ε` 0 et 60 000, côtés 128 et 256, et 129 : impair) ; **un refus ne change rien** : ni une DLA en cours (temps, empreinte, image d'un pixel par case), ni un métal en cours (ses champs, ses mesures), ni le tampon de l'image nette
/// ni l'instantané des masses d'une neige.
///
/// Ne couvre pas : le code 3 (inatteignable : la graine est toujours acceptée aux réglages de la doc) ; une graine refusée au milieu de la construction.
#[test]
#[cfg_attr(not(feature = "etape-p4"), ignore)]
fn demarrer_les_codes_et_leur_ordre() {
    let _v = verrou();
    let refus: [(u32, u32, u32); 9] = [
        (60_001, 128, 1),
        (60_001, 0, 1),
        (u32::MAX, u32::MAX, 1),
        (60_001, 129, 1),
        (0, 127, 2),
        (0, 257, 2),
        (60_000, 0, 2),
        (0, u32::MAX, 2),
        (50_000, 1, 2),
    ];
    // Une DLA en cours : un refus ne la change pas.
    assert_eq!(givre_demarrer(0, 3), 1);
    givre_avancer(40);
    let photo = || {
        let pixels: Vec<u32> =
            (0..givre_hauteur()).flat_map(|y| (0..givre_largeur()).map(move |x| givre_pixel(x, y))).collect();
        (givre_temps(), givre_duree(), empreinte_de_la_vitrine(), givre_sites(), pixels)
    };
    let avant = photo();
    for (e, c, code) in refus {
        assert_eq!(givre_metal_demarrer(e, c), code, "givre_metal_demarrer({e}, {c})");
        assert!(givre_metal_reglage(0).is_nan(), "après un refus, aucun métal n'est en cours");
    }
    assert!(avant == photo(), "un refus a changé la DLA en cours");
    // Un métal en cours : un refus ne le change pas.
    assert_eq!(givre_metal_demarrer(50_000, 128), 0);
    givre_avancer(3);
    let mut natif = MetalNatif::new(50_000, 128);
    natif.avancer(3);
    for (e, c, code) in refus {
        assert_eq!(givre_metal_demarrer(e, c), code, "givre_metal_demarrer({e}, {c}) sur un métal");
    }
    exiger_le_natif(&natif, "après des refus, le métal");
    // Une neige, son image nette et ses masses : un refus ne les touche pas.
    assert_eq!(givre_neige_demarrer(0, 400_000, 1_300_000, 3000, 70_000), 0);
    givre_avancer(5);
    assert_eq!(givre_nette_preparer(64, 2), 0);
    assert_eq!(givre_neige_masses_preparer(), 30_301);
    let (nette, masse) = (givre_nette_pixel(32, 32), givre_neige_masse(15_150));
    for (e, c, code) in refus {
        assert_eq!(givre_metal_demarrer(e, c), code);
    }
    assert_eq!((givre_nette_largeur(), givre_nette_pixel(32, 32), givre_neige_masse(15_150)), (64, nette, masse));
    // Les bornes acceptées.
    for (e, c) in [(0, 128), (60_000, 256), (60_000, 129), (1, 128), (59_999, 255)] {
        assert_eq!(givre_metal_demarrer(e, c), 0, "givre_metal_demarrer({e}, {c})");
        exiger_le_natif(&MetalNatif::new(e, c), &format!("démarré ({e}, {c})"));
    }
}

/// juste : **le module conduit la bibliothèque, au bit près** (V-P4-03). Pour neuf métaux (`ε` = 0 ; 1 ; 5 000 ; 37 001 ; 50 000 ; 60 000 millionièmes, côtés 128, 129, 200, 255 et 256) joués par morceaux (de 0 à 1 500 pas, dont 499, 500 et
/// 501 : les relevés tombent sur des nombres de pas, pas sur des appels), après chaque morceau : ce que rend `givre_avancer`, le temps, la durée, **l'empreinte** (celle de `Dendrite::empreinte()` de la graine ronde jouée par
/// `Dendrite::avancer`), les quinze réglages, les onze mesures (la pointe de `position_pointe`, les relevés de `mesurer_pointe`, la vitesse de `vitesse_de_pointe`, `σ*` de `constante_de_selection`), **les champs `φ` et `u` de chaque
/// case** et les sites et le rayon, tous égaux à ceux du métal natif. Une découpe différente du même nombre de pas donne le même état.
///
/// Ne couvre pas : la fin au mur (voir `la_fin_au_mur`) ; une faute de `src/dendrite.rs` ou de `src/pointe.rs` (le natif est la même bibliothèque : P2 et P3 les éprouvent).
#[test]
#[cfg_attr(not(feature = "etape-p4"), ignore)]
fn le_module_conduit_la_bibliotheque_au_bit_pres() {
    let _v = verrou();
    let cas: [(u32, u32, &[u32]); 9] = [
        (0, 128, &[0, 1, 498, 1, 500]),
        (1, 128, &[10]),
        (5_000, 129, &[700]),
        (37_001, 200, &[1, 998, 2]),
        (50_000, 256, &[500, 1]),
        (50_000, 128, &[1_500]),
        (50_000, 128, &[1, 499, 1, 499, 500]),
        (60_000, 128, &[1_000, 0, 500]),
        (60_000, 255, &[3]),
    ];
    for (epsilon, cote, morceaux) in cas {
        assert_eq!(givre_metal_demarrer(epsilon, cote), 0);
        let mut natif = MetalNatif::new(epsilon, cote);
        exiger_le_natif(&natif, &format!("ε {epsilon}, côté {cote}, au départ"));
        for &morceau in morceaux {
            let joue = givre_avancer(morceau);
            assert_eq!(joue, natif.avancer(morceau), "ε {epsilon}, côté {cote} : les pas joués");
            exiger_le_natif(&natif, &format!("ε {epsilon}, côté {cote}, après {} pas", natif.dendrite.pas_faits()));
        }
    }
}

/// juste : les relevés et les mesures, **à la main** (V-P4-04). Au démarrage (`ε` = 0,05, côté 128), le dernier relevé est celui du pas 0 (son temps est 0 ; sa position, son rayon, ses lignes et son sommet sont ceux de
/// `mesurer_pointe(graine, 2)`) et la vitesse et `σ*` sont `NaN` (un seul relevé) ; après 499 pas, rien de neuf (le temps du relevé est encore 0) ; au 500e pas, le relevé est celui du pas 500, la vitesse est
/// `(position(500) − position(0)) / (temps(500) − 0)` et `σ*` est `2 d0 D / (ρ(500)² V)` ; au 1 000e, les relevés des pas 500 et 1 000. Chaque relevé est calculé ici sur une copie de la dendrite **jouée à part**
/// (`Dendrite::graine_ronde`, `avancer`), et la vitesse et `σ*` par les formules de P3 écrites à la main (puis comparées à `vitesse_de_pointe` et `constante_de_selection`).
///
/// Ne couvre pas : un relevé refusé (`mesurer_pointe` rend une erreur : inatteignable aux réglages de la vitrine, aucun dans les 39 scénarios de la page).
#[test]
#[cfg_attr(not(feature = "etape-p4"), ignore)]
fn les_releves_et_les_mesures_a_la_main() {
    let _v = verrou();
    let r = reglages_de(50_000);
    let mut d = Dendrite::graine_ronde(r, 128, Domaine::Quart, 0.65, 15.0, ModeU::Joue).expect("la graine");
    let p0 = mesurer_pointe(&d, 2.0).expect("le relevé du pas 0");
    assert_eq!(givre_metal_demarrer(50_000, 128), 0);
    let m = mesures_de_la_vitrine();
    assert_eq!(m[4], 0.0, "le relevé du pas 0 a le temps 0");
    for (k, attendu) in [(5, p0.position), (6, p0.lignes as f64), (7, p0.sommet), (8, p0.rayon)] {
        assert_eq!(m[k].to_bits(), attendu.to_bits(), "la mesure {k} au départ");
    }
    assert!(m[9].is_nan() && m[10].is_nan(), "un seul relevé : pas de vitesse");
    assert_eq!(m[0].to_bits(), d.position_pointe().expect("la pointe").to_bits());
    assert_eq!((m[1], m[2], m[3]), (0.0, 0.0, 0.0));

    assert_eq!(givre_avancer(499), 499);
    d.avancer(499).expect("499 pas");
    let m = mesures_de_la_vitrine();
    assert_eq!(m[4], 0.0, "après 499 pas, le dernier relevé est encore celui du pas 0");
    assert!(m[9].is_nan(), "après 499 pas, encore un seul relevé");
    assert_eq!(
        m[0].to_bits(),
        d.position_pointe().expect("la pointe").to_bits(),
        "la position est celle de maintenant"
    );
    assert_eq!(m[1].to_bits(), (499.0 * r.dt).to_bits(), "le temps est celui de maintenant");

    assert_eq!(givre_avancer(1), 1);
    d.avancer(1).expect("le 500e pas");
    let p500 = mesurer_pointe(&d, 2.0).expect("le relevé du pas 500");
    let v = (p500.position - p0.position) / (p500.temps - p0.temps);
    let s = ((2.0 * 0.5) * 1.0) / ((p500.rayon * p500.rayon) * v);
    assert_eq!(p500.temps.to_bits(), (500.0 * r.dt).to_bits());
    assert_eq!(v.to_bits(), vitesse_de_pointe(&p0, &p500).expect("la vitesse").to_bits());
    assert_eq!(s.to_bits(), constante_de_selection(0.5, 1.0, p500.rayon, v).expect("σ*").to_bits());
    let m = mesures_de_la_vitrine();
    for (k, attendu) in [(4, p500.temps), (5, p500.position), (6, p500.lignes as f64), (8, p500.rayon), (9, v), (10, s)]
    {
        assert_eq!(m[k].to_bits(), attendu.to_bits(), "la mesure {k} au pas 500");
    }

    assert_eq!(givre_avancer(500), 500);
    d.avancer(500).expect("500 pas de plus");
    let p1000 = mesurer_pointe(&d, 2.0).expect("le relevé du pas 1 000");
    let v = (p1000.position - p500.position) / (p1000.temps - p500.temps);
    let m = mesures_de_la_vitrine();
    assert_eq!(m[4].to_bits(), (1000.0 * r.dt).to_bits());
    assert_eq!(m[9].to_bits(), v.to_bits(), "la vitesse entre les relevés des pas 500 et 1 000");
    assert_eq!(m[10].to_bits(), (((2.0 * 0.5) * 1.0) / ((p1000.rayon * p1000.rayon) * v)).to_bits());
    assert!(v > 0.05 && v < 0.2, "la pointe avance : {v}");
}

/// juste : **la fin au mur** (V-P4-05). Pour le côté 128 et `ε` = 0,06 puis `ε` = 0 (les plus rapide et plus lent à l'arrêt), `givre_avancer(u32::MAX)` joue exactement les pas que joue la boucle de la doc (le métal natif : vérifier
/// le mur **avant** chaque pas) et s'arrête au premier pas où la pointe a atteint la limite (`⌊128 · 0,55⌋ × 0,8 = 56`) : la pointe y est au moins à la limite, et elle n'y était pas un pas plus tôt (une copie jouée à part). Puis le métal est
/// au mur (mesure 2), pas divergé (mesure 3), le scénario est fini avant la durée, et un `givre_avancer` de plus ne joue rien et ne change rien (l'empreinte, les mesures). L'état final est celui du natif, au bit près.
/// **La mesure 2 lit l'état à l'instant** : rejoué jusqu'au pas d'avant le mur, puis `givre_avancer(1)` (un seul pas, sans vérification après lui), elle vaut 1 **avant tout autre appel** (un drapeau posé seulement par la vérification
/// d'avant un pas ne le serait pas encore).
///
/// Ne couvre pas : la fin à la durée (60 000 pas : jamais atteinte aux réglages de la page, et trop long pour un test) ; une divergence (jamais vue à ces réglages).
#[test]
#[cfg_attr(not(feature = "etape-p4"), ignore)]
fn la_fin_au_mur() {
    for epsilon in [60_000_u32, 0] {
        // Le natif, joué hors du verrou, à côté.
        let natif = std::thread::spawn(move || {
            let mut natif = MetalNatif::new(epsilon, 128);
            let joues = natif.avancer(u32::MAX);
            (natif, joues)
        });
        let _v = verrou();
        assert_eq!(givre_metal_demarrer(epsilon, 128), 0);
        let joues = givre_avancer(u32::MAX);
        let (natif, joues_natif) = natif.join().expect("le natif");
        assert_eq!(joues, joues_natif, "ε {epsilon} : les pas joués jusqu'au mur");
        assert!(joues > 5_000 && joues < PAS, "ε {epsilon} : {joues} pas");
        exiger_le_natif(&natif, &format!("ε {epsilon}, au mur"));
        let limite = limite_de(128);
        assert_eq!(limite, 56.0);
        let position = givre_metal_mesure(0);
        assert!(position >= limite, "ε {epsilon} : la pointe {position} n'a pas atteint la limite");
        let mut juste_avant =
            Dendrite::graine_ronde(reglages_de(epsilon), 128, Domaine::Quart, 0.65, 15.0, ModeU::Joue).expect("graine");
        juste_avant.avancer(u64::from(joues - 1)).expect("les pas d'avant");
        let avant = juste_avant.position_pointe().expect("la pointe");
        assert!(avant < limite, "ε {epsilon} : un pas plus tôt, la pointe {avant} était déjà au mur");
        assert_eq!((givre_metal_mesure(2), givre_metal_mesure(3)), (1.0, 0.0), "ε {epsilon} : au mur, sans divergence");
        let empreinte = empreinte_de_la_vitrine();
        assert_eq!((givre_avancer(1), givre_avancer(u32::MAX)), (0, 0), "ε {epsilon} : le scénario est fini");
        assert_eq!(empreinte_de_la_vitrine(), empreinte);
        exiger_le_natif(&natif, &format!("ε {epsilon}, après des pas refusés"));
        // La mesure 2 lit l'état à l'instant : un pas, le dernier, sans vérification après lui.
        assert_eq!(givre_metal_demarrer(epsilon, 128), 0);
        assert_eq!(givre_avancer(joues - 1), joues - 1);
        assert_eq!(givre_metal_mesure(2), 0.0, "ε {epsilon} : un pas avant, pas au mur");
        assert_eq!(givre_avancer(1), 1);
        assert_eq!(givre_metal_mesure(2), 1.0, "ε {epsilon} : au mur dès le pas qui l'atteint, avant tout autre appel");
    }
}

/// juste : ce que les fonctions d'avant rendent pour le métal (V-P4-06), d'après le tableau de la doc. Un métal (`ε` = 0,03, côté 160) à 37 pas : la largeur et la hauteur 0, `givre_pixel` `u32::MAX` (même en `(0, 0)`),
/// la compacité `NaN`, les sites et le rayon du natif (les cases du quart où `φ > 0`, celles de la ligne 0), `givre_mesurer` 0 (et le rayon et la dimension de la corrélation : 0 et `NaN`), `givre_nette_preparer` le code 1
/// (et le tampon vide), les lecteurs de la neige `u32::MAX` et la préparation des masses 0. Et le démarrage du métal **efface** ce qui appartenait à l'état d'avant : la mesure gardée d'une DLA (sa dimension), le tampon de l'image nette
/// et l'instantané des masses d'une neige. Les sites et le rayon suivent le natif pas à pas sur 2 000 pas (`ε` = 0,06, côté 128), et la ligne 1 y a au moins une fois un autre compte que la ligne 0.
///
/// Ne couvre pas : la neige ou la DLA après un métal (voir `les_autres_modeles_remplacent_le_metal`).
#[test]
#[cfg_attr(not(feature = "etape-p4"), ignore)]
fn les_fonctions_d_avant_pour_le_metal() {
    let _v = verrou();
    // Une DLA mesurée, puis le métal : la mesure gardée est effacée.
    assert_eq!(givre_demarrer(0, 1), 1);
    givre_avancer(900);
    assert!(givre_mesurer() > 0, "la DLA de 901 sites se mesure");
    assert_eq!(givre_metal_demarrer(30_000, 160), 0);
    assert!(givre_dimension().is_nan() && givre_correlation_rayon(0) == 0, "le métal efface la mesure gardée");
    // Une neige, son image nette et ses masses, puis le métal : effacés.
    assert_eq!(givre_neige_demarrer(0, 400_000, 1_300_000, 3000, 70_000), 0);
    assert_eq!(givre_nette_preparer(64, 2), 0);
    assert_eq!(givre_neige_masses_preparer(), 30_301);
    assert_eq!(givre_metal_demarrer(30_000, 160), 0);
    assert_eq!((givre_nette_largeur(), givre_nette_hauteur(), givre_nette_pixel(0, 0)), (0, 0, ABSENT));
    assert_eq!(givre_neige_masse(0), ABSENT, "le métal efface l'instantané des masses");
    // Le métal à 37 pas.
    assert_eq!(givre_avancer(37), 37);
    let mut natif = MetalNatif::new(30_000, 160);
    natif.avancer(37);
    exiger_le_natif(&natif, "le métal à 37 pas");
    assert_eq!((givre_largeur(), givre_hauteur()), (0, 0));
    for (x, y) in [(0, 0), (1, 1), (u32::MAX, 0), (0, u32::MAX)] {
        assert_eq!(givre_pixel(x, y), ABSENT, "givre_pixel({x}, {y})");
    }
    assert!(givre_compacite().is_nan());
    let (sites, rayon) = natif.sites_et_rayon();
    assert!(sites > 250 && rayon > 15 && rayon < 160, "sites {sites}, rayon {rayon}");
    assert_eq!(givre_mesurer(), 0);
    assert_eq!(givre_correlation_rayon(0), 0);
    assert!(givre_correlation_valeur(0).is_nan() && givre_dimension().is_nan());
    assert_eq!(givre_nette_preparer(64, 2), 1, "le métal n'a pas d'image nette");
    assert_eq!((givre_nette_largeur(), givre_nette_hauteur(), givre_nette_pixel(0, 0)), (0, 0, ABSENT));
    for champ in [0, 6, 7, 9, 10, u32::MAX] {
        assert_eq!(givre_neige_reglage(champ), ABSENT, "givre_neige_reglage({champ})");
        assert_eq!(givre_neige_mesure(champ), ABSENT, "givre_neige_mesure({champ})");
    }
    assert_eq!(givre_neige_masses_preparer(), 0);
    assert_eq!(givre_neige_masse(0), ABSENT);
    for i in [8, 100, u32::MAX] {
        assert_eq!(givre_empreinte_mot(i), 0, "givre_empreinte_mot({i})");
    }
    exiger_le_natif(&natif, "le métal après les lectures et les préparations refusées");
    // Les sites et le rayon, compté sur la ligne 0 et nulle part ailleurs : à 40 relevés de 50 pas (`ε` = 0,06, côté 128), au moins une fois la ligne 1 a un autre compte (une lecture de la mauvaise ligne se voit).
    assert_eq!(givre_metal_demarrer(60_000, 128), 0);
    let mut natif = MetalNatif::new(60_000, 128);
    let mut ligne_1_differe = false;
    for _ in 0..40 {
        assert_eq!(givre_avancer(50), natif.avancer(50));
        let (sites, rayon) = natif.sites_et_rayon();
        assert_eq!((givre_sites(), givre_rayon()), (sites, rayon), "au pas {}", natif.dendrite.pas_faits());
        let ligne_1 = natif.dendrite.phi()[128..256].iter().filter(|p| **p > 0.0).count() as u32;
        ligne_1_differe |= ligne_1 != rayon;
    }
    assert!(
        ligne_1_differe,
        "la ligne 1 a toujours le compte de la ligne 0 : le test ne verrait pas une lecture de la mauvaise ligne"
    );
}

/// juste : les autres modèles remplacent le métal, et le métal les remplace (V-P4-07). Après un métal, `givre_demarrer(0, 1)` : les trois lecteurs du métal rendent `NaN`, le temps est 0 et la durée 2 999 (la DLA) ;
/// `givre_neige_demarrer` après un métal : `NaN` aussi, et la neige se lit ; le métal après une neige : les lecteurs de la neige rendent `u32::MAX` ; un métal d'un `ε` après un autre : **tout repart** (le temps 0, les relevés : la vitesse
/// `NaN`, l'empreinte de la graine du nouvel `ε`).
///
/// Ne couvre pas : la page (voir `etape_p4_page.rs`).
#[test]
#[cfg_attr(not(feature = "etape-p4"), ignore)]
fn les_autres_modeles_remplacent_le_metal() {
    let _v = verrou();
    let sans_metal = |contexte: &str| {
        for champ in [0, 1, 8, 14] {
            assert!(givre_metal_reglage(champ).is_nan(), "{contexte} : givre_metal_reglage({champ})");
        }
        for champ in [0, 1, 2, 9] {
            assert!(givre_metal_mesure(champ).is_nan(), "{contexte} : givre_metal_mesure({champ})");
        }
        assert!(givre_metal_case(0, 0).is_nan() && givre_metal_case(1, 0).is_nan(), "{contexte} : les cases");
    };
    assert_eq!(givre_metal_demarrer(40_000, 128), 0);
    givre_avancer(600);
    assert_eq!(givre_demarrer(0, 1), 1);
    sans_metal("après une DLA");
    assert_eq!((givre_temps(), givre_duree()), (0, 2999));
    assert_eq!(givre_metal_demarrer(40_000, 128), 0);
    givre_avancer(600);
    assert_eq!(givre_neige_demarrer(0, 400_000, 1_300_000, 3000, 70_000), 0);
    sans_metal("après une neige");
    assert_eq!(givre_neige_reglage(7), 0, "la neige se lit");
    assert_eq!(givre_metal_demarrer(40_000, 128), 0);
    assert_eq!(givre_neige_reglage(7), ABSENT, "le métal remplace la neige");
    assert_eq!(givre_neige_mesure(0), ABSENT);
    // Un métal après un métal : tout repart.
    assert_eq!(givre_avancer(1_200), 1_200);
    assert!(!givre_metal_mesure(9).is_nan(), "deux relevés : une vitesse");
    assert_eq!(givre_metal_demarrer(10_000, 140), 0);
    exiger_le_natif(&MetalNatif::new(10_000, 140), "un second métal");
    assert!(givre_metal_mesure(9).is_nan(), "le second métal n'a qu'un relevé");
}

/// juste : les bornes des lecteurs (V-P4-08). `givre_metal_case(champ, i)` rend les bits du natif pour `φ` et `u` aux cases 0, 1, `côté`, `côté² − 1` et `NaN` pour `i = côté²`, `côté² + 1` et `u32::MAX` ; `NaN` pour un `champ` de 2, 3 ou
/// `u32::MAX` ; `givre_metal_mesure` et `givre_metal_reglage` rendent `NaN` au-delà de leurs tableaux (11 et 15 champs), `u32::MAX` compris.
///
/// Ne couvre pas : les valeurs elles-mêmes (voir `le_module_conduit_la_bibliotheque_au_bit_pres`).
#[test]
#[cfg_attr(not(feature = "etape-p4"), ignore)]
fn les_bornes_des_lecteurs() {
    let _v = verrou();
    let cote = 131_u32;
    assert_eq!(givre_metal_demarrer(20_000, cote), 0);
    givre_avancer(77);
    let mut natif = MetalNatif::new(20_000, cote);
    natif.avancer(77);
    let n = cote * cote;
    for i in [0, 1, cote, n - 1] {
        assert_eq!(givre_metal_case(0, i).to_bits(), natif.dendrite.phi()[i as usize].to_bits(), "φ de la case {i}");
        assert_eq!(givre_metal_case(1, i).to_bits(), natif.dendrite.u()[i as usize].to_bits(), "u de la case {i}");
    }
    for i in [n, n + 1, u32::MAX] {
        assert!(givre_metal_case(0, i).is_nan() && givre_metal_case(1, i).is_nan(), "la case {i}");
    }
    for champ in [2, 3, u32::MAX] {
        assert!(givre_metal_case(champ, 0).is_nan(), "le champ {champ}");
    }
    for champ in [11, 12, 100, u32::MAX] {
        assert!(givre_metal_mesure(champ).is_nan(), "givre_metal_mesure({champ})");
    }
    for champ in [15, 16, 100, u32::MAX] {
        assert!(givre_metal_reglage(champ).is_nan(), "givre_metal_reglage({champ})");
    }
    assert_eq!(givre_metal_reglage(14), 2.0);
    // Après 1 077 pas, deux relevés (500 et 1 000) : la vitesse et `σ*` existent, et la mesure 11 reste `NaN`.
    assert_eq!(givre_avancer(1000), 1000);
    natif.avancer(1000);
    assert!(!givre_metal_mesure(10).is_nan(), "deux relevés : σ* existe");
    assert_eq!(givre_metal_mesure(10).to_bits(), natif.mesures()[10].to_bits());
    assert!(givre_metal_mesure(11).is_nan());
}

/// forme : jamais de panique, dans le désordre (V-P4-09 ; une robustesse : un code nul le passe). La suite de 400 tours aux arguments absurdes de `tests/etape_p4/metal-absurde.mjs` (`etape_p4_commun::attendu_de_l_absurde` : les quatre fonctions du métal et celles d'avant,
/// 0, 1, 127, 128, 256, 257, 60 001, `u32::MAX` et des nombres tirés), jouée sur le code natif : une panique abattrait ce binaire de test. Elle rend 400 tours d'appels et au moins un démarrage réussi du métal.
///
/// Ne couvre pas : la même suite dans le module (voir `etape_p4_module.rs`, sous Node) ; la justesse des valeurs (les autres tests).
#[test]
#[cfg_attr(not(feature = "etape-p4"), ignore)]
fn jamais_de_panique_dans_le_desordre() {
    let _v = verrou();
    let sortie = etape_p4_commun::attendu_de_l_absurde();
    assert!(sortie.starts_with("appels=") && sortie.contains("exceptions=0"), "{sortie}");
    let appels: u32 = sortie["appels=".len()..].split('\n').next().and_then(|n| n.parse().ok()).expect("un nombre");
    assert!(appels >= 400, "{appels} appels");
}

/// régression : les empreintes de trois métaux, écrites ici (V-P4-10) : `ε` = 0,05 au côté 128 après 1 000 pas, `ε` = 0 au côté 131 après 750 pas, `ε` = 0,06 au côté 256 après 300 pas. **Elles sont les mêmes sous Linux et sous
/// Windows** (la CI joue ce test sur les deux ; le juge les a calculées sous les deux et dans le module sous Node) : seuls `+ − * /` et `sqrt` y arrondissent, `min`, `max`, `abs` et `clamp` sont exacts (P2). Ce sont des résultats de notre code, figés.
///
/// Ne couvre pas : la justesse du modèle (P2) ; d'autres réglages.
#[test]
#[cfg_attr(not(feature = "etape-p4"), ignore)]
fn les_empreintes_de_reference() {
    let _v = verrou();
    for (epsilon, cote, pas, attendue) in [
        (50_000_u32, 128_u32, 1000_u32, "041aa5a3c6fe28ecaa44071e46612419cef3de46e84588813d65c754a58393a4"),
        (0, 131, 750, "248051def31d484f01fd4c9d06ead7f688d10888965b6633012e07cdf7d19e82"),
        (60_000, 256, 300, "2dc315c3650707897f63389cd7ed14b3c3572238f02bc900feda7e1dc7d554c8"),
    ] {
        assert_eq!(givre_metal_demarrer(epsilon, cote), 0);
        assert_eq!(givre_avancer(pas), pas);
        assert_eq!(empreinte_de_la_vitrine(), attendue, "ε {epsilon}, côté {cote}, {pas} pas");
        let mut d =
            Dendrite::graine_ronde(reglages_de(epsilon), cote as usize, Domaine::Quart, 0.65, 15.0, ModeU::Joue)
                .expect("graine");
        d.avancer(u64::from(pas)).expect("les pas");
        assert_eq!(sha256_hex(&d.ecriture_canonique()), attendue, "ε {epsilon}, côté {cote} : la bibliothèque");
    }
}

/// régression : **la limite du mur suffit** (V-P4-11), sur un cas que la mesure du juge a fait (`MODELE.md`, étape P4 : l'effet du mur). La même graine, jouée par la bibliothèque dans un quart de 96 cases et dans un quart de 192, pour `ε` = 0 et
/// `ε` = 0,06 : au premier pas où la pointe du petit quart atteint sa limite (`⌊96 · 0,55⌋ × 0,8 = 41,6`), les deux pointes diffèrent de moins de `1e-2` (la doc ; observé : 1,46e-4 pour `ε` = 0 et 3,2e-6 pour `ε` = 0,06, aux pas 3 749 et 4 992).
///
/// Ne couvre pas : les côtés de la page (128 à 256 : mesurés hors des tests, de même que `ε` = 0,02 ; 0,04 ; 0,05) ; d'autres réglages ; que l'arrêt soit fidèle à un domaine infini (seul un domaine deux fois plus grand est comparé).
#[test]
#[cfg_attr(not(feature = "etape-p4"), ignore)]
fn le_mur_n_a_pas_d_effet_avant_la_limite() {
    let calculs: Vec<std::thread::JoinHandle<(u32, f64)>> = [0_u32, 60_000]
        .into_iter()
        .map(|epsilon| {
            std::thread::spawn(move || {
                let r = reglages_de(epsilon);
                let mut petit = Dendrite::graine_ronde(r, 96, Domaine::Quart, 0.65, 15.0, ModeU::Joue).expect("graine");
                let mut grand =
                    Dendrite::graine_ronde(r, 192, Domaine::Quart, 0.65, 15.0, ModeU::Joue).expect("graine");
                let limite = f64::from(96 * MUR_POURCENT / 100) * DX;
                loop {
                    let p = petit.position_pointe().expect("la pointe du petit quart");
                    if p >= limite {
                        return (epsilon, (p - grand.position_pointe().expect("la pointe du grand quart")).abs());
                    }
                    petit.avancer(1).expect("un pas");
                    grand.avancer(1).expect("un pas");
                }
            })
        })
        .collect();
    for calcul in calculs {
        let (epsilon, ecart) = calcul.join().expect("le calcul");
        assert!(ecart < 1e-2, "ε {epsilon} : à la limite, la pointe du petit quart s'écarte de {ecart:e}");
        assert!(ecart > 0.0, "ε {epsilon} : le mur a un effet, si petit soit-il");
    }
}

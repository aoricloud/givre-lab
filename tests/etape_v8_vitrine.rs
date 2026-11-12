//! Étape V8 : la vitrine côté natif, la masse de chaque case de la neige par `givre_neige_masses_preparer` et `givre_neige_masse`. Valeurs attendues : `references/etape-v8-valeurs.md`, identifiants `V-V8-xx`.
//!
//! On appelle les fonctions `givre_*` comme des fonctions Rust et on les compare à ce que la bibliothèque donne **seule** : une `Neige` construite en face (`etape_g3_commun`), dont les masses sont `valeurs_du_cristal` décalées de 21 bits
//! (`etape_v8_commun::masses_de`), l'ordre canonique écrit comme la doc le dit (deux boucles : `etape_v8_commun::cases_dans_l_ordre`). Les dix-neuf jeux nommés sont **écrits à la main** (`etape_g3_commun::JEUX`). Le rendu qui prouve que
//! la doc et ces nombres suffisent à redessiner l'image nette est dans `etape_v8_rendu.rs`, le module WebAssembly dans `etape_v8_module.rs` (sous Node), l'absence de tout démarrage dans `etape_v8_neuf.rs`.
//!
//! L'état de la vitrine est global au processus et les tests d'un fichier tournent en parallèle : chaque test prend le verrou `VERROU` et le garde jusqu'à la fin. Aucun test de ce fichier n'a de tolérance : tout est une égalité exacte.
//!
//! Tant qu'une des deux fonctions `givre_neige_masse*` de `src/vitrine.rs` a encore son corps `todo!()`, ce binaire de test **s'arrête d'un coup** au lieu de marquer un test en échec : une panique ne peut pas traverser une `extern "C" fn`
//! (« panic in a function that cannot unwind »). C'est un échec, et la ligne citée dit quelle fonction reste à écrire.

mod etape_v8_commun;

use std::sync::{Mutex, MutexGuard};

use etape_v8_commun::g3::{JEUX, avancer_la_neige, neige_jouee, parametres_du_jeu, quatre_du_jeu};
use etape_v8_commun::{
    ABSENT, CASES, INDICE_DU_GERME, cases_dans_l_ordre, e5, indice_de, masses_de, rattachees_de, somme_des_masses,
};
use givre_lab::empreinte::sha256_hex;
use givre_lab::neige::Neige;
use givre_lab::vitrine::{
    NEIGE_MASSE_DECALAGE, NEIGE_MASSE_PLEIN, givre_avancer, givre_compacite, givre_demarrer, givre_duree,
    givre_empreinte_mot, givre_hauteur, givre_largeur, givre_mesurer, givre_neige_demarrer, givre_neige_masse,
    givre_neige_masses_preparer, givre_neige_mesure, givre_neige_reglage, givre_nette_hauteur, givre_nette_largeur,
    givre_nette_pixel, givre_nette_preparer, givre_pixel, givre_rayon, givre_sites, givre_temps,
};

static VERROU: Mutex<()> = Mutex::new(());

/// Prend le verrou de la vitrine ; un test qui a paniqué en le tenant ne bloque pas les suivants.
fn verrou() -> MutexGuard<'static, ()> {
    VERROU.lock().unwrap_or_else(|empoisonne| empoisonne.into_inner())
}

/// Démarre le jeu `jeu` avec ses quatre réglages `quatre` : le code que rend le module.
fn demarrer(jeu: usize, quatre: [u32; 4]) -> u32 {
    givre_neige_demarrer(jeu as u32, quatre[0], quatre[1], quatre[2], quatre[3])
}

/// Les masses de l'instantané, lues une à une par `givre_neige_masse` pour `i` de 0 à `nombre − 1`.
fn lire_les_masses(nombre: usize) -> Vec<u32> {
    (0..nombre).map(|i| givre_neige_masse(i as u32)).collect()
}

/// Exige que l'instantané soit **absent** : toute lecture rend `u32::MAX`, au début, au milieu, à la fin et au-delà.
fn exiger_pas_d_instantane(contexte: &str) {
    for i in [0_u32, 1, 15_149, 15_150, 15_151, 30_299, 30_300, 30_301, 30_302, 65_535, 1 << 31, u32::MAX - 1, u32::MAX]
    {
        assert_eq!(givre_neige_masse(i), ABSENT, "{contexte} : la masse {i} sans instantané");
    }
    // Et partout dans le tampon qu'il y aurait : pas une seule case n'a survécu.
    assert!((0..CASES as u32).all(|i| givre_neige_masse(i) == ABSENT), "{contexte} : une masse a survécu");
}

/// Tout ce que la vitrine dit d'une neige démarrée, sauf les masses : l'état (temps, durée, sites, rayon, compacité `NaN`, les huit mots de l'empreinte), l'image d'un pixel par case (SHA-256 des octets), le tampon de l'image nette (dimensions et
/// SHA-256 des octets, ou `0x0` sans tampon), les dix champs des réglages et les huit des mesures. **Une lecture qui modifie un de ces nombres, ou une préparation des masses qui y touche, change cette photographie.**
#[derive(Debug, PartialEq)]
struct Photo {
    temps: u32,
    duree: u32,
    sites: u32,
    rayon: u32,
    compacite_nan: bool,
    mots: Vec<u32>,
    image: (u32, u32, String),
    nette: (u32, u32, String),
    reglages: Vec<u32>,
    mesures: Vec<u32>,
    correlation: u32,
}

fn photographier() -> Photo {
    let (largeur, hauteur) = (givre_largeur(), givre_hauteur());
    let mut rvb = Vec::new();
    for y in 0..hauteur {
        for x in 0..largeur {
            let c = givre_pixel(x, y);
            rvb.extend_from_slice(&[(c >> 16) as u8, (c >> 8) as u8, c as u8]);
        }
    }
    let (nl, nh) = (givre_nette_largeur(), givre_nette_hauteur());
    let mut nette = Vec::new();
    for y in 0..nh {
        for x in 0..nl {
            let c = givre_nette_pixel(x, y);
            nette.extend_from_slice(&[(c >> 16) as u8, (c >> 8) as u8, c as u8]);
        }
    }
    Photo {
        temps: givre_temps(),
        duree: givre_duree(),
        sites: givre_sites(),
        rayon: givre_rayon(),
        compacite_nan: givre_compacite().is_nan(),
        mots: (0..8).map(|i| givre_empreinte_mot(i)).collect(),
        image: (largeur, hauteur, sha256_hex(&rvb)),
        nette: (nl, nh, sha256_hex(&nette)),
        reglages: (0..10).map(|i| givre_neige_reglage(i)).collect(),
        mesures: (0..8).map(|i| givre_neige_mesure(i)).collect(),
        correlation: givre_mesurer(),
    }
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

/// Exige que le tampon des masses soit celui de `neige` : le nombre de cases, **chaque masse** (`valeurs_du_cristal >> 21`), les cases non rattachées à 0, et les bornes de `i`.
fn exiger_les_masses_de(neige: &Neige, nombre: u32, contexte: &str) {
    assert_eq!(nombre as usize, CASES, "{contexte} : le nombre de cases");
    let attendues = masses_de(neige);
    let lues = lire_les_masses(CASES);
    assert!(lues == attendues, "{contexte} : les masses ne sont pas `valeurs_du_cristal >> 21` de la bibliothèque");
    for (i, rattachee) in rattachees_de(neige).iter().enumerate() {
        assert!(*rattachee || lues[i] == 0, "{contexte} : la case {i} n'est pas rattachée et vaut {}", lues[i]);
    }
    assert_eq!(givre_neige_masse(CASES as u32), ABSENT, "{contexte} : la case 30301 est hors du tampon");
    assert_eq!(givre_neige_masse(u32::MAX), ABSENT, "{contexte} : la case u32::MAX est hors du tampon");
}

/// juste : les constantes de la doc (V-V8-01), **écrites ici à la main** : l'unité est `2^-11` (21 bits jetés), la masse pleine est 3072, un domaine de rayon 100 a 30 301 cases, le germe est la case d'indice 15 150 ; et l'ordre canonique
/// écrit comme la doc le dit (deux boucles) est celui de la formule fermée de la doc, pour **chaque case** de six rayons (de 0 à 100) : le nombre de cases `3 R (R + 1) + 1`, `(0, −R)` en premier, `(0, R)` en dernier, `(0, 0)` à `3 R (R + 1) / 2`,
/// et pour `R = 1` la liste de la doc (`(0, −1)`, `(1, −1)`, `(−1, 0)`, `(0, 0)`, `(1, 0)`, `(−1, 1)`, `(0, 1)`).
///
/// Ne couvre pas : que `Hexagone` suive cet ordre (voir `les_masses_sont_la_lecture_de_la_neige` : les masses lues y sont celles de `valeurs_du_cristal`, rangées dans l'ordre de `Hexagone`, et l'image de `etape_v8_rendu.rs`).
#[test]
#[cfg_attr(not(feature = "etape-v8"), ignore)]
fn les_constantes_et_l_ordre_canonique_de_la_doc() {
    let _verrou = verrou();
    assert_eq!(NEIGE_MASSE_DECALAGE, 21);
    assert_eq!(NEIGE_MASSE_PLEIN, 3072);
    assert_eq!(i64::from(NEIGE_MASSE_PLEIN) << NEIGE_MASSE_DECALAGE, 6_442_450_944, "NEIGE_PLEIN = 3072 × 2^21");
    assert_eq!(3 * 100 * 101 + 1, CASES);
    assert_eq!(3 * 100 * 101 / 2, INDICE_DU_GERME);
    for rayon in [0_i64, 1, 2, 5, 17, 100] {
        let cases = cases_dans_l_ordre(rayon);
        assert_eq!(cases.len() as i64, 3 * rayon * (rayon + 1) + 1, "R = {rayon} : le nombre de cases");
        assert_eq!(cases.first(), Some(&(0, -rayon)), "R = {rayon} : la première case");
        assert_eq!(cases.last(), Some(&(0, rayon)), "R = {rayon} : la dernière case");
        assert_eq!(cases[(3 * rayon * (rayon + 1) / 2) as usize], (0, 0), "R = {rayon} : le germe");
        for (i, &(q, r)) in cases.iter().enumerate() {
            assert_eq!(indice_de(rayon, q, r), i, "R = {rayon} : la formule fermée pour ({q}, {r})");
        }
    }
    assert_eq!(cases_dans_l_ordre(1), [(0, -1), (1, -1), (-1, 0), (0, 0), (1, 0), (-1, 1), (0, 1)]);
}

/// juste : (V-V8-02) pour chacun des dix-neuf jeux nommés, à trois instants (0, 1 et 45 pas), et pour douze réglages tirés (graines écrites dans le test) à trois instants (3, 60 et 150 pas), `givre_neige_masses_preparer()` rend 30 301 et
/// `givre_neige_masse(i)` vaut `valeurs_du_cristal()[i] >> 21` de la bibliothèque **pour tout `i`** (30 301 cases, une à une) ; une case non rattachée vaut 0 ; `i = 30 301` et `u32::MAX` rendent `u32::MAX`. Un jeu joué jusqu'au bout
/// (le jeu 8 : 1300 pas) et une neige rapide arrêtée **au mur** (le jeu 1 avec `ρ = 0,956862`) sont lus de la même façon à la fin. Une unité fausse (`>> 20`, `>> 22`, des millionièmes), une case décalée, deux cases
/// échangées, un nombre de cases faux ou une masse lue sur une autre case se voient.
///
/// Ne couvre pas : que la bibliothèque soit juste (G1) ; l'ordre canonique lui-même (voir `les_constantes_et_l_ordre_canonique_de_la_doc` et `etape_v8_rendu.rs` : tous les états de la vitrine ont les douze symétries de l'hexagone, donc une lecture dans l'ordre inverse
/// ou transposé rend les mêmes nombres : voir `MODELE.md`, « Ce que cette étape n'établit pas »).
#[test]
#[cfg_attr(not(feature = "etape-v8"), ignore)]
fn les_masses_sont_la_lecture_de_la_neige() {
    let _verrou = verrou();
    // La neige native avance **en même temps** que la vitrine : à chaque instant, les deux ont joué le même nombre de pas.
    let jouer_et_comparer = |jeu: usize, quatre: [u32; 4], instants: &[u32], contexte: &str| -> Neige {
        assert_eq!(demarrer(jeu, quatre), 0, "{contexte}");
        let mut neige =
            Neige::essayer_new(JEUX[jeu].rayon, parametres_du_jeu(jeu, quatre[0], quatre[1], quatre[2], quatre[3]))
                .expect("une neige valide");
        let mut deja = 0;
        for &instant in instants {
            let vitrine = givre_avancer(instant - deja);
            let natif = avancer_la_neige(&mut neige, instant, JEUX[jeu].pas);
            assert_eq!(vitrine, natif, "{contexte} : pas joués jusqu'à {instant}");
            deja += vitrine;
            exiger_les_masses_de(&neige, givre_neige_masses_preparer(), &format!("{contexte}, {instant} pas"));
        }
        neige
    };
    for jeu in 0..19_usize {
        let _ = jouer_et_comparer(jeu, quatre_du_jeu(jeu), &[0, 1, 45], &format!("jeu {jeu}"));
    }
    let mut hasard = Hasard(0x5EED_0008);
    for tirage in 0..12 {
        let jeu = hasard.entre(0, 18) as usize;
        let quatre = [
            hasard.entre(300_000, 1_000_000),
            hasard.entre(1_000_000, 3_500_000),
            hasard.entre(0, 100_000),
            hasard.entre(0, 150_000),
        ];
        let _ =
            jouer_et_comparer(jeu, quatre, &[3, 60, 150], &format!("tirage {tirage} : jeu {jeu}, réglages {quatre:?}"));
    }
    // Un jeu entier : la fin du scénario (le jeu 8, figure 10 : 1300 pas).
    let quatre = quatre_du_jeu(8);
    let neige = jouer_et_comparer(8, quatre, &[JEUX[8].pas], "jeu 8 jusqu'au bout");
    assert_eq!(givre_avancer(1), 0, "jeu 8 : le scénario est fini");
    exiger_les_masses_de(&neige, givre_neige_masses_preparer(), "jeu 8, après la fin");
    // Une neige arrêtée au mur : l'instantané est l'état final.
    let rapide = [956_862, 1_122_299, 48_396, 36_667];
    assert_eq!(demarrer(1, rapide), 0);
    let joues = givre_avancer(1000);
    assert!(joues < 1000, "cette neige s'arrête au mur avant 1000 pas : {joues}");
    assert_eq!(givre_neige_mesure(7), 1, "au mur");
    let (neige, natif) = neige_jouee(1, rapide, 1000);
    assert_eq!(joues, natif);
    exiger_les_masses_de(&neige, givre_neige_masses_preparer(), "au mur");
    assert_eq!(givre_avancer(10), 0, "au mur : rien ne se joue");
    let apres = givre_neige_masses_preparer();
    exiger_les_masses_de(&neige, apres, "au mur, après un avancer qui ne joue rien");
}

/// juste : l'instantané ne bouge pas quand on joue (V-V8-03). Démarrer le jeu 6 puis préparer (au temps 0 : le germe seul), `givre_avancer(37)` joue 37 pas et **toutes** les masses lues sont encore celles du temps 0 (le tampon des masses n'est pas
/// celui de l'image nette : celui-là s'efface) ; une seconde préparation le rafraîchit : les masses sont celles de la bibliothèque à 37 pas, et elles diffèrent de celles du temps 0 ; encore 20 pas puis **sans** préparer : toujours celles de 37 ;
/// la préparation de l'image nette, `givre_mesurer`, les lecteurs de la neige et les lectures de l'état ne le changent pas non plus. Un instantané rafraîchi par `givre_avancer`, ou effacé par lui, se voit.
///
/// Ne couvre pas : le temps de l'instantané (la vitrine ne le garde pas : la page sait quand elle a préparé).
#[test]
#[cfg_attr(not(feature = "etape-v8"), ignore)]
fn l_instantane_ne_bouge_pas_quand_on_joue() {
    let _verrou = verrou();
    let quatre = quatre_du_jeu(6);
    assert_eq!(demarrer(6, quatre), 0);
    assert_eq!(givre_neige_masses_preparer() as usize, CASES);
    let (neige0, _) = neige_jouee(6, quatre, 0);
    let (neige37, _) = neige_jouee(6, quatre, 37);
    let (masses0, masses37) = (masses_de(&neige0), masses_de(&neige37));
    assert_ne!(masses0, masses37, "le cristal a poussé en 37 pas");
    assert_eq!(lire_les_masses(CASES), masses0);

    assert_eq!(givre_avancer(37), 37);
    assert_eq!(lire_les_masses(CASES), masses0, "après 37 pas, l'instantané est encore celui du temps 0");
    assert_eq!(givre_neige_masses_preparer() as usize, CASES);
    assert_eq!(lire_les_masses(CASES), masses37, "la préparation le rafraîchit : l'état à 37 pas");

    assert_eq!(givre_avancer(20), 20);
    assert_eq!(lire_les_masses(CASES), masses37, "20 pas de plus, sans préparer : toujours 37");
    // Ce qui ne le change pas.
    assert_eq!(givre_nette_preparer(62, 2), 0);
    assert_eq!(lire_les_masses(CASES), masses37, "préparer l'image nette ne touche pas aux masses");
    assert_eq!(givre_mesurer(), 0);
    for champ in 0..10 {
        let _ = givre_neige_reglage(champ);
    }
    for champ in 0..8 {
        let _ = givre_neige_mesure(champ);
    }
    let _ = (0..8).map(|i| givre_empreinte_mot(i)).collect::<Vec<_>>();
    assert_eq!(lire_les_masses(CASES), masses37, "les lectures de l'état ne touchent pas aux masses");
    // Un `givre_avancer` qui ne joue rien (n = 0) ne touche pas non plus à l'instantané.
    assert_eq!(givre_avancer(0), 0);
    assert_eq!(lire_les_masses(CASES), masses37);
    // Mille lectures de la même case rendent mille fois la même masse (une lecture ne change rien).
    assert!((0..1000).all(|_| givre_neige_masse(INDICE_DU_GERME as u32) == masses37[INDICE_DU_GERME]));
}

/// juste : démarrer efface l'instantané (V-V8-04). Avec un instantané (jeu 6 à 30 pas, préparé), `givre_neige_demarrer` qui réussit (le même jeu, un autre jeu) l'efface : toutes les lectures rendent `u32::MAX` jusqu'à la préparation suivante,
/// qui rend 30 301 et le remplit à l'état de **cette** neige ; `givre_demarrer` (la plaque, la DLA, un claquage) qui réussit l'efface aussi ; **un démarrage refusé ne change rien** : `givre_neige_demarrer` (jeu 19, `u32::MAX` partout, `ρ` hors plage, `β` hors plage)
/// et `givre_demarrer(8, 0)` (un modèle qui n'existe pas) laissent l'instantané, lecture après lecture. Un tampon gardé d'une neige à l'autre (l'image d'un autre état), ou effacé par un refus, se voit.
///
/// Ne couvre pas : le démarrage avant tout démarrage (voir `etape_v8_neuf.rs`).
#[test]
#[cfg_attr(not(feature = "etape-v8"), ignore)]
fn demarrer_efface_l_instantane_et_un_refus_ne_change_rien() {
    let _verrou = verrou();
    let quatre = quatre_du_jeu(6);
    assert_eq!(demarrer(6, quatre), 0);
    assert_eq!(givre_avancer(30), 30);
    assert_eq!(givre_neige_masses_preparer() as usize, CASES);
    let masses = lire_les_masses(CASES);
    assert!(masses.iter().any(|&m| m > 0));

    // Les refus ne changent rien.
    for (jeu, a, b, c, d, code) in [
        (19_u32, 400_000_u32, 1_300_000_u32, 3_000_u32, 70_000_u32, 1_u32),
        (u32::MAX, u32::MAX, u32::MAX, u32::MAX, u32::MAX, 1),
        (6, 1_000_001, 1_900_000, 50_000, 15_000, 2),
        (6, 800_000, 999_999, 50_000, 15_000, 3),
        (6, 800_000, 8_000_001, 50_000, 15_000, 3),
        (6, 800_000, 1_900_000, 1_000_001, 15_000, 4),
        (6, 800_000, 1_900_000, 50_000, 1_000_001, 5),
    ] {
        assert_eq!(givre_neige_demarrer(jeu, a, b, c, d), code);
        assert_eq!(lire_les_masses(CASES), masses, "un démarrage refusé (code {code}) laisse l'instantané");
    }
    assert_eq!(givre_demarrer(8, 0), 0, "le modèle 8 n'existe pas");
    assert_eq!(lire_les_masses(CASES), masses, "givre_demarrer(8, 0) refusé laisse l'instantané");

    // Un démarrage qui réussit l'efface (le même jeu, un autre jeu).
    for (jeu, pas) in [(6_usize, 0_u32), (11, 12), (18, 5)] {
        assert_eq!(demarrer(6, quatre), 0);
        assert_eq!(givre_neige_masses_preparer() as usize, CASES);
        assert_eq!(demarrer(jeu, quatre_du_jeu(jeu)), 0);
        exiger_pas_d_instantane(&format!("après le démarrage du jeu {jeu}"));
        assert_eq!(givre_avancer(pas), pas);
        exiger_pas_d_instantane(&format!("après {pas} pas du jeu {jeu}, sans préparer"));
        let nombre = givre_neige_masses_preparer();
        let (neige, _) = neige_jouee(jeu, quatre_du_jeu(jeu), pas);
        exiger_les_masses_de(&neige, nombre, &format!("jeu {jeu} à {pas} pas, après le démarrage"));
    }
    // Un `givre_demarrer` qui réussit l'efface.
    for (modele, graine) in [(1_u32, 0_u32), (3, 1), (0, 1), (4, 2), (7, 3)] {
        assert_eq!(demarrer(6, quatre), 0);
        assert_eq!(givre_neige_masses_preparer() as usize, CASES);
        assert_eq!(givre_demarrer(modele, graine), 1);
        exiger_pas_d_instantane(&format!("après givre_demarrer({modele}, {graine})"));
    }
}

/// juste : sans neige, la préparation rend 0 et rien n'est lisible (V-V8-05). Pour chacun des modèles de 0 à 7 démarrés (la DLA, les trois flocons de Reiter, les quatre claquages), `givre_neige_masses_preparer()` rend 0 (pas `u32::MAX`, pas un nombre de cases),
/// `givre_neige_masse(i)` rend `u32::MAX` pour `i` de 0, 15 150, 30 300 et `u32::MAX`, et ce **après des pas** aussi ; la préparation ne change rien à ce que le modèle dit (temps, durée, sites, rayon, huit mots de l'empreinte), ni au tampon de l'image nette
/// d'un flocon (préparé avant : toujours là). Un `u32::MAX` rendu à la place de 0, ou un instantané d'une neige d'avant gardé après un autre modèle, se voient.
///
/// Ne couvre pas : l'absence de tout démarrage (voir `etape_v8_neuf.rs`).
#[test]
#[cfg_attr(not(feature = "etape-v8"), ignore)]
fn sans_neige_la_preparation_rend_zero() {
    let _verrou = verrou();
    for modele in 0..8_u32 {
        // Une neige préparée avant : le démarrage du modèle l'efface (voir le test précédent).
        assert_eq!(demarrer(6, quatre_du_jeu(6)), 0);
        assert_eq!(givre_neige_masses_preparer() as usize, CASES);
        assert_eq!(givre_demarrer(modele, 1), 1);
        for pas in [0_u32, 7] {
            assert_eq!(givre_avancer(pas), pas, "modèle {modele}");
            if modele == 1 || modele == 3 {
                assert_eq!(givre_nette_preparer(62, 2), 0, "un flocon a une image nette");
            }
            let avant = (
                givre_temps(),
                givre_duree(),
                givre_sites(),
                givre_rayon(),
                (0..8).map(|i| givre_empreinte_mot(i)).collect::<Vec<_>>(),
            );
            let tampon = (givre_nette_largeur(), givre_nette_hauteur(), givre_nette_pixel(30, 30));
            assert_eq!(givre_neige_masses_preparer(), 0, "modèle {modele} : pas de neige, pas de masses");
            for i in [0_u32, 15_150, 30_300, u32::MAX] {
                assert_eq!(givre_neige_masse(i), ABSENT, "modèle {modele} : la masse {i}");
            }
            let apres = (
                givre_temps(),
                givre_duree(),
                givre_sites(),
                givre_rayon(),
                (0..8).map(|i| givre_empreinte_mot(i)).collect::<Vec<_>>(),
            );
            assert_eq!(avant, apres, "modèle {modele} : la préparation des masses ne change pas l'état");
            assert_eq!(
                tampon,
                (givre_nette_largeur(), givre_nette_hauteur(), givre_nette_pixel(30, 30)),
                "modèle {modele} : ni l'image nette"
            );
        }
    }
}

/// juste : lire et préparer n'effacent et ne changent rien (V-V8-06). Sur une neige (le jeu 8 à 60 pas, l'image nette `(62, 2)` préparée), la photographie de la vitrine (le temps, la durée, les sites, le rayon, la compacité, les huit mots de l'empreinte,
/// **chaque pixel** de l'image d'un pixel par case et du tampon de l'image nette, les dix réglages, les huit mesures, `givre_mesurer`) est la même **avant, après les 30 301 lectures, après une préparation des masses et après cinq préparations
/// de plus** : l'état, le temps, l'empreinte, l'image d'un pixel par case et l'image nette sont rendus octet pour octet. Puis l'empreinte du jeu 6 joué par lots de 5 pas **avec une préparation et des lectures entre chaque lot** est celle de la bibliothèque
/// (`c82b34ee…`, 1300 pas : la même qu'en natif sous Linux) : préparer les masses ne change pas le calcul. Une lecture qui modifie l'état, ou une préparation qui change l'empreinte, se voit.
///
/// Ne couvre pas : le module WebAssembly (voir `etape_v8_module.rs`, qui compare le même cristal sous Node).
#[test]
#[cfg_attr(not(feature = "etape-v8"), ignore)]
fn lire_et_preparer_ne_changent_rien() {
    let _verrou = verrou();
    let quatre = quatre_du_jeu(8);
    assert_eq!(demarrer(8, quatre), 0);
    assert_eq!(givre_avancer(60), 60);
    assert_eq!(givre_nette_preparer(62, 2), 0);
    let avant = photographier();
    assert_ne!(avant.nette.0, 0, "le tampon de l'image nette est là");
    assert_eq!(photographier(), avant, "la photographie ne change pas la vitrine");
    assert_eq!(givre_neige_masses_preparer() as usize, CASES);
    assert_eq!(photographier(), avant, "après une préparation des masses");
    let _ = lire_les_masses(CASES);
    assert_eq!(photographier(), avant, "après 30 301 lectures");
    for _ in 0..5 {
        assert_eq!(givre_neige_masses_preparer() as usize, CASES);
    }
    assert_eq!(photographier(), avant, "après cinq préparations de plus");

    // Le calcul n'a pas bougé : le jeu 6 (figure 10, cristal 1 sur 6, 1300 pas) par lots de 5 pas, une préparation et une lecture entre chaque lot.
    assert_eq!(givre_neige_demarrer(6, 800_000, 1_900_000, 50_000, 15_000), 0);
    let mut joues = 0;
    while joues < 1300 {
        let lot = givre_avancer(5);
        assert!(lot > 0, "la neige s'arrête avant 1300 pas : {joues}");
        joues += lot;
        assert_eq!(givre_neige_masses_preparer() as usize, CASES);
        assert!(givre_neige_masse(INDICE_DU_GERME as u32) > 0, "le germe a une masse");
    }
    assert_eq!(joues, 1300);
    let empreinte: String = (0..8).map(|i| format!("{:08x}", givre_empreinte_mot(i))).collect();
    let (native, _) = neige_jouee(6, [800_000, 1_900_000, 50_000, 15_000], 1300);
    assert_eq!(empreinte, native.empreinte(), "l'empreinte du jeu 6 à 1300 pas est celle de la bibliothèque");
    assert_eq!(
        empreinte, "c82b34ee42c9b14385529c3a5b40371ec40ecc03f5e9172c15b37487ff89217d",
        "et celle du module de G3 (régression)"
    );
}

/// juste : les bornes de `i` (V-V8-07). Pour le jeu 5 à 100 pas, `givre_neige_masse` rend la masse de la bibliothèque pour `i = 0`, `1`, `15 149`, `15 150` (le germe : sa masse est strictement positive), `15 151`, `30 299` et `30 300` (la dernière case), et `u32::MAX`
/// pour `i = 30 301` (le premier hors du tampon), `30 302`, `65 535`, `65 536`, `2^31`, `u32::MAX − 1` et `u32::MAX` ; aucune masse n'est `u32::MAX` ; une masse lue avant la préparation (une neige démarrée, jamais préparée) est `u32::MAX`.
///
/// Ne couvre pas : les valeurs de toutes les cases (voir `les_masses_sont_la_lecture_de_la_neige`).
#[test]
#[cfg_attr(not(feature = "etape-v8"), ignore)]
fn les_bornes_de_i() {
    let _verrou = verrou();
    let quatre = quatre_du_jeu(5);
    assert_eq!(demarrer(5, quatre), 0);
    assert_eq!(givre_avancer(100), 100);
    assert_eq!(givre_neige_masse(15_150), ABSENT, "jamais préparée");
    assert_eq!(givre_neige_masse(0), ABSENT, "jamais préparée");
    assert_eq!(givre_neige_masses_preparer() as usize, CASES);
    let (neige, _) = neige_jouee(5, quatre, 100);
    let attendues = masses_de(&neige);
    for i in [0_usize, 1, 15_149, 15_150, 15_151, 30_299, 30_300] {
        assert_eq!(givre_neige_masse(i as u32), attendues[i], "la masse {i}");
    }
    assert!(attendues[INDICE_DU_GERME] > 0, "le germe a une masse");
    for i in [30_301_u32, 30_302, 65_535, 65_536, 1 << 31, u32::MAX - 1, u32::MAX] {
        assert_eq!(givre_neige_masse(i), ABSENT, "la masse {i} est hors du tampon");
    }
    assert!((0..CASES as u32).all(|i| givre_neige_masse(i) != ABSENT), "aucune masse n'est u32::MAX");
}

/// juste : jamais de panique, et les règles tiennent dans le désordre (V-V8-08). Huit cents appels tirés (graine écrite dans le test) parmi : démarrer une neige (un jeu quelconque, des réglages valides ou absurdes), démarrer un autre modèle (0 à 7) ou un modèle qui
/// n'existe pas, jouer 0 à 40 pas d'une neige (finie, au mur ou non), 0 à 4 d'un autre modèle, préparer les masses, préparer l'image nette (valide ou non), `givre_mesurer`, lire une masse (`i` tiré parmi les bornes, des cases tirées et des valeurs absurdes jusqu'à `u32::MAX`). Un petit modèle
/// de la doc, écrit dans le test, suit ce qui doit être vrai : l'instantané existe après une préparation **de neige** et jusqu'au prochain démarrage réussi (un démarrage refusé, des pas, l'image nette, `givre_mesurer` n'y changent rien) ; la préparation rend
/// 30 301 pour une neige en cours et 0 sinon ; une masse hors du tampon ou sans instantané est `u32::MAX` ; les masses lues de quelques cases restent **les mêmes** tant qu'aucune préparation ni aucun démarrage réussi n'a lieu. Chaque `givre_neige_masse` d'une case du
/// tampon est strictement inférieure à `u32::MAX`.
///
/// Ne couvre pas : les valeurs lues (voir `les_masses_sont_la_lecture_de_la_neige`) ; l'état sans aucun démarrage (voir `etape_v8_neuf.rs`).
#[test]
#[cfg_attr(not(feature = "etape-v8"), ignore)]
fn jamais_de_panique_et_les_regles_dans_le_desordre() {
    let _verrou = verrou();
    // L'état global vient des tests d'avant : on repart d'un démarrage qui réussit (il efface l'instantané).
    assert_eq!(givre_demarrer(1, 0), 1);
    let mut hasard = Hasard(0x00DE_5017);
    let (mut neige_en_cours, mut instantane) = (false, false);
    let mut vues: Option<Vec<u32>> = None;
    let sondes: [u32; 6] = [0, 15_149, 15_150, 15_151, 20_000, 30_300];
    for appel in 0..800 {
        let operation = hasard.entre(0, 9);
        match operation {
            0 | 1 => {
                let jeu = hasard.entre(0, 20);
                let quatre = if hasard.entre(0, 3) == 0 {
                    [hasard.suivant() as u32, hasard.suivant() as u32, hasard.suivant() as u32, hasard.suivant() as u32]
                } else {
                    [
                        hasard.entre(300_000, 1_000_000),
                        hasard.entre(1_000_000, 3_000_000),
                        hasard.entre(0, 100_000),
                        hasard.entre(0, 150_000),
                    ]
                };
                let code = givre_neige_demarrer(jeu, quatre[0], quatre[1], quatre[2], quatre[3]);
                if code == 0 {
                    (neige_en_cours, instantane, vues) = (true, false, None);
                }
            }
            2 => {
                let modele = hasard.entre(0, 9);
                if givre_demarrer(modele, hasard.suivant() as u32) == 1 {
                    (neige_en_cours, instantane, vues) = (false, false, None);
                }
            }
            3 | 4 => {
                let _ = givre_avancer(hasard.entre(0, if neige_en_cours { 40 } else { 4 }));
            }
            5 => {
                let nombre = givre_neige_masses_preparer();
                assert_eq!(nombre as usize, if neige_en_cours { CASES } else { 0 }, "appel {appel} : la préparation");
                instantane = neige_en_cours;
                vues = None;
            }
            6 => {
                let _ = givre_nette_preparer(hasard.entre(0, 9) * 2, hasard.entre(0, 5));
            }
            7 => {
                let _ = givre_mesurer();
            }
            _ => {
                let i = match hasard.entre(0, 3) {
                    0 => sondes[hasard.entre(0, 5) as usize],
                    1 => hasard.entre(0, 30_400),
                    2 => hasard.suivant() as u32,
                    _ => u32::MAX - hasard.entre(0, 2),
                };
                let masse = givre_neige_masse(i);
                if !instantane || i as usize >= CASES {
                    assert_eq!(masse, ABSENT, "appel {appel} : la masse {i} (instantané : {instantane})");
                } else {
                    assert_ne!(masse, ABSENT, "appel {appel} : la masse {i} est dans le tampon");
                }
            }
        }
        // Les masses des sondes ne changent que par une préparation ou un démarrage réussi.
        let lues: Vec<u32> = sondes.iter().map(|&i| givre_neige_masse(i)).collect();
        if instantane {
            assert!(lues.iter().all(|&m| m != ABSENT), "appel {appel} : l'instantané existe");
            match &vues {
                Some(avant) => assert_eq!(
                    &lues, avant,
                    "appel {appel} : les masses ont changé sans préparation (opération {operation})"
                ),
                None => vues = Some(lues),
            }
        } else {
            assert!(lues.iter().all(|&m| m == ABSENT), "appel {appel} : pas d'instantané, pas de masse");
        }
    }
}

/// juste : les modèles 0 à 7 et les trente-deux fonctions d'avant ne changent pas (V-V8-09). Les modèles 0 à 7 (graine 1) jusqu'à 50 pas ou à leur durée ont l'empreinte du code natif (la DLA de graine 1 jusqu'au bout, 2999 particules, celle de
/// `references/dla-graine-1.empreinte`, la plaque jusqu'au bout celle de `references/reiter-plaque.empreinte`), leur image d'un pixel par case et, pour les flocons, leur image nette `(62, 2)`, **avec une préparation des masses et des lectures entre chaque étape** ; la neige du jeu 6 à 300 pas (lots de 7, préparations entre les lots) a, pour chaque pixel de
/// l'image nette `(62, 2)`, la couleur de l'image rendue par `rendu_hex::rendre` appliqué à `valeurs_du_cristal` (le champ de G3). Les nouvelles fonctions ne dérangent aucune des anciennes.
///
/// Ne couvre pas : les trente-deux fonctions une à une (leurs tests sont ceux de leurs étapes, que cette étape ne change pas).
#[test]
#[cfg_attr(not(feature = "etape-v8"), ignore)]
fn les_modeles_et_les_fonctions_d_avant_ne_changent_pas() {
    let _verrou = verrou();
    for modele in 0..8_u32 {
        let pas_voulus = if modele == 0 { 2999 } else { 50 };
        assert_eq!(givre_demarrer(modele, 1), 1);
        let mut joues = 0;
        while joues < pas_voulus {
            let lot = givre_avancer(37.min(pas_voulus - joues));
            assert!(lot > 0, "modèle {modele} : le scénario finit avant {pas_voulus}");
            joues += lot;
            assert_eq!(givre_neige_masses_preparer(), 0);
            let _ = givre_neige_masse(0);
        }
        let mots: Vec<u32> = (0..8).map(|i| givre_empreinte_mot(i)).collect();
        let empreinte: String = mots.iter().map(|mot| format!("{mot:08x}")).collect();
        if modele < 4 {
            let natif = e5::natif(modele, 1, joues);
            assert_eq!(empreinte, natif.empreinte, "modèle {modele} à {joues} pas : l'empreinte");
            let mut rvb = Vec::new();
            for y in 0..givre_hauteur() {
                for x in 0..givre_largeur() {
                    let c = givre_pixel(x, y);
                    rvb.extend_from_slice(&[(c >> 16) as u8, (c >> 8) as u8, c as u8]);
                }
            }
            assert!(rvb == natif.rvb, "modèle {modele} : l'image d'un pixel par case");
            if modele == 0 {
                assert_eq!(empreinte, e5::reference("dla-graine-1.empreinte"), "la DLA de graine 1 jusqu'au bout");
            }
        } else {
            let natif = etape_v8_commun::v7::d3::natif_claquage(modele - 4, 1, joues);
            assert_eq!(empreinte, natif.empreinte, "modèle {modele} à {joues} pas : l'empreinte du claquage");
        }
    }
    // La plaque jusqu'au bout.
    assert_eq!(givre_demarrer(1, 1), 1);
    while givre_avancer(41) > 0 {
        assert_eq!(givre_neige_masses_preparer(), 0);
    }
    let plaque: String = (0..8).map(|i| format!("{:08x}", givre_empreinte_mot(i))).collect();
    assert_eq!(plaque, e5::reference("reiter-plaque.empreinte"), "la plaque jusqu'au bout");

    // La neige : l'image nette est celle de G3, après des préparations des masses entre les lots.
    let quatre = quatre_du_jeu(6);
    assert_eq!(demarrer(6, quatre), 0);
    let mut joues = 0;
    while joues < 300 {
        joues += givre_avancer(7.min(300 - joues));
        assert_eq!(givre_neige_masses_preparer() as usize, CASES);
    }
    assert_eq!(givre_nette_preparer(62, 2), 0);
    let (neige, _) = neige_jouee(6, quatre, 300);
    let attendue = etape_v8_commun::g3::image_nette_attendue(&neige, 62, 2);
    let mut lue = Vec::new();
    for y in 0..62 {
        for x in 0..62 {
            let c = givre_nette_pixel(x, y);
            lue.extend_from_slice(&[(c >> 16) as u8, (c >> 8) as u8, c as u8]);
        }
    }
    assert!(lue == attendue, "l'image nette de la neige est celle de G3 (rendre appliqué à valeurs_du_cristal)");
    assert_eq!(somme_des_masses(&lire_les_masses(CASES)), somme_des_masses(&masses_de(&neige)));
}

/// juste : la préparation et la lecture des masses ne touchent pas à **la mesure gardée d'un amas** (V-V8-42 ; la doc : « ni les réglages et les mesures »). Pour la DLA (modèle 0, graine 1, 1500 pas) et le claquage `η = 0` (modèle 4, 399 pas), `givre_mesurer()` garde une courbe de corrélation
/// (au moins un point, une dimension qui n'est pas `NaN`) ; puis deux `givre_neige_masses_preparer()` (0 : pas de neige) et des lectures de `givre_neige_masse` (toutes `u32::MAX`) : la dimension (bit pour bit) et chaque point de la courbe (rayon, valeur bit pour bit) sont **inchangés**.
/// Une préparation qui efface la mesure (le `etat.mesure = None` de `givre_avancer` recopié) se voit : c'est un mutant du juge (J22) qui survivait aux tests de l'écrivain.
///
/// Ne couvre pas : la mesure elle-même (étapes Z1 et D3) ; la neige (elle n'a pas de mesure de corrélation : `givre_mesurer` y rend 0).
#[test]
#[cfg_attr(not(feature = "etape-v8"), ignore)]
fn la_preparation_ne_touche_pas_a_la_mesure_de_l_amas() {
    use givre_lab::vitrine::{givre_correlation_rayon, givre_correlation_valeur, givre_dimension};
    let _verrou = verrou();
    for (modele, pas) in [(0_u32, 1500_u32), (4, 399)] {
        assert_eq!(givre_demarrer(modele, 1), 1);
        assert_eq!(givre_avancer(pas), pas, "modèle {modele}");
        let points = givre_mesurer();
        assert!(points > 0, "modèle {modele} : l'amas a une mesure");
        assert!(!givre_dimension().is_nan(), "modèle {modele} : la mesure a une dimension");
        let mesure = || {
            let courbe: Vec<(u32, u64)> =
                (0..points).map(|i| (givre_correlation_rayon(i), givre_correlation_valeur(i).to_bits())).collect();
            (givre_dimension().to_bits(), courbe)
        };
        let avant = mesure();
        assert_eq!(givre_neige_masses_preparer(), 0, "modèle {modele} : pas de neige");
        for i in [0_u32, 15_150, 30_300, u32::MAX] {
            assert_eq!(givre_neige_masse(i), ABSENT, "modèle {modele} : la masse {i}");
        }
        assert_eq!(givre_neige_masses_preparer(), 0);
        assert_eq!(mesure(), avant, "modèle {modele} : la préparation des masses a changé la mesure gardée");
    }
}

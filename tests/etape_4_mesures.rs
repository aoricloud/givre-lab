//! Étape 4 : les mesures, l'écriture canonique, l'empreinte, le CSV et l'aperçu de `cinetique`, et deux cas à solution connue
//! (la marche aléatoire `J = 0`, la face de 3 × 3 à l'équilibre). Valeurs attendues : `references/etape-4-valeurs.md`,
//! identifiants `V-MES-xx`.
//!
//! Les valeurs écrites en dur viennent d'outils indépendants du Rust, nommés dans la fiche : un simulateur naïf Python
//! (`simulateur_naif.py`, écrit d'après la doc, avec un modèle de PCG32 et une table `e^(−k/16)` recalculée en `decimal`), le
//! `hashlib` de Python pour les SHA-256, une énumération exacte de toutes les configurations de la face de 3 × 3
//! (`enumeration_l3.rs`, recoupée par `enumeration_l3.py`).

use givre_lab::cinetique::{
    BALAYAGES_MAX, Face, TAILLE_MAX, TAILLE_MIN, rugosite_moyenne, simuler, vitesse_croissance,
};
use givre_lab::math::EXP_NEG;
use std::panic::catch_unwind;

fn canonique(face: &Face) -> Vec<u8> {
    let mut octets = Vec::new();
    face.ecrire_canonique(&mut octets);
    octets
}

fn hex(octets: &[u8]) -> String {
    octets.iter().map(|o| format!("{o:02x}")).collect()
}

/// juste : octets écrits à la main d'après la doc, SHA-256 de Python `hashlib` : l'**écriture canonique** d'une face de 3 × 3
/// (`0 1 -2 / 3 0 0 / -1 5 2`, `J = 16`, `Δμ = −5`, graine 7, aucun tirage) est ces 72 octets : l'étiquette `GIVRE004`
/// (`47 49 56 52 45 30 30 34`), `L = 3` (`03000000`), `J = 16` (`10000000`), `Δμ = −5` en `i32` petit-boutiste (`fbffffff`), les neuf
/// hauteurs en `i32` petit-boutistes dans l'ordre `y·3 + x` (`00000000 01000000 feffffff 03000000 00000000 00000000 ffffffff
/// 05000000 02000000`), puis l'état de `Alea::new(7)` (`d1a0bdb386a42205 6d00000000000000`, voir `etape_0_lancer`). L'empreinte est le
/// SHA-256 de ces octets. `ecrire_canonique` **ajoute** à la suite d'un `Vec` déjà rempli sans toucher à ce qu'il contient (V-MES-01).
/// Après un essai accepté (graine construite : `Face::new(4, 4, 4, 8755884821639507505)`, ajout à `k = 12`, tirage `r =
/// EXP_NEG[12] − 1`, colonne 8) les octets sont ceux de l'état à 100 octets où la hauteur 8 vaut 1 et où le générateur a tiré
/// trois fois (`9a7f956c25e9a560 6d00000000000000`).
///
/// Ne couvre pas : les gros états (voir l'empreinte des cas simulés).
#[test]
#[cfg_attr(not(feature = "etape-4"), ignore)]
fn ecriture_canonique_a_la_main() {
    let face = Face::depuis_hauteurs(3, 16, -5, 7, &[0, 1, -2, 3, 0, 0, -1, 5, 2]);
    let attendu = concat!(
        "4749565245303034", // GIVRE004
        "03000000",         // L = 3
        "10000000",         // J = 16
        "fbffffff",         // Δμ = -5
        "00000000",
        "01000000",
        "feffffff",
        "03000000",
        "00000000",
        "00000000",
        "ffffffff",
        "05000000",
        "02000000",
        "d1a0bdb386a42205",
        "6d00000000000000", // Alea::new(7)
    );
    assert_eq!(hex(&canonique(&face)), attendu);
    assert_eq!(face.empreinte(), "60c39a9a09441822c2a375be6031d31272ef9a169879d153f200ff1adc799343");
    // ajoute à la suite : le préfixe est intact
    let mut sortie = vec![0xAA, 0xBB, 0xCC];
    face.ecrire_canonique(&mut sortie);
    assert_eq!(&sortie[..3], [0xAA, 0xBB, 0xCC]);
    assert_eq!(hex(&sortie[3..]), attendu);

    let mut face = Face::new(4, 4, 4, 8_755_884_821_639_507_505);
    assert!(face.essai());
    let attendu = concat!(
        "4749565245303034",
        "04000000",
        "04000000",
        "04000000", // GIVRE004, L = 4, J = 4, Δμ = 4
        "00000000",
        "00000000",
        "00000000",
        "00000000",
        "00000000",
        "00000000",
        "00000000",
        "00000000", // colonnes 0 à 7
        "01000000", // colonne 8 : un ajout
        "00000000",
        "00000000",
        "00000000",
        "00000000",
        "00000000",
        "00000000",
        "00000000", // colonnes 9 à 15
        "9a7f956c25e9a560",
        "6d00000000000000", // le générateur après trois tirages
    );
    assert_eq!(hex(&canonique(&face)), attendu);
    assert_eq!(face.empreinte(), "3368b14002c5dd4e33b978ada6a0d91626331484eacbadb6f4bd6612ebeae9cc");
}

/// juste : simulateur naïf Python d'après la doc, SHA-256 de `hashlib` : l'**empreinte et les sommes** de cinq faces menées de
/// leur état plan par `balayages(n)` (V-MES-02) : `L = 5, J = 12, Δμ = 4`, graine 3, 20 balayages ; `L = 8, J = 16, Δμ = −6`,
/// graine 11, 30 balayages ; `L = 6, J = 0, Δμ = 8`, graine 2, 25 balayages ; `L = 8, J = 24, Δμ = 12`, graine 5, 40 balayages
/// (une face qui reste plane) ; `L = 7, J = 100, Δμ = 20`, graine 9, 10 balayages (idem). Pour chacune, les valeurs de
/// `S = Σ h`, `Q = Σ h²`, `N·Q − S²`, et l'empreinte, écrites ici, sont celles de Python ; pour les deux premières les hauteurs
/// aussi. Les valeurs sont écrites en dur : une écriture canonique, un nombre de tirages ou une règle fautifs les changent.
///
/// Ne couvre pas : l'égalité de ces faces avec un autre simulateur Rust (c'est `etape_4_exact`).
#[test]
#[cfg_attr(not(feature = "etape-4"), ignore)]
#[allow(clippy::type_complexity)]
fn empreinte_et_sommes_des_cas_python() {
    // (L, J, Δμ, graine, balayages, S, Q, N·Q − S², empreinte)
    let cas: [(usize, u32, i32, u64, usize, i64, u128, i128, &str); 5] = [
        (5, 12, 4, 3, 20, 19, 19, 114, "891ed75a31aa5a3eb4da29823b6c6acf1e32b8b17720649bae7fc57313816abc"),
        (8, 16, -6, 11, 30, -18, 22, 1084, "6aa45e799be5ee5079e79087fd975b579d8b2e1ec3e1b74dcdac94a469c740ee"),
        (6, 0, 8, 2, 25, 168, 1540, 27216, "80297bc34461125e41c9941f3fb236b49295437baaf04561d37d361c92ee7ba1"),
        (8, 24, 12, 5, 40, 0, 0, 0, "683a6ad4fbcbc4160f7fc01cdf91e6083f167cac17b1f519054e8ded687b633e"),
        (7, 100, 20, 9, 10, 0, 0, 0, "67740d745ed0a7688bac65a3e2d1ad189fc02ca377b376f81e4e6c3f52bbc095"),
    ];
    for (l, j, mu, graine, balayages, s, q, numerateur, empreinte) in cas {
        let mut face = Face::new(l, j, mu, graine);
        face.balayages(balayages);
        assert_eq!(face.somme_hauteurs(), s, "L={l} J={j} Δμ={mu} : S");
        assert_eq!(face.somme_carres(), q, "L={l} J={j} Δμ={mu} : Q");
        let n = (l * l) as i128;
        assert_eq!(n * q as i128 - i128::from(s) * i128::from(s), numerateur);
        assert_eq!(face.rugosite_carree(), numerateur as f64 / (n * n) as f64, "L={l} J={j} Δμ={mu} : w²");
        assert_eq!(face.hauteur_moyenne(), s as f64 / n as f64, "L={l} J={j} Δμ={mu} : <h>");
        assert_eq!(face.empreinte(), empreinte, "L={l} J={j} Δμ={mu} graine {graine} : empreinte");
    }
    let mut face = Face::new(5, 12, 4, 3);
    face.balayages(20);
    assert_eq!(face.hauteurs(), [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 1, 0, 0, 0, 0, 1, 1, 1, 1, 0, 1]);
    let mut face = Face::new(8, 16, -6, 11);
    face.balayages(30);
    assert_eq!(
        face.hauteurs(),
        [
            -1, -1, 0, 0, 0, 0, 0, -1, -1, -1, 0, 0, 0, 0, -1, -1, -1, -1, 0, 0, 0, 0, -1, -2, -1, -1, 0, 0, 0, 0, -1,
            -2, 0, 0, 0, 0, 0, 0, 0, -1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0
        ]
    );
}

/// juste : calcul à la main : les **sommes et les mesures** sur une face donnée (V-MES-03). Face de 3 × 3 `0 1 -2 / 3 0 0 / -1 5 2` :
/// `S = 0+1−2+3+0+0−1+5+2 = 8`, `Q = 0+1+4+9+0+0+1+25+4 = 44`, `N = 9`, `N·Q − S² = 396 − 64 = 332`, donc `<h> = 8/9` (`8.0 / 9.0`)
/// et `w² = 332/81` (`332.0 / 81.0`), égalités exactes en `f64` (une conversion et une division : la doc). Une face de hauteur
/// uniforme (`-3` partout, `4` partout) a `w² = 0` exactement et `<h>` égale à cette hauteur ; une face à deux niveaux `0` et `1`
/// par moitié (4 × 4, 8 colonnes à 1) a `S = 8`, `Q = 8`, `w² = (16·8 − 64)/256 = 1/4` ; une face de 4 × 4 où une seule colonne vaut
/// `1000` a `S = 1000`, `Q = 10^6`, `w² = (16·10^6 − 10^6)/256 = 58593.75`.
///
/// Ne couvre pas : les faces de hauteurs au-delà de `2^20` (précondition de la doc).
#[test]
#[cfg_attr(not(feature = "etape-4"), ignore)]
fn mesures_a_la_main() {
    let face = Face::depuis_hauteurs(3, 16, 0, 1, &[0, 1, -2, 3, 0, 0, -1, 5, 2]);
    assert_eq!((face.somme_hauteurs(), face.somme_carres()), (8, 44));
    assert_eq!(face.hauteur_moyenne(), 8.0 / 9.0);
    assert_eq!(face.rugosite_carree(), 332.0 / 81.0);
    for c in [-3, 4] {
        let face = Face::depuis_hauteurs(3, 16, 0, 1, &[c; 9]);
        assert_eq!((face.hauteur_moyenne(), face.rugosite_carree()), (f64::from(c), 0.0));
        assert_eq!((face.somme_hauteurs(), face.somme_carres()), (9 * i64::from(c), 9 * (c * c) as u128));
    }
    let mut deux_niveaux = [0_i32; 16];
    deux_niveaux[..8].fill(1);
    let face = Face::depuis_hauteurs(4, 16, 0, 1, &deux_niveaux);
    assert_eq!((face.somme_hauteurs(), face.somme_carres()), (8, 8));
    assert_eq!((face.hauteur_moyenne(), face.rugosite_carree()), (0.5, 0.25));
    let mut une_colonne = [0_i32; 16];
    une_colonne[5] = 1000;
    let face = Face::depuis_hauteurs(4, 16, 0, 1, &une_colonne);
    assert_eq!((face.somme_hauteurs(), face.somme_carres()), (1000, 1_000_000));
    assert_eq!(face.rugosite_carree(), 58593.75);
    assert_eq!(face.hauteur_moyenne(), 62.5);
}

/// juste : la doc, recalculée dans le test : **`vitesse_croissance` et `rugosite_moyenne`** (V-MES-04). Sur une face de 6 × 6 (`J = 12`,
/// `Δμ = 4`, graine 3, 10 balayages de transitoire), `vitesse_croissance(&mut face, 7)` vaut `(S_après − S_avant) as f64 /
/// (N · 7) as f64` (`N = 36`), la face a avancé de **exactement 7 balayages** (octets canoniques égaux à ceux d'un clone qui a fait
/// `balayages(7)`) ; `rugosite_moyenne(&mut face, 7)` vaut `total as f64 / (N² · 7) as f64` où `total` est la somme, après chacun des
/// 7 balayages, de `N·Q − S²` (entiers exacts ; l'état de départ n'est pas compté), et la face a avancé de 7 balayages ; ces
/// formules sont celles de la doc, comparées `==`. Les deux paniquent pour 0 balayage.
///
/// Ne couvre pas : la valeur physique des mesures (les tests de `etape_4_physique`).
#[test]
#[cfg_attr(not(feature = "etape-4"), ignore)]
fn vitesse_et_rugosite_moyenne_selon_la_doc() {
    let mut depart = Face::new(6, 12, 4, 3);
    depart.balayages(10);
    // vitesse
    let (mut mesuree, mut a_la_main) = (depart.clone(), depart.clone());
    let v = vitesse_croissance(&mut mesuree, 7);
    let avant = a_la_main.somme_hauteurs();
    a_la_main.balayages(7);
    assert_eq!(v, (a_la_main.somme_hauteurs() - avant) as f64 / (36 * 7) as f64);
    assert_eq!(canonique(&mesuree), canonique(&a_la_main));
    assert!(v != 0.0, "le cas est vide : la face n'a pas bougé");
    // rugosité moyenne
    let (mut mesuree, mut a_la_main) = (depart.clone(), depart.clone());
    let w = rugosite_moyenne(&mut mesuree, 7);
    let mut total: u128 = 0;
    for _ in 0..7 {
        a_la_main.balayage();
        let (s, q) = (i128::from(a_la_main.somme_hauteurs()), a_la_main.somme_carres() as i128);
        total += (36 * q - s * s) as u128;
    }
    assert_eq!(w, total as f64 / (36 * 36 * 7) as f64);
    assert_eq!(canonique(&mesuree), canonique(&a_la_main));
    assert!(w > 0.0);
    // zéro balayage : panique
    let mut face = depart.clone();
    assert!(catch_unwind(move || vitesse_croissance(&mut face, 0)).is_err());
    let mut face = depart.clone();
    assert!(catch_unwind(move || rugosite_moyenne(&mut face, 0)).is_err());
}

/// juste : calcul exact, la variance est calculée et non mesurée : **`J = 0` est une marche aléatoire sans interaction** (V-MES-05).
/// Sans liaison, `k = ∓Δμ` ne dépend ni de la colonne ni des hauteurs : avec `Δμ > 0` tout ajout est accepté et chaque retrait
/// l'est avec la probabilité `p = EXP_NEG[Δμ en seizièmes] / 2^32`. Un essai ajoute donc 1 (probabilité `1/2`), retire 1
/// (probabilité `p/2`) ou ne fait rien, indépendamment des autres : la variation de `Σ h` après `M` essais a pour moyenne `M(1 − p)/2`
/// et pour variance `M σ²`, `σ² = (1 + p)/2 − ((1 − p)/2)²`. La vitesse par balayage vaut donc **exactement `(1 − p)/2` en
/// moyenne**, d'écart-type `σ/√M` (`M = N` essais par balayage fois le nombre de balayages). Face de 16 × 16 plane,
/// 4000 balayages (`M = 1 024 000`), `Δμ/kT = 0,25 ; 0,5 ; 1 ; 2` (4, 8, 16, 32 seizièmes), graines 1 et 2 : la vitesse est à moins de
/// `5 σ/√M` de `(1 − p)/2` (`σ/√M` vaut `9,2e-4` à `Δμ = 0,25`, `6,0e-4` à `Δμ = 2`). Un écart d'un cran de table (`EXP_NEG[k ± 1]`) vaut
/// `25` écarts-types à `Δμ = 0,25` et `7` à `Δμ = 2` ; un signe inversé ou un retrait traité comme un ajout, beaucoup plus. C'est la
/// référence de la linéarité à haute température (`(1 − p)/2 ≈ Δμ/2` pour `Δμ` petit).
///
/// Rejoué sur 25 couples de graines que le test ne contient pas (100000 à 102401) : aucun échec.
///
/// Ne couvre pas : `J > 0` (aucun calcul fermé), les faces où `Δμ` change.
#[test]
#[cfg_attr(not(feature = "etape-4"), ignore)]
fn j_nul_marche_aleatoire_vitesse_exacte() {
    for mu in [4_i32, 8, 16, 32] {
        let p = EXP_NEG[mu as usize] as f64 / 4_294_967_296.0;
        let attendu = (1.0 - p) / 2.0;
        let sigma = ((1.0 + p) / 2.0 - attendu * attendu).sqrt();
        let m = 256.0 * 4000.0;
        let tolerance = 5.0 * sigma / f64::sqrt(m);
        for graine in [1_u64, 2] {
            let mut face = Face::new(16, 0, mu, graine);
            let v = vitesse_croissance(&mut face, 4000);
            eprintln!(
                "J=0 Δμ={mu} graine {graine} : v = {v:.5}, attendu {attendu:.5}, écart {:.2} σ",
                (v - attendu) / (sigma / f64::sqrt(m))
            );
            assert!(
                (v - attendu).abs() <= tolerance,
                "Δμ = {mu}, graine {graine} : v = {v}, attendu {attendu} ± {tolerance}"
            );
        }
    }
}

/// juste : valeurs EXACTES de l'énumération de toutes les configurations, tolérance mesurée sur le prototype : **l'équilibre
/// de Boltzmann**, sur la plus petite face permise (3 × 3, 9 colonnes, 18 liaisons) à `Δμ = 0` (V-MES-06). La dynamique de
/// Metropolis de la doc vérifie le bilan détaillé : la loi stationnaire des hauteurs relatives est `e^(−H/kT)`. La rugosité moyenne
/// `<w²>` de cette loi se calcule exactement en énumérant les configurations de hauteurs relatives (la colonne 0 fixée à 0,
/// `|h| <= H`) : pour `J/kT = 1 ; 1,25 ; 1,5` (16, 20, 24 seizièmes) elle vaut `0,0605767076 ; 0,0181303680 ; 0,0056972443`
/// (énumération Rust `enumeration_l3.rs` jusqu'à `|h| <= 7`, stabilisée à `1e-9` de `H = 6` à `H = 7`, et recoupée à `H = 2` par le
/// Python `enumeration_l3.py`, qui donne les mêmes 12 chiffres). La moyenne, sur les graines 1 à 30, de la rugosité moyenne sur
/// 20 000 balayages (après 500 de transitoire) est à moins de `4` erreurs-types de cette valeur : `1,4e-3 ; 6e-4 ; 3e-4`.
///
/// Calibration (quatre blocs de 30 graines, 2001, 3001, 4001 et 1 à 30, mêmes durées) : écart-type d'une graine `1,4 à 2,1e-3 ;
/// 0,8 à 1,0e-3 ; 0,35 à 0,5e-3` selon `J`, soit une erreur-type de la moyenne de 30 graines d'environ `3,5e-4 ; 1,6e-4 ; 7,5e-5` ;
/// écarts de la moyenne à la valeur exacte, en erreurs-types : `+1,2 ; −0,9 ; −1,6 ; +1,3` (`J = 16`), `−1,4 ; −0,7 ; −0,6 ;
/// +0,1` (`J = 20`), `+0,5 ; +0,4 ; +0,7 ; +1,3` (`J = 24`) : aucun biais net. Tolérances = `4` erreurs-types (`2,3 % ; 3,3 % ;
/// 5,3 %` de la valeur). Une règle fautive déplace la valeur de bien plus : d'après les valeurs exactes de l'énumération (à `J = 12`,
/// `w² ≈ 0,195`, contre `0,0606` à `J = 16`), un seizième de `J` change `w²` de 25 à 35 %, plus de dix fois la tolérance ; un `<` pour
/// un `<=` dans le décompte des voisins, ou un pas de Metropolis qui ne vérifie pas le bilan détaillé, change la loi stationnaire.
/// Rejoué sur 25 blocs de 30 graines que le test ne contient pas (graines de 100000 à 102400, par blocs de 100) : aucun échec.
///
/// Ne couvre pas : `Δμ ≠ 0` (aucune loi stationnaire : la face croît), les grandes faces (la face de 3 × 3 est la seule énumérable).
#[test]
#[cfg_attr(not(feature = "etape-4"), ignore)]
fn equilibre_exact_de_la_face_de_3_sur_3() {
    let cas: [(u32, f64, f64); 3] =
        [(16, 0.060_576_707_6, 1.4e-3), (20, 0.018_130_368_0, 6e-4), (24, 0.005_697_244_3, 3e-4)];
    for (j, exact, tolerance) in cas {
        let mut mesures = vec![0.0; 30];
        std::thread::scope(|s| {
            for (i, case) in mesures.iter_mut().enumerate() {
                s.spawn(move || {
                    let mut face = Face::new(3, j, 0, i as u64 + 1);
                    face.balayages(500);
                    *case = rugosite_moyenne(&mut face, 20_000);
                });
            }
        });
        let moyenne = mesures.iter().sum::<f64>() / 30.0;
        let ecart_type = (mesures.iter().map(|m| (m - moyenne) * (m - moyenne)).sum::<f64>() / 29.0).sqrt();
        eprintln!("L=3 J={j} : <w²> = {moyenne:.6} (exact {exact}), écart-type par graine {ecart_type:.2e}");
        assert!((moyenne - exact).abs() <= tolerance, "J = {j} : <w²> = {moyenne}, exact {exact} ± {tolerance}");
    }
}

/// Les hauteurs de `face` sommées à la main (sans passer par `somme_hauteurs`).
fn sommes(face: &Face) -> (i128, i128) {
    let s: i128 = face.hauteurs().iter().map(|&h| i128::from(h)).sum();
    let q: i128 = face.hauteurs().iter().map(|&h| i128::from(h) * i128::from(h)).sum();
    (s, q)
}

/// `num / den` arrondi au millionième, demi-cas vers `+∞`, six chiffres : une autre écriture que celle de la doc (quotient et reste
/// de la division euclidienne, le reste doublé comparé au dénominateur).
fn millionieme(num: i128, den: i128) -> String {
    let produit = num * 1_000_000;
    let mut m = produit.div_euclid(den);
    if 2 * produit.rem_euclid(den) >= den {
        m += 1;
    }
    let signe = if m < 0 { "-" } else { "" };
    format!("{signe}{}.{:06}", m.abs() / 1_000_000, m.abs() % 1_000_000)
}

/// Le CSV attendu d'après la doc de `simuler`, par un `Face` mené balayage par balayage.
fn csv_naif(face: &mut Face, balayages: usize) -> (String, usize, usize) {
    let n = (face.taille() * face.taille()) as i128;
    let pas = std::cmp::max(1, balayages / 50);
    let ligne = |face: &Face, t: usize| {
        let (s, q) = sommes(face);
        format!("{t},{},{}\n", millionieme(s, n), millionieme(n * q - s * s, n * n))
    };
    let mut csv = "balayage,hauteur_moyenne,rugosite\n".to_string();
    csv += &ligne(face, 0);
    let (mut demi_positifs, mut demi_negatifs) = (0, 0);
    for t in 1..=balayages {
        face.balayage();
        if t % pas == 0 || t == balayages {
            csv += &ligne(face, t);
            let (s, _) = sommes(face);
            if (2 * (s * 1_000_000).rem_euclid(n)) == n {
                if s > 0 { demi_positifs += 1 } else { demi_negatifs += 1 }
            }
        }
    }
    (csv, demi_positifs, demi_negatifs)
}

/// juste : valeurs de Python pour deux CSV complets, un second code écrit dans le test pour les autres : le **CSV de `simuler`**
/// (V-MES-07). L'en-tête est `balayage,hauteur_moyenne,rugosite` ; une ligne pour `t = 0` et pour chaque multiple de
/// `pas = max(1, balayages/50)` jusqu'à `balayages`, plus une ligne finale pour `t = balayages` si ce n'est pas un multiple ;
/// chaque nombre en décimal avec six chiffres exactement, arrondi au plus proche, demi-cas vers `+∞`. (a) `L = 8, J = 16, Δμ =
/// −6`, graine 11, 100 balayages : 52 lignes (en-tête, 51), SHA-256 du texte `698e19b9…9881` (Python), lignes `0,0.000000,0.000000`,
/// `2,0.000000,0.062500`, `4,0.015625,0.015381`, dernière `100,-0.968750,0.061523` ; (b) `L = 16, J = 12, Δμ = 8`, graine 4, 120 balayages :
/// 62 lignes, SHA-256 `5645c453…d5fe`, dernière `120,7.402344,0.537338` ; (c) quarante-quatre autres jeux (quatre faces, `L = 3, 5, 16, 16`, `Δμ` positif et
/// négatif, onze durées : `balayages = 1, 7, 49, 50, 51, 99, 100, 101, 125, 149, 150`, dont des durées qui ne sont pas multiples de leur pas)
/// comparés à un CSV bâti dans le test (`Face::balayage` mène la face ; sommes de hauteurs recomptées à la main ; formateur
/// écrit autrement que celui de la doc). Parmi eux se trouvent des demi-cas exacts de l'arrondi, pour une moyenne positive
/// comme négative (`L = 16`, `S ≡ 2 mod 4` : `S/256` à un millionième est un demi) : le test vérifie qu'il y en a, des deux signes. La
/// face de `simuler` a avancé de `balayages` balayages (octets canoniques égaux).
///
/// Ne couvre pas : la valeur physique des lignes, `balayages` à 0 (hors du domaine de la directive : 1 au moins).
#[test]
#[cfg_attr(not(feature = "etape-4"), ignore)]
fn csv_de_simuler() {
    use givre_lab::empreinte::sha256_hex;
    let mut face = Face::new(8, 16, -6, 11);
    let texte = simuler(&mut face, 100);
    assert_eq!(texte.lines().count(), 52);
    assert_eq!(sha256_hex(texte.as_bytes()), "698e19b932373b14d593c23a6513e6474ce939e5f37f1ca75b359ae314969881");
    let lignes: Vec<&str> = texte.lines().collect();
    assert_eq!(
        lignes[..4],
        ["balayage,hauteur_moyenne,rugosite", "0,0.000000,0.000000", "2,0.000000,0.062500", "4,0.015625,0.015381"]
    );
    assert_eq!(lignes[51], "100,-0.968750,0.061523");
    assert!(texte.ends_with('\n') && !texte.contains('\r') && !texte.ends_with("\n\n"));

    let mut face = Face::new(16, 12, 8, 4);
    let texte = simuler(&mut face, 120);
    assert_eq!(texte.lines().count(), 62);
    assert_eq!(sha256_hex(texte.as_bytes()), "5645c4530c9addd3e0d0879e0e7f7f643564f40d32aef93fb0da9a161310d5fe");
    assert_eq!(texte.lines().last().unwrap(), "120,7.402344,0.537338");

    let (mut positifs, mut negatifs) = (0, 0);
    let mut numero = 0;
    for (l, mu) in [(3_usize, 4_i32), (5, -8), (16, 8), (16, -8)] {
        for balayages in [1_usize, 7, 49, 50, 51, 99, 100, 101, 125, 149, 150] {
            numero += 1;
            let mut mesuree = Face::new(l, 12, mu, 100 + numero);
            let mut a_la_main = mesuree.clone();
            let (attendu, demi_p, demi_n) = csv_naif(&mut a_la_main, balayages);
            assert_eq!(simuler(&mut mesuree, balayages), attendu, "L={l} Δμ={mu} balayages={balayages}");
            assert_eq!(canonique(&mesuree), canonique(&a_la_main));
            positifs += demi_p;
            negatifs += demi_n;
        }
    }
    assert!(
        positifs >= 3 && negatifs >= 3,
        "pas assez de demi-cas d'arrondi : {positifs} positifs, {negatifs} négatifs"
    );
}

/// juste : calcul à la main et valeurs de Python : l'**aperçu** (V-MES-08) est une image RVB de `L × L` pixels, la ligne `y` de haut
/// en bas, le pixel `(x, y)` au numéro `y·L + x`, `(v, v, v)` avec `v = 255 · (h − h_min) / (h_max − h_min)` en division entière si
/// `h_max > h_min`, et `v = 128` partout sinon. Face de 3 × 3 `0 … 8` : `v = 0, 31, 63, 95, 127, 159, 191, 223, 255` (`255·k/8` par défaut
/// : `31,875 → 31`, `63,75 → 63`, `95,6 → 95`, `127,5 → 127`, …) ; les mêmes hauteurs décalées de `-100` donnent les mêmes niveaux ; hauteurs
/// à deux valeurs (`h_min = 2`, `h_max = 5`) : `0` et `255` seulement, et trois valeurs `0, 1, 2` : `0, 127, 255` (`255/2 = 127,5 → 127`) ;
/// face plane : 128 partout, y compris `-4` partout. Face de 5 × 5 menée par `balayages(20)` (`J = 12, Δμ = 4`, graine 3) : les niveaux
/// de Python `255 255 … 0 …` (deux niveaux : 255 pour la hauteur 1, 0 pour la hauteur 0) ; face de 6 × 6 menée à `J = 0, Δμ = 8`, graine 2, 25
/// balayages, hauteurs de `-8` à `13` : les 36 niveaux de Python (`133, 121, 133, 194, 97, 255, …`).
///
/// Ne couvre pas : le PNG lui-même (voir `etape_4_lancer`).
#[test]
#[cfg_attr(not(feature = "etape-4"), ignore)]
fn apercu_niveaux_de_gris() {
    let niveaux = |face: &Face| -> Vec<u8> {
        let rvb = face.apercu();
        assert_eq!(rvb.len(), 3 * face.taille() * face.taille());
        rvb.chunks(3)
            .map(|p| {
                assert!(p[0] == p[1] && p[1] == p[2], "pixel non gris {p:?}");
                p[0]
            })
            .collect()
    };
    let suite: Vec<i32> = (0..9).collect();
    let attendu = [0_u8, 31, 63, 95, 127, 159, 191, 223, 255];
    assert_eq!(niveaux(&Face::depuis_hauteurs(3, 16, 0, 1, &suite)), attendu);
    let decalee: Vec<i32> = suite.iter().map(|h| h - 100).collect();
    assert_eq!(niveaux(&Face::depuis_hauteurs(3, 16, 0, 1, &decalee)), attendu);
    assert_eq!(
        niveaux(&Face::depuis_hauteurs(3, 16, 0, 1, &[2, 5, 2, 5, 5, 2, 2, 2, 5])),
        [0, 255, 0, 255, 255, 0, 0, 0, 255]
    );
    assert_eq!(
        niveaux(&Face::depuis_hauteurs(3, 16, 0, 1, &[0, 1, 2, 2, 1, 0, 1, 2, 0])),
        [0, 127, 255, 255, 127, 0, 127, 255, 0]
    );
    assert_eq!(niveaux(&Face::new(3, 16, 0, 1)), [128; 9]);
    assert_eq!(niveaux(&Face::depuis_hauteurs(3, 16, 0, 1, &[-4; 9])), [128; 9]);
    let mut face = Face::new(5, 12, 4, 3);
    face.balayages(20);
    let attendu = [
        255_u8, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 0, 255, 0, 0, 0, 0, 255, 255, 255, 255, 0,
        255,
    ];
    assert_eq!(niveaux(&face), attendu);
    let mut face = Face::new(6, 0, 8, 2);
    face.balayages(25);
    let attendu = [
        133_u8, 121, 133, 194, 97, 255, 194, 133, 170, 182, 60, 230, 255, 97, 145, 194, 182, 97, 85, 170, 242, 72, 194,
        170, 170, 0, 133, 121, 145, 206, 182, 206, 170, 109, 170, 109,
    ];
    assert_eq!(niveaux(&face), attendu);
}

/// juste : les constantes de la doc : `TAILLE_MIN = 3`, `TAILLE_MAX = 4096`, `BALAYAGES_MAX = 1 000 000` (V-MES-09).
///
/// Ne couvre pas : leur usage (`bornes_et_paniques`, `etape_4_lancer`).
#[test]
#[cfg_attr(not(feature = "etape-4"), ignore)]
fn constantes_de_la_doc() {
    assert_eq!((TAILLE_MIN, TAILLE_MAX, BALAYAGES_MAX), (3, 4096, 1_000_000));
}

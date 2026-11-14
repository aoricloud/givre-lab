//! Étape P1 : l'écriture canonique et l'empreinte de `Front1d` et `Front1dFixe` (donc, par l'arithmétique employée, la même sous Linux et
//! sous Windows : des `f64` en `+ - * /` seulement, des entiers). Valeurs attendues : `references/etape-p1-valeurs.md`, identifiants
//! `V-P1-EMP-xx`.

mod etape_p1_commun;

use etape_p1_commun::*;
use givre_lab::empreinte::sha256_hex;
use givre_lab::phase::{Front1d, Front1dFixe, ModeU};

/// Le scénario des empreintes (`p1_ref.py`, `scenario()`) : `w = 1`, `tau = 1,6`, `lambda = 1,4`, `d = 0,9`, `dx = 0,5`, `dt = 0,04`, 24 cases,
/// `φ[i] = -clamp((i - 11,5) / 3, -1, 1)`, `u[i] = i · 0,0125 - 0,3` (des opérations de base seulement : les mêmes bits en Python et en Rust).
fn scenario() -> (givre_lab::phase::Parametres, Vec<f64>, Vec<f64>) {
    let p = param(1.0, 1.6, 1.4, 0.9, 0.5, 0.04);
    let phi = (0..24).map(|i| -((i as f64 - 11.5) / 3.0).clamp(-1.0, 1.0)).collect();
    let u = (0..24).map(|i| i as f64 * 0.0125 - 0.3).collect();
    (p, phi, u)
}

/// juste : (V-P1-EMP-01) l'écriture canonique, octet par octet, recalculée à la main dans le test : l'étiquette `P1F64\n` (6 octets), `n` en `u64`
/// petit-boutiste, le nombre de pas en `u64` petit-boutiste, l'octet du mode (0 gelé, 1 joué), les `n` valeurs de `φ` puis les `n` de `u`
/// (8 octets petit-boutistes : `to_bits` en `f64`, le champ brut `i64` en `Fixe`, étiquette `P1FIX\n`) ; l'empreinte est le SHA-256 en hexadécimal
/// de ces octets. Un état à `n = 3` après un pas (les valeurs à la main de `etape_p1_naif.rs`), en `f64` et en `Fixe`, modes joué et gelé.
///
/// Ne couvre pas : les grands états (voir `empreinte_du_scenario`).
#[test]
#[cfg_attr(not(feature = "etape-p1"), ignore)]
fn ecriture_canonique_a_la_main() {
    let p = param(1.0, 1.0, 2.0, 1.0, 1.0, 0.25);
    let (phi, u) = ([1.0, 0.5, -1.0], [0.0, -0.5, -1.0]);
    let (phi1, u1): ([f64; 3], [f64; 3]) =
        ([7.0 / 8.0, 31.0 / 64.0, -5.0 / 8.0], [-3.0 / 16.0, -65.0 / 128.0, -11.0 / 16.0]);
    let en_tete = |etiquette: &[u8], mode: u8| {
        let mut o = etiquette.to_vec();
        o.extend_from_slice(&3_u64.to_le_bytes());
        o.extend_from_slice(&1_u64.to_le_bytes());
        o.push(mode);
        o
    };
    // f64, joué
    let mut a = Front1d::new(p, &phi, &u, ModeU::Joue).unwrap();
    a.avancer(1).unwrap();
    let mut attendu = en_tete(b"P1F64\n", 1);
    for x in phi1.iter().chain(u1.iter()) {
        attendu.extend_from_slice(&x.to_bits().to_le_bytes());
    }
    assert_eq!(attendu.len(), 6 + 8 + 8 + 1 + 6 * 8);
    assert_eq!(a.ecriture_canonique(), attendu);
    assert_eq!(a.empreinte(), sha256_hex(&attendu));
    // Fixe, joué : les champs bruts (valeur * 2^32, exacts ici)
    let mut b = Front1dFixe::new(p, &phi, &u, ModeU::Joue).unwrap();
    b.avancer(1).unwrap();
    let mut attendu = en_tete(b"P1FIX\n", 1);
    for x in phi1.iter().chain(u1.iter()) {
        attendu.extend_from_slice(&((x * 4_294_967_296.0) as i64).to_le_bytes());
    }
    assert_eq!(b.ecriture_canonique(), attendu);
    assert_eq!(b.empreinte(), sha256_hex(&attendu));
    // gelé : l'octet de mode est 0 et u est celui du départ
    let mut c = Front1d::new(p, &phi, &u, ModeU::Gele).unwrap();
    c.avancer(1).unwrap();
    let ecr = c.ecriture_canonique();
    assert_eq!(ecr[22], 0);
    assert_eq!(&ecr[23 + 24..], &u.iter().flat_map(|x| x.to_bits().to_le_bytes()).collect::<Vec<u8>>()[..]);
    // départ (zéro pas) : le compteur est 0
    let d = Front1d::new(p, &phi, &u, ModeU::Joue).unwrap();
    assert_eq!(&d.ecriture_canonique()[14..22], &0_u64.to_le_bytes());
}

/// juste : (V-P1-EMP-02) les empreintes du scénario de `p1_ref.py` (le code Python indépendant, écrit d'après la doc seule, qui calcule le
/// schéma en flottants Python et en entiers et l'écriture canonique), pour 0 et 500 pas, `u` joué ou gelé, en `f64` ET en `Fixe` : les 8 empreintes
/// sont celles de Python (SHA-256 par `hashlib`). Comme le schéma n'emploie que `+ - * /` en `f64` et des entiers, ce sont les mêmes sous Linux et
/// sous Windows.
///
/// Ne couvre pas : un autre scénario ; une machine qui n'est pas IEEE 754 binaire64 (il n'y en a plus).
#[test]
#[cfg_attr(not(feature = "etape-p1"), ignore)]
fn empreinte_du_scenario() {
    let (p, phi, u) = scenario();
    let attendu: [(u64, ModeU, &str, &str); 4] = [
        (
            0,
            ModeU::Joue,
            "9851d9877f23f221558388abf29a4d54937b66248b435743a652842c550c2add",
            "5f4fb4734df58c95ccfab48fd603786f1bf87e7ba143eb8d47fe89a3b3342be7",
        ),
        (
            0,
            ModeU::Gele,
            "1fb37411683144727287af98c578d53654643059091283419d8bcd0c71c20029",
            "be083a6ce6fb86623ca8234168dbfb13263425fdf60dd12beefc1bc3e2fe913e",
        ),
        (
            500,
            ModeU::Joue,
            "1ced2e38273824f28b5f1ce4a411b030eb0bd53560455f9edb6246ede11578d2",
            "4450daa01e43e52cb2d01a1d707aee2ec010143143c4874fb6c2e341e00e9832",
        ),
        (
            500,
            ModeU::Gele,
            "5b7646d0f133de402bebb18e069a2162a7432ab28d5cbb56a390d607f684bf6d",
            "3ec9a7011fb43e3caa2b9d7add030188f25f037fb605492ad6faf51337e8d94c",
        ),
    ];
    for (pas, mode, h64, hfix) in attendu {
        let mut a = Front1d::new(p, &phi, &u, mode).unwrap();
        a.avancer(pas).unwrap();
        assert_eq!(a.empreinte(), h64, "f64 pas {pas} {mode:?}");
        let mut b = Front1dFixe::new(p, &phi, &u, mode).unwrap();
        b.avancer(pas).unwrap();
        assert_eq!(b.empreinte(), hfix, "Fixe pas {pas} {mode:?}");
    }
}

/// juste : (V-P1-EMP-03) l'empreinte du front poussé (cas 2, `dx = 0,5`, `lambda Delta = 0,1`, `lambda = 2`, `W = 1,25`, `tau = 2`, 20 de temps)
/// en `f64` et en `Fixe`, calculée par `p1_ref.py` (Python indépendant : 320 pas ; les mêmes que celles du prototype) (le profil de départ est linéaire par morceaux : aucune fonction du système, pour que l'empreinte soit la même partout). Elle prouve l'accord avec le calcul indépendant, pas la justesse physique (voir `etape_p1_exact.rs`).
///
/// Ne couvre pas : la justesse physique.
#[test]
#[cfg_attr(not(feature = "etape-p1"), ignore)]
fn empreinte_du_front_pousse() {
    let (w, tau, lambda, dx) = (1.25, 2.0, 2.0, 0.5);
    let mut p = param(w, tau, lambda, 1.0, dx, 0.0);
    p.dt = pas_de_temps(&p, 0.5);
    let n = 120;
    let phi: Vec<f64> = (0..n).map(|i| -(((i as f64 + 0.5) * dx - 10.0) / 3.0).clamp(-1.0, 1.0)).collect();
    let u = vec![-0.05; n];
    let pas = (20.0 / p.dt).round() as u64;
    let mut a = Front1d::new(p, &phi, &u, ModeU::Gele).unwrap();
    a.avancer(pas).unwrap();
    let mut b = Front1dFixe::new(p, &phi, &u, ModeU::Gele).unwrap();
    b.avancer(pas).unwrap();
    assert_eq!(a.empreinte(), REG_F64);
    assert_eq!(b.empreinte(), REG_FIXE);
}

const REG_F64: &str = "c4b62d0d1436d60b0e2f6860eac5df9ff4228249b4b24065043d9bc1c0deb1d8";
const REG_FIXE: &str = "3ffd7597d18e78bd48048282b75098d70a145a37ceeceb25cc46d33527ee2288";

/// juste : (V-P1-EMP-04) un second scénario, celui du juge, calculé par `juge_ref.py` : un outil où CHAQUE opération `f64` est
/// faite en rationnels exacts puis arrondie une seule fois au binaire64 le plus proche (la définition de l'IEEE 754 : il ne dépend
/// ni du processeur ni d'un compilateur), et la virgule fixe en entiers. 31 cases (un nombre impair), `w = 0,9`, `tau = 1,3`,
/// `lambda = 2,3`, `d = 1,7`, `dx = 0,4`, `dt = 0,7 · dt_max()` (= 0,032941176470588245), `φ[i] = ((7 i) mod 11) / 5 - 1`,
/// `u[i] = ((5 i) mod 13) / 26 - 0,5`, `u` joué, après 1 et 257 pas, en `f64` et en `Fixe`. (Le même outil redonne les dix
/// empreintes des deux tests précédents : elles ne sont pas des régressions déguisées.)
///
/// Ne couvre pas : la justesse physique ; `u` gelé (voir `empreinte_du_scenario`).
#[test]
#[cfg_attr(not(feature = "etape-p1"), ignore)]
fn empreinte_du_scenario_du_juge() {
    let mut p = param(0.9, 1.3, 2.3, 1.7, 0.4, 0.0);
    p.dt = 0.7 * p.dt_max();
    assert_eq!(p.dt, 0.032_941_176_470_588_245);
    let phi: Vec<f64> = (0..31).map(|i| ((i * 7) % 11) as f64 / 5.0 - 1.0).collect();
    let u: Vec<f64> = (0..31).map(|i| ((i * 5) % 13) as f64 / 26.0 - 0.5).collect();
    let attendu: [(u64, &str, &str); 2] = [
        (
            1,
            "53744af874406e0feb78859068babd8309d1369fa7b82e6df5b8b123f5bbf349",
            "ffcf5852645fb790542b7303e4357f15a51591bb920c6caf5f163209b419d144",
        ),
        (
            257,
            "74717c74e648a34de7198c94484a2e1157c733a9174f02d4654119675296aff4",
            "fcd091f250f3ed24ff41f0c1e462cb17bf57ee0da72009e798d512a28a614ad3",
        ),
    ];
    for (pas, h64, hfix) in attendu {
        let mut a = Front1d::new(p, &phi, &u, ModeU::Joue).unwrap();
        a.avancer(pas).unwrap();
        assert_eq!(a.empreinte(), h64, "f64 pas {pas}");
        let mut b = Front1dFixe::new(p, &phi, &u, ModeU::Joue).unwrap();
        b.avancer(pas).unwrap();
        assert_eq!(b.empreinte(), hfix, "Fixe pas {pas}");
    }
}

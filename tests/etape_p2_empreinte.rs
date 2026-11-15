//! Étape P2 : l'écriture canonique et l'empreinte de `Dendrite` (donc, par l'arithmétique employée, la même sous Linux et sous Windows :
//! des `f64` en `+ - * /` et `sqrt` (correctement arrondie), des entiers). Valeurs attendues : `references/etape-p2-valeurs.md`,
//! identifiants `V-P2-EMP-xx`.

mod etape_p2_commun;

use etape_p2_commun::Lcg;
use givre_lab::dendrite::{Dendrite, Domaine, ModeU, Reglages};
use givre_lab::empreinte::sha256_hex;

/// juste : (V-P2-EMP-01) l'écriture canonique, octet par octet, recalculée à la main dans le test : l'étiquette ASCII `P2DEN\n` (6 octets), `n`
/// en `u64` petit-boutiste, le nombre de pas faits en `u64` petit-boutiste, un octet de domaine (0 entier, 1 quart), un octet de mode (0 gelé,
/// 1 joué), les `n²` valeurs de `φ` puis les `n²` de `u` (chacune `to_bits()` en 8 octets petit-boutistes) ; l'empreinte est le SHA-256 en
/// hexadécimal de ces octets. L'état est celui de `un_pas_isotrope_a_la_main` après un pas (les valeurs rationnelles de `etape_p2_exact.rs`),
/// `u` joué, dans les deux domaines ; puis avec `u` gelé (l'octet de mode est 0 et `u` est celui du départ) ; et au départ (le compteur est 0).
/// Le signe d'un zéro est écrit tel quel : un état de départ qui porte `-0.0` (dans `φ` et dans `u`) garde ses bits (`0x8000…`, dernier octet
/// `0x80`), et son empreinte n'est pas celle du même état avec `+0.0`.
///
/// Ne couvre pas : les grands états (voir `empreinte_des_scenarios`).
#[test]
#[cfg_attr(not(feature = "etape-p2"), ignore)]
fn ecriture_canonique_a_la_main() {
    let r = Reglages { w0: 1.0, tau0: 1.0, lambda: 1.0, d: 1.0, epsilon: 0.0, dx: 1.0, dt: 1.0 / 16.0 };
    let phi = [0.5, 0.25, 0.0, 0.75, 0.5, -0.25, 0.0, -0.5, -1.0];
    let u = [-0.5, -0.25, 0.0, -0.25, -0.5, -0.5, 0.0, -0.25, -0.75];
    let phi1: [f64; 9] = [
        277.0 / 512.0,
        4817.0 / 16384.0,
        0.0,
        11393.0 / 16384.0,
        221.0 / 512.0,
        -1815.0 / 8192.0,
        1.0 / 64.0,
        -463.0 / 1024.0,
        -59.0 / 64.0,
    ];
    let u1: [f64; 9] = [
        -459.0 / 1024.0,
        -7983.0 / 32768.0,
        -3.0 / 64.0,
        -9599.0 / 32768.0,
        -499.0 / 1024.0,
        -7703.0 / 16384.0,
        -3.0 / 128.0,
        -527.0 / 2048.0,
        -85.0 / 128.0,
    ];
    let en_tete = |domaine: u8, mode: u8, pas: u64| {
        let mut o = b"P2DEN\n".to_vec();
        o.extend_from_slice(&3_u64.to_le_bytes());
        o.extend_from_slice(&pas.to_le_bytes());
        o.push(domaine);
        o.push(mode);
        o
    };
    for (domaine, octet) in [(Domaine::Entier, 0_u8), (Domaine::Quart, 1)] {
        let mut a = Dendrite::new(r, 3, domaine, &phi, &u, ModeU::Joue).unwrap();
        a.avancer(1).unwrap();
        let mut attendu = en_tete(octet, 1, 1);
        for x in phi1.iter().chain(u1.iter()) {
            attendu.extend_from_slice(&x.to_bits().to_le_bytes());
        }
        assert_eq!(attendu.len(), 6 + 8 + 8 + 1 + 1 + 18 * 8);
        assert_eq!(a.ecriture_canonique(), attendu, "{domaine:?}");
        assert_eq!(a.empreinte(), sha256_hex(&attendu));
    }
    // gelé : l'octet de mode est 0 et u est celui du départ
    let mut c = Dendrite::new(r, 3, Domaine::Quart, &phi, &u, ModeU::Gele).unwrap();
    c.avancer(1).unwrap();
    let ecr = c.ecriture_canonique();
    assert_eq!(ecr[..6], *b"P2DEN\n");
    assert_eq!(ecr[22..24], [1, 0]);
    assert_eq!(&ecr[24 + 72..], &u.iter().flat_map(|x| x.to_bits().to_le_bytes()).collect::<Vec<u8>>()[..]);
    // le départ : le compteur est 0 et les valeurs sont celles données (le `0.0` de `phi[2]` : `+0.0`)
    let d = Dendrite::new(r, 3, Domaine::Entier, &phi, &u, ModeU::Joue).unwrap();
    let mut attendu = en_tete(0, 1, 0);
    for x in phi.iter().chain(u.iter()) {
        attendu.extend_from_slice(&x.to_bits().to_le_bytes());
    }
    assert_eq!(d.ecriture_canonique(), attendu);
    // le signe d'un zéro est écrit tel quel
    let (mut phi_neg, mut u_neg) = (phi, u);
    phi_neg[2] = -0.0;
    u_neg[2] = -0.0;
    let e = Dendrite::new(r, 3, Domaine::Entier, &phi_neg, &u_neg, ModeU::Joue).unwrap();
    let ecr = e.ecriture_canonique();
    assert_eq!(ecr[24 + 16..24 + 24], [0, 0, 0, 0, 0, 0, 0, 0x80], "phi[2] = -0.0");
    assert_eq!(ecr[24 + 72 + 16..24 + 72 + 24], [0, 0, 0, 0, 0, 0, 0, 0x80], "u[2] = -0.0");
    assert_ne!(e.empreinte(), d.empreinte());
}

/// Le scénario S1 de `p2_cas_empreinte.py` : `n = 8`, `W0 = 1`, `tau0 = 1,6`, `lambda = 1,4`, `D = 0,9`, `epsilon = 0,05`, `dx = 0,5`, `dt = 0,02`,
/// `φ[i][j] = 0 - clamp((((i · 0,7) + (j · 1,1)) - 4,2) / 2,5, -1, 1)`, `u[i][j] = (((3 i + 5 j) mod 7) / 14) - 0,5` (des opérations de base
/// seulement : les mêmes bits en Python et en Rust).
fn scenario_s1() -> (Reglages, Vec<f64>, Vec<f64>) {
    let r = Reglages { w0: 1.0, tau0: 1.6, lambda: 1.4, d: 0.9, epsilon: 0.05, dx: 0.5, dt: 0.02 };
    let mut phi = Vec::new();
    let mut u = Vec::new();
    for i in 0..8_usize {
        for j in 0..8_usize {
            phi.push(0.0 - ((((i as f64 * 0.7) + (j as f64 * 1.1)) - 4.2) / 2.5).clamp(-1.0, 1.0));
            u.push((((i * 3 + j * 5) % 7) as f64 / 14.0) - 0.5);
        }
    }
    (r, phi, u)
}

/// juste : (V-P2-EMP-02) les empreintes des scénarios de `p2_cas_empreinte.py` (le code Python indépendant, écrit d'après la doc seule, qui
/// calcule le schéma en flottants Python (IEEE 754 binaire64, sans fonction du système sauf `math.sqrt` pour la graine, correctement arrondie
/// comme celle de Rust) et l'écriture canonique, SHA-256 par `hashlib`) : S1 (une formule à opérations de base : 0 pas `u` joué quart, 0 pas
/// `u` gelé entier, 400 pas `u` joué quart, 400 pas `u` gelé entier), S2 (la graine ronde `Quart`, `n = 14`, `dx = 0,8`, rayon 3, `Delta = 0,65`,
/// `depuis_capillarite(1 ; 0,5 ; 1 ; 0,05)`, `dt = 0,04`, `u` joué, 300 pas) et S3 (la graine ronde `Entier`, `n = 12`, 200 pas). Comme le schéma
/// n'emploie que `+ - * /` (et la racine de la graine), ce sont les mêmes sous Linux et sous Windows. Un changement de l'ordre d'une somme, d'un
/// signe, du zéro d'un état de départ (`-0.0` pour `+0.0`) ou de l'écriture change ces empreintes.
///
/// Ne couvre pas : un autre scénario ; une machine qui n'est pas IEEE 754 binaire64 (il n'y en a plus).
#[test]
#[cfg_attr(not(feature = "etape-p2"), ignore)]
fn empreinte_des_scenarios() {
    let (r, phi, u) = scenario_s1();
    let s1: [(u64, ModeU, Domaine, &str); 4] = [
        (0, ModeU::Joue, Domaine::Quart, "5ea5b71d79c322513197475af833631104e52472f367985b08a518a631e3272e"),
        (0, ModeU::Gele, Domaine::Entier, "4668975db2fd863b40a5039a78f8f536b79045893e3052974584b8e60ab71feb"),
        (400, ModeU::Joue, Domaine::Quart, "e7c80b69573579cadce3037bc95034236a807703304b5cc24e0598812493dd76"),
        (400, ModeU::Gele, Domaine::Entier, "80881128d000f4930da702a5788efb847d42c4a182d0dcef7adf71d40baf0538"),
    ];
    for (pas, mode, domaine, attendu) in s1 {
        let mut a = Dendrite::new(r, 8, domaine, &phi, &u, mode).unwrap();
        a.avancer(pas).unwrap();
        assert_eq!(a.empreinte(), attendu, "S1 : {pas} pas {mode:?} {domaine:?}");
    }
    let r = Reglages::depuis_capillarite(1.0, 0.5, 1.0, 0.05, 0.8, 0.04);
    let s23: [(usize, Domaine, u64, &str); 2] = [
        (14, Domaine::Quart, 300, "625599bbc1875f70e936477dc7204163b42f070df91e55d74b880b65e9a033e3"),
        (12, Domaine::Entier, 200, "38b958d6f93e790838df210d8de68a8693aa57fa2472b8d64cf568a879670889"),
    ];
    for (n, domaine, pas, attendu) in s23 {
        let mut a = Dendrite::graine_ronde(r, n, domaine, 0.65, 3.0, ModeU::Joue).unwrap();
        a.avancer(pas).unwrap();
        assert_eq!(a.empreinte(), attendu, "graine ronde n = {n} {domaine:?} {pas} pas");
    }
}

/// juste : (V-P2-EMP-03) les empreintes des scénarios du juge, calculées par `juge_ref.py` (un TROISIÈME code, en Python, écrit d'après la
/// seule doc, sans lire ni le Rust ni `p2_ref.py` : une case à la fois, indices ramenés dans la grille), avec `dt` = la borne de la doc
/// calculée en Python (les bits de `Reglages::dt_borne` sont donc recoupés hors de Rust : la valeur attendue est écrite ici en décimal) :
/// J1 (graine ronde `Entier`, `n = 11` impair, `depuis_capillarite(1,3 ; 0,7 ; 1,7 ; 0,0625)`, `dx = 0,6`, `Delta = 0,4`, rayon 2, `u` joué,
/// 250 pas), J2 (graine ronde `Quart`, `n = 9`, rayon 2,5, `u` GELÉ, 250 pas), J3 et J4 (un état tiré par le congruentiel des tests,
/// graine 20251115 : `φ` dans `[-1, 1[`, puis `u` dans `[-1,2 ; 0,3[`, `n = 7`, `W0 = 0,9`, `tau0 = 0,7`, `lambda = 2,5`, `D = 1,9`,
/// `epsilon = 0,066` (tout près de `1/15`), `dx = 0,45` : `Entier` joué et `Quart` gelé, 500 pas), J5 (`epsilon = 0` : la graine ronde
/// `Quart`, `n = 10`, rayon 4, 300 pas), J6 (`Quart`, `n = 12`, rayon 4, 200 pas : l'empreinte ET les bits de la position de la pointe,
/// `6,9599185337627`).
///
/// Ne couvre pas : la justesse physique (un second et un troisième code prouvent que la doc se lit d'une seule façon, pas qu'elle est juste).
#[test]
#[cfg_attr(not(feature = "etape-p2"), ignore)]
fn empreinte_des_scenarios_du_juge() {
    let avec_borne = |r0: Reglages, u_max: f64, mode: ModeU, attendu: f64| {
        let borne = r0.dt_borne(u_max, mode);
        assert_eq!(borne.to_bits(), attendu.to_bits(), "dt_borne = {borne:e}, calculé hors de Rust : {attendu:e}");
        Reglages { dt: attendu, ..r0 }
    };
    // J1 et J2
    let r0 = Reglages::depuis_capillarite(1.3, 0.7, 1.7, 0.0625, 0.6, 1.0);
    let r = avec_borne(r0, 0.4, ModeU::Joue, 0.012_618_586_934_709_504);
    let mut a = Dendrite::graine_ronde(r, 11, Domaine::Entier, 0.4, 2.0, ModeU::Joue).unwrap();
    a.avancer(250).unwrap();
    assert_eq!(a.empreinte(), "1318e687245a35c18d0f1327b151cf28752309545669adc9930880d2f48fb116", "J1");
    let r = avec_borne(r0, 0.4, ModeU::Gele, 0.012_618_586_934_709_504);
    let mut a = Dendrite::graine_ronde(r, 9, Domaine::Quart, 0.4, 2.5, ModeU::Gele).unwrap();
    a.avancer(250).unwrap();
    assert_eq!(a.empreinte(), "d6c17ad6107069db0127778113e1e558978ad38c5693fd43ac9c9697c702adf1", "J2");
    // J3 et J4
    let mut g = Lcg(20_251_115);
    let phi: Vec<f64> = (0..49).map(|_| g.entre(-1.0, 1.0)).collect();
    let u: Vec<f64> = (0..49).map(|_| g.entre(-1.2, 0.3)).collect();
    let u_max = u.iter().fold(0.0_f64, |m, v| m.max(v.abs()));
    let r0 = Reglages { w0: 0.9, tau0: 0.7, lambda: 2.5, d: 1.9, epsilon: 0.066, dx: 0.45, dt: 1.0 };
    for (mode, domaine, attendu) in [
        (ModeU::Joue, Domaine::Entier, "9f099cebc5b03d6368e4691ecbb8b656651acae1eefc21c79189fc47657ca42d"),
        (ModeU::Gele, Domaine::Quart, "fbe3bc0e3ef45f9aae35359bad433330918d6ff1bfba27eaa72e0a6de7e8263b"),
    ] {
        let r = avec_borne(r0, u_max, mode, 0.009_284_637_797_648_739);
        let mut a = Dendrite::new(r, 7, domaine, &phi, &u, mode).unwrap();
        a.avancer(500).unwrap();
        assert_eq!(a.empreinte(), attendu, "J3/J4 {mode:?} {domaine:?}");
    }
    // J5 : epsilon = 0
    let r0 = Reglages::depuis_capillarite(1.0, 0.5, 1.0, 0.0, 0.8, 1.0);
    let r = avec_borne(r0, 0.65, ModeU::Joue, 0.068_090_492_010_358_92);
    let mut a = Dendrite::graine_ronde(r, 10, Domaine::Quart, 0.65, 4.0, ModeU::Joue).unwrap();
    a.avancer(300).unwrap();
    assert_eq!(a.empreinte(), "b6ce1d96763aef66f20fe402ae902c0817e00569a386dda2214105fa01291b88", "J5");
    // J6 : l'empreinte et la pointe
    let r0 = Reglages::depuis_capillarite(1.0, 0.5, 1.0, 0.05, 0.8, 1.0);
    let r = Reglages { dt: r0.dt_borne(0.65, ModeU::Joue), ..r0 };
    let mut a = Dendrite::graine_ronde(r, 12, Domaine::Quart, 0.65, 4.0, ModeU::Joue).unwrap();
    a.avancer(200).unwrap();
    assert_eq!(a.empreinte(), "aafd9aa2b5c5e4766f10221b2fec761f42f459694e7bcb2a394bf698b9a1ee6e", "J6");
    assert_eq!(a.position_pointe().map(f64::to_bits), Some(0x401b_d6f4_e255_571c), "J6 : la pointe");
}

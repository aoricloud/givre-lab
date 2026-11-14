//! Étape P1 : la bibliothèque (`Front1d`, `Front1dFixe`) donne EXACTEMENT les bits de simulateurs naïfs écrits dans le
//! test d'après la doc de `src/phase.rs`. Valeurs attendues : `references/etape-p1-valeurs.md`, identifiants `V-P1-NAIF-xx`.

mod etape_p1_commun;

use etape_p1_commun::*;
use givre_lab::math::Fixe;
use givre_lab::phase::{Front1d, Front1dFixe, ModeU};

fn bits(v: &[f64]) -> Vec<u64> {
    v.iter().map(|x| x.to_bits()).collect()
}

/// Un état de départ tiré : `n` cases, `φ` dans `[-1,3 ; 1,3]`, `u` dans `[-1 ; 0,4]`.
fn depart(graine: u64, n: usize) -> (Vec<f64>, Vec<f64>) {
    let mut g = Lcg(graine);
    ((0..n).map(|_| g.entre(-1.3, 1.3)).collect(), (0..n).map(|_| g.entre(-1.0, 0.4)).collect())
}

/// juste : (V-P1-NAIF-01) pour 24 graines (tailles 3 à 40, réglages variés, `W` différent de 1, `tau` différent de 1, `dt`
/// de 0,3 à 0,8 fois `dt_max`, `lambda` de 0,2 à 1,5), les deux modes de `u`, 1 à 300 pas, l'état de `Front1d` est celui du naïf `f64` écrit dans
/// `etape_p1_commun` d'après la doc, **bit pour bit** (`φ` et `u`) ; et `pas_faits`, `temps` (`pas_faits as f64 * dt`) sont
/// ceux qu'on attend. Un signe, un facteur `1/2`, un carré, un ordre d'opérations, un bord, une mise à jour en place ou une
/// constante faussés (voir `etape_p1_fautes.rs`) cassent l'égalité.
///
/// Ne couvre pas : la justesse physique (voir `etape_p1_exact.rs`) ; `Fixe` (voir le test suivant).
#[test]
#[cfg_attr(not(feature = "etape-p1"), ignore)]
fn front_f64_egal_au_naif_bit_pour_bit() {
    let tailles = [3, 4, 5, 9, 17, 40];
    for graine in 1..=24_u64 {
        let n = tailles[(graine as usize) % tailles.len()];
        let mut g = Lcg(1000 + graine);
        let (w, tau, lambda, d, dx) =
            (g.entre(0.5, 2.0), g.entre(0.5, 3.0), g.entre(0.2, 1.5), g.entre(0.3, 2.0), g.entre(0.3, 1.2));
        let mut p = param(w, tau, lambda, d, dx, 0.0);
        p.dt = g.entre(0.3, 0.8) * p.dt_max();
        let (phi, u) = depart(graine, n);
        for mode in [ModeU::Gele, ModeU::Joue] {
            for pas in [1_u64, 2, 7, 60, 300] {
                let a = simuler(Moteur::Lib64, &p, &phi, &u, mode, pas);
                let b = simuler(Moteur::Naif64(None), &p, &phi, &u, mode, pas);
                assert_eq!(bits(&a.phi), bits(&b.phi), "graine {graine} n={n} {mode:?} pas {pas} : phi");
                assert_eq!(bits(&a.u), bits(&b.u), "graine {graine} n={n} {mode:?} pas {pas} : u");
            }
        }
        let mut f = Front1d::new(p, &phi, &u, ModeU::Joue).unwrap();
        assert_eq!((f.pas_faits(), f.temps()), (0, 0.0));
        f.avancer(37).unwrap();
        assert_eq!((f.pas_faits(), f.temps()), (37, 37.0 * p.dt));
    }
}

/// juste : (V-P1-NAIF-02) comme le test précédent pour `Front1dFixe` contre le naïf `Fixe` (mêmes constantes converties par
/// `Fixe::depuis_f64`, les opérateurs de `Fixe`, le même ordre) : les champs bruts `i64` sont égaux, case par case, pour
/// 24 graines, les deux modes, 1 à 300 pas. (Les valeurs de départ tirées sont dans `[-1,3 ; 1,3]` : aucun débordement.)
///
/// Ne couvre pas : l'accord entre `f64` et `Fixe` (voir `etape_p1_exact.rs`).
#[test]
#[cfg_attr(not(feature = "etape-p1"), ignore)]
fn front_fixe_egal_au_naif_champ_brut() {
    let tailles = [3, 4, 5, 9, 17, 40];
    for graine in 1..=24_u64 {
        let n = tailles[(graine as usize) % tailles.len()];
        let mut g = Lcg(2000 + graine);
        let (w, tau, lambda, d, dx) =
            (g.entre(0.5, 2.0), g.entre(0.5, 3.0), g.entre(0.2, 1.5), g.entre(0.3, 2.0), g.entre(0.3, 1.2));
        let mut p = param(w, tau, lambda, d, dx, 0.0);
        p.dt = g.entre(0.3, 0.8) * p.dt_max();
        let (phi, u) = depart(graine, n);
        for mode in [ModeU::Gele, ModeU::Joue] {
            for pas in [1_u64, 2, 7, 60, 300] {
                let a = simuler(Moteur::LibFixe, &p, &phi, &u, mode, pas);
                let b = simuler(Moteur::NaifFixe, &p, &phi, &u, mode, pas);
                assert_eq!(bits(&a.phi), bits(&b.phi), "graine {graine} n={n} {mode:?} pas {pas} : phi");
                assert_eq!(bits(&a.u), bits(&b.u), "graine {graine} n={n} {mode:?} pas {pas} : u");
            }
        }
        let mut f = Front1dFixe::new(p, &phi, &u, ModeU::Joue).unwrap();
        f.avancer(37).unwrap();
        assert_eq!((f.pas_faits(), f.temps()), (37, 37.0 * p.dt));
        assert_eq!(f.phi().len(), n);
        assert!(f.phi().iter().all(|x| x.0.abs() < 4 * (1_i64 << 32)));
    }
}

/// juste : (V-P1-NAIF-03) un pas calculé À LA MAIN, en nombres exacts en binaire (`W = dx = tau = D = 1`, `dt = 1/4`,
/// `lambda = 2`, donc `c_phi = 1`, `r = 1/4`, `c_u = 1/4`) : `φ = [1 ; 1/2 ; -1]`, `u = [0 ; -1/2 ; -1]`, `u` joué. Case 0 :
/// `lap = (1 + 1/2) - 2 = -1/2`, `f = 0`, `q = 0`, `vitesse = -1/2`, `dphi = -1/8`, `φ' = 7/8`. Case 1 : `lap = (1 - 1) - 1 = -1`,
/// `f = 3/8`, `q = 3/4`, `g = (2 · -1/2) · 9/16 = -9/16`, `vitesse = -1 + (3/8 + 9/16) = -1/16`, `dphi = -1/64`, `φ' = 31/64`.
/// Case 2 : `lap = (1/2 - 1) + 2 = 3/2`, `vitesse = 3/2`, `dphi = 3/8`, `φ' = -5/8`. Pour `u` : case 0 : `lap_u = (0 - 1/2) - 0`,
/// `u' = 0 + (1/4 · -1/2 + 1/2 · -1/8) = -3/16` ; case 1 : `lap_u = (0 - 1) + 1 = 0`, `u' = -1/2 + (0 + 1/2 · -1/64) = -65/128` ;
/// case 2 : `lap_u = (-1/2 - 1) + 2 = 1/2`, `u' = -1 + (1/8 + 1/2 · 3/8) = -11/16`. Le même résultat en `f64` et en `Fixe`.
///
/// Ne couvre pas : plusieurs pas (voir les deux tests précédents).
#[test]
#[cfg_attr(not(feature = "etape-p1"), ignore)]
fn un_pas_a_la_main() {
    let p = param(1.0, 1.0, 2.0, 1.0, 1.0, 0.25);
    let (phi, u) = ([1.0, 0.5, -1.0], [0.0, -0.5, -1.0]);
    let attendu_phi = [7.0 / 8.0, 31.0 / 64.0, -5.0 / 8.0];
    let attendu_u = [-3.0 / 16.0, -65.0 / 128.0, -11.0 / 16.0];
    let mut a = Front1d::new(p, &phi, &u, ModeU::Joue).unwrap();
    a.avancer(1).unwrap();
    assert_eq!(a.phi(), attendu_phi);
    assert_eq!(a.u(), attendu_u);
    let mut b = Front1dFixe::new(p, &phi, &u, ModeU::Joue).unwrap();
    b.avancer(1).unwrap();
    let vers: Vec<f64> = b.phi().iter().map(|x| x.vers_f64()).collect();
    assert_eq!(vers, attendu_phi);
    let vers: Vec<f64> = b.u().iter().map(|x| x.vers_f64()).collect();
    assert_eq!(vers, attendu_u);
    assert_eq!(b.phi()[0], Fixe(7 * (1_i64 << 29)));
    // gelé : u ne bouge pas, φ est le même
    let mut c = Front1d::new(p, &phi, &u, ModeU::Gele).unwrap();
    c.avancer(1).unwrap();
    assert_eq!(c.u(), u);
    assert_eq!(c.phi()[0], 7.0 / 8.0);
}

/// juste : (V-P1-NAIF-04) `avancer(a)` puis `avancer(b)` donne le même état (bits) et la même empreinte que `avancer(a + b)`,
/// `avancer(0)` ne change rien (état, empreinte, `pas_faits`), et un clone avance indépendamment de l'original.
///
/// Ne couvre pas : le contenu de l'empreinte (voir `etape_p1_empreinte.rs`).
#[test]
#[cfg_attr(not(feature = "etape-p1"), ignore)]
fn avancer_se_compose() {
    let mut g = Lcg(7);
    let n = 12;
    let mut p = param(1.1, 1.3, 1.7, 0.8, 0.6, 0.0);
    p.dt = 0.5 * p.dt_max();
    let phi: Vec<f64> = (0..n).map(|_| g.entre(-1.0, 1.0)).collect();
    let u: Vec<f64> = (0..n).map(|_| g.entre(-0.5, 0.0)).collect();
    let mut a = Front1d::new(p, &phi, &u, ModeU::Joue).unwrap();
    let mut b = Front1d::new(p, &phi, &u, ModeU::Joue).unwrap();
    let e0 = a.empreinte();
    a.avancer(0).unwrap();
    assert_eq!((a.empreinte(), a.pas_faits()), (e0.clone(), 0));
    a.avancer(30).unwrap();
    let copie = a.clone();
    a.avancer(45).unwrap();
    b.avancer(75).unwrap();
    assert_eq!(a.empreinte(), b.empreinte());
    assert_eq!(bits(a.phi()), bits(b.phi()));
    assert_eq!(copie.pas_faits(), 30);
    assert_ne!(copie.empreinte(), a.empreinte());
    let mut fa = Front1dFixe::new(p, &phi, &u, ModeU::Joue).unwrap();
    let mut fb = Front1dFixe::new(p, &phi, &u, ModeU::Joue).unwrap();
    fa.avancer(30).unwrap();
    fa.avancer(45).unwrap();
    fb.avancer(75).unwrap();
    assert_eq!(fa.empreinte(), fb.empreinte());
    assert_eq!(fa.phi(), fb.phi());
}

/// juste : (V-P1-NAIF-05) la position du front, calculée à la main d'après la doc : `φ = [1 ; 1/4 ; -3/4 ; -1]`, `dx = 1/2` :
/// `i = 1`, `x = 1,5 · 0,5 + 0,5 · (1/4 / (1/4 + 3/4)) = 0,875` ; deux traversées (`[1 ; -1 ; 1 ; -1]`) : la première
/// (`i = 0`, `x = 0,25 + 0,5 · 1/2 = 0,5`) ; un zéro exact sur la case suivante (`[1 ; 0 ; -1]` : `φ[i+1] <= 0` compte,
/// `x = 0,25 + 0,5 · 1 = 0,75`) ; deux zéros de suite (`[1 ; 0 ; 0 ; -1]` : `i = 0`, le zéro ne compte pas comme solide, `0,75`) ;
/// solide à droite (`[-1 ; 1 ; 1]`), tout liquide, tout solide, et un zéro suivi de liquide (`[0 ; -1 ; -1]` : il faut
/// `φ[i] > 0` strictement) : `None`. En `f64` et en `Fixe`.
///
/// Ne couvre pas : la position après des pas (voir `etape_p1_exact.rs`).
#[test]
#[cfg_attr(not(feature = "etape-p1"), ignore)]
fn position_du_front_a_la_main() {
    let p = param(1.0, 1.0, 1.0, 1.0, 0.5, 0.05);
    let cas: [(&[f64], Option<f64>); 9] = [
        (&[1.0, 0.25, -0.75, -1.0], Some(0.875)),
        (&[1.0, -1.0, 1.0, -1.0], Some(0.5)),
        (&[1.0, 0.0, -1.0], Some(0.75)),
        (&[-1.0, 1.0, 1.0], None),
        (&[-1.0, -1.0, -1.0], None),
        (&[1.0, 1.0, 1.0], None),
        (&[1.0, 0.0, 0.0, -1.0], Some(0.75)),
        (&[0.0, -1.0, -1.0], None),
        (&[0.0, 0.0, -1.0], None),
    ];
    for (phi, attendu) in cas {
        let u = vec![0.0; phi.len()];
        let a = Front1d::new(p, phi, &u, ModeU::Gele).unwrap();
        let b = Front1dFixe::new(p, phi, &u, ModeU::Gele).unwrap();
        assert_eq!(a.position_front(), attendu, "{phi:?}");
        assert_eq!(b.position_front(), attendu, "{phi:?}");
        assert_eq!(position(phi, 0.5), attendu, "{phi:?}");
    }
}

/// juste : (V-P1-NAIF-06) `position_front` donne EXACTEMENT (au bit près) la formule de la doc, recodée dans `etape_p1_commun`
/// (`((i + 0,5) · dx) + dx · (φ[i] / (φ[i] - φ[i+1]))`, dans cet ordre d'opérations), sur 300 états tirés (4 à 30 cases, `φ` dans
/// `[-1,2 ; 1,2]`, souvent plusieurs traversées, `dx` de 0,1 à 2), en `f64` et en `Fixe` (pour `Fixe`, la formule est appliquée aux
/// `φ` convertis par `vers_f64`) ; les deux issues (`Some` et `None`) se produisent.
///
/// Ne couvre pas : la position après des pas (voir `etape_p1_exact.rs`).
#[test]
#[cfg_attr(not(feature = "etape-p1"), ignore)]
fn position_du_front_bit_pour_bit() {
    let mut g = Lcg(4321);
    let (mut avec, mut sans) = (0, 0);
    for essai in 0..300 {
        let n = 4 + essai % 27;
        let dx = g.entre(0.1, 2.0);
        let mut p = param(1.0, 1.0, 1.0, 1.0, dx, 0.0);
        p.dt = 0.1 * p.dt_max();
        // un essai sur trois : tout liquide sauf quelques cases, pour que `None` arrive aussi
        let bas = if essai % 3 == 0 { -1.2 } else { -0.2 };
        let haut = if essai % 3 == 0 { -0.9 + 0.15 * (essai % 2) as f64 } else { 1.2 };
        let phi: Vec<f64> = (0..n).map(|_| g.entre(bas, haut)).collect();
        let u = vec![0.0; n];
        let a = Front1d::new(p, &phi, &u, ModeU::Gele).unwrap();
        let attendu = position(&phi, dx);
        assert_eq!(a.position_front().map(f64::to_bits), attendu.map(f64::to_bits), "f64, essai {essai} : {phi:?}");
        let b = Front1dFixe::new(p, &phi, &u, ModeU::Gele).unwrap();
        let phi_fixe: Vec<f64> = b.phi().iter().map(|x| x.vers_f64()).collect();
        let attendu_fixe = position(&phi_fixe, dx);
        assert_eq!(b.position_front().map(f64::to_bits), attendu_fixe.map(f64::to_bits), "Fixe, essai {essai}");
        if attendu.is_some() {
            avec += 1;
        } else {
            sans += 1;
        }
    }
    assert!(avec >= 50 && sans >= 20, "le tirage devait donner les deux issues : {avec} avec front, {sans} sans");
}

/// juste : (V-P1-NAIF-07) un pas À LA MAIN sur un demi-cas de la virgule fixe, qui fixe l'ordre `phi - (phi * phi) * phi` et l'arrondi
/// « demi-cas vers +∞ » : `w = 1/2`, `tau = dx = D = lambda = 1`, `dt = 1/2` (donc `r = 1/2`, `c_phi = 1/4`), `φ = 3/2048` partout
/// (champ brut `a = 3 · 2^21 = 6 291 456` ; le laplacien est nul), `u = 0`. `φ · φ` a pour champ brut 9216 (exact) ; `(φ · φ) · φ`
/// vaut `27 · 2^31 / 2^32 = 13,5`, arrondi à 14 ; `f = a - 14` ; `dphi = (a - 14) / 2 = 3 145 721` (exact) ; `φ' = 9 437 177`. (L'écriture
/// factorisée `φ · (1 - φ · φ)` donnerait `a - 13`, puis `9 437 178` : ce n'est pas celle de la doc.) Avec `u` joué :
/// `u' = 0,5 · dphi = 1 572 860,5`, arrondi à `1 572 861`. Depuis `φ = -3/2048` : `(φ · φ) · φ = -13,5`, arrondi à `-13` ; `f = -a + 13` ;
/// `dphi = (-a + 13) / 2 = -3 145 720,5`, arrondi à `-3 145 721` ; `φ' = -9 437 177` ; `u' = -1 572 860,5`, arrondi à `-1 572 860` : ce n'est
/// PAS l'opposé de `1 572 861` (le miroir impair de `Fixe` est perdu dès qu'un demi-cas se présente, voir la doc de `Front1dFixe`).
///
/// Ne couvre pas : `f64` (aucun demi-cas de ce genre : l'arrondi au plus proche pair est symétrique).
#[test]
#[cfg_attr(not(feature = "etape-p1"), ignore)]
fn un_pas_a_la_main_demi_cas_de_la_virgule_fixe() {
    let p = param(0.5, 1.0, 1.0, 1.0, 1.0, 0.5);
    let x = 3.0 / 2048.0;
    let mut a = Front1dFixe::new(p, &[x; 3], &[0.0; 3], ModeU::Gele).unwrap();
    assert_eq!(a.phi(), [Fixe(6_291_456); 3]);
    a.avancer(1).unwrap();
    assert_eq!(a.phi(), [Fixe(9_437_177); 3]);
    assert_eq!(a.u(), [Fixe(0); 3]);
    let mut b = Front1dFixe::new(p, &[x; 3], &[0.0; 3], ModeU::Joue).unwrap();
    b.avancer(1).unwrap();
    assert_eq!((b.phi(), b.u()), (&[Fixe(9_437_177); 3][..], &[Fixe(1_572_861); 3][..]));
    let mut c = Front1dFixe::new(p, &[-x; 3], &[0.0; 3], ModeU::Joue).unwrap();
    c.avancer(1).unwrap();
    assert_eq!((c.phi(), c.u()), (&[Fixe(-9_437_177); 3][..], &[Fixe(-1_572_860); 3][..]));
}

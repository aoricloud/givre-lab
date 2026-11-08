//! Étape 3 : l'écriture canonique de l'état et son empreinte (SHA-256). L'écriture est vérifiée d'abord **à la main** (octet par
//! octet, d'après la doc de `flocon`) et par Python (`hashlib`, un outil indépendant) ; elle ne dépend d'aucune graine. Valeurs
//! attendues : `references/etape-3-valeurs.md`, identifiants `V-EMP-xx`.

mod etape_3_commun;

use etape_3_commun::valeurs::{
    ECRITURE_APRES_UN_PAS_HEX, ECRITURE_APRES_UN_PAS_SHA256, ECRITURE_INITIALE_HEX, ECRITURE_INITIALE_SHA256,
};
use etape_3_commun::{cases, parametres};
use givre_lab::empreinte::sha256_hex;
use givre_lab::flocon::{Flocon, Parametres};
use givre_lab::math::Fixe;

fn hex(octets: &[u8]) -> String {
    octets.iter().map(|o| format!("{o:02x}")).collect()
}

/// L'écriture canonique de la doc, écrite à la main : l'étiquette, l'octet de forme, le rayon, les trois `brut`, le temps, les masses.
fn ecriture_a_la_main(rayon: u32, p: Parametres, temps: u64, masses: &[i64]) -> Vec<u8> {
    let mut octets = b"GIVRE003".to_vec();
    octets.push(1);
    octets.extend_from_slice(&rayon.to_le_bytes());
    octets.extend_from_slice(&p.alpha.0.to_le_bytes());
    octets.extend_from_slice(&p.beta.0.to_le_bytes());
    octets.extend_from_slice(&p.gamma.0.to_le_bytes());
    octets.extend_from_slice(&temps.to_le_bytes());
    for m in masses {
        octets.extend_from_slice(&m.to_le_bytes());
    }
    octets
}

/// juste : l'écriture canonique de l'état initial de `R = 1`, `α = 0.7`, `β = 0.3`, `γ = 0.05` (V-EMP-01) : 101 octets, `"GIVRE003"` (`47 49 56
/// 52 45 30 30 33`), `01`, `01 00 00 00`, `α`, `β` et `γ` en `i64` petit-boutiste, le temps `0` sur 8 octets, puis les 7 masses dans l'ordre
/// canonique `(0,-1) (1,-1) (-1,0) (0,0) (1,0) (-1,1) (0,1)` (`β` partout sauf `(0,0)` qui vaut `2^32`). Écrite à la main dans le test, égale à
/// celle de Python (`ECRITURE_INITIALE_HEX`) et à `ecriture_canonique()` ; son SHA-256 (Python `hashlib`) est `empreinte()`. Un ordre des champs
/// différent, un grand-boutiste, un temps oublié, un octet de forme faux échouent ici.
///
/// Ne couvre pas : les états après des pas (voir `ecriture_apres_un_pas`).
#[test]
#[cfg_attr(not(feature = "etape-3"), ignore)]
fn ecriture_canonique_initiale() {
    let p = parametres("0.7", "0.3", "0.05");
    let f = Flocon::new(1, p);
    let beta = p.beta.0;
    let masses = [beta, beta, beta, Fixe::UN.0, beta, beta, beta];
    let a_la_main = ecriture_a_la_main(1, p, 0, &masses);
    assert_eq!(a_la_main.len(), 45 + 8 * 7);
    assert_eq!(hex(&a_la_main), ECRITURE_INITIALE_HEX);
    assert_eq!(f.ecriture_canonique(), a_la_main);
    assert_eq!(
        &f.ecriture_canonique()[..13],
        &[0x47, 0x49, 0x56, 0x52, 0x45, 0x30, 0x30, 0x33, 0x01, 0x01, 0x00, 0x00, 0x00]
    );
    assert_eq!(f.empreinte(), ECRITURE_INITIALE_SHA256);
    assert_eq!(f.empreinte(), sha256_hex(&a_la_main));
}

/// juste : la même après un pas (V-EMP-02) : le temps vaut 1 dans les octets 37 à 44, et les masses sont celles du pas (calculées par Python :
/// `ECRITURE_APRES_UN_PAS_HEX`, SHA-256 `ECRITURE_APRES_UN_PAS_SHA256`). Les six voisines du centre `R = 1` sont la couronne : elles sont remises
/// à `β` ; le centre vaut `1 + γ`.
///
/// Ne couvre pas : les grands états (voir `etape_3_naif`).
#[test]
#[cfg_attr(not(feature = "etape-3"), ignore)]
fn ecriture_apres_un_pas() {
    let p = parametres("0.7", "0.3", "0.05");
    let mut f = Flocon::new(1, p);
    f.pas();
    let ecriture = f.ecriture_canonique();
    assert_eq!(hex(&ecriture), ECRITURE_APRES_UN_PAS_HEX);
    assert_eq!(&ecriture[37..45], &1_u64.to_le_bytes());
    assert_eq!(f.empreinte(), ECRITURE_APRES_UN_PAS_SHA256);
}

/// juste : l'écriture a `45 + 8 * cases` octets (V-EMP-03) pour `R` de 1 à 8 (`cases = 3 R (R + 1) + 1`), et chaque champ est à sa place : l'octet 8 est
/// 1, les octets 9 à 12 sont `R` en petit-boutiste, les octets 13 à 36 sont les trois paramètres, 37 à 44 le temps, puis les masses.
///
/// Ne couvre pas : la valeur des masses.
#[test]
#[cfg_attr(not(feature = "etape-3"), ignore)]
fn longueur_et_champs() {
    let p = parametres("1.5", "0.55", "0.0001");
    for rayon in 1_u32..=8 {
        let mut f = Flocon::new(rayon, p);
        f.avancer(3);
        let e = f.ecriture_canonique();
        let r = rayon as usize;
        assert_eq!(e.len(), 45 + 8 * (3 * r * (r + 1) + 1), "R={rayon}");
        assert_eq!(e[8], 1);
        assert_eq!(&e[9..13], &rayon.to_le_bytes());
        assert_eq!(&e[13..21], &p.alpha.0.to_le_bytes());
        assert_eq!(&e[21..29], &p.beta.0.to_le_bytes());
        assert_eq!(&e[29..37], &p.gamma.0.to_le_bytes());
        assert_eq!(&e[37..45], &3_u64.to_le_bytes());
        for (i, m) in f.masses().iter().enumerate() {
            assert_eq!(&e[45 + 8 * i..53 + 8 * i], &m.0.to_le_bytes(), "R={rayon} masse {i}");
        }
        assert_eq!(f.empreinte().len(), 64);
        assert!(f.empreinte().bytes().all(|c| c.is_ascii_digit() || (b'a'..=b'f').contains(&c)));
    }
}

/// juste : la sensibilité de l'empreinte (V-EMP-04). Même rayon, mêmes paramètres, même état : même empreinte (deux constructions, un clone,
/// `avancer(5)` contre cinq `pas`). Différent : `α`, `β` ou `γ` changé d'**un seul** `ulp` ; un rayon de plus ; le **temps** seul (un fond uniforme
/// sans glace ne change pas en un pas : seul le temps passe de 0 à 1) ; une masse changée d'un `ulp` ; deux masses échangées (l'ordre compte).
///
/// Ne couvre pas : l'absence de collision (le SHA-256 est testé à l'étape 0).
#[test]
#[cfg_attr(not(feature = "etape-3"), ignore)]
fn sensibilite_de_l_empreinte() {
    let p = parametres("1", "0.4", "0.01");
    let mut a = Flocon::new(6, p);
    let mut b = Flocon::new(6, p);
    assert_eq!(a.empreinte(), b.empreinte());
    assert_eq!(a.empreinte(), a.clone().empreinte());
    a.avancer(5);
    for _ in 0..5 {
        b.pas();
    }
    assert_eq!(a.empreinte(), b.empreinte());

    let base = Flocon::new(6, p).empreinte();
    let plus_un_ulp = |alpha: i64, beta: i64, gamma: i64| {
        let q =
            Parametres { alpha: Fixe(p.alpha.0 + alpha), beta: Fixe(p.beta.0 + beta), gamma: Fixe(p.gamma.0 + gamma) };
        Flocon::new(6, q).empreinte()
    };
    assert_ne!(plus_un_ulp(1, 0, 0), base);
    assert_ne!(plus_un_ulp(0, 1, 0), base);
    assert_ne!(plus_un_ulp(0, 0, 1), base);
    assert_eq!(plus_un_ulp(0, 0, 0), base);
    assert_ne!(Flocon::new(7, p).empreinte(), base);

    // le temps seul : un fond uniforme sans glace est invariant (voir `fond_uniforme_invariant`)
    let uniforme = vec![p.beta; cases(4).len()];
    let mut f = Flocon::depuis_masses(4, p, &uniforme);
    let avant = f.empreinte();
    f.pas();
    assert_eq!(f.masses(), uniforme);
    assert_ne!(f.empreinte(), avant, "seul le temps a changé");

    let mut masses = Flocon::new(4, p).masses();
    let reference = Flocon::depuis_masses(4, p, &masses).empreinte();
    masses[5].0 += 1;
    assert_ne!(Flocon::depuis_masses(4, p, &masses).empreinte(), reference);
    masses[5].0 -= 1;
    let mut avec_germe_deplace = Flocon::new(4, p).masses();
    avec_germe_deplace.swap(cases(4).iter().position(|&c| c == (0, 0)).unwrap(), 9);
    assert_ne!(Flocon::depuis_masses(4, p, &avec_germe_deplace).empreinte(), reference);
}

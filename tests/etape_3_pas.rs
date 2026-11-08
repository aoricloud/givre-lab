//! Étape 3 : UN pas du modèle sur de petits hexagones, avec des valeurs calculées à la main ou par un outil indépendant
//! (Python, `modele_independant.py`) : le noyau de diffusion, la règle de réceptivité de γ, la condition au bord, le fond
//! uniforme, la conservation de la masse, la positivité. Valeurs attendues : `references/etape-3-valeurs.md`, identifiants
//! `V-PAS-xx`.

mod etape_3_commun;

use etape_3_commun::valeurs::{P1_APRES_UN_PAS, P1_AVANT, P2_APRES_UN_PAS, P2_AVANT, P3_APRES_1, P3_APRES_2};
use etape_3_commun::{brut_de, cases, distance, parametres};
use givre_lab::flocon::{Etat, Flocon, Parametres};
use givre_lab::math::Fixe;

fn depuis_brut(rayon: u32, p: Parametres, brut: &[i64]) -> Flocon {
    let masses: Vec<Fixe> = brut.iter().map(|&b| Fixe(b)).collect();
    Flocon::depuis_masses(rayon, p, &masses)
}

/// Une bosse de masse `valeur` à la case `bosse`, 0 partout ailleurs (aucune cellule gelée : toutes sont non réceptives).
fn avec_bosse(rayon: u32, p: Parametres, bosse: (i32, i32), valeur: Fixe) -> Flocon {
    let toutes = cases(rayon as i32);
    let masses: Vec<Fixe> = toutes.iter().map(|&c| if c == bosse { valeur } else { Fixe::ZERO }).collect();
    Flocon::depuis_masses(rayon, p, &masses)
}

/// juste : le noyau de diffusion, calculé à la main (V-PAS-01). Une bosse de `3/4` (`brut = 3 * 2^30`) seule sur un fond de 0,
/// aucune cellule gelée, `R = 3` : la moyenne de la bosse est 0 (ses voisines valent 0) et `ū` de chacune de ses six voisines est
/// `(3/4) / 6 = 1/8` exactement (`2^29`). Pour `α = 1` : la bosse garde `3/8` et chaque voisine reçoit `1/16` ; pour `α = 2` : la
/// bosse tombe à 0 et chaque voisine reçoit `1/8` ; pour `α = 1/2` : la bosse garde `9/16` et chaque voisine reçoit `1/32` (tout
/// est dyadique : aucun arrondi). Les six voisines sont exactement `bosse + DIRECTIONS_HEX[k]` (pour la bosse en `(1, -1)` :
/// `(2,-1) (1,0) (0,0) (0,-1) (1,-2) (2,-2)`), toutes les autres cases restent à 0, et la masse totale est conservée (`3/4`). Un
/// voisinage faussé (un voisin remplacé, un cinquième ou septième voisin), un `α` sans son `1/2`, une moyenne sur 5 voisins
/// échouent ici.
///
/// Ne couvre pas : les cellules réceptives (voir `un_pas_depuis_le_germe_a_la_main`).
#[test]
#[cfg_attr(not(feature = "etape-3"), ignore)]
fn noyau_de_diffusion() {
    let trois_quarts = Fixe(3 << 30);
    for (alpha, centre, voisine) in
        [("1", Fixe(3 << 29), Fixe(1 << 28)), ("2", Fixe::ZERO, Fixe(1 << 29)), ("0.5", Fixe(9 << 28), Fixe(1 << 27))]
    {
        let p = parametres(alpha, "0", "0.25");
        for bosse in [(0, 0), (1, -1)] {
            let mut f = avec_bosse(3, p, bosse, trois_quarts);
            f.pas();
            let voisines: Vec<(i32, i32)> = [(1, 0), (0, 1), (-1, 1), (-1, 0), (0, -1), (1, -1)]
                .iter()
                .map(|&(dq, dr)| (bosse.0 + dq, bosse.1 + dr))
                .collect();
            let mut total: i64 = 0;
            for &(q, r) in &cases(3) {
                let attendu = if (q, r) == bosse {
                    centre
                } else if voisines.contains(&(q, r)) {
                    voisine
                } else {
                    Fixe::ZERO
                };
                assert_eq!(f.masse(q, r), attendu, "alpha {alpha} bosse {bosse:?} case ({q}, {r})");
                total += f.masse(q, r).0;
            }
            assert_eq!(total, trois_quarts.0, "masse conservée");
            assert_eq!(f.temps(), 1);
        }
    }
}

/// juste : un pas depuis l'état initial de `R = 3`, `α = 1`, `β = 1/2`, calculé à la main avec les arrondis de la doc (V-PAS-02).
/// Classes initiales : le centre est gelé, ses six voisins de bord, tout le reste non réceptif. Avec `γ = 0` : centre `1`
/// (`2^32`) ; chaque case de l'anneau 1 garde `1/2` et reçoit `(α/2) ū` avec `ū = (3 * 1/2) / 6 = 1/4` : `5/8`
/// (`2684354560`) ; les 6 coins de l'anneau 2 (sur les axes : `(2,0) (0,2) (-2,2) (-2,0) (0,-2) (2,-2)`) ont 5 voisines non
/// réceptives : `somme = 5 * 2^31`, `moyenne = 1789569707` (arrondi de `1789569706,67`), `ecart = -357913941`, `diffusion =
/// -178956970`, soit `1968526678` ; les 6 autres cases de l'anneau 2 ont 4 voisines non réceptives : `moyenne = 1431655765`,
/// `ecart = -715827883`, `diffusion = -357913941`, soit `1789569707` ; l'anneau 3 (la couronne) vaut `β`. Avec `γ = 1/4` : le
/// centre et l'anneau 1 reçoivent `2^30` de plus (`1.25` et `7/8`) et **rien d'autre ne change** (γ ne va qu'aux réceptives).
/// Un `γ` ajouté aux cellules non réceptives, un arrondi par troncature, un `α` sans `1/2` échouent ici.
///
/// Ne couvre pas : les états sans symétrie (voir `un_pas_python_p1` et `un_pas_python_p2`).
#[test]
#[cfg_attr(not(feature = "etape-3"), ignore)]
fn un_pas_depuis_le_germe_a_la_main() {
    for (gamma, centre, anneau1) in
        [("0", Fixe::UN.0, 2_684_354_560_i64), ("0.25", Fixe::UN.0 + (1 << 30), 2_684_354_560 + (1 << 30))]
    {
        let mut f = Flocon::new(3, parametres("1", "0.5", gamma));
        f.pas();
        let coins = [(2, 0), (0, 2), (-2, 2), (-2, 0), (0, -2), (2, -2)];
        for &(q, r) in &cases(3) {
            let attendu = match distance(q, r) {
                0 => centre,
                1 => anneau1,
                2 if coins.contains(&(q, r)) => 1_968_526_678,
                2 => 1_789_569_707,
                _ => 1 << 31,
            };
            assert_eq!(f.masse(q, r).0, attendu, "gamma {gamma} case ({q}, {r})");
        }
    }
}

/// juste : un fond uniforme sans glace est invariant (V-PAS-03) : toutes les cases à `β` (aucune gelée, `β < 1`), pour six `(α, β)`
/// dont `α = 2` : chaque case garde exactement sa masse à chaque pas, 8 pas, tous les `brut` égaux (la somme de six `brut` égaux
/// divisée par 6 est exacte). Avec une bosse seule (voir `noyau_de_diffusion`) la masse bouge : ce test ne passe pas pour un
/// code nul qui ne change rien si une bosse est ajoutée.
///
/// Ne couvre pas : un fond avec de la glace.
#[test]
#[cfg_attr(not(feature = "etape-3"), ignore)]
fn fond_uniforme_invariant() {
    for (alpha, beta) in [("1", "0.3"), ("2", "0.9"), ("0.5", "0.123456"), ("1.5", "0.999"), ("0", "0.5"), ("0.7", "0")]
    {
        let p = parametres(alpha, beta, "0.01");
        let masses = vec![p.beta; cases(5).len()];
        let mut f = Flocon::depuis_masses(5, p, &masses);
        for t in 1..=8 {
            f.pas();
            assert_eq!(f.masses(), masses, "alpha {alpha} beta {beta} au pas {t}");
        }
    }
}

/// juste : un pas depuis un état sans symétrie sur `R = 2`, `α = 0.7`, `β = 0.3`, `γ = 0.05` (V-PAS-04) : les masses de départ sont
/// `rapport((37 i + 11) % 64, 64)` pour la i-ème case (ordre canonique), sauf `(1,0) = 1`, `(-1,1) = 97/64`, `(0,-1) = 1 - 2^-32`, `(-2,2)
/// = 65/64` (une case gelée sur le coin de la couronne, une case à un `ulp` du seuil) ; les 19 masses attendues sont calculées par
/// Python (`modele_independant.py`, entiers exacts, la règle de la doc). Couvre la classe de chaque case, la couronne remise à `β`
/// (y compris la case gelée de la couronne), `γ` aux seules réceptives, la moyenne, le produit par `α` et les deux arrondis.
///
/// Ne couvre pas : plus d'un pas (voir `etape_3_naif`).
#[test]
#[cfg_attr(not(feature = "etape-3"), ignore)]
fn un_pas_python_p1() {
    let p = parametres("0.7", "0.3", "0.05");
    let mut f = depuis_brut(2, p, &P1_AVANT);
    f.pas();
    assert_eq!(brut_de(&f.masses()), P1_APRES_UN_PAS.to_vec());
    assert_eq!(f.temps(), 1);
}

/// juste : un pas depuis un état sans symétrie sur `R = 3`, `α = 2`, `β = 0.5`, `γ = 0.25` (V-PAS-05) : masses `rapport((53 i + 7) %
/// 97, 97)` sauf `(0,0) = 3/2`, `(2,-1) = 1`, `(-3,3) = 5/2` (coin de la couronne), `(1,1) = 0` ; les 37 masses attendues viennent de
/// Python. `α = 2` : le centre d'une cellule non réceptive passe à 0 de sa propre masse (poids `1 - α/2 = 0`).
///
/// Ne couvre pas : plus d'un pas (voir `etape_3_naif`).
#[test]
#[cfg_attr(not(feature = "etape-3"), ignore)]
fn un_pas_python_p2() {
    let p = parametres("2", "0.5", "0.25");
    let mut f = depuis_brut(3, p, &P2_AVANT);
    f.pas();
    assert_eq!(brut_de(&f.masses()), P2_APRES_UN_PAS.to_vec());
}

/// juste : un et deux pas depuis l'état initial de `R = 3`, `α = 1`, `β = 0.45`, `γ = 0.1` (V-PAS-06), les 37 masses de chacun
/// calculées par Python ; `temps` vaut 1 puis 2. Le second pas part d'un état où des cellules de l'anneau 1 ont changé de
/// masse : il vérifie que la mise à jour est synchrone (tout le pas lit l'état d'avant).
///
/// Ne couvre pas : les longues durées (voir `etape_3_naif`).
#[test]
#[cfg_attr(not(feature = "etape-3"), ignore)]
fn deux_pas_python_p3() {
    let mut f = Flocon::new(3, parametres("1", "0.45", "0.1"));
    f.pas();
    assert_eq!(brut_de(&f.masses()), P3_APRES_1.to_vec());
    assert_eq!(f.temps(), 1);
    f.pas();
    assert_eq!(brut_de(&f.masses()), P3_APRES_2.to_vec());
    assert_eq!(f.temps(), 2);
}

/// juste : la condition au bord (V-PAS-07) : à chaque pas, toutes les cases à distance exactement `R` valent `β`, même si elles
/// valaient autre chose, même gelées ; et ce sont les seules à valoir `β` par construction (les cases intérieures bougent). Depuis
/// l'état `P1_AVANT` (couronne de masses variées, dont une case gelée en `(-2,2)`) puis sur 5 pas : couronne égale à `β` après
/// chaque pas, 12 cases. Un bord non réinitialisé, ou remis à 0 ou à une autre valeur, échoue ici.
///
/// Ne couvre pas : l'effet de la couronne sur ses voisines (voir `un_pas_python_p1`).
#[test]
#[cfg_attr(not(feature = "etape-3"), ignore)]
fn couronne_remise_a_beta() {
    let p = parametres("0.7", "0.3", "0.05");
    let mut f = depuis_brut(2, p, &P1_AVANT);
    let couronne: Vec<(i32, i32)> = cases(2).into_iter().filter(|&(q, r)| distance(q, r) == 2).collect();
    assert_eq!(couronne.len(), 12);
    assert!(couronne.iter().any(|&(q, r)| f.masse(q, r) != p.beta), "le test part d'une couronne qui n'est pas à β");
    for t in 1..=5 {
        f.pas();
        for &(q, r) in &couronne {
            assert_eq!(f.masse(q, r), p.beta, "pas {t} case ({q}, {r})");
        }
    }
}

/// juste : la conservation de la masse (V-PAS-08). Sur `R = 40` depuis l'état initial, 30 pas (la perturbation n'atteint pas la
/// couronne : elle avance d'une case par pas), pour quatre `(α, β, γ)` : à chaque pas, `somme après - somme avant - γ * (nombre de
/// cellules réceptives avant)` est nul à l'arrondi près. La borne est dérivée de la doc : chaque cellule fait au plus 1,25 `ulp` d'erreur
/// (moyenne 0,5, produit par `α <= 2` : 1,5 puis la moitié 0,75, plus 0,5 du dernier arrondi), soit moins de `2 * 4921` unités pour
/// 4921 cases. Un `α` sans `1/2`, une moyenne sur 5 voisins, un voisinage faussé, une diffusion qui ne donne pas aux réceptives la
/// part des voisines non réceptives, un `γ` mal compté violent la conservation de plusieurs ordres de grandeur.
///
/// Ne couvre pas : la valeur des masses (voir les autres tests) ; la masse près du bord.
#[test]
#[cfg_attr(not(feature = "etape-3"), ignore)]
fn conservation_de_la_masse() {
    let somme = |f: &Flocon| -> i128 { f.masses().iter().map(|m| i128::from(m.0)).sum() };
    for (alpha, beta, gamma) in
        [("1", "0.3", "0.01"), ("2", "0.45", "0.1"), ("0.7", "0.6", "0.002"), ("0.5", "0.2", "0")]
    {
        let p = parametres(alpha, beta, gamma);
        let mut f = Flocon::new(40, p);
        let toutes = cases(40);
        let borne = 2 * toutes.len() as i128;
        for t in 1..=30 {
            let avant = somme(&f);
            let receptives = toutes.iter().filter(|&&(q, r)| f.etat(q, r) != Etat::NonReceptive).count() as i128;
            // l'état initial a 7 réceptives (le germe et ses voisines) ; leur nombre ne fait que croître
            assert!(receptives >= 7, "alpha {alpha} beta {beta} gamma {gamma} pas {t} : {receptives} réceptives");
            f.pas();
            assert_eq!(f.masses().len(), toutes.len());
            let ecart = somme(&f) - avant - i128::from(p.gamma.0) * receptives;
            assert!(ecart.abs() <= borne, "alpha {alpha} beta {beta} gamma {gamma} pas {t} : écart {ecart} ulp");
        }
        // la masse a bougé : un pas qui ne fait rien n'est pas conservatif « pour rien »
        assert_ne!(f.masses(), Flocon::new(40, p).masses(), "alpha {alpha} beta {beta} gamma {gamma} : rien n'a bougé");
    }
}

/// juste : les masses ne deviennent jamais négatives pour `0 <= α <= 2` (V-PAS-09), comme le dit la doc (le produit arrondi vaut
/// au moins `-2 u`). Deux séries de 100 pas : depuis `P1_AVANT` (état sans symétrie) avec `α = 2` ; depuis un état de masses de 1 à
/// 7 `ulp` (le pire cas des arrondis) avec `α = 2` puis `α` juste sous 2 (`1.9999999998`). Aucune masse négative, jamais.
///
/// Ne couvre pas : la valeur des masses.
#[test]
#[cfg_attr(not(feature = "etape-3"), ignore)]
fn masses_jamais_negatives() {
    let petites: Vec<i64> = (0..37).map(|i| 1 + (i * 5) % 7).collect(); // R = 3 : 37 cases
    for (alpha, rayon, brut) in
        [("2", 2_u32, P1_AVANT.to_vec()), ("2", 3, petites.clone()), ("1.9999999998", 3, petites)]
    {
        let p = parametres(alpha, "0.3", "0");
        let mut f = depuis_brut(rayon, p, &brut);
        let cases_attendues = brut.len();
        for t in 1..=100 {
            f.pas();
            assert_eq!(f.masses().len(), cases_attendues);
            assert!(f.masses().iter().all(|m| m.0 >= 0), "alpha {alpha} au pas {t}");
        }
        // la couronne est remise à β = 0.3 : l'état a bougé (un pas qui ne fait rien n'est pas une preuve de positivité)
        assert_ne!(brut_de(&f.masses()), brut, "alpha {alpha} : rien n'a bougé");
    }
}

/// juste : `temps` compte les pas (V-PAS-10) : 0 à la construction, `k` après `k` appels de `pas`, `n` de plus après `avancer(n)`
/// (`avancer(0)` ne fait rien) ; et `avancer(n)` donne exactement l'état de `n` appels de `pas`.
///
/// Ne couvre pas : la dynamique.
#[test]
#[cfg_attr(not(feature = "etape-3"), ignore)]
fn temps_et_avancer() {
    let p = parametres("1", "0.4", "0.01");
    let mut a = Flocon::new(8, p);
    let mut b = Flocon::new(8, p);
    assert_eq!(a.temps(), 0);
    for k in 1..=7 {
        a.pas();
        assert_eq!(a.temps(), k);
    }
    b.avancer(0);
    assert_eq!(b.temps(), 0);
    assert_eq!(b.masses(), Flocon::new(8, p).masses());
    b.avancer(7);
    assert_eq!(b.temps(), 7);
    assert_eq!(a.masses(), b.masses());
    a.avancer(5);
    for _ in 0..5 {
        b.pas();
    }
    assert_eq!(a.temps(), 12);
    assert_eq!(a.masses(), b.masses());
}

/// juste : la masse n'est pas modifiée par la lecture (V-PAS-11) : `masse`, `masses`, `etat`, `gelees`, `rayon_cristal`, `pointes`
/// et les défauts sont des lectures : les appeler entre deux pas ne change pas la suite. Deux flocons identiques, l'un lu à chaque
/// pas, l'autre jamais : mêmes masses à la fin.
///
/// Ne couvre pas : la valeur des mesures.
#[test]
#[cfg_attr(not(feature = "etape-3"), ignore)]
fn lire_ne_change_rien() {
    let p = parametres("1", "0.4", "0.01");
    let mut lu = Flocon::new(10, p);
    let mut intact = Flocon::new(10, p);
    for _ in 0..30 {
        let _ = (
            lu.masses(),
            lu.etat(1, 0),
            lu.gelees(),
            lu.rayon_cristal(),
            lu.pointes(),
            lu.compacite(),
            lu.defaut_rotation(),
        );
        let _ = (lu.defaut_reflexion(), lu.empreinte(), lu.apercu(), lu.ecriture_canonique());
        lu.pas();
        intact.pas();
    }
    assert_eq!(lu.masses(), intact.masses());
    // et la suite a bien avancé : 30 pas, des masses qui ne sont plus celles du départ
    assert_eq!(lu.temps(), 30);
    assert_ne!(lu.masses(), Flocon::new(10, p).masses());
}

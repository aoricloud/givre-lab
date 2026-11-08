//! Étape 1 : la DLA (`givre_lab::dla`), tests EXACTS. Valeurs attendues : `references/etape-1-valeurs.md`, identifiants
//! `V-DLA-xx`, `V-ANN-xx`, `V-CAP-xx`, `V-CAN-xx`, `V-DIR-xx`, `V-COL-xx`.
//!
//! Le test le plus fort est `amas_egal_au_simulateur_naif` : un simulateur NAÏF écrit ici (une vingtaine de lignes de boucles
//! simples, rien d'optimisé : un anneau énuméré à chaque lancement, un ensemble de sites) qui répète mot pour mot la section
//! « La dynamique, à la lettre » de la doc de `src/dla.rs`, y compris la consommation du hasard appel par appel ; la
//! bibliothèque doit donner EXACTEMENT ses sites et l'état de son générateur. Il voit un collage à huit voisins, un collage sur la
//! position précédente, un site décalé, un abandon mal placé, un anneau décalé d'un rayon, un tirage dans le mauvais ordre. Il a un
//! défaut qu'on connaît : il est écrit par la même main que la doc. C'est pourquoi `amas_egal_au_code_independant` compare aussi
//! à des valeurs calculées par un TROISIÈME code, en Python, écrit d'après la doc seule (`ref_dla.py`, modèle PCG32 de l'étape 0).

use givre_lab::alea::Alea;
use givre_lab::dla::{
    Amas, DEMI_COTE_MAX, FACTEUR_ABANDON, FOND, MARGE, PALETTE, PARTICULES_MAX, amas_colore, anneau, couleur_du_rang,
    demi_cote_de_la_boite, particules,
};
use givre_lab::empreinte::sha256_hex;
use givre_lab::scenario::{ErreurLecture, lire_scenario};
use std::collections::BTreeSet;

// ------------------------------------------------------------------------------------------------ le simulateur naïf

const DIRECTIONS: [(i32, i32); 4] = [(1, 0), (0, 1), (-1, 0), (0, -1)]; // droite, bas, gauche, haut

/// L'anneau, énuméré comme dit la doc de `dla::anneau` : lignes `y` de `-R` à `R`, dans chaque ligne `x` de `-R` à `R`, on garde
/// `R² <= x² + y² < (R+1)²`.
fn anneau_naif(r: i32) -> Vec<(i32, i32)> {
    let mut points = Vec::new();
    for y in -r..=r {
        for x in -r..=r {
            let d = x * x + y * y;
            if d >= r * r && d < (r + 1) * (r + 1) {
                points.push((x, y));
            }
        }
    }
    points
}

/// Le plus petit entier `r >= 0` tel que tous les sites vérifient `x² + y² <= r²`.
fn rayon_naif(sites: &[(i32, i32)]) -> i32 {
    let mut r = 0;
    while !sites.iter().all(|&(x, y)| x * x + y * y <= r * r) {
        r += 1;
    }
    r
}

/// Le résultat du simulateur naïf : les sites dans l'ordre d'arrivée, et l'état du générateur (`Alea::octets`) après chaque
/// particule (`etats[k]` : l'état quand l'amas a `k + 1` sites, `etats[0]` celui de l'amorçage).
struct Naif {
    sites: Vec<(i32, i32)>,
    etats: Vec<[u8; 16]>,
}

fn naif(graine: u64, n: usize) -> Naif {
    let mut alea = Alea::new(graine);
    let mut sites = vec![(0, 0)];
    let mut dedans: BTreeSet<(i32, i32)> = sites.iter().copied().collect();
    let mut etats = vec![alea.octets()];
    while sites.len() < n {
        let lancement = rayon_naif(&sites) + 5; // R = r + MARGE (la doc : MARGE vaut 5)
        let abandon = 2 * lancement; // A = FACTEUR_ABANDON * R (la doc : 2)
        'particule: loop {
            let anneau = anneau_naif(lancement);
            let k = alea.borne(anneau.len() as u32) as usize;
            let (mut x, mut y) = anneau[k];
            loop {
                if DIRECTIONS.iter().any(|&(dx, dy)| dedans.contains(&(x + dx, y + dy))) {
                    sites.push((x, y));
                    dedans.insert((x, y));
                    etats.push(alea.octets());
                    break 'particule;
                }
                if x * x + y * y > abandon * abandon {
                    break; // abandonné : on relance (nouveau tirage dans l'anneau)
                }
                let (dx, dy) = DIRECTIONS[alea.borne(4) as usize];
                x += dx;
                y += dy;
            }
        }
    }
    Naif { sites, etats }
}

/// Les 16 derniers octets de l'écriture canonique : l'état du générateur.
fn etat_du_generateur(amas: &Amas) -> [u8; 16] {
    let octets = amas.ecriture_canonique();
    octets[octets.len() - 16..].try_into().unwrap()
}

/// juste : le simulateur naïf écrit dans ce fichier (voir plus haut ; V-DLA-05), pour 12 graines (1 à 12) et des amas de 30, 100 puis 200
/// sites (le même `Amas`, grandi par `croitre_jusqu_a(30)`, `(100)`, `(200)`) : à chaque étape les `sites()` sont EXACTEMENT ceux du
/// simulateur naïf, et l'état du générateur (les 16 derniers octets de l'écriture canonique) aussi. Puis trois graines à 600 sites, avec
/// un autre `demi_cote` (la capacité ne change pas la dynamique tant qu'elle n'est pas dépassée).
///
/// Ne couvre pas : les très grands amas (le simulateur naïf y est lent), ni les tirages rejetés de `borne(len)` (voir
/// `amas_egal_au_code_independant`).
#[test]
#[cfg_attr(not(feature = "etape-1"), ignore)]
fn amas_egal_au_simulateur_naif() {
    for graine in 1..=12_u64 {
        let attendu = naif(graine, 200);
        let mut amas = Amas::new(graine, 100);
        for n in [30, 100, 200] {
            amas.croitre_jusqu_a(n);
            assert_eq!(amas.sites(), &attendu.sites[..n], "graine {graine}, {n} sites");
            assert_eq!(
                etat_du_generateur(&amas),
                attendu.etats[n - 1],
                "graine {graine}, {n} sites : état du générateur"
            );
        }
    }
    for graine in [101, 102, 103] {
        let attendu = naif(graine, 600);
        let mut amas = Amas::new(graine, 150);
        amas.croitre_jusqu_a(600);
        assert_eq!(amas.sites(), &attendu.sites[..], "graine {graine}, 600 sites");
        assert_eq!(etat_du_generateur(&amas), attendu.etats[599], "graine {graine} : état du générateur");
    }
}

// -------------------------------------------------------------------------------- le code indépendant (Python, hashlib)

/// juste : les premiers sites de la graine 1, 2, 3 et de deux graines FABRIQUÉES, calculés par le code Python indépendant (V-DLA-01).
/// Les deux graines fabriquées `6848047025811403090` et `10997253461852598610` sont telles que le PREMIER tirage de `Alea::new` vaut
/// 15 et 16 : le premier anneau (`R = 5`) a 40 points, le seuil de rejet de `borne(40)` est `2^32 mod 40 = 16`, donc le premier tirage
/// 15 est REJETÉ (le point de départ vient du tirage suivant) et le premier tirage 16 est ACCEPTÉ (indice 16) : ces deux graines voient
/// un `borne(len)` remplacé par `suivant() % len`, ou un rejet `r > seuil` au lieu de `r >= seuil` (construction : on inverse
/// l'amorçage, comme à l'étape 0, fiche `etape-0-valeurs.md`). Puis le SHA-256 de l'écriture canonique de 200 sites (`demi_cote` 300) pour six graines et
/// de 60 sites pour les deux graines fabriquées, calculé par Python `hashlib` sur les octets écrits par le code Python.
///
/// Ne couvre pas : la capacité (autres tests).
#[test]
#[cfg_attr(not(feature = "etape-1"), ignore)]
fn amas_egal_au_code_independant() {
    let premiers: [(u64, [(i32, i32); 12]); 5] = [
        (1, [(0, 0), (0, -1), (0, 1), (-1, 1), (-1, 2), (1, 1), (-1, -1), (2, 1), (-1, 3), (2, 0), (-2, 3), (0, -2)]),
        (
            2,
            [(0, 0), (1, 0), (0, -1), (0, 1), (-1, -1), (-2, -1), (1, 1), (-3, -1), (-3, -2), (1, 2), (0, -2), (-2, 0)],
        ),
        (3, [(0, 0), (-1, 0), (-2, 0), (-3, 0), (0, 1), (-3, 1), (1, 0), (-4, 1), (2, 0), (2, -1), (-3, -1), (-3, -2)]),
        (
            6_848_047_025_811_403_090,
            [(0, 0), (-1, 0), (0, 1), (0, -1), (1, 1), (1, 2), (0, 2), (-2, 0), (0, 3), (-3, 0), (1, 3), (1, 4)],
        ),
        (
            10_997_253_461_852_598_610,
            [(0, 0), (0, -1), (1, 0), (0, 1), (0, -2), (-1, -2), (-2, -2), (0, 2), (0, 3), (-3, -2), (1, -1), (0, 4)],
        ),
    ];
    for (graine, sites) in premiers {
        let mut amas = Amas::new(graine, 300);
        amas.croitre_jusqu_a(12);
        assert_eq!(amas.sites(), sites, "graine {graine}");
    }
    let sha_200 = [
        (1_u64, "97d7c8b5cb26ab4a6365bc18a7d91560b979ca4632a6fabf491cddca641ae896"),
        (2, "4ffb8e195c6c6bb39fc53f2dbe3892192750a5a7d814e7c6594e65b2fc220c08"),
        (3, "b77ab43e32656785ed6b80426fdbce9b5450bd54a478e51ea24b222c46a27c0d"),
        (7, "8c44410f04991032f20a1e9b4abebb46fc8e8bcc40897967b1dd36d7f0780dab"),
        (42, "477952490759e9d581bad9205caa06770ea4e2ba633a70a5bd1c38272b92de6d"),
        (1000, "d515721a9867aa6393b59b45027bcb5cd9b7338946e56de6d7f151649b059662"),
    ];
    for (graine, sha) in sha_200 {
        let mut amas = Amas::new(graine, 300);
        amas.croitre_jusqu_a(200);
        assert_eq!(sha256_hex(&amas.ecriture_canonique()), sha, "graine {graine}, 200 sites");
        assert_eq!(amas.empreinte(), sha, "graine {graine} : l'empreinte est le SHA-256 de l'écriture canonique");
    }
    for (graine, sha) in [
        (6_848_047_025_811_403_090_u64, "0c60f483f24d937e07de81b7c9fd92d7a2102698322b51face4164213ecde46b"),
        (10_997_253_461_852_598_610, "f401e167d0985d459fad32d26c072ccb3072217b4230307c34034b15e4043653"),
    ] {
        let mut amas = Amas::new(graine, 300);
        amas.croitre_jusqu_a(60);
        assert_eq!(amas.empreinte(), sha, "graine fabriquée {graine}, 60 sites");
    }
}

/// juste : les constantes que la doc de `dla` fixe : `MARGE` 5, `FACTEUR_ABANDON` 2 (le simulateur naïf et le code Python indépendant les emploient
/// en dur, jamais lues dans la bibliothèque : une main qui « calibre » l'une d'elles est vue ici et par les SHA-256 de Python) (V-DLA-04).
///
/// Ne couvre pas : `COTE_APERCU` ni `PARTICULES_MAX` (testés avec leur usage).
#[test]
#[cfg_attr(not(feature = "etape-1"), ignore)]
fn constantes_de_la_doc() {
    assert_eq!(MARGE, 5);
    assert_eq!(FACTEUR_ABANDON, 2);
}

// --------------------------------------------------------------------------------------------------------------- anneau

/// juste : l'anneau écrit à la main (V-ANN-01) : `anneau(0)` est `[(0, 0)]` ; `anneau(1)` est les 8 voisins par un côté ou un coin ;
/// `anneau(2)` a 16 points `(-2,-2) (-1,-2) (0,-2) (1,-2) (2,-2) (-2,-1) (2,-1) (-2,0) (2,0) (-2,1) (2,1) (-2,2) (-1,2) (0,2) (1,2) (2,2)`
/// (`4 <= d² < 9` : `d² = 4, 5, 8`), dans l'ordre ligne par ligne puis `x` croissant. Les longueurs de `anneau(R)` pour `R = 0..=15` sont
/// `1, 8, 16, 20, 24, 40, 36, 48, 56, 56, 68, 64, 80, 92, 88, 96` (comptage Python de `R² <= x² + y² < (R+1)²`), dont
/// `anneau(5)` : 40.
///
/// Ne couvre pas : les grands rayons (voir `anneau_egal_au_comptage_naif`).
#[test]
#[cfg_attr(not(feature = "etape-1"), ignore)]
fn anneau_a_la_main() {
    assert_eq!(anneau(0), [(0, 0)]);
    assert_eq!(anneau(1), [(-1, -1), (0, -1), (1, -1), (-1, 0), (1, 0), (-1, 1), (0, 1), (1, 1)]);
    assert_eq!(
        anneau(2),
        [
            (-2, -2),
            (-1, -2),
            (0, -2),
            (1, -2),
            (2, -2),
            (-2, -1),
            (2, -1),
            (-2, 0),
            (2, 0),
            (-2, 1),
            (2, 1),
            (-2, 2),
            (-1, 2),
            (0, 2),
            (1, 2),
            (2, 2)
        ]
    );
    let longueurs: Vec<usize> = (0..=15).map(|r| anneau(r).len()).collect();
    assert_eq!(longueurs, [1, 8, 16, 20, 24, 40, 36, 48, 56, 56, 68, 64, 80, 92, 88, 96]);
}

/// juste : `anneau(R)` égale l'énumération naïve (double boucle sur toute la boîte `[-R, R]²`, un test d'inégalité par point) pour
/// tous les rayons de 0 à 300, puis pour 777, 1000, 2047 et 2900 (le plus grand rayon de lancement possible : un site au coin d'un réseau de `demi_cote` 2047 a pour rayon `ceil(2047·√2) = 2895`, plus `MARGE`) : même
/// liste, même ordre (V-ANN-02).
///
/// Ne couvre pas : les rayons au-delà de 2900 (hors des bornes de l'étape).
#[test]
#[cfg_attr(not(feature = "etape-1"), ignore)]
fn anneau_egal_au_comptage_naif() {
    for r in (0..=300).chain([777, 1000, 2047, 2900]) {
        let obtenu = anneau(r as u32);
        let attendu = anneau_naif(r);
        assert_eq!(obtenu.len(), attendu.len(), "longueur de l'anneau {r}");
        assert!(obtenu == attendu, "anneau {r} : contenu ou ordre différent");
    }
}

// ------------------------------------------------------------------------------------------------- propriétés de l'amas

/// juste : pour dix graines et 150 sites (V-DLA-02) : le premier site est le germe `(0, 0)` ; il y a exactement 150 sites, tous distincts ;
/// chaque site (sauf le germe) a au moins un voisin PAR UN CÔTÉ (pas par un coin) parmi les sites arrivés avant lui (un collage par la
/// diagonale, ou sur la position précédente du marcheur, viole cette propriété) ; tous sont dans le réseau (`|x|, |y| <= demi_cote`) ;
/// `contient` est vrai exactement sur les sites (et faux, sans paniquer, très loin) ; `rayon()` est le plus petit entier `r` tel que
/// tous les sites vérifient `x² + y² <= r²` ; `ajouter()` rend le site qu'il ajoute, le dernier de `sites()`.
///
/// Ne couvre pas : que le collage se fasse au PREMIER contact (c'est `amas_egal_au_simulateur_naif`).
#[test]
#[cfg_attr(not(feature = "etape-1"), ignore)]
fn proprietes_de_l_amas() {
    for graine in 1..=10_u64 {
        let mut amas = Amas::new(graine, 60);
        assert_eq!(amas.sites(), [(0, 0)]);
        assert_eq!(amas.rayon(), 0, "le germe seul a pour rayon 0");
        assert_eq!(amas.demi_cote(), 60);
        let mut avant: BTreeSet<(i32, i32)> = BTreeSet::new();
        avant.insert((0, 0));
        for k in 1..150 {
            let site = amas.ajouter();
            assert_eq!(amas.sites().len(), k + 1);
            assert_eq!(amas.sites().last(), Some(&site), "ajouter() rend le dernier site");
            assert!(
                DIRECTIONS.iter().any(|&(dx, dy)| avant.contains(&(site.0 + dx, site.1 + dy))),
                "graine {graine}, site {k} = {site:?} : aucun voisin par un côté parmi les sites précédents"
            );
            assert!(avant.insert(site), "graine {graine} : le site {site:?} est déjà dans l'amas");
            assert!(site.0.abs() <= 60 && site.1.abs() <= 60, "dans le réseau");
        }
        let r = amas.rayon() as i32;
        let plus_grand = amas.sites().iter().map(|&(x, y)| x * x + y * y).max().unwrap();
        assert!(
            r * r >= plus_grand && (r == 0 || (r - 1) * (r - 1) < plus_grand),
            "rayon {r}, plus grand carré {plus_grand}"
        );
        assert_eq!(r, rayon_naif(amas.sites()));
        for x in -62..=62 {
            for y in -62..=62 {
                assert_eq!(amas.contient(x, y), avant.contains(&(x, y)), "contient({x}, {y})");
            }
        }
        assert!(!amas.contient(i32::MAX, i32::MIN) && !amas.contient(i32::MIN, 0) && !amas.contient(0, i32::MAX));
    }
}

/// juste : `croitre_jusqu_a(n)` ne fait rien quand l'amas a déjà `n` sites ou plus (les sites et l'état du générateur, donc l'écriture
/// canonique entière, ne changent pas) ; `croitre_jusqu_a(1)` et `(0)` ne font rien sur un germe seul ; deux amas de même graine ont
/// les mêmes sites, deux graines différentes en ont de différents, avec des empreintes différentes ; deux amas de `demi_cote` différents
/// (assez grands) ont les mêmes sites mais des écritures canoniques différentes (V-DLA-03).
///
/// Ne couvre pas : que deux graines donnent des amas « assez différents » (une propriété statistique).
#[test]
#[cfg_attr(not(feature = "etape-1"), ignore)]
fn croissance_graine_et_demi_cote() {
    let mut amas = Amas::new(9, 80);
    let germe = amas.ecriture_canonique();
    amas.croitre_jusqu_a(0);
    amas.croitre_jusqu_a(1);
    assert_eq!(amas.ecriture_canonique(), germe, "croitre_jusqu_a(0 ou 1) ne fait rien sur le germe");
    amas.croitre_jusqu_a(40);
    let a_40 = amas.ecriture_canonique();
    amas.croitre_jusqu_a(40);
    amas.croitre_jusqu_a(25);
    assert_eq!(amas.ecriture_canonique(), a_40, "croitre_jusqu_a(n) ne fait rien si l'amas a déjà n sites ou plus");
    assert_eq!(amas.sites().len(), 40);

    let (mut a, mut b) = (Amas::new(5, 80), Amas::new(5, 80));
    a.croitre_jusqu_a(120);
    b.croitre_jusqu_a(120);
    assert_eq!(a.sites(), b.sites());
    assert_eq!(a.empreinte(), b.empreinte());
    let mut vus = BTreeSet::new();
    for graine in 1..=8_u64 {
        let mut c = Amas::new(graine, 80);
        c.croitre_jusqu_a(120);
        assert!(vus.insert(c.empreinte()), "la graine {graine} donne une empreinte déjà vue");
        if graine != 5 {
            assert_ne!(c.sites(), a.sites(), "les graines {graine} et 5 donnent le même amas");
        }
    }
    let mut grand = Amas::new(5, 200);
    grand.croitre_jusqu_a(120);
    assert_eq!(grand.sites(), a.sites(), "le demi_cote ne change pas la dynamique");
    assert_ne!(grand.ecriture_canonique(), a.ecriture_canonique(), "mais il est dans l'écriture canonique");
}

// ---------------------------------------------------------------------------------------------------------------- capacité

/// juste : la capacité (V-CAP-01). Un réseau de `demi_cote = 3` (7 × 7 = 49 sites au plus) : `essayer_ajouter` finit par rendre `None`
/// (au plus 49 sites alors) et n'ajoute rien quand elle le fait ; tous les sites sont dans `|x|, |y| <= 3` ; sur vingt graines, CHACUN des
/// quatre bords du réseau (`x = 3`, `x = -3`, `y = 3`, `y = -3`) est atteint par au moins 3 amas (mesuré sur le prototype : 9, 12, 7 et 7 amas
/// sur 20) : le bord du réseau est dedans, pas dehors, et un `>=` pour un `>` sur un seul côté n'en atteindrait aucun de ce côté ; pour
/// `demi_cote = 0`, `essayer_ajouter` rend `None` tout de suite et l'amas reste le germe. `ajouter()` PANIQUE dans ce cas, avec un message
/// contenant « capacité » (voir les deux tests suivants).
///
/// Ne couvre pas : la valeur exacte du seuil où la particule déborde (elle dépend du hasard).
#[test]
#[cfg_attr(not(feature = "etape-1"), ignore)]
fn capacite_essayer_ajouter() {
    let mut bords = [0_usize; 4]; // x = 3, x = -3, y = 3, y = -3
    for graine in 1..=20_u64 {
        let mut amas = Amas::new(graine, 3);
        let mut essais = 0;
        while amas.essayer_ajouter().is_some() {
            essais += 1;
            assert!(essais <= 48, "plus de 49 sites dans un réseau de 49 cases");
        }
        assert!(amas.sites().iter().all(|&(x, y)| x.abs() <= 3 && y.abs() <= 3));
        assert_eq!(amas.sites().len(), essais + 1, "l'appel qui a rendu None n'a rien ajouté");
        for (i, atteint) in [
            amas.sites().iter().any(|&(x, _)| x == 3),
            amas.sites().iter().any(|&(x, _)| x == -3),
            amas.sites().iter().any(|&(_, y)| y == 3),
            amas.sites().iter().any(|&(_, y)| y == -3),
        ]
        .into_iter()
        .enumerate()
        {
            bords[i] += usize::from(atteint);
        }
    }
    assert!(bords.iter().all(|&n| n >= 3), "amas qui atteignent x=3, x=-3, y=3, y=-3 (sur 20) : {bords:?}");
    let mut nul = Amas::new(5, 0);
    assert_eq!(nul.essayer_ajouter(), None);
    assert_eq!(nul.sites(), [(0, 0)]);
}

/// juste : au-delà de la capacité, `ajouter` / `croitre_jusqu_a` PANIQUENT avec un message contenant « capacité » (doc de `Amas`) :
/// 60 sites ne tiennent pas dans un réseau de 49 cases (V-CAP-02).
///
/// Ne couvre pas : le message exact.
#[test]
#[cfg_attr(not(feature = "etape-1"), ignore)]
#[should_panic(expected = "capacité")]
fn capacite_croitre_panique() {
    Amas::new(1, 3).croitre_jusqu_a(60);
}

/// juste : `ajouter()` sur un réseau de `demi_cote = 0` panique (message contenant « capacité »), et `Amas::new` refuse un `demi_cote`
/// au-delà de `DEMI_COTE_MAX` = 2047 (V-CAP-03).
///
/// Ne couvre pas : le message de la seconde panique.
#[test]
#[cfg_attr(not(feature = "etape-1"), ignore)]
fn capacite_ajouter_et_demi_cote_max() {
    assert_eq!(DEMI_COTE_MAX, 2047);
    let panique = std::panic::catch_unwind(|| Amas::new(5, 0).ajouter()).expect_err("ajouter devait paniquer");
    let message =
        panique.downcast_ref::<String>().cloned().or_else(|| panique.downcast_ref::<&str>().map(|s| s.to_string()));
    assert!(message.is_some_and(|m| m.contains("capacité")), "le message doit contenir « capacité »");
    assert!(std::panic::catch_unwind(|| Amas::new(1, DEMI_COTE_MAX + 1)).is_err());
    let amas = Amas::new(1, DEMI_COTE_MAX);
    assert_eq!(amas.demi_cote(), 2047);
}

// ------------------------------------------------------------------------------------------- écriture canonique, empreinte

/// juste : l'écriture canonique, octet par octet (V-CAN-01). Le germe seul, `demi_cote = 5`, graine 7 : `GIVRE001` (`47 49 56 52 45 30 30 31`),
/// `05 00 00 00`, `01 00 00 00`, le germe `00 00 00 00 00 00 00 00`, puis l'état de `Alea::new(7)` (`d1a0bdb386a42205 6d00000000000000`,
/// celui de l'étape 0), soit 40 octets ; son SHA-256 `3ff6c507…5238` (Python `hashlib`). Puis les quatre premiers sites de la graine 1 pour
/// `demi_cote = 9` : `(0,0) (0,-1) (0,1) (-1,1)` (`x` puis `y`, chacun `i32` petit-boutiste, `-1` s'écrit `ff ff ff ff`), l'état
/// du générateur après la quatrième particule `91fe57e8fe428c35 6d00000000000000` (modèle PCG de l'étape 0), 64 octets ; SHA-256
/// `70b0b2a7…d7af`. Une écriture grand-boutiste, des `x` et `y` échangés, ou un état du générateur oublié échouent ici.
///
/// Ne couvre pas : les amas de plus de quatre sites (le test de SHA-256 de `amas_egal_au_code_independant`).
#[test]
#[cfg_attr(not(feature = "etape-1"), ignore)]
fn ecriture_canonique_a_la_main() {
    let germe = Amas::new(7, 5);
    let attendu_germe = "47495652453030310500000001000000\
                         0000000000000000\
                         d1a0bdb386a422056d00000000000000";
    assert_eq!(hex(&germe.ecriture_canonique()), attendu_germe);
    assert_eq!(germe.ecriture_canonique().len(), 40);
    assert_eq!(germe.empreinte(), "3ff6c507cfffbac3314c7c1e61c6585fdc3aee961d5ae35eb5e1d22669ae5238");

    let mut quatre = Amas::new(1, 9);
    quatre.croitre_jusqu_a(4);
    assert_eq!(quatre.sites(), [(0, 0), (0, -1), (0, 1), (-1, 1)]);
    let attendu = "47495652453030310900000004000000\
                   0000000000000000\
                   00000000ffffffff\
                   0000000001000000\
                   ffffffff01000000\
                   91fe57e8fe428c356d00000000000000";
    assert_eq!(hex(&quatre.ecriture_canonique()), attendu);
    assert_eq!(quatre.ecriture_canonique().len(), 32 + 8 * 4);
    assert_eq!(quatre.empreinte(), "70b0b2a7f291fab0ddf890adf84b5690e556c7d651505bedd1a567ac4797d7af");
}

fn hex(octets: &[u8]) -> String {
    octets.iter().map(|o| format!("{o:02x}")).collect()
}

// ---------------------------------------------------------------------------------------------------- la directive `dla`

/// juste : `particules` lit la directive `dla <n>` (V-DIR-01) : pas de directive `dla` donne `Ok(None)` ; `dla 3000` `Ok(Some(3000))`
/// (avant ou après la forme) ; `dla 1`, `dla 05` (zéros de tête permis) et `dla 1000000` sont acceptés ; les autres directives sont
/// ignorées. Refusés, avec le numéro de la ligne fautive (lignes ignorées et commentaires comptés) : `dla 0`, `dla 1000001`, `dla`,
/// `dla 1 2`, `dla +5`, `dla -5`, `dla 5.0`, `dla x`, `dla 99999999999999999999` ; une SECONDE directive `dla` (valide ou non) est la
/// fautive, et la première fautive l'emporte.
///
/// Ne couvre pas : l'interprétation par `lancer` (voir `etape_1_lancer`).
#[test]
#[cfg_attr(not(feature = "etape-1"), ignore)]
fn particules_lit_la_directive() {
    let lire = |texte: &str| particules(&lire_scenario(texte).expect("scénario valide"));
    assert_eq!(lire("boite 5 5\n"), Ok(None));
    assert_eq!(lire("boite 5 5\ninconnue 1\nautre\n"), Ok(None));
    assert_eq!(lire("boite 5 5\ndla 3000\n"), Ok(Some(3000)));
    assert_eq!(lire("dla 7\nboite 5 5\n"), Ok(Some(7)));
    assert_eq!(lire("boite 5 5\ndla 05\n"), Ok(Some(5)));
    assert_eq!(lire("boite 5 5\ndla 1\n"), Ok(Some(1)));
    assert_eq!(lire("boite 5 5\ndla 1000000\n"), Ok(Some(1_000_000)));
    assert_eq!(PARTICULES_MAX, 1_000_000);
    assert_eq!(lire("# un commentaire\nboite 5 5\n\ninconnue\ndla 12\n"), Ok(Some(12)));
    for mauvais in
        ["dla 0", "dla 1000001", "dla", "dla 1 2", "dla +5", "dla -5", "dla 5.0", "dla x", "dla 99999999999999999999"]
    {
        assert_eq!(lire(&format!("# c\nboite 5 5\n{mauvais}\n")), Err(ErreurLecture::LigneInvalide(3)), "{mauvais}");
    }
    assert_eq!(lire("boite 5 5\ndla 3\ndla 4\n"), Err(ErreurLecture::LigneInvalide(3)), "seconde directive valide");
    assert_eq!(lire("boite 5 5\ndla 3\n\ndla 0\n"), Err(ErreurLecture::LigneInvalide(4)), "seconde directive fautive");
    assert_eq!(
        lire("boite 5 5\ndla 0\ndla 3\n"),
        Err(ErreurLecture::LigneInvalide(2)),
        "la première fautive l'emporte"
    );
}

/// juste : `demi_cote_de_la_boite` vaut `(min(largeur, hauteur) - 1) / 2` : `401 401` donne 200, `401 300` 149, `300 401` 149, `400 400` 199,
/// `2 9` 0, `1 1` 0, `4096 4096` 2047 (= `DEMI_COTE_MAX`) ; une dimension nulle panique (V-DIR-02).
///
/// Ne couvre pas : l'usage par `lancer` (voir `etape_1_lancer`).
#[test]
#[cfg_attr(not(feature = "etape-1"), ignore)]
fn demi_cote_de_la_boite_a_la_main() {
    for (l, h, attendu) in
        [(401, 401, 200), (401, 300, 149), (300, 401, 149), (400, 400, 199), (2, 9, 0), (1, 1, 0), (4096, 4096, 2047)]
    {
        assert_eq!(demi_cote_de_la_boite(l, h), attendu, "boite {l} {h}");
    }
    assert!(std::panic::catch_unwind(|| demi_cote_de_la_boite(0, 5)).is_err());
    assert!(std::panic::catch_unwind(|| demi_cote_de_la_boite(5, 0)).is_err());
}

// ------------------------------------------------------------------------------------------------------------ les couleurs

/// juste : `couleur_du_rang` (V-COL-01), recalculée par Python d'après la formule de la doc : avec 5 sites, les cinq couleurs de la
/// palette ; avec 9 sites `255,255,255 / 203,243,255 / 150,230,255 / 105,185,255 / 60,140,255 / 100,110,248 / 140,80,240 / 198,85,200 /
/// 255,90,160` (le rang 1 est le milieu arrondi vers le haut de blanc et de glace : `(255·4 + 150·4 + 4) / 8 = 203`) ; avec 3 sites, le
/// rang 1 est `PALETTE[2]` ; avec 2, `PALETTE[0]` puis `PALETTE[4]` ; avec 1, `PALETTE[0]`. La palette est celle de la doc. Un rang hors du
/// nombre ou un nombre nul panique.
///
/// Ne couvre pas : l'aspect (un test ne juge pas la beauté).
#[test]
#[cfg_attr(not(feature = "etape-1"), ignore)]
fn couleur_du_rang_a_la_main() {
    assert_eq!(PALETTE, [[255, 255, 255], [150, 230, 255], [60, 140, 255], [140, 80, 240], [255, 90, 160]]);
    assert_eq!(FOND, [13, 17, 23]);
    for (rang, attendue) in PALETTE.iter().enumerate() {
        assert_eq!(couleur_du_rang(rang, 5), *attendue, "5 sites, rang {rang}");
    }
    let neuf = [
        [255, 255, 255],
        [203, 243, 255],
        [150, 230, 255],
        [105, 185, 255],
        [60, 140, 255],
        [100, 110, 248],
        [140, 80, 240],
        [198, 85, 200],
        [255, 90, 160],
    ];
    for (rang, attendue) in neuf.iter().enumerate() {
        assert_eq!(couleur_du_rang(rang, 9), *attendue, "9 sites, rang {rang}");
    }
    assert_eq!(
        [couleur_du_rang(0, 3), couleur_du_rang(1, 3), couleur_du_rang(2, 3)],
        [PALETTE[0], PALETTE[2], PALETTE[4]]
    );
    assert_eq!([couleur_du_rang(0, 2), couleur_du_rang(1, 2)], [PALETTE[0], PALETTE[4]]);
    assert_eq!(couleur_du_rang(0, 1), PALETTE[0]);
    assert!(std::panic::catch_unwind(|| couleur_du_rang(5, 5)).is_err());
    assert!(std::panic::catch_unwind(|| couleur_du_rang(0, 0)).is_err());
}

/// juste : `amas_colore` sur cinq sites (V-COL-02) `(0,0) (0,-1) (0,1) (-1,1) (-1,2)` (les cinq premiers de la graine 1) : la fenêtre
/// est `[-2, 2]²` (le plus grand `max(|x|, |y|)` est 2, côté 5). Avec `cote_cible = 1` l'échelle est 1 et l'image est 5 × 5, de pixel `(px, py)`
/// = site `(px - 2, py - 2)` : le germe en `(2, 2)` (blanc), `(0,-1)` en `(2, 1)` (glace), `(0,1)` en `(2, 3)` (bleu), `(-1,1)` en `(1, 3)`
/// (violet), `(-1,2)` en `(1, 4)` (rose), et tout le reste est le fond `(13, 17, 23)`, écrits ici un à un. Avec `cote_cible = 15` l'échelle
/// est 3 (image de 15 × 15, chaque site un bloc de 3 × 3, 45 pixels colorés) ; avec 40 elle est 8 (40 × 40) ; avec 0 elle vaut 1 ; les
/// SHA-256 des octets RVB de ces images sont ceux du code Python indépendant. Un germe seul donne une image `cote_cible × cote_cible`
/// blanche. Une liste vide panique.
///
/// Ne couvre pas : l'écriture PNG (étape 0).
#[test]
#[cfg_attr(not(feature = "etape-1"), ignore)]
fn amas_colore_a_la_main() {
    let sites = [(0, 0), (0, -1), (0, 1), (-1, 1), (-1, 2)];
    let (l, h, rvb) = amas_colore(&sites, 1);
    assert_eq!((l, h, rvb.len()), (5, 5, 75));
    let mut attendu = Vec::new();
    for py in 0..5 {
        for px in 0..5 {
            let couleur = match (px, py) {
                (2, 2) => PALETTE[0],
                (2, 1) => PALETTE[1],
                (2, 3) => PALETTE[2],
                (1, 3) => PALETTE[3],
                (1, 4) => PALETTE[4],
                _ => FOND,
            };
            attendu.extend_from_slice(&couleur);
        }
    }
    assert_eq!(rvb, attendu);
    assert_eq!(sha256_hex(&rvb), "53586e11ae050ee5fc3b9eb673cf7b258bd63b7aaf354ee95873e1952d4a1803");

    let (l, h, rvb) = amas_colore(&sites, 15);
    assert_eq!((l, h, rvb.len()), (15, 15, 15 * 15 * 3));
    let colores = rvb.chunks(3).filter(|p| *p != FOND).count();
    assert_eq!(colores, 45);
    assert_eq!(&rvb[3 * (7 * 15 + 7)..3 * (7 * 15 + 7) + 3], PALETTE[0], "centre du bloc du germe");
    assert_eq!(
        &rvb[3 * (13 * 15 + 4)..3 * (13 * 15 + 4) + 3],
        PALETTE[4],
        "bloc du dernier site (-1, 2) : pixels 3..6 de 12..15"
    );
    assert_eq!(sha256_hex(&rvb), "41ca91e2fad59c45adae08b3e4f1e503a310fa05a797153ed0d879108ab84573");
    let (l40, h40, rvb40) = amas_colore(&sites, 40);
    assert_eq!((l40, h40), (40, 40));
    assert_eq!(sha256_hex(&rvb40), "2407fb44f901eaefa57589b99e3f3d92045d126c049d62bfffcc458cd92af7d0");
    assert_eq!(amas_colore(&sites, 0).0, 5);

    let (l, h, rvb) = amas_colore(&[(0, 0)], 512);
    assert_eq!((l, h, rvb.len()), (512, 512, 512 * 512 * 3));
    assert!(rvb.chunks(3).all(|p| p == PALETTE[0]));
    assert!(std::panic::catch_unwind(|| amas_colore(&[], 10)).is_err());
}

//! Étape 2s : les quatre modèles de croissance de surface (dépôt aléatoire, relaxation, balistique, Eden) contre un
//! simulateur naïf écrit dans `etape_2s_commun`, contre des règles locales écrites à la main, contre les traces d'Eden calculées
//! à la main, et contre les hauteurs d'un second prototype écrit en Python. Valeurs attendues :
//! `references/etape-2s-valeurs.md`, identifiants `V-MOD-xx`.

mod etape_2s_commun;

use etape_2s_commun::{HAUTEUR_EDEN, NOMS, Naif, debut_alea};
use givre_lab::alea::Alea;
use givre_lab::surface::{LARGEUR_MAX, Modele, Surface};

fn modele(nom: &str) -> Modele {
    Modele::depuis_nom(nom).expect("nom de modèle")
}

/// Les largeurs et les graines des comparaisons exactes : de très petites largeurs (1, 2, 3 : les deux voisins coïncident ou se
/// touchent) à 32.
const LARGEURS: [usize; 8] = [1, 2, 3, 4, 5, 8, 13, 32];
const GRAINES: [u64; 8] = [1, 2, 3, 4, 5, 6, 7, 8];

/// Compare `Surface` au simulateur naïf à chaque particule : hauteurs, nombre de particules, temps ; pour Eden aussi le
/// périmètre (l'ordre du `Vec` compris) et l'occupation de tous les sites des rangées 0 à `h_max + 1`. Rend le simulateur
/// naïf final (pour lire ses compteurs de branches).
fn comparer(nom: &'static str, l: usize, graine: u64, particules: u64) -> Naif {
    let mut surface = Surface::new(modele(nom), l, graine);
    let mut naif = Naif::new(nom, l, graine);
    for p in 1..=particules {
        surface.deposer();
        naif.pas();
        assert_eq!(surface.hauteurs(), naif.hauteurs(), "{nom} L={l} graine {graine} : hauteurs après {p} particules");
        assert_eq!(surface.particules(), p, "{nom} L={l} : particules");
        if nom == "eden" {
            assert_eq!(
                surface.perimetre(),
                naif.perimetre,
                "{nom} L={l} graine {graine} : périmètre après {p} particules"
            );
        } else {
            assert!(surface.perimetre().is_empty(), "{nom} : le périmètre n'existe que pour Eden");
        }
    }
    assert_eq!(surface.couches(), particules / l as u64, "{nom} L={l} : couches");
    let h_max = naif.hauteurs().into_iter().max().unwrap() as usize;
    for x in 0..l {
        for y in 0..h_max + 2 {
            assert_eq!(surface.occupe(x, y), naif.occupe(x, y), "{nom} L={l} graine {graine} : occupe({x}, {y})");
        }
    }
    naif
}

fn comparer_tout(nom: &'static str) -> etape_2s_commun::Branches {
    let mut total = etape_2s_commun::Branches::default();
    for &l in &LARGEURS {
        let particules = if l >= 13 { 700 } else { 300 };
        for &graine in &GRAINES {
            let b = comparer(nom, l, graine, particules).branches;
            total.reste_strict += b.reste_strict;
            total.reste_egalite += b.reste_egalite;
            total.gauche_sans_tirage += b.gauche_sans_tirage;
            total.droite_sans_tirage += b.droite_sans_tirage;
            total.pile_gauche += b.pile_gauche;
            total.pile_droite += b.pile_droite;
            total.max_vertical += b.max_vertical;
            total.max_gauche += b.max_gauche;
            total.max_droite += b.max_droite;
            total.eden_dernier += b.eden_dernier;
            total.eden_autre += b.eden_autre;
        }
    }
    total
}

/// juste : le dépôt aléatoire donne EXACTEMENT les hauteurs d'un simulateur naïf (`i = borne(L)`, `h[i] += 1`), après chacune
/// des particules, pour 8 largeurs (1, 2, 3, 4, 5, 8, 13, 32) et 8 graines, 300 ou 700 particules (V-MOD-01, la doc du module
/// `surface`). Un tirage de la mauvaise borne (`L - 1`, `L + 1`), une colonne décalée ou un `+= 2` échouent.
///
/// Ne couvre pas : la loi des hauteurs (c'est `etape_2s_rugosite`), ni les grandes largeurs.
#[test]
#[cfg_attr(not(feature = "etape-2s"), ignore)]
fn depot_exact_contre_naif() {
    comparer_tout("depot");
}

/// juste : la relaxation donne EXACTEMENT les hauteurs d'un simulateur naïf qui répète la règle de la doc (`hi <= hg et hi <=
/// hd` : reste ; sinon le plus bas des voisins ; `hg == hd` : pièce `suivant() >> 31`, 0 à gauche, tirée seulement dans ce cas), à
/// chaque particule, pour 8 largeurs dont 1 et 2 et 8 graines (V-MOD-02). Le test vérifie aussi, par les compteurs du simulateur
/// naïf, que CHAQUE branche a été exercée (reste strictement plus bas ; reste avec égalité ; gauche ; droite ; pièce pile ;
/// pièce face) : un `<` pour un `<=`, un voisin oublié, un ex æquo mal résolu, une pièce tirée en trop ou en moins, un modulo
/// faux voient leur divergence ici.
///
/// Ne couvre pas : l'exposant β (`etape_2s_exposants`).
#[test]
#[cfg_attr(not(feature = "etape-2s"), ignore)]
fn relaxation_exacte_contre_naif() {
    let b = comparer_tout("relaxation");
    for (nom, compte) in [
        ("reste_strict", b.reste_strict),
        ("reste_egalite", b.reste_egalite),
        ("gauche_sans_tirage", b.gauche_sans_tirage),
        ("droite_sans_tirage", b.droite_sans_tirage),
        ("pile_gauche", b.pile_gauche),
        ("pile_droite", b.pile_droite),
    ] {
        assert!(compte >= 50, "la branche {nom} n'est exercée que {compte} fois : le test est trop faible");
    }
}

/// juste : le dépôt balistique donne EXACTEMENT les hauteurs d'un simulateur naïf (`h[i] = max(h[i] + 1, h[i-1], h[i+1])`,
/// lu avant la mise à jour), à chaque particule, 8 largeurs (dont 1 et 2) et 8 graines (V-MOD-03) ; les compteurs du simulateur
/// naïf montrent que le maximum vient tour à tour de `h[i] + 1`, du voisin de gauche et du voisin de droite. Un `max` sans le
/// `+ 1`, un voisin oublié, un modulo faux, une lecture après la mise à jour échouent.
///
/// Ne couvre pas : l'exposant β.
#[test]
#[cfg_attr(not(feature = "etape-2s"), ignore)]
fn balistique_exact_contre_naif() {
    let b = comparer_tout("balistique");
    for (nom, compte) in [("max_vertical", b.max_vertical), ("max_gauche", b.max_gauche), ("max_droite", b.max_droite)]
    {
        assert!(compte >= 50, "la branche {nom} n'est exercée que {compte} fois : le test est trop faible");
    }
}

/// juste : Eden donne EXACTEMENT l'état d'un simulateur naïf (le périmètre est un `Vec` ordonné ; `k = borne(len)` ; `swap_remove(k)` ;
/// voisins dans l'ordre (x+1, y), (x, y+1), (x-1, y), (x, y-1), modulo L, ajoutés en fin s'ils sont vides et pas déjà dans le
/// périmètre) : à chaque particule les hauteurs ET le périmètre (dans l'ordre du `Vec`), puis à la fin l'occupation de tous les
/// sites, pour 8 largeurs et 8 graines (V-MOD-04). Le simulateur naïf teste l'appartenance au périmètre par recherche linéaire et
/// tient l'occupation en `Vec<Vec<bool>>` : rien de commun avec la structure de `Surface`. Un `remove` pour un `swap_remove`, un
/// autre ordre de voisins, un voisin oublié, un doublon dans le périmètre, un modulo faux échouent. Les compteurs montrent que
/// `k` était tour à tour le dernier élément et un autre.
///
/// Ne couvre pas : l'exposant β ; les largeurs au-delà de 32.
#[test]
#[cfg_attr(not(feature = "etape-2s"), ignore)]
fn eden_exact_contre_naif() {
    let b = comparer_tout("eden");
    assert!(b.eden_dernier >= 50 && b.eden_autre >= 1000, "{b:?}");
}

/// juste : trois régimes de hasard confirmés par le simulateur naïf sur 4 modèles : l'état du générateur à la fin est exactement
/// celui du simulateur naïf (le nombre de tirages consommés est donc identique, y compris les pièces de la relaxation et les
/// rejets de `borne`), pour 20 graines, L = 7 et 500 particules (V-MOD-05). On le lit dans `Surface::octets()` : pour les
/// trois modèles à hauteurs les 16 octets du générateur sont à l'offset `8 + 1 + 4 + 8 + 8·L` (écriture canonique de la doc).
///
/// Ne couvre pas : Eden (son écriture canonique est vérifiée en entier dans `etape_2s_ecriture`).
#[test]
#[cfg_attr(not(feature = "etape-2s"), ignore)]
fn consommation_du_hasard() {
    for nom in ["depot", "relaxation", "balistique"] {
        for graine in 1..=20_u64 {
            let mut surface = Surface::new(modele(nom), 7, graine);
            let mut naif = Naif::new(nom, 7, graine);
            for _ in 0..500 {
                surface.deposer();
                naif.pas();
            }
            let octets = surface.octets();
            let debut = debut_alea(7);
            assert_eq!(octets[debut..debut + 16], naif.alea.octets(), "{nom} graine {graine}");
        }
    }
}

/// juste : `deposer_couche()` fait exactement `L` pas : après `t` couches, `particules() = t · L`, `couches() = t`, et l'état est
/// celui de `t · L` appels à `deposer()` (V-MOD-06) ; `couches()` est la division entière `particules / L` : après 3 pas sur
/// 5 colonnes, `couches() = 0` ; après 12, 2.
///
/// Ne couvre pas : le temps de Eden quand le périmètre change (c'est le même compteur).
#[test]
#[cfg_attr(not(feature = "etape-2s"), ignore)]
fn couche_et_temps() {
    for nom in NOMS {
        let (mut a, mut b) = (Surface::new(modele(nom), 5, 9), Surface::new(modele(nom), 5, 9));
        for t in 1..=6_u64 {
            a.deposer_couche();
            for _ in 0..5 {
                b.deposer();
            }
            assert_eq!((a.particules(), a.couches()), (5 * t, t), "{nom}");
            assert_eq!(a.hauteurs(), b.hauteurs(), "{nom}");
            assert_eq!(a.octets(), b.octets(), "{nom}");
        }
        let mut c = Surface::new(modele(nom), 5, 9);
        for _ in 0..3 {
            c.deposer();
        }
        assert_eq!((c.particules(), c.couches()), (3, 0), "{nom}");
        for _ in 0..9 {
            c.deposer();
        }
        assert_eq!((c.particules(), c.couches()), (12, 2), "{nom}");
        assert_eq!(c.largeur(), 5);
        assert_eq!(c.modele(), modele(nom));
    }
}

/// juste : même graine, même surface ; deux graines, deux surfaces ; deux largeurs ou deux modèles, deux surfaces (V-MOD-07).
/// Pour chaque modèle, 20 graines donnent 20 écritures canoniques deux à deux différentes (L = 16, 100 particules), et la même
/// graine refait exactement la même.
///
/// Ne couvre pas : l'égalité entre Linux et Windows (la CI rejoue la référence d'empreinte).
#[test]
#[cfg_attr(not(feature = "etape-2s"), ignore)]
fn determinisme_et_graines() {
    for nom in NOMS {
        let mut vues = std::collections::BTreeSet::new();
        for graine in 0..20_u64 {
            let faire = || {
                let mut s = Surface::new(modele(nom), 16, graine);
                for _ in 0..100 {
                    s.deposer();
                }
                s.octets()
            };
            let octets = faire();
            assert_eq!(octets, faire(), "{nom} graine {graine} : deux lancers diffèrent");
            assert!(vues.insert(octets), "{nom} : deux graines donnent le même état (graine {graine})");
        }
    }
}

/// juste : les noms des modèles : `nom()` rend `depot`, `relaxation`, `balistique`, `eden` ; `depuis_nom` est l'inverse
/// exact (même casse seulement : `Depot`, ` depot`, `eden ` et la chaîne vide donnent `None`) ; `TOUS` liste les quatre dans
/// cet ordre ; `code()` rend 0, 1, 2, 3 (V-MOD-08, la doc de `Modele`).
///
/// Ne couvre pas : les autres noms que le fichier de scénario pourrait contenir (c'est `etape_2s_ecriture`).
#[test]
#[cfg_attr(not(feature = "etape-2s"), ignore)]
fn noms_des_modeles() {
    assert_eq!(Modele::TOUS.len(), 4);
    for (k, nom) in NOMS.iter().enumerate() {
        let m = Modele::TOUS[k];
        assert_eq!(m.nom(), *nom);
        assert_eq!(Modele::depuis_nom(nom), Some(m));
        assert_eq!(m.code(), k as u8);
    }
    for mauvais in ["Depot", " depot", "eden ", "", "dépôt", "relaxation2", "EDEN", "balistic"] {
        assert_eq!(Modele::depuis_nom(mauvais), None, "{mauvais:?}");
    }
}

/// Un cas local : `(profil, colonne choisie, profil attendu)`.
type Cas = (&'static [u64], usize, &'static [u64]);

fn appliquer_cas(nom: &str, cas: &[Cas]) {
    for &(avant, i, apres) in cas {
        let mut s = Surface::depuis_hauteurs(modele(nom), avant, 11);
        assert_eq!(s.hauteurs(), avant);
        assert_eq!(s.particules(), avant.iter().sum::<u64>());
        s.appliquer(i);
        assert_eq!(s.hauteurs(), apres, "{nom} : {avant:?}, colonne {i}");
        assert_eq!(
            s.particules(),
            avant.iter().sum::<u64>() + 1,
            "{nom} : {avant:?}, colonne {i} : une particule de plus"
        );
    }
}

/// juste : le dépôt aléatoire sur des profils écrits à la main : `appliquer(i)` ajoute 1 à la colonne `i` et à elle seule
/// (V-MOD-09, la règle 1 de la doc), y compris pour `L = 1` et aux deux bords.
///
/// Ne couvre pas : le tirage de la colonne.
#[test]
#[cfg_attr(not(feature = "etape-2s"), ignore)]
fn depot_regle_a_la_main() {
    appliquer_cas(
        "depot",
        &[
            (&[2, 3, 1, 3, 2], 2, &[2, 3, 2, 3, 2]),
            (&[2, 3, 1, 3, 2], 0, &[3, 3, 1, 3, 2]),
            (&[2, 3, 1, 3, 2], 4, &[2, 3, 1, 3, 3]),
            (&[0], 0, &[1]),
            (&[5, 5], 1, &[5, 6]),
        ],
    );
}

/// juste : la relaxation, règle locale écrite à la main, sans hasard, sur des profils de 5 colonnes (V-MOD-10, la règle 2 de la
/// doc) : reste en `i` si `hi <= hg et hi <= hd` (dont les égalités avec un voisin ou avec les deux) ; sinon va au voisin le
/// plus bas ; le côté gauche et le côté droit ; les bords (colonne 0 : son voisin de gauche est la colonne 4 ; colonne 4 : son
/// voisin de droite est la colonne 0) ; `L = 1`. Les cas b et c distinguent `<=` de `<` (b : hi = hg = hd ; c : hi = hg < hd), le
/// cas g une règle qui ne regarderait que « les deux voisins plus bas » (hd < hi <= hg), le cas h que `hi == hd` n'est pas un
/// départ.
///
/// Ne couvre pas : le cas `hg == hd < hi` (la pièce : `relaxation_piece`).
#[test]
#[cfg_attr(not(feature = "etape-2s"), ignore)]
fn relaxation_regle_a_la_main() {
    appliquer_cas(
        "relaxation",
        &[
            // a : plus bas que les deux voisins : reste
            (&[2, 3, 1, 3, 2], 2, &[2, 3, 2, 3, 2]),
            // b : égal aux deux voisins : reste (la convention : seul un voisin STRICTEMENT plus bas attire)
            (&[2, 3, 3, 3, 2], 2, &[2, 3, 4, 3, 2]),
            // c : égal au voisin de gauche, plus bas que celui de droite : reste
            (&[2, 3, 3, 5, 2], 2, &[2, 3, 4, 5, 2]),
            // c' : égal au voisin de droite, plus bas que celui de gauche : reste
            (&[2, 5, 3, 3, 2], 2, &[2, 5, 4, 3, 2]),
            // e : voisin de droite plus bas que celui de gauche et que hi : va à droite
            (&[2, 5, 4, 3, 2], 2, &[2, 5, 4, 4, 2]),
            // f : voisin de gauche plus bas que celui de droite et que hi : va à gauche
            (&[2, 3, 4, 5, 2], 2, &[2, 4, 4, 5, 2]),
            // g : hd < hi <= hg : va à droite (le voisin de gauche n'est pas plus bas)
            (&[2, 4, 4, 3, 2], 2, &[2, 4, 4, 4, 2]),
            // h : hg < hi, hd == hi : va à gauche
            (&[2, 3, 4, 4, 2], 2, &[2, 4, 4, 4, 2]),
            // i : colonne 0, voisin de gauche = colonne 4 (plus bas) : va en colonne 4
            (&[4, 9, 9, 9, 3], 0, &[4, 9, 9, 9, 4]),
            // j : colonne 4, voisin de droite = colonne 0 (plus bas) : va en colonne 0
            (&[3, 9, 9, 9, 4], 4, &[4, 9, 9, 9, 4]),
            // k : colonne 0 plus basse que les deux : reste
            (&[1, 9, 9, 9, 2], 0, &[2, 9, 9, 9, 2]),
            // l : colonne 4 plus basse que les deux : reste
            (&[7, 9, 9, 9, 1], 4, &[7, 9, 9, 9, 2]),
            // m : la surface plate reste en i
            (&[0, 0, 0, 0, 0], 3, &[0, 0, 0, 1, 0]),
            // n : une seule colonne : ses deux voisins sont elle-même : reste
            (&[4], 0, &[5]),
            // o : deux colonnes, hi < l'autre : reste ; hi == l'autre : reste
            (&[1, 5], 0, &[2, 5]),
            (&[5, 5], 1, &[5, 6]),
        ],
    );
}

/// juste : l'exhaustif de la relaxation : pour TOUS les triplets `(hg, hi, hd)` de 0 à 3 (64 cas), pour la colonne 0, la colonne
/// 2 et la colonne 4 d'un profil de 5 colonnes (les autres colonnes valent 7, pour qu'un voisin mal indexé se voie), la colonne
/// qui reçoit la particule est celle que donne la règle écrite ici en trois lignes, tableau des cas sans tirage compris (V-MOD-11).
/// Pour les cas à pièce (`hg == hd < hi`) la colonne dépend de la pièce, et le test vérifie qu'elle est l'une des deux.
///
/// Ne couvre pas : quelle pièce.
#[test]
#[cfg_attr(not(feature = "etape-2s"), ignore)]
fn relaxation_exhaustive_locale() {
    for hg in 0..4_u64 {
        for hi in 0..4_u64 {
            for hd in 0..4_u64 {
                for i in [0_usize, 2, 4] {
                    let mut profil = vec![7_u64; 5];
                    profil[(i + 4) % 5] = hg;
                    profil[i] = hi;
                    profil[(i + 1) % 5] = hd;
                    let mut s = Surface::depuis_hauteurs(modele("relaxation"), &profil, 3);
                    s.appliquer(i);
                    let apres = s.hauteurs();
                    let differences: Vec<usize> = (0..5).filter(|&c| apres[c] != profil[c]).collect();
                    assert_eq!(differences.len(), 1, "{profil:?}, colonne {i} : une colonne exactement change");
                    let c = differences[0];
                    assert_eq!(apres[c], profil[c] + 1);
                    let attendu: Vec<usize> = if hi <= hg && hi <= hd {
                        vec![i]
                    } else if hg < hd {
                        vec![(i + 4) % 5]
                    } else if hd < hg {
                        vec![(i + 1) % 5]
                    } else {
                        vec![(i + 4) % 5, (i + 1) % 5]
                    };
                    assert!(
                        attendu.contains(&c),
                        "{profil:?}, colonne {i} : la particule va en {c}, attendu {attendu:?}"
                    );
                }
            }
        }
    }
}

/// juste : la pièce de la relaxation (V-MOD-12, la règle 2 de la doc) : quand `hg == hd < hi`, `appliquer` tire exactement un
/// `suivant()` ; `pile = suivant() >> 31` ; pile 0 : la particule va à gauche (colonne `i - 1`), pile 1 : à droite (`i + 1`).
/// Le test prédit la pièce de 40 graines (0 à 39) avec un `Alea::new(graine)` du test, vérifie le résultat, vérifie que le
/// générateur a avancé d'exactement un tirage (ses octets dans `octets()`) et que les deux côtés sont sortis. Dans les cas SANS
/// pièce (la particule reste, ou un voisin est strictement plus bas), le générateur n'a pas avancé. Cas particuliers :
/// colonne 0 (voisins : colonnes 4 et 1) et `L = 2` (les deux voisins sont la même colonne, la pièce est tirée quand même).
///
/// Ne couvre pas : la loi de la pièce (un demi) sur de longues suites (la simulation naïve à 300 particules sur 8 graines la
/// voit exercée 50 fois au moins de chaque côté).
#[test]
#[cfg_attr(not(feature = "etape-2s"), ignore)]
fn relaxation_piece() {
    // (profil, colonne choisie, colonne gauche, colonne droite)
    let cas: [(&[u64], usize, usize, usize); 3] = [
        (&[2, 3, 4, 3, 2], 2, 1, 3),
        (&[5, 2, 9, 9, 2], 0, 4, 1), // le tirage se fait aux bords de la bande
        (&[3, 0], 0, 1, 1),          // L = 2 : une seule autre colonne
    ];
    for (profil, i, gauche, droite) in cas {
        let (mut vu_gauche, mut vu_droite) = (false, false);
        for graine in 0..40_u64 {
            let mut alea = Alea::new(graine);
            let pile = alea.suivant() >> 31;
            let mut s = Surface::depuis_hauteurs(modele("relaxation"), profil, graine);
            let avant = s.octets();
            let l = profil.len();
            s.appliquer(i);
            let apres = s.hauteurs();
            let colonne = if pile == 0 { gauche } else { droite };
            let mut attendu = profil.to_vec();
            attendu[colonne] += 1;
            assert_eq!(apres, attendu, "{profil:?} graine {graine} (pile {pile})");
            let o = s.octets();
            let debut = debut_alea(l);
            assert_eq!(o[debut..debut + 16], alea.octets(), "{profil:?} graine {graine} : un seul tirage de pièce");
            assert_ne!(avant[debut..debut + 16], o[debut..debut + 16]);
            vu_gauche |= pile == 0;
            vu_droite |= pile == 1;
        }
        assert!(vu_gauche && vu_droite, "{profil:?} : les 40 graines n'ont pas donné les deux côtés");
    }
    // sans pièce : le générateur ne bouge pas
    for (profil, i) in [(&[2_u64, 3, 1, 3, 2][..], 2), (&[2, 5, 4, 3, 2], 2), (&[2, 3, 3, 3, 2], 2), (&[4], 0)] {
        let mut s = Surface::depuis_hauteurs(modele("relaxation"), profil, 5);
        let avant = s.octets();
        s.appliquer(i);
        let debut = debut_alea(profil.len());
        assert_eq!(avant[debut..debut + 16], s.octets()[debut..debut + 16], "{profil:?} : aucun tirage attendu");
    }
}

/// juste : la relaxation sur DEUX colonnes est déterministe pour les hauteurs, quelle que soit la graine (V-MOD-21, preuve par récurrence
/// sur la règle 2 de la doc : la particule finit toujours sur une colonne de hauteur minimale, la sienne en cas d'égalité, car
/// les deux voisins de `i` sont l'autre colonne) : après `n` particules, les hauteurs triées sont `(n / 2, n - n / 2)`, la
/// différence vaut `n mod 2`. Vérifié à chaque pas, pour 30 graines et 100 particules. Une règle qui enverrait la particule vers
/// le voisin le plus HAUT, ou qui l'y laisserait quand `hi` est strictement plus haut, échoue.
///
/// Ne couvre pas : `L >= 3` (aucune telle formule fermée).
#[test]
#[cfg_attr(not(feature = "etape-2s"), ignore)]
fn relaxation_deux_colonnes_alternent() {
    for graine in 1..=30_u64 {
        let mut s = Surface::new(Modele::Relaxation, 2, graine);
        for n in 1..=100_u64 {
            s.deposer();
            let mut h = s.hauteurs();
            h.sort_unstable();
            assert_eq!(h, vec![n / 2, n - n / 2], "graine {graine} après {n} particules");
        }
    }
}

/// juste : le dépôt balistique, règle locale écrite à la main (V-MOD-13, la règle 3 de la doc) : `h[i] = max(h[i] + 1, h[i-1],
/// h[i+1])`, les voisins lus avant la mise à jour, modulo `L`. Le cas b (`h[i]` plus haut que les deux voisins) distingue
/// `max(h + 1, …)` de `max(h, …)` ; c et d le voisin de droite et celui de gauche ; e et f les bords ; `L = 1` et `L = 2`.
///
/// Ne couvre pas : le tirage de la colonne.
#[test]
#[cfg_attr(not(feature = "etape-2s"), ignore)]
fn balistique_regle_a_la_main() {
    appliquer_cas(
        "balistique",
        &[
            // a : plus bas que les deux voisins : prend le plus haut des voisins ? non : max(2, 3, 3) = 3
            (&[2, 3, 1, 3, 2], 2, &[2, 3, 3, 3, 2]),
            // b : plus haut que les deux voisins : h + 1
            (&[2, 3, 5, 3, 2], 2, &[2, 3, 6, 3, 2]),
            // c : le voisin de droite domine
            (&[2, 3, 3, 5, 2], 2, &[2, 3, 5, 5, 2]),
            // d : le voisin de gauche domine
            (&[2, 5, 3, 3, 2], 2, &[2, 5, 5, 3, 2]),
            // e : colonne 0, voisin de gauche = colonne 4 (domine)
            (&[1, 3, 9, 9, 6], 0, &[6, 3, 9, 9, 6]),
            // f : colonne 4, voisin de droite = colonne 0 (domine)
            (&[6, 3, 9, 2, 1], 4, &[6, 3, 9, 2, 6]),
            // g : surface plate : h + 1
            (&[3, 3, 3, 3, 3], 2, &[3, 3, 4, 3, 3]),
            // h : h[i] + 1 égal au voisin : le maximum est atteint par les deux
            (&[2, 4, 3, 4, 2], 2, &[2, 4, 4, 4, 2]),
            // i : une colonne : ses deux voisins sont elle-même
            (&[4], 0, &[5]),
            // j : deux colonnes : le voisin est l'autre colonne des deux côtés
            (&[3, 7], 0, &[7, 7]),
            (&[7, 3], 1, &[7, 7]),
            (&[7, 7], 0, &[8, 7]),
        ],
    );
}

/// Les hauteurs et le périmètre attendus d'une trace d'Eden.
type Trace = (usize, &'static [(usize, usize)], &'static [u64]);

fn suivre_eden(largeur: usize, etapes: &[Trace]) {
    let mut s = Surface::new(Modele::Eden, largeur, 1);
    let depart: Vec<(usize, usize)> = (0..largeur).map(|x| (x, 0)).collect();
    assert_eq!(s.perimetre(), depart);
    assert_eq!(s.hauteurs(), vec![0; largeur]);
    for (n, &(k, perimetre, hauteurs)) in etapes.iter().enumerate() {
        s.appliquer(k);
        assert_eq!(s.perimetre(), perimetre, "pas {} (k = {k}) : périmètre", n + 1);
        assert_eq!(s.hauteurs(), hauteurs, "pas {} (k = {k}) : hauteurs", n + 1);
        assert_eq!(s.particules(), n as u64 + 1);
    }
}

/// juste : Eden sur 3 colonnes, tracé À LA MAIN d'après la règle 4 de la doc (V-MOD-14) et recoupé par le second prototype
/// Python (`ref_surface.py`, `trace_eden.py`) : départ `[(0,0),(1,0),(2,0)]` ; `k = 1` occupe `(1,0)`, le périmètre devient
/// `[(0,0),(2,0)]` après `swap_remove` puis `(1,1)` est ajouté en fin (`(2,0)` et `(0,0)` y sont déjà) ; `k = 0` occupe
/// `(0,0)` (le dernier, `(1,1)`, prend la place 0) ; etc. Les cinq pas `k = 1, 0, 1, 2, 0` donnent les périmètres et profils
/// écrits ci-dessous. Voit un `remove` pour un `swap_remove`, un autre ordre de voisins, un voisin du dessus oublié, un
/// doublon, un modulo faux (`(x - 1)` de la colonne 0 est la colonne 2).
///
/// Ne couvre pas : le hasard (le choix `k` est imposé).
#[test]
#[cfg_attr(not(feature = "etape-2s"), ignore)]
fn eden_trace_a_la_main() {
    suivre_eden(
        3,
        &[
            (1, &[(0, 0), (2, 0), (1, 1)], &[0, 1, 0]),
            (0, &[(1, 1), (2, 0), (0, 1)], &[1, 1, 0]),
            (1, &[(1, 1), (0, 1), (2, 1)], &[1, 1, 1]),
            (2, &[(1, 1), (0, 1), (2, 2)], &[1, 1, 2]),
            (0, &[(2, 2), (0, 1), (1, 2)], &[1, 2, 2]),
        ],
    );
}

/// juste : Eden, une trace à trou (V-MOD-15, même méthode que `eden_trace_a_la_main`) : sur 3 colonnes, `k = 1, 2, 2` occupent
/// `(1,0)`, `(1,1)`, puis `(2,1)` alors que `(2,0)` est encore vide : la colonne 2 a la hauteur 2 (le profil du dessus :
/// `1 +` la plus haute rangée occupée), pas 1 ; `(2,0)` reste dans le périmètre et `occupe(2, 0)` est faux, `occupe(2, 1)`
/// vrai.
///
/// Ne couvre pas : l'exposant ; des trous plus grands.
#[test]
#[cfg_attr(not(feature = "etape-2s"), ignore)]
fn eden_trace_avec_trou() {
    suivre_eden(
        3,
        &[
            (1, &[(0, 0), (2, 0), (1, 1)], &[0, 1, 0]),
            (2, &[(0, 0), (2, 0), (2, 1), (1, 2), (0, 1)], &[0, 2, 0]),
            (2, &[(0, 0), (2, 0), (0, 1), (1, 2), (2, 2)], &[0, 2, 2]),
        ],
    );
    let mut s = Surface::new(Modele::Eden, 3, 1);
    for k in [1, 2, 2] {
        s.appliquer(k);
    }
    assert!(!s.occupe(2, 0) && s.occupe(2, 1) && s.occupe(1, 0) && s.occupe(1, 1) && !s.occupe(0, 0));
    assert!(!s.occupe(2, 2) && !s.occupe(0, 5) && !s.occupe(1, 100_000));
}

/// juste : Eden sur une seule colonne et sur deux colonnes, à la main (V-MOD-16) : `L = 1` : `(x + 1) mod 1 = (x - 1) mod 1 = x`,
/// déjà occupé ; le périmètre est toujours `[(0, y)]` avec `y` le nombre de particules, et la hauteur vaut ce nombre ;
/// `L = 2` : `x + 1` et `x - 1` sont la même colonne, ajoutée une seule fois (le doublon est exclu par « déjà dans le
/// périmètre ») : départ `[(0,0),(1,0)]`, `k = 0` occupe `(0,0)`, le périmètre est `[(1,0),(0,1)]` (`(1,0)` y est déjà).
///
/// Ne couvre pas : les largeurs plus grandes (`eden_trace_a_la_main`).
#[test]
#[cfg_attr(not(feature = "etape-2s"), ignore)]
fn eden_petites_largeurs() {
    let mut s = Surface::new(Modele::Eden, 1, 4);
    for n in 1..=10_u64 {
        assert_eq!(s.perimetre(), vec![(0, n as usize - 1)]);
        s.deposer();
        assert_eq!(s.hauteurs(), vec![n]);
    }
    suivre_eden(2, &[(0, &[(1, 0), (0, 1)], &[1, 0]), (0, &[(0, 1), (1, 1)], &[1, 1])]);
}

/// juste : les invariants du périmètre d'Eden, vérifiés à chaque pas sur des suites tirées au hasard (L = 5, 8, 13 ; 6 graines ;
/// 250 particules), SANS regarder l'ordre (V-MOD-17) : chaque site du périmètre est vide, touche un site occupé ou le substrat
/// (la rangée 0), et n'apparaît qu'une fois ; réciproquement tout site vide qui touche un site occupé ou le substrat y est ;
/// le nombre de sites occupés vaut le nombre de particules ; la hauteur de chaque colonne est 1 + la plus haute rangée occupée ;
/// aucun site occupé n'est dans le périmètre. Ces propriétés viennent de la définition du périmètre, pas de l'ordre.
///
/// Ne couvre pas : l'ordre du `Vec` (`eden_exact_contre_naif`).
#[test]
#[cfg_attr(not(feature = "etape-2s"), ignore)]
fn eden_invariants_du_perimetre() {
    for l in [5_usize, 8, 13] {
        for graine in 1..=6_u64 {
            let mut s = Surface::new(Modele::Eden, l, graine);
            for p in 1..=250_u64 {
                s.deposer();
                let perimetre = s.perimetre();
                let h = s.hauteurs();
                let h_max = *h.iter().max().unwrap() as usize;
                let mut occupes = 0;
                for (x, &hauteur) in h.iter().enumerate() {
                    let mut haute = 0;
                    for y in 0..h_max + 2 {
                        if s.occupe(x, y) {
                            occupes += 1;
                            haute = y as u64 + 1;
                        }
                    }
                    assert_eq!(hauteur, haute, "L={l} graine {graine} pas {p} : hauteur de la colonne {x}");
                }
                assert_eq!(occupes, p, "L={l} graine {graine} pas {p} : sites occupés");
                for x in 0..l {
                    for y in 0..h_max + 2 {
                        let touche = y == 0
                            || s.occupe(x, y - 1)
                            || s.occupe((x + 1) % l, y)
                            || s.occupe((x + l - 1) % l, y)
                            || s.occupe(x, y + 1);
                        let attendu = !s.occupe(x, y) && touche;
                        let compte = perimetre.iter().filter(|&&site| site == (x, y)).count();
                        assert_eq!(compte, usize::from(attendu), "L={l} graine {graine} pas {p} : site ({x}, {y})");
                    }
                }
                assert!(perimetre.iter().all(|&(_, y)| y < h_max + 2));
            }
        }
    }
}

/// juste : la capacité d'Eden (V-MOD-18, la doc du module `surface`) : sur une seule colonne, chaque pas occupe un site de plus
/// et ajoute au périmètre celui du dessus ; le pas numéro `HAUTEUR_EDEN` occupe la rangée `HAUTEUR_EDEN - 1` et veut ajouter
/// la rangée `HAUTEUR_EDEN` : il PANIQUE ; les `HAUTEUR_EDEN - 1` précédents non. (Le test recopie la valeur 4096 de la doc.)
///
/// Ne couvre pas : l'état de la surface après la panique (la doc ne le fixe pas).
#[test]
#[cfg_attr(not(feature = "etape-2s"), ignore)]
fn eden_capacite() {
    assert_eq!(givre_lab::surface::HAUTEUR_EDEN, HAUTEUR_EDEN);
    let mut s = Surface::new(Modele::Eden, 1, 1);
    for _ in 0..HAUTEUR_EDEN - 1 {
        s.deposer();
    }
    assert_eq!(s.hauteurs(), vec![HAUTEUR_EDEN as u64 - 1]);
    let resultat = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| s.deposer()));
    assert!(resultat.is_err(), "le pas numéro HAUTEUR_EDEN doit paniquer");
}

/// juste : les paniques documentées (V-MOD-19) : `new` avec `largeur` 0 ou `LARGEUR_MAX + 1` ; `LARGEUR_MAX` lui-même est permis ;
/// `depuis_hauteurs` avec Eden, avec une liste vide ; `appliquer` avec une colonne hors de `0..L` (L et au-delà) pour les trois
/// premiers modèles, un indice hors du périmètre pour Eden ; `occupe` avec `x >= L` ; `depuis_hauteurs` avec plus de `LARGEUR_MAX`
/// colonnes (4096 est permis).
///
/// Ne couvre pas : les messages de panique.
#[test]
#[cfg_attr(not(feature = "etape-2s"), ignore)]
fn paniques_documentees() {
    fn panique<F: FnOnce()>(f: F) -> bool {
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(f)).is_err()
    }
    assert_eq!(LARGEUR_MAX, 4096);
    for m in Modele::TOUS {
        assert!(panique(|| drop(Surface::new(m, 0, 1))), "{m:?} largeur 0");
        assert!(panique(|| drop(Surface::new(m, LARGEUR_MAX + 1, 1))), "{m:?} largeur max + 1");
        assert!(!panique(|| drop(Surface::new(m, LARGEUR_MAX, 1))), "{m:?} largeur max");
        assert!(!panique(|| drop(Surface::new(m, 1, 1))), "{m:?} largeur 1");
    }
    assert!(panique(|| drop(Surface::depuis_hauteurs(Modele::Eden, &[0, 0], 1))));
    assert!(panique(|| drop(Surface::depuis_hauteurs(Modele::Depot, &vec![0; LARGEUR_MAX + 1], 1))));
    assert!(!panique(|| drop(Surface::depuis_hauteurs(Modele::Depot, &vec![0; LARGEUR_MAX], 1))));
    for m in [Modele::Depot, Modele::Relaxation, Modele::Balistique] {
        assert!(panique(|| drop(Surface::depuis_hauteurs(m, &[], 1))), "{m:?} liste vide");
        assert!(panique(|| Surface::depuis_hauteurs(m, &[1, 2, 3], 1).appliquer(3)), "{m:?} colonne L");
        assert!(panique(|| Surface::depuis_hauteurs(m, &[1, 2, 3], 1).appliquer(100)), "{m:?} colonne 100");
        assert!(!panique(|| Surface::depuis_hauteurs(m, &[1, 2, 3], 1).appliquer(2)), "{m:?} colonne L - 1");
        assert!(panique(|| _ = Surface::new(m, 3, 1).occupe(3, 0)), "{m:?} occupe x = L");
    }
    assert!(panique(|| Surface::new(Modele::Eden, 4, 1).appliquer(4)), "Eden : indice = longueur du périmètre");
    assert!(!panique(|| Surface::new(Modele::Eden, 4, 1).appliquer(3)), "Eden : dernier indice");
    assert!(panique(|| _ = Surface::new(Modele::Eden, 3, 1).occupe(3, 0)), "Eden occupe x = L");
}

/// juste : (second prototype) les hauteurs finales de 12 cas calculées par un second prototype écrit en Python
/// (`ref_surface.py` : autre langage, autre structure de données, PCG32 par `ref_alea.py`, tous deux d'après la doc et le C de
/// référence de PCG, rien du code Rust), V-MOD-20 : modèle, largeur, graine, nombre de particules, hauteurs.
///
/// Ne couvre pas : l'état hors des hauteurs (le périmètre et le générateur, dans `etape_2s_ecriture` par l'empreinte).
#[test]
#[cfg_attr(not(feature = "etape-2s"), ignore)]
fn hauteurs_du_second_prototype() {
    let cas: [(&str, usize, u64, u64, &[u64]); 12] = [
        ("depot", 8, 1, 200, &[25, 32, 16, 20, 27, 38, 21, 21]),
        ("depot", 16, 3, 300, &[22, 21, 18, 15, 15, 18, 21, 11, 15, 26, 16, 24, 17, 14, 25, 22]),
        ("depot", 2, 7, 40, &[19, 21]),
        ("relaxation", 8, 1, 200, &[25, 25, 25, 25, 25, 26, 25, 24]),
        ("relaxation", 16, 3, 300, &[19, 19, 19, 18, 18, 18, 18, 18, 18, 17, 20, 20, 20, 20, 19, 19]),
        ("relaxation", 2, 7, 40, &[20, 20]),
        ("balistique", 8, 1, 200, &[44, 46, 46, 47, 47, 49, 51, 49]),
        ("balistique", 16, 3, 300, &[41, 42, 43, 41, 41, 38, 38, 35, 36, 36, 32, 43, 43, 41, 44, 42]),
        ("balistique", 3, 5, 60, &[36, 31, 35]),
        ("eden", 8, 1, 200, &[28, 22, 25, 24, 25, 24, 27, 27]),
        ("eden", 16, 3, 300, &[19, 20, 20, 21, 19, 19, 18, 18, 19, 18, 19, 21, 20, 18, 18, 17]),
        ("eden", 5, 4, 120, &[24, 25, 25, 24, 22]),
    ];
    for (nom, l, graine, n, attendu) in cas {
        let mut s = Surface::new(modele(nom), l, graine);
        for _ in 0..n {
            s.deposer();
        }
        assert_eq!(s.hauteurs(), attendu, "{nom} L={l} graine {graine} N={n}");
    }
}

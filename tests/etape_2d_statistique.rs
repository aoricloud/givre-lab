//! Étape 2d : les deux tendances de la preuve de l'étape (`FEUILLE-DE-ROUTE.md`, moitié (a)), MESURÉES en moyenne sur plusieurs graines :
//! la densité `N / Rg²` d'un amas de `N` sites croît quand la probabilité de collage `p` baisse, et l'anisotropie `S4` croît avec la réduction du
//! bruit `m`. Valeurs : `references/etape-2d-valeurs.md`, identifiants `V-STAT2-xx`.
//!
//! **Ce que les sources disent** (lu le 8/11/2026 ; voir la fiche pour le détail et ce qui n'a pas pu être lu).
//! - Le collage : T. A. Witten et L. M. Sander, Phys. Rev. B 27, 5686 (1983), section sur la condition aux limites (pages 5691 et 5692, texte lu) :
//!   un marcheur qui ne colle que pendant une fraction `s` de ses rencontres introduit une longueur `λ = 1/s` ; les branches s'épaississent
//!   (figure 5 : 3000 particules, `s = 0,5`, « thickening of the branches ») ; la fonction de corrélation est constante aux distances inférieures à `λ`
//!   et suit la loi de puissance de la DLA au-delà. Aucun nombre n'est donné pour la densité d'un amas de `N` sites. B. Ranguelov et al.
//!   (arXiv:1105.5558, non relu par un comité) décrivent la même règle (le marcheur qui échoue continue sa marche) et trouvent un empilement de plus
//!   en plus dense quand `K_stick` baisse (`1`, `0,5`, `0,1`, `0,01`).
//! - Le bruit : C. Tang (1985), J. Kertész et T. Vicsek (1986), J. Nittmann et H. E. Stanley (1986) : textes NON lus (seul le résumé de Tang). Lu : S. G.
//!   Alves et S. C. Ferreira Jr, J. Phys. A 39, 2843 (2006) (introduction et résumé) : des compteurs à chaque site de croissance, un site vide n'est occupé
//!   qu'après avoir été choisi `M` fois, et « the effects of the axial anisotropy are enhanced as the noise reduction parameter is increased » pour la DLA
//!   sur réseau carré ; R. C. Ball, N. E. Bowler, L. M. Sander et E. Somfai, Phys. Rev. E 66, 026109 (2002) (arXiv:cond-mat/0108252, texte lu) : la
//!   réduction du bruit fait apparaître plus tôt les effets de réseau, et les grands amas (de même que ceux à bruit réduit) poussent un « four fingered
//!   dendrite ». Aucune de ces sources ne donne la valeur de `S4` ni une densité pour nos tailles : **les nombres ci-dessous sont mesurés sur le
//!   prototype, non publiés**.
//!
//! **Ce qui est mesuré** : `S4` et `N / Rg²` de `AmasVariante` (la dynamique est fixée appel par appel : toute implémentation fidèle donne EXACTEMENT les
//! mêmes nombres sur les mêmes graines ; les bandes servent à ce que les graines des tests ne soient pas des graines de chance).
//!
//! | grandeur (graines 1 à 30) | moyenne | écart-type d'une graine |
//! |---|---|---|
//! | densité, `N = 1000`, `p = 1` | 1,3079 | 0,1305 |
//! | densité, `p = 0,5` | 1,6072 | 0,1520 |
//! | densité, `p = 0,2` | 2,1599 | 0,1348 |
//! | densité, `p = 0,05` | 3,4216 | 0,2330 |
//! | `S4`, `N = 400`, `m = 1` | 0,1492 | 0,2746 |
//! | `S4`, `m = 2` | 0,4745 | 0,2152 |
//! | `S4`, `m = 4` | 0,7375 | 0,1126 |
//! | `S4`, `m = 8` | 0,8855 | 0,0307 |
//!
//! Les moyennes de 20 graines des deux séries du test (graines 1 à 20, puis 101 à 120, qui n'a pas servi à fixer les marges) : densité `1,3054` / `1,3546` (`p = 1`),
//! `1,6293` / `1,6456`, `2,1403` / `2,0961`, `3,4178` / `3,4288` (`p = 0,05`) ; `S4` `0,1955` / `0,2240` (`m = 1`), `0,4545` / `0,4184`, `0,7269` / `0,7548`, `0,8899` /
//! `0,8861` (`m = 8`).
//!
//! **Les marges** : l'écart entre deux configurations consécutives, mesuré sur 30 graines, moins trois erreurs-types de l'écart pour 20 graines
//! (`sqrt((σ_a² + σ_b²) / 20)`), arrondi vers le bas : densité `0,16` (de `p = 1` à `0,5` : écart 0,2993, 3 ET = 0,1344), `0,41` (de 0,5 à 0,2 : 0,5527, 3 ET =
//! 0,1363), `1,08` (de 0,2 à 0,05 : 1,2617, 3 ET = 0,1805) ; anisotropie `0,09` (de `m = 1` à 2 : 0,3253, 3 ET = 0,2340), `0,10` (de 2 à 4 : 0,2630, 3 ET = 0,1629),
//! `0,06` (de 4 à 8 : 0,1479, 3 ET = 0,0782) ; et `S4` à `m = 8` au moins `0,84` (le juge, voir plus bas). **Validé sur douze blocs de vingt graines neuves**
//! (101 à 120 et 201 à 400 par blocs de 20) : les écarts observés les plus faibles sont `0,1897`, `0,4505`, `1,2115` (densité) et `0,1748`, `0,2382`, `0,1106`
//! (`S4`), la plus faible moyenne de `S4` à `m = 8` `0,8778`. Aucune marge n'a été élargie pour un cas ; aucune graine n'a été choisie parce qu'elle passe.
//!
//! **Le juge** (second prototype écrit d'après la doc, mêmes nombres au bit près ; graines 501 à 700, dix blocs de 20) : densité `1,3464`, `1,5795`, `2,1463`,
//! `3,4200` (écarts-types `0,108` à `0,214`) ; `S4` `0,1480`, `0,4334`, `0,7452`, `0,8815` (écarts-types `0,319`, `0,232`, `0,110`, `0,035`). Les six marges tiennent
//! dans les dix blocs (écarts les plus faibles `0,2017`, `0,4738`, `1,1938` ; `0,2002`, `0,2088`, `0,1003`). Le seuil de `S4(m = 8)` était `0,86` : un bloc sur
//! dix (graines 541 à 560) donne `0,8566`, car l'écart-type d'une graine est `0,035` et non `0,031`. Le seuil est donc `0,84` : la moyenne de 200 graines moins
//! cinq erreurs-types d'une série de 20 (`5 × 0,0079`), `0,017` sous le plus faible bloc observé, et toujours bien au-dessus de `m = 4` (`0,745`). Ce n'est pas
//! pour faire passer les graines du test (elles donnent `0,8899` et `0,8861`) : c'est pour qu'elles ne soient pas des graines de chance.
//!
//! **Les tailles** : `N = 1000` pour le collage et `N = 400` pour le bruit, pas `2000` : chaque site coûte `m` marcheurs, et `m = 8` à `N = 2000` prend
//! `3,8` s par graine (le test de `m = 8` sur 20 graines en demanderait 76).

mod etape_2d_commun;

/// Le délai d'un test de simulation, en secondes (voir `etape_2d_commun::avec_delai`).
const DELAI: u64 = 120;

use givre_lab::dla_variantes::{AmasVariante, Variante, anisotropie, densite};
use std::sync::OnceLock;
use std::sync::atomic::{AtomicUsize, Ordering};

/// Les quatre probabilités de collage, en Q0.32 : `1` (`2^32`), `0,5`, `0,2` (`858993459,2`) et `0,05` (`214748364,8`), arrondis au plus proche.
const COLLAGES: [u64; 4] = [1 << 32, 1 << 31, 858_993_459, 214_748_365];
/// Les quatre niveaux de réduction du bruit.
const BRUITS: [u32; 4] = [1, 2, 4, 8];
const N_COLLAGE: usize = 1000;
const N_BRUIT: usize = 400;
/// Les deux séries de graines (la seconde n'a pas servi à fixer les marges).
const SERIES: [(u64, u64); 2] = [(1, 20), (101, 120)];

/// Les moyennes d'une série de graines : la densité pour chaque `p` de [`COLLAGES`], `S4` pour chaque `m` de [`BRUITS`].
struct Moyennes {
    densite: [f64; 4],
    anisotropie: [f64; 4],
}

fn calculer(premiere: u64, derniere: u64) -> Moyennes {
    let graines: Vec<u64> = (premiere..=derniere).collect();
    // une tâche par (configuration, graine), 8 configurations ; chaque résultat ne dépend que de sa tâche, jamais de l'ordre d'exécution
    let taches = 8 * graines.len();
    let prochaine = AtomicUsize::new(0);
    let mut resultats: Vec<(usize, f64)> = std::thread::scope(|s| {
        let fils: Vec<_> = (0..4)
            .map(|_| {
                s.spawn(|| {
                    let mut faits = Vec::new();
                    loop {
                        let tache = prochaine.fetch_add(1, Ordering::Relaxed);
                        if tache >= taches {
                            return faits;
                        }
                        let (configuration, graine) = (tache / graines.len(), graines[tache % graines.len()]);
                        let valeur = if configuration < 4 {
                            let mut amas = AmasVariante::new(graine, 300, Variante::Collage(COLLAGES[configuration]));
                            amas.croitre_jusqu_a(N_COLLAGE);
                            densite(amas.sites())
                        } else {
                            let mut amas = AmasVariante::new(graine, 300, Variante::Bruit(BRUITS[configuration - 4]));
                            amas.croitre_jusqu_a(N_BRUIT);
                            anisotropie(amas.sites())
                        };
                        faits.push((tache, valeur));
                    }
                })
            })
            .collect();
        fils.into_iter().flat_map(|f| f.join().expect("un fil de mesure a paniqué")).collect()
    });
    resultats.sort_by_key(|&(tache, _)| tache);
    let mut moyennes = [0.0; 8];
    for (configuration, moyenne) in moyennes.iter_mut().enumerate() {
        let somme: f64 =
            resultats[configuration * graines.len()..(configuration + 1) * graines.len()].iter().map(|r| r.1).sum();
        *moyenne = somme / graines.len() as f64;
    }
    Moyennes {
        densite: [moyennes[0], moyennes[1], moyennes[2], moyennes[3]],
        anisotropie: [moyennes[4], moyennes[5], moyennes[6], moyennes[7]],
    }
}

/// Les moyennes de la série `serie` (0 ou 1), calculées une seule fois pour tous les tests du fichier.
fn moyennes(serie: usize) -> &'static Moyennes {
    static CACHE: [OnceLock<Moyennes>; 2] = [OnceLock::new(), OnceLock::new()];
    CACHE[serie].get_or_init(|| calculer(SERIES[serie].0, SERIES[serie].1))
}

/// régression : (direction publiée, aucun nombre publié : voir l'en-tête ; V-STAT2-01) à `N = 1000` sites, la densité moyenne `N / Rg²` de 20 amas CROÎT STRICTEMENT quand la probabilité de
/// collage baisse : `p = 1`, `0,5`, `0,2`, `0,05` donnent quatre moyennes strictement croissantes (Witten et Sander 1983 : les branches
/// s'épaississent, de façon d'autant plus marquée que `s` est petit). Vérifié sur la série des graines 1 à 20 et sur la série 101 à 120. Aucun nombre
/// n'est exigé ici (ceux-là sont mesurés : voir `regression_marges_mesurees`). Un collage qui colle avec la probabilité `1 − p`, un tirage
/// inversé ou un `p` ignoré échouent.
///
/// C'est la direction que publient les sources, mais les nombres jugés sont ceux de NOTRE modèle à cette taille : le test fige ce que fait notre code, il
/// ne prouve pas une valeur publiée (d'où « régression », et non « juste »).
///
/// Ne couvre pas : les valeurs des densités (aucune valeur publiée à comparer), ni les grands amas (`N` de l'ordre de `10^5`, où la DLA retrouve sa
/// loi de puissance au-delà de `1/p`).
#[test]
#[cfg_attr(not(feature = "etape-2"), ignore)]
fn densite_croit_quand_le_collage_baisse() {
    etape_2d_commun::avec_delai(DELAI, || {
        for serie in 0..2 {
            let d = moyennes(serie).densite;
            for k in 0..3 {
                assert!(
                    d[k + 1] > d[k],
                    "série {serie} : densités moyennes {d:?} : pas strictement croissantes quand p baisse"
                );
            }
        }
    });
}

/// régression : (direction publiée, aucun nombre publié : voir l'en-tête ; V-STAT2-02) à `N = 400` sites, l'anisotropie moyenne `S4` de 20 amas CROÎT STRICTEMENT avec la réduction du bruit :
/// `m = 1`, `2`, `4`, `8` donnent quatre moyennes strictement croissantes (Alves et Ferreira 2006 : l'anisotropie axiale augmente avec `M` ; Ball et al.
/// 2002 : les amas à bruit réduit poussent quatre doigts), et à `m = 8` elle est POSITIVE (les bras sont sur les axes, pas sur les diagonales : `S4 > 0`).
/// Vérifié sur les séries 1 à 20 et 101 à 120. Un compteur qui compterait à l'envers, un seuil `m` ignoré, ou un `S4` de mauvais signe (qui donnerait
/// des valeurs négatives) échouent.
///
/// C'est la direction que publient les sources, mais les nombres jugés sont ceux de NOTRE modèle à cette taille (d'où « régression »).
///
/// Ne couvre pas : les valeurs de `S4` (voir `regression_marges_mesurees`), le comportement à de très grands amas, ni l'inversion de l'anisotropie vers les
/// diagonales que d'autres règles de croissance produisent.
#[test]
#[cfg_attr(not(feature = "etape-2"), ignore)]
fn anisotropie_croit_avec_le_bruit() {
    etape_2d_commun::avec_delai(DELAI, || {
        for serie in 0..2 {
            let a = moyennes(serie).anisotropie;
            for k in 0..3 {
                assert!(
                    a[k + 1] > a[k],
                    "série {serie} : anisotropies moyennes {a:?} : pas strictement croissantes avec m"
                );
            }
            assert!(a[3] > 0.0, "série {serie} : S4 moyen à m = 8 : {}", a[3]);
        }
    });
}

/// régression : (V-STAT2-03) les MARGES mesurées sur le prototype (voir l'en-tête : moyenne sur 30 graines moins trois erreurs-types, validée sur douze blocs de
/// vingt graines neuves). Sur chacune des séries 1 à 20 et 101 à 120 : les densités moyennes augmentent d'au moins `0,16`, `0,41` puis `1,08` quand `p`
/// passe de 1 à 0,5, 0,2 puis 0,05 ; les anisotropies moyennes d'au moins `0,09`, `0,10` puis `0,06` quand `m` passe de 1 à 2, 4 puis 8 ; et `S4` à `m = 8`
/// vaut au moins `0,84` (quatre bras sur les axes : `+1` est la valeur d'une croix parfaite). Ces seuils sont des résultats de NOTRE code, non des valeurs
/// publiées.
///
/// Ne couvre pas : que ces seuils soient justes en soi (rien de publié ne les donne), ni les amas d'une autre taille.
#[test]
#[cfg_attr(not(feature = "etape-2"), ignore)]
fn regression_marges_mesurees() {
    etape_2d_commun::avec_delai(DELAI, || {
        for serie in 0..2 {
            let m = moyennes(serie);
            for (k, marge) in [0.16, 0.41, 1.08].into_iter().enumerate() {
                assert!(
                    m.densite[k + 1] - m.densite[k] >= marge,
                    "série {serie} : densités {:?}, marge {marge} entre {k} et {}",
                    m.densite,
                    k + 1
                );
            }
            for (k, marge) in [0.09, 0.10, 0.06].into_iter().enumerate() {
                assert!(
                    m.anisotropie[k + 1] - m.anisotropie[k] >= marge,
                    "série {serie} : anisotropies {:?}, marge {marge} entre {k} et {}",
                    m.anisotropie,
                    k + 1
                );
            }
            assert!(m.anisotropie[3] >= 0.84, "série {serie} : S4 à m = 8 : {}", m.anisotropie[3]);
        }
    });
}

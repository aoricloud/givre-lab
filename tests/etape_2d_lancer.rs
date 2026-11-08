//! Étape 2d : `lancer` et le binaire `givre` sur un scénario `collage` ou `bruit`. Valeurs attendues : `references/etape-2d-valeurs.md`, identifiants
//! `V-LVAR-xx`.
//!
//! Le contrat est dans la documentation de `src/dla_variantes.rs` (section « La directive `collage <p>` ou `bruit <m>` et la sortie de `lancer` »).
//! Les empreintes, les mesures et les aperçus attendus des petits cas viennent du code Python indépendant (`ref_variantes.py`, d'après la doc seule,
//! SHA-256 par `hashlib`) ; seul `regression_references_collage_et_bruit` compare à un fichier produit par NOTRE code.

mod common;

mod etape_2d_commun;

/// Le délai d'un test de simulation, en secondes (voir `etape_2d_commun::avec_delai`).
const DELAI: u64 = 120;

use givre_lab::dla::{COTE_APERCU, amas_colore};
use givre_lab::dla_variantes::{AmasVariante, Variante, anisotropie};
use givre_lab::empreinte::sha256_hex;
use givre_lab::lancer::{ErreurLancer, Sortie, lancer};
use givre_lab::scenario::lire_scenario;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn lancer_texte(texte: &str, graine: u64) -> Result<Sortie, ErreurLancer> {
    lancer(&lire_scenario(texte).expect("scénario valide"), graine)
}

fn racine() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// Le plus petit entier `r` tel que tous les sites vérifient `x² + y² <= r²`.
fn rayon_entier(sites: &[(i32, i32)]) -> i64 {
    let plus_grand =
        sites.iter().map(|&(x, y)| i64::from(x) * i64::from(x) + i64::from(y) * i64::from(y)).max().unwrap();
    let mut r = 0;
    while r * r < plus_grand {
        r += 1;
    }
    r
}

/// Le rayon de giration par les formules de la doc de `mesures::rayon_de_giration` : sommes entières exactes, une division, une racine.
fn giration(sites: &[(i32, i32)]) -> f64 {
    let n = sites.len() as i128;
    let (mut sx, mut sy, mut sxx, mut syy) = (0_i128, 0_i128, 0_i128, 0_i128);
    for &(x, y) in sites {
        let (x, y) = (i128::from(x), i128::from(y));
        sx += x;
        sy += y;
        sxx += x * x;
        syy += y * y;
    }
    let num = n * (sxx + syy) - (sx * sx + sy * sy);
    (num as f64 / (n * n) as f64).sqrt()
}

/// Les mesures attendues d'un amas de `n` sites, d'après la doc : `particules,rayon,rayon_giration,anisotropie`, une ligne par puissance de 2 de 2 à
/// `n`, puis la ligne `n` si elle n'est pas déjà là, les deux nombres à six décimales.
fn mesures_attendues(sites: &[(i32, i32)]) -> String {
    let n = sites.len();
    let mut ks: Vec<usize> = std::iter::successors(Some(2_usize), |k| Some(2 * k)).take_while(|&k| k <= n).collect();
    if ks.last() != Some(&n) {
        ks.push(n);
    }
    let mut texte = String::from("particules,rayon,rayon_giration,anisotropie\n");
    for k in ks {
        texte.push_str(&format!(
            "{k},{},{:.6},{:.6}\n",
            rayon_entier(&sites[..k]),
            giration(&sites[..k]),
            anisotropie(&sites[..k])
        ));
    }
    texte
}

// --------------------------------------------------------------------------------------------------- les petits cas

/// juste : quatre petits scénarios, graines fixées (V-LVAR-01), dont l'empreinte, le CSV et l'aperçu sont ceux du code Python indépendant : (1) `boite 41 41`,
/// `collage 0.5`, `dla 40`, graine 3 : empreinte `ba884d64…a66c`, le CSV `particules,rayon,rayon_giration,anisotropie` à quatre colonnes
/// (lignes `2,1,0.500000,1.000000`, `4,2,0.935414,1.000000`, `8,3,1.403122,0.933333`, `16,5,2.367587,0.787611`, `32,7,3.309993,0.542047`,
/// `40,8,3.783104,0.523582`), un PNG de 510 × 510 pixels dont les octets RVB ont le SHA-256 `6b51baf6…bb91` ; (2) `boite 61 61`, `bruit 3`, `dla 100`,
/// graine 2 : empreinte `8d739b1a…b965`, sept lignes dont `4,2,0.866025,-0.333333` (une anisotropie NÉGATIVE s'écrit avec son signe) et
/// `100,14,6.988068,0.621133`, PNG de 486 × 486, `0b8b9f8e…2e45` ; (3) `boite 61 61`, `collage 0.05`, `dla 64`, graine 1 : empreinte `c7d20b42…1f3f`,
/// PNG de 494 × 494, `83f8aac6…ae81` ; (4) `boite 5 5`, `bruit 2`, `dla 1`, graine 5 : le germe seul, la seule ligne `1,0,0.000000,0.000000`, empreinte
/// `dbdd3d82…c049`, un PNG de 512 × 512 tout blanc.
///
/// Ne couvre pas : de grands amas (voir `exemples_collage_et_bruit`).
#[test]
#[cfg_attr(not(feature = "etape-2"), ignore)]
fn lancer_variantes_a_la_main() {
    etape_2d_commun::avec_delai(DELAI, || {
        let cas: [(&str, u64, &str, &str, u32, &str); 3] = [
            (
                "boite 41 41\ncollage 0.5\ndla 40\n",
                3,
                "ba884d6490c5b05efadddce90df95381c31752fd3ae41de6c4e677a876c3a66c",
                "particules,rayon,rayon_giration,anisotropie\n2,1,0.500000,1.000000\n4,2,0.935414,1.000000\n8,3,1.403122,0.933333\n16,5,2.367587,0.787611\n32,7,3.309993,0.542047\n40,8,3.783104,0.523582\n",
                510,
                "6b51baf6f82026a55bcd50209a963f3c3019f0c97ee05d206e9969006981bb91",
            ),
            (
                "boite 61 61\nbruit 3\ndla 100\n",
                2,
                "8d739b1a706c062eca7c117148b958b35f16becda08e583987d92b42beefb965",
                "particules,rayon,rayon_giration,anisotropie\n2,1,0.500000,1.000000\n4,2,0.866025,-0.333333\n8,3,1.648863,0.722222\n16,5,2.554102,0.750611\n32,7,3.591439,0.793157\n64,10,5.272621,0.705800\n100,14,6.988068,0.621133\n",
                486,
                "0b8b9f8e739d8d7d9cde35d0ddb8ad5b065e57cacb749530c6da01f8101d2e45",
            ),
            (
                "boite 61 61\ncollage 0.05\ndla 64\n",
                1,
                "c7d20b427910623295fb17acc63feb562f03af5073368ba78166e5394d041f3f",
                "particules,rayon,rayon_giration,anisotropie\n2,1,0.500000,1.000000\n4,2,0.866025,-0.333333\n8,3,1.192424,-0.297297\n16,4,1.950561,-0.070707\n32,7,3.553030,0.588535\n64,10,4.779258,0.438004\n",
                494,
                "83f8aac65eecd0492a7d66481d367f2eeb40ffd38362ce328162f9506caeae81",
            ),
        ];
        for (texte, graine, empreinte, mesures, cote, sha_rvb) in cas {
            let sortie = lancer_texte(texte, graine).expect("lancer");
            assert_eq!(sortie.empreinte, empreinte, "{texte:?}");
            assert_eq!(sortie.mesures, mesures, "{texte:?}");
            let png = common::relire_png(&sortie.apercu).expect("PNG valide");
            assert_eq!((png.largeur, png.hauteur), (cote, cote), "{texte:?}");
            assert_eq!(sha256_hex(&png.rvb), sha_rvb, "{texte:?}");
        }
        let germe =
            lancer_texte("boite 5 5\nbruit 2\ndla 1\n", 5).expect("le germe seul tient dans une boîte de 5 × 5");
        assert_eq!(germe.mesures, "particules,rayon,rayon_giration,anisotropie\n1,0,0.000000,0.000000\n");
        assert_eq!(germe.empreinte, "dbdd3d821c866d483dd4421f9496e3f90dfcb691464bb25c6bb26d375936c049");
        let png = common::relire_png(&germe.apercu).expect("PNG valide");
        assert_eq!((png.largeur, png.hauteur), (512, 512));
        assert!(png.rvb.chunks(3).all(|p| p == [255, 255, 255]));
    });
}

/// juste : les valeurs neutres, par `lancer` (V-LVAR-02) : `boite 41 41`, `dla 40`, graine 3, avec `collage 1` ou avec `bruit 1`, donnent les MÊMES sites que la
/// `dla` seule de l'étape 1 : l'aperçu est identique octet pour octet à celui de `dla 40` seul (PNG de 510 × 510, octets RVB de SHA-256 `b66c262c…3ff5`,
/// celui de l'étape 1) et les trois premières colonnes du CSV sont celles de l'étape 1 (`4,3,1.118034`, `40,9,4.295856`…) ; le CSV a une quatrième
/// colonne (`40,9,4.295856,-0.064856`) ; l'empreinte est différente de celle de la `dla` seule (`ae05f58a…43ee`) et de l'une à l'autre : `575d0cc6…17cd`
/// (collage) et `c516a61e…b247` (bruit), étiquette et paramètre comptent. `collage 0.9999999999` (converti en `2^32`) donne la même sortie que `collage 1`.
///
/// Ne couvre pas : les variantes à paramètre non neutre (autres tests).
#[test]
#[cfg_attr(not(feature = "etape-2"), ignore)]
fn valeurs_neutres_par_lancer() {
    etape_2d_commun::avec_delai(DELAI, || {
        let etape_1 = lancer_texte("boite 41 41\ndla 40\n", 3).expect("lancer");
        assert_eq!(etape_1.empreinte, "ae05f58a2dd9cd0183bd7e3d2bc6db329dd62e5ba4e9135799cc10d39fc343ee");
        let attendu = [
            ("boite 41 41\ncollage 1\ndla 40\n", "575d0cc6ceecd03f8bb918e6f4cd49dc79c9584f0acff08687ba514a59bc17cd"),
            ("boite 41 41\ndla 40\nbruit 1\n", "c516a61ea0c02299e5b42c7b5042eeb579676542cb3f51452ecdda81a631b247"),
            (
                "boite 41 41\ncollage 0.9999999999\ndla 40\n",
                "575d0cc6ceecd03f8bb918e6f4cd49dc79c9584f0acff08687ba514a59bc17cd",
            ),
        ];
        for (texte, empreinte) in attendu {
            let sortie = lancer_texte(texte, 3).expect("lancer");
            assert_eq!(sortie.empreinte, empreinte, "{texte:?}");
            assert_eq!(sortie.apercu, etape_1.apercu, "{texte:?} : mêmes sites, même aperçu");
            let colonnes = |csv: &str, n: usize| -> Vec<String> {
                csv.lines().map(|l| l.split(',').take(n).collect::<Vec<_>>().join(",")).collect()
            };
            let quatre = colonnes(&sortie.mesures, 4);
            assert_eq!(quatre[0], "particules,rayon,rayon_giration,anisotropie", "{texte:?}");
            assert_eq!(quatre[6], "40,9,4.295856,-0.064856", "{texte:?}");
            assert_eq!(
                colonnes(&sortie.mesures, 3)[1..],
                colonnes(&etape_1.mesures, 3)[1..],
                "{texte:?} : trois premières colonnes de l'étape 1"
            );
            assert!(sortie.mesures.lines().all(|l| l.split(',').count() == 4), "{texte:?} : quatre colonnes partout");
        }
        let png = common::relire_png(&etape_1.apercu).expect("PNG valide");
        assert_eq!(sha256_hex(&png.rvb), "b66c262cdc61b6c26505acf485f5a4b3022292227c6700625da8645ec4dc3ff5");
    });
}

/// juste : pour plusieurs variantes (`collage 0.5`, `collage 0.2`, `bruit 2`, `bruit 3`), plusieurs `n` (1, 2, 3, 5, 8, 17, 64) et deux graines, les mesures sont
/// celles que la doc décrit, reconstruites ici depuis les sites d'un `AmasVariante` (V-LVAR-03) : l'en-tête `particules,rayon,rayon_giration,anisotropie`,
/// une ligne par puissance de 2 `<= n` à partir de 2, puis la ligne `n` si elle n'y est pas déjà (jamais deux lignes identiques), le rayon étant celui des `k`
/// premiers sites, la giration et l'anisotropie celles de ces `k` sites avec six décimales (l'anisotropie vient de `anisotropie`, jugée à `etape_2d_mesures`) ;
/// l'empreinte est celle de `AmasVariante::empreinte()` d'un amas de même graine, de même variante et de `demi_cote = (min(largeur, hauteur) - 1) / 2` (ici
/// `boite 61 41` : 20 ; la même empreinte pour `boite 41 99`) ; l'aperçu est le PNG de `amas_colore(sites, COTE_APERCU)` ; chaque ligne finit par un seul `\n`.
///
/// Ne couvre pas : les amas qui débordent la boîte (voir `refus_et_debordement_des_variantes`).
#[test]
#[cfg_attr(not(feature = "etape-2"), ignore)]
fn mesures_empreinte_et_apercu_suivent_la_doc() {
    etape_2d_commun::avec_delai(DELAI, || {
        let variantes = [
            ("collage 0.5", Variante::Collage(2_147_483_648)),
            ("collage 0.2", Variante::Collage(858_993_459)),
            ("bruit 2", Variante::Bruit(2)),
            ("bruit 3", Variante::Bruit(3)),
        ];
        for (directive, variante) in variantes {
            for graine in [1_u64, 2] {
                for n in [1_usize, 2, 3, 5, 8, 17, 64] {
                    let mut amas = AmasVariante::new(graine, 20, variante);
                    amas.croitre_jusqu_a(n);
                    let sortie = lancer_texte(&format!("boite 61 41\n{directive}\ndla {n}\n"), graine).expect("lancer");
                    let contexte = format!("{directive}, graine {graine}, {n} particules");
                    assert_eq!(sortie.mesures, mesures_attendues(amas.sites()), "{contexte}");
                    assert!(sortie.mesures.ends_with('\n') && !sortie.mesures.contains('\r'));
                    assert_eq!(sortie.empreinte, amas.empreinte(), "{contexte} : empreinte");
                    let (l, h, rvb) = amas_colore(amas.sites(), COTE_APERCU);
                    let png = common::relire_png(&sortie.apercu).expect("PNG valide");
                    assert_eq!((png.largeur, png.hauteur), (l, h), "{contexte} : dimensions de l'aperçu");
                    assert_eq!(png.rvb, rvb, "{contexte} : pixels de l'aperçu");
                    let autre = lancer_texte(&format!("boite 41 99\ndla {n}\n{directive}\n"), graine).expect("lancer");
                    assert_eq!(
                        autre.empreinte, sortie.empreinte,
                        "{contexte} : le demi_cote ne dépend que du plus petit côté"
                    );
                }
            }
        }
        assert_eq!(COTE_APERCU, 512);
    });
}

/// juste : les deux scénarios d'exemple, lus depuis les fichiers du dépôt (V-LVAR-04). `scenarios/dla-collage.txt` (`boite 301 301`, `collage 0.1`, `dla 2000`),
/// graine 1, et `scenarios/dla-bruit.txt` (`boite 301 301`, `bruit 4`, `dla 1500`), graine 1 : l'empreinte, le CSV et les octets RVB de l'aperçu sont ceux
/// du code Python indépendant (même amas, écriture canonique de `demi_cote` 150). (Voir la fiche : les valeurs Python.)
///
/// Ne couvre pas : les autres graines (leurs mesures sont aux tests statistiques).
#[test]
#[cfg_attr(not(feature = "etape-2"), ignore)]
fn exemples_collage_et_bruit() {
    etape_2d_commun::avec_delai(DELAI, || {
        let collage =
            fs::read_to_string(racine().join("scenarios").join("dla-collage.txt")).expect("scenarios/dla-collage.txt");
        let sortie = lancer_texte(&collage, 1).expect("lancer");
        assert_eq!(sortie.empreinte, "a0194e72fcb388124dbb83cc77c3fa0eaa7fa10e41653450e3716bae123c9f7a");
        assert_eq!(
            sortie.mesures,
            "particules,rayon,rayon_giration,anisotropie\n2,1,0.500000,1.000000\n4,2,0.707107,-0.333333\n8,3,1.198958,-0.052632\n16,5,1.844544,0.209380\n32,9,3.257054,0.690105\n64,11,4.494952,0.527919\n128,14,6.368884,0.571062\n256,19,9.564802,0.141444\n512,26,13.900905,-0.064831\n1024,39,20.280865,-0.212015\n2000,58,29.657825,-0.254681\n"
        );
        let png = common::relire_png(&sortie.apercu).expect("PNG valide");
        assert_eq!((png.largeur, png.hauteur), (436, 436));
        assert_eq!(sha256_hex(&png.rvb), "1f76c713e817fcc8ede9879a7ecbcbc3407ba40c79eaa52e2b18c99486906c8a");

        let bruit =
            fs::read_to_string(racine().join("scenarios").join("dla-bruit.txt")).expect("scenarios/dla-bruit.txt");
        let sortie = lancer_texte(&bruit, 1).expect("lancer");
        assert_eq!(sortie.empreinte, "ae85c651db7678d96ffede20fb509ef3cdaa353c34869c4275bcd7b51fb72e4f");
        assert_eq!(
            sortie.mesures,
            "particules,rayon,rayon_giration,anisotropie\n2,1,0.500000,1.000000\n4,2,1.118034,1.000000\n8,3,1.536026,0.283582\n16,5,2.358495,0.610860\n32,7,3.340536,0.696638\n64,9,5.089411,0.515778\n128,16,7.677136,0.467930\n256,28,11.951110,0.728459\n512,41,18.843856,0.839548\n1024,57,28.294525,0.796180\n1500,68,35.092070,0.790418\n"
        );
        let png = common::relire_png(&sortie.apercu).expect("PNG valide");
        assert_eq!((png.largeur, png.hauteur), (405, 405));
        assert_eq!(sha256_hex(&png.rvb), "4eeb900bba37430c5e21c6d069df649edec8783efba0c7ee12efd791117fb527");
    });
}

/// juste : mêmes entrées, mêmes sorties, octet pour octet (le PNG compris) ; deux graines donnent deux empreintes, deux CSV et deux aperçus différents ;
/// `collage 0.5` et `collage 0.2`, `bruit 2` et `bruit 3`, `collage 0.5` et `bruit 2` donnent des empreintes différentes ; `dla 40` et `dla 41` aussi ;
/// une directive de variante rend une empreinte qui n'est pas celle du scénario vide de la même boîte (V-LVAR-05).
///
/// Ne couvre pas : l'égalité d'empreintes entre Linux et Windows (la CI la joue : tout est en entiers).
#[test]
#[cfg_attr(not(feature = "etape-2"), ignore)]
fn determinisme_et_graines_des_variantes() {
    etape_2d_commun::avec_delai(DELAI, || {
        for directive in ["collage 0.3", "bruit 2"] {
            let texte = format!("boite 41 41\n{directive}\ndla 60\n");
            let a = lancer_texte(&texte, 5).unwrap();
            let b = lancer_texte(&texte, 5).unwrap();
            assert_eq!(a, b);
            let c = lancer_texte(&texte, 6).unwrap();
            assert_ne!(a.empreinte, c.empreinte);
            assert_ne!(a.mesures, c.mesures);
            assert_ne!(a.apercu, c.apercu);
            let d = lancer_texte(&format!("boite 41 41\n{directive}\ndla 61\n"), 5).unwrap();
            assert_ne!(a.empreinte, d.empreinte);
            let vide = lancer_texte("boite 41 41\n", 5).unwrap();
            assert_ne!(a.empreinte, vide.empreinte);
        }
        let empreinte =
            |directive: &str| lancer_texte(&format!("boite 41 41\n{directive}\ndla 60\n"), 5).unwrap().empreinte;
        let toutes = ["collage 0.5", "collage 0.2", "bruit 2", "bruit 3", "collage 1", "bruit 1"].map(empreinte);
        let distinctes: std::collections::BTreeSet<&String> = toutes.iter().collect();
        assert_eq!(distinctes.len(), toutes.len());
    });
}

/// juste : les refus (V-LVAR-06), tous `ErreurLancer::DirectiveInconnue(ligne)` avec la PLUS PETITE ligne refusée (lignes ignorées comptées) : une
/// `collage` ou `bruit` fautive (`collage 0`, `bruit 0`, `collage 1.5`, `collage 0.5 0.5` : sa ligne) même devant une `dla` valide ; une `dla` fautive
/// derrière une `collage` valide (la ligne de la `dla`) ; une `collage` ou `bruit` VALIDE sans aucune directive `dla` (sa ligne) ; une seconde directive
/// de variante, de l'une ou l'autre sorte, même avec une `dla` valide (la ligne de la seconde) ; une directive inconnue (sa ligne ; entre plusieurs lignes refusées, la PLUS PETITE l'emporte, jamais la plus grande) ;
/// sur un `hexagone`, la première des directives `dla`, `collage` et `bruit` ; un amas qui sort de la boîte (`boite 5 5` + `dla 100`, `boite 1 1` +
/// `dla 2`, avec `collage` ou `bruit` : la ligne de la directive `dla`) ; une `surface` (étape 2s) dans un scénario qui a une `dla`, une `collage` ou
/// une `bruit` (sa ligne : le contrat de cette étape vaut dès qu'une des trois est là, et `surface` n'y est pas un nom permis). Et ce qui est permis : `boite 1 1` + `bruit 3` + `dla 1` (le germe seul) ;
/// `boite 5 5` + `collage 0.5` + `dla 3` et + `bruit 2` + `dla 3` (trois sites tiennent toujours dans `demi_cote` 2) ; la `collage` après la `dla`.
///
/// Ne couvre pas : le message du binaire (voir `binaire_variantes`).
#[test]
#[cfg_attr(not(feature = "etape-2"), ignore)]
fn refus_et_debordement_des_variantes() {
    etape_2d_commun::avec_delai(DELAI, || {
        for (texte, ligne) in [
            ("boite 9 9\ncollage 0\ndla 5\n", 2),
            ("boite 9 9\nbruit 0\ndla 5\n", 2),
            ("boite 9 9\ncollage 0.5 0.5\ndla 5\n", 2),
            ("boite 9 9\ndla 5\ncollage 1.5\n", 3),
            ("boite 9 9\ndla 5\nbruit 0\n", 3),
            ("boite 9 9\ncollage 0.5\ndla 0\n", 3),
            ("boite 9 9\ncollage 0.5\n", 2),
            ("boite 9 9\n# c\nbruit 2\n", 3),
            ("boite 9 9\n\ndla 5\nbruit 2\nbruit 3\n", 5),
            ("boite 9 9\ncollage 0.5\nbruit 2\ndla 5\n", 3),
            ("boite 9 9\nbruit 2\ncollage 0.5\ndla 5\n", 3),
            ("boite 9 9\nbruit 2\ninconnue\ndla 5\n", 3),
            ("boite 9 9\ninconnue\nbruit 2\ndla 5\n", 2),
            ("boite 9 9\ncollage 0.5\ndla 3\ndla 4\n", 4),
            ("hexagone 5\ncollage 0.5\ndla 4\n", 2),
            ("hexagone 5\ndla 4\ncollage 0.5\n", 2),
            ("hexagone 5\n\nbruit 2\n", 3),
            ("# c\nhexagone 5\n\ndla 4\nbruit 2\n", 4),
            ("boite 5 5\ncollage 0.5\n\ndla 100\n", 4),
            ("boite 5 5\nbruit 2\ndla 100\n", 3),
            ("boite 1 1\ncollage 0.5\ndla 2\n", 3),
            ("boite 1 1\nbruit 4\n\n\ndla 2\n", 5),
            ("boite 9 9\ncollage 0\ninconnue\n", 2),
            ("boite 9 9\ninconnue\ncollage 0\n", 2),
            ("boite 9 9\ndla 0\ncollage 0.5\ninconnue\n", 2),
            ("boite 9 9\ncollage 0.5\ninconnue\ndla 3\ndla 4\n", 3),
            ("boite 9 9\ncollage 0.5\nbruit 2\ndla 0\n", 3),
            ("boite 9 9\ncollage 0.5\ndla 0\ninconnue\n", 3),
            ("boite 5 5\ncollage 0.5\ninconnue\ndla 100\n", 3),
            ("hexagone 5\ndla 4\ncollage 0\n", 2),
            ("hexagone 5\nbruit 2\ndla 0\n", 2),
            // une `surface` (étape 2s) n'est qu'une directive refusée dès qu'il y a `dla`, `collage` ou `bruit`
            ("boite 9 9\nsurface depot 8 3\nbruit 2\n", 2),
            ("boite 9 9\nsurface depot 8 3\ncollage 0\n", 2),
            ("boite 9 9\nsurface depot 8 3\ncollage 0.5\ndla 5\n", 2),
        ] {
            assert_eq!(lancer_texte(texte, 1).map(|_| ()), Err(ErreurLancer::DirectiveInconnue(ligne)), "{texte:?}");
        }
        let germe =
            lancer_texte("boite 1 1\nbruit 3\ndla 1\n", 4).expect("le germe seul tient dans une boîte de 1 × 1");
        assert_eq!(germe.mesures, "particules,rayon,rayon_giration,anisotropie\n1,0,0.000000,0.000000\n");
        let png = common::relire_png(&germe.apercu).expect("PNG valide");
        assert_eq!((png.largeur, png.hauteur), (512, 512));
        for texte in [
            "boite 5 5\ncollage 0.5\ndla 3\n",
            "boite 5 5\nbruit 2\ndla 3\n",
            "boite 5 5\ndla 3\ncollage 0.5\n",
            "boite 5 5\ndla 3\nbruit 2\n",
        ] {
            for graine in [1, 9] {
                assert!(lancer_texte(texte, graine).is_ok(), "{texte:?}, graine {graine}");
            }
        }
    });
}

// ------------------------------------------------------------------------------------------------------------- le binaire

fn givre(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_givre")).args(args).output().expect("lancement de givre")
}

fn s(chemin: &Path) -> &str {
    chemin.to_str().expect("chemin en UTF-8")
}

/// juste : `givre lancer scenarios/dla-collage.txt --graine 1 --sortie <dossier>` et de même pour `scenarios/dla-bruit.txt` réussissent (code 0) et
/// écrivent exactement `empreinte.txt` (l'empreinte puis `\n`), `mesures.csv` et `apercu.png`, et rien d'autre ; ces fichiers sont ceux de `lancer` octet
/// pour octet (V-LVAR-07). Puis un scénario `collage 0` (ligne 2) et un amas qui déborde (`boite 5 5`, `bruit 2`, `dla 100`, ligne 3) font échouer le
/// binaire : code de sortie 1, un message qui contient `ligne 2` (`ligne 3`) et pas `panicked`, RIEN d'écrit ; un scénario voisin accepté
/// (`boite 21 21`, `bruit 2`, `dla 20`) réussit : un test qui ne vérifierait que des refus ne distinguerait pas ce code de celui de l'étape 1.
///
/// Ne couvre pas : les autres erreurs d'arguments (étape 0).
#[test]
#[cfg_attr(not(feature = "etape-2"), ignore)]
fn binaire_variantes() {
    etape_2d_commun::avec_delai(DELAI, || {
        for nom in ["dla-collage", "dla-bruit"] {
            let dossier = common::dossier_temporaire(nom);
            let scenario = racine().join("scenarios").join(format!("{nom}.txt"));
            let resultat = givre(&["lancer", s(&scenario), "--graine", "1", "--sortie", s(&dossier)]);
            assert_eq!(resultat.status.code(), Some(0), "{nom} : {}", String::from_utf8_lossy(&resultat.stderr));
            let mut noms: Vec<String> =
                fs::read_dir(&dossier).unwrap().map(|e| e.unwrap().file_name().into_string().unwrap()).collect();
            noms.sort();
            assert_eq!(noms, ["apercu.png", "empreinte.txt", "mesures.csv"], "{nom}");
            let attendu = lancer_texte(&fs::read_to_string(&scenario).unwrap(), 1).unwrap();
            assert_eq!(
                fs::read_to_string(dossier.join("empreinte.txt")).unwrap(),
                format!("{}\n", attendu.empreinte),
                "{nom}"
            );
            assert_eq!(fs::read_to_string(dossier.join("mesures.csv")).unwrap(), attendu.mesures, "{nom}");
            assert_eq!(fs::read(dossier.join("apercu.png")).unwrap(), attendu.apercu, "{nom}");
            let _ = fs::remove_dir_all(&dossier);
        }
        for (nom, texte, code, ligne) in [
            ("refus-collage-zero", "boite 9 9\ncollage 0\ndla 5\n", 1, "ligne 2"),
            ("refus-debordement-bruit", "boite 5 5\nbruit 2\ndla 100\n", 1, "ligne 3"),
            ("accepte-bruit", "boite 21 21\nbruit 2\ndla 20\n", 0, ""),
        ] {
            let espace = common::dossier_temporaire(nom);
            fs::create_dir_all(&espace).unwrap();
            let scenario = espace.join("scenario.txt");
            fs::write(&scenario, texte).unwrap();
            let sortie = espace.join("sortie");
            let resultat = givre(&["lancer", s(&scenario), "--graine", "1", "--sortie", s(&sortie)]);
            assert_eq!(resultat.status.code(), Some(code), "{nom} : {}", String::from_utf8_lossy(&resultat.stderr));
            let message = String::from_utf8_lossy(&resultat.stderr);
            assert!(!message.contains("panicked"), "{nom} : {message}");
            if code == 1 {
                assert!(message.contains(ligne), "{nom} : {message}");
                assert!(!sortie.exists(), "{nom} : rien ne doit être écrit");
            } else {
                let mut noms: Vec<String> =
                    fs::read_dir(&sortie).unwrap().map(|e| e.unwrap().file_name().into_string().unwrap()).collect();
                noms.sort();
                assert_eq!(noms, ["apercu.png", "empreinte.txt", "mesures.csv"]);
            }
            let _ = fs::remove_dir_all(&espace);
        }
    });
}

/// régression : `references/dla-collage-graine-1.empreinte` et `references/dla-bruit-graine-1.empreinte` contiennent l'empreinte que NOTRE code donne pour
/// `scenarios/dla-collage.txt` et `scenarios/dla-bruit.txt` avec la graine 1 (64 chiffres hexadécimaux puis `\n`). Elles ont été figées après que
/// `exemples_collage_et_bruit` a vérifié les mêmes valeurs contre le code Python indépendant : ce test ne prouve que la stabilité (V-LVAR-08).
///
/// Ne couvre pas : que la valeur soit juste (voir `exemples_collage_et_bruit`).
#[test]
#[cfg_attr(not(feature = "etape-2"), ignore)]
fn regression_references_collage_et_bruit() {
    etape_2d_commun::avec_delai(DELAI, || {
        for nom in ["dla-collage", "dla-bruit"] {
            let reference = fs::read_to_string(racine().join("references").join(format!("{nom}-graine-1.empreinte")))
                .unwrap_or_else(|_| panic!("references/{nom}-graine-1.empreinte"));
            let texte = fs::read_to_string(racine().join("scenarios").join(format!("{nom}.txt"))).unwrap();
            assert_eq!(reference, format!("{}\n", lancer_texte(&texte, 1).unwrap().empreinte), "{nom}");
        }
    });
}

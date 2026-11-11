//! Étape G1 : le rendu net d'un champ sur la grille hexagonale (`rendu_hex`) : des hexagones à leur vraie géométrie, un dégradé de bleus sur
//! fond blanc, du suréchantillonnage en entiers. Valeurs attendues : `references/etape-g1-valeurs.md`, identifiants `V-G1-RENDU-xx`.
//!
//! Le contrat est dans la documentation de `src/rendu_hex.rs`. Les images sont pour l'œil (`AGENTS.md`, règle 3) : ces tests ne jugent ni
//! l'esthétique ni la page (`image-rendering: pixelated`, qui est l'étape G3), seulement ce que la doc fixe : la géométrie, le dégradé, la
//! couverture, la symétrie, les refus, et un hachage d'image de référence.

mod etape_g1_commun;

use etape_g1_commun::*;
use givre_lab::empreinte::sha256_hex;
use givre_lab::neige::Neige;
use givre_lab::rendu_hex::{ARRETS, ErreurRendu, ReglagesImage, couleur_du_degrade, rendre};

const BLANC: [u8; 3] = [255, 255, 255];
const FONCE: [u8; 3] = [8, 48, 107];

fn pixel(image: &[u8], cote: usize, x: usize, y: usize) -> [u8; 3] {
    let d = 3 * (y * cote + x);
    [image[d], image[d + 1], image[d + 2]]
}

/// Un champ nul sauf une case.
fn champ_une_case(rayon: i32, q: i32, r: i32, valeur: i64) -> Vec<i64> {
    cases(rayon).iter().map(|&c| if c == (q, r) { valeur } else { 0 }).collect()
}

/// L'image attendue, calculée par un code NAÏF écrit d'après la doc et la géométrie EXACTE (en `f64`, avec le vrai `√3`, sans arrondi cubique ni
/// fraction rationnelle) : pour chaque échantillon, la case la plus proche par la distance euclidienne ; blanche si cette case n'est pas dans
/// l'hexagone de rayon `rayon`. Panique si un échantillon est à moins de `1e-6` d'une frontière de case (le cas d'essai serait mal choisi : la
/// doc fixe l'arrondi à `2,6e-10` près, pas au-delà).
fn naif(rayon: i32, valeurs: &[i64], cote: usize, rayon_vue: i32, sur: usize, plein: i64) -> Vec<u8> {
    let racine3 = 3.0_f64.sqrt();
    let centre = |q: i32, r: i32| (-(racine3 / 2.0) * f64::from(r), -(f64::from(q) + f64::from(r) / 2.0));
    let ordre = cases(rayon);
    let n = cote * sur;
    let large = f64::from(2 * rayon_vue + 1);
    let marge = rayon + 3;
    let mut image = vec![255_u8; 3 * cote * cote];
    for py in 0..cote {
        for px in 0..cote {
            let mut somme = [0_u64; 3];
            for j in 0..sur {
                for i in 0..sur {
                    let x = (2.0 * (px * sur + i) as f64 + 1.0 - n as f64) / (2.0 * n as f64) * large;
                    let y = (2.0 * (py * sur + j) as f64 + 1.0 - n as f64) / (2.0 * n as f64) * large;
                    let mut meilleur = (f64::MAX, (0, 0));
                    let mut second = f64::MAX;
                    for q in -marge..=marge {
                        for r in -marge..=marge {
                            let (cx, cy) = centre(q, r);
                            let d = ((x - cx).powi(2) + (y - cy).powi(2)).sqrt();
                            if d < meilleur.0 {
                                second = meilleur.0;
                                meilleur = (d, (q, r));
                            } else if d < second {
                                second = d;
                            }
                        }
                    }
                    assert!(
                        second - meilleur.0 > 1e-6,
                        "l'échantillon ({px}, {py}, {i}, {j}) est sur une frontière de case : cas mal choisi"
                    );
                    let couleur = match ordre.iter().position(|&c| c == meilleur.1) {
                        Some(k) => couleur_naive(valeurs[k], plein),
                        None => BLANC,
                    };
                    for c in 0..3 {
                        somme[c] += u64::from(couleur[c]);
                    }
                }
            }
            let s = (sur * sur) as u64;
            for c in 0..3 {
                image[3 * (py * cote + px) + c] = ((somme[c] + s / 2) / s) as u8;
            }
        }
    }
    image
}

/// Le dégradé de la doc, réécrit ici (valeurs et formules de la doc, sans appeler `rendu_hex`).
fn couleur_naive(valeur: i64, plein: i64) -> [u8; 3] {
    let arrets: [[i128; 3]; 5] = [[255, 255, 255], [198, 219, 239], [107, 174, 214], [33, 113, 181], [8, 48, 107]];
    let v = i128::from(valeur.max(0).min(plein));
    let t = v * 1024 / i128::from(plein);
    if t >= 1024 {
        return [8, 48, 107];
    }
    let (s, f) = ((t / 256) as usize, t % 256);
    let mut c = [0_u8; 3];
    for k in 0..3 {
        c[k] = ((arrets[s][k] * (256 - f) + arrets[s + 1][k] * f + 128) >> 8) as u8;
    }
    c
}

fn reglages(cote: u32, rayon_vue: u32, sur: u32, plein: i64) -> ReglagesImage {
    ReglagesImage { cote, rayon_vue, sur, plein }
}

/// juste : un hexagone isolé, calculé à la main (V-G1-RENDU-01) : `R = 1`, une seule case `(0, 0)` à la valeur pleine, `cote = 12`, `rayon_vue = 1` (la vue fait
/// 3 cases de large, donc 4 pixels par case), `sur = 1` (le centre du pixel). L'hexagone à plat en haut a un côté à `1/2` case = 2 pixels de son centre : les centres
/// de pixels `(xc, yc)` (relatifs au centre de l'image) qui sont dedans vérifient `|yc| <= 2` et `(√3/2)|xc| + |yc|/2 <= 2`. Les lignes `yc = ±1,5` ont `|xc| <= 1,44` :
/// `xc = ±0,5` (2 pixels par ligne) ; les lignes `yc = ±0,5` ont `|xc| <= 2,02` : `xc = ±0,5` et `±1,5` (4 pixels) : au total 12 pixels du bleu foncé `(8, 48, 107)`, aux lignes
/// 4 et 7 (colonnes 5 et 6) et 5 et 6 (colonnes 4 à 7) ; tout le reste est blanc exact. Un hexagone décalé d'un demi-côté, tourné de 30° (pointe en haut) ou plus
/// petit ou plus grand, une image dont le haut est le bas, un fond noir échouent ici.
///
/// Ne couvre pas : la couverture partielle d'un pixel (voir `couverture_et_champs_contre_le_naif`).
#[test]
#[cfg_attr(not(feature = "etape-g1"), ignore)]
fn hexagone_isole_a_la_main() {
    let plein = 1 << 32;
    let image = rendre(1, &champ_une_case(1, 0, 0, plein), &reglages(12, 1, 1, plein)).expect("rendu valide");
    assert_eq!(image.len(), 3 * 12 * 12);
    let couverts: Vec<(usize, usize)> =
        vec![(5, 4), (6, 4), (4, 5), (5, 5), (6, 5), (7, 5), (4, 6), (5, 6), (6, 6), (7, 6), (5, 7), (6, 7)];
    for y in 0..12 {
        for x in 0..12 {
            let attendu = if couverts.contains(&(x, y)) { FONCE } else { BLANC };
            assert_eq!(pixel(&image, 12, x, y), attendu, "pixel ({x}, {y})");
        }
    }
}

/// juste : le dessin d'un champ comparé à un code NAÏF qui ne partage ni la fraction rationnelle ni l'arrondi cubique de la doc (V-G1-RENDU-02) : chaque échantillon
/// prend la case dont le centre est le plus proche (le vrai `√3`, `f64`), puis la couleur de son dégradé (réécrit ici), puis la moyenne entière. Des cases isolées à des
/// endroits variés (le centre, droit vers le haut `(1, 0)`, en haut à gauche `(0, 1)`, en bas à gauche `(-1, 1)`, loin `(2, -1)`, une case du mur), des champs
/// tirés au hasard (graines 1 à 4), des tailles de 12 à 42 pixels, `sur` de 1 à 4, `rayon_vue` de 1 à 4 : chaque octet est celui du naïf. Cela vérifie ensemble la
/// position des centres (`X = -(√3/2) r`, `Y = -(q + r/2)` : le bras `(1, 0)` est vers le haut), la taille et l'orientation des hexagones, l'ordre des lignes, la
/// couverture des bords en `sur * sur` niveaux, la moyenne, le blanc hors de l'hexagone de rayon `R`.
///
/// Ne couvre pas : les grandes images (le coût) ; un champ de grand rayon.
#[test]
#[cfg_attr(not(feature = "etape-g1"), ignore)]
fn couverture_et_champs_contre_le_naif() {
    let plein = 3_i64 << 30;
    type Cas = (i32, (i32, i32), usize, i32, usize);
    let cas: Vec<Cas> = vec![
        (1, (0, 0), 24, 1, 4),
        (2, (1, 0), 30, 2, 3),
        (2, (0, 1), 30, 2, 3),
        (2, (-1, 1), 30, 2, 2),
        (3, (2, -1), 42, 3, 2),
        (2, (2, 0), 30, 2, 4),
        (2, (-2, 2), 20, 2, 1),
        (1, (0, 0), 12, 1, 3),
    ];
    for (rayon, (q, r), cote, vue, sur) in cas {
        let valeurs = champ_une_case(rayon, q, r, plein);
        let obtenu = rendre(rayon as u32, &valeurs, &reglages(cote as u32, vue as u32, sur as u32, plein))
            .expect("rendu valide");
        let attendu = naif(rayon, &valeurs, cote, vue, sur, plein);
        assert_eq!(obtenu, attendu, "case ({q}, {r}) R={rayon} cote={cote} vue={vue} sur={sur}");
        assert!(obtenu.iter().any(|&o| o != 255), "l'image n'est pas blanche");
    }
    for graine in 1..=4_u64 {
        let rayon = 2 + (graine % 2) as i32;
        let mut g = Lcg(graine * 7919);
        let valeurs: Vec<i64> = cases(rayon)
            .iter()
            .map(|_| if g.jusqu_a(4) == 0 { 0 } else { (g.jusqu_a(plein as u64 + 1)) as i64 })
            .collect();
        let (cote, vue, sur) = (36, rayon + 1, 2 + (graine % 2) as usize);
        let obtenu = rendre(rayon as u32, &valeurs, &reglages(cote as u32, vue as u32, sur as u32, plein))
            .expect("rendu valide");
        assert_eq!(obtenu, naif(rayon, &valeurs, cote, vue, sur, plein), "champ au hasard, graine {graine}");
    }
}

/// juste : le dégradé (V-G1-RENDU-03) : la valeur 0 est exactement `(255, 255, 255)` ; toute valeur `>= plein` est exactement `(8, 48, 107)` (y compris `i64::MAX`, sans
/// débordement) ; les cinq arrêts `t = 0, 256, 512, 768, 1024` sont ceux de la doc (`ARRETS`) ; `t = min(v, plein) * 1024 / plein` (division entière) ; entre deux arrêts, l'interpolation
/// entière `(a (256 - f) + b f + 128) >> 8` : `t = 128` donne `(227, 237, 247)`, et `plein = 3`, `valeur = 1` (`t = 341`) donne `(168, 204, 231)` ; chaque canal est monotone
/// (il ne croît jamais quand la valeur croît) sur 1000 valeurs ; une valeur négative donne le blanc, un `plein` inférieur à 1 est lu comme 1. Un dégradé inversé, un fond ou
/// un bleu foncé autres, un arrondi différent échouent ici.
///
/// Ne couvre pas : l'esthétique (que ces bleus soient beaux) ni la lisibilité d'un champ dans cette échelle.
#[test]
#[cfg_attr(not(feature = "etape-g1"), ignore)]
fn degrade_de_bleus() {
    assert_eq!(ARRETS, [[255, 255, 255], [198, 219, 239], [107, 174, 214], [33, 113, 181], [8, 48, 107]]);
    let plein = 1_i64 << 30;
    assert_eq!(couleur_du_degrade(0, plein), BLANC);
    assert_eq!(couleur_du_degrade(plein, plein), FONCE);
    assert_eq!(couleur_du_degrade(plein + 1, plein), FONCE);
    assert_eq!(couleur_du_degrade(i64::MAX, plein), FONCE);
    assert_eq!(couleur_du_degrade(i64::MAX, i64::MAX), FONCE);
    assert_eq!(couleur_du_degrade(0, i64::MAX), BLANC);
    for (t, arret) in [(0_i64, 0_usize), (256, 1), (512, 2), (768, 3), (1024, 4)] {
        assert_eq!(couleur_du_degrade(t * (plein / 1024), plein), ARRETS[arret], "t = {t}");
    }
    assert_eq!(couleur_du_degrade(128 * (plein / 1024), plein), [227, 237, 247]);
    assert_eq!(couleur_du_degrade(1, 3), [168, 204, 231]);
    // l'interpolation, partout, contre la formule réécrite
    for k in 0..=1024_i64 {
        let valeur = k * (plein / 1024);
        assert_eq!(couleur_du_degrade(valeur, plein), couleur_naive(valeur, plein), "t = {k}");
    }
    let mut avant = couleur_du_degrade(0, 1_000_000);
    for v in 1..=1_000_000_i64 {
        if v % 1000 != 0 {
            continue;
        }
        let c = couleur_du_degrade(v, 1_000_000);
        assert!(
            c[0] <= avant[0] && c[1] <= avant[1] && c[2] <= avant[2],
            "le dégradé s'inverse en {v} : {avant:?} puis {c:?}"
        );
        avant = c;
    }
    assert_eq!(couleur_du_degrade(-5, plein), BLANC);
    assert_eq!(couleur_du_degrade(i64::MIN, plein), BLANC);
    assert_eq!(couleur_du_degrade(3, 0), FONCE);
    assert_eq!(couleur_du_degrade(0, -4), BLANC);
}

/// juste : la monotonie dans l'image (V-G1-RENDU-04) : un hexagone isolé dont la valeur croît de 0 à `plein` donne un pixel central de plus en plus foncé (chaque canal ne croît
/// jamais), blanc exact pour la valeur 0 (toute l'image blanche), le bleu foncé de la doc pour `plein`.
///
/// Ne couvre pas : la couleur de deux cases voisines de valeurs différentes (voir `couverture_et_champs_contre_le_naif`).
#[test]
#[cfg_attr(not(feature = "etape-g1"), ignore)]
fn la_couleur_suit_la_valeur() {
    let plein = 1_i64 << 30;
    let reglage = reglages(16, 1, 2, plein);
    let blanche = rendre(1, &champ_une_case(1, 0, 0, 0), &reglage).expect("rendu valide");
    assert!(blanche.iter().all(|&o| o == 255), "valeur 0 : tout blanc");
    let mut avant = BLANC;
    for k in 1..=16 {
        let image = rendre(1, &champ_une_case(1, 0, 0, plein / 16 * k), &reglage).expect("rendu valide");
        let c = pixel(&image, 16, 8, 8);
        assert!(c[0] <= avant[0] && c[1] <= avant[1] && c[2] <= avant[2], "k = {k} : {avant:?} puis {c:?}");
        avant = c;
    }
    assert_eq!(avant, FONCE);
}

/// juste : la symétrie de l'image (V-G1-RENDU-05) : pour un champ symétrique (une fonction de `|q|, |r|, |s|` : invariante par les douze symétries de l'hexagone), `R = 7`,
/// `cote = 64`, `rayon_vue = 8`, `sur = 3`, l'image est identique à son miroir gauche-droite (`X -> -X` : les colonnes à l'envers), à son miroir haut-bas (`Y -> -Y` : les
/// lignes à l'envers) et à sa rotation de 180°, octet pour octet : la doc le garantit (`N` pair, aucun échantillon sur l'axe, aucune égalité). La rotation de 60° n'est pas
/// une symétrie d'une grille de pixels carrée : elle n'est pas testée. Un échantillonnage décalé d'un demi-pixel, un arrondi asymétrique échouent ici.
///
/// Ne couvre pas : la symétrie d'un champ qui n'est pas symétrique (aucune n'est due).
#[test]
#[cfg_attr(not(feature = "etape-g1"), ignore)]
fn l_image_d_un_champ_symetrique_est_symetrique() {
    let plein = 1_i64 << 30;
    let rayon = 7;
    let valeurs: Vec<i64> = cases(rayon)
        .iter()
        .map(|&(q, r)| {
            let (a, b, c) = (i64::from(q.abs()), i64::from(r.abs()), i64::from((q + r).abs()));
            ((a * b * c + a + b + c) % 11) * (plein / 10)
        })
        .collect();
    let cote = 64_usize;
    let image = rendre(rayon as u32, &valeurs, &reglages(cote as u32, 8, 3, plein)).expect("rendu valide");
    let mut distincts = std::collections::BTreeSet::new();
    for y in 0..cote {
        for x in 0..cote {
            let p = pixel(&image, cote, x, y);
            distincts.insert(p);
            assert_eq!(p, pixel(&image, cote, cote - 1 - x, y), "miroir gauche-droite en ({x}, {y})");
            assert_eq!(p, pixel(&image, cote, x, cote - 1 - y), "miroir haut-bas en ({x}, {y})");
            assert_eq!(p, pixel(&image, cote, cote - 1 - x, cote - 1 - y), "rotation de 180° en ({x}, {y})");
        }
    }
    assert!(distincts.len() > 8, "l'image doit montrer plusieurs couleurs : {}", distincts.len());
}

/// juste : les refus (V-G1-RENDU-06), dans l'ordre de la doc : un rayon de 0 ou de `RAYON_MAX + 1` : `RayonHorsBorne` ; puis le premier réglage hors plage
/// (`cote` impair, 0, 1 ou `> 8192` ; `rayon_vue` 0 ou `> 2047` ; `sur` 0 ou 17 ; `plein` 0 ou négatif) avec son nom ; puis `NombreDeValeurs` ; puis `ValeurNegative(i)` pour la première
/// valeur négative ; les bornes (`cote = 2`, `8192` n'est pas rendu ici, `sur = 16`, `rayon_vue = 1`, `plein = 1`) sont permises ; jamais de panique.
///
/// Ne couvre pas : les très grandes images (le coût).
#[test]
#[cfg_attr(not(feature = "etape-g1"), ignore)]
fn refus_du_rendu() {
    let bon = reglages(8, 1, 1, 1 << 32);
    let valeurs = champ_une_case(1, 0, 0, 1 << 32);
    assert!(rendre(1, &valeurs, &bon).is_ok());
    assert_eq!(rendre(0, &valeurs, &bon), Err(ErreurRendu::RayonHorsBorne));
    assert_eq!(rendre(2048, &valeurs, &bon), Err(ErreurRendu::RayonHorsBorne));
    for cote in [0, 1, 3, 9, 8193, 8194, u32::MAX] {
        assert_eq!(
            rendre(1, &valeurs, &reglages(cote, 1, 1, 1)),
            Err(ErreurRendu::ReglagesInvalides("cote")),
            "cote = {cote}"
        );
    }
    for vue in [0, 2048, u32::MAX] {
        assert_eq!(
            rendre(1, &valeurs, &reglages(8, vue, 1, 1)),
            Err(ErreurRendu::ReglagesInvalides("rayon_vue")),
            "rayon_vue = {vue}"
        );
    }
    for sur in [0, 17, u32::MAX] {
        assert_eq!(
            rendre(1, &valeurs, &reglages(8, 1, sur, 1)),
            Err(ErreurRendu::ReglagesInvalides("sur")),
            "sur = {sur}"
        );
    }
    for plein in [0, -1, i64::MIN] {
        assert_eq!(
            rendre(1, &valeurs, &reglages(8, 1, 1, plein)),
            Err(ErreurRendu::ReglagesInvalides("plein")),
            "plein = {plein}"
        );
    }
    // l'ordre : le premier réglage fautif, avant le nombre de valeurs
    assert_eq!(rendre(1, &valeurs[1..], &reglages(7, 0, 0, 0)), Err(ErreurRendu::ReglagesInvalides("cote")));
    assert_eq!(rendre(1, &valeurs[1..], &reglages(8, 0, 0, 0)), Err(ErreurRendu::ReglagesInvalides("rayon_vue")));
    assert_eq!(rendre(1, &valeurs[1..], &reglages(8, 1, 0, 0)), Err(ErreurRendu::ReglagesInvalides("sur")));
    assert_eq!(rendre(1, &valeurs[1..], &reglages(8, 1, 1, 0)), Err(ErreurRendu::ReglagesInvalides("plein")));
    assert_eq!(rendre(1, &valeurs[1..], &bon), Err(ErreurRendu::NombreDeValeurs));
    let mut trop = valeurs.clone();
    trop.push(0);
    assert_eq!(rendre(1, &trop, &bon), Err(ErreurRendu::NombreDeValeurs));
    let mut negatives = valeurs.clone();
    negatives[5] = -1;
    negatives[2] = -9;
    assert_eq!(rendre(1, &negatives, &bon), Err(ErreurRendu::ValeurNegative(2)));
    // les bornes permises
    assert!(rendre(1, &valeurs, &reglages(2, 1, 16, 1)).is_ok());
    assert_eq!(rendre(1, &valeurs, &reglages(2, 1, 1, 1)).map(|i| i.len()), Ok(12));
}

/// régression : l'empreinte `sha256` des octets de l'image d'un cristal de référence (V-G1-RENDU-07) : `Neige::essayer_new(20, FIG9_RHO_050)` après 400 pas, la masse
/// cristalline `c` des cases rattachées (`valeurs_du_cristal`), `cote = 96`, `rayon_vue = 20`, `sur = 4`, `plein = 3 * 2^31` (1,5). La valeur a été calculée par le code
/// Python INDÉPENDANT (`g1_ref.py` : la simulation de la doc en entiers, puis le rendu de la doc en entiers, d'après la doc seule, `hashlib.sha256`) : c'est un résultat de
/// NOTRE code figé, et aussi celui d'un deuxième code. Le cristal est SYMÉTRIQUE (donc l'image l'est : miroirs et rotation de 180°) et son rayon est celui attendu.
///
/// Ne couvre pas : que l'image soit belle ou ressemble à une figure ; seul le hachage est une preuve de stabilité.
#[test]
#[cfg_attr(not(feature = "etape-g1"), ignore)]
fn image_de_reference() {
    let mut neige = Neige::essayer_new(20, p(FIG9_RHO_050)).expect("neige valide");
    neige.avancer(400);
    let champ = neige.valeurs_du_cristal();
    let image = rendre(20, &champ, &reglages(96, 20, 4, 3_i64 << 31)).expect("rendu valide");
    assert_eq!(image.len(), 3 * 96 * 96);
    assert_eq!(sha256_hex(&image), etape_g1_commun::valeurs::EMPREINTE_IMAGE_REFERENCE);
    // le cristal de référence : mesuré par Python
    assert_eq!(neige.cases_rattachees(), etape_g1_commun::valeurs::IMAGE_REFERENCE_RATTACHEES);
    let cote = 96;
    for y in 0..cote {
        for x in 0..cote {
            assert_eq!(pixel(&image, cote, x, y), pixel(&image, cote, cote - 1 - x, y));
            assert_eq!(pixel(&image, cote, x, y), pixel(&image, cote, x, cote - 1 - y));
        }
    }
    // une image qui montre du cristal et du fond
    let blancs = image.chunks(3).filter(|p| p == &BLANC).count();
    assert!(blancs > 96 * 96 / 2 && blancs < 96 * 96, "fond blanc : {blancs} pixels");
    assert!(image.chunks(3).any(|p| p[2] < 200), "du bleu franc");
}

/// juste : la taille de la sortie et le suréchantillonnage (V-G1-RENDU-08) : `3 * cote * cote` octets ; avec le MÊME champ, `sur = 1` et `sur = 4` ne donnent pas la même image
/// (le suréchantillonnage lisse les bords : il apparaît des couleurs intermédiaires), et à `sur = 4` le nombre de couleurs distinctes d'un hexagone isolé est d'au moins 5 et d'au plus
/// 17 (des pixels de bord à couverture partielle : au plus `sur * sur + 1` niveaux), alors qu'avec `sur = 1` il n'y en a que deux (blanc et bleu foncé). Une couverture
/// comptée deux fois (les pixels de bord déjà sombres) ou un agrandissement au plus proche voisin échouent ici.
///
/// Ne couvre pas : le temps de calcul (mesuré dans `MODELE.md`).
#[test]
#[cfg_attr(not(feature = "etape-g1"), ignore)]
fn le_surechantillonnage_lisse_les_bords() {
    let plein = 1_i64 << 32;
    let champ = champ_une_case(2, 0, 0, plein);
    let net = rendre(2, &champ, &reglages(40, 2, 4, plein)).expect("rendu valide");
    let brut = rendre(2, &champ, &reglages(40, 2, 1, plein)).expect("rendu valide");
    assert_eq!(net.len(), 3 * 40 * 40);
    assert_eq!(brut.len(), 3 * 40 * 40);
    let couleurs =
        |image: &[u8]| image.chunks(3).map(|p| [p[0], p[1], p[2]]).collect::<std::collections::BTreeSet<_>>();
    assert_eq!(couleurs(&brut), [BLANC, FONCE].into_iter().collect());
    let nets = couleurs(&net);
    assert!(nets.len() >= 5 && nets.len() <= 17, "{} couleurs", nets.len());
    // les pixels de bord sont strictement entre le blanc et le foncé, jamais plus foncés que le foncé
    for c in &nets {
        assert!(c[0] >= 8 && c[1] >= 48 && c[2] >= 107);
    }
}

/// juste : la fraction rationnelle `A / B` de la doc, à l'unité près (V-G1-RENDU-09, ajouté par le juge) : deux échantillons choisis parce qu'ils tombent à moins de `5e-10` case d'une
/// frontière, calculés à la main avec les entiers de la doc. (1) `R = 202`, `cote = 844`, `rayon_vue = 311`, `sur = 1`, pixel `(658, 421)` : `xs = 473`, `ys = -1`, `nx = 473 * 623 = 294679`,
/// `ny = -623` ; la frontière entre les cases `(101, -202)` et `(101, -201)` est `r - s = -302`, et le point est du côté de `(101, -202)` si et seulement si `3 nx A + ny B > 302 M`, soit
/// `884037 A > 510399 B` : vrai pour `A = 1239850262` (l'écart est de `122541` sur `1,1e15`), faux pour `A - 1` et faux aussi pour le vrai `1/√3` : seule la fraction de la doc donne
/// `(101, -202)`. (2) `R = 39`, `cote = 772`, `rayon_vue = 254`, `sur = 1`, pixel `(436, 385)` : `xs = 101`, `ys = -1`, `nx = 51409`, `ny = -509` ; la frontière entre `(19, -38)` et `(20, -39)` est
/// `q - r = 58`, et le point est du côté de `(19, -38)` si et seulement si `154227 A < 89043 B` : vrai pour `A` (écart `22347`), faux pour `A + 1`. Les deux inégalités sont vérifiées ici en `i128`,
/// puis l'image : le pixel est du bleu foncé quand la case de la doc porte la valeur pleine, blanc quand c'est l'autre case. Un numérateur faux d'une unité, un `√3` calculé en flottant, une
/// autre fraction échouent ici (les autres tests ne voient pas un écart de `1e-9` case).
///
/// Ne couvre pas : les autres échantillons des grandes images (aucun test ne rend une image de plus de 844 pixels de côté).
#[test]
#[cfg_attr(not(feature = "etape-g1"), ignore)]
fn la_fraction_de_la_doc_a_l_unite_pres() {
    let (a, b): (i128, i128) = (1_239_850_262, 2_147_483_647);
    assert!(884_037 * a > 510_399 * b && 884_037 * (a - 1) < 510_399 * b);
    assert!(154_227 * a < 89_043 * b && 154_227 * (a + 1) > 89_043 * b);
    let plein = 1 << 32;
    // (rayon, cote, rayon_vue, le pixel, la case de la doc, l'autre case)
    let cas = [
        (202_i32, 844_u32, 311_u32, (658_usize, 421_usize), (101_i32, -202_i32), (101_i32, -201_i32)),
        (39, 772, 254, (436, 385), (19, -38), (20, -39)),
    ];
    for (rayon, cote, rayon_vue, (x, y), doc, autre) in cas {
        let r = reglages(cote, rayon_vue, 1, plein);
        let image = rendre(rayon as u32, &champ_une_case(rayon, doc.0, doc.1, plein), &r).expect("rendu valide");
        assert_eq!(
            pixel(&image, cote as usize, x, y),
            FONCE,
            "R={rayon} : le pixel ({x}, {y}) est dans la case {doc:?}"
        );
        let image = rendre(rayon as u32, &champ_une_case(rayon, autre.0, autre.1, plein), &r).expect("rendu valide");
        assert_eq!(
            pixel(&image, cote as usize, x, y),
            BLANC,
            "R={rayon} : le pixel ({x}, {y}) n'est pas dans la case {autre:?}"
        );
    }
}

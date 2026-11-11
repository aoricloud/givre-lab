//! Le rendu net d'un champ sur la grille hexagonale (étape G1) : chaque case est dessinée comme un **hexagone à sa vraie géométrie**, avec une
//! **couleur continue** tirée d'une valeur de la case (un dégradé de bleus, clair puis foncé, sur fond blanc), calculée à **haute résolution
//! par suréchantillonnage** et **en entiers** : les octets sont les mêmes sous Linux et sous Windows. Ce module ne sait rien du modèle de
//! neige : il prend, pour chaque case de l'hexagone, un entier (par exemple [`crate::neige::Neige::valeurs_du_cristal`]) et rend les octets RVB que
//! [`crate::image::ecrire_png`] écrit. Il remplace, pour les images, « un pixel par case puis un agrandissement » ([`crate::rendu`], les aperçus
//! d'avant) : ici la case est dessinée pour ce qu'elle est, et aucun pixel n'est jamais un agrandissement au plus proche voisin.
//!
//! # La géométrie (une case = un hexagone à plat en haut, un bras vers le haut)
//!
//! L'écart entre deux cases voisines vaut **1** (l'unité de la vue). Les cases sont les hexagones de Voronoï du réseau triangulaire : chaque
//! hexagone a son centre à la case, ses six côtés à la **distance 1/2** de son centre, perpendiculaires aux six directions de voisinage, et son
//! rayon extérieur (centre-sommet) vaut `1/√3`. Les côtés haut et bas sont horizontaux (« à plat en haut »), les sommets sont à gauche et à droite.
//! La direction 0 de [`crate::reseau::DIRECTIONS_HEX`], `(1, 0)`, est vers le **haut** : un cristal de neige a donc un bras vertical, comme les figures de
//! l'article de Gravner et Griffeath. Le centre de la case `(q, r)`, dans l'image (`X` vers la droite, `Y` vers le **bas**, l'origine au centre de
//! l'image, en unités de case), est
//!
//! ```text
//! X = -(√3 / 2) * r          Y = -(q + r / 2)
//! ```
//!
//! (la rotation de 90° de la position usuelle `x = q + r/2, y = r √3/2`). `√3` n'est jamais calculé : on lit l'**inverse rationnel** `w = A / B` avec
//! `A = 1 239 850 262` et `B = 2 147 483 647` (`B = 2^31 - 1`, premier), qui donne `w = 0,57735026934…` pour `1/√3 = 0,57735026918…` (écart relatif 2,6e-10 :
//! invisible, mais il rend tout le calcul exact en entiers).
//!
//! # L'image : pixels, échantillons
//!
//! [`ReglagesImage`] donne le **côté** de l'image `cote` en pixels (une image carrée, `cote` pair, de 2 à 8192), le **rayon de la vue** `rayon_vue` `V`
//! (de 1 à [`crate::neige::RAYON_MAX`]) et le **suréchantillonnage** `sur` (de 1 à 16 : `sur × sur` échantillons par pixel) et la valeur **pleine**
//! `plein` (`>= 1`, voir « Le dégradé »). L'image montre le carré de côté `2 V + 1` cases centré sur la case `(0, 0)` : de `-(V + 1/2)` à `V + 1/2` en
//! `X` comme en `Y`. (Pour `V >= 1` ce carré contient l'hexagone de rayon `V` ; il montre aussi des cases plus loin, vers la gauche et la droite, si `R` est plus grand.) Un
//! point hors de l'hexagone de rayon `R` est du fond : **blanc**.
//!
//! Pour le pixel `(px, py)` (colonne, ligne, de haut en bas, à partir de 0) et l'échantillon `(i, j)` (`0 <= i, j < sur`), avec `N = cote * sur`, `a = px * sur + i`
//! et `b = py * sur + j`, le **point échantillonné** est le centre de la sous-case : `X = xs (2 V + 1) / (2 N)`, `Y = ys (2 V + 1) / (2 N)` avec
//! `xs = 2 a + 1 - N` et `ys = 2 b + 1 - N` (des entiers impairs : `N` est pair, donc aucun échantillon n'est sur l'axe `X = 0`).
//!
//! # Quelle case contient un point (tout en entiers)
//!
//! Avec `nx = xs (2 V + 1)`, `ny = ys (2 V + 1)` et `M = 2 N B`, les coordonnées axiales fractionnaires du point sont `q = -Y + X w`, `r = -2 X w`,
//! `s = -q - r = Y + X w`, soit **exactement** les entiers sur `M` :
//!
//! ```text
//! Qn = -ny * B + nx * A        Rn = -2 * nx * A        Sn = ny * B + nx * A        (q = Qn / M, r = Rn / M, s = Sn / M)
//! ```
//!
//! La case est trouvée par l'arrondi cubique habituel : `arrondi(Z) = floor((2 Z + M) / (2 M))` (au plus proche, demi-cas vers le haut) pour chacun de
//! `Qn, Rn, Sn`, d'où `(rq, rr, rs)` ; puis `dq = |rq M - Qn|`, `dr = |rr M - Rn|`, `ds = |rs M - Sn|` ; si `dq > dr` et `dq > ds` alors `rq = -rr - rs` ;
//! sinon si `dr > ds` alors `rr = -rq - rs` ; la case est `(rq, rr)`. Aucun échantillon n'est jamais à égalité (une égalité demanderait `X = 0`, car `B` est
//! premier et ne divise aucun dénominateur de `Y`) : la règle des cas d'égalité ci-dessus ne sert pas, et **l'image d'un état symétrique l'est aussi** : le
//! miroir `X -> -X` (les colonnes lues à l'envers), le miroir `Y -> -Y` (les lignes à l'envers) et la rotation de 180° donnent la même image, octet pour octet.
//! (La rotation de 60° ne se garde pas : la grille de pixels est carrée.)
//!
//! # Le dégradé : valeur -> couleur, en entiers
//!
//! `valeur` (un `i64 >= 0`) et `plein` (un `i64 >= 1`) donnent `t = min(valeur, plein) * 1024 / plein` (division entière, en `i128`), de 0 à 1024. La couleur est
//! la interpolation linéaire par canal entre cinq arrêts, `t = 0, 256, 512, 768, 1024` :
//!
//! ```text
//! t = 0     (255, 255, 255)   blanc : la case sans glace
//! t = 256   (198, 219, 239)
//! t = 512   (107, 174, 214)
//! t = 768   ( 33, 113, 181)
//! t = 1024  (  8,  48, 107)   le bleu foncé de la valeur pleine
//! ```
//!
//! Pour `t < 1024` : `s = t / 256`, `f = t % 256`, `canal = (arret_s * (256 - f) + arret_(s+1) * f + 128) >> 8` ; pour `t = 1024` le dernier arrêt. Chaque canal
//! décroît avec la valeur (le dégradé ne s'inverse jamais) ; la valeur 0 est exactement `(255, 255, 255)` et toute valeur `>= plein` est exactement `(8, 48, 107)`.
//! [`couleur_du_degrade`] est cette correspondance.
//!
//! # Le pixel
//!
//! La couleur d'un échantillon est celle de sa case (le dégradé de sa valeur), ou le blanc `(255, 255, 255)` si le point n'est dans aucune case de
//! l'hexagone de rayon `R`. La couleur du pixel est, **pour chaque canal**, la **moyenne** des `sur * sur` échantillons : `(somme + (sur * sur) / 2) / (sur * sur)`
//! (division entière). C'est ce qui lisse le bord d'une case (la couverture du pixel, en `sur * sur` niveaux) et le raccord entre deux couleurs.
//!
//! # Les bornes et les refus
//!
//! [`rendre`] ne panique jamais : `Err(ErreurRendu::RayonHorsBorne)` si `rayon` est 0 ou plus grand que [`crate::neige::RAYON_MAX`] ;
//! `Err(ErreurRendu::ReglagesInvalides(nom))` pour le **premier** réglage hors plage dans l'ordre `cote`, `rayon_vue`, `sur`, `plein` ;
//! `Err(ErreurRendu::NombreDeValeurs)` si `valeurs.len() != 3 R (R + 1) + 1` ; `Err(ErreurRendu::ValeurNegative(i))` pour la première valeur négative (l'indice canonique).
//! Toutes les opérations tiennent dans un `i64` : `|nx|, |ny| <= 131072 * 4095`, et `M * (2 V + 1) < 2^63`.
//!
//! # Ce que ce rendu n'est pas
//!
//! Il ne lisse pas les valeurs entre deux cases (chaque case est d'une couleur : le lissage est celui des bords, pas une interpolation du champ) ; il ne
//! dessine ni légende ni axe ; il n'est qu'une image, **jamais une preuve** (`AGENTS.md`, règle 3). Il n'est pas dans la page (le canvas de la page dessine encore une
//! case par pixel : c'est l'étape G3).

/// Les cinq arrêts du dégradé (voir « Le dégradé »).
pub const ARRETS: [[u8; 3]; 5] = [[255, 255, 255], [198, 219, 239], [107, 174, 214], [33, 113, 181], [8, 48, 107]];

/// Les réglages d'une image (voir « L'image : pixels, échantillons »).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ReglagesImage {
    /// Le côté de l'image en pixels : pair, de 2 à 8192.
    pub cote: u32,
    /// Le rayon de la vue, en cases : de 1 à `RAYON_MAX`.
    pub rayon_vue: u32,
    /// Le suréchantillonnage : `sur * sur` échantillons par pixel, de 1 à 16.
    pub sur: u32,
    /// La valeur qui donne le bleu le plus foncé : au moins 1.
    pub plein: i64,
}

/// Pourquoi un rendu a été refusé.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ErreurRendu {
    /// Le rayon de l'hexagone est 0 ou dépasse `RAYON_MAX`.
    RayonHorsBorne,
    /// Un réglage d'image est hors de sa plage : son nom (`"cote"`, `"rayon_vue"`, `"sur"` ou `"plein"`), le premier dans cet ordre.
    ReglagesInvalides(&'static str),
    /// Le nombre de valeurs n'est pas `3 R (R + 1) + 1`.
    NombreDeValeurs,
    /// La valeur d'indice canonique donné est négative (la première).
    ValeurNegative(usize),
}

/// La couleur du dégradé pour `valeur` et `plein` (voir « Le dégradé ») : `t = min(valeur, plein) * 1024 / plein`, puis l'interpolation entière entre
/// les cinq [`ARRETS`]. Une `valeur` négative donne celle de 0 (blanc) ; un `plein` inférieur à 1 est lu comme 1 (jamais de panique).
pub fn couleur_du_degrade(valeur: i64, plein: i64) -> [u8; 3] {
    let plein = plein.max(1);
    let valeur = valeur.clamp(0, plein);
    let t = i128::from(valeur) * 1024 / i128::from(plein);
    if t >= 1024 {
        return ARRETS[4];
    }
    let s = (t / 256) as usize;
    let f = t % 256;
    let mut couleur = [0_u8; 3];
    for (c, sortie) in couleur.iter_mut().enumerate() {
        let a = i128::from(ARRETS[s][c]);
        let b = i128::from(ARRETS[s + 1][c]);
        *sortie = ((a * (256 - f) + b * f + 128) >> 8) as u8;
    }
    couleur
}

/// Le numérateur de l'inverse rationnel de `√3` (voir « La géométrie »).
const A: i64 = 1_239_850_262;
/// Le dénominateur de l'inverse rationnel de `√3`.
const B: i64 = 2_147_483_647;

/// Rend l'image RVB (`3 * cote * cote` octets, ligne par ligne de haut en bas) du champ `valeurs` (une valeur `>= 0` par case de l'hexagone de rayon `rayon`,
/// **dans l'ordre canonique** de [`crate::reseau::Hexagone`]) avec ces réglages (voir la documentation du module : la géométrie, les échantillons, la case d'un point, le
/// dégradé, le pixel).
///
/// # Erreurs
///
/// Jamais de panique : voir « Les bornes et les refus ».
pub fn rendre(rayon: u32, valeurs: &[i64], reglages: &ReglagesImage) -> Result<Vec<u8>, ErreurRendu> {
    use crate::neige::RAYON_MAX;
    use crate::reseau::Hexagone;

    if rayon == 0 || rayon > RAYON_MAX {
        return Err(ErreurRendu::RayonHorsBorne);
    }
    if reglages.cote < 2 || reglages.cote > 8192 || !reglages.cote.is_multiple_of(2) {
        return Err(ErreurRendu::ReglagesInvalides("cote"));
    }
    if reglages.rayon_vue < 1 || reglages.rayon_vue > RAYON_MAX {
        return Err(ErreurRendu::ReglagesInvalides("rayon_vue"));
    }
    if reglages.sur < 1 || reglages.sur > 16 {
        return Err(ErreurRendu::ReglagesInvalides("sur"));
    }
    if reglages.plein < 1 {
        return Err(ErreurRendu::ReglagesInvalides("plein"));
    }
    let hexagone = Hexagone::new(rayon);
    if valeurs.len() != hexagone.cases() {
        return Err(ErreurRendu::NombreDeValeurs);
    }
    if let Some(i) = valeurs.iter().position(|&v| v < 0) {
        return Err(ErreurRendu::ValeurNegative(i));
    }

    let couleurs: Vec<[u8; 3]> = valeurs.iter().map(|&v| couleur_du_degrade(v, reglages.plein)).collect();
    let cote = reglages.cote as usize;
    let sur = reglages.sur as usize;
    let n = i64::from(reglages.cote) * i64::from(reglages.sur);
    let large = 2 * i64::from(reglages.rayon_vue) + 1;
    let m = 2 * n * B;
    let rayon_i = i64::from(rayon);
    let carre = (sur * sur) as u64;
    let arrondi = |z: i64| (2 * z + m).div_euclid(2 * m);

    let mut image = vec![255_u8; 3 * cote * cote];
    for py in 0..cote {
        for px in 0..cote {
            let mut somme = [0_u64; 3];
            for j in 0..sur {
                let b = (py * sur + j) as i64;
                let ny = (2 * b + 1 - n) * large;
                for i in 0..sur {
                    let a = (px * sur + i) as i64;
                    let nx = (2 * a + 1 - n) * large;
                    let qn = -ny * B + nx * A;
                    let rn = -2 * nx * A;
                    let sn = ny * B + nx * A;
                    let mut rq = arrondi(qn);
                    let mut rr = arrondi(rn);
                    let rs = arrondi(sn);
                    let dq = (rq * m - qn).abs();
                    let dr = (rr * m - rn).abs();
                    let ds = (rs * m - sn).abs();
                    if dq > dr && dq > ds {
                        rq = -rr - rs;
                    } else if dr > ds {
                        rr = -rq - rs;
                    }
                    let rs_final = -rq - rr;
                    let dedans = rq.abs().max(rr.abs()).max(rs_final.abs()) <= rayon_i;
                    let couleur = if dedans { couleurs[hexagone.index(rq as i32, rr as i32)] } else { [255, 255, 255] };
                    for (c, total) in somme.iter_mut().enumerate() {
                        *total += u64::from(couleur[c]);
                    }
                }
            }
            let d = 3 * (py * cote + px);
            for (c, total) in somme.iter().enumerate() {
                image[d + c] = ((total + carre / 2) / carre) as u8;
            }
        }
    }
    Ok(image)
}

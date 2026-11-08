//! Le modèle cellulaire de Reiter pour les flocons de neige, sur la grille hexagonale, sans aucun hasard et en virgule
//! fixe (étape 3). La symétrie d'ordre 6 (et la symétrie miroir) du flocon est **exacte au bit près** : la somme des six
//! voisins est une somme d'entiers, indépendante de l'ordre.
//!
//! # La référence
//!
//! C. A. Reiter, « A local cellular model for snow crystal growth », *Chaos, Solitons & Fractals* 23 (2005) 1111-1119. Lu :
//! la préimpression de l'auteur (Lafayette College, `webbox.lafayette.edu/~reiterc/mvp/sfn/sfn_pp.pdf`), sections
//! « Model and Parameter Diagram » et « Variation on Diffusion », et sa page de matériaux auxiliaires
//! (`webbox.lafayette.edu/~reiterc/mvp/sfn/index.html`, tables β-γ des rayons 100, 200, 400), et le script J de l'auteur
//! (`.../sfn/sfn.html`, `sfn.ijs` : fonctions `dla` et `mk_dla`), qui fait ce qui suit (en flottants, avec un bord circulaire : voir « Ce que le modèle n'est pas »). Les règles ci-dessous en sont
//! la lecture : cellule glacée si sa valeur est au moins 1 ; cellules réceptives = glacées et leurs voisines ; une cellule
//! réceptive garde sa valeur et reçoit la constante γ ; les autres diffusent ; fond β ; bord maintenu à β ; poids de la
//! diffusion : centre `1 - α/2`, voisines `α/12` (α = 1 : centre 1/2, voisines 1/12, le modèle de l'article).
//!
//! # L'état
//!
//! Un [`Flocon`] vit sur l'hexagone de rayon `R` de [`crate::reseau::Hexagone`] (coordonnées axiales `(q, r)`, cases
//! numérotées dans l'ordre canonique de cette doc) et porte une **masse d'eau** [`Fixe`] (Q31.32 : [`crate::math::Fixe`])
//! par case, toujours `>= 0`. Il compte aussi son **temps** (le nombre de pas faits). Il n'y a ni graine ni hasard : le
//! même état de départ donne toujours le même état, sur toutes les machines.
//!
//! Les trois paramètres ([`Parametres`]) : `alpha` (la diffusion, `0 <= α <= 2`), `beta` (la vapeur ambiante,
//! `0 <= β < 1`) et `gamma` (la masse ajoutée par pas aux cellules réceptives, `0 <= γ <= 1`).
//!
//! L'**état initial** de [`Flocon::new`] : toutes les cases valent `β`, sauf la case centrale `(0, 0)` qui vaut 1
//! ([`Fixe::UN`]) : un germe de glace.
//!
//! # Les classes de cellules
//!
//! Pour un état donné (les masses), chaque case est dans **une seule** des trois classes ([`Etat`]) :
//! - **gelée** si sa masse est **au moins** 1 (`s >= Fixe::UN` : la masse exactement égale à 1 est gelée ; un `brut` de
//!   `UN - 1` ne l'est pas) ;
//! - **de bord** si elle n'est pas gelée et si au moins un de ses voisins est gelé. Les voisins sont les six cases
//!   `(q + dq, r + dr)` de [`crate::reseau::DIRECTIONS_HEX`] **qui sont dans l'hexagone** (une case de la couronne
//!   extérieure n'a que 3 ou 4 voisins ; un voisin hors de l'hexagone n'existe pas et ne compte pour rien) ;
//! - **non réceptive** sinon.
//!
//! Une cellule est **réceptive** si elle est gelée ou de bord. La classe se calcule pour **toutes** les cases, couronne
//! comprise, à partir des masses seules.
//!
//! # Un pas, à la lettre
//!
//! Soit `s` l'état. Un pas calcule tout le nouvel état `s'` **à partir de `s` seul** (mise à jour synchrone : aucune case
//! n'est lue après avoir été remplacée ; l'ordre de parcours des cases ne change donc rien) :
//!
//! 1. la classe de chaque case (voir plus haut) ;
//! 2. `u(x) = s(x)` si `x` est non réceptive, `0` sinon (la masse qui diffuse) ;
//! 3. pour chaque case `x` à distance hexagonale **strictement inférieure à `R`** du centre (ses six voisins sont alors
//!    tous dans l'hexagone), avec `Fixe` pour la virgule fixe :
//!    - `somme(x)` = la somme **exacte** des six `u(voisin)` (des entiers : les `brut` s'ajoutent, dans n'importe quel ordre
//!      ; l'inclusion de la couronne est voulue) ;
//!    - `moyenne(x) = Fixe(somme).div_entier(6)` (arrondi au plus proche, demi-cas vers `+∞`, comme [`Fixe::div_entier`]) ;
//!    - `ecart(x) = moyenne(x) - Fixe(u(x))` ;
//!    - `diffusion(x) = (alpha * ecart(x)).div_entier(2)` : d'abord le produit `Fixe * Fixe` (arrondi comme l'opérateur
//!      `*`), puis la division par 2 arrondie comme `div_entier` ; c'est le terme `(α/2)(ū - u)` du modèle ;
//!    - si `x` est non réceptive : `s'(x) = u(x) + diffusion(x)` ;
//!    - si `x` est réceptive (gelée ou de bord) : `s'(x) = s(x) + gamma + diffusion(x)` (elle garde sa masse, reçoit `γ`,
//!      et reçoit la diffusion de ses voisines non réceptives : pour elle `u(x) = 0`) ;
//! 4. pour chaque case à distance hexagonale **exactement `R`** (la **couronne** extérieure : les cases du bord de
//!    l'hexagone) : `s'(x) = beta`. C'est la condition au bord de Reiter (le fond est maintenu à β sur le bord) ; la
//!    couronne est remise à `β` à **chaque** pas, même si elle valait autre chose. Les masses de la couronne servent
//!    donc de valeur aux voisines avant d'être écrasées ;
//! 5. le temps augmente de 1.
//!
//! Toutes les additions sont exactes sur les `brut` ; seuls trois arrondis existent, dans l'ordre : la moyenne (`/6`), le
//! produit par `alpha`, la division par 2. Pour `0 <= α <= 2`, une masse ne devient jamais négative (le produit arrondi
//! `alpha * ecart` vaut au moins `-2 u`, donc `diffusion >= -u`). Le modèle conserve la masse, à l'arrondi près, hors la
//! constante `γ` (une par cellule réceptive et par pas) et le bord : la diffusion d'une cellule non réceptive donne à chacune
//! de ses six voisines `α/12` de sa masse et garde `1 - α/2`.
//!
//! Remarque de calcul (non imposée) : les `brut` des masses réelles restent très au-dessous de `2^60`, la somme des six
//! tient dans un `i64` ; [`Flocon::depuis_masses`] refuse d'ailleurs les masses de `2^60` ou plus. Un pas coûte quelques
//! dizaines de nanosecondes par case : un hexagone de rayon 100 sur 2000 pas tient en quelques secondes.
//!
//! # Les mesures
//!
//! - [`Flocon::gelees`] : le nombre de cases gelées (couronne comprise : une case de la couronne gelée par
//!   [`Flocon::depuis_masses`] compte jusqu'au prochain pas).
//! - [`Flocon::rayon_cristal`] : la plus grande distance hexagonale `distance_hex((0, 0), case)` d'une case gelée au centre
//!   ; 0 s'il n'y a aucune case gelée ou seulement le centre.
//! - [`Flocon::compacite`] : `gelées / cases de l'hexagone de rayon rayon_cristal`, où cet hexagone a `3 r (r + 1) + 1`
//!   cases (`r` = le rayon du cristal) : `Fixe::rapport(gelees, 3 r (r + 1) + 1)`. Elle vaut 1 pour un hexagone plein (une
//!   plaque), environ 0,2 pour une dendrite fine à rayon 50.
//! - [`Flocon::pointes`] : le nombre de cases gelées qui ont **exactement un** voisin gelé (les bouts de branches).
//! - [`Flocon::defaut_rotation`] et [`Flocon::defaut_reflexion`] : le **défaut de symétrie**, le nombre de cases `x`
//!   dont la masse diffère de celle de son image par la rotation de 60° ([`crate::reseau::tourner60`]) ou par la réflexion
//!   `(q, r) -> (r, q)` ([`reflechir`]). Un état à symétrie d'ordre 6 exacte, miroir compris, a ces deux défauts à 0 ; la
//!   rotation de 60° et la réflexion engendrent les 12 symétries de l'hexagone (le groupe diédral), donc deux défauts nuls
//!   disent la symétrie complète.
//!
//! # L'écriture canonique et l'empreinte
//!
//! L'empreinte de l'état ([`Flocon::empreinte`]) est le SHA-256 (`empreinte::sha256_hex`) de [`Flocon::ecriture_canonique`],
//! suite d'octets, dans cet ordre (tout entier est petit-boutiste, et `brut` est le champ de [`Fixe`]) :
//!
//! ```text
//! 8 octets   l'étiquette ASCII "GIVRE003"
//! 1 octet    1 (la forme : un hexagone, comme à l'étape 0)
//! 4 octets   le rayon R de l'hexagone, en u32
//! 24 octets  alpha.brut, beta.brut, gamma.brut, chacun en i64 (8 octets)
//! 8 octets   le temps (le nombre de pas faits), en u64
//! 8 octets   par case, dans l'ordre canonique (r croissant, puis q croissant), la masse : son brut en i64
//! ```
//!
//! Soit `45 + 8 * cases` octets. Il n'y a **aucune graine** dans l'écriture (aucun hasard) : deux lancers du même scénario
//! avec deux graines donnent la même empreinte. Deux jeux de paramètres, deux rayons, deux temps ou deux états différents
//! donnent deux écritures différentes.
//!
//! # L'aperçu
//!
//! [`Flocon::apercu`] rend une image RVB (trois octets par pixel, ligne par ligne de haut en bas, comme
//! [`crate::image::ecrire_png`]) **carrée de côté `2R + 1`**, `R` le rayon de l'hexagone. La case `(q, r)` est le pixel
//! `(x, y)` avec `x = q + R + floor(r / 2)` (la division entière **vers le bas**, `r.div_euclid(2)` : `-1` donne `-1`) et
//! `y = r + R` : c'est la position cartésienne `q + r/2` de la case, arrondie vers le bas, une rangée par `r`. Deux cases
//! n'ont jamais le même pixel. Les pixels qui ne sont l'image d'aucune case (les coins du carré) sont `(0, 0, 0)`. La couleur d'une case :
//! - gelée (masse au moins 1) : `(235, 245, 255)` ;
//! - sinon, avec `niveau = (brut * 255) >> 32` (la masse `s < 1` donne un entier de 0 à 254, décalage arithmétique) :
//!   `(0, niveau / 2, niveau)` (division entière).
//!
//! # La directive `reiter` et ses sorties
//!
//! Dans un scénario : `hexagone <rayon>` et **une seule** autre directive, `reiter <alpha> <beta> <gamma> <pas>`. `alpha`,
//! `beta` et `gamma` sont des décimaux lus par [`Parametres::lire`] ; `pas` est un entier de 1 à [`PAS_MAX`] en chiffres
//! décimaux ASCII seulement. [`lancer_reiter`] (que [`crate::lancer::lancer`] appelle) crée `Flocon::new(rayon, paramètres)`,
//! le fait avancer de `pas` pas, et rend :
//! - `empreinte` : [`Flocon::empreinte`] de l'état final (la graine du lancer n'est pas utilisée) ;
//! - `mesures` : le CSV `temps,gelees,rayon,compacite\n` puis une ligne `temps,gelees,rayon,compacite` pour le temps 0 (l'état
//!   initial) et ensuite pour chaque temps `t` multiple de `intervalle = max(1, pas / 20)` (division entière) avec
//!   `1 <= t <= pas` (le temps 0 n'a qu'une ligne), puis, si `pas` n'est pas un multiple de `intervalle`, une dernière ligne pour `t = pas`. `temps` est
//!   `t`, `gelees` est [`Flocon::gelees`], `rayon` est [`Flocon::rayon_cristal`], et `compacite` est la compacité écrite en
//!   **six décimales exactes** : `ppm = (2_000_000 * gelees + cases) / (2 * cases)` (division entière, `cases` = les
//!   cases de l'hexagone de rayon `rayon_cristal`), écrit `{ppm / 1000000}.{ppm % 1000000 sur 6 chiffres}` (`1.000000`,
//!   `0.368421`). Chaque ligne, la dernière comprise, finit par un seul `\n` ;
//! - `apercu` : le PNG (`image::ecrire_png`) de [`Flocon::apercu`] de l'état final, de côté `2 * rayon + 1`.
//!
//! Le scénario est refusé, avec le numéro de la ligne de la première directive fautive (voir [`lancer_reiter`]), s'il y a
//! une autre directive que l'unique `reiter`, si la forme n'est pas un hexagone, ou si un argument est invalide.
//! [`crate::lancer::lancer`] appelle [`lancer_reiter`] quand la **première** directive du scénario s'appelle `reiter` (et rend
//! `ErreurLancer::DirectiveInconnue(ligne)` pour son `Err(ligne)`) ; toute autre première directive reste refusée comme à
//! l'étape 0.
//!
//! # Ce que le modèle n'est pas
//!
//! Le bord de Reiter est un cercle euclidien ; ici c'est la couronne de l'hexagone (la symétrie d'ordre 6 est exacte dans
//! les deux cas ; la couronne hexagonale tient dans un entier). Pour les tables de l'auteur, son script (`sfn.ijs`, page
//! auxiliaire) fixe à `β` les cases à distance euclidienne au moins `100 sin 60° ≈ 86,6` du germe pour une « région de rayon
//! 100 », et s'arrête quand la glace atteint `98 sin 60° ≈ 84,9`. **Valeur publiée non atteinte, cause établie** : la table de
//! Reiter (région de rayon 100) est reproduite **exactement** par ce modèle sur un hexagone de rayon 100 pour 160 des 219 cases non
//! plafonnées (toutes celles où `β >= 0,55` ou `γ >= 0,1`) ; pour `β <= 0,5` et `γ <= 0,05` ce modèle met 0,1 à 4,9 % de pas en plus,
//! parce que sa couronne est plus loin des pointes que le cercle de 86,6 de l'auteur (avec la géométrie de l'auteur et les mêmes
//! règles, les 240 cases sont retrouvées : voir `references/etape-3-valeurs.md`). Pour un rayon d'hexagone d'au moins 96 le bord ne
//! change plus la croissance d'une dendrite jusqu'au rayon 85 (mesuré sur β = 0,35 et γ = 0 : même temps de 96 à 160). Le modèle
//! est 2D et sans fusion ni sublimation ; il n'est pas une simulation de la physique (Reiter le dit lui-même). Les `α > 2` que
//! l'article essaie (2,003 ; 2,69 : le poids du centre est négatif) sont hors du modèle.

use crate::lancer::Sortie;
use crate::math::Fixe;
use crate::reseau::{DIRECTIONS_HEX, Hexagone};
use crate::scenario::Scenario;

/// Le plus grand nombre de pas qu'une directive `reiter` peut demander.
pub const PAS_MAX: u64 = 1_000_000;

/// Les trois paramètres du modèle (voir la documentation du module). Un jeu est **valide** si
/// `0 <= alpha <= 2`, `0 <= beta < 1` et `0 <= gamma <= 1` (voir [`Parametres::sont_valides`]).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Parametres {
    /// La diffusion `α` : le centre garde `1 - α/2` de sa masse non réceptive, chaque voisine reçoit `α/12`. `α = 1` est
    /// le modèle de l'article.
    pub alpha: Fixe,
    /// La vapeur ambiante `β` : la masse initiale partout, et la valeur fixée sur la couronne.
    pub beta: Fixe,
    /// La masse `γ` ajoutée à chaque pas à chaque cellule réceptive.
    pub gamma: Fixe,
}

impl Parametres {
    /// Vrai si le jeu est valide : `0 <= alpha <= 2` (`Fixe::entier(2)` compris), `0 <= beta < 1` (`Fixe::UN` exclu :
    /// la couronne serait gelée) et `0 <= gamma <= 1` (`Fixe::UN` compris).
    pub fn sont_valides(&self) -> bool {
        self.alpha.0 >= 0
            && self.alpha <= Fixe::entier(2)
            && self.beta.0 >= 0
            && self.beta < Fixe::UN
            && self.gamma.0 >= 0
            && self.gamma <= Fixe::UN
    }

    /// Lit trois décimaux écrits comme dans un scénario (voir [`lire_decimal`]) et rend les paramètres, ou `None` si l'un
    /// d'eux est mal écrit ou si le jeu n'est pas valide ([`Parametres::sont_valides`]).
    pub fn lire(alpha: &str, beta: &str, gamma: &str) -> Option<Parametres> {
        let parametres =
            Parametres { alpha: lire_decimal(alpha)?, beta: lire_decimal(beta)?, gamma: lire_decimal(gamma)? };
        parametres.sont_valides().then_some(parametres)
    }
}

/// Lit un décimal en notation simple, **exactement** : des chiffres ASCII, puis facultativement un point et au moins un
/// chiffre (`0.35`, `1`, `0.0001`, `007`). Ni signe, ni exposant, ni `.5`, ni `5.`, ni espace, ni chaîne vide. Le nombre
/// total de chiffres (partie entière et partie fractionnaire, zéros de tête compris) est au plus 18. Si le texte s'écrit
/// `N` suivi de `k` chiffres après la virgule, la valeur est le rapport **exact** `N / 10^k` arrondi une seule fois, comme
/// `Fixe::rapport(N, 10^k)` (au plus proche, demi-cas vers `+∞` ; avec au plus 18 chiffres il n'y a jamais de demi-cas),
/// sans passer par un flottant. Exemples : `"0.5"` donne `brut = 2^31` ; `"0.1"` donne `brut = 429496730` ; `"1"` donne
/// `Fixe::UN`.
///
/// Rend `None` si le texte n'a pas cette forme, a plus de 18 chiffres, ou vaut `2^31` ou plus.
pub fn lire_decimal(texte: &str) -> Option<Fixe> {
    let octets = texte.as_bytes();
    let mut chiffres = 0_usize;
    let mut valeur: i128 = 0;
    let mut apres_point = false;
    let mut decimaux = 0_usize;
    let mut vu_chiffre_entier = false;
    let mut vu_chiffre_decimal = false;
    for &octet in octets {
        match octet {
            b'0'..=b'9' => {
                chiffres += 1;
                if chiffres > 18 {
                    return None;
                }
                valeur = valeur * 10 + i128::from(octet - b'0');
                if apres_point {
                    decimaux += 1;
                    vu_chiffre_decimal = true;
                } else {
                    vu_chiffre_entier = true;
                }
            }
            b'.' => {
                if apres_point {
                    return None;
                }
                apres_point = true;
            }
            _ => return None,
        }
    }
    // Il faut au moins un chiffre entier, et après un point au moins un chiffre décimal.
    if !vu_chiffre_entier || (apres_point && !vu_chiffre_decimal) {
        return None;
    }
    let denominateur = 10_i128.checked_pow(decimaux as u32)?;
    // floor(N · 2^32 / 10^k + 1/2), en i128 : comme `Fixe::rapport(valeur, denominateur)`.
    let numerateur = valeur * (1_i128 << 32);
    let brut = (2 * numerateur + denominateur).div_euclid(2 * denominateur);
    if brut > i128::from(i64::MAX) {
        return None;
    }
    Some(Fixe(i64::try_from(brut).expect("brut dans l'intervalle i64")))
}

/// La réflexion `(q, r) -> (r, q)` : une symétrie miroir de l'hexagone (elle envoie les directions `[(1, 0), (0, 1),
/// (-1, 1), (-1, 0), (0, -1), (1, -1)]` sur `[(0, 1), (1, 0), (1, -1), (0, -1), (-1, 0), (-1, 1)]`, donc les voisins
/// d'une case sur ceux de son image, et un hexagone centré à l'origine sur lui-même). Elle est son propre inverse.
pub fn reflechir(q: i32, r: i32) -> (i32, i32) {
    (r, q)
}

/// La classe d'une cellule (voir « Les classes de cellules »).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Etat {
    /// Ni gelée, ni voisine d'une cellule gelée : elle diffuse.
    NonReceptive,
    /// Pas gelée, mais au moins un voisin gelé : réceptive.
    DeBord,
    /// Masse au moins 1.
    Gelee,
}

/// Un flocon de Reiter : un hexagone de masses, ses paramètres et son temps (voir la documentation du module). Les champs
/// sont privés : seul le contrat des fonctions compte.
#[derive(Clone, Debug)]
#[allow(dead_code)]
pub struct Flocon {
    hexagone: Hexagone,
    parametres: Parametres,
    temps: u64,
    masses: Vec<Fixe>,
    /// Pour chaque case, l'indice canonique de ses six voisins (`-1` : hors de l'hexagone, à sa place).
    voisins: Vec<[i32; 6]>,
    /// Les cases à distance strictement inférieure à `R`, dans l'ordre canonique : leurs six voisins existent.
    interieurs: Vec<usize>,
    /// Les cases de la couronne (distance exactement `R`), dans l'ordre canonique.
    couronne: Vec<usize>,
    /// Tampon des classes (0 : non réceptive, 1 : de bord, 2 : gelée), réutilisé à chaque pas.
    classes: Vec<u8>,
    /// Tampon de `u(x)` (la masse qui diffuse), réutilisé à chaque pas.
    u: Vec<Fixe>,
    /// Le nouvel état d'un pas, échangé avec `masses` à la fin.
    tampon: Vec<Fixe>,
}

impl Flocon {
    /// Le flocon de l'état initial (voir « L'état ») sur l'hexagone de rayon `rayon`, au temps 0 : toutes les masses valent
    /// `parametres.beta`, sauf la case `(0, 0)` qui vaut `Fixe::UN`.
    ///
    /// # Panique
    ///
    /// Si `rayon` est 0 (ou plus de 32768, comme [`Hexagone::new`]), ou si les paramètres ne sont pas valides
    /// ([`Parametres::sont_valides`]).
    pub fn new(rayon: u32, parametres: Parametres) -> Flocon {
        assert!(parametres.sont_valides(), "paramètres invalides");
        assert!(rayon > 0, "le rayon doit être au moins 1");
        let hexagone = Hexagone::new(rayon);
        let mut masses = vec![parametres.beta; hexagone.cases()];
        masses[hexagone.index(0, 0)] = Fixe::UN;
        Flocon::construire(hexagone, parametres, masses)
    }

    /// Construit un flocon et ses tables à partir d'un état déjà rangé dans l'ordre canonique.
    fn construire(hexagone: Hexagone, parametres: Parametres, masses: Vec<Fixe>) -> Flocon {
        let cases = hexagone.cases();
        assert_eq!(masses.len(), cases, "nombre de masses inattendu");
        let mut voisins = vec![[-1_i32; 6]; cases];
        let mut interieurs = Vec::new();
        let mut couronne = Vec::new();
        for (i, voisins_i) in voisins.iter_mut().enumerate() {
            let (q, r) = hexagone.position(i);
            let mut tous_presents = true;
            for (k, &(dq, dr)) in DIRECTIONS_HEX.iter().enumerate() {
                let (nq, nr) = (q + dq, r + dr);
                if hexagone.contient(nq, nr) {
                    voisins_i[k] = hexagone.index(nq, nr) as i32;
                } else {
                    tous_presents = false;
                }
            }
            if tous_presents {
                interieurs.push(i);
            } else {
                couronne.push(i);
            }
        }
        Flocon {
            hexagone,
            parametres,
            temps: 0,
            masses,
            voisins,
            interieurs,
            couronne,
            classes: vec![0; cases],
            u: vec![Fixe::ZERO; cases],
            tampon: vec![Fixe::ZERO; cases],
        }
    }

    /// Un flocon d'état quelconque : `masses` donne la masse de chaque case **dans l'ordre canonique** de
    /// [`Hexagone`] (`masses[Hexagone::new(rayon).index(q, r)]` est la masse de `(q, r)`), au temps 0. Les masses de la
    /// couronne sont prises telles quelles (elles ne sont remises à `β` qu'au premier pas). Sert à poser une configuration
    /// à la main.
    ///
    /// # Panique
    ///
    /// Si `rayon` est 0 ou trop grand, si les paramètres ne sont pas valides, si `masses.len()` n'est pas le nombre de
    /// cases de l'hexagone, ou si une masse est négative ou vaut `2^60` en `brut` (ou plus).
    pub fn depuis_masses(rayon: u32, parametres: Parametres, masses: &[Fixe]) -> Flocon {
        assert!(parametres.sont_valides(), "paramètres invalides");
        assert!(rayon > 0, "le rayon doit être au moins 1");
        let hexagone = Hexagone::new(rayon);
        assert_eq!(masses.len(), hexagone.cases(), "nombre de masses inattendu");
        for masse in masses {
            assert!(masse.0 >= 0, "masse négative");
            assert!(masse.0 < (1_i64 << 60), "masse de 2^60 brut ou plus");
        }
        Flocon::construire(hexagone, parametres, masses.to_vec())
    }

    /// Le rayon `R` de l'hexagone.
    pub fn rayon(&self) -> u32 {
        self.hexagone.rayon()
    }

    /// Les paramètres.
    pub fn parametres(&self) -> Parametres {
        self.parametres
    }

    /// Le temps : le nombre de pas faits depuis la construction (0 à la construction).
    pub fn temps(&self) -> u64 {
        self.temps
    }

    /// La masse de la case `(q, r)`.
    ///
    /// # Panique
    ///
    /// Si `(q, r)` n'est pas dans l'hexagone.
    pub fn masse(&self, q: i32, r: i32) -> Fixe {
        self.masses[self.hexagone.index(q, r)]
    }

    /// Toutes les masses, **dans l'ordre canonique** de [`Hexagone`] (`cases()` valeurs).
    pub fn masses(&self) -> Vec<Fixe> {
        self.masses.clone()
    }

    /// La classe de la case `(q, r)` pour l'état courant (voir « Les classes de cellules »).
    ///
    /// # Panique
    ///
    /// Si `(q, r)` n'est pas dans l'hexagone.
    pub fn etat(&self, q: i32, r: i32) -> Etat {
        let i = self.hexagone.index(q, r);
        if self.masses[i].0 >= Fixe::UN.0 {
            return Etat::Gelee;
        }
        for &voisin in &self.voisins[i] {
            if voisin >= 0 && self.masses[voisin as usize].0 >= Fixe::UN.0 {
                return Etat::DeBord;
            }
        }
        Etat::NonReceptive
    }

    /// Un pas du modèle, à la lettre de « Un pas, à la lettre » : l'état est remplacé, le temps augmente de 1.
    pub fn pas(&mut self) {
        let alpha = self.parametres.alpha;
        let beta = self.parametres.beta;
        let gamma = self.parametres.gamma;
        let seuil = Fixe::UN.0;

        // 1–2. La classe de chaque case et la masse qui diffuse, à partir de l'état seul.
        for (i, &masse) in self.masses.iter().enumerate() {
            if masse.0 >= seuil {
                self.classes[i] = 2;
                self.u[i] = Fixe::ZERO;
            } else {
                let de_bord =
                    self.voisins[i].iter().any(|&voisin| voisin >= 0 && self.masses[voisin as usize].0 >= seuil);
                if de_bord {
                    self.classes[i] = 1;
                    self.u[i] = Fixe::ZERO;
                } else {
                    self.classes[i] = 0;
                    self.u[i] = masse;
                }
            }
        }

        // 3. Les cases intérieures : diffusion, puis γ pour les réceptives.
        for &i in &self.interieurs {
            let mut somme: i64 = 0;
            for &voisin in &self.voisins[i] {
                somme += self.u[voisin as usize].0;
            }
            let moyenne = Fixe(somme).div_entier(6);
            let ecart = moyenne - self.u[i];
            let diffusion = (alpha * ecart).div_entier(2);
            self.tampon[i] =
                if self.classes[i] == 0 { self.masses[i] + diffusion } else { self.masses[i] + gamma + diffusion };
        }

        // 4. La couronne extérieure est maintenue à β.
        for &i in &self.couronne {
            self.tampon[i] = beta;
        }

        std::mem::swap(&mut self.masses, &mut self.tampon);
        // 5. Le temps.
        self.temps += 1;
    }

    /// `n` pas de suite : `n` appels de [`Flocon::pas`].
    pub fn avancer(&mut self, n: u64) {
        for _ in 0..n {
            self.pas();
        }
    }

    /// Le nombre de cases gelées (masse au moins 1), couronne comprise.
    pub fn gelees(&self) -> usize {
        todo!()
    }

    /// Le rayon du cristal : la plus grande distance hexagonale au centre d'une case gelée (0 s'il n'y en a aucune, ou
    /// seulement le centre).
    pub fn rayon_cristal(&self) -> u32 {
        todo!()
    }

    /// La compacité : `Fixe::rapport(gelees, 3 r (r + 1) + 1)` avec `r = rayon_cristal()` (voir « Les mesures »).
    pub fn compacite(&self) -> Fixe {
        todo!()
    }

    /// Le nombre de cases gelées qui ont exactement un voisin gelé (voir « Les mesures »).
    pub fn pointes(&self) -> usize {
        todo!()
    }

    /// Le défaut de rotation : le nombre de cases `(q, r)` telles que `masse(q, r) != masse(tourner60(q, r))`.
    pub fn defaut_rotation(&self) -> usize {
        todo!()
    }

    /// Le défaut de réflexion : le nombre de cases `(q, r)` telles que `masse(q, r) != masse(reflechir(q, r))`.
    pub fn defaut_reflexion(&self) -> usize {
        todo!()
    }

    /// L'écriture canonique de l'état, `45 + 8 * cases` octets (voir « L'écriture canonique et l'empreinte »).
    pub fn ecriture_canonique(&self) -> Vec<u8> {
        todo!()
    }

    /// L'empreinte : le SHA-256 en hexadécimal minuscule (`empreinte::sha256_hex`) de [`Flocon::ecriture_canonique`].
    pub fn empreinte(&self) -> String {
        todo!()
    }

    /// L'aperçu : l'image RVB carrée de côté `2R + 1`, `3 * (2R + 1)^2` octets (voir « L'aperçu »).
    pub fn apercu(&self) -> Vec<u8> {
        todo!()
    }
}

/// Lance la directive `reiter` d'un scénario (voir « La directive `reiter` et ses sorties ») : à n'appeler que lorsque
/// `scenario.directives` n'est pas vide et que sa première directive s'appelle `reiter`
/// ([`crate::lancer::lancer`] le fait).
///
/// # Erreurs
///
/// `Err(ligne)`, le numéro de ligne de la **première directive fautive dans l'ordre du fichier** : la directive `reiter`
/// elle-même si la forme n'est pas un hexagone, si elle n'a pas exactement quatre arguments, si `alpha`, `beta`, `gamma`
/// ne se lisent pas par [`Parametres::lire`] ou si `pas` n'est pas un entier décimal de 1 à [`PAS_MAX`] ; sinon la ligne de
/// la directive suivante, quelle qu'elle soit (il n'y a qu'une directive en plus de la forme). Rien n'est calculé alors.
#[allow(unused_variables)]
pub fn lancer_reiter(scenario: &Scenario) -> Result<Sortie, usize> {
    todo!()
}

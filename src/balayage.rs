//! Les balayages de la neige (étape G2) : on fait varier **un seul** réglage de la neige de G1 ([`crate::neige`]), on fait grandir un cristal par valeur,
//! et on range chaque cristal par des **mesures en nombres entiers exacts** et par une **étiquette** (plaque, secteurs, dendrite, fougère). Tout est
//! en entiers : aucun flottant, aucune `HashMap`, aucun hasard ; les mesures, les étiquettes et les empreintes sont les mêmes sous Linux et sous
//! Windows. Ce module ne change rien à [`crate::neige`] (l'état, l'empreinte et le rendu d'un cristal de G1 restent à l'octet).
//!
//! # Ce que ce module fait, et ce qu'il ne fait pas
//!
//! Il fait trois choses : [`mesurer`] (six entiers sur un cristal fini), [`classer`] (une étiquette [`Forme`] tirée de ces entiers par une **table de
//! règles à seuils** : voir « L'étiquette ») et [`balayage`] (un cristal par valeur d'un réglage, dans l'ordre des valeurs). Les trois jeux de
//! balayage de l'article (`ρ`, `β`, `μ`) sont [`jeu_rho`], [`jeu_beta`] et [`jeu_mu`].
//!
//! **Il ne prouve pas que notre neige ressemble à celle de l'article.** L'article ne donne que des images et des réglages, **aucun nombre** pour ces
//! figures : aucune mesure d'ici n'est une valeur publiée. **Les mesures sont exactes** (ce sont des entiers que la doc définit au bit près) ;
//! **l'étiquette est un modèle à nous** : ses seuils sont les nôtres, tirés de nos images et de nos mesures, à la taille d'ici (voir « L'étiquette »).
//! Ce qui s'établit : ce que **notre code** fait quand on balaie un réglage, et si l'**ordre** que les auteurs décrivent revient (`MODELE.md`, « Ce que
//! l'on retrouve de l'article »).
//!
//! # La référence : ce qui est lu, ce qui ne l'est pas
//!
//! J. Gravner et D. Griffeath, « Modeling snow crystal growth II: A mesoscopic lattice map with plausible dynamics », *Physica D* 237 (2008) 385-404,
//! <https://doi.org/10.1016/j.physd.2007.09.008>. **Lu** : la préimpression de l'auteur (« Revised version, September 2007 », 37 pages ; `sha256`
//! `0b2ce753…`), section 5 (pages imprimées 20 à 23 : les huit paramètres, les figures 9, 10 et 12 et le texte qui les commente), l'annexe 8 (page
//! imprimée 36 : les réglages de chaque figure) ; **les figures 9, 10 et 12 ont été vues** (images extraites du PDF) : la figure 9 porte `ρ = .40, .41, .42,
//! .44, .46, .50`, la figure 10 porte `β = 1.9, 2.2, 2.4, 2.6, 2.7, 2.8`, la figure 12 porte `μ = .04, .05, .06, .07, .08, .09`, chacune en six cristaux. **Non lu** :
//! la version publiée dans *Physica D* (l'éditeur refuse l'accès ; la copie du site de l'auteur est le même fichier que la préimpression) ; la figure 11 (le
//! réglage `κ`), les figures 13 à 17 (celles-ci sont hors de cette étape). La fiche `references/etape-g2-valeurs.md` dit, pour chaque réglage, la page, la
//! figure et la ligne de l'annexe d'où il est lu, et les endroits où la source est ambiguë.
//!
//! **Ce que l'article dit de ces trois figures** (section 5, paraphrasé) : figure 9, « les cristaux croissent plus vite quand `ρ` augmente » et la forme passe de la
//! plaque à la plaque à secteurs puis à la dendrite ; figure 10, `β` de 1,9 à 2,8 : de la fougère à la dendrite puis à la plaque à secteurs (le début de la
//! première instabilité est retardé, la plaque hexagonale centrale grandit, les pointes des branches deviennent plus hexagonales, le goût de la ramification latérale
//! baisse) ; figure 12, `μ` de 0,04 à 0,09 : la fonte favorise les facettes, les pointes paraboliques de `μ = .04` et `.05` laissent la place à des pointes hexagonales (d'abord
//! aux branches principales, puis aux grandes branches latérales) et, à `μ = .09`, à ce que l'article appelle une « stellar plate » (une plaque étoilée : six bras larges à bouts hexagonaux ;
//! **pas** « sectored plate » : notre étiquette n'a pas de classe pour elle et la range [`Forme::Secteurs`]). L'article écrit aussi que les frontières de toute classification
//! sont arbitraires (à propos du cristal de `μ = .08`). **La figure 9 montre des cristaux « grown to comparable size »** (menés à une taille comparable, donc à des temps différents) ;
//! **les balayages d'ici s'arrêtent tous au même nombre de pas** : un cristal lent y est plus petit, donc plus tôt dans sa croissance, qu'un cristal rapide. **Les cristaux de l'article font 400 à 600 cases de diamètre en 10 000 à 100 000 pas** ; ceux d'ici font de 25 à 62
//! cases de rayon en 1 300 à 2 000 pas : on y voit un **stade précoce** de la croissance, pas la forme finale (`MODELE.md`).
//!
//! # Les mesures (des entiers exacts)
//!
//! Soit `A` l'ensemble des cases **rattachées** de la neige (celles dont `Case::a` est vrai : on les lit par [`Neige::cases`], dans l'ordre canonique de [`crate::reseau::Hexagone`], ou par
//! [`Neige::case`], qui rend `None` hors de l'hexagone), `Rd` le rayon de son hexagone ([`Neige::rayon`], de 1 à [`crate::neige::RAYON_MAX`]), et pour une case `x` d'`A`, `dist(x)` la distance
//! hexagonale de `(0, 0)` à `x` ([`crate::reseau::distance_hex`]). Pour une case `x` de `A`, `libres(x)` est le nombre des **six** cases `x + d`, `d` dans
//! [`crate::reseau::DIRECTIONS_HEX`], qui **ne sont pas dans `A`** : une case voisine qui est **hors de l'hexagone** (au-delà du mur) n'est pas dans `A` (le mur n'est pas
//! du cristal), elle compte donc comme libre ; une case voisine de l'hexagone qui n'est pas rattachée est libre (qu'elle porte de la masse de bord, de glace ou de vapeur :
//! **seul `a` compte**). Les six mesures de [`Mesures`], toutes des `u32`, calculées en entiers, **les produits et les quotients en `u64`** (`1000 · rattachees` dépasse un `u32` dès 4 294 968 cases rattachées, ce qu'un hexagone de rayon
//! 1197 ou plus peut porter : un produit fait en `u32` y déborde) :
//!
//! | champ | définition (`R` désigne `rayon`) |
//! |---|---|
//! | `rattachees` | `card(A)` : le nombre de cases rattachées |
//! | `rayon` | `max dist(x)` pour `x` dans `A` (0 si `A` est vide) : le rayon du cristal (c'est [`Neige::rayon_cristal`]) |
//! | `compacite` | `floor(1000 · rattachees / (3 R (R + 1) + 1))` : la part du disque hexagonal de même rayon que le cristal remplit, en millièmes (`3 R (R + 1) + 1` est le nombre de cases de l'hexagone de rayon `R`) |
//! | `pointes` | `card{ x dans A : libres(x) >= 3 }` : le nombre de cases rattachées qui ont **au moins trois** voisines libres (les coins et les pointes ; une case sur un bord droit n'en a que deux) |
//! | `aiguilles` | `card{ x dans A : libres(x) >= 4 }` : le nombre de cases rattachées qui ont **au moins quatre** voisines libres, donc **au plus deux voisines rattachées** : les bouts de branches d'une case de large, les cases isolées (une branche large n'en a pas : son bout a trois voisines libres) |
//! | `hexagonalite` | `floor(1000 · m / max(1, 6 R))` avec `m = card{ x dans A : dist(x) = R }` : la part de l'anneau extérieur (les `6 R` cases à la distance exactement `R`) qui est rattachée, en millièmes (une pointe fine n'y laisse que quelques cases, une facette hexagonale en laisse beaucoup ; pour `R = 0` : `m` vaut 1 si le germe est rattaché, 0 sinon, et le dénominateur 1) |
//!
//! **Exemples calculés à la main** : le germe seul (`A = {(0, 0)}`) : `rattachees = 1`, `rayon = 0`, `compacite = 1000`, `pointes = 1` et `aiguilles = 1` (ses six voisines sont libres),
//! `hexagonalite = 1000`. Le disque hexagonal plein de rayon `r >= 1` (dans un hexagone de rayon `Rd > r`) : `rattachees = 3 r (r + 1) + 1`, `rayon = r`, `compacite = 1000`, `pointes = 6`, `aiguilles = 0`
//! (une case d'angle a trois voisines libres, une case de côté en a deux), `hexagonalite = 1000` (l'anneau est plein : `6 r` cases). Un bras `(0, 0), (1, 0), …, (L, 0)` : `rattachees = L + 1`,
//! `rayon = L`, `pointes = L + 1` et `aiguilles = L + 1` (chaque case a quatre voisines libres au moins), `hexagonalite = floor(1000 / (6 L))` (une seule case à la distance `L`).
//!
//! **Les bornes** : `0 <= compacite <= 1000` et `0 <= hexagonalite <= 1000` ; `aiguilles <= pointes <= rattachees`. **L'ordre des calculs ne change aucun bit** (des entiers). Les mesures
//! ne regardent que `a` : deux neiges qui ont les mêmes cases rattachées ont les mêmes mesures, quelles que soient leurs masses, leur temps ou leurs réglages.
//! **Les mesures ne changent pas** quand on applique à tout l'état une des douze symétries de l'hexagone (rotations de 60° et réflexions : `libres`, `dist` et `A` sont envoyés les uns sur les autres).
//!
//! # L'étiquette (notre modèle, pas l'article)
//!
//! [`classer`] range une [`Mesures`] dans une [`Forme`] par la table suivante : **la première règle dont la condition est vraie s'applique** (l'ordre est celui de la table ; les
//! comparaisons sont celles de la colonne, `>=` partout) :
//!
//! | n° | condition | forme | raison |
//! |---|---|---|---|
//! | 1 | `rayon < 10` | [`Forme::Aucune`] | un cristal de moins de 10 cases de rayon est un germe : toutes les formes y sont un hexagone |
//! | 2 | `compacite >= 950` | [`Forme::Plaque`] | un hexagone presque plein (l'article : la « plaque ») |
//! | 3 | `compacite >= 700` | [`Forme::Secteurs`] | six secteurs larges dont l'ensemble remplit encore les sept dixièmes du disque (la « plaque à secteurs ») |
//! | 4 | `aiguilles >= 6` | [`Forme::Fougere`] | au moins six bouts de branches d'une case de large (une par secteur : un cristal à six branches en a des multiples de six) : ce que `β` bas permet (la « fougère ») |
//! | 5 | sinon | [`Forme::Dendrite`] | des branches d'au moins deux cases de large, ou pas d'aiguille (la « dendrite ») |
//!
//! **Les constantes** : [`RAYON_MIN_FORME`] `= 10`, [`SEUIL_PLAQUE`] `= 950`, [`SEUIL_SECTEURS`] `= 700`, [`SEUIL_FOUGERE`] `= 6` (la règle 4 est `aiguilles >= SEUIL_FOUGERE`).
//! **Ces seuils sont à nous** : ils viennent des 18 cristaux des trois balayages de cette étape (à la taille de [`jeu_rho`], [`jeu_beta`],
//! [`jeu_mu`]), regardés et mesurés (`MODELE.md`, « Les seuils de l'étiquette ») ; l'article dit lui-même que les frontières d'une classification sont arbitraires. Ils ne sont **pas** des
//! valeurs publiées. La coupure la plus fragile est celle de la règle 4 (fougère / dendrite : un seul cristal des balayages est de l'autre côté, `β = 1,9`, qui a 12, 24 ou 36 aiguilles selon la taille, dans les huit tailles essayées ; tous les autres cristaux en ont 0 ; hors des dix-huit, à la même taille, `β = 1,8` en a 24, `β = 1,7` 36 et `β = 2,0` aucune, et `μ = 0,03` en a 12 : `MODELE.md`). Une étoile de lignes d'une case est rangée
//! `Fougere` (chacune de ses cases est une aiguille) et un seul bras long aussi : l'étiquette est un seuil sur nos mesures, pas une définition de la forme. L'étiquette ne dépend que de [`Mesures`] : mêmes mesures, même forme.
//!
//! # Un balayage
//!
//! [`balayage`] prend un rayon `rayon` (celui de l'**hexagone** où grandit chaque cristal, le « domaine »), un nombre de pas `pas` (le même pour toutes les valeurs), un jeu de réglages de base `base`,
//! **un** réglage `reglage` ([`Reglage`]) et une liste de valeurs. Pour chaque valeur, **dans l'ordre de la liste** : les réglages sont ceux de `base`, **sauf** `reglage`, qui prend la valeur
//! (le réglage balayé de `base` n'est jamais lu) ; on crée `Neige::essayer_new(rayon, réglages)`, on la fait avancer de `pas` pas (`Neige::avancer(pas)`), puis on rend un [`Resultat`] :
//! la `valeur`, les [`Mesures`] de l'état final (`mesurer`), sa [`Forme`] (`classer` de ces mesures) et son `empreinte` (`Neige::empreinte()` de l'état final : celle de G1, l'écriture canonique de G1).
//! Les cristaux sont **indépendants** (rien ne passe d'une valeur à la suivante) ; chaque cristal est calculé à tour de rôle puis oublié. Une fois les refus ci-dessous passés, `Neige::essayer_new` **ne peut plus
//! refuser** (le rayon est entre 1 et 400, donc au plus [`crate::neige::RAYON_MAX`], et les réglages viennent de passer [`Parametres::verifier`]) : le code peut s'appuyer dessus (un `expect`), aucun appel ne l'atteint.
//!
//! **Les refus** : [`balayage`] rend `Err` (jamais de panique, **rien n'est calculé** quand il refuse), dans cet ordre, le premier qui s'applique :
//! 1. [`ErreurBalayage::RayonHorsBorne`] si `rayon` est 0 ou plus grand que [`RAYON_MAX_BALAYAGE`] (400) ;
//! 2. [`ErreurBalayage::PasHorsBorne`] si `pas` est 0 ou plus grand que [`PAS_MAX_BALAYAGE`] (100 000) ;
//! 3. [`ErreurBalayage::ListeVide`] si `valeurs` est vide ;
//! 4. [`ErreurBalayage::ListeTropLongue`] si `valeurs` a plus de [`VALEURS_MAX`] (16) éléments ;
//! 5. [`ErreurBalayage::ReglageHorsPlage`] `{ nom, indice }` : pour chaque valeur, dans l'ordre, on forme ses réglages (`base` avec `reglage` remplacé) et on appelle
//!    [`Parametres::verifier`] ; le **premier** jeu invalide donne `indice` (son rang dans `valeurs`, à partir de 0) et `nom` (le nom que `verifier` donne : le **premier** réglage hors plage dans l'ordre `rho`, `beta`,
//!    `alpha`, `theta`, `kappa`, `mu`, `gamma`). C'est donc le réglage balayé quand sa valeur est hors plage, mais c'est un autre réglage si c'est `base` qui est fautive (alors `indice` vaut 0).
//!
//! **Coût** : un balayage coûte la somme des coûts de ses cristaux (G1 : 22 à 40 ns par case et par pas) ; les bornes ci-dessus ne sont pas des garanties de temps (`RAYON_MAX_BALAYAGE` et `PAS_MAX_BALAYAGE` ensemble
//! demanderaient des heures) : ce sont des bornes de **forme**, pour qu'un appel absurde soit refusé.
//!
//! # Les trois jeux de balayage
//!
//! Chacun est un [`JeuDeBalayage`] : le nom, le réglage balayé, les réglages de base, les six valeurs, et le rayon de domaine et le nombre de pas retenus **dans cette étape** (mesurés : `MODELE.md`, « Le coût » ; le
//! mur réfléchit : le domaine, de rayon 100, est assez grand pour que **chacun des dix-huit cristaux ait exactement les mêmes cases rattachées que dans un domaine de rayon 120** (mesuré ; le plus grand cristal a 62 cases de rayon ;
//! dans un domaine de rayon 84 les cristaux de `β = 1,9` et de `μ = 0,05` sentent déjà le mur : `MODELE.md`, « Le mur »). Tous ont `σ = 0` (pas de bruit : G1 n'en a pas). Chaque réglage est lu en décimal, **exactement**
//! (`Parametres::lire`, donc `lire_decimal`), la valeur balayée de `base` est la première valeur de la liste.
//!
//! | jeu | réglage balayé | valeurs | `ρ` | `β` | `α` | `θ` | `κ` | `μ` | `γ` | domaine | pas | source |
//! |---|---|---|---|---|---|---|---|---|---|---|---|---|
//! | [`jeu_rho`], `"rho"` | `ρ` | `0.40 0.41 0.42 0.44 0.46 0.50` | (balayé) | `1.3` | `0.08` | `0.025` | `0.003` | `0.07` | `0.00005` | 100 | 2000 | figure 9 ; annexe 8, ligne « 9 » ; la ligne « 15tr » en est la copie avec `ρ = .5` |
//! | [`jeu_beta`], `"beta"` | `β` | `1.9 2.2 2.4 2.6 2.7 2.8` | `0.8` | (balayé) | `0.004` | `0.001` | `0.05` | `0.015` | `0.0001` | 100 | 1300 | figure 10 ; annexe 8, ligne « 10 » |
//! | [`jeu_mu`], `"mu"` | `μ` | `0.04 0.05 0.06 0.07 0.08 0.09` | `0.5` | `1.4` | `0.1` | `0.005` | `0.001` | (balayé) | `0.001` | 100 | 2000 | figure 12 ; annexe 8, ligne « 12 » |
//!
//! **Ce qui est lu, et ce qui est déduit** : les valeurs balayées sont lues **sur les figures** (la légende de chaque cristal). Les autres réglages sont lus sur l'annexe 8, dont l'ordre des colonnes est
//! `ρ β α θ κ μ γ σ` (en-tête présent dans le texte) ; **dans les lignes « 9 », « 10 » et « 12 », la colonne du réglage qui varie est absente** (la ligne a sept nombres au lieu de huit) : l'attribution des
//! sept nombres aux sept autres colonnes, dans l'ordre, est une **déduction** (recoupée : la ligne « 10 » partage `κ μ γ = .05 .015 .0001` avec les lignes « 2tr » et « 6b », qui ont `ρ = .8` ; la ligne « 9 » est
//! recopiée par « 15tr » ; les images obtenues par ces réglages ont les traits des figures : fougère à `β = 1.9`, dendrite fine à `μ = .04`). Aucune de ces trois lignes n'est contredite par le texte de
//! l'article ; la préimpression se contredit ailleurs (figure 13 : `references/etape-g1-valeurs.md`), pas ici.
//!
//! # Le CSV
//!
//! [`EN_TETE_CSV`] est la première ligne d'un fichier de résultats ; [`ligne_csv`] écrit une ligne par cristal. Les deux finissent par un seul `\n` (jamais de `\r`). **Les champs, séparés par une virgule**, dans cet ordre :
//! `balayage` (le nom du jeu, tel qu'il est passé), `valeur` (la valeur balayée en **six décimales exactes**, arrondie au plus proche, le demi-cas vers le haut : avec `brut` le `i64` de la valeur (`Fixe.0`, des `2^-32`), `ppm = (2_000_000 · brut + 2^32) / 2^33` en `i128`
//! (division entière), écrit `{ppm / 1_000_000}.{ppm % 1_000_000 sur six chiffres, complété par des zéros à gauche}` : `0,05` s'écrit `0.050000`, `1/128` s'écrit `0.007813` ; la valeur est supposée **non négative**, comme le sont les
//! sept plages : pour une valeur négative le texte n'est pas défini, et aucun test ne le regarde), `rayon_domaine`, `pas`, `rattachees`, `rayon`, `compacite`, `pointes`, `aiguilles`, `hexagonalite` (ces huit entiers en décimal, sans signe ni zéro de tête), `forme` (le nom de [`Forme::nom`]), `empreinte` (recopiée telle quelle).
//!
//! # Ce que ce module n'est pas
//!
//! - **Ce n'est pas une comparaison aux figures de l'article** : aucune image, aucun nombre publié n'est comparé ; les planches (`apercus/`) sont pour l'œil.
//! - **Ce n'est pas la neige de l'article à sa taille** (400 à 600 cases, 10 000 à 100 000 pas) : les balayages d'ici sont des **stades précoces** ; le classement dépend de la taille et du temps (les mêmes réglages donnent d'autres formes à d'autres tailles).
//! - **Le mur est à nous** (G1) : un cristal qui l'atteint n'est pas celui de l'article ; les balayages gardent les cristaux à distance du mur (voir `MODELE.md`).
//! - **Ni le bruit `σ`, ni la 3D, ni la forme selon la température** (le diagramme de Nakaya : question ouverte, les auteurs le disent).

use crate::math::Fixe;
use crate::neige::{Neige, Parametres};

/// Le plus grand rayon de domaine qu'un balayage accepte (la taille de l'article en rayon : 400 cases ; un tel domaine a 481 201 cases).
pub const RAYON_MAX_BALAYAGE: u32 = 400;

/// Le plus grand nombre de pas qu'un balayage accepte (la borne haute de l'article : 100 000 pas).
pub const PAS_MAX_BALAYAGE: u64 = 100_000;

/// Le plus grand nombre de valeurs d'un balayage.
pub const VALEURS_MAX: usize = 16;

/// Règle 1 de l'étiquette : un cristal de rayon inférieur à cette valeur est [`Forme::Aucune`].
pub const RAYON_MIN_FORME: u32 = 10;

/// Règle 2 de l'étiquette : `compacite >= SEUIL_PLAQUE` donne [`Forme::Plaque`].
pub const SEUIL_PLAQUE: u32 = 950;

/// Règle 3 de l'étiquette : `compacite >= SEUIL_SECTEURS` donne [`Forme::Secteurs`].
pub const SEUIL_SECTEURS: u32 = 700;

/// Règle 4 de l'étiquette : `aiguilles >= SEUIL_FOUGERE` donne [`Forme::Fougere`].
pub const SEUIL_FOUGERE: u32 = 6;

/// La première ligne du CSV des résultats (voir « Le CSV »), terminée par `\n`.
pub const EN_TETE_CSV: &str =
    "balayage,valeur,rayon_domaine,pas,rattachees,rayon,compacite,pointes,aiguilles,hexagonalite,forme,empreinte\n";

/// Un des sept réglages de la neige ([`Parametres`]), celui qu'un balayage fait varier.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Reglage {
    /// `ρ`, la vapeur de départ ([`Parametres::rho`]).
    Rho,
    /// `β`, le seuil de masse de bord ([`Parametres::beta`]).
    Beta,
    /// `α` ([`Parametres::alpha`]).
    Alpha,
    /// `θ` ([`Parametres::theta`]).
    Theta,
    /// `κ` ([`Parametres::kappa`]).
    Kappa,
    /// `μ`, la fonte ([`Parametres::mu`]).
    Mu,
    /// `γ` ([`Parametres::gamma`]).
    Gamma,
}

impl Reglage {
    /// Le nom du réglage, celui que [`Parametres::verifier`] donne : `"rho"`, `"beta"`, `"alpha"`, `"theta"`, `"kappa"`, `"mu"` ou `"gamma"`.
    #[allow(unused_variables)]
    pub fn nom(self) -> &'static str {
        todo!()
    }

    /// Les réglages de `parametres`, **sauf** celui-ci, qui prend `valeur` (les six autres sont ceux de `parametres`, à l'identique). Ne vérifie rien.
    #[allow(unused_variables)]
    pub fn fixer(self, parametres: Parametres, valeur: Fixe) -> Parametres {
        todo!()
    }
}

/// Les six mesures d'un cristal (voir « Les mesures »). Chaque champ est un entier exact défini au bit près par la doc du module.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Mesures {
    /// Le nombre de cases rattachées.
    pub rattachees: u32,
    /// Le rayon du cristal : la plus grande distance hexagonale au centre d'une case rattachée (0 si aucune).
    pub rayon: u32,
    /// La part du disque hexagonal de même rayon que remplit le cristal, en millièmes (partie entière).
    pub compacite: u32,
    /// Le nombre de cases rattachées qui ont au moins trois voisines libres.
    pub pointes: u32,
    /// Le nombre de cases rattachées qui ont au moins quatre voisines libres (au plus deux voisines rattachées).
    pub aiguilles: u32,
    /// La part de l'anneau extérieur (à la distance exactement `rayon`) qui est rattachée, en millièmes (partie entière).
    pub hexagonalite: u32,
}

/// Une étiquette de forme (voir « L'étiquette » : notre modèle, seuils à nous).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Forme {
    /// Trop petit pour être rangé (règle 1).
    Aucune,
    /// Un hexagone presque plein (règle 2).
    Plaque,
    /// Une plaque à secteurs (règle 3).
    Secteurs,
    /// Une dendrite (règle 5).
    Dendrite,
    /// Une fougère (règle 4).
    Fougere,
}

impl Forme {
    /// Le nom écrit dans le CSV : `"aucune"`, `"plaque"`, `"secteurs"`, `"dendrite"` ou `"fougere"` (minuscules, sans accent).
    #[allow(unused_variables)]
    pub fn nom(self) -> &'static str {
        todo!()
    }
}

/// Ce qu'un balayage rend pour une valeur du réglage (voir « Un balayage »).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Resultat {
    /// La valeur du réglage balayé.
    pub valeur: Fixe,
    /// Les mesures de l'état final.
    pub mesures: Mesures,
    /// L'étiquette de ces mesures.
    pub forme: Forme,
    /// L'empreinte de G1 de l'état final (`Neige::empreinte`).
    pub empreinte: String,
}

/// Pourquoi un balayage a été refusé (voir « Un balayage » : l'ordre des refus).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ErreurBalayage {
    /// Le rayon de domaine est 0 ou dépasse [`RAYON_MAX_BALAYAGE`].
    RayonHorsBorne,
    /// Le nombre de pas est 0 ou dépasse [`PAS_MAX_BALAYAGE`].
    PasHorsBorne,
    /// La liste de valeurs est vide.
    ListeVide,
    /// La liste de valeurs a plus de [`VALEURS_MAX`] éléments.
    ListeTropLongue,
    /// Le jeu de réglages de la valeur d'indice `indice` (la première fautive, à partir de 0) a un réglage hors plage : `nom` est le premier dans l'ordre de [`Parametres::verifier`].
    ReglageHorsPlage {
        /// Le nom du réglage fautif (`"rho"`, `"beta"`, …).
        nom: &'static str,
        /// Le rang, dans la liste des valeurs, du premier jeu invalide.
        indice: usize,
    },
}

/// Un jeu de balayage de l'article (voir « Les trois jeux de balayage »).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct JeuDeBalayage {
    /// Le nom, écrit dans le CSV : `"rho"`, `"beta"` ou `"mu"`.
    pub nom: &'static str,
    /// Le réglage qui varie.
    pub reglage: Reglage,
    /// Les réglages de base ; la valeur du réglage balayé y est la première de `valeurs`.
    pub base: Parametres,
    /// Les six valeurs balayées, dans l'ordre de la figure de l'article.
    pub valeurs: Vec<Fixe>,
    /// Le rayon de l'hexagone où grandit chaque cristal, pour cette étape.
    pub rayon: u32,
    /// Le nombre de pas, pour cette étape.
    pub pas: u64,
}

/// Les six mesures de la neige `neige` (voir « Les mesures » : les définitions au bit près). Ne regarde que les `a` (les cases rattachées) ; jamais de panique.
#[allow(unused_variables)]
pub fn mesurer(neige: &Neige) -> Mesures {
    todo!()
}

/// L'étiquette de ces mesures (voir « L'étiquette » : la table de règles, la première qui s'applique).
#[allow(unused_variables)]
pub fn classer(mesures: &Mesures) -> Forme {
    todo!()
}

/// Un balayage : un cristal par valeur de `reglage`, dans l'ordre de `valeurs` (voir « Un balayage » : les calculs et l'ordre des refus). Jamais de panique.
#[allow(unused_variables)]
pub fn balayage(
    rayon: u32,
    pas: u64,
    base: Parametres,
    reglage: Reglage,
    valeurs: &[Fixe],
) -> Result<Vec<Resultat>, ErreurBalayage> {
    todo!()
}

/// Le jeu de la figure 9 : `ρ` (voir « Les trois jeux de balayage »).
#[allow(unused_variables)]
pub fn jeu_rho() -> JeuDeBalayage {
    todo!()
}

/// Le jeu de la figure 10 : `β` (voir « Les trois jeux de balayage »).
#[allow(unused_variables)]
pub fn jeu_beta() -> JeuDeBalayage {
    todo!()
}

/// Le jeu de la figure 12 : `μ` (voir « Les trois jeux de balayage »).
#[allow(unused_variables)]
pub fn jeu_mu() -> JeuDeBalayage {
    todo!()
}

/// Une ligne du CSV pour un cristal (voir « Le CSV ») : `nom` est le nom du balayage, `rayon` le rayon de domaine, `pas` le nombre de pas ; la ligne finit par `\n`.
#[allow(unused_variables)]
pub fn ligne_csv(nom: &str, rayon: u32, pas: u64, resultat: &Resultat) -> String {
    todo!()
}

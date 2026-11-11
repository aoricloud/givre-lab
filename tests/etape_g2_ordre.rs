//! Étape G2 : ce que NOTRE code fait le long de chacun des trois balayages de l'article (`ρ`, figure 9 ; `β`, figure 10 ; `μ`, figure 12), à la taille de l'étape (un domaine de rayon 100 ; 2000, 1300 et 2000 pas : voir
//! `jeu_rho`, `jeu_beta`, `jeu_mu`). **Tous les tests de ce fichier sont des `régression :`** : ce sont des résultats de notre code, figés, pas des valeurs publiées. L'article ne donne AUCUN nombre pour ces
//! figures (des images et des réglages) ; l'étiquette est un modèle à nous (ses seuils sont à nous). Un test dit « l'ordre revient » quand l'ordre que NOS mesures donnent est celui que le texte de l'article décrit
//! (section 5, pages imprimées 21 à 24), et « ne revient pas » quand il ne l'est pas : on n'affirme dans aucun test un ordre publié que notre code ne donne pas. Valeurs : `references/etape-g2-valeurs.md`,
//! identifiants `V-G2-ORD-xx` ; la mesure à d'autres tailles est dans `MODELE.md`.
//!
//! Le calcul de chaque balayage prend de 12 à 24 secondes d'un cœur (profil `dev` à `opt-level = 3`, PC chargé), autant pour les deux cristaux de `β` à 3000 pas : il est fait une seule fois par balayage et partagé entre les tests du fichier.

use givre_lab::balayage::{
    EN_TETE_CSV, Forme, JeuDeBalayage, Resultat, balayage, jeu_beta, jeu_mu, jeu_rho, ligne_csv,
};
use givre_lab::image::ecrire_png;
use std::path::Path;
use std::sync::OnceLock;

/// `(rattachees, rayon, compacite, pointes, aiguilles, hexagonalite, forme)` d'un cristal.
type Cristal = (u32, u32, u32, u32, u32, u32, Forme);

/// Les six cristaux du balayage `rho` à la taille de l'étape : `(rattachees, rayon, compacite, pointes, aiguilles, hexagonalite, forme)`, mesurés sur le prototype (régression).
const RHO: [Cristal; 6] = [
    (1687, 26, 800, 18, 0, 500, Forme::Secteurs),
    (1849, 28, 758, 30, 0, 535, Forme::Secteurs),
    (1837, 30, 658, 78, 0, 300, Forme::Dendrite),
    (2227, 35, 588, 78, 0, 314, Forme::Dendrite),
    (2713, 40, 551, 102, 0, 225, Forme::Dendrite),
    (3937, 55, 426, 174, 0, 90, Forme::Dendrite),
];
/// Les six empreintes (`Neige::empreinte`, SHA-256 en hexadécimal) du balayage `rho`, mesurées sur le prototype (régression).
const EMPREINTES_RHO: [&str; 6] = [
    "a686de99063bc8415f56bd9b371b133338725c0ed09772d4fb43a2749e4fd1ce",
    "fc9c7484079a7a977267528c087420d7efbfb7fa6c0af1e5ee181fa1b70c54e7",
    "aae67410a5068ce1614b81d7fbb6db0ed58cf14a08a8e95e6fe4093b14852402",
    "138c5ca88c64ac57bf3f14575616f99a4cdad3e703ace26e5b8f7787c040b580",
    "963ac54fbf73654dde4384b80398753076b4a80291171a8fa5c1d18c84990bb7",
    "6e4420dd225539ffdc2bbf9f27e7f08c4eb3aca4d74a804c28f55fe5ceb10550",
];
/// Les six cristaux du balayage `beta` à la taille de l'étape : `(rattachees, rayon, compacite, pointes, aiguilles, hexagonalite, forme)`, mesurés sur le prototype (régression).
const BETA: [Cristal; 6] = [
    (6679, 62, 569, 234, 12, 241, Forme::Fougere),
    (4153, 41, 803, 78, 0, 414, Forme::Secteurs),
    (3247, 33, 964, 18, 0, 818, Forme::Plaque),
    (2437, 28, 1000, 6, 0, 1000, Forme::Plaque),
    (2107, 26, 1000, 6, 0, 1000, Forme::Plaque),
    (1951, 25, 1000, 6, 0, 1000, Forme::Plaque),
];
/// Les six empreintes (`Neige::empreinte`, SHA-256 en hexadécimal) du balayage `beta`, mesurées sur le prototype (régression).
const EMPREINTES_BETA: [&str; 6] = [
    "c82b34ee42c9b14385529c3a5b40371ec40ecc03f5e9172c15b37487ff89217d",
    "1a8e0b28dbe5c6037cdd9b6e2a537587860bfcf1ef54c53f80fcba8ee8d8a7e6",
    "3083772238b51c9118e0e5b9dd1e8946e0a43cca100218d8aa21d1c20e2a7ec1",
    "b00de67b6e4efc970a2c45b22951c5a90306f004e74f0b4a29038f94c72a51b3",
    "28aec1ecfad7b7912776bf26bf98a5f90c2e4be257c78bf2e6e8127ae4dd6eca",
    "2ac8f7b0519a05c59eca0e68b630c203f0fe4d28aa8e6b65137e6b2312ebab0d",
];
/// Les six cristaux du balayage `mu` à la taille de l'étape : `(rattachees, rayon, compacite, pointes, aiguilles, hexagonalite, forme)`, mesurés sur le prototype (régression).
const MU: [Cristal; 6] = [
    (4639, 59, 436, 150, 0, 84, Forme::Dendrite),
    (3931, 51, 494, 138, 0, 137, Forme::Dendrite),
    (3277, 44, 551, 102, 0, 204, Forme::Dendrite),
    (2893, 38, 650, 90, 0, 289, Forme::Dendrite),
    (2491, 33, 739, 66, 0, 575, Forme::Secteurs),
    (2209, 30, 791, 30, 0, 566, Forme::Secteurs),
];
/// Les six empreintes (`Neige::empreinte`, SHA-256 en hexadécimal) du balayage `mu`, mesurées sur le prototype (régression).
const EMPREINTES_MU: [&str; 6] = [
    "ce8158b960df9d86dcce2902da74372a69a21ad40d0b33ad757b428258176a51",
    "86fad82129699be440db8ee1103773a6d00ef3ca60e96c1cd6d332162aa9abed",
    "d6d4f95c5e8d0ba14d6cef7e5bc878bbab0ebba179a70a3159364ec286072fe1",
    "d26ee160eae0d7853e21b3a964309a5b154d98514a462fccf3f2c31620bcbeb5",
    "7171d88ef0a843d3f7347b60581761897f7cf6ac0ab205e727f9c74755b9ddc1",
    "5711ce88750526358566799358edf49ff1855f48ded3b7d07a3b708e73b0500a",
];

/// Le balayage d'un jeu, à la taille de l'étape, calculé une seule fois.
fn calculer(jeu: &JeuDeBalayage) -> Vec<Resultat> {
    balayage(jeu.rayon, jeu.pas, jeu.base, jeu.reglage, &jeu.valeurs).expect("balayage valide")
}

fn rho() -> &'static Vec<Resultat> {
    static R: OnceLock<Vec<Resultat>> = OnceLock::new();
    R.get_or_init(|| calculer(&jeu_rho()))
}

fn beta() -> &'static Vec<Resultat> {
    static R: OnceLock<Vec<Resultat>> = OnceLock::new();
    R.get_or_init(|| calculer(&jeu_beta()))
}

fn mu() -> &'static Vec<Resultat> {
    static R: OnceLock<Vec<Resultat>> = OnceLock::new();
    R.get_or_init(|| calculer(&jeu_mu()))
}

/// Les deux cristaux de `β = 2,2` et `β = 2,4` à un temps plus long (3000 pas, un domaine de rayon 130), calculés une seule fois. **Ils sont loin du mur** : ils ont exactement les mêmes cases rattachées dans un domaine de rayon 150
/// (mesuré par le juge de l'étape ; dans un domaine de rayon 110 le cristal de `β = 2,2`, de rayon 79, sent déjà le mur : 12 133 cases au lieu de 12 169). Le cristal de `β = 1,9` n'est pas rejoué à 3000 pas : il y atteint
/// 136 cases de rayon et demanderait un domaine de plus de 190 cases de rayon (des dizaines de secondes).
fn beta_long() -> &'static Vec<Resultat> {
    static R: OnceLock<Vec<Resultat>> = OnceLock::new();
    R.get_or_init(|| {
        let jeu = jeu_beta();
        balayage(130, 3000, jeu.base, jeu.reglage, &jeu.valeurs[1..3]).expect("balayage valide")
    })
}

/// Les cristaux de `β = 2,2` et `β = 2,4` à 130 × 3000 : `(rattachees, rayon, compacite, pointes, aiguilles, hexagonalite, forme)`, mesurés sur le prototype (régression).
const BETA_LONG: [Cristal; 2] =
    [(12169, 79, 641, 198, 0, 265, Forme::Dendrite), (9265, 61, 816, 90, 0, 573, Forme::Secteurs)];

fn formes(resultats: &[Resultat]) -> Vec<Forme> {
    resultats.iter().map(|r| r.forme).collect()
}

fn rayons(resultats: &[Resultat]) -> Vec<u32> {
    resultats.iter().map(|r| r.mesures.rayon).collect()
}

fn compacites(resultats: &[Resultat]) -> Vec<u32> {
    resultats.iter().map(|r| r.mesures.compacite).collect()
}

fn pointes(resultats: &[Resultat]) -> Vec<u32> {
    resultats.iter().map(|r| r.mesures.pointes).collect()
}

fn strictement_croissante(v: &[u32]) -> bool {
    v.windows(2).all(|w| w[0] < w[1])
}

fn strictement_decroissante(v: &[u32]) -> bool {
    v.windows(2).all(|w| w[0] > w[1])
}

fn croissante(v: &[u32]) -> bool {
    v.windows(2).all(|w| w[0] <= w[1])
}

fn decroissante(v: &[u32]) -> bool {
    v.windows(2).all(|w| w[0] >= w[1])
}

fn comparer(nom: &str, resultats: &[Resultat], mesures: &[Cristal; 6], empreintes: &[&str; 6]) {
    assert_eq!(resultats.len(), 6, "{nom}");
    for (k, (r, attendu)) in resultats.iter().zip(mesures).enumerate() {
        let m = &r.mesures;
        let obtenu = (m.rattachees, m.rayon, m.compacite, m.pointes, m.aiguilles, m.hexagonalite, r.forme);
        assert_eq!(obtenu, *attendu, "{nom}, cristal {k} : mesures et forme");
        assert_eq!(r.empreinte, empreintes[k], "{nom}, cristal {k} : empreinte");
    }
}

/// régression : les mesures, les étiquettes et les empreintes des dix-huit cristaux des trois balayages, tels que le prototype les donne (V-G2-ORD-01). **Ce n'est pas une valeur publiée** : les nombres viennent
/// de notre code (le modèle de G1 et les mesures de G2, en entiers, les mêmes sous Linux et sous Windows). Les empreintes sont celles de G1 (`Neige::empreinte` d'après les `76 + 25 * cases` octets de l'écriture
/// canonique de G1) : une main qui aurait changé la neige de G1 les ferait bouger ; les mesures, elles, ne bougent que si le calcul des mesures change. Valeurs de la table : celles de `apercus/etape-g2-mesures.csv`.
///
/// Ne couvre pas : que ces nombres soient justes (voir les tests `juste`) ni qu'ils ressemblent à l'article (aucun nombre publié n'existe) ; leur stabilité si le modèle de G1 changeait (ce serait une autre étape).
#[test]
#[cfg_attr(not(feature = "etape-g2"), ignore)]
fn les_dix_huit_cristaux_tels_que_le_prototype_les_donne() {
    comparer("rho", rho(), &RHO, &EMPREINTES_RHO);
    comparer("beta", beta(), &BETA, &EMPREINTES_BETA);
    comparer("mu", mu(), &MU, &EMPREINTES_MU);
}

/// régression : le long du balayage de `ρ` (0,40 à 0,50 ; réglages de la ligne « 9 »), le rayon atteint en 2000 pas croît strictement (26, 28, 30, 35, 40, 55), la compacité décroît strictement (800 à 426) et le nombre
/// de pointes ne décroît jamais (18 à 174) (V-G2-ORD-02). **L'ordre revient** : l'article dit (figure 9, page imprimée 21) que les cristaux croissent plus vite quand `ρ` augmente (le rayon à pas égal est la vitesse)
/// et que la forme passe de la plaque à la plaque à secteurs puis à la dendrite ; notre compacité qui baisse et nos pointes qui montent sont ce passage en nombres. Le test affirme ce que NOTRE code donne, pas l'article.
///
/// Ne couvre pas : la plaque du bout de la figure (voir `ce_que_l_article_decrit_et_que_notre_code_ne_donne_pas`) ; d'autres tailles (voir `MODELE.md`, où ces ordres sont mesurés à d'autres tailles) ; **un pas de `ρ` plus fin** : la compacité
/// n'est monotone que sur ces six valeurs (à `ρ = .40, .405, .41, .415` elle vaut 800, 719, 758, 733 : mesuré par le juge) ; que la figure 9 compare des cristaux de taille comparable, et nous des cristaux de même âge.
#[test]
#[cfg_attr(not(feature = "etape-g2"), ignore)]
fn rho_la_croissance_accelere_et_la_forme_se_ramifie() {
    let r = rho();
    assert!(strictement_croissante(&rayons(r)), "rayons : {:?}", rayons(r));
    assert!(strictement_decroissante(&compacites(r)), "compacités : {:?}", compacites(r));
    assert!(croissante(&pointes(r)), "pointes : {:?}", pointes(r));
    assert_eq!(
        formes(r),
        [Forme::Secteurs, Forme::Secteurs, Forme::Dendrite, Forme::Dendrite, Forme::Dendrite, Forme::Dendrite]
    );
}

/// régression : le long du balayage de `β` (1,9 à 2,8 ; réglages de la ligne « 10 », `ρ = 0,8`), le rayon atteint en 1300 pas décroît strictement (62, 41, 33, 28, 26, 25), la compacité ne décroît jamais (569 à 1000) et le nombre de
/// pointes ne croît jamais (234 à 6) ; les étiquettes vont de `Fougere` à `Plaque` en passant par `Secteurs` (V-G2-ORD-03). **L'ordre revient en partie** : l'article dit (figure 10, page imprimée 22) que `β` de 1,9 à 2,8 mène de
/// la fougère à la dendrite puis à la plaque à secteurs, que la plaque hexagonale centrale grandit et que la propension à la ramification latérale baisse : la ramification qui baisse (les pointes) et la plaque qui grandit (la
/// compacité) reviennent, la fougère aussi (grâce aux aiguilles) ; la dendrite intermédiaire n'est pas une étiquette de ce balayage à 1300 pas (voir `ce_que_l_article_decrit_et_que_notre_code_ne_donne_pas` ; elle apparaît à 3000 pas, et, à 1300 pas, à `β = 2,0`, qui n'est pas une valeur de la figure : `MODELE.md`).
///
/// Ne couvre pas : l'ordre des formes à la taille de l'article (400 à 600 cases) ; d'autres tailles.
#[test]
#[cfg_attr(not(feature = "etape-g2"), ignore)]
fn beta_la_plaque_centrale_grandit_et_la_ramification_baisse() {
    let r = beta();
    assert!(strictement_decroissante(&rayons(r)), "rayons : {:?}", rayons(r));
    assert!(croissante(&compacites(r)), "compacités : {:?}", compacites(r));
    assert!(decroissante(&pointes(r)), "pointes : {:?}", pointes(r));
    assert_eq!(
        formes(r),
        [Forme::Fougere, Forme::Secteurs, Forme::Plaque, Forme::Plaque, Forme::Plaque, Forme::Plaque]
    );
}

/// régression : à un temps plus long (3000 pas, un domaine de rayon 130), les étiquettes de `β = 2,2` et de `β = 2,4` ne sont plus celles de 1300 pas (V-G2-ORD-08) : `β = 2,2`, `Secteurs` à 1300 pas (compacité 803), est une `Dendrite`
/// (rayon 79, compacité 641, 198 pointes) ; `β = 2,4`, `Plaque` à 1300 pas (compacité 964), est `Secteurs` (compacité 816). **L'étiquette dépend du temps** : la dendrite que la figure 10 montre à `β = 2.2` et qui manque à 1300 pas
/// (voir `ce_que_l_article_decrit_et_que_notre_code_ne_donne_pas`) apparaît quand le cristal a grandi. Les deux valeurs sont la deuxième et la troisième de `jeu_beta` (le test le vérifie) ; les nombres viennent du prototype ; les deux
/// cristaux sont loin du mur (voir `beta_long`). Ce test ne dit pas que la suite entière de la figure 10 revient à 3000 pas : la fougère de `β = 1,9` et les trois `β` forts n'y sont pas rejoués (mesurés à part : `MODELE.md`).
///
/// Ne couvre pas : la taille de l'article ; les empreintes (non figées pour cette taille) ; les quatre autres valeurs de `β` à 3000 pas.
#[test]
#[cfg_attr(not(feature = "etape-g2"), ignore)]
fn beta_a_un_temps_plus_long_retrouve_la_dendrite() {
    let r = beta_long();
    assert_eq!(r.len(), 2);
    let jeu = jeu_beta();
    assert_eq!((r[0].valeur, r[1].valeur), (jeu.valeurs[1], jeu.valeurs[2]));
    for (k, (x, attendu)) in r.iter().zip(&BETA_LONG).enumerate() {
        let m = &x.mesures;
        let obtenu = (m.rattachees, m.rayon, m.compacite, m.pointes, m.aiguilles, m.hexagonalite, x.forme);
        assert_eq!(obtenu, *attendu, "beta 130 x 3000, cristal {k}");
    }
    assert_eq!(formes(r), [Forme::Dendrite, Forme::Secteurs]);
    // Les mêmes valeurs à 1300 pas : d'autres étiquettes.
    assert_eq!(formes(&beta()[1..3]), [Forme::Secteurs, Forme::Plaque]);
}

/// régression : le long du balayage de `μ` (0,04 à 0,09 ; réglages de la ligne « 12 »), le rayon atteint en 2000 pas décroît strictement (59 à 30), la compacité croît strictement (436 à 791), le nombre de pointes décroît strictement (150, 138, 102, 90, 66, 30), l'hexagonalité des pointes du dernier
/// cristal est plus de quatre fois celle du premier (566 contre 84), et les étiquettes vont de `Dendrite` (quatre fois) à `Secteurs` (deux fois) (V-G2-ORD-04). **L'ordre revient** : l'article dit (figure 12, pages imprimées 23 et 24) que
/// la fonte favorise les facettes, que les pointes paraboliques de `μ = .04` et `.05` laissent la place à des pointes hexagonales à mesure que `μ` croît, et que le dernier cristal (`μ = .09`) est une « stellar plate » (une plaque étoilée, que notre étiquette range `Secteurs` : elle n'a pas d'autre classe pour elle) ; le passage
/// de la dendrite aux secteurs est à `μ = .08` (l'article dit qu'une frontière de classification est arbitraire à cet endroit). **Mais l'hexagonalité n'est pas monotone** (575 à `μ = .08`, 566 à `.09`) : le test ne l'affirme pas.
///
/// Ne couvre pas : la finesse des pointes (parabolique ou hexagonale : une image, pas une mesure de ce fichier) ; d'autres tailles.
#[test]
#[cfg_attr(not(feature = "etape-g2"), ignore)]
fn mu_les_pointes_deviennent_hexagonales_et_le_cristal_se_remplit() {
    let r = mu();
    assert!(strictement_decroissante(&rayons(r)), "rayons : {:?}", rayons(r));
    assert!(strictement_croissante(&compacites(r)), "compacités : {:?}", compacites(r));
    assert!(strictement_decroissante(&pointes(r)), "pointes : {:?}", pointes(r));
    assert!(
        r[5].mesures.hexagonalite > 4 * r[0].mesures.hexagonalite,
        "hexagonalité : {} contre {}",
        r[5].mesures.hexagonalite,
        r[0].mesures.hexagonalite
    );
    assert_eq!(
        formes(r),
        [Forme::Dendrite, Forme::Dendrite, Forme::Dendrite, Forme::Dendrite, Forme::Secteurs, Forme::Secteurs]
    );
}

/// régression : ce que l'article décrit et que NOTRE code, à cette taille, ne donne PAS (V-G2-ORD-05). (a) La figure 9 montre, à `ρ = .40`, une plaque : aucun cristal du balayage de `ρ` n'est une `Plaque`
/// (le premier est `Secteurs`, de compacité 800 : une plaque à secteurs aux concavités remplies mince par l'instabilité en lame de couteau). (b) La figure 10 va de la fougère à la dendrite puis à la plaque à secteurs :
/// aucun cristal du balayage de `β` n'est une `Dendrite` à 1300 pas (elle apparaît à 3000 pas : `beta_a_un_temps_plus_long_retrouve_la_dendrite` ; à 1300 pas la suite est `Fougere`, `Secteurs`, puis des hexagones pleins de compacité 964 à 1000 : à ce stade précoce, les `β` forts n'ont pas encore ramifié, ce que l'article
/// décrit comme une plaque hexagonale centrale qui grandit avant que les branches se développent). (c) Les trois derniers cristaux de `β` (2,6, 2,7, 2,8) ont une compacité de 1000 : ce sont des hexagones pleins, que la figure
/// 10 montre à plus grande taille avec des bras et des plaques à secteurs ; **notre classement ne les sépare pas** (il les range tous `Plaque`). (d) Aucun cristal n'a l'étiquette `Aucune` : ils ont tous plus de 10 cases de rayon.
/// Ces constats sont les NÔTRES, à NOTRE taille : ils ne disent pas que l'article se trompe.
///
/// Ne couvre pas : la raison de ces écarts (la taille et le temps, le mur, nos seuils) ; ce que donnerait un balayage à la taille de l'article.
#[test]
#[cfg_attr(not(feature = "etape-g2"), ignore)]
fn ce_que_l_article_decrit_et_que_notre_code_ne_donne_pas() {
    assert!(formes(rho()).iter().all(|&f| f != Forme::Plaque), "{:?}", formes(rho()));
    assert!(formes(beta()).iter().all(|&f| f != Forme::Dendrite), "{:?}", formes(beta()));
    let b = beta();
    assert!(b[3..].iter().all(|r| r.mesures.compacite == 1000 && r.forme == Forme::Plaque));
    for r in rho().iter().chain(beta()).chain(mu()) {
        assert!(r.mesures.rayon >= 10 && r.forme != Forme::Aucune);
    }
}

/// régression : l'aiguille (une case rattachée qui a au plus deux voisines rattachées) ne sépare qu'UN cristal des dix-huit (V-G2-ORD-07) : celui de `β = 1,9` en a 12 (deux par secteur), les dix-sept autres 0. C'est
/// ce qui fait la règle 4 de l'étiquette (`Fougere` : au moins six aiguilles) ; **parmi les dix-huit, la coupure fougère / dendrite repose donc sur ce seul cristal** : c'est le résultat, pas une règle établie (hors des dix-huit, à la même taille : 24 aiguilles à `β = 1,8`, 36 à `β = 1,7`, aucune à `β = 2,0`, et 12 à `μ = 0,03` : mesuré par le juge, `MODELE.md`). La mesure à d'autres tailles
/// (`MODELE.md`, « Les seuils de l'étiquette ») donne à `β = 1,9` 12, 24 ou 36 aiguilles et 0 aux autres cristaux, dans les huit tailles essayées. L'article dit (figure 10) que `β` bas donne la fougère : `β` est le seuil de masse de bord
/// pour se rattacher avec une ou deux voisines rattachées, donc un `β` bas laisse naître des branches d'une case de large.
///
/// Ne couvre pas : que les aiguilles soient la bonne mesure d'une fougère (une fougère de l'article a bien plus de ramifications) ; d'autres cristaux.
#[test]
#[cfg_attr(not(feature = "etape-g2"), ignore)]
fn l_aiguille_ne_separe_que_beta_un_virgule_neuf() {
    let aiguilles = |r: &[Resultat]| r.iter().map(|x| x.mesures.aiguilles).collect::<Vec<u32>>();
    assert_eq!(aiguilles(rho()), [0, 0, 0, 0, 0, 0]);
    assert_eq!(aiguilles(beta()), [12, 0, 0, 0, 0, 0]);
    assert_eq!(aiguilles(mu()), [0, 0, 0, 0, 0, 0]);
}

/// régression : les fichiers que l'exemple `apercus_etape_g2` écrit sont cohérents avec la bibliothèque (V-G2-ORD-09). `apercus/etape-g2-mesures.csv` est, **octet pour octet**, `EN_TETE_CSV` suivi des dix-huit `ligne_csv` des trois
/// balayages à la taille de l'étape (dans l'ordre `rho`, `beta`, `mu`) ; chacune des trois planches `apercus/etape-g2-rho.png`, `etape-g2-beta.png`, `etape-g2-mu.png` est un PNG de 900 x 600 pixels (signature et en-tête `IHDR` lus) de la longueur
/// exacte d'un PNG de cette taille écrit par `image::ecrire_png` (non compressé : `3 * 900 * 600` octets de pixels plus l'en-tête). **Ces fichiers ne sont pas dans `main` avant la vague du code** : ce test les attend (il est ignoré sans la fonctionnalité).
///
/// Ne couvre pas : que les images soient les bonnes ni qu'elles soient belles (ce que la doc laisse libre se contrôle à l'œil et par `sha256` contre le prototype : leçon 20) ; ni que le fichier soit celui du prototype (le `sha256` se compare à part).
#[test]
#[cfg_attr(not(feature = "etape-g2"), ignore)]
fn les_fichiers_de_l_exemple_sont_ceux_de_la_bibliotheque() {
    let racine = Path::new(env!("CARGO_MANIFEST_DIR")).join("apercus");
    let mut attendu = String::from(EN_TETE_CSV);
    for (jeu, resultats) in [(jeu_rho(), rho()), (jeu_beta(), beta()), (jeu_mu(), mu())] {
        assert_eq!(resultats.len(), 6);
        for r in resultats {
            attendu.push_str(&ligne_csv(jeu.nom, jeu.rayon, jeu.pas, r));
        }
    }
    let csv = std::fs::read_to_string(racine.join("etape-g2-mesures.csv")).expect("apercus/etape-g2-mesures.csv");
    assert_eq!(csv, attendu);
    assert_eq!(csv.lines().count(), 19);
    let longueur = ecrire_png(900, 600, &vec![255_u8; 3 * 900 * 600]).len();
    for nom in ["etape-g2-rho.png", "etape-g2-beta.png", "etape-g2-mu.png"] {
        let octets = std::fs::read(racine.join(nom)).unwrap_or_else(|e| panic!("{nom} : {e}"));
        assert_eq!(octets.len(), longueur, "{nom} : la longueur d'un PNG non compressé de 900 x 600");
        assert_eq!(&octets[..8], &[0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A], "{nom} : la signature");
        assert_eq!(u32::from_be_bytes([octets[16], octets[17], octets[18], octets[19]]), 900, "{nom} : la largeur");
        assert_eq!(u32::from_be_bytes([octets[20], octets[21], octets[22], octets[23]]), 600, "{nom} : la hauteur");
    }
}

/// régression : le mur est loin du cristal à cette taille (V-G2-ORD-06). Pour chacun des dix-huit cristaux, le rayon du cristal est au plus 62 % du rayon du domaine de son jeu (100) : le plus grand est de 62 (`β = 1,9`). **À cette distance le mur ne
/// compte pas** : chacun des dix-huit cristaux a exactement les mêmes cases rattachées, donc les mêmes mesures, dans un domaine de rayon 120 (mesuré par le juge de l'étape avec un outil à part : `MODELE.md`, « Le mur » ; ce test ne le rejoue pas,
/// il doublerait le coût). Dans un domaine de rayon 84, deux des dix-huit sentaient déjà le mur (`β = 1,9`, à 73,8 % : 6643 cases au lieu de 6679 ; `μ = 0,05`, à 60,7 % : 3943 au lieu de 3931) : un rapport de rayons ne suffit pas à le dire,
/// c'est la comparaison à un domaine plus grand qui le dit.
///
/// Ne couvre pas : l'égalité avec un domaine plus grand (mesurée à part) ; l'effet du mur à d'autres tailles ou pour d'autres réglages.
#[test]
#[cfg_attr(not(feature = "etape-g2"), ignore)]
fn le_cristal_n_atteint_pas_le_mur() {
    // Un compte vide n'est pas un compte vert : dix-huit cristaux, sinon le test serait à vide.
    assert_eq!(rho().len() + beta().len() + mu().len(), 18);
    for (jeu, resultats) in [(jeu_rho(), rho()), (jeu_beta(), beta()), (jeu_mu(), mu())] {
        assert_eq!(jeu.rayon, 100, "{}", jeu.nom);
        for r in resultats {
            assert!(100 * r.mesures.rayon <= 62 * jeu.rayon, "{} : {:?}", jeu.nom, r.mesures);
        }
    }
}

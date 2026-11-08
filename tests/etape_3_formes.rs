//! Étape 3 : les classes de formes (plaques, étoiles, secteurs, dendrites) retrouvées aux paramètres de Reiter, et la vitesse radiale. Ce sont des tests
//! `régression :` **mesurés** (ce ne sont pas des preuves : un résultat de NOTRE code, avec sa tolérance mesurée, que les formes des vignettes de l'auteur appuient
//! à l'œil) : chaque intervalle a été mesuré sur le prototype, sur 3 tailles de région (85, 100, 120) et 7 rayons de cristal (20 à 50) ; le modèle n'a
//! aucun hasard, donc une implémentation fidèle à la doc donne exactement les mêmes nombres, et l'intervalle ne dit pas « la graine qui passe » mais
//! « la classe de forme ». Valeurs attendues : `references/etape-3-valeurs.md`, identifiants `V-FORM-xx`.
//!
//! **Les paramètres et leur source** (Reiter 2005, préimpression et page auxiliaire, lues le 8/11/2026) : plaque : `β = 0`, `γ > 0` (« When β = 0 and
//! γ > 0, plates form, the rate depending upon γ », exemple de la page : `β = 0, γ = 1`) ; plaques et secteurs à `β` élevé (« Regardless of γ, at high
//! levels of β, we get plates, sectors or broad stellar forms ») ; dendrites : `β = 0.35, γ = 0` ; `β = 0.4, γ = 0.001` ; `β = 0.4, γ = 0.0001` (les
//! exemples de la page auxiliaire et de la figure 5, « they all exhibit dendrite growth ») ; étoiles : « for high levels of γ we obtain stellar or more
//! abstract forms » : les jeux `(0.45, 0.1)` et `(0.5, 0.05)` de la table publiée. Les vignettes de la table (images lues le 8/11/2026,
//! `th_nn_h0_x0o05_01699.png` : hexagone plein ; `h0o9_x0o1` : hexagone plein ; `h0o45_x0o1` : étoile pleine à six pointes ; `h0o5_x0o05` : étoile à six
//! pointes ; `h0o35_x0` : dendrite fine ; `h0o4_x0o001` : dendrite épaisse ; `h0o95_x0` : secteur à motif fractal ; `h0o8_x0o002` : six pétales fins)
//! montrent les mêmes formes que le prototype aux mêmes paramètres. **Ce que je n'ai pas pu lire** : les figures 3 à 5 de la préimpression (des images
//! sans texte) : les paramètres de la figure 4 (« thin dendrite, thick dendrite, nearly collapsed into a stellar form, abstract sector form ») ne sont
//! pas lus ; la classe « étoile » est fondée sur la vignette et le texte, non sur un jeu nommé par l'auteur. Les `α` au-dessus de 2 (2.003, 2.69 de
//! la figure 5) sont hors du modèle documenté (`α <= 2`).

mod etape_3_commun;

use etape_3_commun::parametres;
use givre_lab::flocon::Flocon;
use std::sync::OnceLock;

/// Ce qu'on relève au premier pas où le rayon du cristal atteint un jalon.
#[derive(Clone, Copy, Debug, PartialEq)]
struct Releve {
    rayon: u32,
    temps: u64,
    gelees: usize,
    compacite: f64,
    pointes: usize,
}

/// Fait avancer un flocon et relève les mesures à chaque jalon (le rayon du cristal croît d'au plus 1 par pas : le jalon est atteint pile). Rend aussi
/// la trajectoire `(temps, rayon du cristal)` à chaque pas jusqu'au dernier jalon.
fn suivre(region: u32, alpha: &str, beta: &str, gamma: &str, jalons: &[u32]) -> (Vec<Releve>, Vec<(u64, u32)>) {
    let mut f = Flocon::new(region, parametres(alpha, beta, gamma));
    let mut releves = Vec::new();
    let mut trajectoire = vec![(0, 0)];
    let mut faits: u64 = 0;
    for &jalon in jalons {
        while f.rayon_cristal() < jalon {
            // le plus long des parcours légitimes (α = 0.5, rayon 50) prend 2064 pas : un code qui ne fait pas grandir le cristal échoue ici, il ne
            // boucle pas (les pas sont comptés ici, pas par `temps`)
            assert!(
                faits < 4000,
                "le rayon du cristal n'atteint pas {jalon} en 4000 pas (β = {beta}, γ = {gamma}, α = {alpha})"
            );
            f.pas();
            faits += 1;
            assert_eq!(f.temps(), faits, "temps compte les pas");
            trajectoire.push((f.temps(), f.rayon_cristal()));
        }
        assert_eq!(f.rayon_cristal(), jalon, "le rayon du cristal croît d'au plus 1 par pas");
        releves.push(Releve {
            rayon: jalon,
            temps: f.temps(),
            gelees: f.gelees(),
            compacite: f.compacite().vers_f64(),
            pointes: f.pointes(),
        });
    }
    (releves, trajectoire)
}

/// Les rayons de cristal où l'on relève les mesures. Les intervalles de classe ont été mesurés de 20 à 50 (les sept premiers) ; le rayon 55 ne sert qu'à
/// la trajectoire de la vitesse radiale.
const JALONS: [u32; 8] = [20, 25, 30, 35, 40, 45, 50, 55];
const JALONS_DE_CLASSE: usize = 7;

type Simulation = (Vec<Releve>, Vec<(u64, u32)>);

/// Les sept classes de `CLASSES` simulées une seule fois pour tout le fichier (région de rayon 85, `α = 1`, jalons `JALONS`) : le relevé à chaque jalon et la
/// trajectoire du rayon du cristal. Les tests qui en ont besoin la partagent (un seul calcul, sept fils d'exécution).
fn simulations() -> &'static Vec<Simulation> {
    static CACHE: OnceLock<Vec<Simulation>> = OnceLock::new();
    CACHE.get_or_init(|| {
        std::thread::scope(|s| {
            let taches: Vec<_> =
                CLASSES.iter().map(|c| s.spawn(move || suivre(85, "1", c.beta, c.gamma, &JALONS))).collect();
            taches.into_iter().map(|t| t.join().unwrap()).collect()
        })
    })
}

/// La pente et le R² de la droite des moindres carrés `rayon = a t + b` sur les points de la trajectoire dont le rayon est dans `[bas, haut]` (formules
/// des moindres carrés, `+ - * /` seulement).
fn droite(trajectoire: &[(u64, u32)], bas: u32, haut: u32) -> (f64, f64) {
    let points: Vec<(f64, f64)> = trajectoire
        .iter()
        .filter(|&&(_, r)| (bas..=haut).contains(&r))
        .map(|&(t, r)| (t as f64, f64::from(r)))
        .collect();
    let n = points.len() as f64;
    let (sx, sy): (f64, f64) = points.iter().fold((0.0, 0.0), |(a, b), &(x, y)| (a + x, b + y));
    let sxx: f64 = points.iter().map(|&(x, _)| x * x).sum();
    let sxy: f64 = points.iter().map(|&(x, y)| x * y).sum();
    let syy: f64 = points.iter().map(|&(_, y)| y * y).sum();
    let pente = (n * sxy - sx * sy) / (n * sxx - sx * sx);
    let r2 = (n * sxy - sx * sy) * (n * sxy - sx * sy) / ((n * sxx - sx * sx) * (n * syy - sy * sy));
    (pente, r2)
}

/// Une classe de forme : un jeu de paramètres et les intervalles mesurés de la compacité (bornes arrondies vers l'extérieur à la deuxième décimale) et du
/// nombre de pointes, à chaque jalon de 20 à 50.
struct Classe {
    nom: &'static str,
    beta: &'static str,
    gamma: &'static str,
    compacite: (f64, f64),
    pointes_min: usize,
    pointes_max: usize,
}

const CLASSES: [Classe; 7] = [
    // plaque : un hexagone plein, exactement (voir `etape_3_exact` : forme close)
    Classe {
        nom: "plaque (β = 0, γ = 0.05)",
        beta: "0",
        gamma: "0.05",
        compacite: (1.0, 1.0),
        pointes_min: 0,
        pointes_max: 0,
    },
    // plaque à vapeur élevée : un anneau par pas, un hexagone plein (vignette `h0o9_x0o1`)
    Classe {
        nom: "plaque à vapeur élevée (β = 0.9, γ = 0.1)",
        beta: "0.9",
        gamma: "0.1",
        compacite: (1.0, 1.0),
        pointes_min: 0,
        pointes_max: 0,
    },
    // étoile pleine à six pointes : mesuré 0.7764 à 0.7914, pointes 6 partout
    Classe {
        nom: "étoile pleine (β = 0.45, γ = 0.1)",
        beta: "0.45",
        gamma: "0.1",
        compacite: (0.77, 0.80),
        pointes_min: 6,
        pointes_max: 6,
    },
    // étoile à bord frangé : mesuré 0.7017 à 0.7420, pointes 18 à 78
    Classe {
        nom: "étoile frangée (β = 0.5, γ = 0.05)",
        beta: "0.5",
        gamma: "0.05",
        compacite: (0.70, 0.75),
        pointes_min: 6,
        pointes_max: 200,
    },
    // secteur à motif fractal : mesuré 0.5485 à 0.6290
    Classe {
        nom: "secteur (β = 0.95, γ = 0)",
        beta: "0.95",
        gamma: "0",
        compacite: (0.54, 0.63),
        pointes_min: 0,
        pointes_max: 500,
    },
    // dendrite épaisse : mesuré 0.2911 à 0.3577, pointes 18 à 114
    Classe {
        nom: "dendrite (β = 0.4, γ = 0.001)",
        beta: "0.4",
        gamma: "0.001",
        compacite: (0.29, 0.36),
        pointes_min: 12,
        pointes_max: 500,
    },
    // dendrite fine : mesuré 0.2025 à 0.2958, pointes 6 à 198
    Classe {
        nom: "dendrite fine (β = 0.35, γ = 0)",
        beta: "0.35",
        gamma: "0",
        compacite: (0.20, 0.30),
        pointes_min: 6,
        pointes_max: 500,
    },
];

/// régression : chaque classe de forme (V-FORM-01) tombe dans l'intervalle mesuré de la compacité (gelées / cases de l'hexagone du rayon du cristal) et des
/// pointes (cases gelées à un seul voisin gelé) **à chacun des sept rayons de cristal de 20 à 50**, sur la région de rayon 85, `α = 1` : plaques (compacité
/// exactement 1, aucune pointe), étoile pleine (compacité 0.77 à 0.80 et **exactement 6 pointes**), étoile frangée (0.70 à 0.75), secteur (0.54 à 0.63),
/// dendrites (0.29 à 0.36 et 0.20 à 0.30, au moins 12 et 6 pointes). Les sept intervalles sont disjoints deux à deux sauf les deux plaques et les deux
/// dendrites qui se recouvrent (0.29 - 0.30) : la séparation plaque, étoile, secteur, dendrite se voit à la compacité (1, 0.7 à 0.8, 0.54 à 0.63, 0.2 à
/// 0.36). Mesure : `scan_formes.txt` et `scan2.txt` des notes de la session (3 régions × 7 rayons, identiques d'une région à l'autre).
///
/// Ne couvre pas : la ressemblance visuelle (l'aperçu est pour l'œil) ; le rayon du cristal au-delà de 50 ; un `α` autre que 1 (voir `dendrite_pour_plusieurs_alpha`).
#[test]
#[cfg_attr(not(feature = "etape-3"), ignore)]
fn classes_de_formes() {
    for (classe, (releves, _)) in CLASSES.iter().zip(simulations()) {
        for r in &releves[..JALONS_DE_CLASSE] {
            let (bas, haut) = classe.compacite;
            assert!(
                (bas..=haut).contains(&r.compacite),
                "{} au rayon {} : compacité {} hors de [{bas}, {haut}]",
                classe.nom,
                r.rayon,
                r.compacite
            );
            assert!(
                (classe.pointes_min..=classe.pointes_max).contains(&r.pointes),
                "{} au rayon {} : {} pointes hors de [{}, {}]",
                classe.nom,
                r.rayon,
                r.pointes,
                classe.pointes_min,
                classe.pointes_max
            );
        }
    }
}

/// régression : l'ordre des classes (V-FORM-02) : à rayon de cristal égal (40 et 50), la compacité de la plaque (1) dépasse celle de l'étoile pleine, qui
/// dépasse celle du secteur, qui dépasse celle de la dendrite épaisse, qui dépasse celle de la dendrite fine ; et le nombre de pointes de la dendrite
/// fine dépasse celui de l'étoile pleine (6). Mesuré : plaque 1, étoile 0.789 / 0.791, secteur 0.615 / 0.614, dendrite 0.309 / 0.291, dendrite fine
/// 0.227 / 0.203, pointes dendrite fine 138 / 198.
///
/// Ne couvre pas : les écarts entre les classes (les intervalles de `classes_de_formes` les bornent).
#[test]
#[cfg_attr(not(feature = "etape-3"), ignore)]
fn ordre_des_classes() {
    // plaque, étoile pleine, secteur, dendrite, dendrite fine : les indices de `CLASSES`
    let jeux = [0, 2, 4, 5, 6];
    for k in [4, 6] {
        let releves: Vec<&Releve> = jeux.iter().map(|&i| &simulations()[i].0[k]).collect();
        let c: Vec<f64> = releves.iter().map(|r| r.compacite).collect();
        assert!(c[0] > c[1] && c[1] > c[2] && c[2] > c[3] && c[3] > c[4], "compacités au jalon {} : {c:?}", JALONS[k]);
        assert_eq!(c[0], 1.0);
        assert!(releves[4].pointes > releves[1].pointes);
        assert_eq!(releves[1].pointes, 6);
    }
}

/// régression : la vitesse radiale (V-FORM-03). Pour chaque jeu, la pente `a` du rayon du cristal contre le temps (moindres carrés, `rayon = a t + b`) sur la
/// fenêtre de pas où le rayon du cristal va de 30 à 55 puis de 20 à 50, de 25 à 55 et de 30 à 50 (quatre fenêtres), tombe dans l'intervalle mesuré, et le `R²` de
/// la droite dépasse 0.995. Mesuré sur 8 fenêtres : plaque `β = 0`, `γ = 0.05` : 0.049887 à 0.049962 (exactement `γ` : un anneau tous les 20 pas, la pente
/// est `γ` à l'effet d'escalier près) ; étoile `(0.45, 0.1)` : 0.249468 à 0.249819 ; dendrite `(0.4, 0.001)` : 0.047511 à 0.047582 ; dendrite fine `(0.35, 0)` :
/// 0.031179 à 0.031226 ; `R²` de 0.99773 à 0.99928 (l'escalier d'un anneau par `m` pas limite le `R²`). Les intervalles sont `[0.99 min, 1.01 max]`. Le
/// rayon du cristal croît à vitesse constante : à rayon 55 la pente de `[30, 55]` est celle de `[20, 45]` à 0,1 % près. Comparaison à la publication (la
/// table de rayon 100 donne la vitesse moyenne jusqu'au rayon 85, transitoire du début comprise) : étoile 85/339 = 0.2507, plaque 85/1700 = 0.05, dendrite
/// `(0.4, 0.001)` 85/1738 = 0.0489 (le prototype : 85/1746 = 0.0487, 0,5 % plus lent : voir l'en-tête de `etape_3_publie`).
///
/// Ne couvre pas : la vitesse au-delà du rayon 55 ; un `α` autre que 1.
#[test]
#[cfg_attr(not(feature = "etape-3"), ignore)]
fn vitesse_radiale() {
    // (indice dans `CLASSES`, β, γ, pente min mesurée, pente max mesurée)
    let jeux = [
        (0, "0", "0.05", 0.049887, 0.049962),
        (2, "0.45", "0.1", 0.249468, 0.249819),
        (5, "0.4", "0.001", 0.047511, 0.047582),
        (6, "0.35", "0", 0.031179, 0.031226),
    ];
    let fenetres = [(30, 55), (20, 50), (25, 55), (30, 50)];
    for (i, beta, gamma, min, max) in jeux {
        let trajectoire = &simulations()[i].1;
        for (bas, haut) in fenetres {
            let (pente, r2) = droite(trajectoire, bas, haut);
            assert!(
                (0.99 * min..=1.01 * max).contains(&pente),
                "β = {beta}, γ = {gamma}, fenêtre [{bas}, {haut}] : pente {pente} hors de [{}, {}]",
                0.99 * min,
                1.01 * max
            );
            assert!(r2 > 0.995, "β = {beta}, γ = {gamma}, fenêtre [{bas}, {haut}] : R² = {r2}");
        }
    }
}

/// régression : la forme ne dépend pas de la taille de la région (V-FORM-04), tant que la couronne est loin du cristal : pour la dendrite `(0.4, 0.001)`, l'étoile
/// `(0.45, 0.1)` et la dendrite fine `(0.35, 0)`, les régions de rayon 85 et 120 donnent, à chacun des rayons de cristal 20 à 50, exactement le même
/// temps, le même nombre de cases gelées, la même compacité et les mêmes pointes (mesuré sur 85, 100 et 120 : écart nul, car la perturbation due au bord est, à cette distance,
/// plus petite que le dernier chiffre). La taille de la région n'a donc pas d'effet sur ces mesures jusqu'au rayon de cristal 50 (ce test ne dit rien
/// du bord circulaire de l'auteur, qui n'est pas dans ce modèle : voir l'en-tête de `etape_3_publie`).
///
/// Ne couvre pas : une région plus petite que le cristal plus 35 cases (là le bord compte : voir la doc de `flocon`).
#[test]
#[cfg_attr(not(feature = "etape-3"), ignore)]
fn independance_de_la_region() {
    // la dendrite, l'étoile pleine, la dendrite fine : les indices de `CLASSES`
    for i in [5, 2, 6] {
        let (beta, gamma) = (CLASSES[i].beta, CLASSES[i].gamma);
        let grande_region = suivre(120, "1", beta, gamma, &JALONS[..JALONS_DE_CLASSE]).0;
        assert_eq!(
            simulations()[i].0[..JALONS_DE_CLASSE],
            grande_region[..],
            "β = {beta}, γ = {gamma} : 85 contre 120"
        );
    }
}

/// régression : « varying α [0.5, 0.501, 0.502, 1] gives very different pictures; however, they all exhibit dendrite growth » (la préimpression, section
/// « Variation on Diffusion », figure 5, `β = 0.4`, `γ = 0.0001` ; V-FORM-05). Pour ces quatre `α`, aux rayons de cristal 35 à 50 : compacité de 0.22 à 0.26
/// (mesuré 0.2291 à 0.2589) et au moins 60 pointes (mesuré au moins 66) : une dendrite ; et au rayon 40 les quatre nombres de cases gelées sont deux à deux
/// différents (mesuré : 1261, 1177, 1201, 1189 : « des images très différentes »). Le temps pour atteindre le rayon 50 est inversement proportionnel à `α`
/// à 5 % près (`α = 0.5` : 2064 pas, `α = 1` : 1058 pas : le rapport 1.951, intervalle retenu 1.90 à 2.00).
///
/// Ne couvre pas : les `α > 2` de la figure (2.003 et 2.69), hors du modèle documenté ; la ressemblance entre `α = 1` et `α = 0.5` que l'article note.
#[test]
#[cfg_attr(not(feature = "etape-3"), ignore)]
fn dendrite_pour_plusieurs_alpha() {
    let alphas = ["0.5", "0.501", "0.502", "1"];
    let mut releves = Vec::new();
    std::thread::scope(|s| {
        let taches: Vec<_> = alphas
            .iter()
            .map(|&alpha| s.spawn(move || suivre(85, alpha, "0.4", "0.0001", &[35, 40, 45, 50]).0))
            .collect();
        for t in taches {
            releves.push(t.join().unwrap());
        }
    });
    for (alpha, r) in alphas.iter().zip(&releves) {
        for m in r {
            assert!((0.22..=0.26).contains(&m.compacite), "α = {alpha} rayon {} : compacité {}", m.rayon, m.compacite);
            assert!(m.pointes >= 60, "α = {alpha} rayon {} : {} pointes", m.rayon, m.pointes);
        }
    }
    let gelees: Vec<usize> = releves.iter().map(|r| r[1].gelees).collect();
    for i in 0..4 {
        for j in i + 1..4 {
            assert_ne!(
                gelees[i], gelees[j],
                "α = {} et {} : même nombre de cases gelées au rayon 40",
                alphas[i], alphas[j]
            );
        }
    }
    let (t_demi, t_un) = (releves[0][3].temps as f64, releves[3][3].temps as f64);
    let rapport = t_demi / t_un;
    assert!(
        (1.90..=2.00).contains(&rapport),
        "temps pour atteindre le rayon 50 : α = 0.5 contre α = 1 : rapport {rapport}"
    );
}

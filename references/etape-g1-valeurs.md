# Étape G1 : les valeurs attendues, leur source, leur tolérance

Cette fiche dit, pour chaque valeur qu'un test de l'étape G1 (la neige de Gravner et Griffeath, le cœur : le modèle sans bruit sur la grille hexagonale, et son rendu net)
attend, d'où elle vient, avec quelle tolérance et quel critère de réussite. Chaque identifiant (`V-G1-…`) est cité dans la doc des tests qui l'emploient
(`tests/etape_g1_*.rs`). Elle est séparée des résultats figés de notre propre code (`references/*.empreinte`, que la vague du code produit) : **aucune valeur « juste » de cette fiche ne vient
du prototype de l'écrivain**. Les seules valeurs tirées de notre code sont celles des tests étiquetés « régression » (V-G1-REG-07, V-G1-RENDU-07), qui ont AUSSI été calculées par
le code Python indépendant. Une valeur vient d'une publication lue (dite lue, ou dite non lue), d'un calcul exact fait par un outil indépendant (nommé), d'un calcul fait à la main (écrit dans
le test) ou d'une convention que la doc de `src/neige.rs` ou de `src/rendu_hex.rs` fixe (dite telle, avec sa raison).

**Outils indépendants** (Python 3.13, dans `essai-g1-outils/` des notes de la session ; le code Rust n'est jamais l'outil) : `g1_ref.py`, écrit d'après la **doc seule** de `src/neige.rs`
et de `src/rendu_hex.rs`, sans lire le code Rust : le modèle en entiers `brut` (diffusion, gel, rattachement, fonte, écriture canonique, SHA-256 par `hashlib`), le même modèle en réels (`decimal`,
70 chiffres : la précision large), le CSV de `lancer` et le rendu net (arrondi cubique, dégradé, moyenne des échantillons). `faire_valeurs.py` écrit les fragments Rust collés dans
`tests/etape_g1_commun/valeurs.rs`. Le prototype Rust et Python donnent les mêmes états, les mêmes empreintes, les mêmes CSV et les mêmes octets d'image sur tous les cas listés ci-dessous.

**Critères écrits avant les calculs** (ce qu'on accepte comme source et comme tolérance) :

1. Les tests « exacts » (états, bits, écriture canonique, empreintes, CSV, octets d'image) n'ont **aucune** tolérance : égalité d'entiers, de chaînes ou d'octets. La doc fixe chaque opération, chaque
   arrondi et leur ordre : une implémentation fidèle donne les mêmes bits.
2. La masse est jugée par une **borne déduite de la formule de l'arrondi**, pas par une tolérance empirique : la division par 7 se trompe d'au plus 3/7 d'unité par case non rattachée et par pas
   (V-G1-MASSE-01) ; quand elle est exacte (vapeurs multiples de 7), la masse est conservée **exactement** (V-G1-MASSE-02).
3. **Aucune forme publiée n'est un test chiffré.** L'article de Gravner et Griffeath ne donne aucun nombre pour ses figures (des images, des réglages) : un test « ressemble à la figure 9 » se réglerait
   pour passer. Les réglages des figures sont des **entrées**, relus à la source (V-G1-01 à V-G1-04) ; la sortie de ces entrées, à petit rayon, est un test « régression » (V-G1-REG-07) qui dit en toutes lettres
   qu'il ne prouve pas la ressemblance.
4. Un jeu de réglages que le texte et l'annexe de l'article ne donnent pas pareil n'est **pas** employé comme celui de l'article (V-G1-03) : les jeux `JEU_B` à `JEU_E` sont des jeux d'essai, dits « à nous ».
5. Les tailles des tests viennent du **coût mesuré** (`MODELE.md`, « Le coût ») : de 3 à 30 de rayon, de 25 à 4000 pas ; toute la suite de G1 tourne en moins de quelques secondes.
6. Le rendu est jugé par des **calculs exacts** sur de petits cas (un hexagone isolé calculé à la main, un code naïf de géométrie exacte), les extrémités et la monotonie du dégradé, la symétrie de
   l'image, les refus, et **un hachage** d'image de référence (régression, avec un second calcul indépendant). Aucune esthétique n'est testée ; les aperçus sont pour l'œil (`AGENTS.md`, règle 3).

**Ce que les tests « juste » ne disent pas** est écrit dans chaque test, ligne `Ne couvre pas :`. Les tests `régression :` de l'étape G1 sont
`etape_g1_regimes.rs::cristal_du_jeu_de_la_figure_9_a_petit_rayon` et `etape_g1_rendu.rs::image_de_reference`.

---

## Les références publiées (lues à la source)

**V-G1-00 : ce qui a été lu.** J. Gravner et D. Griffeath, « Modeling snow crystal growth II: A mesoscopic lattice map with plausible dynamics », *Physica D* 237 (2008) 385-404,
https://doi.org/10.1016/j.physd.2007.09.008. **Lu** : la préimpression de l'auteur, « Revised version, September 2007 » (37 pages, copie `mk.bcgsc.ca/snowflakes/paper/modeling-snow-crystal-growth-2.pdf`), texte extrait
par `pypdf` (les lettres grecques sont conservées) : la section 5 en entier (l'état, le simulateur : les équations (1) à (5), les remarques sur l'implémentation, les paramètres), la section 6 (figure 13, figures 14 à 16 : texte),
l'annexe 8 (« parameter values used in figures »). **Vues** (images extraites du PDF, lues avec l'outil d'image) : la figure 9 (six cristaux, `ρ = .40 .41 .42 .44 .46 .50`) et la figure 13 (trois cristaux).
**NON LU** : la version publiée dans *Physica D* (elle peut différer de la préimpression : une relecture par le juge est à faire si l'accès est possible), les figures 10, 11, 12, 14, 15, 16, 17 (images), les sections 1 à 4 au-delà d'un
survol, l'article en trois dimensions (Gravner et Griffeath, *Phys. Rev. E* 79 (2009) : lu seulement pour les équations de diffusion, qui sont les mêmes ; ses règles d'attachement sont celles de la 3D, pas de cette étape).

**V-G1-01 : le jeu de la figure 9** (`FIG9_RHO_040`, `FIG9_RHO_050`). Source : annexe 8 de la préimpression, ligne « 9 » : `β = 1.3`, `α = .08`, `θ = .025`, `κ = .003`, `μ = .07`, `γ = .00005`, `σ = 0` (la colonne `ρ` est absente
de cette ligne : `ρ` varie) ; `ρ` lu sur la figure 9 elle-même, qui porte `ρ = .40, .41, .42, .44, .46, .50` ; le texte de la section 5 dit « vapor density varying from .4 to .5 and all other parameters identical ». **Recoupement** : la ligne « 15tr »
de l'annexe donne `ρ = .5` et les mêmes six autres valeurs. **Aucune contradiction trouvée** pour ce jeu (le texte et l'annexe ne se contredisent pas, et la figure 15 en copie la ligne). Employé comme entrée de tests ;
sa sortie à petit rayon est V-G1-REG-07. Tolérance : sans objet (une entrée). Critère : les sept décimaux sont lus exactement par `lire_decimal` (`"0.00005"` donne `brut = 214748`).

**V-G1-02 : le jeu du milieu de la figure 13** (`FIG13_MILIEU`, « a stellar crystal with plate ends »). Source : texte de la section 6 : `ρ = .36`, `β = 1.09`, `κ = .0001`, `μ = .14`, `γ = .00001`, `α = .01`, `θ = .0745` ; annexe, ligne « 13m » :
`.36 1.09 .01 .0745 .0001 .14 .00001 0` dans l'ordre `ρ β α θ κ μ γ σ`. **Les deux sources donnent les mêmes valeurs.** La figure demande plus de 100 000 pas : ici ce jeu ne sert que d'entrée à des essais de quelques centaines de pas.

**V-G1-03 : les contradictions de la source** (jeux NON employés comme ceux de l'article). (a) **Figure 13 gauche** (« a simple star ») : le texte dit `α = .026`, `θ = .2`, `γ = .00001` ; l'annexe (ligne « 13l ») dit `α = .2`, `θ = .026`, `γ = .0001` : `α` et `θ`
sont échangés et `γ` diffère d'un facteur 10 ; la ligne « 15bm » (le même cristal, avec un changement au temps 10 000) donne `α = .2`, `θ = .026`, `γ = .00001` (donc l'annexe pour `α` et `θ`, le texte pour `γ`). (b) **Figure 13 droite** (« a plate with dendrite ends ») :
`γ = .00006` dans le texte, `.0006` dans l'annexe (ligne « 13r »). (c) **La prose de l'instabilité en lame de couteau** (section 5, après la figure 10) dit « un seuil de masse de bord `θ < 1` quand la masse diffusive du voisinage passe sous `α` » : c'est l'inverse de
l'équation (3b) (`b >= α` et `Σ d < θ`) ; **le module suit l'équation (3b) et l'ordre des colonnes de l'annexe** ; les valeurs de l'annexe (`α` de 0 à .6, `θ` de 0 à .2) sont cohérentes avec (3b) (`θ` est une somme de vapeur sur sept cases, petite devant `ρ`). (d) La préimpression écrit `n >= 3`
en (3b) et `n >= 4` en (3c), alors que sa prose parle de « 3 voisins » puis de « 4 voisins ou plus » : le résultat est le même (voir la doc de `neige`). (e) L'équation (3d) écrit `b°_t(x) = 0` là où elle veut dire `b'_t(x) = 0`. Ces points sont dits dans `MODELE.md`.
Les jeux `JEU_B` (annexe « 13r », `γ = .0006`), `JEU_D` (annexe « 13l » pour `α` et `θ`, `γ = .00001`), `JEU_C` et `JEU_E` sont des entrées d'essai « à nous » : aucun test n'en tire une affirmation sur la figure 13.

**V-G1-04 : les équations lues** (la spécification de `src/neige.rs`). État `(a, b, c, d)` ; départ « crystal mass 1 at the origin and diffusive mass `ρ` everywhere else » ; (1) diffusion : poids `1/7` sur le centre et chaque voisin, « conditions
réfléchissantes au bord du cristal » (un terme `y` du cristal est remplacé par `d°(x)`) ; (2) gel : `b' = b° + (1 - κ) d°`, `c' = c° + κ d°`, `d' = 0` aux cases de bord ; (3a) `n = 1` ou 2 et `b° >= β` ; (3b) `n >= 3` et (`b° >= 1` ou (`Σ_{y ∈ N_x} d°(y) < θ` et `b° >= α`)) ;
(3c) `n >= 4` ; (3d) `c' = b° + c°`, `b' = 0` ; (4) fonte : `b' = (1 - μ) b°`, `c' = (1 - γ) c°`, `d' = d° + μ b° + γ c°` ; (5) bruit (hors de cette étape) ; « sans bruit, la masse totale est conservée » ; « symétrie exacte dans un hexagone avec raccord ». **Les choix à nous** que l'article ne fixe pas : l'arrondi
(Q31.32, `div_entier(7)`, produit `Fixe`), le mur (réfléchissant, non périodique), la fonte (cases non rattachées seulement : « plus de dynamique aux cases rattachées »), les `d°` de (3b) (après le gel : un `d°` est la valeur au début du sous-pas).

---

## Le modèle (`etape_g1_exact.rs`, `etape_g1_symetrie.rs`, `etape_g1_masse.rs`, `etape_g1_regimes.rs`)

**V-G1-10 : un naïf égal à la bibliothèque, depuis le départ** (`naif_depuis_l_etat_de_depart`). Valeur : tous les `(a, b, c, d)`, la masse, les mesures, à chaque pas, pour neuf trajectoires. Source : le simulateur naïf écrit dans le test d'après la doc (dictionnaire ordonné, `i128`, voisins dans l'ordre inverse).
Tolérance : aucune. Critère : égalité exacte. Le test compte les branches de rattachement prises et exige chacune.

**V-G1-11 : un naïf égal à la bibliothèque, depuis des états sans symétrie** (`naif_depuis_des_etats_sans_symetrie`). Quarante états tirés par un congruentiel linéaire (graines 1 à 40, écrites dans le test). Tolérance : aucune.

**V-G1-12 : le premier pas à la main** (`premier_pas_a_la_main`). Source : calcul fait à la main dans la doc du test (`R = 1`, `ρ = 1/2`, `κ = 1/4`, `μ = γ = 1/2`) : après un pas chaque voisine du germe vaut `(a = 0, b = 3 * 2^28, c = 2^28, d = 2^30)`, la masse `2^31` est conservée,
l'état est stationnaire au pas suivant. Tolérance : aucune.

**V-G1-13 : le mur à la main** (`le_mur_reflechit_a_la_main`). Source : calcul à la main (7000 `brut` en `(1, 0)` de `R = 1` : 4000, 1000, 1000, 1000, masse 7000). Tolérance : aucune.

**V-G1-14 : chaque règle de rattachement une à une** (`regles_de_rattachement_une_a_une`). Source : calcul à la main sur un état à vapeur nulle (19 lignes : `n` de 1 à 6, `b` à l'unité du seuil, `θ` = 0, 1/8, 8). Les résultats (rattachée ou non, `b' = b - ((b + 1) >> 1)`, `d' = (b + 1) >> 1` si elle fond) sont écrits à la main. Tolérance : aucune.

**V-G1-15 : le rattachement est synchrone** (`le_rattachement_est_synchrone`). Source : calcul à la main (deux cases de bord voisines, mêmes deux voisines rattachées, `n = 2`, deux rôles). **V-G1-16 : `θ` est strict, sur la vapeur après le gel** (`theta_est_strict`) : calcul à la main (`T = m = 2^28`, `b2 = α` pile).
Tolérance : aucune.

**V-G1-17 : des trajectoires de Python** (`trajectoires_calculees_par_python`). Huit empreintes, nombres de cases rattachées, rayons, masses totales et masses du cristal calculés par `g1_ref.py` (outil indépendant, d'après la doc seule). Tolérance : aucune.

**V-G1-18 : l'écriture canonique à la main** (`ecriture_canonique_a_la_main`) : 251 octets écrits dans le test. **V-G1-19 : l'état de départ et les accesseurs**. **V-G1-20, V-G1-21 : les refus** de `essayer_new`, de `verifier` et de `essayer_depuis_cases` (chaque borne et le `brut` au-delà). **V-G1-22 : les réglages extrêmes** sans panique, égaux au naïf.
**V-G1-23 : `Parametres::lire`** (les décimaux, les plages). Source : conventions de la doc (les plages et leurs raisons sont dans le tableau de `neige`). Tolérance : aucune.

**V-G1-30 : le cristal de référence des régimes** (`CRISTAL_R30` : `R = 30`, `FIG9_RHO_050`, de 100 à 500 pas : 109, 235, 397, 523, 709 cases ; rayons 6, 10, 13, 16, 19 ; masses du cristal ; empreinte et masse totale à 500 pas). Source : `g1_ref.py`. **V-G1-31 : le scénario** `hexagone 6` + `neige 45 …` (`SCENARIO_R6_*` :
empreinte, CSV à 24 lignes) et le CSV de `neige 7`. Source : `g1_ref.py` (`csv_scenario`). Tolérance : aucune.

**V-G1-24 : de grandes masses** (`naif_avec_de_grandes_masses`, ajouté par le juge) : `R = 2`, vapeurs de `2^51` plus un tirage, masse totale entre `2^55` et `2^56` (le plus grand état permis par la doc) ; source : le naïf du test en `i128` ; aucune tolérance. Raison : la doc promet qu'aucune somme ne déborde jusqu'à `2^56`, et un calcul passé par un `f64` (53 bits) ne se voit qu'à ces tailles.

**V-G1-SC-07 : l'aperçu plafonne la vapeur** (`apercu_plafonne_la_vapeur`, ajouté par le juge) : sept vapeurs posées à la main sur `R = 1` (0, `1 - 2^-32`, 1, `1 + 2^-32`, 2, 1/2, 3/4) ; `niveau = min(255, (brut * 255) >> 32)` calculé à la main : 0, 254, 255, 255, 255, 127, 191. Source : la doc (« L'aperçu »). Aucune tolérance.

**V-G1-SYM-01 à 03 : la symétrie.** Source : la doc (« somme d'entiers ») ; les douze symétries écrites dans le test. Aucune tolérance : le défaut est exactement 0, à chaque pas (V-G1-SYM-01), pour des états sans symétrie par équivariance (V-G1-SYM-02), contre le mur et pour des réglages extrêmes (V-G1-SYM-03).

**V-G1-MASSE-01 à 03 : la masse.** Source : la doc, section « Ce qui est conservé » : `7 |M' - M| <= 3 C`, calculé à partir de la formule de l'arrondi (le reste d'une division par 7 est 0 à 6, l'erreur de l'arrondi au plus proche est `0`, `±1/7`, `±2/7` ou `±3/7`). Tolérance : aucune, c'est une borne déduite. V-G1-MASSE-02 : exacte (égalité) quand les vapeurs sont
des multiples de 7. La **dérive mesurée** (mille fois sous la borne) est dans `MODELE.md`, pas dans un test.

**V-G1-REG-01 à 06 : des propriétés vraies à tout réglage permis**, chacune justifiée par la doc (invariants, permanence, connexité, vitesse d'au plus une case par pas, congruence à 1 modulo 6, rien ne naît sans vapeur, `κ = 1` : le germe ne grandit pas, `κ = 0` : pas de glace hors du cristal, `μ = γ = 0` : la masse gelée ne décroît pas).
**V-G1-REG-07 : régression** du cristal de V-G1-30 (nombres, six pointes, mur loin) : **ce n'est pas la preuve que le cristal ressemble à la figure 9**.

**V-G1-SC-01 à 06 : la directive `neige`**, ses refus (avec le numéro de ligne), les mesures (instants et formats), l'aperçu écrit à la main (`R = 1`, 3 × 3 pixels : `(235, 245, 255)`, `(0, 31, 63)`, `(0, 0, 0)`), le binaire. Source : la doc de la directive ; Python (`g1_ref.py`) pour l'empreinte et le CSV.

---

## Le rendu net (`etape_g1_rendu.rs`)

**V-G1-RENDU-01 : un hexagone isolé à la main.** `R = 1`, une case pleine en `(0, 0)`, `cote = 12`, `rayon_vue = 1`, `sur = 1` : douze pixels `(8, 48, 107)` (lignes 4 et 7 : colonnes 5 et 6 ; lignes 5 et 6 : colonnes 4 à 7), le reste blanc. Source : calcul à la main (les centres de pixels dans l'hexagone à plat de côté 1/2 case = 2 pixels :
`|yc| <= 2` et `(√3/2)|xc| + |yc|/2 <= 2`). Tolérance : aucune.

**V-G1-RENDU-02 : le dessin contre un naïf de géométrie exacte.** Source : un code écrit dans le test (le vrai `√3` en `f64`, la case la plus proche par la distance euclidienne, la moyenne entière), sans l'arrondi cubique ni la fraction rationnelle de la doc ; le test refuse un échantillon à moins de `1e-6` d'une frontière (la doc est exacte à `2,6e-10` près :
`1/√3 ≈ 1239850262 / 2147483647`). Tolérance : aucune (égalité d'octets) sur les cas choisis.

**V-G1-RENDU-03 à 06 : le dégradé, la monotonie, la symétrie de l'image, les refus.** Source : la doc (les cinq arrêts `(255, 255, 255)`, `(198, 219, 239)`, `(107, 174, 214)`, `(33, 113, 181)`, `(8, 48, 107)` : **choisis par nous** : des bleus clairs puis foncés sur blanc, comme le repère visuel d'André ; ils sont à nous, aucune valeur n'est reprise d'une source) ;
`t = 128` donne `(227, 237, 247)` (calcul à la main) ; `plein = 3`, `valeur = 1` donne `(168, 204, 231)` (calcul à la main). La symétrie de l'image (miroirs gauche-droite et haut-bas, rotation de 180°) est **exacte** par construction (aucun échantillon sur l'axe, aucune égalité) ; la rotation de 60° ne s'applique pas à une grille de pixels carrée.

**V-G1-RENDU-07 : régression, l'empreinte de l'image de référence.** `Neige::essayer_new(20, FIG9_RHO_050)`, 400 pas, `cote = 96`, `rayon_vue = 20`, `sur = 4`, `plein = 3 * 2^31` : `f535822dd6cdac22ee1cf78c62e0d02e1acc86be9fd648343426e2075230a88a` ; 439 cases rattachées. Source : `g1_ref.py image` (simulation et rendu de la doc, `hashlib.sha256`) : un second calcul
indépendant. **V-G1-RENDU-08 : le suréchantillonnage lisse les bords** (propriétés de nombre de couleurs, calcul de la doc). **V-G1-RENDU-09 : la fraction de la doc à l'unité près** (`la_fraction_de_la_doc_a_l_unite_pres`, ajouté par le juge) : deux échantillons à moins de `5e-10` case d'une frontière, trouvés par une recherche du juge (`chercher_frontiere2.py`, `choisir_frontiere.py`) et calculés à la main avec les entiers de la doc (`884037 A > 510399 B` ; `154227 A < 89043 B`) ; aucune tolérance.

---

## Ce que cette fiche ne contient pas

Aucune valeur de forme publiée (la figure 9, la figure 13, le diagramme de Nakaya : aucun nombre) ; aucune valeur du coût ou de la dérive (mesurées, dans `MODELE.md`) ; aucune valeur de la 3D, du bruit `σ`, des figures 15 à 17 (hors de l'étape).

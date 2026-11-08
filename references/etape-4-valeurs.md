# Étape 4 : les valeurs attendues, leur source, leur tolérance

Cette fiche dit, pour chaque valeur qu'un test de l'étape 4 attend, d'où elle vient, avec quelle tolérance et quel critère de
réussite. Chaque identifiant (`V-CIN-01`…) est cité dans la doc des tests qui l'emploient (`tests/etape_4_*.rs`). Elle est
séparée des résultats figés de notre propre code (`references/*.empreinte`, étiquetés « régression » dans les tests) : **aucune
valeur de cette fiche ne vient du code qu'elle juge**. Une valeur y vient d'une publication lue, d'un calcul exact fait par un outil
indépendant (nommé), d'une mesure du prototype (dite telle, avec ses graines et ses écarts), ou d'une convention que la doc de
`cinetique` fixe (dite telle, avec la raison).

**Étiquettes.** Les tests de l'étape 4 sont `juste :` (calcul exact, simulateur naïf écrit dans le test, valeur d'un outil indépendant)
sauf six `régression :` : les cinq de `etape_4_physique.rs` (V-PHY-01 à 05 : des seuils mesurés sur le prototype, qui disent que la
physique du modèle spécifié a la forme des sources, sans être la preuve du code ; V-PHY-05 confronte en plus une valeur publiée) et `regression_reference_cinetique_graine_1` (V-DIR-08).

**Le modèle** (décision de conception de l'étape, dite dans la doc de `src/cinetique.rs`) : le SOS à valeur absolue sur le réseau
carré périodique, `H = J · Σ_liaisons |h_i − h_j| − Δμ · Σ_i h_i` (chaque paire de voisins une fois), dynamique de Metropolis à
proposition symétrique (une colonne au hasard, ajouter ou ôter avec une chance sur deux), probabilités `e^(−k/16)` lues dans
`math::EXP_NEG`, `J/kT` et `Δμ/kT` en seizièmes entiers, le temps en balayages de `L²` essais. **Ce n'est pas la cinétique de Gilmer
et Bennema** (taux d'arrivée indépendant de l'environnement, taux de départ `e^(−nφ/kT)` : leur résumé dit seulement que les
probabilités vérifient la réversibilité microscopique) : l'équilibre est le même (Boltzmann), pas les vitesses ; aucune vitesse de ce
dépôt n'est comparable en nombre à celles de leur article, et on ne le prétend pas.

## Les sources : ce qui a été lu et ce qui ne l'a pas été

**Honnêteté sur la lecture.** Les pages ont été lues avec l'outil `WebFetch`, qui rend une réponse d'un petit modèle à une question posée
sur la page, non le texte brut. Les nombres ci-dessous ont été obtenus par deux lectures indépendantes quand c'est dit. Les PDF
(`arxiv.org/pdf/…`) n'ont pas pu être décodés par l'outil ; les articles de revue payants n'ont pas pu être lus, sauf leur résumé.

| Source | Ce qui a été lu | Ce qui n'a pas pu l'être |
|---|---|---|
| M. Hasenbusch et K. Pinn, *Computing the roughening transition of Ising and solid-on-solid models by BCSOS model matching*, J. Phys. A 30, 63 (1997), arXiv cond-mat/9605019 (page HTML ar5iv, lue le 8/11/2026) | l'ASOS est défini par `H = K^ASOS · \|h_x − h_y\|` (équation 4, hauteurs entières) ; « un facteur `1/k_B T` est absorbé dans la définition du hamiltonien » (donc `K = J/kT`) ; **`K_R^ASOS = 0,80608(2)`** (résumé, tableaux 2 et 6) ; `K_R^DG = 0,6653(2)` ; interface d'Ising 3D `K_R^Ising = 0,40758(1)` ; `K_R^BCSOS = ½ ln 2` (équation 12) ; le résumé donne la valeur `0,80608(2)` à deux lectures (page de résumé et page HTML) | la phrase qui dit que la somme porte sur chaque paire de plus proches voisins **une seule fois** n'a pas été lue pour l'ASOS (elle l'est pour l'Ising, équation 1 : « paires non ordonnées ») ; le rapport `K_R^ASOS / K_R^Ising = 1,98` (`≈ 2`, ce que donne le passage du modèle d'Ising à l'ASOS, où chaque hauteur de différence coûte deux liaisons rompues) est la seule indication que `K` compte les paires une fois |
| M. Hasenbusch, M. Marcu, K. Pinn, Physica A 208, 124 (1994), « High precision renormalization group study of the roughening transition », arXiv hep-lat/9404016 | page de résumé : `K_R^ASOS = 0,8061(3)`, `K_R^DG = 0,6645(6)` (même valeur à l'erreur près) | le texte (PDF illisible par l'outil) |
| G. H. Gilmer et P. Bennema, J. Appl. Phys. 43, 1347 (1972), « Simulation of Crystal Growth with Surface Diffusion » (page AIP, **résumé seul**) | la simulation mesure des taux de croissance au-dessus et au-dessous de la température de rugosité ; **sous elle, la face (100) d'un cristal de Kossel montre une barrière de nucléation bidimensionnelle à faible sursaturation ; au-dessus, la croissance est presque linéaire en la sursaturation**, quelle que soit la diffusion de surface ; un modèle de marches combiné à la théorie de la nucléation 2D est cohérent avec les taux simulés sous la rugosité | le texte, les paramètres, aucun nombre |
| Gilmer et Bennema, J. Cryst. Growth 13-14, 266 (1972) | rien : non trouvé ni lu | tout |
| J. D. Weeks et G. H. Gilmer, Adv. Chem. Phys. 40, 157 (1979) | rien directement ; la notice bibliographique et des articles qui la citent (Chui et Weeks 1978) : croissance linéaire non activée au-dessus de la rugosité, sous-linéaire, par nucléation, au-dessous | le chapitre |
| H. van Beijeren, Phys. Rev. Lett. 38, 993 (1977) | la notice et le résumé (modèle exactement soluble, isomorphe au modèle à six vertex) ; `K_R^BCSOS = ½ ln 2` lu dans Hasenbusch et Pinn (équation 12) | le texte |
| W. K. Burton, N. Cabrera, F. C. Frank, Phil. Trans. R. Soc. A 243, 299 (1951) | le résumé (partie IV : la structure d'une surface parfaite comme phénomène coopératif) | le texte |
| **Relu par le juge** : Hasenbusch et Pinn 1997 (ar5iv, deux lectures) | éq. 4 `H_ASOS = K^ASOS \|h_x − h_y\|`, `1/k_B T` absorbé ; `K_R^ASOS = 0,80608(2)` ; l'Ising écrit avec « toutes les paires (non ordonnées) de plus proches voisins » ; « la largeur diverge en `ln L` dans la phase rugueuse et reste finie dans la phase lisse » | la phrase « chaque paire une fois » pour l'ASOS (même constat que l'écrivain) |
| *Surface width of the Solid-On-Solid models*, arXiv hep-lat/9710079 (ar5iv, lu par le juge ; auteurs non relevés) | l'ASOS écrit `H = J Σ_<i,j> \|h_i − h_j\|` ; point de rugosité `u_r = e^(−2J/kT) = 0,19945(1)` (repris de l'appariement BCSOS de Hasenbusch et al.), soit `J/kT_R = −½ ln 0,19945 = 0,8061` : **la même valeur que `K_R` avec `K = J/kT` et `J` par liaison**, recoupement de la convention | le texte des séries |
| M. Hasenbusch, S. Meyer, M. Pütz, *The roughening transition of the 3D Ising interface: a Monte Carlo study*, arXiv hep-lat/9601011 (ar5iv, lu par le juge) | éq. 20 : `W² = const + (1/(2π β_eff)) ln L` pour `L` assez grand ; « la pente à la transition rugueuse devrait être `1/π²` » ; les corrections ne décroissent qu'en `1/ln L` ; aux plus grandes tailles la pente mesurée dépasse de 10 % l'asymptotique | l'application au SOS est par universalité (même classe que le BCSOS, hep-lat/9710079), non lue dans un article sur le SOS |
| K. G. Libbrecht, *The physics of snow crystals*, Rep. Prog. Phys. 68, 855 (2005) (texte complet, copie déposée par l'auteur, lu par le juge) | éq. 1 `v_n = α v_kin σ` ; § 2.5 : surface rugueuse, `α ≈ 1` ; § 2.5.1, éq. 19-20 : croissance limitée par la nucléation, `α ≈ A exp(−σ_crit/σ)`, `σ_crit = π β² Ω₂ / (3 (kT)²)` ; § 2.6.1 : au-dessus de `T_R`, `α ≈ 1` et croissance non facettée | — (aucune valeur pour le SOS : c'est une revue sur la glace) |
| Gilmer et Bennema 1972, page AIP (résumé relu par le juge) | une barrière de nucléation bidimensionnelle à faible sursaturation sous la température de rugosité ; une dépendance presque linéaire en la sursaturation au-dessus | le texte |
| M. van Veenendaal et al., Surf. Sci. 417, 121 (1998) | le résumé : Monte-Carlo sur un SOS de Kossel (100) avec seulement l'attachement et le détachement | le texte |

**Conséquence.** Les valeurs publiées lues sont `K_R^ASOS = 0,80608(2)` (et `½ ln 2` pour le BCSOS), la pente universelle `1/π²` de `w²`
en `ln L` à la transition, et **la forme qualitative** des lois de croissance (nucléation en `exp(−σ_crit/σ)` sous `T_R`, linéaire au-dessus).
Le modèle est confronté en nombres à une seule d'entre elles, par V-PHY-05 (ajouté par le juge) : la pente `c` de `w²` en `ln L` (tailles 8 et
32) vaut `1,75/π²` à `J/kT = 0,75`, `1,25/π²` à `0,8125` et `0,47/π²` à `0,875` : elle passe par `1/π²` là où la source place `K_R`. Ni vitesse
ni seuil ne sont reproduits en nombres. Ce que le prototype montre de `K_R` : la rugosité `w²` à `Δμ = 0`, pour `J/kT = 0,8125` (13 seizièmes, juste au-dessus de `K_R`),
croît avec la taille (`0,308 ; 0,381 ; 0,450 ; 0,523` pour `L = 8, 16, 32, 64`, 12 graines, environ `+0,07` par doublement de `L`), alors
que pour `J/kT = 1` elle sature (`0,081 ; 0,087 ; 0,0905 ; 0,0898`) : la transition est entre les deux, d'accord avec `0,806`, **sans la
mesurer** (une grille au seizième est trop grossière, `K` varie de `0,0625`). Les valeurs de `J/kT` des tests encadrent `K_R` (`0,375` à `2`).

## Les outils indépendants

Tous dans les notes de la session (`essai-4-outils/`) ; le code Rust n'est jamais l'outil.

- `simulateur_naif.py` : un simulateur Python de la dynamique **écrit d'après la doc** (modèle de PCG32 transcrit de `pcg_basic.c`, comme `essai-0-outils/ref_alea.py` ; table `e^(−k/16)` recalculée par `decimal` à 80 chiffres et recoupée avec V-EXP-01 de l'étape 0 ; arrondi du CSV en rationnels exacts `fractions`). Il donne les empreintes, les sommes, les hauteurs, les CSV et les aperçus écrits en dur. `valeurs_lancer.py` l'emploie pour les scénarios.
- `graines_bords.py` : fabrique des graines de PCG32 dont les trois premiers tirages sont (peu importe, un bit voulu, **une cible exacte**), par inversion de l'amorçage (le multiplicateur de PCG32 est inversible modulo `2^64` ; les 27 bits de poids faible de l'état n'influent pas sur la sortie XSH-RR et servent à fixer le bit de pile). Vérifié par un second pas de PCG dans le même script.
- `enumeration_l3.rs` (Rust, autonome, `rustc -O`, aucun lien avec le crate) : la rugosité moyenne exacte `<w²>` de la loi `e^(−H/kT)` sur le tore 3 × 3, en énumérant toutes les configurations de hauteurs relatives (la colonne 0 fixée à 0, `|h| <= H`), sommes de Neumaier ; `enumeration_l3.py` (Python, `Fraction` pour `w²`) le recoupe à `H = 2` : mêmes 12 chiffres pour `J = 12, 16, 20`.
- `mutations.py` et `nul_cinetique.rs` : les mutants et le code nul (voir le rapport de l'écrivain).
- `explorer.rs`, `calibration.rs`, `calibration3.rs` (exemples du prototype jetable) : les mesures qui ont fixé les tolérances statistiques.

**Tolérance, règle générale.** Les tests exacts (entiers, octets, chaînes, flottants rendus par une formule de la doc) n'ont **aucune**
tolérance. Les tests statistiques en ont une, qui vient d'un calcul (`J = 0`) ou d'une mesure faite **avant** d'écrire le seuil, sur d'autres
graines que celles du test ; elle n'a jamais été élargie pour un cas.

---

## cinetique : la dynamique exacte (`tests/etape_4_exact.rs`)

| Identifiant | Valeur | Source | Tolérance | Critère |
|---|---|---|---|---|
| V-CIN-01 | `Face` égale à un simulateur naïf écrit dans le test : décision de chaque essai, hauteurs, octets canoniques (hauteurs + générateur) après chaque balayage ; 12 jeux (`L` 3 à 16, `J` 0 à 100, `Δμ` −12 à 24, plans ou au hasard, graines 1 à 12) ; `balayages(n)` et `balayage()` répété aboutissent au même état | la doc de `cinetique` (décision de conception) : le naïf recalcule `H` en entier avant et après chaque proposition, donc vérifie `k = J(2n − 4) ∓ Δμ` contre sa définition ; recoupé par `simulateur_naif.py` (V-MES-02) | aucune | égalité exacte ; chaque jeu accepte au moins 5 essais et en refuse au moins 5 |
| V-CIN-02 | quatorze essais sur la face plane 4 × 4, graines construites : `r = EXP_NEG[k]` refusé et `EXP_NEG[k] − 1` accepté pour `k = 12` (ajout), `20` (retrait), `1` ; `k = 365` : `r = 0` accepté, `r = 1` refusé ; `k = 366`, `370` : refusé **sans tirage** ; `k = 0`, `−5` (ajout et retrait) : accepté **sans tirage** ; le générateur a tiré exactement 2 ou 3 fois | `graines_bords.py` (graines listées dans le test et dans `graines_bords_resultat.txt`) ; `EXP_NEG[12] = 2028798896`, `EXP_NEG[20] = 1230528733`, `EXP_NEG[1] = 4034748382`, `EXP_NEG[365] = 1` : V-EXP-01 de l'étape 0, recalculés par `decimal` | aucune | égalité exacte ; le test recontrôle d'abord, sur `Alea`, que la graine donne les tirages annoncés |
| V-CIN-03 | `Face::indice` sur chaque colonne de la face `0000 / 0110 / 0130 / 0000`, `J = 16`, `Δμ = ±5` : 18 valeurs (`59, 5, −59, 69, 27`…) | calcul à la main d'après la doc (voisins `(x+1,y),(x,y+1),(x−1,y),(x,y−1)`, `n = ` voisins `<= h` pour l'ajout, `>= h` pour le retrait) ; recoupé par le Python | aucune | égalité |
| V-CIN-04 | 200 faces au hasard (`L` 3 à 5, hauteurs −2 à 2, `J` 0 à 40, `Δμ` −30 à 30) : `indice` = différence exacte de `H` | définition du hamiltonien, recalculée en entier dans le test | aucune | égalité sur chaque colonne et chaque sens |
| V-CIN-05 | `Δμ = ±1000`, `J = 16`, 3000 essais : ajout toujours accepté (retrait jamais) pour `+`, et inversement ; 2 tirages par essai exactement ; hauteurs = nombre de tirages avec le bon bit de pile | `k <= 64 − 1000 < 0` ; `k >= 1000 − 64 >= 366` ; tirages rejoués dans le test | aucune | égalité exacte |
| V-CIN-06 | `J = 100`, `Δμ = 0, ±30`, face plane : `k = 400, 370, 430`, tous `>= 366` : aucune hauteur ne change, 2 tirages par essai, sommes et mesures nulles | calcul | aucune | égalité exacte |
| V-CIN-07 | face 3000 × 3000, 40 000 essais : au moins 5 rejets de `borne` ; hauteurs et générateur égaux à ceux d'un naïf creux qui réécrit la boucle de `borne` | `seuil = n.wrapping_neg() % n = 1 967 296` pour `n = 9 000 000` (`4,6e-4` de rejets) d'après la doc de `Alea::borne` (étape 0) | aucune | égalité exacte |
| V-CIN-08 | paniques : taille 0, 1, 2, 4097, 100 000 ; mauvaise longueur de `depuis_hauteurs` ; colonne hors de la face ; 3 et 4096 permis | « Panique » écrit dans la doc | — | `catch_unwind` |
| V-CIN-09 | `hauteur(x, y)` lit le numéro `y·L + x` | convention de la doc | aucune | égalité |

## cinetique : mesures, canonique, CSV, aperçu (`tests/etape_4_mesures.rs`)

| Identifiant | Valeur | Source | Tolérance | Critère |
|---|---|---|---|---|
| V-MES-01 | l'écriture canonique de la face 3 × 3 `0 1 −2 / 3 0 0 / −1 5 2` (`J = 16`, `Δμ = −5`, graine 7), 72 octets écrits un à un ; sa SHA-256 `60c39a9a…9343` ; après un essai accepté (graine construite) l'état de 100 octets, SHA-256 `3368b140…e9cc` ; `ecrire_canonique` ajoute à un `Vec` non vide | octets posés d'après la doc ; l'état de `Alea::new(7)` `d1a0bdb386a42205 6d00…` est celui de `etape_0_lancer` ; SHA-256 de Python `hashlib` ; le générateur après trois tirages par `simulateur_naif.py` | aucune | égalité |
| V-MES-02 | empreinte, `S`, `Q`, `N·Q − S²` de 5 faces menées de l'état plan : `(5,12,4,graine 3,20 balayages) → S=19, Q=19, num=114, 891ed75a…6abc` ; `(8,16,−6,11,30) → −18, 22, 1084, 6aa45e79…40ee` ; `(6,0,8,2,25) → 168, 1540, 27216, 80297bc3…7ba1` ; `(8,24,12,5,40)` et `(7,100,20,9,10)` : faces planes `683a6ad4…633e` et `67740d74…c095` ; hauteurs des deux premières | `simulateur_naif.py` (modèle PCG32 + table `decimal` + `hashlib`) | aucune | égalité |
| V-MES-03 | face 3 × 3 ci-dessus : `S = 8`, `Q = 44`, `N·Q − S² = 332`, `<h> = 8/9`, `w² = 332/81` ; faces uniformes `w² = 0` ; deux niveaux `w² = 1/4` ; une colonne à 1000 `w² = 58593,75` | calcul à la main (arithmétique exacte) | aucune (égalité de flottants : une conversion et une division, la doc) | égalité exacte |
| V-MES-04 | `vitesse_croissance` = `(S_après − S_avant) as f64 / (N·B) as f64` ; `rugosite_moyenne` = `total as f64 / (N²·B) as f64` avec `total = Σ (N·Q − S²)` après chacun des `B` balayages, l'état de départ non compté ; la face avance de `B` balayages ; panique pour `B = 0` | la doc (les formules y sont écrites) ; recalculées dans le test | aucune | égalité exacte |
| V-MES-05 | `J = 0`, `Δμ > 0` : la vitesse par balayage vaut en moyenne `(1 − p)/2`, `p = EXP_NEG[Δμ en seizièmes]/2^32`, d'écart-type `σ/√M` avec `σ² = (1 + p)/2 − ((1 − p)/2)²` et `M = N · balayages` | calcul : `k = ∓Δμ` ne dépend ni de la colonne ni des hauteurs, chaque essai est indépendant (+1 avec probabilité 1/2, −1 avec `p/2`, sinon 0) ; **la variance est calculée, non mesurée** | `5 σ/√M` (`σ/√M = 9,2e-4` à `Δμ/kT = 0,25` et `6,0e-4` à 2, pour `M = 1 024 000`) ; relevé : écarts de `−1,9 σ` à `−0,25 σ` sur les 8 points | `|v − (1 − p)/2| <= 5σ/√M` pour `Δμ/kT = 0,25 ; 0,5 ; 1 ; 2`, graines 1 et 2 |
| V-MES-06 | rugosité moyenne exacte `<w²>` de la loi de Boltzmann sur le tore 3 × 3, `Δμ = 0` : `0,0605767076` (`J = 16`), `0,0181303680` (`J = 20`), `0,0056972443` (`J = 24`) | `enumeration_l3.rs`, `|h| <= 7`, stabilisé à `1e-9` entre `H = 6` et `H = 7` (`0,060576706742 → 0,060576707558`) ; recoupé à `H = 2` par `enumeration_l3.py` (`0,060045915872`, `0,018110524115` identiques) ; **recoupé par le juge** (énumération Rust écrite à part, poids `exp` en `f64`) : à `H = 5`, `0,0605766791 ; 0,0181303680 ; 0,0056972443` (`J = 12` : `0,19529`) ; la dynamique de Metropolis vérifie le bilan détaillé, donc sa loi stationnaire est celle-là | `4` erreurs-types de la moyenne de 30 graines : `1,4e-3 ; 6e-4 ; 3e-4` (mesure : écart-type d'une graine `1,4 à 2,1e-3`, `0,8 à 1,0e-3`, `0,35 à 0,5e-3`, sur quatre blocs de 30 graines ; écarts à la valeur exacte de `−1,6` à `+1,3` erreur-type) | `|moyenne − exacte| <= tolérance`, graines 1 à 30, 500 balayages de transitoire, moyenne sur 20 000 |
| V-MES-07 | le CSV de `simuler` : pour `(8,16,−6,graine 11,100 balayages)` 52 lignes, SHA-256 `698e19b9…9881`, lignes `0,0.000000,0.000000`, `2,0.000000,0.062500`, `4,0.015625,0.015381`, … `100,-0.968750,0.061523` ; `(16,12,8,4,120)` 62 lignes `5645c453…d5fe`, dernière `120,7.402344,0.537338` ; 44 autres jeux comparés à un CSV bâti dans le test | `simulateur_naif.py` (arrondi exact en rationnels : `floor(x·10^6 + 1/2)`) ; formateur du test écrit autrement (quotient et reste) ; demi-cas exacts de l'arrondi vérifiés présents, des deux signes | aucune | égalité exacte |
| V-MES-08 | les niveaux de gris de l'aperçu : `255·(h − h_min)/(h_max − h_min)` en division entière, `128` pour une face plane ; faces 3 × 3 de `0` à `8` (`0, 31, 63, 95, 127, 159, 191, 223, 255`), trois faces de 5 × 5 et 6 × 6 de Python | calcul à la main ; `simulateur_naif.py` | aucune | égalité |
| V-MES-09 | `TAILLE_MIN = 3`, `TAILLE_MAX = 4096`, `BALAYAGES_MAX = 1 000 000` | convention de la doc (3 : avec 2 colonnes deux voisins coïncident ; 4096 : comme `COTE_MAX` de `scenario`) | aucune | égalité |

## La physique, mesurée sur le prototype (`tests/etape_4_physique.rs`)

Valeurs de J/kT en seizièmes : `8` = 0,5 ; `16` = 1 ; `24` = 1,5 ; `32` = 2 ; `K_R = 0,806 ≈ 12,9`. **Toutes les mesures sont celles du prototype**,
moyennes sur les graines 1 à 20 ; la calibration (**avant** les seuils) est faite sur 30 autres graines (1001 à 1030), recoupée sur 1 à 30,
2001 à 2030, 3001 à 3030, 4001 à 4030. Le générateur est déterministe : toute implémentation fidèle à la doc rend exactement ces nombres.

| Identifiant | Valeur | Source | Tolérance | Critère |
|---|---|---|---|---|
| V-PHY-01 | face lisse (`J = 24`, `L = 16`, 300 balayages jetés, vitesse sur 1000) : `v/Δμ` à `Δμ/kT = 0,25 ; 1 ; 1,5 ; 2` vaut `−6e-6 ; 0,0098 ; 0,0219 ; 0,0346` (calibration) ; `v` à `Δμ/kT = 1,5` : 0,033 | **forme** : Gilmer et Bennema 1972 (résumé) : barrière de nucléation 2D à faible sursaturation sous la température de rugosité ; **nombres** : mesure du prototype | rapport `(v/Δμ)(0,25)/(v/Δμ)(1,5) <= 1 %` (mesuré au plus `3e-4` sur 5 blocs) ; `v(1,5) > 0,02` (60 % de 0,033) ; `v/Δμ` croît de plus de 5 erreurs-types à chaque point | les trois critères |
| V-PHY-02 | face rugueuse (`J = 8`, mêmes durées) : `v/Δμ` à `Δμ/kT = 0,125 ; 0,25 ; 0,5 ; 1` vaut `0,2452 ; 0,2391 ; 0,2348 ; 0,2348` (rapport `1,044` ; sur 5 blocs `1,013` à `1,044`, moyenne `1,03`, écart-type `0,013`) ; mobilité à `Δμ/kT = 0,5` : `J = 0` : `0,3935` (calcul exact), `J = 4` : `0,3468`, `J = 8` : `0,2348` | **forme** : Gilmer et Bennema 1972 (résumé) : croissance presque linéaire au-dessus de la température de rugosité ; `J = 0` : calcul exact (V-MES-05) ; le reste : mesure du prototype | rapport plus grand/plus petit `<= 1,10` (plus de 4 écarts-types au-dessus de la moyenne `1,03`) ; mobilité décroissante en `J`, écarts de plus de 5 erreurs-types | les deux critères |
| V-PHY-03 | `w²` à `Δμ = 0`, `L = 12`, 1000 + 1000 balayages, `J = 32, 24, 16, 12, 8, 6` : `0,00074 ; 0,0062 ; 0,090 ; 0,46 ; 1,33 ; 2,53` ; rapports consécutifs `8,4 ; 14,5 ; 5,1 ; 2,9 ; 1,9` | **forme** : la rugosité croît avec la température ; **nombres** : mesure du prototype | chaque pas de plus de 5 erreurs-types combinées et d'un rapport `>= 1,3` (68 % du plus petit mesuré) | strictement croissante |
| V-PHY-05 | pente `c = (w²(32) − w²(8))/ln 4`, `Δμ = 0`, graines 1 à 20, 1000 + 2000 balayages : `c·π² = 1,746 ; 1,247 ; 0,467` pour `J = 12, 13, 14` (graines du test) ; calibration du juge, 12 blocs de 20 graines (7 600 000 et suivantes) : `1,64 à 1,80 ; 1,12 à 1,29 ; 0,41 à 0,56`, erreur-type d'un bloc `0,03 à 0,07` | **valeurs publiées** : `K_R = 0,80608(2)` (Hasenbusch et Pinn 1997), pente `1/π²` à la transition (hep-lat/9601011, éq. 20) avec des corrections en `1/ln L` qui la font dépasser sur de petites tailles ; **nombres** : mesure du prototype | `c(12)·π² >= 1,3` ; `c(13)·π²` dans `[0,9 ; 1,6]` ; `c(14)·π² <= 0,75` (au moins 4 erreurs-types de marge sur 240 graines) | les trois critères |
| V-PHY-04 | `w²` selon `L = 6, 12, 24` à `Δμ = 0` : rugueuse (`J = 8`) `0,988 ; 1,331 ; 1,696` (rapport 24/6 : `1,72`, `1,68` à `1,72` sur 5 blocs) ; lisse (`J = 16`) `0,0819 ; 0,0889 ; 0,0881` (rapport 24/12 : `0,992`, de `0,992` à `1,030` sur 5 blocs) | **forme** : la largeur d'une face rugueuse croît avec `L` (en `ln L`), celle d'une face lisse sature (théorie de la rugosité ; **non lue** : seul le résumé de Gilmer et Bennema l'a été) ; **nombres** : mesure du prototype | rugueuse : chaque doublement de plus de 5 erreurs-types et rapport 24/6 `>= 1,4` ; lisse : rapport 24/12 dans `[0,90 ; 1,10]` | les deux critères |

## lancer et le scénario d'exemple (`tests/etape_4_lancer.rs`)

| Identifiant | Valeur | Source | Tolérance | Critère |
|---|---|---|---|---|
| V-DIR-01 | quatre scénarios `boite L L` + `cinetique J Δμ B` : empreinte, nombre de lignes et SHA-256 du CSV, SHA-256 des octets RVB de l'aperçu (`(8,16,−6,100,graine 11)`, `(16,12,8,120,4)`, `(5,12,4,20,3)`, `(3,20,0,7,5)`) | `valeurs_lancer.py` (par `simulateur_naif.py`) | aucune | égalité |
| V-DIR-02 | `lancer` = `Face::new`, `simuler`, `ecrire_png(L, L, apercu())` ; directives dans l'autre ordre, lignes vides et commentaires, zéros de tête, `-0` | la doc de `lancer` (décision de conception) | aucune | égalité exacte |
| V-DIR-03 | déterminisme ; 20 graines, 20 empreintes et 20 CSV distincts ; paramètres différents, empreintes différentes | propriétés exactes | — | vérifié |
| V-DIR-04 | directives refusées par `DirectiveInconnue(ligne)` : arguments en trop ou en moins, `+4`, `4.0`, `0x4`, `1e2`, `-`, `--4`, `٤`, bornes (`2^32` pour `J`, `±2^31` pour `Δμ`, 0 et 1 000 001 balayages), seconde directive, autre directive, forme `hexagone`, boîte non carrée ou de côté 2 ; acceptés : `J = 0` et `u32::MAX`, `Δμ = i32::MAX` et `i32::MIN`, 1 balayage, 1 000 000 balayages sur 3 × 3 | la doc de `lancer` et de `cinetique` (décision : réutiliser `DirectiveInconnue(ligne)` plutôt qu'une nouvelle variante, pour ne pas toucher à `ErreurLancer` ni à `main.rs`) | — | égalité |
| V-DIR-05 | sans directive `cinetique`, `lancer` se comporte comme à l'étape 0 : `boite 64 48` graine 1 → `5a21e2ae…f613`, CSV `temps,cases` | V-LANC-01 et 02 de l'étape 0 | aucune | égalité |
| V-DIR-06 | `scenarios/cinetique.txt` : `boite 32 32` et `cinetique 16 24 400` ; 51 lignes de CSV, `<h>(400) > 50`, `w²(400) > 0,5` | le contenu du fichier est écrit dans la doc de l'étape ; seuils : mesure du prototype (`<h> = 59,05`, `w² = 1,13` à la graine 1), loin des seuils | `> 50` et `> 0,5` | inégalités |
| V-DIR-07 | le binaire écrit exactement `empreinte.txt`, `mesures.csv`, `apercu.png` égaux à ceux de `lancer` ; un scénario refusé : code 1, `ligne 2`, rien d'écrit | la doc de `src/main.rs` (étape 0) | aucune | égalité |
| V-DIR-08 | `references/cinetique-graine-1.empreinte` (régression, produite par la vague du code) | résultat figé de notre code : **ne prouve que la stabilité** | aucune | égalité |

## Ce que cette étape n'établit pas

- **Aucune vitesse ni aucun seuil publié n'est reproduit en nombres.** Les résumés lus de Gilmer et Bennema (1972) ne donnent que la forme
  qualitative ; leur texte, celui de Weeks et Gilmer (1979), de Burton, Cabrera et Frank (1951) et de van Beijeren (1977) n'ont pas été lus.
  Les nombres publiés confrontés sont `K_R^ASOS = 0,80608(2)` et la pente `1/π²` (V-PHY-05) : le modèle **encadre** `K_R` au seizième de `J/kT`
  près (entre `0,75` et `0,875`) sans le **mesurer**, et sa pente à `0,8125` dépasse `1/π²` de 12 à 29 % (corrections de taille finie, attendues
  en `1/ln L` mais non chiffrées pour le SOS par une source lue).
- **Il n'y a pas de seuil de nucléation au sens strict** : la vitesse d'une face lisse suit `exp(−σ_crit/Δμ)` (Libbrecht 2005, éq. 19), jamais
  nulle ; « quasi nulle sous un seuil » (V-PHY-01) veut dire « non mesurable sur `16 × 16` colonnes et 1000 balayages ». La loi et `σ_crit` ne
  sont pas testées.
- **La linéarité à haute température** (V-PHY-02) est un fait mesuré (`v/Δμ` constante à 10 % sur `Δμ/kT` de `0,125` à `1`, `J/kT = 0,5`,
  `L = 16`) ; sa seule référence exacte est `J = 0`, où la loi `(1 − p)/2` n'est **pas** linéaire sur cette plage (rapport `1,49`).
- **Ce n'est pas la cinétique de Gilmer et Bennema** : c'est un Metropolis à proposition symétrique. La loi stationnaire (Boltzmann, vérifiée exactement sur 3 × 3)
  est la même ; les vitesses ne le sont pas. Une comparaison de nombres avec leur article demanderait leur loi de taux.
- **La convention `K = J/kT` avec chaque paire de voisins comptée une fois** n'est pas lue en toutes lettres dans la source ASOS (voir le tableau
  des sources) : la soutiennent le rapport `K_R^ASOS/K_R^Ising ≈ 1,98`, l'écriture `H = J Σ_<i,j>` et `u_r = e^(−2J/kT_R) = 0,19945` de
  hep-lat/9710079 (qui redonne `0,8061`), et V-PHY-05 (avec `K` doublé, la face à `J/kT = 0,75` serait lisse).
- **La nucléation n'est testée que par sa signature** (`v/Δμ` croissante, vitesse quasi nulle à `Δμ/kT = 0,25`), pas par la loi en `exp(−B/Δμ)`, ni par la
  dépendance en taille et en durée (la nucléation est un événement rare : une face plus grande ou observée plus longtemps croît davantage).
- **Les tailles testées sont petites** (`L` de 3 à 32 pour les mesures, 3000 pour le test de `borne`) : le coefficient du logarithme de la rugosité,
  l'exposant de croissance cinétique (`β`) et la rugosité cinétique à grande échelle ne sont pas mesurés.
- **Le temps** est compté en balayages de `L²` essais d'un Metropolis à site aléatoire : aucun lien avec un temps physique.
- **L'égalité sous Linux et sous Windows** est promise par l'arithmétique (entiers, table) mais prouvée seulement par la CI qui rejoue `references/cinetique-graine-1.empreinte`.

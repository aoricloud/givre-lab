# Étape 3 : les valeurs attendues, leur source, leur tolérance

Cette fiche dit, pour chaque valeur qu'un test de l'étape 3 attend, d'où elle vient, avec quelle tolérance et quel critère de
réussite. Chaque identifiant (`V-LECT-01`, `V-PAS-04`…) est cité dans la doc du test qui l'emploie (`tests/etape_3_*.rs`). Elle est
séparée des résultats figés de notre propre code (`references/*.empreinte`). Les tests étiquetés « régression » sont `regression_reference_reiter_plaque` (le
fichier de référence, que la vague du code produit) et les cinq tests de `etape_3_formes.rs` (des intervalles **mesurés sur le prototype**, voir la section
« mesurées » : ce sont les seules valeurs de cette fiche qui viennent de notre code, dites telles). **Toute autre valeur de cette fiche ne vient pas de notre code Rust** : elle vient d'une publication
lue (nommée), d'un calcul exact fait par un outil indépendant (nommé), d'une formule close dérivée à la main (écrite), ou d'une convention
que la doc de `flocon` fixe (dite telle).

**Outils indépendants.** Python 3.13 : `modele_independant.py` (la règle de la doc de `flocon` réécrite avec des entiers de taille quelconque
et `hashlib`, sans rien partager avec le Rust) et `valeurs_etape3.py` (il écrit `tests/etape_3_commun/valeurs.rs`). Le modèle Python et le
prototype Rust ont donné les mêmes SHA-256 sur cinq trajectoires avant que le moindre test soit écrit. Les scripts et les sorties brutes sont dans
les notes de la session (`essai-3-outils/`).

**Tolérance, règle générale.** Les tests exacts (entiers, octets, chaînes, `brut` de `Fixe`) n'ont **aucune** tolérance. Les trois familles
qui en ont une sont les tests *mesurés* (`classes_de_formes`, `vitesse_radiale`, `dendrite_pour_plusieurs_alpha`), la borne d'arrondi de la
conservation de la masse (dérivée de la doc, pas mesurée) et les valeurs publiées qui sont exactes en nombre de pas. Chaque tolérance a été
mesurée avant d'être fixée ; aucune n'a été élargie pour faire passer un cas.

**Ce que les tests « juste » ne disent pas** est écrit dans chaque test, ligne `Ne couvre pas :`.

---

## Ce que la référence dit de la règle (ce qui fonde la doc de `flocon`)

Sources lues le 8/11/2026 : (1) C. A. Reiter, *A local cellular model for snow crystal growth*, Chaos, Solitons & Fractals 23 (2005) 1111-1119,
**préimpression de l'auteur** (`webbox.lafayette.edu/~reiterc/mvp/sfn/sfn_pp.pdf`, 17 pages, texte extrait entièrement) ; (2) la **page de
matériaux auxiliaires** de l'auteur (`.../sfn/index.html`) et sa **table β-γ de rayon 100** (`.../sfn/nn/index.html`, 240 vignettes, noms de
fichiers lus une fois en deux moitiés, dont 26 relus par une seconde question ciblée : mêmes valeurs) ; dix vignettes (images PNG) vues ; (3) J. Gravner et D. Griffeath, *Modeling snow crystal growth II: a mesoscopic lattice
map with plausible dynamics*, Physica D 237 (2008) 385, **version révisée de septembre 2007** (texte extrait entièrement). **Non lus** : la version
éditée de Chaos, Solitons & Fractals (derrière un accès payant : c'est la préimpression qui est citée) ; les figures 1 à 7 de la préimpression (ce
sont des images ; je n'ai pas pu en faire le rendu : pas d'outil pour rastériser un PDF) ; Gravner et Griffeath, Phys. Rev. E 79 (2009) 011601 et
Libbrecht, Rep. Prog. Phys. 68 (2005) 855 (non nécessaires à la règle ; le diagramme de morphologie de Libbrecht n'est pas utilisé).

| Règle de la doc | Ce que la source dit | Où |
|---|---|---|
| gelée si `s >= 1` | « Values of one or higher are taken to correspond to ice » ; « We consider a cell to be solid ice if its value is greater than or equal to one » | préimpression p. 4 ; page auxiliaire |
| réceptive = gelée ou voisine d'une gelée | « The first stage is to determine the receptive sites. These are the sites that are ice or that have an immediate neighbor that is ice » | préimpression p. 4 |
| une réceptive garde sa masse et reçoit `γ` | « the values of the cells are given by the values at the receptive sites plus a constant plus a diffusion term » ; « receptive sites are viewed as permanently storing any mass that arrives at that point » ; « the constant added to receptive sites » | préimpression p. 4 |
| les autres diffusent, les réceptives comptent pour 0 dans la moyenne | « The diffusion term is a local average of a modified cellular field obtained by setting the receptive sites to zero » ; « the zeros in the array of unreceptive sites are used as values in the averaging while the zeros in the array of receptive sites are placeholders » | préimpression p. 4 (figure 2, légende) |
| poids : centre `1/2`, voisines `1/12` (`α = 1`) | « the center cell has weight 1/2 while the six neighboring cells each have weight 1/12 » | préimpression p. 5 |
| `α` : centre `1 - α/2`, voisines `α/12` | section « Variation on Diffusion » : la discrétisation de `u_t = a ∇²u` sur le réseau hexagonal, `u(t+1, P) = u(t, P) + (α/12)(Σ_{N ∈ nn(P)} u(t, N) - 6 u(t, P))`, « our model with center weight 1/2 and weight 1/12 for neighbors corresponds to taking α = 1 ». **La formule est lue dans une extraction de texte où les lettres grecques et les fractions sont dégradées ; je l'ai reconstruite à partir de la phrase sur les poids, qui, elle, est lisible.** Le terme `(α/2)(ū - u)` de la doc est cette équation avec `ū = (1/6) Σ u(N)` | préimpression p. 6-7 |
| état initial : un germe de 1 dans un fond `β` | « We will ordinarily begin with a single cell of value one (an ice seed) in a sea of a constant background » | préimpression p. 4-5 |
| bord maintenu à `β` | « The boundary conditions are taken to be fixed at the background level at a fixed (Euclidean) distance from the initial cell ». **Écart du modèle documenté : un cercle euclidien chez l'auteur, la couronne de l'hexagone ici** | préimpression p. 5 |
| `β = γ = 0` : rien ne pousse ; `β = 0`, `γ > 0` : des plaques | « When β = 0 and γ = 0, no growth appears. When β = 0 and γ > 0, plates form, the rate depending upon γ » (les lettres grecques sont perdues dans l'extraction : lues d'après l'ordre des phrases et d'après les exemples `β = 0, γ = 1` de la page auxiliaire) | préimpression p. 5 |
| classes selon `β` et `γ` | « Regardless of γ, at high levels of β we get plates, sectors or broad stellar forms. For low levels of γ and at low levels of β, we get dendrite growth; for high levels of γ we obtain stellar or more abstract forms » (mêmes réserves sur les lettres) | préimpression p. 5-6 |
| `α = 0.5, 0.501, 0.502` donnent des images très différentes, toutes des dendrites | « Notice that α = 0.5, 0.501, 0.502 each give very different pictures; however, they all exhibit dendrite growth. The figure also shows the standard model when α = 1 which appears very similar to the 0.5 case. A pretty growth pattern ... when α = 2.003 and an extreme situation, with explosive behavior, ... when α = 2.69 » (`β = 0.4`, `γ = 0.0001`) | préimpression p. 7, figure 5 |
| Reiter vu par Gravner et Griffeath | « Reiter's model has two parameters: vapor density and an ad hoc constant shift in the mass field. Since the attachment rate is constant, his crystals lack the correct geometry for instabilities, macrosteps, and faceting, and display quite sensitive dependence on system parameters. The only sectored plate candidates found within his phase portrait evolve more like a Packard CA than a real snow crystal » | Physica D 237 (2008), section 4 |

**Rappel de ce qui n'est pas dans la doc** : les `α > 2` de la figure 5 (`2.003`, `2.69` : le poids du centre est négatif, la masse peut
l'être) ; le modèle documenté a `0 <= α <= 2`.

---

## lecture : `lire_decimal`, `Parametres::lire`, `Parametres::sont_valides`

Source : **convention** de la doc de `flocon` (un texte décimal simple, lu exactement, un seul arrondi, au plus proche, demi-cas vers `+∞`, comme
`Fixe::rapport`), choisie pour qu'une valeur de scénario ne dépende pas d'un flottant. Les `brut` viennent de Python (entiers exacts :
`(2 N 2^32 + 10^k) // (2 · 10^k)`).

| Identifiant | Valeur | Source | Tolérance | Critère |
|---|---|---|---|---|
| V-LECT-01 | 27 textes et leur `brut` (dont `1.00000000011641532` → `2^32` et `1.00000000034924596` → `2^32 + 1`, que le chemin par flottant arrondit à `2^32 + 1` et `2^32 + 2`) : `0.5` → `2^31`, `0.1` → 429496730, `0.3` → 1288490189, `0.05` → 214748365, `0.7` → 3006477111, `0.0001` → 429497, `0.35` → 1503238554, `0.45` → 1932735283, `0.002` → 8589935, `0.0035` → 15032386, `1.5` → `3·2^31`, `007`, `0.250`, `1.00`, 18 chiffres, `2147483647.5`, et `0.00000000011641532` → 0 contre `0.00000000011641533` → 1 (de part et d'autre de `2^-33`) | Python, entiers exacts (`valeurs_etape3.py`, constante `DECIMAUX`) ; `0.5`, `0.25`, `0.1`, `1`, `0`, `2` recoupés à la main | aucune | égalité exacte |
| V-LECT-02 | refus : vide, `.5`, `5.`, `.`, signes, exposants, virgule, espaces, `0..5`, `1.2.3`, lettres, `0x10`, `inf`, `nan`, chiffres arabes-indiens et pleine largeur, 19 chiffres, `2147483648`, `9999999999`, `2147483648.5` ; acceptés : `2147483647`, 18 chiffres (arrondi à 0), `007` | doc de `lire_decimal` | aucune | `None` ou `Some` exact |
| V-LECT-03 | `lire("1", "0.4", "0.001")` : `brut` `2^32`, 1717986918, 4294967 ; un champ mal écrit donne `None` | Python (`floor(0.4 · 2^32 + 1/2) = 1717986918`, `floor(0.001 · 2^32 + 1/2) = 4294967`) | aucune | égalité |
| V-LECT-04 | validité : `0 <= α <= 2`, `0 <= β < 1`, `0 <= γ <= 1` jugées sur le `Fixe` lu ; `0.9999999999` → `UN` (refusé), `0.9999999998` → `UN - 1` (accepté) | doc de `Parametres` (convention : `β < 1` pour que la couronne ne gèle pas, `α <= 2` pour qu'une masse ne devienne jamais négative, dérivé dans la doc) ; les deux `brut` limites par Python | aucune | égalité |
| V-LECT-05 | `Flocon::new` panique pour un rayon 0 et des paramètres invalides, accepte les bornes | doc de `Flocon::new` | — | `catch_unwind` |

## état et classes

Source : définitions de la doc (voir plus haut pour la règle publiée). Les cas sont écrits à la main ou recomptés par le simulateur naïf du test
(un dictionnaire de cases).

| Identifiant | Valeur | Source | Tolérance | Critère |
|---|---|---|---|---|
| V-ETAT-01 | état initial : `β` partout, `UN` au centre, temps 0, une gelée, six voisines du centre de bord, le reste non réceptif ; rayon du cristal 0, 0 pointe, compacité 1 ; `R` = 1, 3, 12, `β` = 0, 0.3, 0.9 | doc de `Flocon::new` | aucune | égalité |
| V-ETAT-02 | `depuis_masses` : la masse de la i-ième case de l'ordre canonique est `masses[i]` | doc de `reseau::Hexagone` (ordre `r` puis `q`) réécrit dans le test | aucune | égalité sur `R` = 1..4 |
| V-ETAT-03 | paniques de `depuis_masses` : longueur, masse négative, masse de `2^60` ou plus, rayon 0, paramètres invalides ; `2^60 - 1` et 0 acceptés | doc de `depuis_masses` | — | `catch_unwind` |
| V-ETAT-04 | une configuration de `R = 2` à la main : gelées `(0,0)` (égalité à 1) et `(2,-2)` (coin de la couronne, `1 + 5 ulp`) ; de bord les 6 voisins du centre et `(2,-1) (1,-2)` ; `(-2,1)` à `1 - 1 ulp` **non** gelée et sans effet sur ses voisines ; 9 non réceptives | calcul à la main, voisinage de `DIRECTIONS_HEX` ; le voisin hors de l'hexagone ne compte pas | aucune | égalité |
| V-ETAT-05 | `etat` = le naïf sur 25 états (`R` de 1 à 6, 5 formules de masses, valeurs frôlant le seuil) | simulateur naïf du test | aucune | égalité |
| V-ETAT-06 | `masse`, `etat` paniquent hors de l'hexagone | doc | — | `catch_unwind` |

## un pas

| Identifiant | Valeur | Source | Tolérance | Critère |
|---|---|---|---|---|
| V-PAS-01 | noyau de diffusion : bosse de `3/4` seule, `R = 3` : `α = 1` : centre `3/8` (`3·2^29`), voisines `1/16` (`2^28`) ; `α = 2` : 0 et `1/8` ; `α = 0.5` : `9/16` et `1/32` ; masse totale conservée | **calcul à la main**, tout est dyadique : `ū = (3/4)/6 = 1/8` exactement | aucune | égalité sur toutes les cases, bosse en `(0,0)` et en `(1,-1)` |
| V-PAS-02 | un pas depuis l'état initial, `R = 3`, `α = 1`, `β = 1/2` : centre `2^32` ; anneau 1 `5/8` (2684354560) ; coins de l'anneau 2 1968526678 ; autres de l'anneau 2 1789569707 ; couronne `2^31` ; avec `γ = 1/4` : le centre et l'anneau 1 gagnent `2^30`, rien d'autre ne change | **calcul à la main avec les arrondis de la doc** : `5 · 2^31 / 6 = 1789569706,67 → 1789569707` ; `ecart = -357913941`, `diffusion = -178956970` ; `4 · 2^31 / 6 = 1431655765,33` ; `ecart = -715827883`, `diffusion = -357913941` ; recoupé par le modèle Python | aucune | égalité |
| V-PAS-03 | fond uniforme `β` (aucune glace) invariant, 6 couples `(α, β)` dont `α = 2` | doc : la somme de six `brut` égaux, divisée par 6, est exacte, donc `ecart = 0` | aucune | égalité sur 8 pas |
| V-PAS-04 | un pas depuis `P1_AVANT` (`R = 2`, `α = 0.7`, `β = 0.3`, `γ = 0.05`, 19 masses sans symétrie dont une gelée sur la couronne et une à `1 - 2^-32`) | Python, `modele_independant.py` | aucune | égalité des 19 `brut` |
| V-PAS-05 | un pas depuis `P2_AVANT` (`R = 3`, `α = 2`, `β = 0.5`, `γ = 0.25`) | Python | aucune | égalité des 37 `brut` |
| V-PAS-06 | un et deux pas depuis l'état initial de `R = 3`, `α = 1`, `β = 0.45`, `γ = 0.1` | Python | aucune | égalité des 37 `brut` |
| V-PAS-07 | la couronne vaut `β` après chaque pas, même gelée au départ | doc (condition au bord) | aucune | égalité |
| V-PAS-08 | conservation de la masse, hors `γ` et bord : `|Σ après - Σ avant - γ · N_réceptives| <= 2 · cases` par pas, `R = 40`, 30 pas, 4 jeux | **dérivée de la doc** : erreur d'arrondi d'au plus 1,25 ulp par case (moyenne 0,5 ; produit par `α <= 2` : 1,5 ; moitié : 0,75, plus 0,5 du dernier arrondi) ; à `α` sans `1/2` l'écart est de l'ordre de la masse | borne dérivée, non élargie | `<=` |
| V-PAS-09 | masses jamais négatives, `0 <= α <= 2`, 100 pas depuis trois états | doc (preuve : `alpha * ecart >= -2u` donc `diffusion >= -u`) | aucune | `>= 0` |
| V-PAS-10 | `temps` compte les pas ; `avancer(n)` = `n` pas | doc | aucune | égalité |
| V-PAS-11 | les lectures ne changent rien | doc | aucune | égalité |

## formes closes

| Identifiant | Valeur | Source | Tolérance | Critère |
|---|---|---|---|---|
| V-EXACT-01 | `α = 0`, `β = 1/2`, `γ = 1/4`, `R = 8` : l'anneau `k` (`1 <= k <= R - 1`) vaut `β` jusqu'au temps `2(k - 1)` puis `β + (t - 2(k - 1)) γ` ; gèle au temps `2k` (`1/2 + 2/4 = 1` pile : le seuil est `>=`) ; centre `UN + t γ` ; couronne `β` ; cristal = hexagone plein de rayon `min(⌊t/2⌋, R - 1)` | **solution close dérivée à la main** : sans diffusion les non réceptives ne bougent pas ; un anneau devient de bord quand l'anneau intérieur gèle | aucune | tout l'état et les classes à chaque pas de 0 à 17 |
| V-EXACT-02 | `α = 0`, `β = 0.3`, `γ = 0.07` (`brut` 1288490189 et 300647711) : `m = ceil((2^32 - β)/γ) = 10` (`10 γ` dépasse de 3 ulp) | même forme close, `m` calculé en entiers par le test | aucune | tout l'état de 0 à `5 m + 2` |
| V-EXACT-03 | plaque sans vapeur `β = 0`, quel que soit `α` (1, 0.7, 2, 0.3), `γ` = 0.25, 0.1, 0.05 : `m = ceil(2^32 / γ)` = 4, 10, 20 ; l'anneau `k` vaut `max(0, t - m(k - 1)) γ` ; hexagone plein, compacité 1, 0 pointe | forme close : sans vapeur `u ≡ 0` donc la diffusion est nulle | aucune | tout l'état à chaque pas jusqu'à `4 m + 3` |

## le simulateur naïf et Python

| Identifiant | Valeur | Source | Tolérance | Critère |
|---|---|---|---|---|
| V-NAIF-01 | 7 jeux `(α, β, γ)`, `R` de 5 à 9, 40 à 70 pas : toutes les masses = celles du naïf à chaque pas | simulateur naïf du test (`etape_3_commun::Naif` : dictionnaire ordonné, `i128`, arrondis à la main, voisins dans un autre ordre) | aucune | égalité |
| V-NAIF-02 | `P1_AVANT`, `P2_AVANT`, `ASYM_AVANT` (`R = 6`, 4 cases gelées, une à `1 + 1 ulp`) : 40 pas contre le naïf | idem | aucune | égalité |
| V-NAIF-03 | 6 trajectoires : empreinte, gelées, rayon du cristal, pointes ; et `P3_EMPREINTE_20` | **Python** : SHA-256 de l'écriture canonique (`hashlib`) | aucune | égalité |

## symétrie

| Identifiant | Valeur | Source | Tolérance | Critère |
|---|---|---|---|---|
| V-SYM-01 | depuis le germe : défauts de rotation et de réflexion nuls à chaque pas (9 jeux, jusqu'à 300 pas) et les douze symétries de l'hexagone laissent l'état inchangé | doc : somme de six entiers, indépendante de l'ordre | aucune | égalité |
| V-SYM-02 | défauts à la main, `R = 1` : une voisine à `3/4` : (rotation, réflexion) = (2, 2) ; alternance : (6, 6) ; deux voisines contiguës (`k = 0, 1`) : (2, 0) ; toutes égales : (0, 0) | calcul à la main : rotation `k → k + 1`, réflexion `k → 1 - k` | aucune | égalité |
| V-SYM-03 | `ASYM` après 15 pas : défaut de rotation 90, de réflexion 86 | Python | aucune | égalité |
| V-SYM-04 | équivariance : pas(`g(S)`) = `g`(pas(`S`)) pour les 12 symétries, 2 états sans symétrie, 3 pas | propriété du modèle (la symétrie du germe ne la voit pas) | aucune | égalité |
| V-SYM-05 | `reflechir(q, r) = (r, q)`, involution, permutation des directions | doc | aucune | égalité |

## empreinte et aperçu

| Identifiant | Valeur | Source | Tolérance | Critère |
|---|---|---|---|---|
| V-EMP-01 | écriture canonique de `R = 1`, `α = 0.7`, `β = 0.3`, `γ = 0.05`, état initial : 101 octets, écrits un à un dans le test, égaux à ceux de Python | doc de l'écriture canonique ; SHA-256 par Python `hashlib` | aucune | égalité |
| V-EMP-02 | la même après un pas ; temps = 1 aux octets 37 à 44 | Python | aucune | égalité |
| V-EMP-03 | longueur `45 + 8 · cases` et chaque champ à sa place, `R` de 1 à 8 | doc | aucune | égalité |
| V-EMP-04 | sensibilité : un `ulp` de `α`, `β` ou `γ`, un rayon, le temps seul, une masse, deux masses échangées changent l'empreinte ; mêmes entrées, même empreinte | doc | aucune | `!=` et `==` |
| V-APER-01 | aperçu de `P1_AVANT` : 75 octets | Python d'après la doc de `apercu` | aucune | égalité |
| V-APER-02 | correspondance case-pixel `(q + R + ⌊r/2⌋, r + R)` pour `R` de 1 à 6, couleurs, pixels des coins noirs | doc, réécrite dans le test | aucune | égalité |
| V-APER-03 | niveaux `0, 63, 127, 191, 254` pour `0, 1/4, 1/2, 3/4, UN - 1` | calcul à la main : `⌊255 · m⌋` | aucune | égalité |

## lancer

| Identifiant | Valeur | Source | Tolérance | Critère |
|---|---|---|---|---|
| V-LANC3-01 | 4 scénarios : CSV, empreinte, SHA-256 des octets RVB de l'aperçu (PNG relu par le décodeur indépendant de l'étape 0) | Python | aucune | égalité |
| V-LANC3-02 | quatre graines, mêmes sorties | doc (aucun hasard) | aucune | égalité |
| V-LANC3-03 | 26 refus avec la ligne fautive, trois formes acceptées | doc de `lancer_reiter` | aucune | égalité |
| V-LANC3-04 | `pas` maximal 1 000 000 accepté, 1 000 001 refusé ; CSV de 21 lignes | doc | aucune | égalité |
| V-LANC3-05 | les 3 scénarios d'exemple du dépôt : CSV et empreinte | Python | aucune | égalité |
| V-LANC3-06 | le binaire : code 0 et trois fichiers égaux à Python ; un scénario refusé : code 1, `ligne 2`, aucun dossier | doc de `src/main.rs` (étape 0) | aucune | égalité |

## publiées (Reiter)

| Identifiant | Valeur | Source | Tolérance | Critère |
|---|---|---|---|---|
| V-PUB-01 | ligne `β = 0` de la table de rayon 100 : `γ` = 0.01, 0.02, 0.05, 0.1, 0.5, 1 → 8499, 4249, 1699, 849, 169, 84 ; `γ <= 0.005` et `γ = 0` → 9999 | table publiée (`th_nn_h0_x<γ>_<n>.png`, lue une fois). **Lecture de `n` : nombre de pas moins 1 jusqu'à ce que la glace atteigne le cercle intérieur, plafonné à 10 000 pas.** Vérifiée par le juge dans le code de l'auteur (`sfn.ijs`, page `.../sfn/sfn.html` : `mk_dla` rend `k - 1`, `k` le nombre de pas faits, et s'arrête quand une glace a une distance euclidienne `>= 98 sin 60°`) et par le calcul (les 240 numéros retrouvés avec la géométrie de l'auteur) ; le script qui a produit la table elle-même n'est pas publié. Elle est aussi fondée sur : la ligne `β = 0` est exactement `ceil(85/γ) - 1` (forme close V-EXACT-03) ; 9999 = 10 000 pas ; toutes les lignes `β >= 0.9` donnent 84 (un anneau par pas) | aucune | égalité entière, puis simulation du rayon 100 pour `γ` = 1, 0.5, 0.1 : exactement `85 m` pas |
| V-PUB-02 | 23 cases de la table (voir le test), `α = 1`, rayon de région 100 : le nombre de pas pour que le rayon du cristal atteigne 85 est `n + 1` | table publiée ; les 240 noms relus par le juge dans le HTML de la page (identiques) ; ces 23 cases sont parmi les 181 où la couronne de l'hexagone de rayon 100 et le bord circulaire de l'auteur donnent le même nombre (calcul du juge, `table_juge.rs`) | aucune | égalité entière |
| V-PUB-03 | `β = 0`, `γ = 0` : rien ne grandit, 300 pas | préimpression p. 5 : « When β = 0 and γ = 0, no growth appears » ; page : `th_nn_h0_x0_09999.png` | aucune | état inchangé |

**Comparaison de toute la table.** `comparaison_table_rayon100_hexagone.txt` (notes de l'écrivain) et `table_hex100_fixe.txt` (outil du juge,
recalcul indépendant : mêmes nombres) comparent les 240 cases au modèle sur l'hexagone de rayon 100 : 21 cases plafonnées (9999), d'accord ; des 219 autres,
**160 sont exactement égales**, 184 à moins de 0,5 %, 192 à moins de 1 %, 219 à moins de 5 %, le modèle étant toujours **plus lent**. Toutes les cases `β >= 0.55`
sont exactes, ainsi que toutes celles où `γ >= 0.1` ; les 59 cases inexactes ont `β` de 0.05 à 0.5 et `γ <= 0.05` (au pire `β = 0.2`, `γ = 0` : 9435 pas contre
8995 publiés, + 4,9 %). **Valeur publiée non atteinte, cause établie par le juge : le bord.** Le script J de l'auteur (`sfn.ijs`, fonctions `mk_dla` et `cir`)
range l'hexagone dans un tableau de `202 x 202` aux rangées espacées de `sin 60°` et décalées d'une demi-case, fixe à `β` après chaque pas toutes les cases à
distance euclidienne `d² >= (100 sin 60°)² = 7500` du germe (`ocir`), et s'arrête dès qu'une glace a `d² >= (98 sin 60°)² = 7203` (`icir`). Son bord est donc un
cercle de rayon 86,6, à 1,6 case seulement des pointes quand elles atteignent 85, alors que la couronne de l'hexagone de rayon 100 est à 15 cases des pointes
(les pointes vont vers les sommets de l'hexagone). Avec la géométrie de l'auteur, les mêmes règles et les arrondis de la doc de `flocon`, l'outil du juge
(`table_juge.rs disque`) redonne **les 240 numéros, tous exactement** (`table_disque_fixe.txt` : 240/240). Les règles du modèle sont donc celles de l'auteur ;
seule la géométrie du bord diffère, et elle ne compte que pour les dendrites que la vapeur nourrit. Le test ne prend que des cases où les deux géométries donnent le
même nombre. Le modèle garde la couronne hexagonale (la symétrie d'ordre 6 et la couronne tiennent en entiers, sans flottant).

## mesurées

Toutes les mesures ont été faites sur le prototype honnête, **à 3 tailles de région (85, 100, 120) et 7 rayons de cristal (20 à 50)** ; les nombres sont
**identiques d'une taille de région à l'autre** (le modèle n'a pas de hasard, et le bord est assez loin). Sorties brutes : `scan_formes.txt`, `scan2.txt`,
`vitesses_fenetres.txt` (notes de la session). Les bornes des intervalles sont arrondies vers l'extérieur à la deuxième décimale (compacité) ou élargies de 1 %
(pente) ; la raison : la valeur est exacte pour toute implémentation fidèle (aucun hasard), l'intervalle est lisible, il dit « la classe », non « la valeur ».

| Identifiant | Valeur | Source | Tolérance | Critère |
|---|---|---|---|---|
| V-FORM-01 | classes de formes. **Plaque** `(β = 0, γ = 0.05)` et `(0.9, 0.1)` : compacité exactement 1, 0 pointe ; **étoile pleine** `(0.45, 0.1)` : compacité 0.7764 à 0.7914, exactement 6 pointes ; **étoile frangée** `(0.5, 0.05)` : 0.7017 à 0.7420 ; **secteur** `(0.95, 0)` : 0.5485 à 0.6290 ; **dendrite** `(0.4, 0.001)` : 0.2911 à 0.3577, 18 à 114 pointes ; **dendrite fine** `(0.35, 0)` : 0.2025 à 0.2958, 6 à 198 pointes | **mesure** sur le prototype ; les formes concordent avec les vignettes de la table publiée aux mêmes paramètres (plaque : hexagone plein ; `h0o45_x0o1` : étoile pleine à six pointes ; `h0o5_x0o05` : étoile à six pointes ; `h0o95_x0` : secteur à motif fractal ; `h0o35_x0`, `h0o4_x0o001`, `h0o4_x0o0001` : dendrites ; `h0o8_x0o002` : six pétales fins) ; la classe « étoile » n'est pas nommée par l'article : elle vient de la phrase « for high levels of γ we obtain stellar ... » et de ces vignettes | intervalles : `[1, 1]`, `[0.77, 0.80]`, `[0.70, 0.75]`, `[0.54, 0.63]`, `[0.29, 0.36]`, `[0.20, 0.30]` ; pointes : exactement 6 pour l'étoile pleine, au moins 12 pour la dendrite (min mesuré 18), au moins 6 pour la fine, 0 pour les plaques | chaque rayon de 20 à 50 dans son intervalle |
| V-FORM-02 | ordre : à rayon 40 et 50, compacité plaque (1) > étoile pleine (0.789, 0.791) > secteur (0.615, 0.614) > dendrite (0.309, 0.291) > dendrite fine (0.227, 0.203) ; pointes de la fine (138, 198) > 6 | mesure | strict | `>` |
| V-FORM-03 | vitesse radiale : pente de `rayon du cristal` contre `temps` sur les fenêtres `[30, 55] [20, 50] [25, 55] [30, 50]` : plaque 0.049887 à 0.049962 (la valeur close est `γ = 0.05`, au pas d'escalier près) ; étoile `(0.45, 0.1)` 0.249468 à 0.249819 ; dendrite `(0.4, 0.001)` 0.047511 à 0.047582 ; dendrite fine 0.031179 à 0.031226 ; `R²` de 0.99773 à 0.99928 | mesure sur 8 fenêtres (`vitesses_fenetres.txt`) ; **comparaison à la publication** : la vitesse moyenne jusqu'au rayon 85 vaut 85/339 = 0.2507 (étoile : le prototype 0.2495-0.2498), 85/1700 = 0.0500 (plaque), 85/1738 = 0.0489 (dendrite `(0.4, 0.001)` : le prototype donne 85/1746 = 0.0487, 0,5 % plus lent : valeur publiée non atteinte, voir ci-dessus) | `[0.99 min, 1.01 max]` : un anneau gagné par `m` pas donne un effet d'escalier d'environ 0,1 %, la marge est dix fois plus large ; `R² > 0.995` (le minimum mesuré est 0.99773 : l'escalier limite le `R²`) | pente dans l'intervalle et `R²` au-dessus, pour 4 fenêtres |
| V-FORM-04 | indépendance de la région : régions 85, 100, 120, mêmes relevés (temps, gelées, compacité, pointes) à chaque rayon de 20 à 50, pour la dendrite, l'étoile, la dendrite fine | mesure : écart nul | 0 | égalité des relevés |
| V-FORM-05 | `α` = 0.5, 0.501, 0.502, 1 à `β = 0.4`, `γ = 0.0001` : dendrites (compacité 0.2291 à 0.2589, au moins 66 pointes aux rayons 35 à 50) ; nombres de gelées au rayon 40 : 1261, 1177, 1201, 1189 (deux à deux différents) ; temps pour atteindre le rayon 50 : 2064 contre 1058 (rapport 1.951) | texte de la préimpression, figure 5 (« very different pictures; ... all exhibit dendrite growth ») ; mesure | compacité `[0.22, 0.26]`, pointes `>= 60`, rapport `[1.90, 2.00]` | dans les intervalles ; nombres différents |

---

## Le bord circulaire (étape 3c)

Valeurs des tests `tests/etape_3c_*.rs` (identifiants `V-BORD-xx`). **Outils indépendants** : Python 3.13, `modele_independant_3c.py` (le modèle de la doc de `flocon` avec un bord au
choix, écrit avec des entiers de taille quelconque et `hashlib`, étendant l'outil du juge de l'étape 3 `modele_independant.py` ; il redonne, à l'octet, le modèle du juge quand le bord est la couronne) et
`valeurs_etape3c.py` (il écrit `tests/etape_3c_commun/valeurs.rs`) ; l'outil du juge de l'étape 3 `table_juge.rs` (Rust autonome, géométrie de l'auteur : 240/240). **Aucune valeur de cette section ne
vient de notre code Rust**, sauf les mesures « avant/après » de V-BORD-29 (dites telles). Les tests de ce fichier sont tous « juste », aucun n'est une « régression ». Tolérance : aucune (des entiers, des
octets, des chaînes), sauf mention. Les notes et sorties brutes sont dans `essai-3c-outils/` des notes de la session.

**Sources.** (1) Le script J de l'auteur, `sfn.ijs` (page `webbox.lafayette.edu/~reiterc/mvp/sfn/sfn.html`, lu par le juge de l'étape 3 le 8/11/2026) : après chaque pas `b = ocir * β + b * (1 - ocir)`, où `ocir`
est l'ensemble des cases du tableau de `202 x 202` (rangées espacées de `sin 60°`, décalées d'une demi-case) à `d² >= (100 sin 60°)² = 7500` du germe ; arrêt dès qu'une glace est dans `icir`, `d² >= (98 sin 60°)² = 7203` ;
la fonction rend `k - 1`. (2) La table β-γ de rayon 100 de l'auteur (`.../sfn/nn/index.html`, 240 noms de fichiers, relus par le juge de l'étape 3). (3) Le calcul exact : `x = q + r/2`, `y = r sin 60°`,
`x² + y² = q² + q r + r²`, donc `d >= R sin 60°` s'écrit `4 (q² + q r + r²) >= 3 R²` (car `sin² 60° = 3/4`).

| Identifiant | Valeur | Source | Tolérance | Critère |
|---|---|---|---|---|
| V-BORD-01 | nombre de cases du bord pour chaque `R` de 1 à 100 : couronne `6 R` ; circulaire `6 R` jusqu'à `R = 7`, puis 54, 60, 66, 84, 90, 96, 114... (`R` = 8, 9, 10, 11, 12, 13, 14...), 3126 pour `R = 100` | Python entiers (`valeurs_etape3c.py`, `NOMBRES_DU_BORD`) | aucune | égalité |
| V-BORD-02 | liste complète (ordre canonique) du bord circulaire pour `R` = 3, 5 (la couronne), 8 (54 cases : la couronne et les coins `(7, 0)`... de la rangée 7), 10 (66), 14 (114) | Python entiers | aucune | égalité |
| V-BORD-03 | `R = 14` : 12 cases à distance hexagonale `< 14` avec `4 · distance2 = 3 R² = 588` exactement (`147 = 3 · 7²`) ; elles sont du bord circulaire (comparaison large). Les milieux de côté `(R/2, R/2)` (`R` pair) aussi | Python entiers ; calcul à la main | aucune | égalité |
| V-BORD-04 | masses posées (formule + cinq gelées dont une sur la couronne et une exactement au bord circulaire), `R` = 9 et 14, deux bords : toutes les masses après 1 pas, après 20 pas, l'empreinte, les gelées, `distance2_cristal` | Python entiers | aucune | égalité |
| V-BORD-05 | six trajectoires depuis le germe, `R` de 10 à 16 : empreintes avec chaque bord (toutes différentes, masses comprises), gelées, rayon, pointes, `distance2_cristal` | Python entiers | aucune | égalité |
| V-BORD-06 | écriture canonique de `R = 1`, `α = 0.7`, `β = 0.3`, `γ = 0.05` avec le bord circulaire : celle de l'étape 3 (101 octets) avec l'octet de forme 2 ; son SHA-256 ; la même après 1 pas | écrite à la main d'après la doc ; Python `hashlib` | aucune | égalité |
| V-BORD-07 | `R = 100` : bord circulaire = `distance2 >= 7500` : 3126 cases, plus petit `distance2` du bord 7500, plus grand `distance2` hors du bord 7492 | Python entiers ; `ocir` de `sfn.ijs` | aucune | égalité |
| V-BORD-08 | `distance2` : 0 au centre, 1 aux six voisins, `(3, 4)` 37, `(3, -4)` 13, `(5, 3)` 49, `(49, 49)` 7203, `(7, 0)` 49 contre `(4, 4)` 48 ; `4 · distance2 = (2q + r)² + 3 r²` pour `q, r` de -30 à 30 ; exact sur les extrêmes d'`i32` | calcul à la main ; autre écriture du même entier ; entiers 128 bits | aucune | égalité |
| V-BORD-09 | `distance2` invariante par les douze symétries de l'hexagone | propriété (identité algébrique) | aucune | égalité |
| V-BORD-10 | la couronne est dans le bord circulaire pour `R` de 1 à 40 ; le bord hexagonal est exactement la couronne ; égal à la couronne pour `R <= 7`, strictement plus grand pour `R` de 8 à 40 | preuve dans la doc de `flocon` (« Le bord », fait 1) ; vérifiée par énumération | aucune | égalité |
| V-BORD-11 | le bord (les deux) est stable par les douze symétries, `R` de 1 à 30 | propriété | aucune | égalité |
| V-BORD-12 | `est_du_bord` faux hors de l'hexagone (y compris `i32::MIN`, `i32::MAX`), le centre n'est jamais du bord, `bord()` ne change pas | doc | aucune | égalité |
| V-BORD-13 | `R = 8`, `α = 1`, `β = 0`, `γ = 0`, toutes les masses nulles sauf `(7, 0)` à 1/2 : bord circulaire, après un pas, les trois cases `(6, 0)`, `(6, 1)`, `(7, -1)` valent `178956971` et le reste 0 ; bord hexagonal : `(7, 0)` vaut `1073741824` en plus | calcul à la main (`357913941 = ⌊(2^32 + 6) / 12⌋`, `178956971 = ⌊(2 · 357913941 + 2) / 4⌋`, `-2^30`) ; Python | aucune | égalité |
| V-BORD-14 | après chacun des 11 premiers pas, toutes les cases du bord valent exactement `β` ; l'intérieur n'est pas partout `β` | doc, bord calculé par le test d'une autre écriture | aucune | égalité |
| V-BORD-15 | le simulateur naïf du test (dictionnaire, `i128`, voisins dans un autre ordre, bord `(2q + r)² + 3 r² >= 3 R²`) donne les mêmes `brut` à chaque pas : 8 jeux de paramètres, `R` de 8 à 16, 2 bords ; 5 états posés | doc | aucune | égalité |
| V-BORD-16 | `depuis_masses` reste hexagonal ; il diffère du bord circulaire dès le premier pas pour `R = 9` | doc | aucune | égalité |
| V-BORD-17 | l'état initial de `avec_bord`, quel que soit le bord : `β` partout, le germe à 1 au centre, une seule gelée, `distance2_cristal` 0 | doc | aucune | égalité |
| V-BORD-18 | `new` égal à `avec_bord(.., Hexagonal)` à l'octet (initial et après 30 pas), et les références de l'étape 3 (Python) inchangées : `ECRITURE_INITIALE_SHA256`, six trajectoires, `P3_EMPREINTE_20` | Python (étape 3) | aucune | égalité |
| V-BORD-19 | deux bords, deux empreintes ; pour `R <= 7` les masses sont égales et seul l'octet 8 diffère ; pour `R >= 8` les masses diffèrent aussi | doc, Python | aucune | égalité |
| V-BORD-20 | symétrie exacte avec le bord circulaire : 7 jeux à `R = 15`, 60 pas, 2 à `R = 40`, 300 pas ; l'état diffère de celui du bord hexagonal pour `β > 0` | doc : `distance2` invariante | aucune | égalité |
| V-BORD-21 | équivariance sous les douze symétries avec le bord circulaire : deux états sans symétrie (`R` = 9, 14), 3 pas | propriété du modèle | aucune | égalité |
| V-BORD-22 | `distance2_cristal` à la main : aucune gelée 0 ; le germe seul 0 ; `(3, 4)` 37 ; `(7, 0)` et `(4, 4)` : 49 (et `rayon_cristal` 8) ; `UN - 1` non gelée, `UN` gelée ; `(-3, -1)` et `(2, -5)` : 19 | calcul à la main ; Python | aucune | égalité |
| V-BORD-23 | une case du bord gelée compte (`(-8, 0)` : 64 ; `(0, -9)` : 81, `R = 9`) ; les cases du bord sont remises à `β` au pas suivant | doc | aucune | égalité |
| V-BORD-24 | `distance2_cristal` ne change pas par les douze symétries | propriété | aucune | égalité |
| V-BORD-25 | `distance2_cristal` à chaque pas égal au plus grand `distance2` des cases de masse `>= 1`, recalculé par le test (`R = 30`, 120 pas) ; ne décroît jamais | doc | aucune | égalité |
| V-BORD-26 | `R = 100` : `(31, -96)` 7201 (pas d'arrêt), `(49, -98)` et `(49, 49)` 7203 (arrêt), `(48, -98)` 7204 ; aucune case à 7202 | Python entiers ; `icir` de `sfn.ijs` | aucune | égalité |
| V-BORD-27 | 20 cases de la table publiée (8 aujourd'hui fausses, dont un écart de 4,1 %, et 12 aujourd'hui exactes) : `n` calculé = `n` publié avec `Bord::Circulaire` | table de l'auteur ; boucle de `mk_dla` (`sfn.ijs`) ; vérifiées sur les 240 par l'outil du juge | aucune | égalité entière |
| V-BORD-28 | les 240 cases : `n` calculé = `n` publié (219 égales, 21 plafonnées sans arrêt avant le 10000e pas) | idem | aucune | égalité entière (test ignoré par défaut) |
| V-BORD-29 | **mesure avant/après (notre code, dite telle)** : sur les 240 cases, le même calcul avec `Bord::Hexagonal` donne 181 numéros égaux à la table (160 non plafonnés et 21 plafonnés) avec l'arrêt `distance2 >= 7203` et aussi avec l'arrêt `rayon_cristal >= 85` ; avec `Bord::Circulaire` : 240 | notre prototype (`examples/mesure_table_3c.rs`), recoupé par l'outil du juge (`table_hex100_fixe.txt` : 181/240 ; `table_disque_fixe.txt` : 240/240) | aucune | 181 puis 240 |

**Ce que la mesure « avant » dit.** Les 59 cases que le bord hexagonal rate ont `β` de 0,05 à 0,5 et `γ <= 0,05` (voir `MODELE.md`, « Ce qui est mesuré »), de + 0,09 % (1 pas) à + 4,90 % (`β = 0,2`, `γ = 0` : numéro 9435 contre 8994 publié).
Les 21 cases plafonnées et les 160 non plafonnées exactes le restent avec le bord circulaire.

**Relu par le juge de l'étape 3c (8/11/2026).** Le script `sfn.ijs` (page `sfn.html`, rouverte) : `k = 0`, puis `while. (roomq b_sf) *. k < {:x.` (un pas, `k = k + 1`), et
`(k-1),x.` ; `roomq` est vrai tant qu'aucune case de `icir` n'a une valeur `>= 1` ; `cir` prend pour seuils le plus petit `x² + y²` de la première rangée (`ocir`) et de la troisième
(`icir`). Le plafond n'est pas écrit dans le script (2000 dans l'appel d'exemple et par défaut de `mk_dla`, 100 000 par défaut de `mk_hxs`) : les 21 numéros 9999 de la table disent un plafond de 10 000 pas. Le calcul en
`f64` du script, refait en Python pour `R = 100`, donne **exactement** les mêmes cases que les entiers : `ocir` = `4 · distance2 >= 3 · 100²` et `icir` = `distance2 >= 7203`
(0 case de différence ; `icir` vaut `7202,999999999999` en `f64`), et toute case du tableau de l'auteur hors de l'hexagone ou sur la couronne est dans `ocir`. Les 240 noms de
`.../sfn/nn/index.html` (page rouverte, et le HTML téléchargé par le juge de l'étape 3, lu à part) sont les 240 `(β, γ, n)` de `tests/etape_3c_table.rs`, dans le même ordre.

---

## Ce que cette étape n'établit pas

- **Que le modèle est fidèle à l'article au-delà des cas lus.** Les règles viennent de la préimpression ; la version éditée (Chaos, Solitons & Fractals 23) n'a pas été lue ; les
  figures 3 à 7 non plus (des images). La table publiée est reproduite entièrement avec `Bord::Circulaire` (240 cases sur 240, étape 3c, section « Le bord circulaire »
  ci-dessus) ; avec le bord hexagonal par défaut, seulement 181 cases sur 240 : les dendrites que la vapeur nourrit (`β <= 0.5`, `γ <= 0.05` : 59 cases) y sont jusqu'à 4,9 %
  plus lentes. La cause est établie : le bord circulaire de rayon 86,6 de l'auteur.
- **La lecture du numéro de fichier de la table.** Le script d'animation de l'auteur rend `k - 1` ; le script qui a produit la table n'est pas publié. Les 240
  numéros retrouvés exactement avec la géométrie de l'auteur rendent toute autre lecture invraisemblable.
- **Le bord circulaire dans le reste de la chaîne** : il n'est qu'une option de la bibliothèque (étape 3c) ; ni `lancer`, ni la vitrine, ni la page ne le proposent. Pour un rayon
  d'au moins 96, le bord hexagonal ne change rien aux mesures de cristaux de rayon 85 (mesuré pour une dendrite), mais pas pour une région plus petite.
- **Les `α > 2`** de la figure 5 de l'article (2.003 « many levels of dendrites » ; 2.69 « explosive »).
- **La ressemblance visuelle** : les classes se jugent par la compacité et les pointes ; les aperçus sont pour l'œil. Les vignettes de l'auteur ont été regardées une à une
  (dix), pas comparées par un calcul.
- **Le diagramme de morphologie de Libbrecht** (température, sursaturation) : le modèle de Reiter n'a pas de température.
- **La reproductibilité entre Linux et Windows** : tout est en entiers, mais seule la CI la rejoue (l'empreinte de `references/reiter-plaque.empreinte`).
- **La physique** : Reiter dit lui-même qu'il ne cherche pas à ajuster les équations de la physique ; Gravner et Griffeath notent que son taux d'attachement constant
  ne donne ni facettes ni instabilités correctes.

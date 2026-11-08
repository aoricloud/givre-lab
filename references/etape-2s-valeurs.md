# Étape 2s : les valeurs attendues, leur source, leur tolérance

Cette fiche dit, pour chaque valeur qu'un test de l'étape 2s (la croissance de surface en bande) attend, d'où elle vient, avec quelle
tolérance et quel critère de réussite. Chaque identifiant (`V-MOD-01`…) est cité dans la doc des tests qui l'emploient
(`tests/etape_2s_*.rs`). Elle est séparée des résultats figés de notre propre code (`references/*.empreinte`, étiquetés « régression »
dans les tests) : **aucune valeur de cette fiche ne vient du code Rust qu'elle juge**. Une valeur y vient d'une publication lue
(avec ce qui a été lu, ou « non lue »), d'un calcul exact fait par un outil indépendant (nommé), ou de la définition que la documentation
du module `surface` fixe (dite telle).

**Outils indépendants** : Python 3.13 (`fractions` : rationnels exacts ; `hashlib` ; `itertools`) ; `ref_alea.py` (transcription du C de
référence de PCG32, validée sur le vecteur officiel) ; **`ref_surface.py`, un second prototype en Python des quatre modèles**, écrit
d'après la documentation de `src/surface.rs` et PCG32 de `ref_alea.py` : autre langage, autre structure de données (Eden : un dictionnaire
d'états ; périmètre : une liste) ; `variance_depot.py` (variance de `w²` du dépôt aléatoire, trois façons) ; `valeurs_ecriture.py`
(écriture canonique et SHA-256). Les scripts sont dans `givre/essai-2s-outils/` des notes de la session. Le code Rust n'est jamais l'outil :
le prototype Rust et le second prototype Python donnent les MÊMES hauteurs et les MÊMES octets d'écriture canonique (SHA-256 identiques) sur
28 cas (4 modèles × 7 couples largeur/graine) ; c'est ce qui valide la doc comme spécification autonome.

**Ce que cette fiche dit des tolérances** : les tests exacts n'en ont aucune. Les tolérances des tests statistiques sont soit CALCULÉES
(le dépôt aléatoire : la variance exacte de `w²`, V-RD-02), soit MESURÉES sur le prototype sur 60 groupes indépendants de graines
(les exposants, V-EXP-xx) et jamais élargies pour un cas.

---

## Les références lues (et ce qui n'a pas pu l'être)

| Id | Source | Ce qui a été lu | Ce qui dit la source | Statut |
|---|---|---|---|---|
| V-REF-01 | F. Family et T. Vicsek, *Scaling of the active zone in the Eden process on percolation networks and the ballistic deposition model*, J. Phys. A 18 (1985) L75–L81 | le résumé (page de l'éditeur et notices bibliographiques) ; pas le texte | pour le dépôt balistique, la largeur de la zone active ne diverge pas comme la hauteur moyenne ; la forme de scaling `w(L, t) ~ L^α f(t / L^z)` est attribuée à cet article par Family 1986 (résumé IOP) et par Penrose, arXiv:math/0608540 (réf. [7]) | **texte non lu** ; le résumé suffit pour la forme de scaling, pas pour des valeurs |
| V-REF-02 | F. Family, *Scaling of rough surfaces: effects of surface diffusion*, J. Phys. A 19 (1986) L441–L446 | le résumé (IOP, qui dit : dépôt aléatoire avec diffusion de surface en 2D ; largeur auto-affine ; même forme de scaling que Family-Vicsek 1985 mais d'autres exposants ; autre classe d'universalité que le balistique et le remplissage aléatoire) ; le texte est payant | la règle du modèle et les valeurs sont lues dans des sources SECONDAIRES : Mal, Ray, Shamanna, arXiv:0805.0647 (2008) et arXiv:1005.3513 (2010) ; Roma, Horowitz, Albano, arXiv:cond-mat/0210200 (2002) | **texte non lu** ; voir V-REF-02a |
| V-REF-02a | Mal, Ray, Shamanna, arXiv:1005.3513 (texte lu) ; arXiv:0805.0647 (texte lu) ; Roma, Horowitz, Albano arXiv:cond-mat/0210200 (texte lu) | la définition du modèle de Family | **1005.3513** : la particule déposée en `i` « colle au sommet de la colonne `i`, `i+1` ou `i-1` selon laquelle des trois est la plus basse » ; si les trois ont la même hauteur « elle reste en `i` et ne diffuse pas » ; si `i+1` et `i-1` sont égales et plus basses que `i`, elle va à l'une des deux avec la même probabilité. **0805.0647** : même phrase, mais « si `i`, `i+1` et `i-1` ont la même hauteur, la particule colle à `i+1` ou `i-1` avec la même probabilité ». **cond-mat/0210200** : la particule « peut relaxer vers une colonne voisine si sa hauteur est inférieure à celle de la colonne choisie » ; la figure 1(b) montre un cas où elle va à droite ou à gauche avec la même probabilité | **DIVERGENCE entre les sources sur le cas d'égalité des trois hauteurs** (reste en `i` pour 1005.3513 ; va à un voisin pour 0805.0647). La doc de `surface` retient « ne quitte `i` que si un voisin est STRICTEMENT plus bas » (accord de 1005.3513 et de cond-mat/0210200). C'est une convention de ce dépôt, dite telle ; l'autre lecture est une autre dynamique, de même classe. **Relu par le juge** (textes ouverts) : 1005.3513 §II et 0805.0647 §II disent bien ce qui est cité ; Chou et Pleimling, arXiv:1007.2380 §2.1 : la particule saute vers un voisin « si ce site a une hauteur plus basse » (strictement), et leur modèle se réduit à celui de Family à température nulle ; Raychaudhuri et al., arXiv:cond-mat/0105483 : « en cas d'égalité impliquant le site i » la particule reste en i, sinon l'égalité se tranche au hasard. La lecture majoritaire est donc celle de la doc. Le texte de Family 1986 et Barabási-Stanley ch. 5 restent non lus. L'exposant n'en dépend presque pas (V-EXP-06) |
| V-REF-03 | M. Kardar, G. Parisi, Y.-C. Zhang, *Dynamic scaling of growing interfaces*, Phys. Rev. Lett. 56 (1986) 889 | la notice et le résumé (nucssp.rmki.kfki.hu) : « le profil d'une interface qui croît », forme de scaling dynamique exacte en dimension 1, « en excellent accord avec les simulations précédentes » ; la liste de références cite Eden 1961 | aucun nombre dans la notice | **Juge** : texte lu (copie du cours UCSD Physics 116, OCR) : `z = 3/2` exact par fluctuation-dissipation (p. 889), `x = 1/2` et `β = x / z = 1/3` (p. 891, fractions reconstruites d'un OCR abîmé) ; confirmé proprement par Kardar, arXiv:cond-mat/9507019, p. 5 : en `d = 2` (1 + 1) « α = 1/2 et β = 1/3 ». Chez l'écrivain : texte non lu. `β = 1/3`, `α = 1/2`, `z = 3/2` en 1 + 1 dimensions : lus dans H. Spohn, arXiv:1601.00499 (« les fluctuations typiques de hauteur sont d'ordre `t^{1/3}` », exposant de hauteur 1/2, `z = 3/2`) et rapportés par Penrose, arXiv:math/0608540, qui cite Barabási-Stanley p. 56 pour « α = 1/2 et β = 1/3 » en `d = 1` |
| V-REF-04 | S. F. Edwards, D. R. Wilkinson, *The surface statistics of a granular aggregate*, Proc. R. Soc. Lond. A 381 (1982) 17 | rien | — | **non lu**. `β = (2-d)/4 = 1/4`, `α = (2-d)/2 = 1/2`, `z = 2` en `d = 1` : résultats de recherche (résumés d'arXiv:cond-mat/0601567, arXiv:1008.0119, arXiv:2505.05311), non ouverts. **Juge** : l'article lui-même reste non lu, mais Kardar, arXiv:cond-mat/9507019, éq. (9) (texte lu) donne pour l'équation d'Edwards-Wilkinson `z = 2`, `α = (3-d)/2`, `β = (3-d)/4`, « en d = 2, α = 1/2 et β = 1/4 » (`d` compte la hauteur : 1 + 1) : **vérifiée sur une revue** |
| V-REF-05 | M. Eden, *A two-dimensional growth process*, Proc. 4th Berkeley Symp. on Math. Stat. and Prob., vol. IV (1961) 223–239 | rien (la référence est confirmée par arXiv:2005.12349, qui la cite) | — | **non lu**. La définition « un site vide choisi uniformément parmi tous ceux qui touchent la structure » (Eden « A », site perimeter) est lue dans Manin, Roldán, Schweinhart, arXiv:2005.12349 (ar5iv) : « on ajoute une tuile choisie uniformément parmi toutes les tuiles adjacentes à la structure et pas encore dedans ». `β = 1/3` pour Eden (classe KPZ) : arXiv:1805.11061 (« appartient à la classe KPZ », `β = 1/3`, `α = 1/2`, `z = 3/2`) ; arXiv:cond-mat/0611159 (Ferreira, Alves : `β₀ = 0,333 ± 0,010` pour l'Eden hors réseau radial, origine fixe) |
| V-REF-06 | A.-L. Barabási, H. E. Stanley, *Fractal Concepts in Surface Growth*, Cambridge UP (1995) | la table des matières (chapitre 5 « Linear theory », section 5.1 « Random deposition with surface relaxation ») ; la page 56 n'est lue que par ricochet (Penrose) | — | **non ouvert** |
| V-REF-07 | la règle du dépôt balistique `h[i] = max(h[i] + 1, h[i-1], h[i+1])` | aucun texte ne la donne mot pour mot parmi ceux lus : Roma et al. (cond-mat/0210200) décrivent « la particule tombe verticalement jusqu'à l'interface, où elle colle » ; Penrose (math/0608540) en donne la forme `last-arrival` | un dépôt vertical qui colle au premier contact (dessus de la colonne `i` ou côté d'une colonne voisine) s'arrête à la hauteur `max(h[i] + 1, h[i-1], h[i+1])` dans la représentation sans surplomb : c'est la forme usuelle de la règle (Barabási-Stanley) ; ici, une équivalence géométrique que la doc fixe | **Juge : vérifiée** : B. Farnudi et D. D. Vvedensky, *Large-scale simulations of ballistic deposition: the approach to asymptotic scaling*, Phys. Rev. E 83 (2011) 020103(R), éq. (2) (texte lu) : `h_i(n+1) = max[h_{i-1}(n), h_i(n) + 1, h_{i+1}(n)]`, la particule « se fixe irréversiblement au premier voisin vertical ou latéral rencontré » ; c'est la règle de la doc |
| V-REF-08 | Eden dans la bande de largeur `L` périodique, avec le périmètre en `Vec` et `swap_remove` | — | l'ordre du `Vec`, `swap_remove`, l'ordre des voisins et la capacité sont des CONVENTIONS de la doc de `surface` (pour que le hasard soit consommé de façon identique par toute implémentation), pas des résultats publiés | convention |

---

## Les modèles : valeurs exactes (V-MOD)

| Id | Valeur | Source | Tolérance | Critère |
|---|---|---|---|---|
| V-MOD-01 à 04 | les hauteurs de `Surface` (et, pour Eden, le périmètre dans l'ordre du `Vec` et l'occupation de chaque site) égales à celles d'un simulateur naïf écrit dans `tests/etape_2s_commun/mod.rs`, à chaque particule, 8 largeurs (1, 2, 3, 4, 5, 8, 13, 32) × 8 graines (1 à 8) × 300 ou 700 particules | la règle de la doc de `surface` répétée mot pour mot, avec la même consommation du hasard (PCG32, `Alea::borne`) ; le simulateur naïf partage avec `Surface` seulement `Alea` (étape 0, testé contre le vecteur officiel) | aucune | égalité exacte ; les compteurs de branches du simulateur naïf (relaxation : reste strict, reste avec égalité, gauche, droite, pile, face ; balistique : trois origines du maximum ; Eden : dernier élément ou autre) dépassent chacun 50 : le test exerce toutes les branches |
| V-MOD-05 | l'état du générateur après 500 particules (L = 7, 20 graines, trois modèles) est celui du simulateur naïf | doc : nombre de tirages exactement fixé (pièce seulement dans le cas `hg == hd < hi`) | aucune | égalité exacte des 16 octets |
| V-MOD-06 | `t` couches = `t·L` particules ; `couches() = particules / L` (division entière) | définition de la doc | aucune | égalité |
| V-MOD-07, 08 | déterminisme ; noms `depot`, `relaxation`, `balistique`, `eden`, `depuis_nom` exact | doc | aucune | égalité |
| V-MOD-09, 10, 13 | règles locales sur des profils de 5 colonnes écrits à la main (relaxation : 16 cas dont les égalités, les bords, `L = 1`, `L = 2` ; balistique : 12 cas ; dépôt : 5 cas) | calcul à la main d'après la doc (chaque cas est commenté dans le test) ; recoupé par les deux prototypes | aucune | égalité exacte |
| V-MOD-11 | l'exhaustif de la relaxation : les 64 triplets `(hg, hi, hd)` de 0 à 3 × les colonnes 0, 2, 4 | la règle écrite en trois lignes dans le test | aucune | une seule colonne change, de 1, et c'est celle de la règle (une des deux si pièce) |
| V-MOD-12 | la pièce : `pile = suivant() >> 31`, 0 à gauche, 1 à droite, un seul tirage, seulement dans le cas `hg == hd < hi`, même pour `L = 2` | doc ; prédite par un `Alea::new(graine)` du test sur 40 graines | aucune | égalité exacte du profil et de l'état du générateur ; les deux côtés sortent |
| V-MOD-14, 15, 16 | traces d'Eden à la main sur 3 colonnes (`k = 1, 0, 1, 2, 0` et `k = 1, 2, 2` : un trou) et sur 1 et 2 colonnes | calcul à la main d'après la doc, **recoupé par le second prototype Python** (`trace_eden.py` : mêmes listes) | aucune | égalité exacte des périmètres (ordre compris) et des profils |
| V-MOD-17 | invariants du périmètre : vide, touche un occupé ou le substrat, sans doublon, complet ; sites occupés = particules ; hauteur = 1 + plus haute rangée occupée | définition du périmètre (doc) | aucune | égalité à chaque pas (L = 5, 8, 13 ; 6 graines ; 250 particules) |
| V-MOD-18 | la capacité : sur une colonne, le pas numéro `HAUTEUR_EDEN = 4096` panique et le précédent non | doc (convention) | aucune | `catch_unwind` |
| V-MOD-19 | paniques documentées (largeur 0 ou 4097, `depuis_hauteurs` d'Eden ou vide, colonne hors bande, indice hors périmètre, `occupe` hors bande) | doc | — | `catch_unwind` |
| V-MOD-20 | les hauteurs de 12 cas, par le second prototype Python | `ref_surface.py` (écrit en Python d'après la doc ; PCG32 de `ref_alea.py`) | aucune | égalité exacte |
| V-MOD-21 | relaxation sur 2 colonnes : après `n` particules les hauteurs triées sont `(n/2, n - n/2)` pour toute graine | preuve par récurrence (la particule finit toujours sur une colonne de hauteur minimale : la sienne à égalité ; les deux voisins de `i` sont l'autre colonne) | aucune | égalité exacte (30 graines, 100 particules) |

## La rugosité, le dépôt aléatoire et la pente (V-RUG, V-RD, V-PENTE, V-MOY, V-CONS)

| Id | Valeur | Source | Tolérance | Critère |
|---|---|---|---|---|
| V-RUG-01 | `rugosite_exacte` : `[0,2] → (4,4)` ; `[1,1,1] → (0,9)` ; `[0,3,0,3] → (36,16)` ; `[0,0,0,9] → (243,16)` ; `[1,2,3,4,5] → (50,25)` ; `[7] → (0,1)` ; `[10¹²+1, 10¹², 10¹²] → (2,9)` ; `[2⁴⁰, 0] → (2⁸⁰, 4)` ; `[5,5,5,5,6,5,5,5] → (7,64)` | calcul à la main de `Σ(h - moyenne)²` en rationnels (la variance de 1 à 5 est 2 ; etc.), recoupé en entiers `u128` par le test et par Python | aucune | égalité exacte |
| V-RUG-02 | `w² = num as f64 / den as f64` ; `[10¹²+1, 10¹², 10¹²]` donne `2/9` aux bits (`0x3FCC71C71C71C71C`, Python `struct`) ; `[0,3,0,3]` donne 2,25 et 1,5 | doc, Python | aucune (bit à bit) | égalité exacte |
| V-RD-01 | `E[w²] = n (L-1) / L²`, soit `t (1 - 1/L)` pour `n = t·L`, par énumération de toutes les suites : `Σ numérateurs = L^n · n · (L-1)` pour (n, L) = (4,2), (4,3), (5,3), (6,3), (6,4), (7,4), (8,3), (7,5), (9,3) | énumération en entiers dans le test (`itertools` de Python redonne les mêmes sommes dans `variance_depot.py`) ; preuve : `S2 = n + 2 × (paires de particules dans la même colonne)`, chaque paire a la probabilité `1/L` | aucune | égalité exacte |
| V-RD-02 | `Var(w²) = 2 n (n-1) (L-1) / L⁴` : 3/2 pour (4,2), 16/27 pour (4,3), 45/64 pour (6,4) | preuve : les indicatrices de paire sont deux à deux indépendantes, `Var(S2) = 4 · C(n,2) · (1/L)(1-1/L)` ; recoupée par `variance_depot.py` : factorielles descendantes de la multinomiale (formule indépendante) ET énumération des `L^n` suites, en `Fraction`, égales exactement sur 11 couples | `1e-12` en relatif sur le flottant | accord avec l'énumération du test |
| V-RD-03 | `E[w²] = t (1 - 1/L)` pour `Surface` dépôt aléatoire, moyenne de `G` graines | V-RD-01 ; tolérance CALCULÉE `6 · sqrt(Var(w²) / G)` (V-RD-02) | 6 écarts-types (z observé du test : -1,62 ; -0,18 ; -1,12 ; 0,67 ; -0,44 pour L = 2, 3, 4, 8, 32 ; sur 100 groupes de graines : écart-type de z entre 1,02 et 1,06, `|z|` maximal 3,35) et tolérance plus étroite que `t / L` | `|z| < 6` |
| V-CONS-01 | dépôt, relaxation : une colonne monte de exactement 1 par pas, `Σ h = particules` ; balistique : une colonne monte d'au moins 1 ; Eden : sites occupés = particules, `Σ h >= particules` ; aucune hauteur ne baisse | définitions (règles de la doc) | aucune | égalité / inégalité à chaque pas |
| V-PENTE-01 | `pente` : 2 (exact) ; 0,8 (deux jeux) ; -2 ; 1 ; `171/182` (x = 0, 1, 10 ; y = 0, 0, 9) ; invariance par translation et dilatation des `y` ; `NaN` si tous les `x` sont égaux | calcul à la main de `Sxy / Sxx` (x̄ = 11/3, ȳ = 3, Sxy = 57, Sxx = 546/9) | `1e-12` en relatif (sommes en `f64`) | accord |
| V-PENTE-02 | paniques de `pente`, `rugosite_exacte`, `rugosites_moyennes`, `exposant` | doc | — | `catch_unwind` |
| V-MOY-01 | `rugosites_moyennes` = la moyenne des `w²` du simulateur naïf, dans l'ordre des graines | doc (somme en `f64` dans l'ordre des graines à partir de 0,0, divisée par `graines.len() as f64`) | aucune (bit à bit) | égalité exacte |
| V-MOY-02 | `exposant` = `pente(ln t, 0,5 · ln w²moyen)` | doc ; refaite avec `givre_lab::math::logarithme` (étape 0) et la formule développée | `1e-9` en relatif (autre ordre de sommation) | accord |

## L'écriture canonique, l'empreinte, la directive `surface` (V-ECR, V-LAN)

| Id | Valeur | Source | Tolérance | Critère |
|---|---|---|---|---|
| V-ECR-01 | l'écriture complète de 4 états à une colonne (45 octets ; 64 pour Eden) écrite en hexadécimal : `4749565245325330` (`GIVRE2S0`), le modèle, `01000000`, `0300000000000000`, `0300000000000000`, `d0c2a4bb078b789b6d00000000000000`, et pour Eden `01000000 0000000003000000 03000000 010101` | octet par octet d'après la doc ; le générateur après 3 tirages de (7, 54) : `ref_alea.py` ; recoupé par `ref_surface.py` | aucune | égalité exacte |
| V-ECR-02 | `octets()` égal à l'encodeur du test (`Naif::octets`) pour 4 modèles × 2 largeurs × 2 graines × 4 états | doc | aucune | égalité exacte |
| V-ECR-03 | 8 SHA-256 de l'écriture canonique de (L, graine, N) = (8, 1, 200) et (5, 4, 120) pour les quatre modèles, et 4 de l'état initial de (3, 2, 0) | Python `hashlib` sur l'écriture du second prototype (`valeurs_ecriture.py`) | aucune | égalité exacte |
| V-LAN-01 | le CSV `couches,rugosite` des quatre modèles pour 7 couples (L, couches) ; instants 1, 2, 4, …, puis la dernière couche | doc ; refait avec le simulateur naïf (entiers exacts, `sqrt(num / den)`, `{:.6}`) | aucune | égalité de chaînes |
| V-LAN-02 | l'empreinte de `lancer` = SHA-256 de l'écriture du simulateur naïf ; la forme du fichier est ignorée | doc | aucune | égalité |
| V-LAN-03 | l'aperçu : dimensions et tous les pixels (fond `(240,240,240)`, rangée remplie `(v, 96, 255 - v)`) | doc ; refait couche par couche avec le simulateur naïf ; décodeur PNG du test (étape 0) | aucune | égalité de tous les pixels |
| V-LAN-04 | les refus : `DirectiveInconnue(ligne)` pour 18 directives fautives et 5 combinaisons ; les bornes incluses (4096 × 1, 1 × 1024) | doc | — | égalité |
| V-LAN-05, 06 | `scenarios/surface.txt` (relaxation, 64 colonnes, 200 couches) ; le binaire | doc | aucune | égalité |

## Les exposants β (V-EXP) : valeurs publiées, mesures du prototype, tolérances

**Mesure** : `L = 1024` ; instants 16, 24, 32, 48, 64, 96, 128, 192, 256, 384, 512 ; une pente sur la courbe moyenne de `w²` de `G` graines
(1 à `G` dans le test) ; `G` = 25 (dépôt), 100 (relaxation), 300 (balistique), 150 (Eden). Les nombres de graines n'ont pas été choisis pour
« faire passer » : ils ont été fixés AVANT la mesure finale pour équilibrer le bruit (écart-type de l'exposant entre groupes) d'après une
première exploration (G = 20 à 150), puis la mesure finale a été faite une fois sur 60 groupes indépendants (graines `100000 + 1000 r + 1 …`).
La moyenne des pentes de chaque graine n'est pas retenue : une `w` isolée est bruitée et le logarithme d'une moyenne est moins biaisé que
la moyenne de logarithmes (non mesuré ici, argument de principe).

| Id | Modèle | β publié | Source de la valeur publiée | Mesuré (60 groupes) : moyenne ± sd ; min – max | Tolérance du test | Écart à la valeur publiée |
|---|---|---|---|---|---|---|
| V-EXP-01 | dépôt aléatoire | 1/2 exact | V-RD-01 (calcul exact) | 0,50036 ± 0,00181 ; 0,4958 – 0,5046 | 0,5 ± 0,010 | +0,0004 |
| V-EXP-02 | relaxation | ≈ 1/4 | V-REF-04 (non lu), V-REF-02a : Family 0,25 ± 0,01 (cité), Mal et al. 0,241 (sat., N = 1) | 0,24631 ± 0,00364 ; 0,2378 – 0,2531 | 0,246 ± 0,019 | -0,0037 : 1/4 est dans l'intervalle |
| V-EXP-03 | balistique | 1/3 (asymptotique) ; **0,280 ± 0,002 à `L = 1024`** | V-REF-03 ; pour la valeur à cette taille : Farnudi et Vvedensky 2011, tableau II et figure 3 (texte lu par le juge), fenêtre de croissance jusqu'à `t_g = 284` couches, 10⁵ réalisations | 0,28249 ± 0,00279 ; 0,2768 – 0,2887 | 0,282 ± 0,014 | **-0,0508** à 1/3 : 1/3 n'est PAS dans l'intervalle ; +0,0025 à la valeur publiée pour `L = 1024`, qui y est |
| V-EXP-04 | Eden | 1/3 | V-REF-05 | 0,30300 ± 0,00410 ; 0,2942 – 0,3105 | 0,303 ± 0,021 | **-0,0303** : 1/3 n'est PAS dans l'intervalle |
| V-EXP-05 | ordre | — | — | différences de groupes : balistique - relaxation 0,03617 ± 0,00457 (min 0,0259) ; Eden - relaxation 0,05669 ± 0,00579 (min 0,0442) ; dépôt - max(bal., Eden) 0,19736 ± 0,00459 (min 0,1871) ; Eden - balistique 0,02051 ± 0,00497 (0,0110 à 0,0330) | marges 0,013 ; 0,027 ; 0,17 ; borne 0,046 | — |

**Tolérance** : 5 écarts-types observés entre groupes, arrondis au millième vers le haut (la moyenne arrondie à trois décimales sert de
centre, sauf pour le dépôt aléatoire dont le centre est la valeur exacte 0,5). Marge d'un ordre : moyenne de la différence moins 5 écarts-types,
arrondie à la baisse. Le pire écart observé sur les 60 groupes ne dépasse pas 2,5 écarts-types : la tolérance couvre l'observé avec une
marge de 2 fois ou plus. Le groupe du test (graines 1 à `G`) donne 0,50047 ; 0,24280 ; 0,28035 ; 0,29865 (écarts à la moyenne : +0,06, -0,96,
-0,76, -1,06 écarts-types).

**L'écart à 1/3 pour le balistique et Eden se dit** (règle 6 du brief commun) : à `L = 1024` et `t` de 16 à 512, l'exposant effectif de ces
deux modèles est sous la valeur asymptotique (corrections de taille et de temps finis ; dans la littérature l'exposant effectif du
balistique est de 0,3 à 0,339 selon le travail, d'après un résumé lu par recherche, non ouvert). Les tests ne figent pas 1/3 : ils figent ce
que fait un code correct dans cette fenêtre, avec une tolérance mesurée, et l'ordre des classes. Pour la relaxation, 1/4 est dans l'intervalle.

**Pourquoi la fenêtre** (correction du juge : la fenêtre finit avant la saturation proprement dite, mais PAS avant l'effet de la taille finie ; voir V-EXP-06) : avant la saturation (sur 6 graines à `L = 1024`, la rugosité du balistique cesse de croître vers 4 000 à
8 000 couches, la relaxation croît encore à 16 000), après les premières couches où tous les modèles ressemblent au dépôt aléatoire
(l'exposant local du balistique vaut 0,5 à `t = 4`, 0,36 à `t = 8`, 0,28 à `t = 16`). Des fenêtres plus courtes et de plus petites
largeurs (`L` = 256, `t` de 8 à 128 ; `L` = 512, `t` de 16 à 256) donnent des exposants effectifs encore plus bas (0,259 et 0,271 pour le
balistique) et un écart relaxation/balistique plus faible (0,019 et 0,030) : elles ont été mesurées et écartées.

**V-EXP-06 : la mesure indépendante du juge** (simulateur Rust écrit par le juge sans rien reprendre du prototype : générateur
xoshiro256** amorcé par splitmix64 au lieu de PCG32, Eden par grille d'états et index inverse ; programme `juge-2s/outils/sim`,
analyse `analyse.py`). À `L = 1024`, fenêtre 16 … 512, 60 groupes de `G` graines (les `G` du test), une pente sur la moyenne :

| Modèle | Juge : moyenne ± sd (min – max) | Pire écart au centre du test / tolérance | Tolérance / sd du juge |
|---|---|---|---|
| dépôt | 0,49981 ± 0,00191 (0,4953 – 0,5043) | 0,47 | 5,2 |
| relaxation | 0,24607 ± 0,00309 (0,2387 – 0,2523) | 0,38 | 6,2 |
| balistique | 0,28298 ± 0,00339 (0,2761 – 0,2926) | 0,76 | 4,1 |
| Eden | 0,30398 ± 0,00429 (0,2962 – 0,3117) | 0,41 | 4,9 |

Différences (mêmes groupes) : balistique - relaxation 0,0369 ± 0,0048 (min 0,0259, borne 0,013) ; Eden - relaxation 0,0579 ± 0,0044 (min 0,0499,
borne 0,027) ; dépôt - max 0,1958 ± 0,0046 (min 0,1883, borne 0,17) ; |Eden - balistique| 0,0210 ± 0,0056 (max 0,0338, borne 0,046). Toutes
les tolérances tiennent sur un autre générateur et un autre code.

La taille finie : le `w²` moyen à `t = 512` vaut, à `L = 1024`, 0,895 fois celui de `L = 8192` (balistique), 0,932 fois celui de `L = 4096`
(Eden), 0,957 fois celui de `L = 8192` (relaxation) ; l'écart commence dès `t ≈ 16` (0,99). La même fenêtre 16 … 512 à `L = 8192` donne
0,2965 ± 0,0011 (balistique, 240 graines) et 0,2510 ± 0,0014 (relaxation, 120 graines) ; à `L = 4096`, Eden : 0,3129 ± 0,0020 (120 graines).
Des fenêtres plus tardives à `L = 8192` (balistique) : 0,3019 (32 … 1024), 0,3049 (64 … 2048), 0,3062 (128 … 4096), 0,3066 (256 … 8192) :
l'exposant effectif MONTE avec `t` et `L`, comme chez Farnudi et Vvedensky (0,308 à `L = 8192`), sans atteindre 1/3 à ces tailles. Eden à
`L = 4096` reste vers 0,31 jusqu'à `t = 4096` (pas de convergence visible à cette échelle). À `L = 1024` au contraire, l'exposant ne monte pas
avec `t` (0,2824 sur 16 … 128, 0,2834 sur 64 … 512) : la taille finie l'aplatit ; c'est pourquoi aucun test de TENDANCE n'est écrit (un test
de tendance en `L` à 5 écarts-types coûterait de l'ordre de 8·10⁸ pas de plus, plusieurs fois ce fichier).

La convention de la relaxation (V-REF-02a) : à `L = 1024`, fenêtre 16 … 512, 60 groupes de 100 : la règle de la doc (stricte) 0,2461 ± 0,0031 ;
« quitte `i` pour un voisin plus bas OU égal » 0,2390 ± 0,0037 ; « uniforme parmi les minima des trois » 0,2425 ± 0,0039 : les trois tiennent
dans la tolérance (0,246 ± 0,019) ; seuls les tests exacts distinguent les conventions.

---

## Ce que ces valeurs n'établissent pas

- Elles n'établissent PAS que l'exposant effectif tende vers 1/3 : le juge l'a vu monter avec `L` et `t` pour le balistique (0,283 → 0,307,
  V-EXP-06) et Farnudi et Vvedensky le publient, mais aucun test ne le vérifie.
- Elles n'établissent PAS que les exposants asymptotiques sont 1/4 et 1/3 : le prototype ne les atteint pas pour le balistique et Eden à
  cette taille (écarts de 0,05 et 0,03) ; l'exposant de croissance de la relaxation est dans la tolérance de 1/4 ; le dépôt aléatoire est exact.
- Elles n'établissent pas α (la rugosité de saturation en `L^α`), ni `z`, ni le collapse `w(L, t) = L^α f(t / L^z)`.
- La règle d'égalité de la relaxation est une convention : les sources lues divergent (V-REF-02a) ; le texte de Family (1986) n'a pas été lu.
- La forme `max` du balistique et l'Eden « A » (uniforme sur le périmètre) sont des définitions retenues d'après des sources secondaires ou
  géométriques : le livre de Barabási et Stanley et le texte d'Eden (1961) n'ont pas été ouverts.
- L'indépendance des graines voisines de PCG32 (même flux) est supposée ; les z-scores du dépôt aléatoire (écart-type 1,02 à 1,06) ne la
  contredisent pas.
- Aucune propriété n'est testée au-delà de `L = 4096` ni au-delà de 1024 couches.

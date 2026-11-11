//! Le modèle mésoscopique de la neige de Gravner et Griffeath, sans bruit (étape G1) : sur la grille hexagonale, en virgule
//! fixe, **sans aucun hasard**. Trois masses par case (la vapeur qui diffuse, une couche « quasi-liquide » au bord du cristal,
//! la glace) et un indicateur de rattachement ; un pas fait quatre sous-pas (diffusion, gel, rattachement, fonte). Sans bruit,
//! la symétrie des douze images de l'hexagone est **exacte au bit près** (toutes les sommes sont des sommes d'entiers) et la
//! masse est conservée, à l'arrondi de la division par 7 près.
//!
//! # La référence
//!
//! J. Gravner et D. Griffeath, « Modeling snow crystal growth II: A mesoscopic lattice map with plausible dynamics »,
//! *Physica D* 237 (2008) 385-404, <https://doi.org/10.1016/j.physd.2007.09.008>. **Lu** : la préimpression de l'auteur
//! (« Revised version, September 2007 », copie `mk.bcgsc.ca/snowflakes/paper/modeling-snow-crystal-growth-2.pdf`), section 5
//! (« Our algorithm for two-dimensional snow crystal growth » : l'état, le simulateur, les équations (1) à (5)), section 6
//! (figures 13 à 16) et l'annexe 8 (les réglages de chaque figure) ; les figures 9 et 13 ont été vues. **Non lu** : la version
//! publiée dans *Physica D* (elle peut différer de la préimpression ; l'éditeur en refuse l'accès, et la copie du site de
//! l'auteur, `math.ucdavis.edu/~gravner/papers/h2l.pdf`, est le même fichier que la préimpression), les figures 10, 11, 12,
//! 14 à 17. La fiche
//! `references/etape-g1-valeurs.md` dit pour chaque valeur d'où elle vient, et les contradictions de la source (la
//! préimpression se contredit à deux endroits).
//!
//! **Ce que ce module est** : les équations (1) à (4) de l'article (diffusion, gel, rattachement, fonte), dans leur ordre, sans
//! l'équation (5) (le bruit, `σ = 0`). **Ce que ce module n'est pas** : voir « Ce que le modèle n'est pas ».
//!
//! # L'état
//!
//! Une [`Neige`] vit sur l'hexagone de rayon `R` de [`crate::reseau::Hexagone`] (coordonnées axiales `(q, r)`, cases numérotées
//! de 0 à `3 R (R + 1)` dans l'ordre canonique de cette doc : `r` croissant, puis `q` croissant). Chaque case porte une
//! [`Case`] `(a, b, c, d)` :
//!
//! - `a` : le **rattachement** (booléen) : la case appartient au cristal. Il est **stocké** (il ne se déduit pas des masses : une
//!   case de bord peut avoir les mêmes masses qu'une case du cristal) ;
//! - `b` : la masse de **bord** (la couche quasi-liquide) ;
//! - `c` : la masse **cristalline** (la glace) ;
//! - `d` : la masse **diffusive** (la vapeur).
//!
//! Les trois masses sont des [`Fixe`] (Q31.32, [`crate::math::Fixe`]), **toujours `>= 0`**. Le **temps** est le nombre de pas
//! faits. Il n'y a ni graine ni hasard : le même état de départ donne toujours le même état, sur toutes les machines.
//!
//! Trois classes de cases, pour un état donné (les `a` seuls) : une case est **rattachée** si `a = 1` ; elle est **de bord**
//! si `a = 0` et si au moins un de ses six voisins est rattaché (les voisins sont les cases `(q + dq, r + dr)` de
//! [`crate::reseau::DIRECTIONS_HEX`] **qui sont dans l'hexagone** : une case du mur n'en a que 3 ou 4) ; elle est **lointaine**
//! sinon. Pour une case `x`, `n(x)` est le **nombre de voisins rattachés** (0 à 6). Une case de bord est donc une case non
//! rattachée avec `n >= 1`.
//!
//! **Invariants** (vrais de tout état atteignable, et exigés de tout état donné à [`Neige::essayer_depuis_cases`]) :
//! (I1) une case rattachée a `b = 0` et `d = 0` (« le flocon n'est fait que de masse cristalline » : l'article) ; (I2) une case
//! lointaine a `b = 0` et `c = 0` (les masses de bord et cristalline n'existent qu'au bord du cristal) ; (I3) toutes les masses
//! sont `>= 0`. Les cases rattachées ne changent **plus jamais** : « le rattachement est permanent, et il n'y a plus de
//! dynamique aux cases rattachées » (l'article).
//!
//! **L'état de départ** de [`Neige::essayer_new`] (l'article : « masse cristalline 1 à l'origine, masse diffusive `ρ` partout
//! ailleurs ») : la case `(0, 0)` est `(a = 1, b = 0, c = 1, d = 0)` ([`Fixe::UN`]), et toute autre case est
//! `(a = 0, b = 0, c = 0, d = ρ)`. Le germe est un cristal d'une case, dont les six voisines sont les premières cases de bord.
//!
//! # Les réglages
//!
//! Sept [`Parametres`] (le huitième de l'article, `σ`, est le bruit : hors de cette étape). Ils sont **valides** dans ces
//! plages, qui sont celles de [`Parametres::verifier`] ; chaque plage dit sa raison :
//!
//! | réglage | rôle (l'article) | plage permise | raison |
//! |---|---|---|---|
//! | `rho` `ρ` | la densité de vapeur de départ | `0 <= ρ <= 1` | une masse de vapeur par case au plus égale à la masse d'une case de glace ; et la masse totale reste sous `2^24` pour tout rayon permis (pas de débordement) |
//! | `beta` `β` | le seuil de masse de bord pour 1 ou 2 voisins rattachés | `1 <= β <= 8` | l'article suppose `β > 1` (« 1 est le seuil de base du cas à 3 voisins ») ; `8` est bien au-delà de tous les réglages lus (le plus grand : `3,2`) |
//! | `alpha` `α` | le seuil de masse de bord de l'instabilité « en lame de couteau » (3 voisins) | `0 <= α <= 1` | un seuil au moins égal à 1 serait sans effet (`b >= 1` rattache déjà) ; les réglages lus vont de 0 à 0,6 |
//! | `theta` `θ` | le seuil de vapeur du voisinage de cette même instabilité | `0 <= θ <= 8` | une somme de 7 vapeurs, chacune au plus `1` ; les réglages lus vont de 0 à 0,2 |
//! | `kappa` `κ` | la part de la vapeur du bord qui gèle d'un coup | `0 <= κ <= 1` | une proportion |
//! | `mu` `μ` | la part de la masse de bord qui fond en vapeur | `0 <= μ <= 1` | une proportion |
//! | `gamma` `γ` | la part de la masse cristalline de bord qui fond en vapeur | `0 <= γ <= 1` | une proportion |
//!
//! Les bornes sont **incluses**. Les réglages de l'article (annexe, `references/etape-g1-valeurs.md`) sont tous dans ces plages.
//!
//! # Le domaine et son mur
//!
//! L'article calcule sur une grille finie avec des conditions périodiques (« hexagone avec raccord périodique » pour garder la
//! symétrie) : sa masse est conservée. **Ce module prend un hexagone fermé par un mur qui réfléchit** (un choix à nous : voir
//! « Ce que le modèle n'est pas ») : une case de l'hexagone qui n'a pas ses six voisins (les cases à distance hexagonale exactement
//! `R` du centre) traite un voisin absent **comme l'article traite un voisin du cristal** : dans la diffusion, ce terme est
//! remplacé par la vapeur de la case elle-même ; ailleurs un voisin absent n'existe pas (il ne compte ni dans `n(x)` ni dans une
//! somme). Le mur est stable par les douze symétries de l'hexagone, donc la symétrie reste **exacte** ; la masse totale est
//! conservée comme dans l'article. Comme l'article le dit (« la grille doit être au moins 1,5 fois le diamètre final, 2 fois
//! pour une plaque lente »), le cristal ne doit pas atteindre le mur : [`Neige::rayon_cristal`] `== R` dit qu'il l'a touché (le
//! modèle n'est plus celui de l'article).
//!
//! # Un pas, à la lettre
//!
//! **Cette section est la spécification** : une implémentation qui la suit donne EXACTEMENT les mêmes états qu'une autre, bit
//! pour bit. Rien n'est laissé libre. L'ordre est celui de l'article (« les étapes (i) à (iv) sont faites dans l'ordre à chaque
//! pas »). Le pas est **synchrone dans chaque sous-pas** : un sous-pas calcule les nouvelles valeurs de toutes les cases à partir
//! des valeurs qu'avait l'état au début de ce sous-pas, jamais d'une valeur déjà remplacée dans le même sous-pas (l'ordre de
//! parcours des cases ne change donc rien). `n(x)` et la classe d'une case sont calculés **une fois**, sur les `a` du début du
//! pas (les sous-pas 1 et 2 ne changent aucun `a`).
//!
//! Notations : `UN = Fixe::UN` ; pour un réel `y`, les valeurs d'un sous-pas sont `d1`, `b2`, `c2`, `d2`, `a3`, `b3`, `c3`, `b4`,
//! `c4`, `d4` ; toute valeur non citée d'un sous-pas est égale à celle du sous-pas précédent. `Fixe(s).div_entier(7)` et
//! `p * q` sont ceux de [`crate::math::Fixe`] (arrondi au plus proche, demi-cas vers `+∞`).
//!
//! ## 1. La diffusion (équation (1))
//!
//! Pour chaque case `x` **non rattachée** (`a(x) = 0` : de bord ou lointaine), avec `y_0 … y_5` ses voisins dans l'ordre de
//! [`crate::reseau::DIRECTIONS_HEX`] (l'ordre ne change aucun bit : la somme est une somme d'entiers) :
//!
//! ```text
//! terme(y) = d(y)   si y est dans l'hexagone et a(y) = 0
//!          = d(x)   sinon (y rattaché, ou absent : le mur)        (« conditions réfléchissantes »)
//! somme(x) = d(x) + terme(y_0) + terme(y_1) + terme(y_2) + terme(y_3) + terme(y_4) + terme(y_5)      (sept entiers `brut`, exacte)
//! d1(x)    = Fixe(somme(x)).div_entier(7)         (au plus proche ; il n'y a jamais de demi-cas, car 7 est impair)
//! ```
//!
//! C'est la moyenne sur sept cases de l'article (« poids uniforme 1/7 sur le centre et chacun de ses voisins »). Une case
//! rattachée garde sa vapeur : `d1(x) = d(x)` (qui vaut 0, par (I1)).
//!
//! ## 2. Le gel (équation (2))
//!
//! Pour chaque case **de bord** `x` (`a(x) = 0` et `n(x) >= 1`), une part `κ` de sa vapeur gèle d'un coup, le reste devient
//! masse de bord :
//!
//! ```text
//! g(x) = κ * d1(x)               (le produit `Fixe`, arrondi au plus proche ; 0 <= g <= d1)
//! c2(x) = c(x) + g(x)
//! b2(x) = b(x) + (d1(x) - g(x))   (le reste exact : `b2 + c2 + d2` est exactement `b + c + d1`)
//! d2(x) = 0
//! ```
//!
//! ## 3. Le rattachement (équations (3a) à (3d))
//!
//! Pour chaque case **de bord** `x`, avec `n = n(x)` (les voisins rattachés **au début du pas**, pas ceux qui se rattachent dans
//! ce sous-pas) et les valeurs `b2`, `d2` de l'état **après le sous-pas 2** (donc `d2 = 0` sur les cases de bord et sur les cases
//! rattachées ; `d2 = d1` sur les cases lointaines) :
//!
//! ```text
//! T(x) = d2(x) + somme des d2(y) sur les voisins y de x qui sont dans l'hexagone        (la vapeur du voisinage de sept cases ; entier exact)
//! x se rattache  <=>   n >= 4                                                                      (3c : sans condition sur b2)
//!                  ou  n = 3  et  ( b2(x) >= UN   ou   ( T(x) < θ  et  b2(x) >= α ) )             (3b : « >= 1 » ; « < θ » strict ; « >= α »)
//!                  ou  n = 1 ou 2  et  b2(x) >= β                                                   (3a)
//! ```
//!
//! Toutes les décisions sont prises **avant** de changer un seul `a` (deux cases voisines de bord ne se voient pas rattachées
//! l'une l'autre dans le même pas). Puis, pour chaque case qui se rattache (3d) : `a3(x) = 1`, `c3(x) = c2(x) + b2(x)`,
//! `b3(x) = 0` (sa vapeur `d2(x)` vaut déjà 0). Les comparaisons sont celles des entiers `brut`. **L'article écrit `n >= 3` en
//! (3b) et `n >= 4` en (3c)** : pour `n >= 4` le résultat est le même (rattachement), la règle ci-dessus lit (3b) pour `n = 3`.
//!
//! ## 4. La fonte (équation (4))
//!
//! Pour chaque case non rattachée **ayant au moins un voisin rattaché après le sous-pas 3** (le bord du cristal de la fin du
//! sous-pas 3 ; une case qui vient de se rattacher n'y est plus : « plus de dynamique aux cases rattachées » ; une case de bord
//! d'avant qui ne s'est pas rattachée y est encore ; une case devenue de bord à l'instant a `b3 = c3 = 0`, rien ne change) :
//!
//! ```text
//! fb(x) = μ * b3(x)         fc(x) = γ * c3(x)        (les produits `Fixe` : deux arrondis)
//! b4(x) = b3(x) - fb(x)     c4(x) = c3(x) - fc(x)     d4(x) = d2(x) + fb(x) + fc(x)
//! ```
//!
//! (Par (I2), c'est exactement l'ensemble des cases non rattachées qui ont `b3 > 0` ou `c3 > 0` : une implémentation peut
//! parcourir celles-là.) Puis le temps augmente de 1. L'état est celui de `a3`, `b4`, `c4`, `d4`.
//!
//! **Ce qui est lu ou non dans chaque sous-pas** : le 1 lit `d` et `a` ; le 2 lit `d1`, `n`, `b`, `c` ; le 3 lit `n`, `b2`, `d2`
//! (le voisinage) ; le 4 lit `a3`, `b3`, `c3`, `d2`.
//!
//! # Ce qui est conservé, et pourquoi rien ne déborde
//!
//! La **masse totale** `M = Σ_x (b + c + d)` ([`Neige::masse_totale`], en `i128`, de tous les `brut`) :
//! - les sous-pas 2, 3 et 4 la conservent **exactement** (leurs arrondis sont repris sur le reste : `d1 - g`, `b3 = 0`, `fb`, `fc`) ;
//! - le sous-pas 1 la conserve en réels (les poids sont symétriques : chaque couple de cases voisines non rattachées s'échange
//!   `1/7` de ses vapeurs, un voisin du cristal ou du mur est remplacé par la case elle-même, qui ne coûte et ne rapporte rien) ;
//!   en virgule fixe, l'arrondi de chaque division par 7 se trompe d'au plus `3/7` d'unité (le reste d'une division par 7 est 0 à
//!   6 : l'erreur est `0`, `±1/7`, `±2/7` ou `±3/7`). Donc, **à chaque pas, `|M' - M|` est au plus `3 C / 7` unités du dernier
//!   chiffre**, où `C` est le nombre de cases non rattachées au début du pas. C'est la seule perte ou création de masse de tout le
//!   modèle. Elle est sans biais (les erreurs `±k/7` sont également réparties) ; `MODELE.md` en donne la mesure.
//!
//! **Positivité** : `0 <= g <= d1`, `0 <= fb <= b3`, `0 <= fc <= c3` (le produit par une proportion de 0 à 1 d'un entier `>= 0`
//! est arrondi entre 0 et cet entier), donc toutes les masses restent `>= 0` (I3). Les invariants (I1) et (I2) sont conservés par
//! chaque sous-pas (le gel et le rattachement ne touchent que les cases de bord ; une case n'est de bord que si un voisin est
//! rattaché, et ce voisin le reste).
//!
//! **Aucun débordement, aucune panique** : [`Neige::pas`] ne panique jamais sur un état permis. La masse totale d'un état permis
//! est `< 2^56` en `brut` (soit `< 2^24`), et elle ne varie de `3 C / 7` unités par pas qu'au plus : tant que le nombre de pas est
//! au plus `10^10`, `M < 2^57`. Chaque masse est `<= M` et chaque somme de sept masses est `< 7 · 2^57 < 2^60`, bien en deçà de
//! `i64::MAX`. Les produits `Fixe` se font en `i128` et leur résultat est `<=` leur opérande entier.
//!
//! # Les mesures
//!
//! - [`Neige::cases_rattachees`] : le nombre de cases `a = 1` ; au départ 1, et il ne décroît **jamais**.
//! - [`Neige::masse_du_cristal`] : la somme (en `i128`, des `brut`) de `c` sur les cases **rattachées** ; il ne décroît jamais
//!   (une case rattachée ne change plus, et en se rattachant elle reçoit `b2 >= 0`).
//! - [`Neige::rayon_cristal`] : la plus grande distance hexagonale `distance_hex((0, 0), case)` d'une case rattachée (0 pour le
//!   germe seul) ; il croît d'au plus 1 par pas (une case ne se rattache que si un voisin l'est déjà, au début du pas).
//! - [`Neige::masse_totale`] : voir ci-dessus.
//!
//! # L'écriture canonique et l'empreinte
//!
//! L'empreinte ([`Neige::empreinte`]) est le SHA-256 (`empreinte::sha256_hex`) de [`Neige::ecriture_canonique`], suite d'octets,
//! dans cet ordre (tout entier est petit-boutiste ; `brut` est le champ de [`Fixe`]) :
//!
//! ```text
//! 8 octets    l'étiquette ASCII "GIVREG01"
//! 4 octets    le rayon R de l'hexagone, en u32
//! 56 octets   rho.brut, beta.brut, alpha.brut, theta.brut, kappa.brut, mu.brut, gamma.brut, chacun en i64 (8 octets), dans cet ordre
//! 8 octets    le temps (le nombre de pas faits), en u64
//! 25 octets   par case, dans l'ordre canonique (r croissant, puis q croissant) : a (1 octet : 0 ou 1), puis b.brut, c.brut, d.brut (chacun en i64)
//! ```
//!
//! Soit `76 + 25 * cases` octets. Aucune graine (aucun hasard). Deux jeux de réglages, deux rayons, deux temps ou deux états
//! différents donnent deux écritures différentes.
//!
//! # L'aperçu
//!
//! [`Neige::apercu`] rend une image RVB (trois octets par pixel, ligne par ligne de haut en bas, comme
//! [`crate::image::ecrire_png`]) **carrée de côté `2R + 1`**, comme celle de [`crate::flocon::Flocon::apercu`] : la case `(q, r)`
//! est le pixel `(x, y)` avec `x = q + R + floor(r / 2)` (la division entière **vers le bas** : `r.div_euclid(2)`) et `y = r + R` ;
//! deux cases n'ont jamais le même pixel. Les pixels qui ne sont l'image d'aucune case (les coins du carré) sont `(0, 0, 0)`.
//! La couleur d'une case :
//! - **rattachée** : `(235, 245, 255)` (la glace) ;
//! - sinon, avec `niveau = min(255, (d.brut * 255) >> 32)` (`d >= 0`, décalage arithmétique en `i128`) : `(0, niveau / 2, niveau)`
//!   (division entière) : le fond bleu est la vapeur, et sa déplétion autour du cristal fait un halo plus sombre.
//!
//! # La directive `neige` et la sortie de `lancer`
//!
//! Dans un scénario : `hexagone <rayon>` et **une seule** autre directive,
//! `neige <pas> <rho> <beta> <alpha> <theta> <kappa> <mu> <gamma>`. Les sept réglages sont des décimaux lus par
//! [`Parametres::lire`] (donc par [`crate::flocon::lire_decimal`] : exactement, arrondi **une seule fois** au plus proche, au plus 18
//! chiffres) ; `pas` est un entier de 1 à [`PAS_MAX`] en chiffres décimaux ASCII seulement (ni signe, ni point ; des zéros de
//! tête sont permis : `0045` vaut 45 ; c'est `Directive::entier`). [`lancer_neige`] (que
//! [`crate::lancer::lancer`] appelle quand la **première** directive s'appelle `neige`) crée `Neige::essayer_new(rayon,
//! paramètres)`, la fait avancer de `pas` pas, et rend :
//! - `empreinte` : [`Neige::empreinte`] de l'état final (la graine du lancer n'est pas utilisée) ;
//! - `mesures` : le CSV `temps,rattachees,rayon,masse_cristal\n` puis une ligne pour le temps 0 (l'état initial), pour chaque `t`
//!   multiple de `intervalle = max(1, pas / 20)` (division entière) avec `1 <= t <= pas`, puis, si `pas` n'est pas un multiple de
//!   `intervalle`, une dernière ligne pour `t = pas`. La ligne est `temps,rattachees,rayon,masse_cristal` : `temps` est `t`,
//!   `rattachees` est [`Neige::cases_rattachees`], `rayon` est [`Neige::rayon_cristal`], et `masse_cristal` est
//!   [`Neige::masse_du_cristal`] en **six décimales exactes** : `ppm = (2_000_000 * brut + 2^32) / 2^33` (en `i128`, division
//!   entière), écrit `{ppm / 1000000}.{ppm % 1000000 sur 6 chiffres}` (`1.000000` au temps 0). Chaque ligne finit par un seul
//!   `\n` ;
//! - `apercu` : le PNG (`image::ecrire_png`) de [`Neige::apercu`] de l'état final, de côté `2 * rayon + 1`.
//!
//! Le scénario est refusé, avec le numéro de la ligne de la **première directive fautive dans l'ordre du fichier**, si la
//! directive `neige` n'a pas exactement huit arguments, si `pas` n'est pas un entier de 1 à [`PAS_MAX`], si un des sept
//! réglages ne se lit pas ou n'est pas dans sa plage ([`Parametres::lire`]), si la forme n'est pas un hexagone (la ligne de la
//! directive `neige`) ; sinon, s'il y a une autre directive (la ligne de la directive suivante, quelle qu'elle soit). Rien n'est
//! calculé alors. [`crate::lancer::lancer`] rend `ErreurLancer::DirectiveInconnue(ligne)` pour cet `Err(ligne)`. Un scénario qui a
//! une directive nommée `dla`, `collage` ou `bruit` reste celui de la DLA (la règle de `lancer`), et toute autre première
//! directive reste refusée ou comprise comme avant.
//!
//! # Ce que le modèle n'est pas
//!
//! - **Ce n'est pas une preuve que la neige ressemble à celle de l'article**, ni à une neige réelle : aucune forme publiée n'est
//!   testée (les figures ne donnent aucun nombre). L'article lui-même dit qu'« un réglage qui donne un cristal proche d'un cristal
//!   réel » demande du doigté, et qu'il ne sait pas expliquer le diagramme de Nakaya (la forme selon la température et la
//!   sursaturation).
//! - **Le mur est à nous** : l'article utilise un hexagone avec raccord périodique (ou un carré périodique) ; ici l'hexagone est
//!   fermé par un mur réfléchissant (même règle que celle de l'article pour le bord du cristal). Les deux conservent la masse et la
//!   symétrie ; ils ne donnent pas les mêmes états dès que le cristal ou son halo de déplétion atteint le mur.
//! - **L'arrondi est à nous** : la division par 7 arrondit au plus proche en virgule fixe (l'article est en flottants).
//! - **Le modèle est 2D, sans bruit** (`σ = 0`) ; la 3D (Gravner et Griffeath 2009) et le bruit sont hors de cette étape.
//! - **Les lectures du texte** : l'ordre de la fonte (le sous-pas 4 : seules les cases non rattachées fondent) et l'ensemble sur
//!   lequel elle s'applique sont une lecture de « no further dynamics at attached sites » et de « the snowflake consists
//!   entirely of crystal mass » (faire fondre une case qui vient de se rattacher lui laisserait de la vapeur : les deux phrases
//!   l'excluent) ; `T` lit la vapeur d'**après** le gel parce que l'article note `d°` la valeur « avant le sous-pas » (3), donc
//!   après le sous-pas (2) : c'est la lecture littérale, que rien d'autre dans l'article ne confirme ; la préimpression écrit
//!   `n >= 3` en (3b), et **une** de ses deux proses de l'instabilité (après la figure 10 : `θ` et `α`) échange les deux lettres
//!   par rapport à l'équation (3b), à la phrase qui précède (3b) et à l'annexe, que ce module suit.

use crate::empreinte;
use crate::flocon::lire_decimal;
use crate::image;
use crate::lancer::Sortie;
use crate::math::Fixe;
use crate::reseau::{DIRECTIONS_HEX, Hexagone, distance_hex};
use crate::scenario::{Forme, Scenario};

/// Le plus grand rayon permis (celui d'une directive `hexagone` : `scenario::RAYON_MAX`). Un hexagone de rayon `R` a
/// `3 R (R + 1) + 1` cases : au plus `12 576 769` ; la masse totale d'un état permis est alors `< 2^24`.
pub const RAYON_MAX: u32 = 2047;

/// Le plus grand nombre de pas qu'une directive `neige` peut demander.
pub const PAS_MAX: u64 = 1_000_000;

/// Les sept réglages du modèle (voir « Les réglages » : la plage de chacun). Un jeu est **valide** s'il passe
/// [`Parametres::verifier`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Parametres {
    /// `ρ` : la masse de vapeur de chaque case au départ (le germe excepté). `0 <= ρ <= 1`.
    pub rho: Fixe,
    /// `β` : le seuil de masse de bord pour se rattacher avec 1 ou 2 voisins rattachés. `1 <= β <= 8`.
    pub beta: Fixe,
    /// `α` : le seuil de masse de bord de l'instabilité « en lame de couteau » (3 voisins rattachés). `0 <= α <= 1`.
    pub alpha: Fixe,
    /// `θ` : le seuil de vapeur du voisinage de sept cases de cette instabilité. `0 <= θ <= 8`.
    pub theta: Fixe,
    /// `κ` : la part de la vapeur d'une case de bord qui gèle d'un coup. `0 <= κ <= 1`.
    pub kappa: Fixe,
    /// `μ` : la part de la masse de bord qui fond en vapeur. `0 <= μ <= 1`.
    pub mu: Fixe,
    /// `γ` : la part de la masse cristalline d'une case de bord qui fond en vapeur. `0 <= γ <= 1`.
    pub gamma: Fixe,
}

/// Pourquoi une construction a été refusée.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ErreurNeige {
    /// Le rayon est 0 ou dépasse [`RAYON_MAX`].
    RayonHorsBorne,
    /// Un réglage est hors de sa plage : son nom (`"rho"`, `"beta"`, `"alpha"`, `"theta"`, `"kappa"`, `"mu"` ou `"gamma"`),
    /// le **premier** dans cet ordre.
    ParametreHorsPlage(&'static str),
    /// Le tableau de cases n'a pas `3 R (R + 1) + 1` éléments.
    NombreDeCases,
    /// La case d'indice canonique donné viole un invariant (I1, I2 ou I3) : la **première** dans l'ordre canonique.
    EtatInvalide(usize),
    /// La masse totale donnée est `2^56` en `brut` ou plus.
    MasseTropGrande,
}

/// L'état d'une case : le rattachement et les trois masses (voir « L'état »).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Case {
    /// Le rattachement `a` : vrai si la case est dans le cristal.
    pub a: bool,
    /// La masse de bord `b` (quasi-liquide), `>= 0`.
    pub b: Fixe,
    /// La masse cristalline `c` (glace), `>= 0`.
    pub c: Fixe,
    /// La masse diffusive `d` (vapeur), `>= 0`.
    pub d: Fixe,
}

impl Parametres {
    /// Rend `Ok(())` si les sept réglages sont dans leurs plages (voir « Les réglages », bornes incluses), sinon
    /// `Err(ErreurNeige::ParametreHorsPlage(nom))` avec le nom du **premier** réglage hors plage dans l'ordre `rho`, `beta`,
    /// `alpha`, `theta`, `kappa`, `mu`, `gamma`.
    pub fn verifier(&self) -> Result<(), ErreurNeige> {
        let un = Fixe::UN.0;
        let plages: [(&'static str, i64, i64, i64); 7] = [
            ("rho", self.rho.0, 0, un),
            ("beta", self.beta.0, un, 8 * un),
            ("alpha", self.alpha.0, 0, un),
            ("theta", self.theta.0, 0, 8 * un),
            ("kappa", self.kappa.0, 0, un),
            ("mu", self.mu.0, 0, un),
            ("gamma", self.gamma.0, 0, un),
        ];
        for (nom, valeur, bas, haut) in plages {
            if valeur < bas || valeur > haut {
                return Err(ErreurNeige::ParametreHorsPlage(nom));
            }
        }
        Ok(())
    }

    /// Lit sept décimaux écrits comme dans un scénario (chacun par [`crate::flocon::lire_decimal`], dans l'ordre `rho`, `beta`,
    /// `alpha`, `theta`, `kappa`, `mu`, `gamma`) et rend les réglages, ou `None` si l'un est mal écrit ou si le jeu n'est pas
    /// valide ([`Parametres::verifier`]).
    pub fn lire(
        rho: &str,
        beta: &str,
        alpha: &str,
        theta: &str,
        kappa: &str,
        mu: &str,
        gamma: &str,
    ) -> Option<Parametres> {
        let parametres = Parametres {
            rho: lire_decimal(rho)?,
            beta: lire_decimal(beta)?,
            alpha: lire_decimal(alpha)?,
            theta: lire_decimal(theta)?,
            kappa: lire_decimal(kappa)?,
            mu: lire_decimal(mu)?,
            gamma: lire_decimal(gamma)?,
        };
        parametres.verifier().ok()?;
        Some(parametres)
    }
}

/// Une neige de Gravner et Griffeath sans bruit : un hexagone de cases, ses réglages et son temps (voir la documentation du
/// module). Les champs sont privés : seul le contrat des fonctions compte.
#[derive(Clone, Debug)]
pub struct Neige {
    hexagone: Hexagone,
    parametres: Parametres,
    temps: u64,
    /// Le rattachement de chaque case (0 ou 1), dans l'ordre canonique.
    a: Vec<u8>,
    /// Les trois masses (le `brut` de chaque `Fixe`), dans l'ordre canonique.
    b: Vec<i64>,
    c: Vec<i64>,
    d: Vec<i64>,
    /// Pour chaque case, l'indice canonique de ses six voisins (`-1` : hors de l'hexagone, à sa place).
    voisins: Vec<[i32; 6]>,
    /// Tampons d'un pas : `n(x)`, la vapeur du sous-pas 1, la liste des cases qui se rattachent.
    n: Vec<u8>,
    d_suivant: Vec<i64>,
    nouveaux: Vec<usize>,
}

impl Neige {
    /// La neige de l'état de départ (voir « L'état ») sur l'hexagone de rayon `rayon`, au temps 0 : le germe `(a = 1, c = 1)`
    /// en `(0, 0)` et la vapeur `ρ` partout ailleurs.
    ///
    /// # Erreurs
    ///
    /// Jamais de panique : `Err(ErreurNeige::RayonHorsBorne)` si `rayon` est 0 ou plus grand que [`RAYON_MAX`] (testé en
    /// premier), puis `Err(ErreurNeige::ParametreHorsPlage(nom))` si les réglages ne sont pas valides ([`Parametres::verifier`]).
    pub fn essayer_new(rayon: u32, parametres: Parametres) -> Result<Neige, ErreurNeige> {
        if rayon == 0 || rayon > RAYON_MAX {
            return Err(ErreurNeige::RayonHorsBorne);
        }
        parametres.verifier()?;
        let hexagone = Hexagone::new(rayon);
        let cases = hexagone.cases();
        let mut a = vec![0_u8; cases];
        let b = vec![0_i64; cases];
        let mut c = vec![0_i64; cases];
        let mut d = vec![parametres.rho.0; cases];
        let origine = hexagone.index(0, 0);
        a[origine] = 1;
        c[origine] = Fixe::UN.0;
        d[origine] = 0;
        Ok(Neige::assembler(hexagone, parametres, a, b, c, d))
    }

    /// Assemble une neige au temps 0 à partir des quatre tableaux (de la taille des cases) : remplit la table des voisins et
    /// les tampons.
    fn assembler(
        hexagone: Hexagone,
        parametres: Parametres,
        a: Vec<u8>,
        b: Vec<i64>,
        c: Vec<i64>,
        d: Vec<i64>,
    ) -> Neige {
        let cases = hexagone.cases();
        let mut voisins = Vec::with_capacity(cases);
        for i in 0..cases {
            let (q, r) = hexagone.position(i);
            let mut six = [-1_i32; 6];
            for (k, &(dq, dr)) in DIRECTIONS_HEX.iter().enumerate() {
                if hexagone.contient(q + dq, r + dr) {
                    six[k] = hexagone.index(q + dq, r + dr) as i32;
                }
            }
            voisins.push(six);
        }
        Neige {
            hexagone,
            parametres,
            temps: 0,
            a,
            b,
            c,
            d,
            voisins,
            n: vec![0_u8; cases],
            d_suivant: vec![0_i64; cases],
            nouveaux: Vec::new(),
        }
    }

    /// Une neige d'état quelconque au temps 0 : `cases` donne la [`Case`] de chaque case **dans l'ordre canonique** de
    /// [`Hexagone`] (`cases[Hexagone::new(rayon).index(q, r)]` est la case `(q, r)`). Sert à poser une configuration à la main.
    ///
    /// # Erreurs
    ///
    /// Jamais de panique, dans cet ordre : `Err(RayonHorsBorne)` (rayon 0 ou `> RAYON_MAX`) ; `Err(ParametreHorsPlage(nom))` ;
    /// `Err(NombreDeCases)` si `cases.len() != 3 R (R + 1) + 1` ; `Err(EtatInvalide(i))` pour la **première** case `i` (ordre
    /// canonique) qui viole (I1), (I2) ou (I3) (une masse négative ; une case rattachée avec `b` ou `d` non nul ; une case non
    /// rattachée sans voisin rattaché, avec `b` ou `c` non nul), les `a` étant ceux de `cases` ; enfin `Err(MasseTropGrande)` si
    /// la somme des `brut` de `b`, `c` et `d` de toutes les cases est `2^56` ou plus.
    pub fn essayer_depuis_cases(rayon: u32, parametres: Parametres, cases: &[Case]) -> Result<Neige, ErreurNeige> {
        if rayon == 0 || rayon > RAYON_MAX {
            return Err(ErreurNeige::RayonHorsBorne);
        }
        parametres.verifier()?;
        let hexagone = Hexagone::new(rayon);
        if cases.len() != hexagone.cases() {
            return Err(ErreurNeige::NombreDeCases);
        }
        let a: Vec<u8> = cases.iter().map(|case| u8::from(case.a)).collect();
        let b: Vec<i64> = cases.iter().map(|case| case.b.0).collect();
        let c: Vec<i64> = cases.iter().map(|case| case.c.0).collect();
        let d: Vec<i64> = cases.iter().map(|case| case.d.0).collect();
        let neige = Neige::assembler(hexagone, parametres, a, b, c, d);
        for i in 0..cases.len() {
            let case = cases[i];
            let negative = case.b.0 < 0 || case.c.0 < 0 || case.d.0 < 0;
            let rattachee_fautive = case.a && (case.b.0 != 0 || case.d.0 != 0);
            let lointaine_fautive = !case.a
                && (case.b.0 != 0 || case.c.0 != 0)
                && !neige.voisins[i].iter().any(|&v| v >= 0 && cases[v as usize].a);
            if negative || rattachee_fautive || lointaine_fautive {
                return Err(ErreurNeige::EtatInvalide(i));
            }
        }
        if neige.masse_totale() >= 1_i128 << 56 {
            return Err(ErreurNeige::MasseTropGrande);
        }
        Ok(neige)
    }

    /// Le rayon `R` de l'hexagone.
    pub fn rayon(&self) -> u32 {
        self.hexagone.rayon()
    }

    /// Les réglages.
    pub fn parametres(&self) -> Parametres {
        self.parametres
    }

    /// Le temps : le nombre de pas faits depuis la construction (0 à la construction).
    pub fn temps(&self) -> u64 {
        self.temps
    }

    /// La case `(q, r)`, ou `None` si `(q, r)` n'est pas dans l'hexagone (jamais de panique, pour tous les `i32`).
    pub fn case(&self, q: i32, r: i32) -> Option<Case> {
        if !self.hexagone.contient(q, r) {
            return None;
        }
        Some(self.case_d_indice(self.hexagone.index(q, r)))
    }

    /// La case d'indice canonique `i`.
    fn case_d_indice(&self, i: usize) -> Case {
        Case { a: self.a[i] == 1, b: Fixe(self.b[i]), c: Fixe(self.c[i]), d: Fixe(self.d[i]) }
    }

    /// Toutes les cases, **dans l'ordre canonique** de [`Hexagone`] (`cases()` valeurs).
    pub fn cases(&self) -> Vec<Case> {
        (0..self.a.len()).map(|i| self.case_d_indice(i)).collect()
    }

    /// Un pas, à la lettre de « Un pas, à la lettre » : l'état est remplacé, le temps augmente de 1. Ne panique jamais (voir
    /// « Ce qui est conservé, et pourquoi rien ne déborde »).
    pub fn pas(&mut self) {
        let cases = self.a.len();
        let p = self.parametres;

        // Bloc 0 : n(x), le nombre de voisins rattachés, sur les `a` du début du pas.
        for x in 0..cases {
            let mut n = 0_u8;
            for &v in &self.voisins[x] {
                if v >= 0 && self.a[v as usize] == 1 {
                    n += 1;
                }
            }
            self.n[x] = n;
        }

        // Bloc 1 : la diffusion. Lit `d`, écrit `d_suivant`.
        for x in 0..cases {
            if self.a[x] == 0 {
                let mut somme = self.d[x];
                for &v in &self.voisins[x] {
                    somme += if v >= 0 && self.a[v as usize] == 0 { self.d[v as usize] } else { self.d[x] };
                }
                self.d_suivant[x] = Fixe(somme).div_entier(7).0;
            } else {
                self.d_suivant[x] = self.d[x];
            }
        }

        // Bloc 2 : le gel des cases de bord.
        for x in 0..cases {
            if self.a[x] == 0 && self.n[x] >= 1 {
                let g = (p.kappa * Fixe(self.d_suivant[x])).0;
                self.c[x] += g;
                self.b[x] += self.d_suivant[x] - g;
                self.d_suivant[x] = 0;
            }
        }

        // Bloc 3 : le rattachement. Toutes les décisions d'abord, puis les changements.
        self.nouveaux.clear();
        for x in 0..cases {
            if self.a[x] == 0 && self.n[x] >= 1 {
                let n = self.n[x];
                let se_rattache = if n >= 4 {
                    true
                } else if n == 3 {
                    let mut t = self.d_suivant[x];
                    for &v in &self.voisins[x] {
                        if v >= 0 {
                            t += self.d_suivant[v as usize];
                        }
                    }
                    self.b[x] >= Fixe::UN.0 || (t < p.theta.0 && self.b[x] >= p.alpha.0)
                } else {
                    self.b[x] >= p.beta.0
                };
                if se_rattache {
                    self.nouveaux.push(x);
                }
            }
        }
        for &x in &self.nouveaux {
            self.a[x] = 1;
            self.c[x] += self.b[x];
            self.b[x] = 0;
        }

        // Bloc 4 : la fonte des cases non rattachées qui portent de la masse de bord ou cristalline.
        for x in 0..cases {
            if self.a[x] == 0 && (self.b[x] != 0 || self.c[x] != 0) {
                let fb = (p.mu * Fixe(self.b[x])).0;
                let fc = (p.gamma * Fixe(self.c[x])).0;
                self.b[x] -= fb;
                self.c[x] -= fc;
                self.d_suivant[x] += fb + fc;
            }
        }

        std::mem::swap(&mut self.d, &mut self.d_suivant);
        self.temps += 1;
    }

    /// `n` pas de suite : `n` appels de [`Neige::pas`].
    pub fn avancer(&mut self, n: u64) {
        for _ in 0..n {
            self.pas();
        }
    }

    /// Le nombre de cases rattachées (`a = 1`).
    pub fn cases_rattachees(&self) -> usize {
        self.a.iter().filter(|&&a| a == 1).count()
    }

    /// La masse totale `M = Σ (b + c + d)` : la somme, en `i128`, des `brut` des trois masses de toutes les cases.
    pub fn masse_totale(&self) -> i128 {
        let somme = |v: &Vec<i64>| v.iter().map(|&m| i128::from(m)).sum::<i128>();
        somme(&self.b) + somme(&self.c) + somme(&self.d)
    }

    /// La masse du cristal : la somme, en `i128`, des `brut` de `c` sur les cases **rattachées**.
    pub fn masse_du_cristal(&self) -> i128 {
        (0..self.a.len()).filter(|&i| self.a[i] == 1).map(|i| i128::from(self.c[i])).sum()
    }

    /// Le rayon du cristal : la plus grande distance hexagonale au centre d'une case rattachée (0 pour le germe seul).
    pub fn rayon_cristal(&self) -> u32 {
        (0..self.a.len())
            .filter(|&i| self.a[i] == 1)
            .map(|i| distance_hex((0, 0), self.hexagone.position(i)))
            .max()
            .unwrap_or(0)
    }

    /// Le champ que dessine le rendu net ([`crate::rendu_hex::rendre`]) : pour chaque case, **dans l'ordre canonique**, le `brut` de `c` si la case est
    /// **rattachée**, 0 sinon (la case sans glace est blanche). C'est la masse cristalline : l'article la dessine en gris (« les gris dans le cristal représentent
    /// `c`, plus foncé pour une valeur plus haute »). Une case rattachée n'a ni `b` ni `d` (I1) : seule sa glace compte.
    pub fn valeurs_du_cristal(&self) -> Vec<i64> {
        (0..self.a.len()).map(|i| if self.a[i] == 1 { self.c[i] } else { 0 }).collect()
    }

    /// L'écriture canonique de l'état, `76 + 25 * cases` octets (voir « L'écriture canonique et l'empreinte »).
    pub fn ecriture_canonique(&self) -> Vec<u8> {
        let mut octets = Vec::with_capacity(76 + 25 * self.a.len());
        octets.extend_from_slice(b"GIVREG01");
        octets.extend_from_slice(&self.hexagone.rayon().to_le_bytes());
        let p = self.parametres;
        for reglage in [p.rho, p.beta, p.alpha, p.theta, p.kappa, p.mu, p.gamma] {
            octets.extend_from_slice(&reglage.0.to_le_bytes());
        }
        octets.extend_from_slice(&self.temps.to_le_bytes());
        for i in 0..self.a.len() {
            octets.push(self.a[i]);
            octets.extend_from_slice(&self.b[i].to_le_bytes());
            octets.extend_from_slice(&self.c[i].to_le_bytes());
            octets.extend_from_slice(&self.d[i].to_le_bytes());
        }
        octets
    }

    /// L'empreinte : le SHA-256 en hexadécimal minuscule (`empreinte::sha256_hex`) de [`Neige::ecriture_canonique`].
    pub fn empreinte(&self) -> String {
        empreinte::sha256_hex(&self.ecriture_canonique())
    }

    /// L'aperçu : l'image RVB carrée de côté `2R + 1`, `3 * (2R + 1)^2` octets (voir « L'aperçu »).
    pub fn apercu(&self) -> Vec<u8> {
        let rayon = self.hexagone.rayon();
        let cote = (2 * rayon + 1) as usize;
        let mut image = vec![0_u8; 3 * cote * cote];
        for i in 0..self.a.len() {
            let (q, r) = self.hexagone.position(i);
            let x = (q + rayon as i32 + r.div_euclid(2)) as usize;
            let y = (r + rayon as i32) as usize;
            let couleur = if self.a[i] == 1 {
                [235, 245, 255]
            } else {
                let niveau = ((i128::from(self.d[i]) * 255) >> 32).min(255) as u8;
                [0, niveau / 2, niveau]
            };
            let debut = 3 * (y * cote + x);
            image[debut..debut + 3].copy_from_slice(&couleur);
        }
        image
    }
}

/// Écrit une ligne du CSV des mesures pour l'état courant.
fn ecrire_mesure(mesures: &mut String, neige: &Neige) {
    let ppm = (2_000_000 * neige.masse_du_cristal() + (1_i128 << 32)) / (1_i128 << 33);
    mesures.push_str(&format!(
        "{},{},{},{}.{:06}\n",
        neige.temps(),
        neige.cases_rattachees(),
        neige.rayon_cristal(),
        ppm / 1_000_000,
        ppm % 1_000_000
    ));
}

/// Lance la directive `neige` d'un scénario (voir « La directive `neige` et la sortie de `lancer` ») : à n'appeler que lorsque
/// `scenario.directives` n'est pas vide et que sa première directive s'appelle `neige` ([`crate::lancer::lancer`] le fait).
///
/// # Erreurs
///
/// `Err(ligne)`, le numéro de ligne de la **première directive fautive dans l'ordre du fichier** : la directive `neige`
/// elle-même si la forme n'est pas un hexagone, si elle n'a pas exactement huit arguments, si `pas` n'est pas un entier décimal
/// de 1 à [`PAS_MAX`] ou si les sept réglages ne se lisent pas par [`Parametres::lire`] ; sinon la ligne de la directive
/// suivante, quelle qu'elle soit (il n'y a qu'une directive en plus de la forme). Rien n'est calculé alors.
pub fn lancer_neige(scenario: &Scenario) -> Result<Sortie, usize> {
    let directive = &scenario.directives[0];
    let ligne = directive.ligne;
    let Forme::Hexagone { rayon } = scenario.forme else {
        return Err(ligne);
    };
    if directive.args.len() != 8 {
        return Err(ligne);
    }
    let pas = directive.entier(0).map_err(|_| ligne)?;
    if !(1..=PAS_MAX).contains(&pas) {
        return Err(ligne);
    }
    let a = &directive.args;
    let parametres = Parametres::lire(&a[1], &a[2], &a[3], &a[4], &a[5], &a[6], &a[7]).ok_or(ligne)?;
    if let Some(autre) = scenario.directives.get(1) {
        return Err(autre.ligne);
    }
    let mut neige = Neige::essayer_new(rayon, parametres).map_err(|_| ligne)?;

    let intervalle = std::cmp::max(1, pas / 20);
    let mut mesures = String::from("temps,rattachees,rayon,masse_cristal\n");
    ecrire_mesure(&mut mesures, &neige);
    for t in 1..=pas {
        neige.pas();
        if t % intervalle == 0 || t == pas {
            ecrire_mesure(&mut mesures, &neige);
        }
    }
    let cote = 2 * rayon + 1;
    Ok(Sortie { empreinte: neige.empreinte(), mesures, apercu: image::ecrire_png(cote, cote, &neige.apercu()) })
}

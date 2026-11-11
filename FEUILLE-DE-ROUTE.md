# Givre Lab — la feuille de route de la V0

Ce document est le plan de la V0. Ses étapes deviennent des issues et des plans de vagues, et il change par PR, comme
le code. Seules l'autonomie de Lydite et ses trois limites sont décidées par l'humain ; le reste est proposé, et se
corrige en route.

## Ce que c'est

Un simulateur libre de croissance cristalline, en Rust, orienté recherche, avec un rendu qu'on prend plaisir à
regarder : l'agrégation limitée par diffusion (DLA, Witten et Sander), la croissance de surface, les flocons de neige à
facettes (le modèle cellulaire de Reiter), l'attachement et le détachement thermiques. La littérature de ce domaine est
riche en figures chiffrées : chaque mécanisme se refait en nombres et se compare à son article.

C'est le second banc d'essai de Lydite, de même forme que Mycelium Lab mais d'un autre domaine : deux bancs montrent si
l'orchestrateur tient d'un projet à l'autre. Ce que le premier a appris s'applique dès le départ ; la leçon principale
est qu'un test faux fait tricher les mains (elles plient le code pour le contenter), d'où le juge des tests ci-dessous.

## Ce que la V0 est, et ce qu'elle n'est pas

- **Elle est** : cinq mécanismes en 2D, sur grille carrée ou hexagonale ; une graine fixe, donc même graine, même état ;
  des mesures en nombres (dimension fractale, exposant de croissance, symétrie, vitesse de croissance) ; des
  références publiées reproduites ; une page web où l'on regarde pousser.
- **Elle n'est pas encore** : la 3D, la DLA hors réseau, la forme d'équilibre de Wulff, le champ de phase. Tout cela
  attend une V0 qui tourne.

## Les principes

1. **Rust sans dépendance** : générateur pseudo-aléatoire, écrivain PNG et empreinte écrits dans le dépôt. La CI reste
   simple et rien n'est à auditer.
2. **Déterminisme strict sur toutes les machines** : même graine, même état et mêmes nombres, sous Linux comme sous
   Windows. Donc aucune fonction de `f64` qui passe par la bibliothèque mathématique du système (`sin`, `cos`, `exp`,
   `ln`, `powf`…) : des tables ou des versions écrites dans le dépôt (les probabilités d'Arrhenius, en nombre fini,
   deviennent une table) ; des entiers ou de la virgule fixe partout où une symétrie doit être exacte ; aucune
   `HashMap` parcourue ; un ordre de mise à jour fixe. La liste exacte est dans `AGENTS.md`.
3. **La preuve, c'est l'état et les nombres** : l'empreinte de l'état par scénario et par graine
   (`references/<scenario>.empreinte`) et des mesures avec une tolérance. Les aperçus (`apercus/`) sont pour l'œil :
   ils sont commités pour se voir dans le diff de la PR, y compris sur un téléphone, et ne sont pas une preuve.
4. **Chaque mécanisme arrive avec sa référence** (article, figure, équation) et sa reproduction en nombres. Une valeur
   publiée qu'on n'atteint pas se dit telle quelle dans `MODELE.md`.
5. **Les tests avant les mains, et un juge des tests** (voir plus bas).

## Les modules

| Module | Rôle |
|---|---|
| `alea` | PCG32, la graine |
| `reseau` | les grilles carrée et hexagonale, leurs voisinages, la géométrie (disque, bande) |
| `math` | les fonctions écrites dans le dépôt : tables d'Arrhenius, racines entières, virgule fixe, logarithme déterministe |
| `dla` | l'agrégation limitée par diffusion : marcheurs, lancement, abandon, collage, réduction du bruit |
| `surface` | la croissance de surface en bande : dépôt aléatoire, avec relaxation, balistique, Eden |
| `flocon` | le modèle de Reiter sur grille hexagonale (vapeur, cellules réceptives, diffusion), sans hasard |
| `cinetique` | attachement et détachement thermiques (modèle solide-sur-solide, Monte-Carlo cinétique) |
| `mesures` | dimension fractale (masse-rayon, rayon de giration, comptage de boîtes), densité, rugosité w(L,t), vitesse radiale, symétrie (ordre 4 ou 6), anisotropie, pointes |
| `scenario`, `empreinte`, `image`, `rendu` | le fichier de scénario ; l'empreinte (SHA-256) sur une écriture canonique ; l'écrivain PNG ; les aperçus |
| binaire `givre` | `givre lancer <scénario> --graine <n> --sortie <dossier>` : l'empreinte, les mesures en CSV, les aperçus |

## Les étapes de la V0

Chaque étape a deux vagues : les tests, puis le code. Sa preuve est
`cargo test && cargo clippy --all-targets -- -D warnings && cargo fmt --check`. Les tolérances sont fixées par le juge
des tests, sur un prototype honnête.

**État au 08/11 : les six étapes sont fusionnées**, chacune avec sa preuve et sa section « Ce que cela n'établit pas » dans `MODELE.md`. Les écarts sont dits là : l'étape 3 retrouve les 240 cases de la table de l'auteur avec son bord circulaire (`Bord::Circulaire`, en option : le bord hexagonal reste le défaut) ; l'étape 4 est un Metropolis à une particule, pas la cinétique de Gilmer et Bennema ; la page de l'étape 5 n'est éprouvée dans un navigateur que par sa forme et un coup d'œil humain.

| Étape | Ce qu'elle fait | Sa preuve | Ce qu'on voit |
|---|---|---|---|
| 0, le socle | `alea`, `reseau`, `math`, `scenario`, `empreinte`, `image` et le binaire qui lance un scénario vide | même graine, même empreinte, sous les deux systèmes ; deux graines, deux empreintes ; PNG relu octet par octet ; les tables de `math` égales à leurs valeurs écrites | la graine seule |
| 1, l'agrégat de Witten et Sander | la DLA sur grille carrée : marcheurs lancés sur un cercle, abandonnés au-delà d'un rayon, collage au premier contact ; les mesures, d'abord sur des images synthétiques | segment → 1, carré plein → 2, fractale de Vicsek → ln 5 / ln 3 ≈ 1,465 ; l'amas : dimension masse-rayon autour de 1,71 sur plusieurs graines, dans la tolérance du juge ; rayon de giration en N^(1/D) | l'amas coloré par ordre d'arrivée, et sa courbe log-log |
| 2, les variantes et les surfaces | (a) probabilité de collage < 1 et réduction du bruit (dendrites à quatre bras) ; (b) croissance de surface en bande : dépôt aléatoire, avec relaxation, balistique, Eden | (a) à N égal, l'amas est plus dense quand le collage baisse ; l'anisotropie (axes contre diagonales) croît avec la réduction du bruit ; (b) exposant de croissance β : 1/2 pour le dépôt aléatoire, environ 1/4 avec relaxation (Edwards-Wilkinson), environ 1/3 pour le balistique et Eden (KPZ), chacun dans sa tolérance mesurée | les dendrites ; les fronts de surface et leurs courbes de rugosité |
| 3, les flocons de Reiter | le modèle local de Reiter sur grille hexagonale (paramètres α, β, γ), en virgule fixe | symétrie d'ordre 6 exacte depuis une graine symétrique ; les classes de formes de la figure de référence (plaques, étoiles, dendrites) retrouvées aux mêmes paramètres ; vitesse radiale | une planche de flocons selon β et γ |
| 4, attachement et détachement | le modèle solide-sur-solide en Monte-Carlo de type Metropolis à une particule (même état d'équilibre que la cinétique de Gilmer et Bennema, pas sa loi de taux) : énergie de liaison, température effective, sursaturation, probabilités en table | la vitesse de croissance selon la sursaturation, pour plusieurs températures : quasi nulle sous un seuil quand la face est lisse (nucléation), linéaire quand elle est rugueuse ; la rugosité qui croît avec la température | la face qui passe de lisse à rugueuse |
| 5, la vitrine | la page web : `wasm32-unknown-unknown` sans dépendance, quelques fonctions exportées, un canvas | la page charge, et l'empreinte d'une graine y est la même qu'en natif | la page elle-même |

La vitrine se publie sur GitHub Pages une fois que l'humain l'a ouvert, son geste, une fois ; ensuite chaque fusion la
republie.

**La suite (décidée par André le 08/11)** : onze étapes, chacune en deux vagues (les tests, puis le code), dans cet ordre : le zinc (Z1), la décharge électrique (D1 à D3), la neige de Gravner et Griffeath en 2D (G1 à G3), le métal par champ de phase en 2D (P1 à P4). La 3D et le manganèse restent hors de cette suite. Une valeur publiée d'un matériau réel est un repère affiché avec la taille d'amas du banc, jamais une preuve gelée ; la règle des nombres d'`AGENTS.md` ne change pas avant P1, qui la rouvre avant la dendrite de métal. **Z1 est faite** : la DLA se mesure aussi par la corrélation `C(r)` (voir `MODELE.md`, « Étape Z1 »). **D1 est faite** : le claquage diélectrique (`η` entier) calcule le champ de Laplace et fait croître un amas en `φ^η` (voir `MODELE.md`, « Étape D1 »). **D2 est faite** : la courbe `D(η)` du claquage (`η` de 0 à 3, 100 sites, une fenêtre faite pour les petits amas) est mesurée et mise à côté de points publiés (Hastings 2001 pour `η = 2` et `3`, le rappel de la DLA pour `η = 1`), sans qu'aucun test les compare ni qu'aucun accord soit revendiqué : seuls l'ordre et la baisse sont établis (voir `MODELE.md`, « Étape D2 »). **D3 est faite** : la page montre le claquage (le modèle « Claquage diélectrique » et un curseur `η` de 0 à 3, des amas de 100 à 400 sites), mesure `D` à côté du repère publié de `η` (« ce n'est pas un accord ») et calcule la courbe `D(η)` un amas à la fois ; la bibliothèque a des entrées qui rendent une erreur au lieu de paniquer (voir `MODELE.md`, « Étape D3 »).

**Après la V0** : la 3D (le modèle mésoscopique de Gravner et Griffeath, la DLA 3D autour de D ≈ 2,5), la DLA hors
réseau, la forme d'équilibre de Wulff, puis peut-être le champ de phase.

## Les tests avant les mains

Chaque étape a deux vagues. La première écrit les signatures publiques de l'étape (des corps qui paniquent) et ses
tests d'acceptation, et chaque test porte `#[cfg_attr(not(feature = "etape-<n>"), ignore)]`. Les fonctionnalités
`etape-0` à `etape-5` et les modules de `src/` sont déclarés d'avance (`Cargo.toml`, `src/lib.rs`, des fichiers de
module presque vides) : une vague de tests ne remplace que des fichiers presque vides ou neufs, jamais un fichier qu'une
étape précédente a fait remplir, et deux vagues n'éditent jamais la même ligne. Elle est fusionnée comme les autres. La
seconde remplit les corps et ajoute `etape-<n>` à `default` (la seule ligne de `Cargo.toml` qu'une vague de code
touche), ce qui lève les `ignore` sans toucher aux tests : ses tâches n'ont pas les fichiers de tests dans leurs
`fichiers`, et Lydite refuse une main qui y touche.

**Le juge des tests** passe avant que les tests soient gelés, parce qu'un test faux ou trop faible fait tricher les
mains, ou les laisse passer à vide. Un second agent, plus fort que les mains, vérifie trois choses :

1. **un code nul échoue** : des corps qui rendent zéro ne passent aucun test de comportement ;
2. **un prototype honnête passe, sur plusieurs graines** : une implémentation jetable, fidèle au modèle, passe tous les
   tests, sur une série de graines et pas sur une seule ;
3. **chaque tolérance est mesurée sur ce prototype, et chaque référence est relue dans son article** : une valeur
   publiée n'entre jamais d'après la mémoire de quelqu'un.

**Deux fichiers par cas, et des étiquettes.** Pour chaque étape, `references/etape-<n>-valeurs.md` donne, avant que le
code existe, la référence de chaque valeur attendue : la valeur, sa source (article, page, figure, tableau, équation,
ou un calcul exact écrit à côté), la tolérance et sa raison, et le critère de réussite. Les résultats de notre code
(`references/*.empreinte`) sont un autre fichier, figé seulement une fois que le cas passe contre sa référence. Le juge
refuse une valeur attendue sans source ou tirée de notre propre code, et on n'élargit jamais une tolérance pour faire
passer un cas. Chaque test est étiqueté `juste` (un calcul exact, un simulateur naïf écrit dans le test, une valeur
publiée) ou `régression` (un résultat figé de notre code), et dit ce qu'il ne couvre pas ; seuls les tests `juste`
sont une preuve. Chaque étape prévoit des cas à solution connue, et finit par une section « Ce que cela n'établit
pas » dans `MODELE.md`.

Une propriété fausse, ou seulement vraie en moyenne, n'entre jamais telle quelle : une propriété moyenne se teste en
moyenne, sur assez de graines, avec une tolérance mesurée. Les fichiers de tests que le juge a éprouvés sont ceux qui
sont fusionnés, à l'octet.

## L'autonomie de Lydite

Lydite mène Givre Lab en autonomie, dans trois limites décidées par l'humain :
1. la preuve et le mur restent : Lydite rejoue lui-même chaque preuve, et les mains travaillent derrière le mur ;
2. ce seul dépôt : un jeton limité à Givre Lab ; les réglages du dépôt (règles, vérifications exigées, auto-fusion,
   Pages) restent à l'humain, et Lydite ne peut pas lever ses propres garde-fous (il ne pousse jamais `.github/`, n'a
   aucun droit d'administration) ;
3. seulement les issues de l'humain ou celles de Lydite : sur un dépôt public, le texte d'un inconnu ne devient jamais
   la consigne d'une main.

Le reste est libre : écrire ses issues et ses plans, choisir l'ordre, enchaîner.

**La fusion, sans une ligne de Lydite** : l'auto-fusion du dépôt est permise, et la règle de `main` exige la CI (Linux
et Windows) sans contournement possible. `auto-fusion.yml` demande l'auto-fusion des seules PR des branches `lydite/`
du dépôt vers `main`, déclenché par `pull_request` (jamais `pull_request_target`) ; GitHub fusionne quand tout est
vert. La même CI refuse qu'une dépendance ou un `.cargo/` entre.

## Les règles que les mains ne lisent pas encore

Une main de Lydite ne reçoit pas encore les consignes du dépôt (`AGENTS.md`, `CLAUDE.md`). Chaque tâche d'un plan redit
donc les règles qui comptent pour elle : le déterminisme (aucune fonction mathématique du système, aucune `HashMap`
parcourue), les tests qu'on ne touche pas, le « sans dépendance », les fichiers qu'elle seule touche. La CI sous Linux
et sous Windows attrape un état qui différerait d'un système à l'autre : l'empreinte de référence n'y serait plus la
même.

## Les références à lire

Chacune se vérifie sur l'article avant qu'un test la fige.

- T. A. Witten et L. M. Sander, *Diffusion-limited aggregation, a kinetic critical phenomenon*, Phys. Rev. Lett. 47,
  1400 (1981). La valeur admise de D en 2D, environ 1,71, vient de grands amas plus tardifs ; l'article de 1981 donne
  une valeur un peu plus basse.
- Réduction du bruit et anisotropie : Tang (1985), Kertész et Vicsek (1986), Nittmann et Stanley (1986).
- Surfaces : Family et Vicsek (1985), Family (1986), Kardar, Parisi et Zhang (1986) ; Barabási et Stanley, *Fractal
  Concepts in Surface Growth* (1995).
- Flocons : C. A. Reiter, *A local cellular model for snow crystal growth*, Chaos, Solitons & Fractals 23 (2005) ;
  Gravner et Griffeath (2008, 2009) pour la suite ; les revues de Libbrecht pour la physique.
- Attachement et détachement : Gilmer et Bennema (1972), Weeks et Gilmer (1979).

## Les gestes de l'humain

Faits, une fois : le dépôt, public, sous MIT ; la règle de `main` et l'auto-fusion ; un jeton limité à lui ; la première
PR (la CI et `auto-fusion.yml`). Pages est ouvert : le dossier `/docs` de la branche `main` est publié (https://aoricloud.github.io/givre-lab/). Plus aucune fusion ensuite ; son œil revient après coup.

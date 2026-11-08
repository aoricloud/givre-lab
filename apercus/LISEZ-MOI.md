# Aperçus de l'étape 0

- `etape-0-graine-1.png` : la graine seule, une image de 128 × 96 pixels de bruit tiré par PCG32 avec la graine 1 (flux 54).
- `etape-0-graine-2.png` : la graine seule, une image de 128 × 96 pixels de bruit tiré par PCG32 avec la graine 2 (flux 54).

Un aperçu est pour l'œil, jamais une preuve : les valeurs attendues sont dans `references/etape-0-valeurs.md` et les
empreintes dans `references/*.empreinte` ; embellir un aperçu ne change aucune référence.

## Aperçus de l'étape 2s

- `etape-2s-fronts.png` : les quatre modèles de croissance de surface côte à côte, dans l'ordre dépôt aléatoire, relaxation, balistique et Eden, sur L = 128 colonnes et 96 couches avec la graine 1, chaque rangée colorée selon la couche où le front l'a atteinte.
- `etape-2s-rugosite.png` : les courbes de rugosité w(t) des quatre modèles en échelle log-log (L = 256, instants 1 à 256 par puissances de 2, w² moyenné sur 8 graines), un panneau et une couleur par modèle, dans le même ordre (bleu pour le dépôt, orange pour la relaxation, vert pour le balistique, violet pour Eden).

Un aperçu est pour l'œil, jamais une preuve : les valeurs attendues et les empreintes sont dans `references/`, et
embellir une image ne change aucune d'entre elles.

## Aperçus de l'étape 1

- `etape-1-amas.png` : l'amas de DLA de la graine 1 (4000 sites, réseau de demi-côté 200), chaque site coloré par son ordre d'arrivée, du blanc pour le germe au rose pour le dernier site.
- `etape-1-loglog.png` : la courbe log-log de la masse dans le rayon autour du germe contre le rayon (rayons 2 à R), avec deux axes, leurs repères et la droite de pente 1,71 en repère pointillé.

Un aperçu est pour l'œil, jamais une preuve : les valeurs attendues et les empreintes sont dans `references/`, et
embellir une image ne change aucune d'entre elles.

## Aperçus de l'étape 3

- `etape-3-planche.png` : neuf flocons de Reiter (α = 1, hexagone de rayon 60, 250 pas) en grille 3 × 3, β = 0,3, 0,45 et 0,6 en colonnes et γ = 0,001, 0,01 et 0,1 en lignes, chacun l'aperçu de `Flocon::apercu`, séparés par un filet.
- `etape-3-plaque.png` : l'aperçu, écrit tel quel, du scénario `scenarios/reiter-plaque.txt` (hexagone de rayon 30, α = 1, β = 0, γ = 0,05, 400 pas).

Un aperçu est pour l'œil, jamais une preuve : les valeurs attendues et les empreintes sont dans `references/`, et
embellir une image ne change aucune d'entre elles.


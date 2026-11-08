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


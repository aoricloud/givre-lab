# Givre Lab : les règles du dépôt

Ce dépôt est mené par Lydite : des mains (des agents de toute sorte) y travaillent sur des issues, chacune sur une
branche `lydite/…`, et GitHub fusionne une PR quand sa CI est verte sous Linux et sous Windows. Ces règles valent pour
toute main. Le plan est dans `FEUILLE-DE-ROUTE.md` ; chaque modèle, ses équations, sa référence et ses limites, dans
`MODELE.md`.

## La preuve

`cargo test && cargo clippy --all-targets -- -D warnings && cargo fmt --check`, verte sous Linux et sous Windows.

## Les règles

1. **Sans dépendance** : rien dans `[dependencies]`, `[dev-dependencies]` ni `[build-dependencies]`. Le générateur
   pseudo-aléatoire, l'écrivain PNG et l'empreinte s'écrivent dans le dépôt.
2. **Déterministe sur toutes les machines, pour l'état comme pour les mesures** : même scénario et même graine, même
   état et mêmes nombres, sous Linux comme sous Windows.
   - sur un `f64`, seuls sont permis `+`, `-`, `*`, `/`, `sqrt`, `abs`, `floor`, `ceil`, `trunc`, `round`, `min`, `max`,
     `clamp`, `copysign`, `to_bits` et `from_bits` ; tout le reste passe par la bibliothèque mathématique du système et
     peut différer d'un système à l'autre : ni `sin`, `cos`, `tan`, `asin`, `acos`, `atan`, `atan2`, `exp`, `exp2`,
     `ln`, `log`, `log2`, `log10`, `powf`, `powi`, `hypot`, `cbrt`, `mul_add`, `sinh`, `cosh`, `tanh`. Ce qu'il faut de
     plus s'écrit dans le module `math` : des tables, ou des versions écrites avec `+ - * /` seulement ;
   - là où une symétrie doit être exacte, des entiers ou de la virgule fixe, jamais des flottants ;
   - aucune `HashMap` ni aucun `HashSet` parcourus : des `Vec` ou des `BTreeMap` ;
   - un ordre de mise à jour fixe, sans horloge ni fil d'exécution dans la simulation.
3. **La preuve, c'est l'état et les nombres** : l'empreinte de l'état simulé, par scénario et par graine
   (`references/<scénario>.empreinte`), et des mesures avec une tolérance. Les aperçus de `apercus/` sont pour l'œil :
   les embellir ne change aucune référence.
4. **Un changement voulu de l'état** met à jour son empreinte de référence dans la même PR, et la PR le dit.
5. **Les tests d'acceptation d'une étape** s'écrivent avant son code, par une vague à eux, et un juge les éprouve avant
   qu'ils soient gelés (`FEUILLE-DE-ROUTE.md`, « Les tests avant les mains »). La vague du code ne touche pas à leurs
   fichiers. Chaque test d'une étape porte `#[cfg_attr(not(feature = "etape-<n>"), ignore)]`.
6. **Un test qui semble faux se dit** dans le compte rendu de la tâche, sans le toucher et sans plier le code pour lui
   plaire : ni cas particulier, ni valeur forcée, ni constante calibrée sur une graine.
7. **Chaque mécanisme arrive avec sa référence publiée** (un article, une figure ou une équation), nommée dans
   `MODELE.md`, et sa reproduction en nombres. Une valeur publiée qu'on n'atteint pas se dit telle quelle dans
   `MODELE.md`.
8. **Rien sous `.github/`** : la CI et la fusion restent à l'humain. **Aucun `.cargo/`** : la CI le refuse, car sa
   configuration peut changer ce que `cargo test` joue.
9. **Le profil `dev` est optimisé** (`opt-level = 3` dans `Cargo.toml`) pour que les tests de simulation tournent vite :
   ne pas le changer, ni `[lints]`, ni `rustfmt.toml`, ni `rust-toolchain.toml`, hors d'une tâche qui le demande.

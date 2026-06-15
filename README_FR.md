# Luau Engine
## Qu'est-ce que ce projet ?

Luau Engine est un moteur de jeu optimisé conçu pour simplifier la création de jeux vidéo en utilisant le langage Luau. Développé en Rust et s'appuyant sur la puissance du moteur Bevy ainsi que sur le moteur physique Rapier3D, il vise à offrir des performances élevées tout en proposant une API intuitive.

L'objectif principal de ce projet est de reprendre les concepts familiers et l'architecture de l'API de Roblox (comme la manipulation des instances `Part`, le système de `Camera`, ou encore l'utilisation de signaux via `RunService` et `UserInputService`). Cela permet aux développeurs de retrouver un environnement de développement connu, mais de manière totalement indépendante de la sphère Roblox, évitant ainsi les inconvénients et les limitations liés à cette plateforme. À terme, le projet prévoit également le développement de son propre éditeur visuel.
## Comment l'utiliser ?

Pour utiliser le moteur Luau Engine, deux options s'offrent à vous :

- Compiler depuis les sources : Si vous disposez de l'écosystème Rust installé sur votre machine, vous pouvez compiler le projet manuellement en utilisant `rustc` et le gestionnaire de paquets `cargo`.

- Utiliser une Release : Vous pouvez utiliser directement un exécutable pré-compilé disponible dans les releases du projet.

### Configuration de votre jeu :

Une fois que vous avez l'exécutable du moteur, vous devez préparer l'arborescence de votre projet pour exécuter votre code :

1. Créez un dossier nommé `scripts` à la racine (au même niveau que l'exécutable).

2. À l'intérieur de ce dossier `scripts`, créez un fichier nommé `startup.luau`.

3. Ce fichier sert de point d'entrée ; il sera automatiquement lu et exécuté par le moteur lors de son lancement.

## Comment contribuer ?

Les contributions sont grandement appréciées pour faire évoluer Luau Engine ! Le projet est architecturé de manière modulaire autour de plusieurs "crates" Rust (`engine_core`, `luau_classes`, `luau_runtime`, et `services`), ce qui facilite l'ajout de nouvelles fonctionnalités.

Voici comment vous pouvez aider :

- Ajout de nouvelles classes ou services : Vous pouvez implémenter de nouvelles classes Luau (dans le crate luau_classes) ou de nouveaux services globaux (dans le crate services) pour enrichir l'API disponible pour les `scripts`.

- Améliorations du moteur : Optimisation du système de files d'attente (`EngineQueue`) entre l'environnement Luau et le monde ECS de Bevy, ajouts liés au moteur physique, ou corrections de bugs.

- Processus de contribution : N'hésitez pas à forker le dépôt, créer une branche dédiée à votre ajout, puis soumettre une Pull Request. Vous pouvez également ouvrir des Issues pour discuter de nouvelles idées, signaler des problèmes, ou participer aux réflexions sur la conception du futur éditeur de jeu.

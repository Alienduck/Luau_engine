| **Module**                | **Responsabilité Technique**                                              | **Équivalent Roblox Studio** |
| ------------------------- | ------------------------------------------------------------------------- | ---------------------------- |
| **Interface Utilisateur** | Rendu des panneaux, gestion des fenêtres ancrables via `bevy_egui`.       | Studio UI (Panels)           |
| **Gestionnaire de Scène** | Sérialisation/Désérialisation des entités Bevy et de leurs composants.    | `rbxl` / `rbxmx` format      |
| **Moteur de Réflexion**   | Inspection et modification des composants en direct via `bevy_reflect`.   | Properties Window            |
| **Pont Luau (mlua)**      | Injection des API Bevy dans l'environnement Luau, capture de `stdout`.    | Script Context / Output      |
| **Système de Sélection**  | Raycasting depuis la caméra de l'éditeur pour sélectionner les maillages. | Viewport Selection           |


## MVP des Fonctionnalités

Pour valider le comportement du moteur et de l'éditeur, voici les fonctionnalités indispensables à intégrer dans la première itération :

- [ ] **Séparation des États :** Implémentation d'un `AppState::Editor` et d'un `AppState::Play` pour isoler la logique de jeu de la logique d'édition.
- [ ] **Viewport 3D Basique :** Une caméra de navigation libre (orbite, panoramique, zoom) active uniquement dans l'état éditeur.
- [ ] **Outliner (Explorer) :** Une liste hiérarchique lisant la structure parent/enfant des entités Bevy via `bevy_egui`.
- [ ] **Inspecteur de Composants :** Un panneau affichant les composants `Transform` et les scripts Luau attachés à l'entité sélectionnée.
- [ ] **Exécution de Scripts Luau :** Un bouton "Play" qui instancie l'environnement `mlua`, compile le bytecode Luau, l'attache aux entités, et lance la boucle de simulation.


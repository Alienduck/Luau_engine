# Architecture du Luau Engine

Ce moteur de jeu repose sur une architecture hybride combinant l'Entity Component System (ECS) de **Bevy** en Rust et le moteur de script **Luau**. L'objectif est d'offrir une API de scripting proche des standards de l'industrie (comme Roblox) tout en conservant les performances du bas niveau.

## 1. Flux d'Exécution et Pont Rust-Luau

Pour respecter les contraintes d'emprunt (borrowing) de Rust et la séparation des espaces mémoire, le moteur utilise une architecture unidirectionnelle basée sur une file d'attente (Queue).

- **`EngineQueue` & `EngineCommand` :** Les scripts Luau ne modifient jamais le `World` Bevy directement. Lorsqu'une propriété est modifiée en Luau (ex: `Part.Position = ...`), la classe Luau envoie un `EngineCommand` (ex: `SetTranslation`) dans l'`EngineQueue`.
- **`process_engine_queue` :** À chaque frame de la boucle Bevy, ce système exclusif draine la file d'attente et applique toutes les mutations mises en cache directement sur les composants ECS concernés.

## 2. Organisation des Crates (Workspaces)

Le projet est divisé en plusieurs crates pour isoler les responsabilités et éviter les dépendances circulaires :

### `engine_core`

C'est le socle de données du moteur.

- Contient l'initialisation de base de l'application Bevy (`EngineApp`).
- Définit les composants ECS purs sans logique (ex: `LuauBloom`, `LuauAtmosphere`, `LuauCharacterController`).
- Gère les ressources globales comme le mappage des inputs (`ActionMap`) et les groupes de collision (`PhysicsCollisionGroups`).

### `luau_runtime`

Le cœur de l'intégration du langage.

- **VM & Scheduler :** Instancie la machine virtuelle Luau (`LuaVm`) et gère les threads et les délais via le `LuaScheduler`.
- **Bridge :** Gère le `HandleMap` (qui relie un identifiant `u64` Luau à une `Entity` Bevy) et implémente la file de commandes `EngineQueue`.

### `luau_classes`

L'API exposée aux scripts.

- Implémente le trait `UserData` de `mlua` pour toutes les structures du moteur.
- **Instances :** Fournit les classes manipulables en script telles que `Part`, `MeshPart`, `Model`, `Camera`, `Workspace`.
- **Physique & UI :** Expose les objets `Collider`, `Rigidbody`, `CharacterController`, ainsi que les éléments d'interface (`Frame`, `TextLabel`, `ImageButton`, etc.).
- **Types primitifs :** Implémente la logique mathématique pour `Vector2`, `Vector3`, `CFrame`, `Color3`, `UDim2`, etc.

### `services`

Les systèmes Bevy qui synchronisent le moteur avec les singletons Luau.

- **`RunService` :** Déclenche les événements de boucle principale comme `RenderStepped`.
- **`UserInput` :** Récupère les entrées clavier/souris de Bevy et déclenche les signaux `InputBegan` / `InputEnded`.
- **`Lighting` & `TweenService` :** Systèmes appliquant les interpolations et les paramètres d'environnement visuel à chaque frame.

## 3. L'Environnement de Scripting (Luau)

Les scripts se trouvent dans le dossier `scripts/` et sont exécutés par la VM.

- **`startup.luau` :** Le point d'entrée qui orchestre l'initialisation du jeu.
- **API Roblox-like :** Les scripts manipulent les objets via une syntaxe familière, utilisant des signaux (`:Connect()`), la création d'instances (`Part.new()`), et des services globaux (`TweenService:Create(...)`).
- **Gestion d'état :** Un module `reactive.luau` permet de gérer des états réactifs et d'écouter leurs changements via un système de callbacks (`on_change`).
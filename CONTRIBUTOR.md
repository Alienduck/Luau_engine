# Contribuer au Projet

Merci de l'intérêt que vous portez à ce projet ! Pour maintenir un code propre, lisible et une architecture cohérente, nous vous demandons de respecter scrupuleusement les règles suivantes lors de vos contributions.

## 1. Politique d'utilisation de l'Intelligence Artificielle

Nous privilégions la réflexion humaine et la compréhension profonde du code.

- **Assistance autorisée :** L'utilisation de LLM (Large Language Models) pour vous aider, vous débloquer ou optimiser une logique est parfaitement acceptée.
    
- **Génération aveugle interdite :** Le code généré par IA sans réflexion ni relecture est strictement interdit. Vous devez être capable d'expliquer et de justifier chaque ligne de code que vous soumettez.
    

## 2. Règle de la Responsabilité Unique (Pull Requests)

Chaque Pull Request (PR) doit être atomique et ciblée.

- **Une PR = Une modification.** Si votre objectif est de modifier une classe spécifique, ne touchez **qu'à cette seule classe**.
    
- Ne modifiez pas plusieurs scripts ou d'autres fichiers de manière aléatoire ou non liée à l'objectif principal de votre PR. Si vous repérez d'autres problèmes, ouvrez une PR distincte.
    

## 3. Politique des Commentaires

Le code doit être suffisamment clair et expressif par lui-même.

- **Pas de commentaires internes :** Les commentaires explicatifs ou inutiles à l'intérieur du corps des fonctions sont interdits, même pour des algorithmes complexes.
    
- **Documentation uniquement :** Seuls les commentaires de documentation (doc comments, c'est-à-dire `///` en Rust ou `--` de haut niveau en Luau) situés **en dehors et au-dessus** des fonctions, des classes ou des structures sont acceptés.
    

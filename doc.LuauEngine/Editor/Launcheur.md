# Résumé

## Principe

Le launcheur est une petite plateforme d'aide qui sera la pour aider à gérer, lancer des projets.

Il proposera quelques outils en plus.

## Liste features

| Feature name     | Description                                                    |
| ---------------- | -------------------------------------------------------------- |
| Voir projets     | Liste des projets créés et présents dans la machine            |
| Créer projet     | Bouton nouveau projet, renseigner un nom et ouvrir le projet   |
| Supprimer projet | Bouton de suppression derrière menu avec confirmation          |
| Ouvrir un projet | Lancer l'éditeur avec le projet précis (utiliser le .lscn)     |
| Templates        | Créer, Utiliser, Modifier, Supprimer des templates de projets  |
| Assets           | Un onglet pour voir des assets                                 |
| Mettre à jour    | Un bouton pour mettre à jour (utiliser reqwest et self_update) |

# Architecture

Le projet doit avoir une architecture propre, il doit être scalable et être séparé en crates distinctes.

Pour ça on va séparer en plusieurs crates le projet avec chaque crate contenant sa logique.


| Crate name      | Description                                                                                                                                                             |
| --------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `launcher_core` | Uniquement la data, structures de config, la représentation des données.                                                                                                |
| `launcher_ui`   | Utilise `bevy_egui` pour l'affichage, le layout etc, utilise les données de `launcher_core` pour afficher la liste des projets et écoutera les clics pour interactions. |
| `launcher_bin`  | Exécutable final, repose entièrement sur les 2 crates du launcheur et gère la logique la plus haute comme l'instanciation du processeur de l'éditeur ainsi que Bevy     |


# En profondeur

Actuellement on va voir les possibles problèmes que l'on va rencontrer ainsi que les crates qui seront utilisées pour résoudre ces problème.

| Problème       | Solution                                                                                                                                                                                                                        |
| -------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Interface      | La majeure partie de l'interface sera gérée par [`bevy_egui`](https://docs.rs/bevy_egui/latest/bevy_egui/)                                                                                                                      |
| Images/Icônes  | Pour charger des assets on utilisera [`egui_extras`](https://docs.rs/egui_extras/latest/egui_extras/)                                                                                                                           |
| Configs        | Lors du premier démarrage il faut choisir quel dossier contiendra les projets et enregistrer ce dossier dans un fichier de config, [`directories`](https://docs.rs/directories/latest/directories/) est utile et cross-platform |
| Dossier Projet | Pour choisir le dossier de projet on va utiliser la crate [`rfd`](https://docs.rs/rfd/latest/rfd/) (Rust File Dialog)                                                                                                           |
| Versioning     | Pour savoir si le launcheur est à jour il faut comparer avec les release sur Github, [`reqwest`](https://docs.rs/reqwest/latest/reqwest/) permettra de faire les requêtes.                                                      |
| Mettre à jour  | Pour mettre un jour une app qui est en cours d'exécution c'est compliquer surtout sur Windows, heureusement que la crate [`self_replace`](https://docs.rs/self-replace/latest/self_replace/) existe !                           |
| Sérialisation  | Pour sérialiser les données on va utiliser [`serde`](https://docs.rs/serde/latest/serde/) et [`serde_json`](https://docs.rs/serde-json-core/latest/serde_json_core/)                                                            |

# Personas

## Premier lancement

L'utilisateur lance pour la première fois après installation le launcheur, le launcheur s'ouvre sur une fenêtre spécifique avec un dialogue et un bouton pour ouvrir le `rfd` et sélectionner le dossier qui va contenir les projets.
Après ça le launcheur s'ouvre normalement sur la page de base et l'utilisateur peut créer un premier projet puis l'ouvrir.

## Après création projet

L'utilisateur reviens le lendemain, il avait créé un projet de test, il revient pour supprimer cet ancien projet pour en faire un nouveau plus propre qui sera un vrai projet. Pour ça il clique sur les paramètres de son projet et clique sur **Supprimer le projet** et clique ensuite pour confirmer la suppression, puis il clique pour créer un nouveau projet, renseigne un nom et l'ouvre.

## Mise à jour

L'utilisateur démarre le launcheur, le launcheur affiche une notification toaster avec afficher **Mettre à jour le launcheur** et le launcheur se met automatiquement à jour sans soucis.


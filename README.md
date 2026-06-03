# Luau Engine
## What is this project?

Luau Engine is an optimized game engine designed to simplify video game creation using the Luau language. Developed in Rust and leveraging the power of the Bevy engine as well as the Rapier3D physics engine, it aims to deliver high performance while providing an intuitive API.

The main goal of this project is to recreate the familiar concepts and architecture of the Roblox API (such as the manipulation of `Part` instances, the `Camera` system, or the use of signals via `RunService` and `UserInputService`). This allows developers to work in a familiar development environment, but completely independent of the Roblox ecosystem, thus avoiding the drawbacks and limitations associated with that platform. Eventually, the project also plans to develop its own visual editor.
## How to use it?

To use the Luau Engine, two options are available to you:

- Compile from source: If you have the Rust ecosystem installed on your machine, you can compile the project manually using `rustc` and the `cargo` package manager.

- Use a Release: You can directly use a pre-compiled executable available in the project's releases.

### Game Configuration:

Once you have the engine's executable, you must set up your project's directory structure to run your code:

1. Create a folder named `scripts` in the root directory (at the same level as the executable).

2. Inside this `scripts` folder, create a file named `startup.luau`.

3. This file serves as the entry point; it will be automatically read and executed by the engine upon launch.

## How to contribute?

Contributions are greatly appreciated to help Luau Engine evolve! The project is structured modularly around several Rust "crates" (`engine_core`, `luau_classes`, `luau_runtime`, and `services`), which makes it easy to add new features.

Here is how you can help:

- Adding new classes or services: You can implement new Luau classes (in the luau_classes crate) or new global `services` (in the services crate) to enrich the API available to scripts.

- Engine improvements: Optimizing the queue system (`EngineQueue`) between the Luau environment and Bevy's ECS world, adding physics engine features, or fixing bugs.

- Contribution process: Feel free to fork the repository, create a dedicated branch for your addition, and then submit a Pull Request. You can also open Issues to discuss new ideas, report problems, or participate in brainstorming the design of the future game editor.

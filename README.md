# Heaven AI Arena

A 3D AI agent battle arena built with Bevy Engine, featuring intelligent agents that interact, fight, and strategize in real-time.

## Features

- **3D Agent Battles**: Watch AI agents battle in real-time 3D environments
- **Intelligent Agent Behavior**: Agents find enemies, move strategically, and engage in combat
- **Team-Based Combat**: Agents work together with teammates and fight against enemies
- **Diverse Agent Types**: Warriors, Mages, Archers, Tanks, and Scouts with unique appearances
- **Multiple Arena Themes**: Forest, Desert, Ice, Volcano, Cyberpunk, and more
- **Custom Agent Creation**: Create agents with custom stats, roles, and positions
- **Real-time Combat System**: Agents deal damage, lose health, and respawn automatically
- **Dynamic AI Decision Making**: Agents adapt their behavior based on health and nearby entities

## Quick Start

### Prerequisites

- **Rust** (1.70+): Install from [rustup.rs](https://rustup.rs/)

### Installation and Running

1. **Clone the repository**:
   ```bash
   git clone https://github.com/WaiperOK/Heaven.git
   cd Heaven
   ```

2. **Run the application**:
   ```bash
   cd bevy_viewer
   cargo run --release
   ```

That's it! The arena will start with 3 agents that will immediately begin interacting and fighting.

## What You'll See

When you run the application, you'll see:

1. **3 Initial Agents**: Red Gladiator Alpha, Blue Warrior Beta, and Red Scout Gamma
2. **Real-time Combat**: Agents will find enemies and engage in combat
3. **Strategic Behavior**: Agents will hunt enemies, regroup with allies, and defend when low on health
4. **Console Messages**: Live updates showing agent decisions and combat results
5. **Health System**: Agents lose health when attacked and respawn when defeated

## Agent Behavior

Agents demonstrate intelligent behavior:

- **High Health (>70%)**: Aggressive, seeks out enemies to attack
- **Medium Health (30-70%)**: Cautious, defends position or moves carefully
- **Low Health (<30%)**: Defensive, seeks allies or takes defensive stance
- **Team Coordination**: Agents distinguish between allies and enemies
- **Combat**: Agents deal damage to enemies within range and can be defeated

## Controls

- **Mouse**: Look around the arena
- **WASD**: Move camera
- **F1**: Time simulation panel
- **F2**: Agent chat panel
- **F3**: Ollama server monitor
- **Quick Actions Panel**: Add random agents or create custom agents

## Project Structure

```
Heaven/
├── bevy_viewer/          # Main 3D application
│   ├── src/
│   │   └── main.rs      # Complete application code
│   └── Cargo.toml       # Dependencies
├── arena_core/          # Additional arena logic
├── scripts/             # Helper scripts
└── README.md           # This file
```

## Customization

### Adding New Agents

1. Use the "Add Random Agent" button for quick agent creation
2. Use "Create Custom Agent" for detailed configuration
3. Agents will immediately start interacting with existing agents

### Arena Themes

Change the visual environment using the Arena Theme selector:
- **Default**: Standard arena
- **Forest**: Natural woodland environment
- **Desert**: Sandy dunes setting
- **Ice**: Frozen landscape
- **Volcano**: Lava and ash environment
- **Cyberpunk**: Futuristic metallic setting

## Technical Details

- **Engine**: Bevy 0.12 (Rust)
- **AI**: Custom decision-making system with team-based logic
- **Combat**: Real-time damage system with health tracking
- **Respawn**: Automatic agent respawn after defeat
- **Performance**: Optimized for smooth real-time interaction

## Development

### Building from Source

```bash
git clone https://github.com/WaiperOK/Heaven.git
cd Heaven/bevy_viewer
cargo build --release
```

### Code Structure

The main application is entirely contained in `bevy_viewer/src/main.rs` with:
- **AI Decision System**: Intelligent agent behavior
- **Combat System**: Real-time fighting mechanics
- **Movement System**: Smooth agent movement
- **Respawn System**: Automatic agent revival
- **UI System**: Real-time arena management

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Acknowledgments

- **Bevy Engine**: For the excellent game engine
- **Rust Community**: For the amazing ecosystem

---

**Made with love by [WaiperOK](https://github.com/WaiperOK)**

For questions, issues, or contributions, please visit our [GitHub repository](https://github.com/WaiperOK/Heaven).
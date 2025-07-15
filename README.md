# Heaven AI Arena

A 3D AI agent battle arena built with Bevy Engine, featuring intelligent agents powered by Ollama LLM, diverse arena environments, and real-time visualization.

## Features

- **3D Agent Battles**: Watch AI agents battle in real-time 3D environments
- **Intelligent AI**: Powered by Ollama LLM with custom prompts and decision-making
- **Diverse Agent Types**: Warriors, Mages, Archers, Tanks, and Scouts with unique appearances
- **Multiple Arena Themes**: Forest, Desert, Ice, Volcano, Cyberpunk, and more
- **Custom Agent Creation**: Create agents with custom stats, roles, and AI prompts
- **Real-time Monitoring**: Live logs, agent stats, and battle analysis
- **Port Conflict Resolution**: Automatic Ollama server port management

## Quick Start

### Prerequisites

- **Rust** (1.70+): Install from [rustup.rs](https://rustup.rs/)
- **Ollama**: Install from [ollama.ai](https://ollama.ai/)
- **Git**: For cloning the repository

### Installation

1. **Clone the repository**:
   ```bash
   git clone https://github.com/WaiperOK/Heaven.git
   cd Heaven
   ```

2. **Install Ollama model**:
   ```bash
   ollama pull llama3.2:1b
   ```

3. **Run the application**:
   ```bash
   cd bevy_viewer
   cargo run --release
   ```

## Usage

### Starting the Arena

1. Launch the application
2. The arena will automatically start with 3 predefined agents
3. Ollama server will start automatically (or use existing instance)

### Creating Custom Agents

1. Click "Create Custom Agent" in the Quick Actions panel
2. Configure agent properties:
   - **Name**: Custom agent name
   - **Team**: red, blue, green, yellow, purple
   - **Role**: warrior, mage, archer, tank, scout
   - **Stats**: Health and Energy (1-200)
   - **Position**: Spawn coordinates
   - **AI Prompt**: Custom behavior instructions

### Changing Arena Themes

1. Navigate to "Arena Theme" in the left panel
2. Select from available themes:
   - **Default**: Standard arena
   - **Forest**: Natural woodland environment
   - **Desert**: Sandy dunes setting
   - **Ice**: Frozen landscape
   - **Volcano**: Lava and ash environment
   - **Cyberpunk**: Futuristic metallic setting

### Agent Roles

Each role has unique visual characteristics and capabilities:

- **Warrior**: Metallic armor, balanced stats
- **Archer**: Agile build, ranged focus
- **Mage**: Tall and thin, magical glow
- **Tank**: Heavy armor, high defense
- **Scout**: Light and fast, high mobility

## Technical Details

### Architecture

- **Engine**: Bevy 0.12 (Rust)
- **AI**: Ollama LLM integration
- **UI**: egui for immediate mode GUI
- **Graphics**: PBR rendering with dynamic lighting
- **Networking**: HTTP client for LLM communication

### Project Structure

```
Heaven/
├── bevy_viewer/          # Main 3D application
│   ├── src/
│   │   └── main.rs      # Core application logic
│   ├── Cargo.toml       # Rust dependencies
│   └── assets/          # 3D models and textures
├── arena_core/          # Core arena logic
├── llm_service/         # LLM service integration
├── web_ui/             # Web interface
└── README.md           # This file
```

### Key Components

- **ArenaTheme**: Visual theme system with materials and lighting
- **AgentCreator**: Custom agent creation interface
- **OllamaConnection**: LLM integration with port management
- **Agent Types**: Role-based agent system with visual diversity
- **Real-time Logging**: Comprehensive event tracking

## Customization

### Adding New Themes

1. Implement new theme in `ArenaTheme::new_theme()`
2. Add to `get_available_themes()` list
3. Configure colors, materials, and lighting

### Creating New Agent Roles

1. Add role to `AgentCreator::get_available_roles()`
2. Implement role-specific appearance in `create_diverse_agent()`
3. Configure unique materials and dimensions

### Custom AI Prompts

Each agent can have custom AI behavior through prompts:
- Define agent personality and goals
- Set tactical preferences
- Configure team cooperation strategies

## Configuration

### Ollama Setup

The application automatically manages Ollama server instances:
- Default port: 11434
- Automatic port conflict resolution
- Model: llama3.2:1b (configurable)

### Performance Settings

- **Resolution**: 1920x1080 (configurable)
- **Shadows**: Enabled by default
- **Agent Limit**: Scalable based on system performance

## Troubleshooting

### Common Issues

1. **Ollama Connection Failed**:
   - Ensure Ollama is installed
   - Check if port 11434 is available
   - Verify model is downloaded

2. **Low Performance**:
   - Reduce number of agents
   - Disable shadows in lighting
   - Use lower resolution

3. **Agent Creation Issues**:
   - Verify spawn position is within arena bounds
   - Check agent name uniqueness
   - Ensure team and role are valid

### Debug Information

Access debug panels via:
- **F1**: Time simulation
- **F2**: Agent chat
- **F3**: Ollama server monitor

## Contributing

Contributions are welcome! Please:

1. Fork the repository
2. Create a feature branch
3. Commit your changes
4. Push to the branch
5. Create a Pull Request

### Development Setup

```bash
# Clone and navigate
git clone https://github.com/WaiperOK/Heaven.git
cd Heaven/bevy_viewer

# Install dependencies
cargo build

# Run in development mode
cargo run

# Run tests
cargo test
```

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Acknowledgments

- **Bevy Engine**: For the excellent game engine
- **Ollama**: For local LLM capabilities
- **egui**: For the immediate mode GUI
- **Rust Community**: For the amazing ecosystem

## Future Plans

- **Multiplayer Support**: Network-based agent battles
- **Advanced AI**: More sophisticated decision-making
- **Tournament Mode**: Automated agent competitions
- **Plugin System**: Extensible architecture
- **VR Support**: Immersive 3D experience

---

**Made with love by [WaiperOK](https://github.com/WaiperOK)**

For questions, issues, or contributions, please visit our [GitHub repository](https://github.com/WaiperOK/Heaven).
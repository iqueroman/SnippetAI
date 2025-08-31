# SnippetAI

A powerful desktop application that transforms your clipboard into an AI analysis tool. SnippetAI automatically detects screenshots and text copied to your clipboard and analyzes them using state-of-the-art AI models via the OpenRouter API.

## ✨ Features

### 📸 **Screenshot Analysis**
- **Automatic Detection**: Monitors your clipboard for screenshots taken with Windows' built-in snipping tool (`Win + Shift + S`)
- **Instant Analysis**: AI analyzes your screenshots immediately upon detection
- **Visual Context**: Perfect for analyzing UI elements, diagrams, code snippets, or any visual content

### 📝 **Text Analysis** 
- **Smart Text Detection**: Automatically detects when you copy text (`Ctrl + C`) from any application
- **Intelligent Processing**: Filters out short text snippets (< 5 characters) to avoid noise
- **Universal Compatibility**: Works with text from any application - browsers, documents, IDEs, etc.

### 🤖 **AI Model Integration**
- **OpenRouter API**: Access to dozens of cutting-edge AI models including GPT-4, Claude, Gemini, and more
- **Model Selection**: Choose the perfect model for your specific use case
- **Cost Optimization**: Pick between speed, accuracy, and cost based on your needs

### 🎨 **Modern Interface**
- **Responsive Design**: Seamlessly adapts from 400px to ultra-wide displays
- **Clean Chat Interface**: Intuitive conversation flow with your AI assistant
- **Real-time Feedback**: Live typing indicators and error handling
- **Dark Glass Design**: Modern, professional appearance

### 🚀 **True Portability**
- **Single Executable**: No installation required - just run the .exe file
- **Zero Dependencies**: Everything needed is bundled into one portable file
- **Windows Optimized**: Deep integration with Windows clipboard API for reliable performance

### ⚙️ **Advanced Configuration**
- **Custom System Prompts**: Tailor the AI's behavior for specific workflows
- **Model & Prompt Flexibility**: Easy switching between different AI models and change your system-prompt mid-conversation
- **Persistent Settings**: Your configuration is saved and restored automatically

## 🚀 Getting Started

### Quick Start (End Users)

1. **Download**: Get the latest `snippet-ai.exe` from the releases
2. **Run**: Double-click the executable (no installation needed)
3. **Configure**: Click the settings gear and enter your [OpenRouter API key](https://openrouter.ai/)
4. **Select Model**: Choose your preferred AI model from the dropdown
5. **Start Using**: Copy text with `Ctrl + C` or take screenshots with `Win + Shift + S`

### Development Setup

#### Prerequisites
- [Node.js](https://nodejs.org/) v16+ (for frontend development)
- [Rust](https://rustup.rs/) (latest stable, for Tauri backend)
- [OpenRouter API Key](https://openrouter.ai/) (for AI model access)

#### Installation
1. **Clone the repository**:
   ```bash
   git clone <repository-url>
   cd SnippetAI
   ```

2. **Install dependencies**:
   ```bash
   npm install
   ```

3. **Run in development mode**:
   ```bash
   npm run tauri:dev
   ```

#### Building
Create a portable executable:
```bash
npm run tauri:build
```

The executable will be created at:
```
src-tauri/target/release/snippet-ai.exe
```

## 🎯 Usage Examples

### Screenshot Analysis
1. Press `Win + Shift + S` to open Windows snipping tool
2. Select any area of your screen
3. SnippetAI automatically detects the screenshot and sends it to your selected AI model
4. Get instant analysis, explanations, or answers about what's in the image

**Perfect for:**
- Debugging UI issues
- Analyzing code snippets
- Getting explanations of complex diagrams
- OCR text extraction from images

### Text Analysis
1. Select any text in any application
2. Press `Ctrl + C` to copy
3. SnippetAI automatically detects the text and analyzes it
4. Get summaries, explanations, translations, or any text processing you need

**Perfect for:**
- Code review and explanation
- Document summarization
- Language translation
- Error message debugging

## ⚙️ Configuration Options

### API Settings
- **OpenRouter API Key**: Your personal API key for model access
- **Model Selection**: Choose from 350+ available models including:
  - **GPT-4 Turbo**: Best for complex reasoning and analysis
  - **Claude 3**: Excellent for code analysis and writing
  - **Gemini Pro**: Great for multimodal content
  - **Llama 2**: Cost-effective open-source option

### System Prompts
Customize the AI's behavior with system prompts:
- **Developer Mode**: "You are a senior software engineer. Analyze code and provide technical insights."
- **Learning Assistant**: "Explain concepts clearly and provide examples for better understanding."
- **Creative Helper**: "Think creatively and provide innovative solutions and ideas."

### Interface Options
- **Responsive Layout**: Automatically adjusts to window size
- **Keyboard Shortcuts**: All major functions accessible via keyboard
- **Dark Theme**: Professional appearance optimized for long use

## 🛠️ Technical Architecture

### Frontend Stack
- **Framework**: Svelte 4 for reactive UI components
- **Build Tool**: Vite for fast development and optimized builds
- **Styling**: Modern CSS with glass morphism design
- **State Management**: Reactive Svelte stores

### Backend Stack
- **Runtime**: Tauri (Rust-based) for native performance and security
- **Clipboard Integration**: Direct Windows API access for reliable monitoring
- **Event System**: Async event-driven architecture
- **Memory Management**: Rust's zero-cost abstractions for optimal performance

### Security Features
- **Process Isolation**: Tauri's security model prevents code injection
- **API Key Protection**: Secure storage in localStorage (not transmitted)
- **Minimal Attack Surface**: No network listeners or exposed ports
- **Safe Clipboard Access**: Proper Windows API usage with error handling

## 🔧 Development

### Project Structure
```
SnippetAI/
├── src/                    # Svelte frontend code
│   ├── App.svelte         # Main application component
│   └── main.js            # Frontend entry point
├── src-tauri/             # Rust backend code
│   ├── src/main.rs        # Tauri application and clipboard monitoring
│   ├── tauri.conf.json    # Tauri configuration
│   └── Cargo.toml         # Rust dependencies
├── dist/                  # Build output (auto-generated)
└── public/                # Static assets
```

### Key Components
- **Clipboard Monitor**: Rust-based Windows API integration for real-time clipboard detection
- **Event System**: Async communication between frontend and backend
- **AI Integration**: OpenRouter API client with error handling and retry logic
- **UI State Management**: Reactive state for chat messages, settings, and loading states

### Building and Distribution
The application builds into a single portable executable with:
- **Zero Dependencies**: All libraries statically linked
- **Small Size**: Optimized Rust backend with minimal overhead
- **Fast Startup**: Native performance with sub-second launch times
- **Cross-Version Compatible**: Works on Windows 10 and 11

## 🤝 Contributing

We welcome contributions! Here are some ways you can help:

- **Bug Reports**: Found an issue? Please create a detailed bug report
- **Feature Requests**: Have an idea? We'd love to hear it
- **Code Contributions**: Submit PRs for bug fixes or new features
- **Documentation**: Help improve our documentation and examples

## 📝 License

This project is licensed under the MIT License - see the LICENSE file for details.

## 🙏 Acknowledgments

- [OpenRouter](https://openrouter.ai/) for providing access to multiple AI models
- [Tauri](https://tauri.app/) for the excellent Rust-based app framework
- [Svelte](https://svelte.dev/) for the reactive UI framework
- The open source community for the amazing tools and libraries
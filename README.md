# FireNotes

<div align="center">

![FireNotes Logo](https://img.shields.io/badge/FireNotes-Markdown%20Editor-FF5722?style=for-the-badge&logo=markdown)
![Svelte](https://img.shields.io/badge/Svelte-4-FF3E00?style=flat-square&logo=svelte)
![TypeScript](https://img.shields.io/badge/TypeScript-5-3178C6?style=flat-square&logo=typescript)
![License](https://img.shields.io/badge/License-GPLv3-blue.svg?style=flat-square)

**A modern, distraction-free markdown editor built with Svelte**

✨ Live Preview • 🎨 Syntax Highlighting • 📑 Multi-Document • 🌙 Dark Mode • 💾 Auto-Save

</div>

---

## 📖 About

**FireNotes** is a modern, elegant markdown editor developed to provide a fluid and productive writing experience. It's a fork of [Aire](https://github.com/yourusername/aire) with an organized source structure.

### 🎯 Features

- ✍️ **Creative writing** - Articles, blog posts, technical documentation
- 📝 **Notes** - Quick notes and personal organization
- 📚 **Documentation** - Project documentation, READMEs, manuals
- 💻 **Development** - Technical writing with code support

---

## 🚀 Getting Started

### Prerequisites

| Software | Minimum Version |
|----------|---------------|
| Node.js | v16.0.0+ |
| npm | v8.0.0+ (or yarn/pnpm) |
| Git | Any recent version |

### Installation

```bash
# Clone the repository
git clone https://github.com/yourusername/firenotes.git

# Enter the directory
cd firenotes

# Install dependencies
npm install

# Start the development server
npm run dev
```

### Accessing the Application

After starting, the application will be available at:

```
http://localhost:5173
```

### Production Build

```bash
# Create the optimized build
npm run build

# Preview the build locally
npm run preview
```

---

## 🛠️ Technologies

This project was built with the following technologies:

### Frontend

| Technology | Version | Purpose |
|------------|--------|-----------|
| [Svelte](https://svelte.dev/) | 4.x | Reactive framework |
| [TypeScript](https://www.typescriptlang.org/) | 5.x | Static typing |
| [Vite](https://vitejs.dev/) | 5.x | Bundler and dev server |

### Libraries

| Library | Version | Purpose |
|------------|--------|-----------|
| [Marked](https://marked.js.org/) | 11.x | Markdown Parser |
| [highlight.js](https://highlightjs.org/) | 11.x | Syntax highlighting |
| [lucide-svelte](https://lucide.dev/) | 0.x | SVG icons |
| [KaTeX](https://katex.org/) | 0.x | Math rendering |

---

## 📊 Project Structure

```
firenotes/
├── src/
│   ├── components/       # Svelte components
│   │   ├── EmojiPicker.svelte
│   │   ├── CommandPalette.svelte
│   │   └── SearchPanel.svelte
│   ├── stores/           # Svelte stores (global state)
│   │   └── stores.ts
│   ├── styles/           # Global styles
│   │   └── app.css
│   ├── assets/           # Static assets
│   │   └── svelte.svg
│   ├── lib/              # Library components
│   │   └── Counter.svelte
│   ├── App.svelte        # Root component
│   └── main.ts           # Entry point
├── public/               # Static files
├── index.html            # HTML template
├── package.json
├── svelte.config.js      # Svelte configuration
├── vite.config.ts        # Vite configuration
├── tsconfig.json         # TypeScript configuration
└── README.md
```

---

## 📜 License

This project is licensed under the **GNU General Public License v3.0**.

See the [LICENSE](LICENSE) file for more information.

---

<div align="center">

⭐️ If this project was useful, consider giving it a star!

</div>
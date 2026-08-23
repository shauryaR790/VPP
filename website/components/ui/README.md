# CodeBlock component (React / shadcn)

The **live V++ website** (`website/*.html`) is static HTML on GitHub Pages  -  no React build step. Code blocks there are rendered by `generate_pages.py` + Prism (`css/prism-vpp.css`, `js/main.js`).

These React files are the shadcn-style source for a future Next.js app or docs redesign.

## If you add a React app to this repo

1. **Create Next.js + TypeScript + Tailwind** (from repo root or `website/`):

   ```bash
   npx create-next-app@latest website-app --typescript --tailwind --eslint --app --src-dir
   cd website-app
   npx shadcn@latest init
   ```

2. **Why `/components/ui`?** shadcn CLI installs shared primitives here (`button`, `code-block`, etc.). Keeping this path lets you run `npx shadcn add …` without reconfiguring paths.

3. **Install dependencies**:

   ```bash
   npm install lucide-react react-syntax-highlighter
   npm install -D @types/react-syntax-highlighter
   ```

4. **Copy files** (already in this folder):

   - `code-block.tsx` → `components/ui/code-block.tsx`
   - `code-block-demo.tsx` → use in a page, e.g. `app/demo/page.tsx`

5. **Wire `@/` alias** in `tsconfig.json`:

   ```json
   "paths": { "@/*": ["./src/*"] }
   ```

6. **Update GitHub Pages workflow** to `npm run build &&` deploy the Next export, or keep static HTML as today.

## Live site (no React)

| Feature | Location |
|--------|----------|
| Syntax colors | `website/css/prism-vpp.css` |
| Min 5 lines | `website/generate_pages.py` (`MIN_CODE_LINES`) |
| Copy button + line numbers | `website/js/main.js` (`initDocCodeBlocks`) |
| Regenerate docs | `python website/generate_pages.py` |

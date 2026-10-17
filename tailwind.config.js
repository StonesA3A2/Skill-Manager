// Tokens are RGB channels so opacity modifiers (`bg-surface/70`) compile; with a
// plain `var(--color-x)` Tailwind 3 silently generates nothing for them.
const token = (name) => `rgb(var(--color-${name}-rgb) / <alpha-value>)`;
// Tokens that carry their own transparency (`--color-accent-bg` = accent at 8%).
// Without a modifier they stay the plain variable, exactly as before; a modifier
// scales the built-in alpha, computed here so the CSS needs no calc() in rgb().
const tinted = (name, base, alpha) => ({ opacityValue }) => {
  const scale = Number(opacityValue);
  return opacityValue !== undefined && Number.isFinite(scale)
    ? `rgb(var(--color-${base}-rgb) / ${Number((scale * alpha).toFixed(4))})`
    : `var(--color-${name})`;
};

/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./index.html",
    "./src/**/*.{js,ts,jsx,tsx}",
  ],
  darkMode: 'class',
  theme: {
    extend: {
      colors: {
        background: token('bg'),
        'bg-secondary': token('bg-secondary'),
        surface: token('surface'),
        'surface-hover': token('surface-hover'),
        'surface-active': token('surface-active'),
        border: token('border'),
        'border-subtle': token('border-subtle'),
        'border-faint': token('border-faint'),
        accent: {
          DEFAULT: token('accent'),
          light: token('accent-light'),
          dark: token('accent-dark'),
          bg: tinted('accent-bg', 'accent', 0.08),
          border: tinted('accent-border', 'accent', 0.25),
        },
        danger: {
          DEFAULT: token('danger'),
          bg: tinted('danger-bg', 'danger', 0.08),
        },
      },
      boxShadow: {
        card: 'var(--shadow-card)',
        'card-hover': 'var(--shadow-card-hover)',
      },
      textColor: {
        primary: token('text-primary'),
        secondary: token('text-secondary'),
        tertiary: token('text-tertiary'),
        muted: token('text-muted'),
        faint: token('text-faint'),
      },
      fontFamily: {
        sans: [
          '"SF Pro Text"',
          '"PingFang SC"',
          '"Hiragino Sans GB"',
          '"Noto Sans SC"',
          '"Microsoft YaHei"',
          '-apple-system',
          'BlinkMacSystemFont',
          '"Segoe UI"',
          'system-ui',
          'sans-serif',
        ],
        mono: [
          '"SF Mono"',
          '"Fira Code"',
          '"JetBrains Mono"',
          'Menlo',
          'Monaco',
          'monospace',
        ],
      },
    },
  },
  plugins: [],
}
